# Concurrent request serving: research, measurements, verdicts, design

**Status: research + design (no production code in this wave).**
Spike harnesses: `bench/serve/` (standalone cargo project; see its README).
Measurement machine: **Apple M1 (4 performance + 4 efficiency cores), 16 GB, macOS 26.4.1**,
mimalloc, `--release` + thin LTO, 2026-06-12. Load generator co-located with the server
(steals CPU from it), so every HTTP number below **understates** what this code does on a
dedicated box; relative comparisons and orders of magnitude are the signal, not the
absolute values. Re-run `bench/serve` on target hardware before quoting any number.

Deployment context this document designs for: the **Solid-pod profile** of
`prod-solid-server` (read-only reference at `/Users/jesght/Documents/GitHub/jeswr/prod-solid-server`)
— sparq's original motivation is to sit where QLever sits in that stack. Profile:
high tenancy, access-controlled queries over many small named graphs (one graph per
resource, graph IRI = resource IRI), a **continuous stream of small SPARQL UPDATEs (one
per resource write, ~300–500 bytes, 8–9 triples + a parent containment triple)**, mostly
point-ish reads, occasional large analytical queries. QLever's documented
OOM-under-sustained-updates risk (qlever#2481; also #673, #2174, #2338) shaped that
server's write-path design; **beating that failure mode is a concrete design goal here**
(§4.4 shows we already do).

---

## 1. The throughput envelope, honestly quantified

The user's target is **millions of requests per second** (throughput, single node first;
horizontal scaling is a separate, queued effort). Before any architecture: what physics
allows.

### 1.1 What "millions of req/s" requires

