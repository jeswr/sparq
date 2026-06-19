# Streaming Federation Client — architecture (`sparq-fedclient`, epic sq-3183) [OPUS-4.8]

> 🤖 SPARQ agent. **Graduated** (bead `sq-zwr4`) from a *design record* to an
> **architecture document**: the 8-phase build plan it once proposed (Phases 0–7)
> is **fully shipped** — every phase maps to a module in
> `crates/sparq-fedclient/src/`, the umbrella epic **sq-dnko** is **CLOSED**, and the
> latest slice (native HTTP `FragmentTransport` + brTPF/TPF, bead `sq-yzca`) landed on
> `main` (#548). This doc now describes **what the federation client does and where**
> — verified against the crate source, not the original plan. The speculative
> phased-build framing has been dropped; the §"Honest risks & hard parts" caveats are
> preserved because they remain true of the shipped code.
>
> **Companions.** Task-oriented USE guidance for the planner this client reuses, and a
> phase-by-phase landing narrative, live in
> [`skills/federated-planning/SKILL.md`](../skills/federated-planning/SKILL.md). The
> crate's own one-screen overview is
> [`crates/sparq-fedclient/README.md`](../crates/sparq-fedclient/README.md). This file
> is the **architecture-of-record**: the layered model, the module map, the reuse
> seams, and the honest limits.
>
> **Honesty mandate.** Every claim about sparq is traced to a file + symbol. Every
> quantitative claim from the literature is *theirs* and workload-specific — none is a
> sparq measurement. sparq's ZK/MPC estate is not externally audited and is irrelevant
> here. Work-box timings are non-canonical and are not cited. The `StreamJoin`
> correctness invariant cited below is **test-asserted in-crate**, not independently
> benchmarked. Any "better than Comunica" statement is an architectural **hypothesis**
> to be validated head-to-head (FedBench / WatDiv / LDBC) before being asserted as fact.
>
> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).

---

## 1. What the federation client is

`sparq-fedclient` is a **streaming federation client**: given one SPARQL query and a set
of **heterogeneous remote sources** (full SPARQL endpoints, bindings-restricted brTPF
servers, plain TPF servers, and the *local* sparq engine), it **discovers** each source's
capability, **plans** a federated execution that pushes the most precise sub-query each
source can answer, and **streams** results back through non-blocking federation operators —
emitting solutions before any input is exhausted and before the whole query completes.

