# Wave D — pull-streaming SELECT response body (design record)

> 🤖 SPARQ agent — design deliverable for **sq-7d3dj.13** (roadmap item 11 of
> `research/optimization-audit-2026-07.md`). [OPUS-4.8]

**Status: design + decomposition (no production code in this bead).** This record
specifies the "Wave D" pull-streaming HTTP response body — the change that stops
time-to-first-byte (TTFB) and peak response memory from scaling with the *full* result
size — decomposes it into implementable child beads, and pins the client-visible
contract changes and the load-bearing correctness invariant it introduces. It refines
`research/concurrent-serving.md` verdict #5 and §6.6 against the code that has actually
landed since that document was written (the generation ring, the chunk seam, the
deadline-only parallel serializer), and is deliberately scoped so that the pieces
achievable *today* are separated from the pieces that remain blocked on the engine's
streaming-operator program.

No performance numbers are quoted here as claims; where a measurement is load-bearing it
is referenced to its source (the `research/concurrent-serving.md` §4.6 spike, measured
on an M1 work box and framed there as a non-canonical directional bracket) and the
eventual impl win is gated on the canonical `sq-7d3dj.23` TTFB series.

---

## 1. What "Wave D" is, and what already landed

The codebase flags true lazy streaming as future "Wave D" in the `chunked_response`
docstring (`crates/sparq-server/src/http.rs`): *"Today the chunks are fully materialised
strings, so this is belt-and-braces; it becomes load-bearing the moment chunks evaluate
lazily (Wave D push/pull streaming)."* This record is the design that precedes turning
that comment into behaviour.

Three prerequisite pieces have already landed and this design builds directly on them:

- **The generation ring** (`crates/sparq-serve/src/ring.rs`, Wave A1). `PinnedGen =
  Arc<Generation<Graph>>`; `AppState::current()` pins the current generation lock-free;
  the writer publishes forward and never waits for a reader. `live_generations()` counts
  generations alive anywhere (retained by the ring *or* pinned by a reader) — the §6.4
  pressure signal. Retention bound `K = RingConfig::retain` (`DEFAULT_RETAIN = 4`) bounds
  only the ring's *own* references; a reader's `Arc` keeps a generation resident
  regardless of `K`.
- **The chunk seam** (`sparq_engine::query_json_chunks_with_budget` →
  `exec::eval_select_json_chunks`). The engine already returns the SELECT-JSON body as an
  ordered `Vec<String>` whose concatenation is byte-identical to the single-string form,
  flushed at `JSON_CHUNK_BYTES = 64 KiB`. `chunked_response` hands those chunks to hyper
  one at a time via `Body::from_stream`, and moves the `PinnedGen` into the stream
  closure so the snapshot stays pinned until the body finishes.
