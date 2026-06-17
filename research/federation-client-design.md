# Streaming Federation Client — design record (epic sq-3183) [OPUS-4.8]

> 🤖 SPARQ agent. Design-for-maintainer-review. This is a **design record**, not an
> implementation. It proposes a new **opt-in crate** that is a **streaming federation
> CLIENT** over heterogeneous remote RDF sources. It is grounded FIRST in sparq's
> *actual* federation code (so it proposes the real gap, not a re-proposal of what
> exists), then distils the SOTA, then specifies an architecture that **reuses**
> `sparq-fedplan` (planner + streaming operator) and `sparq-engine` (local eval +
> SERVICE bind-join + SSRF guard) and stays **out of** `sparq-core` / `sparq-engine`.
>
> **Honesty mandate.** Every claim about sparq is traced to a file + symbol. Every
> quantitative claim from the literature is *theirs* and workload-specific — none is a
> sparq measurement. sparq's ZK/MPC estate is not externally audited and is irrelevant
> here. Work-box timings are non-canonical and are not cited. The `StreamJoin`
> correctness invariant cited below is **test-asserted in-crate**, not independently
> benchmarked. This record makes **architectural predictions**, not measured claims:
> any "better than Comunica" statement is a hypothesis to be validated head-to-head
> (FedBench / WatDiv / LDBC) before being asserted as fact.
>
> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).

---

## 1. Problem & scope

### 1.1 What we are building

A **streaming federation client**: a component that, given one SPARQL query and a set of
**heterogeneous remote sources** (full SPARQL endpoints, bindings-restricted TPF servers,
plain TPF servers, RDF dumps / HDT files, and the *local* sparq engine), **discovers each
source's capability**, **plans** a federated execution that pushes the most precise
sub-query each source can answer, and **streams** results back through non-blocking
federation operators — emitting solutions before any input is exhausted and before the
whole query completes.

This is the **client** half of federation — the query *consumer* that fans out to many
servers — distinct from the **server** half (sparq-server's discovery descriptors, §3).

### 1.2 Hard scope constraint: a separate, opt-in crate — explicitly NOT in core

The client is a **new standalone workspace member** (proposed name **`sparq-fedclient`**,
§5), gated behind an OFF-by-default cargo feature, following the project's
opt-in-feature-architecture rule. It is **explicitly NOT** in `sparq-core` or
`sparq-engine`:

- `sparq-core` (terms, dictionary, graph, ingest) gains **nothing** — no network, no async
  runtime, no federation types.
- `sparq-engine` gains **nothing new**: the federation client *depends on* the engine (for
  local BGP evaluation and the existing `service` machinery) but the engine never depends
  on the client. The default engine build and the WASM artifact are byte-identical with or
  without `sparq-fedclient`.

This mirrors the existing boundary: `sparq-fedplan` is already a standalone member with
**no `sparq-core`/`sparq-engine` dependency** and is `publish = false`, feature-gated
`fedplan` OFF by default (`crates/sparq-fedplan/Cargo.toml:29`, `src/lib.rs:71-100`); a
build without the feature *compiles an empty crate*. The federation client sits one layer
above: it is allowed to depend on the engine (unlike fedplan, which deliberately does not),
but the dependency arrow only ever points *into* the engine, never out of it.

### 1.3 Out of scope (named, not hand-waved)

- Re-architecting the engine's result model into a global async stream (the engine stays
  materialised; the client owns the streaming boundary — see §4.4 and the honest caveat in
  §7).