This is the **client** half of federation — the query *consumer* that fans out to many
servers — distinct from the **server** half (sparq-server's discovery descriptors, §4.3).

### 1.1 Opt-in, out of `sparq-core`/`sparq-engine` (load-bearing)

The client is a standalone workspace member, `publish = false`, gated behind the
**`fedclient` cargo feature (OFF by default)**; the adaptive half adds a further
default-OFF `fedclient-adaptive` feature. A build that does not enable `fedclient`
compiles an **empty crate** (`crates/sparq-fedclient/src/lib.rs:12-138` — every module is
`#[cfg(feature = "fedclient")]`-gated).

`sparq-core` (terms, dictionary, graph, ingest) gains nothing — no network, no async
runtime, no federation types. `sparq-engine` gains nothing: the client *depends on* the
engine (for local BGP evaluation and the existing `service` machinery) but the engine
**never** depends on the client. **The default engine build and the WASM artifact are
byte-identical with or without `sparq-fedclient`.** This boundary is proved before any
logic and re-checked on every build (§5).

### 1.2 Out of scope (named, not hand-waved)

- **Re-architecting the engine into a global async stream.** The engine stays materialised;
  the client owns the streaming boundary (see §4.4 and the honest limit in §7).
- **Link-Traversal Query Processing** (follow-your-nose over unknown link graphs) — a
  separate paradigm (Comunica's `query-sparql-link-traversal`); a possible *later* opt-in
  mode, not part of this crate.
- **Becoming a TPF/brTPF *server*.** sparq-server is a full-SPARQL endpoint, not an LDF
  server. (The client *consumes* fragment servers; it does not host one.)
- **The pushed-down streaming bind-join.** A `JoinAlgo::Bind` / brTPF leaf currently runs as
  a complete (unbound) scan that the interpreter hash-joins locally — the same multiset, a
  different execution discipline. The per-block streaming bind-join feeder is a roadmap bead
  under epic sq-3183 (§7), not shipped.

---

## 2. State of the art (distilled, with citations)

The literature splits along an interface axis and a planning axis. Citations are to the
primary sources collated in `research/feature-research-hartig.md`,
`research/feature-research-federation.md`, and the deep-research findings behind this record.

### 2.1 The Linked-Data-Fragments cost axis

LDF (Verborgh, Hartig et al., *JWS* 2016) models any Web RDF interface as offering
*fragments* = (data, metadata, hypermedia controls), on an axis from **data dumps** (all
work on the client) to **full SPARQL endpoints** (all work on the server). **TPF** sits
near the dump end: the server answers only *single triple-pattern* requests, paginated,
attaching an **estimated total-match count** (a cardinality oracle) plus hypermedia controls
(next-page link, URI template). The client *is* the query engine: it decomposes the BGP,
fetches per-pattern fragments, and joins locally, driving a **greedy join order** off the
count metadata (smallest count first). The trade-off: TPF shifts load server→client, making
public datasets cheap and **HTTP-cacheable** to host, at the cost of **more requests + more
bandwidth** and slower-but-more-stable query times. It loses on non-selective queries where
intermediate results blow up.
*(Verborgh et al., JWS 2016, https://linkeddatafragments.org/publications/jws2016.pdf.)*

### 2.2 brTPF — standardised bind-join pushdown

brTPF (Hartig & Buil-Aranda, ODBASE 2016; arXiv:1608.08148) extends TPF so the client may
**attach a finite block of solution mappings** (up to `maxMpR`, max-mappings-per-request)
to a pattern request; the server returns only triples that join with at least one attached
binding. This pushes the **bind-join** into the data-access interface. On WatDiv (145
queries — *their* workload, not sparq's): `maxMpR=50` cut HTTP requests to ~6.5% and data
transferred to ~53.5% of plain TPF. The GET prototype capped `maxMpR` at ~50 (HTTP 414);
POST lifts the cap. *Client action:* block/bind nested-loop join — gather up-to-`maxMpR`
upstream bindings, attach, stream matches, repeat.
*(arXiv:1608.08148.)*

### 2.3 SPARQL 1.1 SERVICE + VALUES pushdown

W3C SPARQL 1.1 Federated Query standardises `SERVICE <iri> { … }` (evaluate the inner
pattern at the named endpoint, join locally) and `SERVICE SILENT` (a failed service →
single empty solution). Crucially, the spec **explicitly blesses `VALUES`** to constrain a
remote call by locally-computed bindings — the standardised hook for **bind-join
pushdown**, the same lever FedX's bound join and brTPF's `maxMpR` exploit, surfaced at the
query-language level. Decomposition goal (FedX/GraphDB framing): group triple patterns into
per-member **exclusive groups**, push each as one subquery, then bound-join across groups
with VALUES blocks — minimise time while preserving completeness.
*(https://www.w3.org/TR/sparql11-federated-query/.)*

### 2.4 Source selection + cardinality estimation (the endpoint-federation lineage)

- **FedX** (ISWC 2011): zero-preprocessing source selection via **ASK** probes; **exclusive
  groups**; **bound joins** (VALUES/UNION block pushdown); variable-counting join order.
  Still the RDF4J/GraphDB baseline.
- **SPLENDID** (COLD 2011) / **SemaGrow** (SEMANTiCS 2015): **VoID**-statistics-driven
  planning, ASK fallback; SemaGrow adds a reactive non-blocking engine + vocabulary mapping.
- **HiBISCuS** (ESWC 2014): **join-aware** source pruning via a URI-authority hypergraph —
  keep recall, query far fewer sources.
- **CostFed** (SEMANTiCS 2018): **cost-based** planning with trie/prefix authority indices +
  triple-and-join cardinality estimates.
- **Characteristic Sets / Pairs** (Neumann & Moerkotte, ICDE 2011): accurate cardinality
  for **star-shaped** subqueries and their joins. **Odyssey** (ISWC 2017) generalises CS/CP
  to federation (Federated Characteristic Pairs) for **DP cost-based** planning.
- **Empirical meta-finding** (Qudus et al., *SWJ* 2021, arXiv:2104.00984): **cardinality-
  estimation accuracy matters more for final plan quality than the source-selection
  strategy alone**; estimation errors >100× cause bad plans. (Treat the paper's specific
  percentages as directional.)

### 2.5 Capability discovery — Service Description + VoID

**VoID** (W3C Note) describes a *dataset* (triple/distinct-subject/object counts, class/
property partitions) — the inputs cardinality planners want. **SPARQL Service Description**
(W3C; SD 1.2 WD) describes an *endpoint* (supported language features, entailment regimes,
result formats, default/named graphs) and can embed VoID. *Client action:* prefer published
SD/VoID over live ASK probing; build the cardinality model from VoID partitions; read SD
before pushing a feature a source may not support, to avoid silent incompleteness; fall back
to ASK for bound patterns the stats miss. Do **not** assume publication — much of the public
LOD cloud lacks accurate VoID, which is exactly why FedX (ASK, zero-metadata) and
characteristic-set engines (build their own summaries) exist.
*(https://www.w3.org/TR/void/, https://w3c.github.io/sparql-service-description/spec/.)*

### 2.6 Streaming + adaptive execution

Networked sources have unpredictable latency and may stall; blocking operators (sort-merge,
build-then-probe hash) can deadlock a plan on one slow source. **ANAPSID** (Acosta, Vidal et
al., ISWC 2011) is the canonical adaptive federated engine: **non-blocking** physical
operators (symmetric/double-pipelined hash join + XJoin, adaptive dependent join) that
produce results incrementally and **detect blocked/bursty sources**, rescheduling
(Eddies/MJoin lineage). *Client action:* pipelined non-blocking joins so first results
stream early and a stalled source can't block siblings; detect stalls and adapt; run
adaptivity over *block* requests (brTPF/VALUES), not per-tuple.

### 2.7 The interface spectrum (recent SOTA filling the middle)

SaGe (web preemption, WWW 2019 — full BGP in bounded quanta, suspend/resume), Star Pattern
Fragments (server answers star subqueries), smart-KG (ship compressed HDT predicate-family
partitions, client evaluates locally), and the **Heterogeneous-LDF federation framework**
(Heling & Acosta, WWW 2022, arXiv:2102.03269) — a **single client federating across mixed
interface types**, modelling each member's capabilities to pick the right access method per
source. *This is the endgame the client targets* (minus the SaGe/SPF/smart-KG adapters, not
built): endpoint > brTPF > TPF > dump/HDT > local engine, chosen per source.

---

## 3. The sparq seams it reuses, and the gap it filled

Every claim here is traced to a file + symbol against the current tree.

### 3.1 What it REUSES (none of it re-implemented in the client)

**A. `crates/sparq-fedplan/` — the pure, no-I/O planner + streaming operator (opt-in).**

- **Source selection** — `select_sources(bgp, sources) -> Vec<PatternSources>`
  (`selection.rs:95`): HiBISCuS-style **recall-safe** pruning via `can_contribute`
  (`selection.rs:123`); CostFed-style skew-aware per-(pattern, source) cardinality in
  `estimate_cardinality` (`selection.rs:171`).
- **Source descriptor model** — `SourceDescriptor` (`descriptor.rs:90`), built via
  `builder()` or **`SourceDescriptor::from_void_nt(id, nt)`** (`descriptor.rs:322`), which
  parses exactly the VoID + `scs:` N-Triples a sparq-server emits (`PredPartition`,
  `ClassPartition`, `CharSet`, and the Neumann–Moerkotte `star_cardinality` /
  `star_subjects` estimates).
- **Join planner** — `plan_bgp(bgp, selection, descriptors, opts) -> Option<JoinTree>`
  (`plan.rs:180`): greedy left-deep order; per-join **bind-vs-hash** decision in `cost_join`;
  large hash-class joins tagged `JoinAlgo::Streaming`. Public surface: `JoinAlgo`
  (`Bind`/`Hash`/`Streaming`), `JoinNode`, `JoinTree::join_order()`, `PlanOptions`.
- **Streaming join OPERATOR** — `StreamJoin` (`stream.rs`): a symmetric (XJoin-style)
  non-blocking hash join, push-based, with bounded **operator spill** to a temp file. The
  correctness invariant (streamed+spilled result multiset-equal to the blocking join, any
  interleaving / any budget) is **test-asserted in-crate only**.
- **Adaptive RE-planner** — `AdaptiveExecutor` (`fedplan` `adaptive` module, behind the
  `adaptive-replan` feature, bead `sq-7s4z`): the divergence trigger, hysteresis margin,
  suffix re-ordering, latency weighting, and the commutativity soundness proof. The client
  *drives* it; it does not re-implement it.

**B. `crates/sparq-engine/src/service.rs` + `eval_service`/`try_bound_join_service` in
`exec.rs` — the SERVICE-clause executor (opt-in `service` feature).**

- `eval_remote` → `Transport::fetch(endpoint, query) -> String` → `parse_srj`. Production
  transport `HttpTransport` is a blocking ureq POST, form-encoded, `Accept:
  application/sparql-results+json` (gated off wasm).
- **Bound join (brTPF/FedX-style VALUES pushdown)** — `try_bound_join_service`: DISTINCT,
  fully-bound, pushable join-key tuples rendered into `VALUES` blocks of `bind_block_size()`
  (default `DEFAULT_BIND_BLOCK = 50`).
- **SSRF egress policy** — default-DENY private/internal ranges via `is_forbidden_ip`,
  installed as ureq's resolver `EgressFilterResolver` (DNS-rebinding-safe). **A genuine
  differentiator** — most engines ship SSRF-open `SERVICE`. The client reuses this guard for
  every native transport.

**C. `crates/sparq-server/` — the SERVER side (NOT a TPF/brTPF server).** SPARQL 1.1
Protocol + Graph Store HTTP; **federation DISCOVERY descriptors** (opt-in
`federation-descriptors` feature, OFF): `void_descriptor` serves `/.well-known/void` and
`service_description` serves `GET /sparql` with no query. VoID rides with the `scs:`
characteristic-set extension via `Introspection::to_void_with_cs`
(`sparq-introspect/src/lib.rs`) — the **exact producer** for `SourceDescriptor::from_void_nt`.
A brTPF binding-block server wire landed under `sq-dxhb` (the text wire the client's `wire`
module also speaks, §4.6).

**D. The engine result model is MATERIALISED.** `QueryResult { vars, rows }`
(`sparq-engine/src/lib.rs`) is fully materialised; there is **no solution-level
`Iterator`**. The client therefore owns the streaming boundary itself (§4.4); a local
sub-BGP evaluation through the engine remains a materialised call (§7).

### 3.2 The gap the client filled

The planning brain, the CS cardinality model, the streaming-join operator, the per-source
SPARQL adapter + bind-join primitive, the SSRF guard, and the discovery payload
producer/consumer seam were all done and reusable. The client supplied the **execution glue**:
network execution over heterogeneous sources, capability discovery + most-precise-query
selection, streaming federation operators wired to real fetches, an interpreter that turns a
`JoinTree` into fetches (the missing index→adapter mapping), adaptive re-planning, and
bounded concurrent fan-out. All of this is now shipped — §4 describes it as built.

---

## 4. Architecture (as built) — five layers + the module map

Five layers, top to bottom. Each names the existing sparq seam it reuses **and the module
that realises it**.

```text
                          ┌─────────────────────────────────────────────┐
   query string  ────────▶│  sparq-fedclient  (opt-in crate, feature     │
                          │                    `fedclient`, OFF)          │
   discovery / capability │  discovery.rs:  GET VoID+SD → Capability      │
                          │   (REUSE from_void_nt) + ASK-probe fallback   │
   source abstraction      │  source.rs:  SourceType {Endpoint|BrTpf|Tpf  │
                          │   |Local}, FederatedSource, Capability,       │
                          │   Endpoint + Tpf/BrTpf adapters, Transport,   │
                          │   FragmentTransport, EgressGuard              │
   planner bridge          │  planner.rs:  lower BGP → fedplan Bgp;        │
                          │   select_sources; plan_bgp; SourceResolver    │
                          │   (index → adapter)         (REUSE fedplan)   │
   pushdown                │  pushdown.rs:  exclusive groups; push_group;  │
                          │   common_variable_check; render_values_block  │
   physical exec           │  operators.rs / stream.rs:  JoinTree →        │
                          │   Bind/Hash/Streaming → StreamJoin (REUSE),   │
                          │   Local → sparq-engine eval; SolutionStream,  │
                          │   ScatterPool; wire.rs brTPF block codec      │
   adaptive (opt-in)       │  adaptive.rs:  observed card → re-plan suffix │
                          │   (REUSE fedplan AdaptiveExecutor)            │
                          └───────────────┬──────────────────────────────┘
                                          │ local BGP eval, term interning, SSRF guard
                                          ▼
                          sparq-engine (REUSE: service.rs transport+VALUES+SSRF, local eval)
                          sparq-core    (UNCHANGED)
```

| Layer | Module | What it does (verified) | Reuses |
| --- | --- | --- | --- |
| Capability discovery | `discovery.rs` | GET SD (`GET <endpoint>` no `query`) + `/.well-known/void`, parse → `Capability` (+ optional `SourceDescriptor`); FedX-style ASK-probe fallback; SSRF-guarded `Fetcher` seam | `from_void_nt`; engine SSRF guard |
| Source abstraction | `source.rs` | `SourceType {Endpoint\|BrTpf\|Tpf\|Local}`, `FederatedSource` trait, fine-grained `Capability`, `Endpoint` (SRJ) + `TpfSource`/`BrTpfSource` (fragment) adapters, `Transport`/`FragmentTransport` seams, native `HttpTransport`/`HttpFragmentTransport` behind `EgressGuard` | engine SRJ transport + SSRF resolver |
| Planner bridge | `planner.rs` | lower query BGP → fedplan light `Bgp`; `SourceResolver` (plan pattern/source *index* → `TriplePattern` / adapter); `lower_leaf` / `lower_leaf_fragment` | `select_sources`, `plan_bgp` |
| Pushdown | `pushdown.rs` | FedX `exclusive_groups`; `push_group` (max evaluable sub-algebra: projection + capability-covered FILTERs + ORDER/LIMIT); the **exact** `common_variable_check`; `render_values_block` / `bind_block_size` | engine `service.rs` block primitive (mirrored) |
| Physical exec | `operators.rs`, `stream.rs` | `materialize_single_source` / `materialize_multi_source` (materialised left-deep) + `stream_single_source` / `stream_multi_source` (the `SolutionStream` boundary, `ScatterPool` bounded blocking pool, `StreamingJoin` over fedplan's `StreamJoin`) | fedplan `StreamJoin`; engine local eval |
| brTPF block wire | `wire.rs` | compact self-describing BINARY mapping wire (`encode_bindings`/`decode_bindings`) + the line-oriented TEXT wire (`encode_bindings_text`) the server parses — a codec only | server brTPF wire (`sq-dxhb`) |
| Adaptive re-plan | `adaptive.rs` (feature `fedclient-adaptive`) | `execute_adaptive_single_source`: leaf-scan records REAL observed cardinalities, then re-plans the unjoined remainder at each operator boundary, at most once per boundary | fedplan `AdaptiveExecutor` |

### 4.1 Source-type abstraction + capability discovery (`source.rs`, `discovery.rs`)

A `SourceType` enum with one adapter per interface, each implementing the `FederatedSource`
trait — the sparq analogue of Comunica's hypermedia "negotiate down to the most-capable
handler", but with a **fine-grained, statically-resolved** `Capability` descriptor (interface,
SPARQL version, pushable FILTER class, bind-join mode, aggregates / property-paths /
ORDER+LIMIT pushability, result formats) instead of Comunica's coarse "service-description? /
search-form? / totalItems?" runtime check.

- **Endpoint discovery.** GET the SD document (`GET <endpoint>` no `query`) + `/.well-known/void`,
  parse VoID+`scs:` via the existing **`SourceDescriptor::from_void_nt`** seam (the
  producer/consumer match exists end-to-end: server `to_void_with_cs` → client `from_void_nt`),
  parse SD into a `Capability` (the one genuinely-new client-side parser this layer needed).
  ASK-probe fallback when nothing is published. Every fetch is behind a default-deny
  SSRF-guarded `Fetcher`.
- **Fragment sources.** `TpfSource` / `BrTpfSource` read the Hydra `hydra:totalItems` count
  (cardinality oracle) and answer one triple pattern completely over the `FragmentTransport`
  seam. The native `HttpFragmentTransport` serialises the Hydra URI template
  (`?subject=&predicate=&object=` + the brTPF `values` block), follows `hydra:next` to
  exhaustion, and parses the Turtle/TriG body (splitting Hydra/VoID control triples from data).
- **Local.** Capability is "everything"; statistics from `sparq-introspect`; evaluation
  through `sparq-engine` (no network; the SSRF guard is moot).

### 4.2 Planner-to-physical bridge — REUSE `sparq-fedplan` (`planner.rs`)

The client **does not write a new planner.** It lowers the parsed query's BGP into fedplan's
light `Bgp`/`TriplePattern`/`Term`/`Var`, builds one `SourceDescriptor` per discovered source,
and calls **`select_sources`** then **`plan_bgp`**. The plan speaks pattern *indices* and
source *indices* with no endpoint mapping; **`SourceResolver`** is the index→adapter resolution
layer (it pairs the BGP and the adapter slice, requires them in the same order, and range-checks
every lookup). The cost-based join order, the bind-vs-hash decision, and the CS star cardinality
are already correct, tested, and deterministic — the client supplies the descriptors and
consumes the `JoinTree`.

### 4.3 Capability-aware pushdown — ask each server its MOST PRECISE answerable query (`pushdown.rs`)

For each FedX-style **exclusive group** (`exclusive_groups`: a connected sub-pattern whose only
retained source is one member — exactly-one-source, same-source, share-a-variable, via
union-find), `push_group` builds the **maximal evaluable sub-algebra** for that source's
`Capability`:

- **Full endpoint** — the whole group as one multi-pattern `SELECT`: projection trimmed to the
  join + output vars, the FILTER conjuncts the source's `FilterClass` covers **and** that pass
  the `common_variable_check`, `ORDER`/`LIMIT` when the capability allows. Bind-join across
  groups via **VALUES blocks** (`render_values_block` + `bind_block_size`).
- **Fragment source** (brTPF/TPF) — one triple pattern only (no collapse, no FILTER pushed —
  honest about a fragment server's access unit); brTPF carries a `maxMpR`-bounded binding block.
- **Local engine** — evaluate the sub-BGP directly.

`common_variable_check(filter, group_vars)` is the **exact** check Comunica is documented to
omit (#834/#609): push a conjunct **only when every variable it references is bound by the
group**. Pushdown only ever *narrows* what a source returns, so it is correctness-preserving by
the same argument the engine's `try_bound_join_service` uses; anything a source cannot evaluate
is kept local (the engine evaluates the residual).

### 4.4 Streaming federation operators — REUSE `StreamJoin` + engine local eval (`operators.rs`, `stream.rs`)

- **`SolutionStream`** (`stream.rs`) — the bounded, backpressured `Iterator` the client owns at
  its boundary (the engine stays materialised, §7), built over a `std::sync::mpsc::sync_channel`
  (the channel bound *is* the backpressure). Adapters yield it; operators consume and produce it.
  Bounded by Rust ownership + explicit buffer bounds + the reused `StreamJoin` spill — not a GC
  heap, structurally avoiding Comunica's documented heap-OOM/broken-backpressure bugs.
- **Materialised interpreter** (`materialize_single_source`) — walks the `JoinTree`, fetches each
  leaf's SRJ through the Phase-2 adapter, parses it, and natural-joins in the plan's join order
  with a left-deep hash join. Result equals local `sparq-engine` evaluation of the same query
  (the load-bearing invariant, `tests/planner_result_equals_local_eval.rs`).
- **Streaming interpreter** (`stream_single_source`) — fans each leaf's blocking fetch onto the
  bounded `ScatterPool` and chains the leaves through `StreamingJoin` (which drives fedplan's
  `StreamJoin` over two `SolutionStream`s, bridging `oxrdf::Term` rows ↔ the light `Tuple` model
  via the canonical N-Triples form). Results **emit before inputs are exhausted**; the streamed
  multiset is multiset-equal to the materialised result for any source-arrival interleaving
  (`tests/streaming_result_equals_phase3.rs`, with injected delays + a forced spill).
- **Multi-source UNION-per-leaf** (`materialize_multi_source` / `stream_multi_source`, bead
  `sq-7yf0`) — a leaf the planner retained >1 source for is answered as the **bag-union** of
  every retained source's solutions (SPARQL UNION multiset semantics). The single-source entry
  points keep the fail-closed `InterpError::MultiSource` contract; multi-source is the opt-in
  entry point.
- **Concurrent fan-out** — the `ScatterPool` is a bounded **blocking thread-pool** over the
  blocking transport: **no async runtime is pulled in**; all concurrency is `std`-only and
  confined to the opt-in crate (the §7 async/runtime decision, resolved this way).

### 4.5 Adaptive re-planning (the ANAPSID half) — `adaptive.rs`, feature `fedclient-adaptive`

A feedback loop: a leaf-scan phase fetches each leaf once through the real adapter and records
its **REAL observed row count**; an adaptive join-ordering phase then drives fedplan's
`AdaptiveExecutor` — at each operator boundary it re-invokes the planner on the **unjoined
remainder** when an observation diverges past `divergence_factor`, adopting the cheaper suffix
only when it clears the hysteresis margin, **at most once per boundary** (no thrash). The
re-plan DECISION engine (trigger, hysteresis, suffix re-ordering, soundness proof) lives in
`sparq-fedplan`'s `AdaptiveExecutor` (bead `sq-7s4z`); the client supplies real observed
cardinalities and joins the re-ordered suffix with the same `natural_join`. **Re-planning
changes the plan, never the answer** — the adaptive result is multiset-equal to the static plan
and to local engine eval (`tests/adaptive_result_equals_static.rs`, across a genuine
large-divergence switch). Behind the default-OFF `fedclient-adaptive` feature (which pulls
`sparq-fedplan/adaptive-replan`); off → the `adaptive` module compiles to nothing.

The ANAPSID "adaptive operator" refinement (estimate a leaf's cardinality from a *prefix* of its
rows while still streaming it) and live source failover remain roadmap beads under epic sq-3183.

### 4.6 The brTPF binding-block wire codec — `wire.rs`

The brTPF bind-join attaches a SET of upstream solution mappings (a `&[FragBinding]` block, at
most `maxMpR`) to each fragment request. The sparq server (`sq-dxhb`) parses a **line-oriented
TEXT wire** (one mapping per line, `position=term` pairs, each term N-Triples-decorated). The
`wire` module adds the **compact, self-describing BINARY wire** (`encode_bindings` /
`decode_bindings`) — a 1-byte per-mapping header bitmask records which of `s`/`p`/`o` is bound
(the header bit IS the name), a 1-byte kind tag distinguishes IRI/blank/literal (bare lexical
bytes, length-prefixed, no framing) — plus the text-wire writer (`encode_bindings_text`) so a
client can speak either form. A 4-byte magic (`BINARY_MAGIC`, ASCII `bTPF`) + 1-byte
`BINARY_VERSION` make a future revision detectable, and `decode_bindings` validates every length
(a truncated / bad-magic / bad-kind buffer is a clean `WireError`, never a panic — the crate is
`forbid(unsafe_code)`). **Honest scope — a codec only:** it converts `&[FragBinding]` ↔ bytes; it
issues no request. The native `HttpFragmentTransport` attaches the **text** form on the `values`
parameter; the binary wire is the compact alternative a body-carrying transport emits.

### 4.7 How this aims to do BETTER than Comunica (architectural predictions, not measurements)

Comunica's own authors state its goal is **"modularity, and not absolute performance"** (ISWC
2018), and its maintainers catalogue the gaps (issue #846). The predictions below are grounded
in those *documented* gaps and must be validated head-to-head before being asserted as fact (§7).

1. **Precise, capability-aware pushdown vs Comunica's coarse model.** A fine-grained
   `Capability` (§4.1) + the exact `common_variable_check` (§4.3) lets the client push the
   maximal evaluable sub-algebra per source, only when shared variables exist — Comunica admits
   FILTER/expression pushdown gaps and a missing common-variable check (#834/#609).
2. **Real cost-based join ordering with a true cardinality estimator.** sparq reuses
   characteristic-set star cardinality (`fedplan` `star_cardinality`) + a cost-based bind-vs-hash
   decision; Comunica has no selectivity estimator beyond `totalItems × 0.0001`.
3. **No mediation indirection in the hot path** — operator/algorithm choice resolved at plan
   time, a monomorphised Rust pipeline.
4. **Bounded-memory streaming with real backpressure** (Rust ownership + `StreamJoin` spill).
5. **Dictionary-encoded execution** end-to-end for the *local* portion.
6. **SSRF-safe `SERVICE` by default** (the engine's `EgressFilterResolver`).

**What to KEEP from Comunica** (genuinely good design): capability negotiation as a first-class
step; the multi-dimensional join cost model (CPU + memory + **blocking** + **I/O**); lazy
incremental streaming; an EXPLAIN that surfaces chosen physical operator + estimated-vs-actual
cardinality; adaptive deferral **when sources are genuinely unknown** (opt-in mode, not default).

---

## 5. Crate / dependency boundaries — proving it stays OUT of core

`sparq-fedclient` is a workspace member, `publish = false`.

```text
sparq-fedclient  (feature `fedclient`, OFF by default; `fedclient-adaptive` for §4.5)
   ├── depends on  sparq-fedplan   (feature `fedplan` / `adaptive-replan`) — planner + StreamJoin + AdaptiveExecutor (REUSE)
   ├── depends on  sparq-engine    (feature `service`)   — SERVICE transport + VALUES + SSRF + local eval (REUSE)
   ├── depends on  sparq-introspect (optional)           — local-source stats
   └── depends on  oxrdf / spargebra (already in tree)   — term + algebra types

   does NOT touch:  sparq-core      (unchanged — no network, no async, no fed types)
   the arrow into sparq-engine is one-way: engine NEVER depends on fedclient.
```

The boundary is **proved before any logic** and re-checked on every build, in both feature
states, by two complementary checks (so neither `sparq-core` nor `sparq-engine` gains an edge
*to* `sparq-fedclient`):

- **`scripts/fedclient-boundary-guard.sh`** — a CI step (in `feature-matrix.yml`) that inverts
  the dependency graph (`cargo tree -i sparq-fedclient`) and fails if `sparq-core` or
  `sparq-engine` appears as a dependent.
- **`crates/sparq-fedclient/tests/boundary.rs`** — a hermetic `cargo metadata` test asserting
  the same invariant from inside the suite.

Feature-gated OFF by default exactly like `fedplan` and `service`: a build that does not enable
`fedclient` compiles nothing of it; the default engine build and the WASM artifact are
byte-identical with or without the crate. Mirrors the existing precedent (`sparq-fedplan`,
`sparq-canon`, `sparq-prov`, `sparq-mpc`, `sparq-zk` are all standalone opt-in members).

---

## 6. Honest risks & hard parts (no over-claim)

*(Carried verbatim-in-spirit from the original design record — every one of these still holds
of the shipped code; verified against the crate source for this graduation, bead `sq-zwr4`.)*

- **Pushdown correctness is the sharpest edge.** Pushing a filter/projection/ORDER/LIMIT a
  source evaluates with *different* semantics (numeric coercion, collation, language-tag/
  datatype handling, `NaN`, timezone) than local evaluation can silently change results. The
  safe rule: push only the sub-algebra whose remote semantics are provably identical to local;
  when in doubt, **keep it local**. The engine's `try_bound_join_service` takes this posture
  (it falls back to verbatim forwarding on any non-pushable shape — blank node, triple term,
  variable endpoint, unbound key), and the client keeps that discipline as the surface grows:
  the `common_variable_check` is exact (push a conjunct only when all its variables are bound in
  the group — `pushdown.rs:329`).

- **Partial-capability fallback.** A source may advertise SPARQL 1.1 but reject a specific
  feature at runtime (timeouts, query-complexity limits, partial SD). The client must degrade
  gracefully: on a rejected pushdown, fall back to a less-precise query (or local evaluation of
  the residual) without losing results — and must **not** treat `SERVICE SILENT`-style swallow
  as a substitute for completeness (silently empty ≠ correct). Discovery data can be stale or
  wrong; ASK fallback and conservative (recall-safe) defaults are mandatory.

- **Result completeness under TPF/brTPF.** Plain TPF shifts all joins to the client; on a
  non-selective query the intermediate-result/request blow-up is real (the central TPF
  trade-off, §2.1) and can dominate. brTPF's `maxMpR` cap (≈50 on GET) bounds binding-block
  size; an under-sized block re-introduces request explosion (Comunica's documented #1196).
  Completeness itself is preserved by these interfaces (they return all matching triples for a
  pattern/binding), but *performance* completeness — finishing in acceptable time/bandwidth —
  is workload-dependent and **must not be over-promised**. **Live note:** the pushed-down
  *streaming* bind-join is not yet shipped — a `JoinAlgo::Bind`/brTPF leaf runs as a complete
  scan the interpreter hash-joins locally (`operators.rs:344-365`); the per-block streaming
  feeder is a roadmap bead. The result multiset is identical; the request/bandwidth
  optimisation that brTPF's `maxMpR` exists to deliver is the part still ahead.

- **The engine boundary is materialised; the client owns the stream.** `sparq-engine` returns a
  fully-materialised `QueryResult` and evaluates operators blocking (§3.1.D). This design does
  **not** re-architect the engine into a global async stream; the client streams at *its*
  boundary (adapter outputs + federation operators), and local sub-BGP evaluation through the
  engine remains a materialised call (`stream.rs:8-14`). That is a deliberate, honest limit:
  end-to-end solution-level laziness *through* the engine is out of scope and would be a much
  larger change.

- **Async/runtime choice was a real decision — resolved without an async runtime.** The
  engine's transport is blocking ureq; true concurrent fan-out wants either a bounded blocking
  worker pool or an async runtime. The client uses a **bounded blocking thread pool**
  (`ScatterPool`, `operators.rs:694`) over the existing blocking transport — **no async runtime
  is pulled into the tree**, and all concurrency stays `std`-only and inside the opt-in crate
  (never leaking into core/engine). An async transport behind the same `Transport` seam remains
  a future option if measurement warrants it.

- **Adaptive re-planning can thrash or regress.** Re-planning on noisy early estimates can pick a
  worse suffix; it is opt-in (`fedclient-adaptive`), bounded (at most once per operator boundary,
  hysteresis margin + `max_replans` budget), and must be measured before being recommended.
  ANAPSID-style adaptivity genuinely wins only when cardinalities are *unknowable* up front —
  where descriptors exist and are accurate, the static cost-based plan is simpler and at least as
  good (which is why it stays the default).

- **The "better than Comunica" claims are predictions, not measurements.** Every advantage in
  §4.7 is an *architectural* prediction grounded in Comunica's own documented gaps. Comunica is
  battle-tested across many real interfaces and has a large SPARQL-1.1 operator surface; "we have
  a better cost model" is only an advantage once `sparq-fedclient` reaches comparable interface +
  SPARQL-1.1 completeness, and it must be validated head-to-head (FedBench / WatDiv / LDBC)
  before being stated as fact. No timings appear in this record; the in-crate `StreamJoin`
  correctness invariant is test-asserted, not independently audited.

---

## 7. References (primary)

- Verborgh, Hartig et al., *Triple Pattern Fragments*, JWS 2016 —
  https://linkeddatafragments.org/publications/jws2016.pdf
- Hartig & Buil-Aranda, *brTPF*, ODBASE 2016 — https://arxiv.org/abs/1608.08148
- W3C, *SPARQL 1.1 Federated Query* — https://www.w3.org/TR/sparql11-federated-query/
- W3C, *VoID* — https://www.w3.org/TR/void/ ; *SPARQL Service Description* —
  https://w3c.github.io/sparql-service-description/spec/
- Schwarte et al., *FedX*, ISWC 2011; Acosta, Vidal et al., *ANAPSID*, ISWC 2011;
  Görlitz & Staab, *SPLENDID*, COLD 2011; Charalambidis et al., *SemaGrow*, SEMANTiCS 2015;
  Saleem & Ngonga Ngomo, *HiBISCuS*, ESWC 2014; Saleem et al., *CostFed*, SEMANTiCS 2018.
- Neumann & Moerkotte, *Characteristic Sets*, ICDE 2011; Montoya, Skaf-Molli, Hose,
  *Odyssey*, ISWC 2017 — https://arxiv.org/abs/1705.06135.
- Qudus, Saleem, Ngonga Ngomo, Lee, *An Empirical Evaluation of Cost-based Federated SPARQL
  Query Processing Engines*, SWJ 2021 — https://arxiv.org/abs/2104.00984.
- Minier, Skaf-Molli, Molli, *SaGe*, WWW 2019 — https://arxiv.org/pdf/1902.04790;
  Heling & Acosta, *Heterogeneous-LDF federation*, WWW 2022 — https://arxiv.org/pdf/2102.03269.
- Comunica: ISWC 2018 (https://comunica.github.io/Article-ISWC2018-Resource/), JWS 2019,
  the comunica.dev architecture/joins/hypermedia/explain docs, and maintainer issue #846
  (https://github.com/comunica/comunica/issues/846).

### sparq code grounding (current tree)

`crates/sparq-fedclient/src/{lib.rs,discovery.rs,source.rs,planner.rs,pushdown.rs,operators.rs,stream.rs,adaptive.rs,wire.rs}`
+ `tests/`; `crates/sparq-fedplan/src/{lib.rs,selection.rs,plan.rs,descriptor.rs,stream.rs,pattern.rs,adaptive.rs}`;
`crates/sparq-engine/src/service.rs` + `exec.rs` (`eval_service`, `try_bound_join_service`);
`crates/sparq-engine/src/lib.rs` (`QueryResult`); `crates/sparq-server/src/descriptors.rs`;
`crates/sparq-introspect/src/lib.rs` (`to_void_with_cs`); `scripts/fedclient-boundary-guard.sh`;
and `research/feature-research-{federation,hartig}.md`. Task-oriented USE guidance:
`skills/federated-planning/SKILL.md`. Crate overview: `crates/sparq-fedclient/README.md`.

[OPUS-4.8] — graduated from a design record to an architecture doc under bead `sq-zwr4`;
flagged for Fable re-review when available.