- **The deadline-only parallel serializer** (`sq-7d3dj.10`, landed #1368). Above
  `PAR_THRESHOLD = 50_000` rows the general path serializes chunks with rayon
  `par_chunks`, then re-orders them so the bytes are identical to the serial path, with a
  mid-serialize deadline re-check. §7 below specifies how streaming composes with this.

**The honest gap.** Today's seam is *memory-bounded*, not *incremental*. Both
`eval_select_json_chunks` code paths **fully evaluate the result before the first byte is
returned**:

- The general path calls `eval_modified(...)`, which materialises the entire `Bindings`
  (`Vec<Row>`), *then* serializes chunks.
- The streaming fast path (`single_pattern_scan_json`) materialises the whole scan range
  (`Scan::rows: Cow<[[Id;3]]>`) before serializing.

So the `Vec<String>` handed to `chunked_response` is complete before the response starts.
Streaming that already-complete `Vec` gives ~zero TTFB win: the `research/concurrent-
serving.md` §4.6 streaming-seam spike confirmed the hypothesis directly — *time-to-first-
chunk ≈ time-to-last*, and over HTTP the first byte lands only after evaluation has
essentially finished. The chunk seam is a real win, but it is a **space** win (it removes
the single giant concatenated `String` copy), not a *time* win.

There are, then, two O(result) residencies in flight for a large SELECT:

1. the id-level result — a borrowed index range for a bare scan (`Cow::Borrowed`,
   zero-copy), or an owned `Bindings` for anything with a join / sort / aggregate /
   projection-with-dedup;
2. the serialized `Vec<String>` (owned JSON text) — held in full before the body starts.

Wave D targets **both TTFB and residency (2)**. Residency (1) — the owned `Bindings` for
non-scan shapes — is a *deeper* win owned elsewhere (§6).

---

## 2. Design overview

Pull-streaming replaces "evaluate-all → collect `Vec<String>` → stream the `Vec`" with
"evaluate-and-serialize incrementally → push each chunk into a bounded channel → hyper
drains the channel". Concretely:

```
  spawn_blocking task (owns PinnedGen, budget guard installed)
    engine produces chunk_0, chunk_1, …  ──►  tx.blocking_send(chunk_i)   (bounded, cap C)
                                                     │  backpressure
  async body:  Body::from_stream(ReceiverStream(rx)) ◄┘  drains, hyper writes to socket
```

Four coupled decisions, each specified below: (§3) the producer seam the engine exposes;
(§4) the transport and `PinnedGen` lifetime; (§5) the client-visible framing contract;
(§6) the mid-stream budget / cancellation semantics and the truncation-safety invariant;
(§7) composition with the landed parallel serializer and admission under slow clients.

The design is **format-scoped to SELECT-JSON and (via `sq-7d3dj.12`) CSV/TSV** — the
row-oriented formats. Turtle / RDF-XML / JSON-LD stay buffered (prefix compaction and
single-document structure make them non-chunkable; this matches the audit's
already-optimised do-not-touch list).

---

## 3. The producer seam (engine)

The engine must **hand chunks to a sink as it produces them** instead of collecting them
into a `Vec`. The minimal, allocation-neutral seam is a *callback* rather than a returned
iterator (an `Iterator<Item = String>` would force the engine to become re-entrant across
its internal parallel/borrowed state; a `FnMut(String) -> ControlFlow` sink does not):

```rust
// sparq-engine (sketch — not final signature)
pub fn query_json_stream_with_budget(
    graph: &Graph,
    sparql: &str,
    budget: &QueryBudget,
    sink: &mut dyn FnMut(String) -> std::ops::ControlFlow<()>,   // Break = consumer gone
) -> Result<(), String>;
```

The sink returns `ControlFlow::Break` when the consumer has hung up (channel closed);
the engine stops promptly (cooperative, like a budget abort). The existing
`eval_select_json_chunks` becomes a thin adapter that pushes into a `Vec` sink, so the
buffered entry point and its byte-identical guarantee are preserved unchanged.

**Three shape tiers, ranked by how much of the win they unlock — this is the honest
scoping boundary:**

- **Tier S1 — already-materialised producer, incremental *serialize*.** For every shape,
  once the id-level rows exist (`Bindings` or the scan slice), serialize-and-push each
  64 KiB chunk instead of collecting. This removes residency (2) (the owned `Vec<String>`
  copy) and shaves the *serialize* phase off TTFB. It does **not** improve the
  evaluate-phase TTFB — for a join/sort/aggregate the first row is not known until
  evaluation finishes, so TTFB stays ≈ evaluation time. **Achievable now**, no
  engine-operator change. This is the bulk of the *memory* win and the honest floor of
  the *TTFB* win.

- **Tier S2 — lazy single-pattern scan.** `single_pattern_scan_json` scans the index in
  order; it can yield the first chunk after the first 64 KiB of matches instead of
  materialising the whole range. This drops TTFB to ≈ first-chunk time for the highest-
  volume export shape (a bare `SELECT ?s ?p ?o WHERE { ?s ?p ?o }`, LIMIT-less dumps).
  Requires making the scan iterate rather than collect its `Cow` — a *bounded, local*
  change to the fast path only. **Achievable now, independently**; it is the single most
  valuable TTFB bead.

- **Tier S3 — lazy general operators.** For join / `ORDER BY` / `GROUP BY` / `DISTINCT`,
  true incremental production needs pull-based operators. Two sub-cases:
  - *Pipeline-breakers* (`ORDER BY` without an index-order match, blocking aggregation,
    hash-`DISTINCT`) are **fundamentally non-incremental**: the operator must see all
    input before the first output row is correct. No streaming design can beat their
    evaluate-phase TTFB; they get only Tier S1.
  - *Pipelineable* joins/filters could stream, but the engine materialises between
    operators today. Lazy pull for these is the **M4 / streaming-pull-operators program
    (epic `sq-pntvh`)** — explicitly *out of scope* here (the audit folds "streaming
    pull-based operators" into `sq-pntvh`). Wave D consumes that seam when it lands; it
    does not build it.

The decomposition (§9) ships S1 + S2 and records S3 as a consumer of `sq-pntvh`.

---

## 4. Transport and `PinnedGen` lifetime

**Bounded channel.** A `tokio::sync::mpsc::channel::<Bytes>(C)` with a small capacity
`C` (e.g. 2–4 buffered chunks ⇒ ≤ `C × 64 KiB` in flight). The blocking engine thread
calls `tx.blocking_send(chunk)`; when the buffer is full it parks the engine thread
(backpressure) — exactly the property that keeps residency at `O(chunk)` regardless of
how slow the client drains. The async side wraps `rx` in
`tokio_stream::wrappers::ReceiverStream` and hands it to `axum::body::Body::from_stream`.

**Why `spawn_blocking` and not the async reactor.** The engine is synchronous CPU work
with a thread-local budget guard (`exec::budget::install`) and thread-local query base /
view scope. It must not run on a reactor worker (it would block the executor and the
thread-locals would not survive an `.await`). It already runs under `spawn_blocking`
today; Wave D keeps that and adds the channel as its output.

**`PinnedGen` lifetime — simpler than the current model, and the key correctness point.**
The `PinnedGen` is owned by the `spawn_blocking` task (which derives `gen.snapshot()`
inside the closure, as `render_select` already does). The engine reads the graph *only*
while producing chunks; every chunk pushed into the channel is **owned** JSON text
(`String`/`Bytes`) that no longer borrows the graph. Therefore:

> The pin need only live as long as the engine is still *reading* the graph — i.e. the
> lifetime of the blocking task — **not** until the client finishes *receiving*. Owned
> chunk bytes decouple graph residency from socket lifetime.

This is strictly better than the current `chunked_response`, which moves the `PinnedGen`
into the async stream closure and thus holds it until the last byte is *written* to the
(possibly slow) client. Under the bounded channel the blocking task is still alive
(parked on `blocking_send`) while the client drains, so the pin is naturally held for
exactly as long as chunks are still being produced — and dropped as soon as the engine
finishes, even if buffered chunks remain to be flushed to the socket.

**But** (§7) that same backpressure means a *slow client* keeps the blocking task — and
therefore the pinned generation — alive for the whole slow-drain duration. That is a new
resource-lifetime property, addressed in §7.

---

## 5. Client-visible contract change: Content-Length → chunked transfer-encoding

This is a **breaking framing change** and must be documented as such.

- **Today:** `chunked_response` sets `Content-Length` (the chunks are fully evaluated, so
  the total length is known before the response starts). The body is streamed but the
  length is declared up front.
- **Wave D (incremental producer):** the total length is *unknown* when the response
  starts (that is the whole point — we start before evaluation finishes). The response
  therefore uses **`Transfer-Encoding: chunked`** and carries **no `Content-Length`**.

Consequences, all client-visible:

- Clients / proxies that rely on `Content-Length` (progress bars, some buffering reverse
  proxies, naive length-prefixed readers) lose it. The SPARQL 1.1 Protocol does **not**
  require `Content-Length` and `sparql-results+json` is defined to stream (head, then the
  bindings array element-wise — `research/concurrent-serving.md` §2.7), so this is
  spec-conformant, but it *is* a change from what sparq emits today.
- Chunked bodies **defeat naive HTTP caches** keyed on length (the WDQS lesson, §2.7).
  This does not affect sparq today (no such cache), but if the response cache
  (concurrent-serving.md §6.3) lands, streamed responses must be keyed/handled
  deliberately, not length-cached.
- HTTP/1.1-only is a deliberate scope choice (audit do-not-touch list); chunked
  transfer-encoding is an HTTP/1.1 mechanism and composes with it directly. No HTTP/2
  work is implied.

**Design decision — make streaming a bounded, opt-in-per-response choice, not a blanket
flip.** Small results should keep the buffered, `Content-Length`-bearing path (streaming
a 300-byte point query is pure overhead and loses the length header for no benefit). The
producer serializes into a buffer and only *switches to* chunked transfer-encoding once
it has produced more than a threshold (e.g. the first `JSON_CHUNK_BYTES` flush) without
finishing. Under the threshold it returns the complete buffer with `Content-Length`
exactly as today. This preserves the current contract for the common case and confines
the framing change to genuinely large results. (This "first-chunk decides framing"
pattern is the standard buffer-then-switch approach; it keeps HEAD responses and small
GETs byte-for-byte unchanged.)

---

## 6. Mid-stream budget / cancellation — the load-bearing invariant

This is the single highest-risk part of the design and the reason the bead is
design-first.

**The irreversibility.** Once the first byte of a `200 OK` streamed body is on the wire,
the status is committed — the server **cannot** retract it into an error. In the buffered
path, a budget abort (deadline / `max_rows`) mid-serialize replaces the *entire* response
with a `4xx/5xx` error (nothing has been sent). In the streaming path, a budget abort
*after* the first byte can only **truncate** the body. This matches every production
engine: QLever and Virtuoso truncate mid-stream (concurrent-serving.md §2.7).

**The invariant (answer-safety / fail-closed):**

> A client MUST NOT be able to mistake a truncated stream for a complete result.

Truncation happens on: budget deadline fired mid-stream, `max_rows` reached mid-stream,
the engine panicked mid-stream, or (§7) the stream was cancelled under snapshot pressure.
Three mechanisms, in order of robustness, satisfy the invariant:

1. **Unterminated JSON (baseline, free).** The complete body ends with the closing
   `]}}`. If the stream truncates, those closing bytes are never written, so the received
   body is **not valid JSON** and any conformant `sparql-results+json` parser errors
   rather than silently accepting a short result. This is the QLever/Virtuoso baseline
   and it is correct-by-construction — a truncated body is syntactically incomplete. It
   is the *floor* guarantee and it holds without any new protocol surface.
2. **HTTP trailer (robust, additive).** Emit an HTTP trailer
   (`Trailer: X-Sparq-Complete`, value `true` only when the last chunk was reached, or an
   `X-Sparq-Truncated: deadline|max-rows|panic|snapshot-pressure` reason on abort).
   Trailers are the clean HTTP/1.1 mechanism for "the outcome is known only after the
   body". Support is uneven across clients, so the trailer is **belt-and-braces on top of
   (1)**, never a substitute for it. sparq's own clients (CLI / JS) can read it.
3. **Never a silent short 200.** The one outcome the design forbids: a well-formed,
   correctly-terminated JSON body that happens to contain fewer rows than the true
   result with a `200` and no signal. Because `max_rows` truncation currently produces a
   *deliberate* error (`QueryBudget` aborts, never silently truncates — see
   `http.rs`/`make_budget` docs), the streaming path must preserve that: hitting
   `max_rows` mid-stream drops the closing `]}}` (invariant 1) and sets the trailer
   reason (2). It must **not** append `]}}` and return a clean-but-short 200.

**Budget checkpoint placement.** The engine calls `budget::check(rows_so_far)` between
chunks (the natural checkpoint — once per 64 KiB). On abort it returns `Err`; the
producer stops pushing, drops the channel *without* writing the terminator, and (if
trailers are wired) the async side attaches the truncation trailer. The already-installed
thread-local budget guard means no new budget plumbing is needed — only the *placement*
of the check moves from "after full serialize" to "between chunks", which the deadline
re-check from `sq-7d3dj.10` already established as sound.

**Client disconnect.** When the client goes away, hyper drops the body → `ReceiverStream`
drops `rx` → the engine's next `tx.blocking_send` returns `Err` (or the sink returns
`ControlFlow::Break`) → the engine stops and the pin drops. No orphaned compute, no
leaked generation. This is the cancellation half of the seam and it is why the sink
returns `ControlFlow` (§3).

---

## 7. Composition with the landed parallel serializer, and admission under slow clients

**Parallel serialize (`sq-7d3dj.10`) vs. streaming.** The landed parallel path collects
`par_chunks` fragments into a `Vec` and re-orders them for byte-identity — it is a
*batch-collect*, structurally opposite to a serial push-as-you-go stream. They do not
compose for free. The design resolves this by *shape*:

- **Tier S1/S3 (already-materialised `Bindings`):** the rows exist, so there is no
  evaluate-phase TTFB to save; the choice is purely serialize strategy. Keep the parallel
  serializer for its throughput, and stream its *output* chunks in completion-order-then-
  reordered batches — i.e. streaming here is the memory win only (residency (2) removed by
  pushing reordered chunks into the channel instead of a `Vec`; TTFB is unchanged because
  the parallel serializer still finishes all fragments before the ordered prefix is
  known). Honestly: for materialised shapes above `PAR_THRESHOLD`, streaming buys peak
  memory, **not** TTFB.
- **Tier S2 (lazy scan):** inherently serial (index order, incremental) — it does **not**
  use the parallel serializer; it pushes each 64 KiB chunk as scanned. This is where the
  TTFB win lives and it is serial by nature. No conflict: the parallel serializer applies
  to the materialised `Bindings` path, the lazy scan to the single-pattern path; they are
  disjoint code paths already.

So the two optimisations are **complementary, not competing**: parallel serialize wins
throughput on materialised results; lazy-scan streaming wins TTFB + memory on the export
shape. The design keeps both, selected by the same shape dispatch that
`eval_select_json_chunks` already performs.

**Admission and the snapshot-pin-duration risk (new — must be bounded).** §4 established
that a slow client keeps the blocking task and thus the pinned generation alive for the
whole drain. The generation ring's `K` bound does **not** cover reader pins (a reader's
`Arc` outlives ring retention — `ring.rs` docs are explicit that stream admission /
cancellation is deferred to waves C/D). Wave D therefore must add the missing bound:

