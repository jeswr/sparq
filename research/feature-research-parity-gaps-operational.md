# Competitive parity gaps — the OPERATIONAL / admin surface

<!-- [OPUS-4.8] design-for-review: vendor-parity lens, operational/admin slice. -->

Research record (design-for-review). Lens: **competitive parity gaps** — features mature
RDF/SPARQL engines ship that sparq lacks. This doc is the **operational / server-admin
slice**: transactions over the wire, query-plan introspection for tooling, result caching,
stored queries, anytime/partial results, columnar export. It deliberately does **not**
re-cover the *data/expressivity* parity gaps — those were surveyed in
[`research/feature-research-broad-sparql-vendors.md`](feature-research-broad-sparql-vendors.md)
(sq-3183), and **most of that doc's candidates have since shipped** (see §0). My job is to
verify the current ground truth and find what is *still* a genuine gap.

Surveyed engines: Oxigraph, Apache Jena/Fuseki, Eclipse RDF4J, Ontotext GraphDB, Stardog,
OpenLink Virtuoso, QLever, Amazon Neptune, Blazegraph.

---

## 0. Ground truth — what sparq ALREADY has (verified against `origin/main`)

I read the crates and the prior research before proposing anything. The 2026-06 vendor-gap
doc (sq-3183) proposed ~22 candidates; checking the current tree, **the large majority are
now implemented** — so they are NOT gaps and I do not re-propose them:

| Prior candidate (sq-3183) | Status now | Evidence |
|---|---|---|
| RDF serializer matrix (Turtle/TriG/N-Quads writers) | **Implemented** | `crates/sparq-engine/src/serialize.rs` — `write_turtle`, `write_turtle_pretty`, `write_nquads`; CLI `dump` subcommand |
| RDFC-1.0 canonicalization public API | **Implemented** | `crates/sparq-canon` (own crate, lifted from `sparq-zk`) |
| Window functions / ranking | **Implemented** | `crates/sparq-engine/src/window.rs` (sq-83l0: windowed aggregates + `ROWS`/`RANGE` frames) |
| SHACL-AF rules (`sh:rule`) | **Implemented** | sq-1m0n / sq-mk9n — node-expr algebra, `sh:expression`, function registry in `sparq-shacl` |
| JSON-LD parse | **Implemented** | sq-dvyi / sq-gg3j — engine-side JSON-LD parse + `@list` collapsing |
| Graph-analytics algorithms | **Implemented** | `crates/sparq-algos` — PageRank, degree centrality, WCC, label-propagation |
| PROV-O provenance | **Implemented** | `crates/sparq-prov` |
| SPARQL Service Description endpoint | **Implemented** | `crates/sparq-server/src/descriptors.rs` (sq-optl) — `sd:Service`, GET-with-no-query |
| VoID dataset descriptor | **Implemented** | `descriptors.rs` — `GET /.well-known/void` |
| MVCC / snapshot-isolation transactions (engine) | **Implemented (engine-level)** | `crates/sparq-engine/src/txn.rs` (sq-it1x) — `TransactionManager`, SI + first-committer-wins OCC, ACID doc — **but not exposed over HTTP** (see §2.1) |
| Query EXPLAIN / EXPLAIN ANALYZE | **Implemented (text only)** | `crates/sparq-engine/src/explain.rs` — **text output, no machine-readable plan** (see §2.2) |
| Query timeouts / concurrency cap / body limit | **Implemented** | `sparq-server` `--query-timeout`, `--update-where-timeout`, `--max-concurrent` (429), `--max-body-bytes`, `--header-read-timeout` (slow-loris guard) |
| Result formats JSON/XML/CSV/TSV (SPARQL 1.1/1.2) | **Implemented** | server content negotiation (`negotiate.rs`, `results.rs`) |
| Prometheus `/metrics`, health probes, audit log | **Implemented** | `metrics.rs`, `health_probe.rs`, `audit.rs`, `access_audit.rs` |
| Custom scalar functions, prepared queries, query budgets | **Implemented** | `FunctionRegistry`, `PreparedQuery`, `QueryBudget` |
| Horizontal scaling / HA / clustering | **Designed (PROPOSED ADR, awaiting sign-off)** | `research/adr-horizontal-scaling.md` — pod-partitioned, generation-shipping; **not a fresh gap, do not re-propose** |
| Response-bytes result cache | **Designed (RECOMMEND) but NOT implemented** | `research/concurrent-serving.md` §5 verdict 2; the *epoch invalidation primitives* exist in `sparq-serve`/`http.rs`, the cache itself does not (see §2.3) |