TechEmpower-class context: through Round 22 the *entire top cluster* of plaintext
results plateaued at **~7 M req/s — explicitly 10 GbE network-limited**
(TFB issue #3538); Round 23 (56-core Xeon Gold 6330, 40 Gbps NIC) moved the plateau to
**~30 M req/s**, again NIC/loadgen-bound. Tuned Rust HTTP servers land in that cluster;
realistic per-core figures for *trivial pre-serialized responses* are **~100–500 K
req/s/core**, hardware-dependent. Conclusion before measuring anything of ours:
**Mreq/s is reachable on a single node only for near-zero-allocation, near-zero-parse
responses** — i.e. a result-cache hit that is a hash lookup plus a `writev` of
pre-serialized bytes. Everything else is arithmetic over the workload mix.

### 1.2 Measured per-request costs on this machine (`bench/serve`)

| operation | cost | ceiling implied |
|---|---|---|
| spargebra parse, 106-char point query | **2.24 µs** | ~450 K parses/s/core — any parse-per-request path is sub-Mreq/s/core *before doing work* |
| result-cache hit, `RwLock<HashMap>`, 1 thread | 247 ns | 4.0 M ops/s |
| result-cache hit, `RwLock<HashMap>`, 8 threads | 1 665 ns/op | **4.8 M ops/s — lock collapse** (barely above 1 thread) |
| result-cache hit, sharded(64) RwLock, 8 threads | 643 ns/op | 12.5 M ops/s |
| result-cache hit, **Arc-swapped immutable map**, 8 threads | 336 ns/op | **23.8 M ops/s** |
| cache miss lookup (all-distinct degenerate) | **52 ns** | the full overhead the cache adds when it cannot help |
| cache insert (lower bound) | 161 ns | |
| point BGP SELECT→JSON, in-process, 1 thread | **9.34 µs** | 107 K ops/s |
| point BGP SELECT→JSON, in-process, 8 threads | 8.6 µs/op | **0.93 M ops/s** (M1 4P+4E; near-linear on P-cores) |
| fully-bound ASK, in-process | 10.0 µs | ~100 K ops/s/core |
| analytical: full-scan COUNT w/ CONTAINS filter, warm, rayon-parallel | ~110–220 ms | ~5–9 q/s **using all cores** |
| sparq-server today, HTTP point query, closed loop, 8–16 conns | **22.6–24.8 K req/s** (p50 239 µs) | see §3 for why |

### 1.3 The mix arithmetic

CPU-side cost per request ≈ `h·c_hit + m_point·c_point + m_heavy·c_heavy` where measured
`c_hit ≈ 0.3–0.5 µs` (map) + HTTP framing (~1–2 µs achievable with pre-serialized bodies
answered on the network thread), `c_point ≈ 9–10 µs`, `c_heavy ≈ 10⁵–10⁶ µs`.

- **100 % cache hits**: ~0.3–0.5 M req/s/core CPU-side → **Mreq/s plausible on an
  8-core server box, tens of M on a TFB-class box — NIC-bound before CPU-bound.**
- **90 % hit / 10 % point-miss**: ≈ 1.4 µs CPU/req → ~300 K req/s/core with framing;
  still Mreq/s territory on a real server.
- **1 % analytical at 100 ms**: that 1 % costs 1 ms/req on average → **≤ ~8 K req/s on
  this whole machine no matter what the other 99 % cost.** Analytical queries must be
  rare, budgeted, and isolated; no scheduler makes them free.

**Plain statement of what needs horizontal scaling instead:** any workload whose
*miss stream* alone exceeds ~1 M point-queries/s, anything analytical-heavy, all
network-bound regimes past one NIC, and per-pod working sets past one node's RAM.
Single-node Mreq/s is a **cache-hit-path property**, full stop. The honest pitch is:
fast path at memory speed, executor tier in the high hundreds of K/s for point misses,
analytical tier explicitly budgeted — and generation snapshots + a deterministic update
log that make sharding/replication a later, mechanical step (§6.8).

---

## 2. Literature review

(Searched and compiled 2026-06-12; key sources only, each with the design implication
that survives contact with our measurements. Extended citation trail — including
BSBM/LDBC/WatDiv/IGUANA benchmark specifics used in §7 — in
`research/concurrent-serving-litreview-{A,B,C}-*.md`.)

### 2.1 MQO / work sharing

- **Sellis, ACM TODS 1988** — classical MQO: find common subexpressions in a *batch*;
  NP-hard search; pays only when shared work ≫ per-query work. Follow-ups (Roy SIGMOD
  2000; Zhou SIGMOD 2007) repeatedly confirm planning overhead is the killer.
- **Crescando (VLDB 2009)** — Clock Scan: thousands of queries ride one continuous
  memory scan ("query-data joins"). Property bought: *predictability* (~1–2 s latency
  for **every** query, point or not). Trade: point queries pay full scan latency.
- **SharedDB (PVLDB 2012)** "Killing One Thousand Queries With One Stone" — one global
  always-running shared plan; robust under hundreds of concurrent queries; *not* a
  peak-latency win at low concurrency.
- **IBM Blink / scan sharing (VLDB 2008)** — sharing motivated by *memory bandwidth*
  on concurrent scans; BatchSharing to avoid cache thrash.
- **BatchDB (SIGMOD 2017)** — don't share OLTP/OLAP plans; use workload-specific
  replicas + batched update propagation; isolation beats cleverness.
- **SPARQL MQO: Le et al., ICDE 2012** — NP-hard; cluster batch, evaluate common
  sub-BGPs once, rewrite rest over the cached intermediate. Wins only on batches with
  large shared BGPs; essentially no production adoption; online (non-batch) SPARQL MQO
  evidence: none found.

*Implication:* for a point-query-dominated Solid mix, structural MQO loses: common-
subexpression detection on a 9 µs query costs more than running it (§1.2: parse alone
2.24 µs). What survives: (a) **in-flight identical-request dedup** ("leases", Facebook
memcache NSDI 2013) — the degenerate MQO with ~zero planning cost; (b) **batching the
update stream** (BatchDB-flavoured); (c) shared scans only ever as a future analytical
side-lane, never on the hot path.

### 2.2 Result / sub-plan / semantic caching + invalidation

- **Dar et al., VLDB 1996** semantic caching (regions + remainder queries) — requires
  containment reasoning; for full SPARQL that is expensive-to-undecidable; practical
  systems restrict to conjunctive fragments.
- **Noria (OSDI 2018, Rust)** — cache as partially-stateful incrementally-maintained
  materialized view; reads via the left-right/evmap pattern at millions of reads/s.
  The closest existing blueprint for "memory-speed hits + continuous small updates".
- **Martin et al., ESWC 2010** — SPARQL proxy result cache with *pattern-overlap
  invalidation* (invalidate cached query iff an update's triples could match one of
  its patterns): sound, conservative; their own eval shows the cache **hurts** on
  non-repetitive workloads.
- **Papailiou et al., SIGMOD 2015** — canonically-labeled graph-pattern keys so
  isomorphic queries (different constants/variable names) share cache entries: the
  right key idea for *templated* endpoint workloads.
- **Williams & Weaver, ISWC 2011** — per-query `Last-Modified`/`ETag` from index
  timestamps; lightest standards-aligned design.
- **Facebook memcache (NSDI 2013)** — look-aside, delete-on-write via commit-log
  tailer, **leases** against thundering herds and stale sets.
- **Negative evidence: Wikimedia T126730** — Varnish caching of WDQS results was
  near-useless: exact-string repetition rare, chunked responses defeat HTTP caches,
  hot clients want fresh data. Result caching pays **only** with normalized keys and
  owned invalidation.
- **QLever** caches *operation subtrees* keyed by canonical operator tree — a working
  production sub-plan cache, in an engine optimized for huge analytical queries.

*Implication:* cache **complete serialized response bytes** keyed by
(canonicalized query, **visibility scope**, **generation/epoch**); invalidate by
**per-graph (per-pod) epoch bumps** — Solid updates are naturally scoped to one
resource graph, so invalidation is precise and O(1), avoiding Martin-style global
pattern matching. Leases for stampedes. Semantic/remainder caching: skip. Sub-plan
caching: skip for v1, revisit on telemetry.

### 2.3 Scheduling

- **Schrage 1968**: SRPT minimizes mean response time on a single server, at any load,
  sample-path-wise. Multiserver SRPT asymptotically optimal in heavy traffic
  (Grosof/Scully/Harchol-Balter, PEVA 2018).
- **Starvation**: weaker than folklore at moderate load (Harchol-Balter TOCS 2003 web
  servers: "all can win"), but unbounded postponement of the largest jobs as ρ→1 ⇒
  production policies need **aging/boosts** — the MLFQ mechanism (CTSS 1962; OSTEP ch.8).
- **Robustness to size-estimation error**: Wierman & Nuyens (SIGMETRICS 2008) — SRPT-like
  policies tolerate ~constant-factor size errors with bounded degradation; Dell'Amico
  2014 confirms empirically. **A crude cost estimate is enough.**
- **Morsel-driven parallelism (Leis et al., SIGMOD 2014)**; **Umbra self-tuning query
  scheduling (Wagner et al., SIGMOD 2021)** — lock-free stride scheduler, tasks sized
  ~1 ms, **decaying priorities approximate SRPT while guaranteeing shares** to long
  queries; overhead flat at thousands of concurrent queries. The reference design for
  our executor tier.
- **SEDA (SOSP 2001/USITS 2003)** — bounded queues + adaptive admission control against
  a p90 latency target; shed early, never queue unboundedly.
- **Coordinated omission (Gil Tene)** — closed-loop generators silently drop the samples
  a stall would have produced; open-loop / scheduled-arrival measurement is mandatory.
  `bench/serve/loadgen` charges latency from *scheduled* arrival for this reason.

### 2.4 High-throughput Rust server architecture

- **TechEmpower plateaus**: §1.1. Top entries pre-serialize whole responses, vectored
  writes, per-second date headers.
- **tokio vs thread-per-core**: monoio's vendor benchmarks show ~2–3× tokio on echo
  workloads at 4–16 cores (per-core sharding, io_uring); independent evidence thin, and
  thread-per-core *loses* under skewed per-connection work (no stealing). With a shared
  store and heavily skewed request costs, work-stealing is the safer default; the design
  below is **runtime-agnostic** so this remains swappable.
- **SO_REUSEPORT** sharding: NGINX measured 2–3× accept throughput; Cloudflare's caveat:
  it balances *connections*, not load.
- **Lock-free read paths**: `arc-swap` (load ≈ uncontended-mutex cost, **no degradation
  under reader concurrency** — our cache spike's 23.8 M ops/s Arc-swapped map vs the
  RwLock's 4.8 M collapse is this effect, measured), `left-right`/evmap (reads with ~zero
  sync, 2× memory, slightly stale — fine for epoch caches), `crossbeam-epoch` (general
  EBR when a real concurrent map is needed). Published head-to-heads are scarce; our own
  spike numbers stand in.

### 2.5 Concurrency control

- **SI/MVCC**: readers on a frozen snapshot never block writers; SI admits write skew.
  **SSI** (Cahill SIGMOD 2008; Ports & Grittner VLDB 2012, Postgres 9.1) costs a few
  percent + false aborts. **HyPer MVCC (Neumann SIGMOD 2015)**: near single-version
  speed via undo buffers + precision locking.
- **Calvin (SIGMOD 2012) / Bohm (VLDB 2015)**: sequence transactions first, execute
  deterministically; readers do **zero** concurrency bookkeeping. Collapsed to one node
  ⇒ "single sequenced writer + immutable published snapshots" — serializable by
  construction, no SSI machinery.
- **Scalable commutativity rule (SOSP 2013)**: commuting interface ops admit conflict-
  free implementations. RDF set-semantics inserts/deletes of distinct triples commute;
  updates to different named graphs (pods) commute *except* for shared dictionary/
  containment structures.
- **Group commit**: classical; on NVMe 10–100 K+ commits/s with batching.

### 2.6 Snapshot retention under long-lived readers

- **Postgres**: one long transaction holds the xmin horizon database-wide → bloat;
  production answer: kill/timeout long snapshots (`old_snapshot_threshold`).
- **RocksDB**: iterators pin memtables+SSTs from creation (memory/disk blowups,
  issue #3216, `Iterator::Refresh` as mitigation); snapshots force compaction to keep
  all visible versions.
- **Oxigraph**: RocksDB snapshot per SPARQL query ⇒ a slow analytical query is exactly
  a long-lived pinned snapshot.
- **Steam GC (VLDB 2019, Umbra/HyPer)**: watermark ("global minimum") GC collapses under
  long-running queries; **interval-based** reclamation (retain only versions some active
  snapshot can see) bounds chains. Same shape as our "bounded retained generations".

*Implication:* budget memory as `live_store + K_pinned_generations × generation_delta`,
**cap K, and define behavior at the cap** (§6.4). Never let a stream silently hold the
writer hostage — which is precisely what the current double-buffer does (§4.3, measured
5.4 s writer stall from one held snapshot).

### 2.7 Streaming results

- SPARQL 1.1 Protocol: single request→response; **no cursors/pagination/partial
  results**; nothing forbids chunked bodies; `sparql-results+json` streams naturally
  (head, then bindings array element-wise). Mid-stream errors can only truncate
  (status already committed) — QLever/Virtuoso truncate.
- Implementations: Jena/Fuseki iterator-streaming; Oxigraph lazy solution iterators +
  `sparesults` per-solution serialization; QLever lazy/chunked evaluation + streamed
  export (~2024–25); Tentris ships a dedicated `/stream` endpoint; Comunica et al.
  parse results incrementally client-side. **SaGe (WWW 2019)**: server-side preemption
  + continuation tokens — the principled web-scale pagination design.
- Chunked responses defeat naive HTTP caches (the WDQS lesson); compute ETags from
  (query, scope, generation) instead (Williams & Weaver).

### 2.8 Multi-tenant / access-controlled serving

- **Rizvi et al., SIGMOD 2004** authorization views: if authz is a *view/scope*, the
  cache key needs the scope, not the identity.
- **Hasura's response cache**: key = (role, query, **only the session variables the
  permission plan actually references**) — the canonical defense against cache-key
  fragmentation at high tenancy.
- **Postgres RLS practice**: per-tenant predicates + plan-cache-friendly session
  variables; results above RLS must re-key on tenant.
- Failure modes both ways: key on full identity → hit rate ∝ 1/users; key on too
  little → authorization leaks through the cache (real CDN bug class).

*Implication:* sparq-solid already computes exactly the right object: the **accessible
graph set per (session, mode)** (`AuthIndex::accessible`, cached). Hash that set (or its
cache-entry identity) = the **visibility-scope key**. Public traffic collapses to one
scope; each owner to one scope per pod. ACL changes bump the pod's auth epoch. No
published study covers hit-rate vs tenancy for Solid-like loads — our benchmark plan
measures it (§7).

### 2.9 QLever specifically

Compressed on-disk permutations, value-ids, subtree result cache, lazy evaluation,
delta-triples overlay for live updates (~10 ms per single update; periodic merge) —
structurally the same shape sparq's T17 overlay already has. Robustness is where it
bends: #673 (single queries DoS the endpoint), #2174 (query OOM), #2481 (**UPDATE
OOM — the issue prod-solid-server designs around**), #2338 (crash under consecutive
DELETE WHERE). Lesson: per-request budgets, admission control, and update/query
isolation are first-class requirements, not afterthoughts. §4.4 measures sparq against
the #2481 failure mode directly.