- **Admission on `live_generations()`.** Before starting a *streaming* (chunked) response,
  check the ring pressure signal. If `live_generations()` is at/over a configured
  streaming cap, either (a) refuse the *stream* and fall back to the buffered path
  (bounded memory, no held generation past serialize) with a `Warning` header, or (b)
  shed with `503` + `Retry-After` (concurrent-serving.md §6.4 `Shed(SnapshotPressure)`).
  Preference: **(a) graceful degradation to buffered** — the buffered path is always
  memory-bounded and never pins past serialize, so it is the safe fallback.
- **Per-stream wall-clock / byte-budget cap.** A streaming response that a client has not
  drained within a configurable deadline is truncated (§6 truncation-safety applies) so a
  Slowloris-style slow reader cannot pin a generation indefinitely. This is the
  read-side analogue of the update wall-clock cap (`await_update_worker`).

These two mechanisms are the streaming-specific slice of the wave-C/D admission work the
ring intentionally left as a substrate. They are **prerequisites for enabling streaming
by default** and are beaded as such (§9): S1/S2 can land behind an opt-in flag *without*
them (bounded blast radius — only clients that opt in can pin), but streaming-by-default
must not ship until admission + the pin cap exist.

---

## 8. Feature gating, defaults, and non-goals

- **Opt-in.** Per the sparq opt-in-feature architecture, pull-streaming ships behind a
  cargo feature and/or a server config flag (e.g. `--stream-large-select` /
  `SPARQ_STREAM_THRESHOLD`), OFF by default in the first waves. The default build keeps
  the landed buffered+chunk-seam behaviour byte-for-byte. This confines the framing change
  (§5) and the pin-lifetime change (§4/§7) to operators who opt in, and lets the
  differential guards prove equivalence before the default flips.