Net: sparq is now at or beyond table-stakes on **data formats, expressivity, reasoning,
descriptors, and single-request operational hygiene**. The remaining parity gaps are
concentrated in **protocol-level operational affordances** that the *client/tooling*
ecosystem expects from a "real" endpoint, plus two designed-but-unbuilt items.

A note on the brief's premise: the brief lists "transactions/isolation, EXPLAIN/plan
visualisation, named-graph/dataset management, bulk-load+export, query timeouts/quotas" as
candidate gaps. **Several of those are already done** — engine MVCC transactions, text
EXPLAIN/ANALYZE, named-graph CLEAR/DROP/CREATE management, the `ingest`/`build`/`save`/`dump`
CLI tooling, and request timeouts/concurrency caps all exist. The *genuine* residue is
narrower and is what this doc proposes. Backup/restore/PITR is explicitly declared
**out of engine scope** in the code ("the storage/backup tier must handle per the
retention-erasure runbook" — `sparq-cli/src/main.rs`, `sparq-server/src/http.rs`); I treat
that as a deliberate boundary, not a gap, and reflect it in §2.4.

---

## 1. The external landscape (operational/admin features, by engine)

| Engine | Operational features relevant to this lens | Ref |
|---|---|---|
| **RDF4J / Fuseki** | RDF4J Server REST API adds a **transaction resource** beyond the W3C protocol: `POST .../transactions` → a txn URL, then `PUT`/`DELETE` for operations/commit/rollback — multi-request ACID over HTTP. RDF4J also defines a **binary tuple-result format** (`application/x-binary-rdf-results-table`) with id-dedup for low parse overhead. | [RDF4J REST API](http://docs.rdf4j.org/rest-api/), [RDF4J binary format](https://rdf4j.org/documentation/reference/rdf4j-binary/) |
| **Stardog** | **Stored Query Service**: queries stored by name (CLI/Java/HTTP API), reusable as building blocks and **callable from inside another query via `SERVICE`**; shareable with per-DB permissions. Machine-readable **query plan** (JSON profiler output) drives plan-tree visualisers. | [Stored Query Service](https://docs.stardog.com/query-stardog/stored-query-service), [Stored Queries admin](https://docs.stardog.com/operating-stardog/database-administration/stored-queries) |
| **GraphDB Workbench** | Browser admin console: repository management, YASGUI editor with autocomplete + **saved query tabs**, and **Query monitoring** (list + interrupt long-running queries). | [GraphDB Workbench guide](https://graphdb.ontotext.com/documentation/7.2/standard/workbench-user-guide.html) |
| **Virtuoso** | **Anytime Queries**: a per-request `timeout=<ms>` (floored by a server `MaxQueryExecutionTime`) that returns **meaningful partial results** instead of failing, with `X-SQL-State` / `X-Exec-Milliseconds` response headers signalling incompleteness — the canonical DoS-resistant public-endpoint pattern. | [Virtuoso Anytime Queries](https://docs.openlinksw.com/virtuoso/anytimequeries/), [W3C TimeoutAndResourceConstraints](https://www.w3.org/2009/sparql/wiki/Feature_TimeoutAndResourceConstraints.html) |
| **Neptune / Blazegraph** | MVCC snapshot-isolation transactions; bulk loader; Neptune Analytics 25+ graph algos. (sparq has engine MVCC + algos already.) | [Neptune transactions](https://docs.aws.amazon.com/neptune/latest/userguide/transactions-neptune.html) |
| **QLever** | Context-sensitive **autocompletion** + live query analysis. (sparq has the sorted-permutation substrate; autocomplete already flagged in sq-3183 — out of this slice.) | [QLever](https://github.com/ad-freiburg/qlever) |
| **DuckDB / Arrow ecosystem** | **Apache Arrow** as the lingua-franca for zero-copy columnar transfer into pandas/Polars/DuckDB/BI tools. No RDF engine ships it natively yet — an open differentiator. | [Arrow result transfer](https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/) |

---

## 2. The genuine gaps, with honest trade-offs

### 2.1 Multi-request transactions over HTTP (RDF4J-style `/transactions`)

**State:** The engine MVCC machinery is *done* — `crates/sparq-engine/src/txn.rs`
(`TransactionManager`, snapshot isolation, first-committer-wins OCC, atomic commit/rollback,
WAL durability inherited from the directory-backed `Graph`). What is missing is the **HTTP
protocol surface**: every SPARQL UPDATE to the server today is its own committed transaction.
There is no way for a client to `BEGIN`, send several updates/queries that read-your-own-write
on a private fork, then `COMMIT`/`ROLLBACK` atomically.

**Why valuable:** This is *the* RDF4J differentiator and a real ETL/ingest need — batching
thousands of statements into one transaction is RDF4J's documented performance advice, and
read-your-writes-within-a-transaction is required by any client that builds a multi-step
mutation. sparq already paid the hard part (isolation correctness); only the wire binding is
left.

**Trade-off / risk:** Stateful sessions break the stateless-server assumption the
horizontal-scaling ADR leans on (a txn pins a writer fork to one node) — so this must be
single-node-scoped (or sticky-routed) and the SD/docs must say so. Transaction lifetime caps
+ abandoned-txn GC are mandatory (a held write fork blocks the single sequenced writer).
Fits `sparq-server` behind the existing `txn` cargo feature.

**Opt-in:** Yes — gate on the engine's `txn` feature; default server build unaffected.

### 2.2 Machine-readable query plan (EXPLAIN → JSON / Graphviz / Mermaid)

**State:** `explain.rs` produces a **human-readable text** plan and an `EXPLAIN ANALYZE`
execution trace. There is **no structured (serde-serialisable) plan tree** — so no tool can
consume it, and there is no plan-visualisation surface for the GUI.

**Why valuable:** Stardog/GraphDB both expose a JSON plan that drives a tree visualiser; this
is the standard query-debugging UX. For sparq specifically, a structured plan is the natural
substrate for **(a)** the in-development GUI's query-plan panel (maintainer direction: GUI as
an embedded-engine app), and **(b)** agent/LLM consumption — the NL→SPARQL loop (`sparq-nlq`)
could read a structured plan to self-correct. The text already encodes join order, strategy,
and cardinality estimates; this is a refactor to emit a typed `PlanNode` tree + a `serde`
JSON projection (and a trivial Graphviz/Mermaid renderer), not new analysis.

**Trade-off:** Keep the text output as the default (stability); the JSON is additive. The
risk is the plan schema becoming a de-facto API — version it explicitly as unstable.

**Opt-in:** Partially — the typed tree lives in `sparq-engine` (small); JSON/dot rendering
can sit behind a `serde`/`explain-json` feature.

### 2.3 Response-bytes result cache (designed, not built)

**State:** `research/concurrent-serving.md` §5 **RECOMMENDS** a canonicalised exact-match
response-bytes cache (keyed on query-hash × visibility-scope × per-pod epoch vector), and the
**epoch invalidation primitives already exist** (`sparq-serve::PodEpochs`,
`http.rs::touched_pods` / per-graph epoch tracking). The cache *itself* is unimplemented.

**Why valuable:** The cited production reality is ~69% exact-duplicate SPARQL traffic
(Bonifati et al., PVLDB 2017, via the lit review) and the cache-hit path is the only route to
the Mreq/s regime for a read-mostly Solid deployment. Virtuoso/DBpedia and WDQS both run a
result cache (or HTTP-layer cache) in front of the engine; sparq has the design and the
invalidation substrate but not the cache.

**Trade-off:** Invalidation is the hard part and the design already chose the safe answer
(epoch validation, not precise dependency tracking — WDQS rejected the latter as infeasible).
Memory budget + admission policy needed. This is *executing an existing recommended design*,
not new research — lower risk than the others.

**Opt-in:** Yes — a `sparq-serve` cache layer, off by default (a flag/feature), since the
all-distinct adversary case must pay nothing.

### 2.4 Backup / restore / point-in-time — CONFIRM the boundary (likely NOT a gap)

**State:** The code **explicitly declares** physical backup out of engine scope
(`sparq-cli`, `sparq-server/http.rs`: "the storage/backup tier must handle per the
retention-erasure runbook"). sparq *does* have logical equivalents: the `save`/`compact`
CLI, directory-backed persistence with WAL, time-travel `?generation` pinning (system-time
snapshots), and immutable shippable generations.

**Why this is a *decision*, not a build:** A consistent online snapshot ("hot backup to a
single file you can restore") is a recognised table-stakes feature (RDF4J/GraphDB/Neptune all
have it) and sparq's immutable-generation model makes a **consistent on-disk snapshot export +
import** cheap to add — distinct from "leave it to the storage tier" because that answer only
covers cold/offline copies, not an online consistent dump. The maintainer should decide
whether this is in-scope or stays a storage-tier concern.

**Opt-in:** N/A (CLI subcommand if in scope).

### 2.5 Anytime / partial-result queries with incompleteness signalling

**State:** sparq has cooperative **query budgets** (deadline + row cap) that **abort** with a
`503` timeout / `413` row-cap — it never returns *partial* results. Virtuoso's Anytime model
returns the best-effort partial answer plus headers marking it incomplete.

**Why valuable:** For a *public* read-only endpoint this is the gentler DoS posture — a
runaway query yields a truncated answer instead of an error, which is more useful to clients
and is the documented Virtuoso/DBpedia pattern. There is an open W3C `sparql-dev` thread (#51)
on standardising partial-result signalling, so the header convention is not yet settled.

**Trade-off:** Partial results have **no completeness guarantee** and can silently mislead —
this must be explicit opt-in per request, clearly flagged in the response (a header +,
ideally, a marker in the result body), never the default. The engine's executor is pull-based
with budget checkpoints, so "emit what you have at the deadline" is feasible for SELECT but
semantically murky for aggregates/ORDER BY (a partial aggregate is wrong, not just
incomplete) — scope to the streamable operators first.

**Opt-in:** Yes — per-request flag, off by default.

### 2.6 Stored / saved query registry (named queries, `SERVICE`-callable)

**State:** sparq has **ephemeral** prepared queries (`PreparedQuery`, server `Prepared`) but
no **persisted, named** query store. Stardog's Stored Query Service lets a query be stored by
name, managed via API, shared, and invoked from another query via `SERVICE`.

**Why valuable:** Named reusable query building-blocks are a real productivity feature for
templated/app workloads (which is exactly the Solid/pod profile — repeated parameterised
queries). It also pairs naturally with the result cache (a stored query is a stable cache
key) and the GUI (saved query tabs, à la GraphDB Workbench). Lower urgency than 2.1–2.3.

**Trade-off:** Persistence + access-control of the query store is new surface; the
`SERVICE <stored:name>` invocation needs a resolver hook in the SERVICE path. Keep v1 to
store/list/run by name; defer the `SERVICE`-callable form.

**Opt-in:** Yes — a server-side store behind a feature; the engine is untouched for v1.

### 2.7 Apache Arrow / columnar result export

**State:** None. sparq executes on columnar `DataChunk`s of `u64` ids internally, so emitting
Arrow is a natural projection of data already in columnar form (flagged but not built in
sq-3183 #16).

**Why valuable:** Arrow is the zero-copy bridge into pandas/Polars/DuckDB/BI — **no RDF
engine ships it natively**, so it is a genuine differentiator that fits the GenAI/analytics
direction (feed query results straight into a dataframe/ML pipeline without a CSV round-trip).
Pairs with the Python binding (`sparq-py`) where the payoff is largest.

**Trade-off:** RDF term → Arrow type mapping is lossy/awkward (IRIs and typed literals are not
native Arrow types — you either widen to strings or carry a side-channel for datatypes).
Lower priority; design the schema carefully before committing.

**Opt-in:** Yes — a `sparq-arrow` crate or an `arrow` feature on the serialization surface.

---

## 3. Recommendation

Ranked by **value ÷ effort** against operational table-stakes, honesty-checked against what
is already built or designed:

1. **HTTP multi-request transactions (§2.1)** — highest leverage: the hard correctness work
   (engine MVCC) is *done*, only the wire binding is missing, and it is a concrete RDF4J
   parity gap with a real ETL use-case. Single-node-scoped, behind the `txn` feature.
2. **Machine-readable EXPLAIN (§2.2)** — directly serves two maintainer directions (the GUI
   plan panel + agent/LLM self-correction) and is a refactor of existing analysis, not new
   work.
3. **Response-bytes result cache (§2.3)** — *executes an already-RECOMMENDED design* with the
   invalidation substrate already in place; the throughput story for the Solid profile.
4. **Anytime / partial results (§2.5)** — the right DoS posture for a public endpoint;
   medium effort, must be opt-in + clearly flagged.
5. **Stored query registry (§2.6)** and **Arrow export (§2.7)** — solid, lower-urgency
   differentiators; do after 1–4.
6. **Backup/restore (§2.4)** — *decision first*, not a build: confirm whether an online
   consistent snapshot export belongs in the engine or stays a storage-tier concern.

**Cross-cutting:** items touching the HTTP surface (2.1, 2.3, 2.5, 2.6) all land in
`sparq-server`/`sparq-serve` — the **contended surface** (one server-touching branch at a
time per the charter) — so they should be *sequenced*, not parallelised, and each must update
the `skills/http-server/SKILL.md` and the status-contract doc.

---

## 4. Phased plan (each phase = a future bead)

1. **Phase 1 — HTTP transaction endpoint (§2.1).** Wire `TransactionManager` to an RDF4J-style
   `/transactions` resource: `BEGIN` → txn id, queued query/update on the fork, `COMMIT`/
   `ROLLBACK`; lifetime cap + abandoned-txn GC; single-node-scoped; `txn` feature; update the
   status-contract + `skills/http-server/SKILL.md`.
2. **Phase 2 — typed plan tree + JSON projection (§2.2).** Refactor `explain.rs` to build a
   typed `PlanNode` tree; add a `serde` JSON projection + a Graphviz/Mermaid renderer behind
   an `explain-json` feature; keep text as default. (Unblocks the GUI plan panel.)
3. **Phase 3 — response-bytes result cache (§2.3).** Implement the `concurrent-serving.md` §5
   cache in `sparq-serve` over the existing `PodEpochs` invalidation; single-flight dedup;
   memory budget + admission; off by default. (Cite no canonical perf numbers — the work-box
   is non-canonical.)
4. **Phase 4 — anytime/partial results (§2.5).** Per-request opt-in flag; emit best-effort
   partial SELECT results at the deadline with an explicit incompleteness header/marker;
   scope to streamable operators; refuse silently-wrong partials for blocking aggregates.
5. **Phase 5 — stored query registry (§2.6).** Server-side store/list/run-by-name with
   access control; defer `SERVICE <stored:name>` to a follow-up.
6. **Phase 6 — Arrow columnar export (§2.7).** `sparq-arrow` projection of `DataChunk`s;
   careful RDF-term→Arrow datatype schema; wire into `sparq-py` first.
7. **Phase 0 (decision, blocks nothing) — backup/restore scope (§2.4).** Maintainer decides
   whether an online consistent snapshot export/import is in engine scope or stays
   storage-tier; if in-scope, a CLI `snapshot`/`restore` subcommand over immutable
   generations.

---

## 5. Open questions for the maintainer

1. **Transactions over HTTP (2.1):** acceptable to scope to single-node / sticky-routed given
   the horizontal-scaling ADR's stateless assumption? Or keep transactions engine-only?
2. **Backup/restore (2.4):** is an online consistent snapshot export in engine scope, or
   firmly a storage-tier concern (as the current code comments assert)?
3. **Anytime/partial (2.5):** desired at all, given the honesty risk of silently-incomplete
   answers? If yes, what header/marker convention (W3C `sparql-dev` #51 is unsettled)?
4. **Plan JSON (2.2):** is the GUI plan panel a near-term GUI deliverable (which would raise
   this to #1), or later?
5. **Stored queries (2.6):** worth it before the GUI exists, or fold into the GUI's
   saved-query tabs?

---

## 6. Honesty notes

- Every "implemented" claim in §0 is grounded in a named file/crate I read on `origin/main`
  (commit `3adfaca4`); every "gap" was negatively verified (grep + read) on the same tree.
- No performance numbers are asserted here; the §2.3 throughput motivation cites the existing
  lit review's external figure (Bonifati et al.), not any work-box measurement (work-box
  timings are non-canonical).
- These are *parity / operational* features; none touch the ZK/MPC estate, so no privacy
  claims are made. The result cache's visibility-scope keying must preserve the existing
  access-control contract — that is a correctness obligation noted for the implementer, not a
  privacy guarantee.

## Sources

- [RDF4J Server REST API (transactions)](http://docs.rdf4j.org/rest-api/),
  [RDF4J Binary RDF Format](https://rdf4j.org/documentation/reference/rdf4j-binary/)
- [Stardog Stored Query Service](https://docs.stardog.com/query-stardog/stored-query-service),
  [Stardog stored-queries admin](https://docs.stardog.com/operating-stardog/database-administration/stored-queries)
- [GraphDB Workbench user guide](https://graphdb.ontotext.com/documentation/7.2/standard/workbench-user-guide.html)
- [Virtuoso Anytime Queries](https://docs.openlinksw.com/virtuoso/anytimequeries/),
  [W3C TimeoutAndResourceConstraints](https://www.w3.org/2009/sparql/wiki/Feature_TimeoutAndResourceConstraints.html),
  [w3c/sparql-dev #51 partial results](https://github.com/w3c/sparql-dev/issues/51)
- [Neptune transactions](https://docs.aws.amazon.com/neptune/latest/userguide/transactions-neptune.html)
- [Apache Arrow result transfer](https://arrow.apache.org/blog/2025/01/10/arrow-result-transfer/)
- Internal: `research/feature-research-broad-sparql-vendors.md`,
  `research/concurrent-serving.md`, `research/concurrent-serving-litreview-B-mqo-caching.md`,
  `research/adr-horizontal-scaling.md`,
  `crates/sparq-engine/src/{txn.rs,explain.rs,serialize.rs,window.rs}`,
  `crates/sparq-serve/src/epoch.rs`, `crates/sparq-server/src/{descriptors.rs,http.rs,main.rs}`,
  `crates/{sparq-algos,sparq-canon,sparq-prov}`.