### 2.10 The prod-solid-server integration contract (read from source)

From `decisions/0003-qlever-live-update.md`, `decisions/0001-foundational-architecture.md`,
`src/storage/QLeverIndex.ts`, `src/storage/S3QLeverStore.ts`:

- **One QLever server, two POST endpoints** (env-configured), dispatch by
  `Content-Type: application/sparql-query` vs `application/sparql-update`;
  **`Authorization: Bearer <token>` on updates only**; queries gated by WAC at the
  Solid layer. 45 s request timeout, 3 attempts with linear backoff.
- **Named graph per resource, graph IRI = resource IRI.** Update shapes a drop-in must
  execute: `DROP SILENT GRAPH <r> ; INSERT DATA { GRAPH <r> {…} GRAPH <parent> {…} }`
  (putDocument), `DELETE {…} INSERT {…} WHERE {…}` (putContainer, setAclPointer),
  `DROP SILENT GRAPH <r> ; DELETE DATA { GRAPH <parent> {…} }` (delete). ~300–500 bytes,
  8–9 triples each. Results: `application/sparql-results+json`.
- **The OOM contingency, verbatim-ish**: *"QLever's live-update support … carries an
  OOM-under-load risk (#2481) … Per-write UPDATE is the v1 path. If write throughput
  stresses QLever … the fallback is to batch/queue updates through the same `apply()`
  seam."* — i.e. the customer is already designed to tolerate (and prefer) a server
  that batches; our group-commit writer (§6.5) is contract-compatible.
- Scale decision doc: single instance v1, horizontal scaling explicitly deferred.

---

## 3. Current-state audit (what exists in this repo today)

### 3.1 Request path (`crates/sparq-server`)

`http.rs`: axum; `AppState { graph: Arc<RwLock<Arc<Graph>>>, writer: Mutex<Writer>, … }`.
Every query request: axum extract → `exec::prepare()` **parses with spargebra to
classify the form** → `state.snapshot()` (read-lock + Arc clone, measured **27 ns**) →
`tokio::task::spawn_blocking` → engine entry point **which parses the query again**
→ serialize → respond. Hardening: `concurrency_limit(max_concurrent=32)` + load-shed
429, body limit 413, per-request `QueryBudget` (deadline + max-rows) with a hard
await cap of `timeout + 2 s`.

Costs this implies, against §1.2: **two full SPARQL parses per request** (2 × 2.24 µs
on a 9 µs query), a blocking-pool hop, and a per-request response allocation. Measured
end-to-end: **22.6–24.8 K req/s** on 8–16 keep-alive connections (loadgen co-located).
The per-request architecture, not the engine, is the binding constraint — in-process
the same point query does 107 K/s on one thread.

The **`concurrency_limit` gate is form-blind**: 32 in-flight slots shared by point
queries and 30 s analytical queries alike. Under a flood of expensive queries the gate
fills and *cheap queries get 429s* — admission head-of-line. (At default settings we
measured the softer variant; see §4.5.)

`exec.rs`: parse + classify only; `Prepared.runnable` is the raw string (engine
re-parses). The **engine-seams agent is concurrently adding a pre-parsed-algebra entry
point** — this design assumes it and does not duplicate that work.

`subscriptions.rs` (the **push** seam, T23): SEPA-style WebSocket; a
`tokio::sync::watch<u64>` commit generation bumped after every publish; per-connection
task re-evaluates registered SELECTs against a fresh snapshot and sends added/removed
binding diffs; bursts coalesce via the watch channel; global + per-conn caps. This is a
**complete, working push-streaming substrate** — re-evaluate + diff, not incremental
view maintenance, so its cost is O(query) per commit burst per subscription. Good
enough for v1 push; IVM is explicitly out of scope.

`metrics.rs`: Prometheus counters incl. per-status; `negotiate.rs`/`results.rs`: pure
serializers (JSON fast path id→JSON; XML/CSV/TSV via materialized `QueryResult`).

### 3.2 Engine seams (`crates/sparq-engine`, read-only for this effort)

- **`QueryBudget { deadline, max_rows }`** — cooperative, checked at coarse operator
  boundaries; pure-closure variant for rayon branches. This is the **time-slice
  checkpoint** any scheduler must piggyback on; there is no re-entrant suspend/resume,
  so true preemption is out — what's available is "abort at next checkpoint" and
  "bounded morsel between checkpoints".
- **`query_json_chunks_with_budget`** — returns `Vec<String>` chunks. **Verified
  (spike §4.6): the Vec is fully evaluated before return** — it streams in *space*
  (no second whole-result copy) but not in *time* (TTFB = full evaluation). True pull
  streaming needs an iterator/callback seam in the engine — an engine-seams ask, noted
  in §6.6.
- **Morsel parallelism**: rayon `par_iter` over scan rows / materialization, chunk =
  `len / (threads*4)`, with a row threshold below which everything is serial — point
  queries never touch rayon (this is why one huge query did *not* destroy point-query
  latency in §4.5: the OS preempts rayon workers; cheap queries run on blocking-pool
  threads).
- **`DatasetView<'g>`** — *borrowed* zero-copy visibility restriction:
  `{ base: &Graph, named: Arc<FxHashSet<Term>>, default: DefaultGraphMode }`, installed
  thread-locally (`with_view`), propagated into rayon workers, suspended correctly for
  FROM/FROM NAMED. **Ownership answer: a view does not pin anything itself — the
  `Arc<Graph>` snapshot it borrows from does.** A long-lived stream must therefore hold
  `Arc<Graph>` (the generation) + the `Arc<FxHashSet>` (the scope) for its lifetime.
- **`update_in_place` / `Graph::apply_delta`** — delta overlay: sorted `added` vec +
  `deleted` hash set over an immutable base; insert is binary-search + `Vec::insert`
  (O(overlay) shifts — fine while overlays stay small between compactions);
  `compact()` folds the overlay back, O(graph). `compact_every` (default 1024 batches)
  bounds overlay growth. Cost model measured in §4.3/§4.4.
- **Cost signal**: `store.estimate(&Pattern)` + `pred_stat(p)` (`PredStat { count,
  ndv_subj, ndv_obj }`) — exactly enough for the crude SRPT size estimate the
  scheduling literature says suffices (§2.3, Wierman & Nuyens).

### 3.3 The writer (`http.rs::Writer`) — the central object of this design

Double-buffered: published `Arc<Graph>` + one spare. Update = reclaim spare
(`Arc::try_unwrap`, **polling every 200 µs until the last reader snapshot drops, capped
at `query_timeout + 2 s`**) → replay lag → `update_in_place` (O(batch)) → publish by
pointer swap → old published becomes spare. Failed updates stay atomic (buffer
discarded, next update rebuilds O(graph)). ~2× graph residency steady-state.

Two structural facts that drive §6:

1. **A single long-lived reader snapshot stalls the writer.** The reclaim wait is the
   *second* update after the snapshot was taken; at the deadline the writer pays an
   **O(graph) rebuild** instead. Measured (§4.3): 5.4 s with a 1 s timeout on 1 M
   triples — with the production 30 s timeout that is **a ~32 s write stall + a 2.4 s
   rebuild caused by one held snapshot**. Snapshot-consistent *streaming* (minutes-long
   snapshot lifetimes) is therefore **architecturally incompatible with the
   double-buffer**; it needs a generation ring (§6.4).
2. The writer mutex serializes updates — fine (single-writer is the design), but every
   update publishes a generation: at 1–2 K updates/s the "generation" granularity must
   come from **batching windows**, not per-update, or retained-generation counts and
   cache invalidation churn explode (§6.5).

### 3.4 `crates/sparq-solid` (read-only) — the tenancy/visibility model

Named graph per document; WAC/ACP evaluated by N3 rules (sparq-reason) into the
materialized auth view `<urn:sparq:auth>`; `AuthIndex` reads it into a transient index;
a `Session {agent, client}` expands to ≤6 principals; **`accessible(session, mode)` =
`∪allow ∖ ∪deny`, cached**; enforcement via the zero-copy `DatasetView`. Fail-closed
(no view ⇒ empty visibility).

For serving, this gives us, off the shelf: the **visibility-scope cache key** (identity
of the accessible graph set), the **auth epoch** (the auth view changes only when ACL
documents change → rerun of `materialize_*`), and the per-query enforcement mechanism
(install view, run query) that makes cached results **safe by construction** — we cache
what a scope is allowed to see because the engine evaluated under that scope.

### 3.5 Attachment points identified (library-level, HTTP-agnostic)

| concern | attach to | note |
|---|---|---|
| generations/snapshots | replace `Arc<RwLock<Arc<Graph>>>` + `Writer` with a generation ring in a new crate | §6.4 |
| result cache | in front of `prepare()`/engine dispatch, keyed before parse (raw-string hash first, canonical hash second) | §6.3 |
| scheduler | around the `spawn_blocking` boundary: classify (cost estimate from parsed algebra + `store.estimate`) → lane | §6.2 |
| pull streaming | `query_json_chunks_with_budget` today (space-only); engine iterator seam tomorrow | §6.6 |
| push streaming | `subscriptions.rs` machinery, lifted into the library crate | §6.6 |
| auth scope | `sparq_solid::AuthIndex::accessible` → scope hash + `DatasetView` | §6.3 |
| txn/write path | `update_in_place`/`apply_delta` + group-commit batching | §6.5 |

sparq-server then *consumes* the crate; axum specifics (extractors, response types)
stay in sparq-server. Nothing in the new crate may depend on axum/tokio types in its
core API (tokio adapters behind a feature).

---

## 4. Empirical spikes (measured, this machine; harnesses committed in `bench/serve`)

### 4.1 Result-cache hit-path ceiling (`cache_spike`)

Zipfian (s=1.0) repeats over 10 K keys, 1 KiB pre-serialized responses — §1.2 table.
Headlines: **RwLock collapses under readers (4.8 M ops/s at 8 threads ≈ 1-thread
throughput); an Arc-swapped immutable map does 23.8 M ops/s**; the all-distinct
degenerate workload pays **52 ns lookup + 161 ns insert ≈ 0.2 µs** per request it
cannot help — ~2 % of a point query, ~0.01 % of anything bigger. **Cache overhead in
the degenerate case is negligible; the lock choice is not.**

### 4.2 Point-lookup ceiling (`point_spike`)

§1.2 table. Parse is 24–30 % of a point query's 9.34 µs. 8 threads scale to 0.93 M
ops/s in-process (M1 efficiency cores drag; expect closer to linear×P-cores on server
parts). **The non-cached point path is a ~100 K/s/core business — never an Mreq/s one.**

### 4.3 Snapshot cost + retention semantics (`snapshot_spike`, 1 M triples)

- snapshot (read-lock + Arc clone): **27 ns** — consistency for readers is free.
- first update (O(graph) rebuild + 2nd-buffer materialization): **2.63 s**; RSS
  527 → 741 MB (the documented ~1.4× residency step).
- steady in-place updates: **p50 17 µs**, max 12.3 ms.
- update while a reader holds the previous generation: first update fine (demotion),
  **second update: 5.44 s = 3 s reclaim-wait (1 s timeout + 2 s grace) + 2.4 s O(graph)
  rebuild**. Production timeout (30 s) ⇒ **~32 s stall**. RSS with one pinned
  generation: 789 MB (~+50 MB for the delta of one update at this size; grows with
  divergence).
- after the snapshot drops: p50 back to 40 µs.

**Verdict baked into §6: snapshots are cheap to *take* and ruinous to *hold* under the
current writer.** The generation ring removes the reclaim-wait entirely (writer never
waits for readers; it builds forward), at the price of bounded extra residency.

### 4.4 Sustained small-update stream — the QLever #2481 check (`update_stream_spike`)

20 000 `putDocument`-shaped updates (DELETE WHERE + 9-triple INSERT DATA, ~350 bytes)
round-robin over 2 000 resources, on a 1 M-triple base, `compact_every=1024`:

- throughput **1.1–2.5 K updates/s sustained** (p50 75–90 µs/update); each ~1 K-update
  window contains one ~150–240 ms spike — the periodic O(graph) compaction, exactly as
  documented.
- **RSS: flat-to-falling. 1 002 MB early peak → 327 MB at the end; no growth trend
  whatsoever over 20 K updates.** The overlay+compaction design does **not** exhibit
  the QLever #2481 failure mode (memory climbing under sustained live updates). This
  was the single most important go/no-go measurement for the Solid deployment and it
  passes.
- with 4 hot readers (snapshot-per-query loop, 214 K reads/s sustained alongside):
  updates degrade to **~200–300/s sustained, p50 ~1 ms, p99 20–30 ms** — the 200 µs
  reclaim-poll dance under reader churn. Memory still stable. (The generation ring
  also removes this: no reclaim, no poll.)

prod-solid-server's measured profile is *one* update per user-facing resource write —
hundreds/s would already be a very large deployment. **Current sparq sustains the
Solid write stream with 1–2 orders of magnitude of headroom, with bounded memory.**
The compaction spike (~200 ms every 1 024 updates) is the number to watch at p999.

### 4.5 Server baseline + head-of-line (`loadgen` against `sparq-server`, 1 M triples)

Loadgen co-located (8 worker threads stealing CPU from the server — absolute numbers
are conservative):

- closed-loop point query: **22.6 K req/s** (8 conns; p50 239 µs, p99 2.1 ms);
  24.8 K req/s at 16 conns. **~40× below the in-process 8-thread ceiling** — request
  framing + double parse + blocking-pool hop is the gap (§3.1).
- open-loop 5 K req/s (Poisson, coordinated-omission-safe), 25 s: p50 683 µs,
  **p99 30 ms, p999 77 ms** — at 20 % of saturation the tail is already tens of ms
  (queueing waves through the blocking pool).
- **HoL test A — one huge query**: a 12 s full-scan REGEX query injected twice into the
  5 K req/s point stream: point p50 631 µs / p99 18 ms / p999 72 ms — **no measurable
  degradation**. Why: analytical queries occupy rayon workers; point queries never
  enter rayon (§3.2) and the OS preempts freely. The "one whale blocks the harbor"
  scenario is *already absorbed* by the two de-facto thread pools — an honest negative
  result against the naive HoL narrative.
- **HoL test B — many medium queries**: 56 closed-loop connections of a warm 150 ms
  full-scan CONTAINS query + 2 K req/s point stream: point queries held p50 1.04 ms /
  p99 37 ms / p999 75 ms (modest), **but the medium queries inflated each other to
  p50 735 ms / p90 23.1 s — 150× their solo latency** — and at `max_concurrent=64`
  this mix sat one connection away from shedding the *cheap* traffic with 429s.
  **The real head-of-line problem in this architecture is (a) the form-blind admission
  gate and (b) expensive queries destroying each other**, not cheap-vs-one-big.
- Cold-start artifact worth recording: the *first* full-scan after load took 5.8 s
  (one-time lazy permutation decompression), then 110–220 ms warm. First-touch costs
  belong in deployment runbooks.

### 4.6 Streaming seam (`stream_spike` + HTTP TTFB)

Hypothesis confirmed: `query_json_chunks_with_budget` fully evaluates before returning.
1 M-row full scan: count-only 11.7 ms; **time-to-first-chunk 79.1 ms ≈ time-to-last**
(drain 0 ms); single-string `query_json` 304.7 ms (the chunk path is 3.8× faster
end-to-end by avoiding the giant concat — a real win, but a *space* win). Over HTTP,
500 K rows / 51 MB: **TTFB 146 ms of 215 ms total (68 %)**. LIMIT 100 K: first chunk at
85 % of full evaluation. **Today's seam gives memory-bounded responses, not incremental
delivery; pull-streaming needs an engine iterator/callback seam** (§6.6).

---

## 5. Honest verdicts

Pre-registered hypotheses from the task are marked ✓ (held) / ✗ (falsified by
measurement).

| # | idea | expected win | wins when | loses when | cost | verdict |
|---|------|--------------|-----------|------------|------|---------|
| 1 | **MQO / shared patterns between queued queries** | none on the Solid mix; planning cost ≥ point-query cost (parse alone 2.24 µs vs 9.34 µs total). ✓ hypothesis held: online MQO loses on point mixes | concurrent scan-heavy analytics batches (Crescando/SharedDB regime) | point-dominated mixes; latency-sensitive paths; any batching delay | high (NP-hard batch planning, batch formation latency) | **REJECT** for the hot path. Keep exactly one degenerate form: **in-flight identical-request dedup** (lease/single-flight on the cache key) — near-zero cost, converts stampedes into one execution. Shared-scan analytics lane: future, only if telemetry shows concurrent-scan load |
| 2 | **intermediate-result caching for frequent requests** | result-bytes cache: hit ≈ 0.3–0.5 µs vs 9 µs–seconds; the only route to Mreq/s (§1) | repeated queries (app polling, templated workloads — the Solid profile), read-mostly graphs | all-distinct adversary (measured overhead 52–213 ns/req — negligible); per-update *global* invalidation (must be per-graph epochs); high-tenancy key fragmentation if keyed on identity (must be keyed on **visibility scope** — Hasura lesson) | medium | **RECOMMEND** (response-bytes cache, keys §6.3, leases, per-graph epochs). **Sub-plan/semantic caching: REJECT for v1** — containment reasoning beats re-execution only for expensive sub-plans we don't yet observe; QLever-style subtree cache is a measured-need follow-up |
| 3 | **prioritise cheap queries** | SRPT ≈ optimal mean latency (Schrage); robust to crude estimates (Wierman & Nuyens). ✓ near-universal win *in the executor tier* | mixed-cost contention — measured regime B (§4.5): medium queries inflate 150× | uniform workloads (scheduler is pure overhead — must stay <µs/decision); estimate inversions (cap damage via lanes + aging) | medium | **RECOMMEND-WITH-SCOPE**: two-lane split (cheap/heavy) by estimate from parsed algebra + `store.estimate`, SRPT-approx ordering *within* the heavy lane, MLFQ-style aging against starvation. *Scope honesty:* cheap-vs-one-big is already fine (§4.5 A); the win is medium-vs-medium and tail protection, not the headline scenario |
| 4 | **very expensive queries must not block later arrivals** | gate-shaping, not preemption | form-blind admission gate under expensive floods (the actual measured risk: 429s for cheap traffic) | n/a — this is a correctness-of-service property | low–medium | **RECOMMEND**: per-lane admission (cheap lane never queues behind heavy admission), bounded heavy-lane concurrency (≤ cores/2 rayon occupancy), budget-checkpoint cooperative abort (exists), SEDA-style shed per lane. True preemption/suspend-resume: **REJECT** (engine has no re-entrant operators; cost/benefit far underwater given §4.5 A) |
| 5 | **pull + push snapshot-consistent streaming** | TTFB 146 ms → ~ms for large results (pull); push exists | large results; live dashboards (push) | ✓ hypothesis held: the risk is snapshot retention — **one held snapshot = ~32 s writer stall + O(graph) rebuild today (measured §4.3)** — and slow clients pinning generations under sustained writes | high (engine iterator seam + generation ring) | **RECOMMEND** with the §6.4 generation ring as a *prerequisite*: streams pin generations cheaply; retained generations **bounded at K** with defined behavior at the bound (refuse new streams / cancel oldest with a truncation trailer). Pull = engine iterator seam (engine-seams ask); push = lift `subscriptions.rs` re-eval+diff into the crate |
| 6 | **"intelligent ACID" — reorder updates around disjoint queries** | ✓ hypothesis held: **moot for readers** — SI means a query *never* waits on an update (27 ns snapshot, swap-only contention) regardless of overlap; "reordering a query around an update" is what the architecture already does | the real question is **writer-side**: batching + commutativity | n/a | low (batching) / high (parallel partitioned commit) | **RECOMMEND-WITH-SCOPE, reframed**: single sequenced writer + **group-commit batching window** (§6.5) gives serializability by construction (Calvin/Bohm collapsed to one node), no SSI needed, write skew impossible. Per-pod (named-graph) **conflict tagging** retained for *parallel delta computation* and future sharding — but **independent per-partition commit: REJECT for v1** (shared dictionary + cross-graph containment triples make true disjointness narrower than it looks; the measured writer has 10–100× headroom over the Solid write rate, so the complexity buys nothing today) |

---

## 6. Design proposal: the `sparq-serve` crate (library-first)

### 6.1 Shape and non-negotiables

A new crate `crates/sparq-serve` (engine-level API, **no HTTP types, no async-runtime
types in the core**). Consumers: (a) `sparq-server`'s axum endpoint (thin), (b) a
**QLever-contract-compatible endpoint mode** for prod-solid-server (single POST
endpoint, Content-Type dispatch, Bearer-gated updates, sparql-results+json — §2.10),
(c) bindings (wasm/python) via the same sync core.

```rust
// core, sync, runtime-agnostic (sketch)
pub struct Serve { /* generations, cache, lanes, writer, auth hook */ }
pub struct AuthCtx { pub scope: ScopeKey, pub view: Option<ViewSpec> } // from sparq-solid or none
pub enum Submitted {
    Ready(Bytes),                  // cache hit or fast-tier completion: pre-serialized body
    Stream(StreamHandle),          // pull: pinned generation + chunk iterator
    Queued(Ticket),                // heavy lane: poll/block/callback to completion
    Shed(Reason),                  // bounded-queue refusal: caller maps to 429/503
}
impl Serve {
    pub fn submit(&self, q: QueryInput, auth: &AuthCtx, prio: Priority) -> Submitted;
    pub fn update(&self, u: &str, auth: &UpdateAuth) -> Result<CommitId, UpdateError>;
    pub fn generation(&self) -> GenHandle;          // explicit snapshot API
    pub fn subscribe(&self, q: QueryInput, auth: &AuthCtx) -> SubHandle; // push
}
```

`QueryInput` accepts raw string **or pre-parsed algebra** (the engine-seams agent's
new entry point — we consume it, we do not implement it). Async adapters
(`feature = "tokio"`) wrap `Ticket`/`StreamHandle` in futures/streams; a monoio/glommio
adapter remains possible because the core never assumes a reactor.

### 6.2 Two-tier execution

- **Tier 0 (network/caller thread, lock-free):** raw-string hash → cache probe
  (52 ns) → on hit, hand back `Arc<Bytes>`; on miss, parse once, canonicalize,
  second probe, classify cost (`store.estimate` over the algebra's patterns +
  per-template moving averages). Point-class queries (estimate below threshold)
  execute *inline* on the calling thread against the current generation — 9 µs is
  cheaper than any queue hop.
- **Tier 1 (executor pool):** everything else. Two lanes: **standard** and **heavy**
  (estimate above a second threshold). SRPT-approx ordering within lanes; lane-level
  aging (a job promoted after waiting > T_age — bounds starvation); **heavy-lane
  concurrency cap ≈ physical_cores/2** so rayon occupancy from analytics can never
  saturate the box (the §4.5 B fix); per-lane bounded queues with shed (`Shed` →
  429/503 + Retry-After). Budgets per lane (heavy gets the long deadline). Cooperative
  cancellation via the existing `QueryBudget` checkpoints.

### 6.3 Result cache

- **Key = (canonical-query hash, visibility-scope hash, dependency-epoch vector hash).**
  Canonicalization: whitespace/prefix normalization + variable renaming (Papailiou-style
  canonical labeling is the ceiling; start with cheap normalization, measure hit-rate
  delta). Scope = identity of `AuthIndex::accessible(session, mode)`'s cached set
  (Hasura's minimal-footprint lesson: *never* the WebID). Epochs: **per-named-graph
  (per-pod) epoch counters** bumped by the writer per touched graph; a cached entry
  records the max epoch of the graphs its query touched (conservative superset from the
  algebra's GRAPH/visibility footprint; queries with unbounded footprint key on the
  global generation).
- Storage: sharded map for writes + **arc-swap'd read snapshot** republished on a
  short cadence (the measured 4.8 M → 23.8 M ops/s gap is exactly this choice), or
  evmap/left-right — benchmark both in wave 1, the spike says either beats RwLock 3–5×.
- **Leases / single-flight** per key (one execution, N waiters) — the only MQO survivor.
- Values: complete serialized response bodies per (format) — what Tier 0 can `writev`.
- Bounds: byte-budget LRU (W-TinyLFU if scan-resistance proves necessary); entries
  above a size threshold are never cached (streams aren't cacheable anyway).
- **Honest fragmentation risk (pre-registered):** per-(query, scope, generation) keys
  at high tenancy can blow the key space. Mitigations measured in §7: public-scope
  collapse (most Solid reads), per-pod epochs (so unrelated writes don't churn keys),
  and the degenerate-suite gate (all-distinct ⇒ ≤ 0.2 µs/req overhead, measured).

### 6.4 Generations: arc-swapped immutable store, bounded retention ring

Replace the double buffer with a **generation chain**:

- `ArcSwap<Generation>` where `Generation { id, graph: Arc<Graph>, epochs: PodEpochs }`.
  Readers/streams `load()` (lock-free) and hold the `Arc` as long as they
  live; **the writer never waits for readers and never reclaims in place** — it builds
  generation N+1 forward (apply batch to a writer-private working copy via the existing
  overlay machinery, fold periodically) and swaps.
- **Retention bound K** (config; e.g. 4–8): the ring counts live pinned generations.
  At the bound: new streams are refused (`Shed(SnapshotPressure)`), and the oldest
  pinned stream past a wall-clock cap is cancelled with a defined truncation signal
  (chunked transfer: close + trailing error comment, the QLever/Virtuoso convention;
  Postgres `old_snapshot_threshold` precedent). Memory budget = `live + K × delta`
  (measured delta at 1 M triples: ~50 MB/generation early, amortizing with divergence).
- This **directly removes** the two measured pathologies: the 5.4 s/32 s
  pinned-snapshot writer stall (§4.3) and the reclaim-poll degradation under reader
  churn (§4.4), in exchange for bounded extra residency — the Steam/interval-GC trade,
  on purpose.

### 6.5 Write path: single sequenced writer, group commit, pod-epoch bumps

- One writer thread owns the update log. Updates are parsed/validated on arrival
  (fail fast, atomic per request as today), appended with their **conflict tag**
  (set of named graphs touched — for prod-solid-server shapes this is statically the
  resource graph + parent graph; pod/named-graph granularity is the natural and honest
  conflict unit, **not** predicate- or subject-hash-level, which the shared dictionary
  makes fictional).
- **Batching window**: commit every `min(T_window, N_max)` (e.g. 2–5 ms or 256
  updates) → one `apply_delta` batch → one generation publish → per-touched-graph epoch
  bumps → one `watch` bump for subscriptions. At the measured 17 µs/update in-place
  cost, a 5 ms window absorbs ~300 updates comfortably; commit acks return when the
  batch publishes (group commit). Solid's write rate (≤ hundreds/s) runs at ~1 update/
  window — the window adds ≤5 ms p50 write latency, far inside the 45 s contract.
- Updates inside a batch are applied in arrival order (deterministic, replayable —
  load-bearing for replication later). Cross-batch reordering: none needed — SI makes
  reads independent, and writes are serialized by the single log. ACID: A per update
  (existing atomic-failure discard), C by single-order application, I = SI for readers
  + serial writer, D explicitly out of scope for the in-memory server v1 (documented;
  the directory-backed WAL is the existing seam when persistence lands).
- Compaction stays amortized (`compact_every`); its ~200 ms spike (§4.4) moves off the
  ack path: the writer can compact the *next* working copy before swapping, never
  blocking acks beyond the window.

### 6.6 Streaming

- **Pull**: `StreamHandle` pins a generation + scope view, and produces
  backpressure-driven chunks. v1 ships on today's seam (chunks materialized, memory-
  bounded — TTFB unchanged) so the API is stable; the **engine-seams ask** is an
  iterator/callback chunk producer with budget checkpoints between chunks, which then
  drops TTFB from "full evaluation" (146 ms measured on 51 MB) to first-chunk time with
  no API change. Slow clients: per-stream byte-buffer cap + the §6.4 lifetime cap;
  the stream's view is the generation at *submit* time ⇒ **snapshot-consistent
  streaming is free by construction**, updates proceed concurrently (the ring's whole
  point).
- **Push**: lift the `subscriptions.rs` re-eval+diff machinery into the crate keyed on
  the same commit `watch`; per-subscription scope views; caps as today. (IVM is a
  non-goal; re-eval cost is bounded by the standard lane's budgets.)

### 6.7 Endpoint modes (consumers, all thin)

1. `sparq-server` native (existing routes, rewired through `Serve`).
2. **QLever-compat mode**: one POST endpoint; `application/sparql-query` → submit;
   `application/sparql-update` → Bearer check → update; sparql-results+json; honors the
   §2.10 update shapes. **Dependency flag (honest):** the engine's named-graph support
   is partial at the *server* surface today (exec.rs documents single-default-graph
   semantics; update.rs already routes per named graph). The drop-in for
   prod-solid-server requires GRAPH-aware query + `DROP SILENT GRAPH` at the endpoint —
   verify engine coverage in wave 1 before promising the drop-in.
3. Embedding: the sync core is the binding surface (wasm constraint: no
   `Instant`-based deadlines — already handled by `QueryBudget`'s cfg split).

### 6.8 Non-goals and horizontal-scaling load-bearing notes

Non-goals (v1): durability/WAL, incremental view maintenance, semantic/sub-plan
caching, query suspension/preemption, distributed anything, HTTP/3, TLS termination.

Decisions deliberately load-bearing for the queued horizontal-scaling effort:
**immutable generations are shippable units** (snapshot transfer to replicas);
the **deterministic single-order update log** is replayable (replica catch-up,
Calvin-style cross-node sequencing later); **per-pod epochs** partition cleanly
(pods are the shard key); the **cache key contract** (query, scope, epoch-vector) is
node-independent, so an external cache tier can front many nodes unchanged.

---

## 7. Benchmark and test plan

### 7.1 Harness (from scratch, `bench/serve` grows into it)

Open-loop, coordinated-omission-safe (latency from scheduled arrival — `loadgen`
already does this), Poisson arrivals + square-wave bursts (4× rate for 2 s every 20 s),
HdrHistogram-style percentile capture (p50/p99/p999/max), per-class breakdown
(hit/point/medium/heavy/update/stream), RSS + retained-generation count sampled 1 Hz.
Closed-loop runs only for saturation-ceiling discovery, clearly labeled.

### 7.2 Workloads

| suite | composition | what it proves |
|---|---|---|
| **point-Zipf** | WatDiv- or olympics-derived point/star templates, Zipfian(1.0) parameter repetition | cache hit-rate → §1.3 envelope claims |
| **BSBM-style mix** | BSBM explore + update mix over the endpoint, scaled concurrency 1→512 | end-to-end read/update interplay, generation churn |
| **Solid-profile** (the headline) | sparq-solid fixture (~1.1 K graphs) scaled up; sessions drawn from {public, owner, shared-group} per pod; queries = templated per-pod reads (Zipf over pods); **continuous update stream at 10/100/1 000 updates/s in prod-solid-server's putDocument shape**; 1 % analytical | tenancy × auth-scope cache behavior, epoch invalidation precision, write-stream stability |
| **streaming** | N concurrent large CONSTRUCT/SELECT streams (slow clients at 1 MB/s) + update stream | snapshot consistency **byte-for-byte** (stream started at gen G must equal a re-run pinned at G), retention bound behavior at K, cancellation semantics |
| **WatDiv stress** | WatDiv 10M, all 20 templates, mixed | planner/scheduler estimate quality (estimate-vs-actual scatter) |

### 7.3 Degenerate suites (no-noticeable-drop gates)

1. **all-distinct queries** (cache can't help): overhead budget ≤ 1 % vs cache-disabled
   build (measured headroom: 0.2 µs vs 9.3 µs ⇒ ~2 % worst case on pure point queries —
   gate at 3 %, investigate above).
2. **disjoint-partition access** (every query a different pod, every update a different
   pod): scheduler+epoch machinery must add ≤ 1 % vs baseline.
3. **all-expensive**: heavy lane saturates, cheap lane idle: heavy throughput within
   5 % of an unscheduled run (SRPT ordering shouldn't tax a uniform queue).
4. **write-heavy** (10 K updates/s synthetic): ack p999 < 3× window; RSS slope ~0 over
   30 min (the §4.4 invariant, now CI-checked at smaller scale).
5. **cache-thrash adversary**: unique queries sized to evict constantly: hit path for
   a protected hot set must not collapse (W-TinyLFU trigger criterion).
6. **slow-client streams** at the retention bound K: writer ack latency must stay
   < 2× window (the §4.3 regression test — the single most important new invariant).

### 7.4 Metrics + CI gating

Throughput, p50/p99/p999 per class, time-to-first-chunk, update-ack latency, fairness
(per-class max wait / starvation count), RSS ceiling, retained-generation high-water,
cache hit/fragmentation (keys per logical query). **CI regression gate (minutes, every
PR):** point-Zipf small, degenerate 1+2+6 small, update-stream 60 s memory-slope check,
snapshot-consistency byte test. **Ad-hoc big runs (manual/nightly):** Solid-profile at
scale, BSBM/WatDiv full, multi-hour soak.

---

## 8. Recommended implementation waves

1. **Wave A — generations + writer** (prereq for everything): generation ring +
   arc-swap publish, group-commit window, per-pod epochs, retention bound + defined
   cap behavior; port `AppState` onto it; keep HTTP surface unchanged. Owns:
   new `crates/sparq-serve` (core), `crates/sparq-server` rewiring.
2. **Wave B — cache + tier 0**: response cache (keys §6.3), leases, canonicalization,
   inline point execution; consume the pre-parsed-algebra entry point. Owns:
   `crates/sparq-serve` cache module + sparq-server glue.
3. **Wave C — scheduler**: lanes, estimates, aging, per-lane admission/shed.
4. **Wave D — streaming**: StreamHandle on the chunk seam, subscriptions lift,
   engine iterator seam adoption when the engine-seams work lands; QLever-compat
   endpoint mode + named-graph surface verification.
5. **Wave E — benchmark suite** per §7 (can start parallel to B; the CI gate subset
   lands with each wave's feature).

Waves A and B alone realize the envelope's fast path; C and D are quality-of-service
and Solid-contract completion.