- **Non-goals (this program):** durability/WAL; incremental view maintenance; HTTP/2 or
  HTTP/3 (deliberate HTTP/1.1 scope); streaming the graph serialisations
  (Turtle/RDF-XML/JSON-LD — non-chunkable, do-not-touch list); query suspension/
  continuation tokens (SaGe-style pagination is a separate, larger design); building the
  lazy general-operator engine (that is `sq-pntvh` / M4).
- **Rejected alternative — returned `Iterator<Item=String>` from the engine.** Rejected in
  §3: it forces the engine's internal parallel/borrowed state to become re-entrant across
  the iterator boundary and fights the thread-local budget/view guards. The `FnMut` sink
  with `ControlFlow` back-signal is the lower-risk seam and preserves the buffered adapter
  trivially.
- **Rejected alternative — hold the `PinnedGen` in the async stream (as today) under the
  incremental producer.** Rejected in §4: it would extend graph residency to socket
  lifetime for no benefit, since chunk bytes are already owned. Owning the pin on the
  blocking task is both simpler and strictly better for residency.

---

## 9. Decomposition into impl beads

Each child is wired to depend on **`sq-7d3dj.23`** (the TTFB series) — per the architect's
note, *no impl bead may claim a streaming win without the metric that measures it*. The
memory-only beads (S1) additionally depend on `sq-7d3dj.5`'s peak-RSS series (already
landed). Beads are ordered by value × independence.