- Link-Traversal Query Processing (follow-your-nose over unknown link graphs). LTQP is a
  separate paradigm (Comunica's `query-sparql-link-traversal`); it can be a *later* opt-in
  mode but is not part of this crate's first delivery.
- Becoming a TPF/brTPF *server* — sparq-server is a full-SPARQL endpoint, not an LDF
  server (`grep` finds no TPF/brTPF/Hydra fragment endpoint in `crates/sparq-server/src`;
  the only "fragment" hits are unrelated error-redaction comments).

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
source. *This is the endgame the present design targets* (minus the SaGe/SPF/smart-KG
adapters, deferred): endpoint > brTPF > TPF > dump/HDT > local engine, chosen per source.

---

## 3. Where sparq is today, and the precise GAP

Every claim here is traced to a file + symbol. Verified directly against the tree at the
base of this branch.

### 3.1 What EXISTS and is reusable

**A. `crates/sparq-fedplan/` — a pure, no-I/O planner + streaming operator (opt-in, no
consumer yet).**

- **Source selection** — `select_sources(bgp, sources) -> Vec<PatternSources>`
  (`src/selection.rs:95`): HiBISCuS-style **recall-safe** pruning via `can_contribute`
  (`selection.rs:123`) — three positive-evidence prune rules (bound-predicate, bound-class,
  bound subject/object authority only when `authorities_complete`, which a VoID-parsed
  descriptor never is, so it stays disabled — recall-safe). CostFed-style skew-aware
  per-(pattern, source) cardinality in `estimate_cardinality` (`selection.rs:171`).
- **Source descriptor model** — `SourceDescriptor` (`src/descriptor.rs:90`), built via
  `builder()` or **`SourceDescriptor::from_void_nt(id, nt)`** (`descriptor.rs:322`), which
  parses exactly the VoID + `scs:` N-Triples a sparq-server emits. Carries `PredPartition`,
  `ClassPartition`, `CharSet`, and the Neumann–Moerkotte `star_cardinality` /
  `star_subjects` estimates (`descriptor.rs:203-232`).
- **Join planner** — `plan_bgp(bgp, selection, descriptors, opts) -> Option<JoinTree>`
  (`src/plan.rs:180`): greedy left-deep order seeded by smallest leaf; per-join
  **bind-vs-hash** decision in `cost_join` (`plan.rs:272-314`) — `bind_cost = L·(request_cost
  + fan_out)` vs `hash_cost = R + L`; a large hash-class join (`L + R > stream_threshold`) is
  tagged `JoinAlgo::Streaming`. CS star estimate in `star_estimate` (`plan.rs:319`). Public
  surface: `JoinAlgo` (`Bind`/`Hash`/`Streaming`), `JoinNode`, `JoinTree::join_order()`,
  `PlanOptions { request_cost, stream_threshold }` (`plan.rs:71-175`).
- **Streaming join OPERATOR** — `StreamJoin` (`src/stream.rs:256`): a symmetric
  (XJoin-style) non-blocking hash join, push-based `push_left`/`push_right`, with bounded
  **operator spill** to a temp file (`StreamJoinOptions::mem_budget_tuples`,
  `SpillStore::{TempFile,Memory}`). Plus `blocking_hash_join` oracle and `run_streaming`
  driver. The correctness invariant (streamed+spilled result is multiset-equal to the
  blocking join, any interleaving / any budget) is **test-asserted in-crate only**.
- **Light pattern model** — `Bgp`, `TriplePattern`, `Term`, `Var` (`src/pattern.rs`),
  deliberately decoupled from the engine's spargebra algebra.
- **Wiring status: `sparq-fedplan` has ZERO consumers.** `grep` confirms it is referenced
  only by its own `Cargo.toml` and the workspace `Cargo.toml`; nothing in `sparq-engine` /
  `sparq-server` / `sparq-cli` depends on it. **It plans, but nothing consumes the plan and
  no fetch is ever issued.**

**B. `crates/sparq-engine/src/service.rs` + `eval_service` in `exec.rs` — the SERVICE-clause
executor (opt-in `service` feature, `Cargo.toml:58`).**

- `eval_remote(transport, endpoint, query)` (`service.rs:73`) → `Transport::fetch`
  (`service.rs:66`) → `parse_srj` (`service.rs:250`). Production transport `HttpTransport`
  is a **blocking ureq POST**, form-encoded, `Accept: application/sparql-results+json`, 30 s
  timeout (gated off wasm).
- **Bound join (brTPF/FedX-style VALUES pushdown)** — `try_bound_join_service`
  (`exec.rs:2077`): collects DISTINCT, fully-bound, **pushable** join-key tuples
  (`pushable_term` rejects bnodes/triple-terms), renders `VALUES` blocks
  (`render_values_block`) of `bind_block_size()` (default `DEFAULT_BIND_BLOCK = 50`,
  tunable via `with_service_bound_join_block_size` / `SPARQ_SERVICE_BIND_BLOCK`), sends
  `SELECT * WHERE { VALUES… inner }` per block, and **accumulates ALL blocks into `acc_rows`
  before joining**. Wired into `Join`/`LeftJoin`, symmetric on either side.
- **SSRF egress policy** — default-DENY private/internal ranges via `is_forbidden_ip`
  (`service.rs:406`) installed as ureq's resolver `EgressFilterResolver` (DNS-rebinding-safe);
  modes `DenyPrivate` / `AllowlistOnly`; scoped via `with_service_egress_allow` /
  `with_service_egress_policy`. The server installs strict `AllowlistOnly`. **This is a
  genuine differentiator** — most engines ship SSRF-open `SERVICE`.
- **SILENT** → join identity. The SRJ parser handles uri/bnode/literal/typed-literal/
  triple-term and round-trips RDF 1.2 `its:dir` direction.

**C. `crates/sparq-server/` — the SERVER side (NOT a TPF/brTPF server).** SPARQL 1.1
Protocol + Graph Store HTTP; **federation DISCOVERY descriptors** (opt-in
`federation-descriptors` feature + flag, both OFF): `void_descriptor` serves
`/.well-known/void` and `service_description` serves `GET /sparql` with no query
(`descriptors.rs:148`, `:173`). VoID rides with the `scs:` characteristic-set extension via
`Introspection::to_void_with_cs` (`sparq-introspect/src/lib.rs:1053`) — the **exact
producer** for `SourceDescriptor::from_void_nt`. No TPF/brTPF/Hydra fragment endpoint exists.

**D. Result model is MATERIALISED, not a federation-grade stream.** `QueryResult { vars,
rows: Vec<Vec<Option<Term>>> }` (`sparq-engine/src/lib.rs:597`) is fully materialised; every
public entry returns it; there is **no solution-level `Iterator`/`next`**. Internally,
`eval_graph_pattern` builds a full `Bindings` relation at every operator node (joins are
blocking; SERVICE results fetched fully then joined). The only streaming surface is
**byte-level**: `query_json_chunks_with_budget` (`lib.rs:546`) chops the serialised JSON
string into 64 KiB chunks (`JSON_CHUNK_BYTES`, `lib.rs:540`). No solution-level pipelined
iterator, no cross-source streaming.

### 3.2 The precise GAP (what the client must fill)

The planning brain, the CS cardinality model, the streaming-join operator, the per-source
SPARQL adapter + bind-join primitive, the SSRF guard, and the discovery *payload*
producer/consumer seam are **all done and reusable**. The missing piece is the **execution
glue**:

1. **Network execution over heterogeneous sources.** The only network path is `service.rs`'s
   blocking ureq POST to a full SPARQL endpoint speaking SRJ. The `Transport` trait
   (`service.rs:66`) is `fetch(endpoint, query) -> String` — it assumes "send SPARQL, get
   SRJ", so it is **not** a general source abstraction. There is no source-type-polymorphic
   adapter interface (Comunica's `IQuerySource`); no TPF/brTPF/dump/HDT adapter exists.
2. **Capability discovery + most-precise-query-per-source.** `fedplan` consumes descriptors
   but **never fetches them**, and there is **no client-side Service-Description parser**
   (the server has only a writer, `descriptors.rs`). A client must GET `/.well-known/void` +
   SD per source, hand the served N-Triples to `from_void_nt` (this seam exists), read SD to
   learn supported features, and negotiate which precise sub-query each source can answer.
3. **Streaming federation operators wired to real fetches.** `StreamJoin` is the right shape
   but `run_streaming` consumes `&[Tuple]` **slices** with a fixed schedule
   (`stream.rs:446`) — i.e. fully-known inputs, no async source. A client must feed federated
   sub-result *streams* into `StreamJoin` at their arrival rates, and bridge fedplan's light
   `Tuple`/`Term`/`Var` model to the engine's spargebra / `oxrdf::Term` / id-level
   `Bindings`.
4. **An interpreter that turns a `JoinTree` into fetches.** `JoinTree`/`JoinNode`/`JoinAlgo`
   is a static plan with **no executor**: nothing turns a `Bind` node into VALUES-pushdown, a
   `Hash` node into a local hash join, or a `Streaming` node into a fed `StreamJoin`. The plan
   speaks pattern *indices* and source *indices* with **no mapping to endpoint URLs /
   transports**.
5. **Adaptive re-planning.** Explicitly deferred and unbuilt (fedplan `README.md:100-103`,
   `lib.rs:62-68`): the plan is static; no feedback loop from observed cardinalities/rates.
   (The harder ANAPSID half.)
6. **Async / concurrency.** The SERVICE transport is blocking, one request at a time; bound-
   join blocks are issued sequentially and fully accumulated (`exec.rs:2165`). A client wants
   concurrent fan-out to N sources, backpressured streaming, and incremental emission.

---

## 4. Proposed architecture

Five layers, top to bottom. Each names the existing sparq seam it reuses.

```
                          ┌─────────────────────────────────────────────┐
   query string  ────────▶│  sparq-fedclient  (NEW, opt-in crate)        │
                          │                                              │
   (4.1) Source registry  │  SourceType: Endpoint | BrTpf | Tpf | Local  │
   + capability discovery  │   ▲ discover(): GET VoID+SD, parse → SourceDescriptor (REUSE from_void_nt)
                          │   │   + Capability (which features pushable)  │
   (4.2) Planner bridge    │  parse → light Bgp; select_sources;          │
                          │  plan_bgp → JoinTree   (REUSE sparq-fedplan)  │
   (4.3) Pushdown          │  per leaf/group: build the MOST PRECISE      │
                          │  sub-query the source can answer             │
   (4.4) Physical exec     │  JoinTree → operators:                       │
                          │   Bind→VALUES/brTPF | Hash→local | Streaming→StreamJoin (REUSE)
                          │   over an async SolutionStream                │
   (4.5) Adaptive          │  feedback: observed card/rate → re-plan       │
                          └───────────────┬──────────────────────────────┘
                                          │ local BGP eval, term interning, SSRF guard
                                          ▼
                          sparq-engine (REUSE: service.rs transport+VALUES+SSRF, local eval)
                          sparq-core    (UNCHANGED)
```

### 4.1 Source-type abstraction + capability discovery

A `SourceType` enum with one adapter per interface, each implementing a single trait — the
sparq analogue of Comunica's hypermedia "negotiate down to the most-capable handler", but
with a **fine-grained, statically-resolved** capability descriptor instead of Comunica's
coarse "service-description? / search-form? / totalItems?" runtime check.

```rust
/// What a source can do — far richer than Comunica's coarse model.
pub struct Capability {
    pub interface: Interface,            // Endpoint | BrTpf | Tpf | LocalEngine
    pub sparql_version: Option<SparqlVersion>, // from SD sd:supportedLanguage
    pub pushable_filters: FilterClass,   // which FILTER ops / expressions evaluate remotely
    pub bind_join: BindJoin,             // VALUES (endpoint) | maxMpR(n) (brTPF) | none (TPF)
    pub aggregates: bool, pub property_paths: bool,
    pub order_limit: bool,               // can ORDER BY / LIMIT be pushed?
    pub result_formats: Vec<MediaType>,  // from SD sd:resultFormat
}

pub trait FederatedSource {
    /// Discover capability + statistics (one-shot, cached).
    fn discover(&self) -> Result<(Capability, Option<SourceDescriptor>), FedError>;
    /// Stream solutions for the most-precise sub-query this source can answer.
    fn execute(&self, sub: &SubQuery) -> SolutionStream;
}
```

- **Discovery.** For an `Endpoint`: GET `/.well-known/void` and the SD document (`GET
  /sparql` with no query), parse the VoID+`scs:` N-Triples with the existing
  **`SourceDescriptor::from_void_nt`** seam (`descriptor.rs:322`) — *the producer/consumer
  match already exists end-to-end* (server `to_void_with_cs` → client `from_void_nt`). Parse
  SD into `Capability`. **A client-side SD parser is the one genuinely new parser this layer
  needs** (the server has only the writer). For a `BrTpf`/`Tpf` source: read the
  Hydra/`hydra:totalItems` count + search template (cardinality oracle + bind-join capability
  flag). For `Local`: capability is "everything", statistics from `sparq-introspect`. When no
  descriptor is published: fall back to **ASK probes** (FedX-style) for bound patterns, and
  to per-fragment count metadata (TPF). `discover()` is cached so the hot path pays nothing.

### 4.2 Planner-to-physical-operator bridge — REUSE `sparq-fedplan`

The client **does not write a new planner.** It:

1. Lowers the parsed query's BGP(s) into fedplan's light `Bgp`/`TriplePattern`/`Term`/`Var`
   (`pattern.rs`). (Non-BGP algebra — UNION, OPTIONAL, sub-SELECT, aggregation — is handled
   by composing fedplan-planned BGP sub-results through engine operators; see §7 caveat.)
2. Builds one `SourceDescriptor` per discovered source (§4.1) and calls
   **`select_sources`** then **`plan_bgp`** (with `PlanOptions::request_cost` /
   `stream_threshold` tuned per the discovered transport — a high `request_cost` for a
   per-tuple TPF source pushes the planner toward hash/streaming).
3. Resolves the plan's **pattern indices → patterns** and **source indices → endpoint URLs /
   adapters** (the missing mapping called out in §3.2(4)).

The cost-based join order, the bind-vs-hash decision, and the CS star cardinality are
**already correct, tested, and deterministic** — the client supplies the descriptors and
consumes the `JoinTree`.

### 4.3 Capability-aware pushdown — ask each server its MOST PRECISE answerable query

For each leaf and each FedX-style **exclusive group** (a connected sub-pattern whose only
relevant source is one member), the client builds the **maximal evaluable sub-algebra** for
that source's `Capability` and pushes it:

- **Full endpoint** — decompose into exclusive groups, push each as one `SERVICE`-style
  subquery; push **projections** (only the join + output variables), **filters** the
  endpoint's `pushable_filters` cover (decomposing combined/disjunctive filters; pushing a
  conjunct only when its variables are bound in the group — the **common-variable check**
  Comunica is documented to omit), and `ORDER BY`/`LIMIT` when `order_limit`. Bind-join across
  groups via **VALUES blocks**, reusing `render_values_block` + `bind_block_size`
  (`service.rs:192`, `:107`).
- **brTPF** — bind-join with `maxMpR`-sized binding blocks (the same block primitive, but the
  block size is `Capability::bind_join`'s `maxMpR` rather than the VALUES default; use POST to
  lift the GET cap).
- **Plain TPF** — per-pattern fetch with greedy client-side join driven by the count metadata;
  no bind pushdown (capability says none), so the planner's `request_cost` is set high so it
  prefers fetching the whole (selective) fragment and hash-joining locally.
- **Local engine** — evaluate the sub-BGP directly through `sparq-engine` on the local
  `Graph` (no network; the SSRF guard is moot).

Anything a source cannot evaluate is **kept locally** (the engine evaluates the residual).
Pushdown only ever *narrows* what a source returns, so it is correctness-preserving by the
same argument the existing `try_bound_join_service` uses (VALUES inner-joins the pushed
bindings; the local join reattaches them identically).

### 4.4 Streaming federation operators — REUSE `StreamJoin` + `sparq-engine` local eval

- **`SolutionStream`** — the new async/iterator abstraction the client owns at its boundary
  (the engine stays materialised, §3.2(3)). Adapters yield `SolutionStream`s; operators
  consume and produce them. Backpressured and bounded (Rust ownership + explicit buffer
  bounds + spill — *not* a GC heap, structurally avoiding Comunica's documented heap-OOM and
  broken-backpressure bugs, issue #846/#676/#835).
- **Bind-join operator** (`JoinAlgo::Bind`) — gather a block of upstream bindings, push via
  VALUES/brTPF (§4.3), stream matches. This is exactly the existing `try_bound_join_service`
  generalised from "accumulate ALL blocks then join" to "emit per block as it returns".
- **Symmetric-hash operator** (`JoinAlgo::Hash` / `JoinAlgo::Streaming`) — feed each side's
  `SolutionStream` into **`StreamJoin`** (`stream.rs`), which already builds+probes both sides
  non-blocking and spills over-budget partitions. The single new piece is a **stream feeder**
  replacing `run_streaming`'s `&[Tuple]` slices, plus the **light-Tuple ↔ engine-`Bindings`
  bridge** (§3.2(3)) so engine-evaluated local results and remote sub-results can be joined.
- **Concurrent fan-out** — issue independent leaf/group fetches concurrently (the blocking
  ureq transport is single-shot; the client wraps it with a bounded worker pool, or adopts an
  async transport behind the same `Transport` seam). First results stream as soon as the
  earliest source responds.

### 4.5 Adaptive re-planning (the deferred ANAPSID half) — LANDED (Phase 7, `sq-ij5x`)

Last and hardest. A feedback loop: each operator reports **observed** cardinality / arrival
rate; when these diverge materially from the `JoinTree` estimate (or a source stalls), the
client re-invokes `plan_bgp` on the *unjoined remainder* with corrected leaf cardinalities,
switching a `Bind` node to `Hash`/`Streaming` (or re-ordering) for the not-yet-executed
suffix. This is opt-in and bounded (re-plan at most once per operator boundary) to avoid
thrash.

**Status: implemented** as `sparq_fedclient::adaptive::execute_adaptive_single_source`, behind
the default-OFF `fedclient-adaptive` feature (which pulls `sparq-fedplan/adaptive-replan`). The
re-plan DECISION engine — the divergence trigger, the hysteresis margin, the suffix re-ordering,
and the commutativity soundness proof — already lived in `sparq-fedplan`'s `AdaptiveExecutor`
(`sq-7s4z`); Phase 7 is the client-side execution loop that drives it with **real observed
cardinalities** (a leaf-scan phase records each leaf's true row count, then an adaptive
join-ordering phase re-plans the unjoined remainder at each boundary). Re-planning changes the
plan, never the answer — the adaptive result is multiset-equal to the static plan and to local
engine eval, verified across a genuine large-divergence switch
(`tests/adaptive_result_equals_static.rs`). The ANAPSID "adaptive operator" refinement (estimate
a leaf's cardinality from a *prefix* of its rows while still streaming it) and live source
failover remain roadmap beads under epic sq-3183.

### 4.6 How this does BETTER than Comunica (architectural predictions, not measurements)

Comunica's own authors state its goal is **"modularity, and not absolute performance"**
(ISWC 2018), and its maintainers catalogue the gaps (issue #846). The predictions below are
grounded in those *documented* gaps and must be validated head-to-head before being asserted
as fact (§7).

1. **Precise, capability-aware pushdown vs Comunica's coarse model.** Comunica's capability
   model is "service-description? / search-form? / totalItems?" and it admits FILTER/
   expression pushdown gaps and a **missing common-variable check** (#834/#609). A
   fine-grained `Capability` (§4.1) lets the client push the **maximal evaluable sub-algebra**
   per source and only when shared variables exist.
2. **Real cost-based join ordering with a true cardinality estimator.** Comunica has **no
   selectivity estimator** beyond `totalItems × selectivityModifier (default 0.0001)` — a
   magic constant, not a statistic; their own EXPLAIN shows `bindCardEst:~2` vs `cardReal:43`.
   sparq reuses **characteristic-set** star cardinality (`fedplan` `star_cardinality`) and a
   cost-based bind-vs-hash decision — and the meta-finding (Qudus et al.) is that *estimation
   accuracy* is the dominant factor in plan quality.
3. **No mediation indirection in the hot path.** Comunica broadcasts a `test`-phase estimate
   to every subscribed actor on every bus, every operator. sparq resolves operator/algorithm
   choice at plan time and runs a monomorphised Rust pipeline.
4. **Bounded-memory streaming with real backpressure** (Rust ownership + `StreamJoin` spill)
   vs Comunica's documented heap-OOM + broken backpressure.
5. **Dictionary-encoded execution** end-to-end (Comunica lists "dictionary-encoded triple
   processing not implemented") for the *local* portion.
6. **SSRF-safe `SERVICE` by default** (the engine's `EgressFilterResolver`) — most engines
   ship SSRF-open federation.

**What to KEEP from Comunica** (genuinely good design): capability negotiation as a
first-class step; the multi-dimensional join cost model (CPU + memory + **blocking** + **I/O**
— the latter two are the right shape for networked sources, and richer than fedplan's current
two-term cost); lazy incremental streaming; an EXPLAIN that surfaces chosen physical operator
+ estimated-vs-actual cardinality; adaptive deferral **when sources are genuinely unknown**
(opt-in mode, not default).

---

## 5. Crate / dependency boundaries — proving it stays OUT of core

**Proposed crate: `sparq-fedclient`** (new workspace member, `publish = false`).

```
sparq-fedclient  (NEW; feature `fedclient`, OFF by default)
   ├── depends on  sparq-fedplan   (feature `fedplan`)   — planner + StreamJoin (REUSE)
   ├── depends on  sparq-engine    (feature `service`)   — SERVICE transport + VALUES + SSRF + local eval (REUSE)
   ├── depends on  sparq-introspect (optional)           — local-source stats
   └── depends on  oxrdf / spargebra (already in tree)   — term + algebra types

   does NOT touch:  sparq-core      (unchanged — no network, no async, no fed types)
   the arrow into sparq-engine is one-way: engine NEVER depends on fedclient.
```

Boundary proofs (all enforceable in CI):

- **`sparq-core` gains zero dependents.** The federation client never appears in
  `sparq-core/Cargo.toml` (it can't — core is the leaf). A `cargo tree -p sparq-core` shows no
  fedclient edge.
- **`sparq-engine` is unchanged.** No `sparq-fedclient` entry in `sparq-engine/Cargo.toml`;
  the dependency arrow points *into* the engine. The default engine build (`default =
  ["parallel","regex","digest"]`, `Cargo.toml:32`) and the WASM artifact are byte-identical
  with or without the new crate — guaranteed because nothing in the default graph references
  it.
- **Feature-gated OFF by default**, exactly like `fedplan` (`fedplan = []`, off) and
  `service` (`service = [...]`, off): a build that does not enable `fedclient` compiles
  nothing of it. CI builds the crate in both feature states (off → empty; on → full) as the
  `fedplan`/`service` crates already do.
- **Mirrors existing precedent**: `sparq-fedplan`, `sparq-canon`, `sparq-prov`, `sparq-mpc`,
  `sparq-zk` are all standalone opt-in members; `sparq-fedclient` is one more.

---

## 6. Phased build plan — each phase a crisp future-bead deliverable + its test

Each phase is a small, independently-reviewable slice (smallest context-independent
deliverable), with build+clippy+tests green in both feature states before PR. Each becomes a
bead under epic **sq-3183**.

- **Phase 0 — crate skeleton + boundary CI.** Create `sparq-fedclient` (empty behind feature
  `fedclient`, OFF), wire into the workspace, add the dependency edges (fedplan + engine
  `service`). **Test:** `cargo build`/`clippy`/`test` green with the feature off (empty crate)
  and on (skeleton); a CI check asserts `sparq-core` and `sparq-engine` have **no** edge to
  `sparq-fedclient` (`cargo tree` grep). *Deliverable: the boundary, proven, before any logic.*

- **Phase 1 — discovery client + SD parser.** Fetch `/.well-known/void` + SD per endpoint
  source; parse VoID+`scs:` via the existing `from_void_nt`; write the new **client-side SD
  parser** → `Capability`; ASK-probe fallback. **Test:** against a local sparq-server with
  `--federation-descriptors`, `discover()` returns a `SourceDescriptor` round-tripping the
  server's `to_void_with_cs` output and a `Capability` listing the advertised
  languages/formats; an endpoint with no descriptors falls back to ASK and still produces a
  usable (coarser) `Capability`.

- **Phase 2 — `SourceType`/`FederatedSource` abstraction + the Endpoint adapter.** The trait
  (§4.1) plus a `Endpoint` adapter that wraps the engine's existing SRJ transport behind
  `execute(&SubQuery) -> SolutionStream`. **Test:** an `Endpoint::execute` of a simple BGP
  against a loopback sparq-server returns the same solutions as the engine evaluating it
  locally; SSRF guard rejects a private-IP endpoint by default.

- **Phase 3 — planner bridge + plan interpreter (single-source, no streaming yet).** Lower a
  query BGP to fedplan's `Bgp`; run `select_sources` + `plan_bgp`; map plan indices →
  adapters; walk the `JoinTree` materialising each node (correctness first, streaming later).
  **Test:** a 3-pattern chained BGP over one endpoint yields results equal to the engine's
  local evaluation of the same data; `join_order()` matches the planner's choice.

- **Phase 4 — capability-aware pushdown + exclusive groups + VALUES bind-join.** Build the
  maximal pushable sub-algebra per group (projection + common-variable-checked filters);
  bind-join across groups via the reused `render_values_block`. **Test:** a federated query
  over two endpoints returns the complete, correct result; an instrumented transport asserts
  the pushed sub-queries carry the expected `VALUES` blocks and projected variables, and that
  a filter is pushed only when its variables are bound in the group (the common-variable
  check).

- **Phase 5 — streaming operators (`SolutionStream` + StreamJoin feeder + bind-join
  streaming).** Replace materialisation with the `SolutionStream`; feed sides into
  `StreamJoin`; emit bind-join results per block; concurrent fan-out. **Test:** results are
  emitted before all inputs are exhausted (the non-blocking property, mirroring fedplan's
  `emits_before_inputs_exhausted`); the streamed federated result is **multiset-equal** to the
  Phase-3 materialised result for any source-arrival interleaving (the `StreamJoin` invariant,
  extended to the fed feeder).

- **Phase 6 — brTPF + TPF adapters.** Add the bind-restricted (`maxMpR`) and plain-TPF
  adapters and their count-metadata cardinality. **Test:** against a fixture brTPF/TPF server,
  a query returns the complete result; the brTPF adapter issues `maxMpR`-bounded binding
  blocks and the plain-TPF adapter falls back to greedy count-driven client-side joins.

- **Phase 7 — adaptive re-planning (opt-in).** Feedback loop: observed cardinality/rate →
  re-`plan_bgp` the unjoined remainder; switch algorithm/order for the suffix; bounded to
  avoid thrash. **Test:** a query whose true cardinality diverges >10× from the descriptor
  estimate produces a different (cheaper) suffix plan than the static plan, and the same
  (correct) result multiset; re-planning fires at most once per operator boundary.

---

## 7. Honest risks & hard parts (no over-claim)

- **Pushdown correctness is the sharpest edge.** Pushing a filter/projection/ORDER/LIMIT a
  source evaluates with *different* semantics (numeric coercion, collation, language-tag/
  datatype handling, `NaN`, timezone) than local evaluation can silently change results. The
  safe rule: push only the sub-algebra whose remote semantics are provably identical to local;
  when in doubt, **keep it local**. The existing `try_bound_join_service` already takes this
  posture (it falls back to verbatim forwarding on any non-pushable shape — blank node, triple
  term, variable endpoint, unbound key). The client must keep that discipline as the surface
  grows, and the common-variable check must be exact (push a conjunct only when all its
  variables are bound in the group).

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
  is workload-dependent and **must not be over-promised**.

- **The engine boundary is materialised; the client owns the stream.** `sparq-engine` returns
  a fully-materialised `QueryResult` and evaluates operators blocking (§3.1.D). This design
  does **not** re-architect the engine into a global async stream; the client streams at *its*
  boundary (adapter outputs + federation operators), and local sub-BGP evaluation through the
  engine remains a materialised call. That is a deliberate, honest limit: end-to-end
  solution-level laziness *through* the engine is out of scope and would be a much larger
  change.

- **Async/runtime choice is a real decision.** The engine's transport is blocking ureq; true
  concurrent fan-out wants either a bounded blocking worker pool or an async runtime behind the
  `Transport` seam. Introducing an async runtime is a dependency-weight and complexity cost
  that must stay **inside the opt-in crate** (never leaking into core/engine). The first
  delivery can use a bounded thread pool over the existing blocking transport to avoid pulling
  an async runtime into the tree prematurely.

- **Adaptive re-planning can thrash or regress.** Re-planning on noisy early estimates can pick
  a worse suffix; it is opt-in, bounded (at most once per operator boundary), and must be
  measured before being recommended. ANAPSID-style adaptivity genuinely wins only when
  cardinalities are *unknowable* up front — where descriptors exist and are accurate, the
  static cost-based plan is simpler and at least as good.

- **The "better than Comunica" claims are predictions, not measurements.** Every advantage in
  §4.6 is an *architectural* prediction grounded in Comunica's own documented gaps. Comunica is
  battle-tested across many real interfaces and has a large SPARQL-1.1 operator surface; "we
  have a better cost model" is only an advantage once `sparq-fedclient` reaches comparable
  interface + SPARQL-1.1 completeness, and it must be validated head-to-head (FedBench / WatDiv
  / LDBC) before being stated as fact. No timings appear in this record; the in-crate
  `StreamJoin` correctness invariant is test-asserted, not independently audited.

---

## 8. References (primary)

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

### sparq code grounding (this branch's base)

`crates/sparq-fedplan/src/{lib.rs,selection.rs,plan.rs,descriptor.rs,stream.rs,pattern.rs}`;
`crates/sparq-engine/src/service.rs` + `exec.rs` (`eval_service` 1976, `try_bound_join_service`
2077); `crates/sparq-engine/src/lib.rs` (`QueryResult` 597, `query_json_chunks_with_budget`
546); `crates/sparq-engine/Cargo.toml` (`service` 58); `crates/sparq-server/src/descriptors.rs`;
`crates/sparq-introspect/src/lib.rs` (`to_void_with_cs` 1053); and `research/
feature-research-{federation,hartig}.md`.

[OPUS-4.8] — flagged for Fable re-review when available.