| Child | Bead | Scope | Tier | Win | Depends on | Risk |
|---|---|---|---|---|---|---|
| **D1 — engine chunk-sink seam** | `sq-7d3dj.24` | Add `query_json_stream_with_budget(graph, q, budget, &mut sink)`; refactor `eval_select_json_chunks` into a `Vec`-sink adapter over it (byte-identical). No behaviour change yet. | S1 substrate | none (enabler) | — | LOW (pure refactor; differential vs the `Vec` form) |
| **D2 — bounded-channel streaming body** | `sq-7d3dj.25` | Server: `spawn_blocking` engine → `mpsc(C)` → `ReceiverStream` → `Body::from_stream`; `PinnedGen` owned by the blocking task; opt-in flag; buffer-then-switch framing (§5). Client-disconnect cancellation (`ControlFlow::Break`). | S1 | peak RSS: residency (2) removed on large SELECT-JSON | D1, `sq-7d3dj.23`, `sq-7d3dj.5` | MEDIUM (framing change, cancellation) |
| **D3 — truncation-safety + trailer** | `sq-7d3dj.26` | Mid-stream budget-abort ⇒ unterminated body (invariant 1) + `X-Sparq-Truncated` trailer (invariant 2); test that a `max_rows`/deadline abort never yields a clean short 200. | S1 | correctness (the load-bearing invariant) | D2 | MEDIUM-HIGH (answer-safety) |
| **D4 — lazy single-pattern scan** | `sq-7d3dj.27` | Make `single_pattern_scan_json` iterate the scan and push per 64 KiB instead of materialising the `Cow` range; TTFB drops to first-chunk on the export shape. | S2 | **TTFB** on bare-scan / dump exports | D1, D2, `sq-7d3dj.23` | MEDIUM (scan iteration; byte-identity guard) |
| **D5 — streaming admission + pin cap** | `sq-7d3dj.28` | Admission on `live_generations()` (graceful fallback to buffered under pressure); per-stream wall-clock/byte cap (truncate a slow reader per D3). Prerequisite for streaming-by-default. | — | resource safety (bounded held generations) | D2, D3 | MEDIUM (new resource control) |
| **D6 (deferred) — lazy general operators** | `sq-7d3dj.29` | Adopt the `sq-pntvh` streaming-pull-operator seam so pipelineable join/filter shapes get evaluate-phase TTFB too. Consumer of M4, not built here. | S3 | TTFB on pipelineable non-scan shapes | `sq-pntvh` (epic — gated textually; bd forbids epic-as-blocker), D1, D2, D3 | HIGH (engine operators) |

`sq-7d3dj.12` (chunked CSV/TSV, already in flight) is the row-oriented sibling of D1–D3:
once the chunk-sink seam (D1) exists, CSV/TSV push per-row through the same transport
(D2) and inherit the same truncation-safety (D3). D1 should land in a shape CSV/TSV can
reuse.

**Recommended landing order:** D1 → D2 → D3 (transport + safety) → D4 (the TTFB headline)
→ D5 (enables default-on) → D6 (waits on `sq-pntvh`).

---

## 10. Measurement and acceptance (for the eventual impl)

- **TTFB:** `sq-7d3dj.23`'s time-to-first-byte series on the large-SELECT scenario —
  request-sent → first response body byte. D4's acceptance is a *canonical* TTFB drop on
  the bare-scan export (EC2 runner; the work-box run only proves plumbing). D2's TTFB is
  expected flat (memory-only) and must be reported as such, not spun as a TTFB win.
- **Peak memory:** `sq-7d3dj.5`'s peak-RSS-while-serving series — D2/D4 must show peak
  drop toward `O(chunk)` on the large-SELECT export; the buffered baseline is the control.
- **Correctness (hard guards):**
  - *Byte-identity* of the completed stream vs. the buffered `query_json_chunks`
    concatenation (the seam's existing invariant — extend the
    `query_json_chunks_concat_is_byte_identical` test to the streamed path).
  - *Truncation-safety* (D3): a forced mid-stream abort (deadline / `max_rows` / injected
    panic) yields a body that fails JSON parsing at the closing brace **and** carries the
    truncation trailer — never a clean short 200. This is the load-bearing test.
  - *Cancellation*: dropping the client mid-stream stops engine compute and drops the pin
    (assert `live_generations()` returns to baseline).
  - *Admission* (D5): under a forced-pressure `live_generations()`, a new stream degrades
    to the buffered path (or sheds), never exceeding the configured held-generation cap.
  - Hardening middleware (`--max-body-bytes`, timeouts, auth) unchanged; small-SELECT and
    HEAD responses byte-for-byte identical (framing unchanged below the threshold).

Canonical numbers are EC2-only (`sq-vw3ax.12` runner). No number in this record or its
impl PRs is a claim on the work box; the deterministic perf-gate and the EC2 series are
the only sources of record.
