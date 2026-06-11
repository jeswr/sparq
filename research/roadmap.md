# sparq — consolidated roadmap, dependency graph & parallel-execution plan

Status: the in-memory + out-of-core engine is **SOTA-competitive-to-faster than QLever** on
compute (synthetic 10M/100M + real olympics; 2.3–20×), hardware-tuned across 4 silicon families.
16 commits banked. What follows is everything *left* — features, optimisations, and scaling —
organised into work **threads**, with an explicit map of which threads touch the same code (and
must be serialised) vs which are independent (and can run concurrently, optionally on separate
EC2 boxes).

---

## 1. The work threads

### T1 — Many-core parallelism & NUMA scaling  *(primary goal: near-linear to ~196 cores)*
Source: `research/parallelism-scaling.md`. Touches the hot path: `exec.rs`, `lib.rs`, `dict.rs`.
- **T1.0 (days, low-risk):** mimalloc/jemalloc global allocator (one line, gated off wasm);
  rayon worker pinning; `numactl --interleave=all` probe; **thread-local `LocalVocab` + deferred
  merge** (port the parse-path `ShardedDict` pattern to query time — kills the `exec.rs:57`
  serialization point).
- **T1.1 (1–2 wk):** radix-partitioned parallel hash aggregation (direct GROUP BY fix) + parallel
  hash-join build; same for `distinct`/`minus`.
- **T1.2:** NUMA-aware index placement / per-socket replication; custom pinned threadpool.
- **T1.3 (deep):** morsel-driven pipelined execution; parallelise the serial merge/outer/bind/WCOJ
  joins. The real ceiling-lifter; months.
- **Prereq:** validate on a 2-socket box (M1 cannot reveal NUMA). Needs **T8**.
- **Status 2026-06-11: measurement INFRA-BLOCKED.** Launch ladder (spot+on-demand
  c7i.48xlarge → c7i.24xlarge → c7i.4xlarge/c7g.4xlarge) all refused at `RunInstances`:
  spot SLR still missing, on-demand quota still 16 vCPU *and now 2-consumed by the prod
  t3.large*, so even 16-vCPU boxes fail. Harness is launch-ready (`hwrun/launch.sh` +
  `hwrun/remote.sh`); M1 1→8-thread fallback curve captured. See
  `research/hardware-validation-blocked.md` for verbatim errors + unblock options.
- **Status 2026-06-11 (pm, rung 5):** 1–8-thread sweep re-measured on homogeneous x86
  (r7i.2xlarge, 4 physical cores + HT): load plateaus at **1.81× @4 / 1.99× @8** —
  matches the M1 (1.82× @4), so the `merge_remap` ceiling is a **real measured
  serialization target**, not core asymmetry. NUMA + >8-thread questions still blocked.

### T2 — Wikidata ingestion under 24 min
Source: `research/wikidata-ingestion-benchmark.md`, `fast-ingestion.md`. Currently **1.28–1.30 M/s**
on the M1 (≈ QLever's server rate on a laptop). 24 min for ~9.4 B truthy ⇒ need ~6.5 M/s.
- **The one bottleneck is `Dict::merge_remap`** (profiled) — the mandatory global-id serialization
  point; 8 cores net only ~1.3 M/s vs 0.96 single-core. **This is the SAME fix as T1.0's
  thread-local-vocab/sharded-dict** → T2's core *is* a T1 deliverable.
- Also: faster decompression default (zstd over bz2, already supported), bigger external-merge
  buffers, parallel k-way merge.
- **Hard gate:** the full dump needs ~680–850 GB disk + >16 GB dict RAM — **cannot run on the M1
  at all**; needs the big instance. So T2's *validation* is hardware-bound, but its *code* is the
  merge_remap parallelisation (shared with T1).
- **Status 2026-06-11 (am): validation INFRA-BLOCKED** (same launch failures as T1 — see
  `research/hardware-validation-blocked.md` and the results section appended to
  `research/wikidata-ingestion-benchmark.md`). The timed ≥1 B-triple ingest is scripted and
  ready in `hwrun/remote.sh`; <24-min target neither met nor refuted.
- **Status 2026-06-11 (pm): partial-scale MEASURED @1 B, full target quota-blocked.**
  Sanctioned rung-5 run (on-demand r7i.2xlarge, 8 vCPU/64 GB, ~$1.50 total): **1 B real
  truthy triples gz→queryable store in 737.8 s = 1.355 M/s, 51.5 GB peak RSS, 84 GB index,
  COUNTs correct over mmap.** Measured ceiling: dict consolidation+remap is ~200 s/1 B
  *serial* even with the sharded dict ⇒ Amdahl-capped ≥31 min full-truthy on ANY core
  count until that bucket shrinks below ~153 s/1 B; load scaling plateaus at 1.8× on 4
  homogeneous x86 cores (merge_remap ceiling confirmed real, not M1 E-core artifact).
  Full-truthy <24-min validation remains quota-blocked (needs 192-core box + spill dict).
  See "Rung 5 MEASURED" in `research/wikidata-ingestion-benchmark.md` +
  `research/hardware-validation-blocked.md`.

### T3 — Tagged/inline ValueIds (u64)  *(#23 + #26; foundational)*
Inline numerics/dates/decimals into the id so range FILTER + ORDER BY skip the dict (QLever's main
remaining compute edge). Blocked today: u32 inline range is exhausted → needs **u64 Id**. Changes
the fundamental `Id` type across `dict.rs`, `exec.rs`, the store, `lib.rs`. **Memory tradeoff**
(doubles perm width → may cost bandwidth-bound scans) → must be measured both ways. *Foundational:
every hot-path edit (T1, T4) builds on the Id representation.*

### T4 — Compressed on-disk permutation blocks (PForDelta/prefix)
The one place QLever still wins: smaller on-disk index at billions (a **disk-footprint** win, not
speed — sparq already wins query speed). Touches the store/index + every scan in `exec.rs`. Risk:
decode cost in the hot loop. Source: `term-index-compression.md`, `bit-level-encoding.md`,
`dict-compression-measured.md`.

### T5 — Inference completion  *(sparq-reason; isolated crate)*
- **T5.1 (#27): DONE** — OWL-RL single-pass extension (monotone subset out of the fixpoint, ~6.5×).
- **Independent:** `sparq-reason` depends on `sparq-core` but **not** on `exec.rs` → does not
  conflict with T1/T3/T4.

### T12 — Inference FEATURE COMPLETENESS  *(sparq-reason; isolated; user-requested)*
The goal is **feature-complete inference**, especially **Notation3 toward EYE-reasoner parity**.
Current state: the N3 forward chainer executes `{ premise } => { conclusion }` rules to a fixpoint
(semi-naive, FactIndex-accelerated) with variables + formulae, and the v1 builtins are the
*comparison* family (`math:greaterThan/lessThan/notGreaterThan/notLessThan/equalTo/notEqualTo`,
`log:equalTo/notEqualTo`). The completeness work is **builtin coverage**, added batch-by-batch and
each validated against EYE's own test cases (`sparq-reason/tests/eye_cases.rs`):
- **T12.1 — `math:` functional** (`sum`/`difference`/`product`/`quotient`/`negation`/`abs`/`exponentiation`,
  rounding) over `( … )` list arguments (needs in-rule list resolution).
- **T12.2 — `string:`** (`concatenation`, `contains`, `startsWith`, `endsWith`, `matches`, `replace`,
  case, `length`), **`list:`** (`member`, `length`, `append`, `in`), and more **`log:`**
  (`includes`, `notIncludes`, `conjunction`).
- **T12.3 — full OWL-RL** rule set audit (any rules still missing vs the spec) + RDFS edge cases.
- **T12.4 — N3 syntax** completeness (quoting/paths/`@forAll`/`@forSome` as needed) + proof output.
- **Discipline:** each builtin batch ships with EYE-validated tests; differential against the prior
  closure to guarantee no regression. `sparq-reason` only — conflict-free with the engine threads.

### T6 — RDF 1.2 / SPARQL 1.2
Source: `rdf12-parser.md`, `rdf12-indexing.md`, `sparql12-engine.md`.
- **T6a — parser** (triple terms, RDF-star successor syntax): `sparq-core` parser files —
  **independent** of the exec cluster.
- **T6b — indexing** (triple terms in the store): joins the **store/id cluster** (conflicts with
  T3/T4).
- **T6c — engine** (SPARQL 1.2 algebra/eval): touches `exec.rs` (conflicts with T1).

### T7 — AI / GenAI supports  *(4 new feature-gated crates; isolated)*
Source: `genai-*.md` (5 reports). Crates: `sparq-sim` (structural IRI similarity from the 6 perms —
training-free, the novel edge), `sparq-introspect` (characteristic sets — **also feeds the planner**,
soft-links to T1/optimisation), `sparq-nlq` (NL→SPARQL retrieve-repair loop), `sparq-vectors` (mmap
embedding store + ANN). One-directional dep, zero main-path impact → **independent**. User directive:
sequence *after* inference (T5) + RDF/SPARQL 1.2 (T6) are optimised. First step: synthesise the 5
reports into one prioritised design doc.

### T8 — Measurement harness  *(sparq-bench; additive, independent)*
`sparq-bench` has **no thread-count sweep today**. Add one reporting per-subsystem parallel
*efficiency* vs `RAYON_NUM_THREADS` (load / scan / join / aggregate / serialise / infer). This is the
instrument T1 and T2 are blind without. Additive — conflicts with nothing.

### T9 — SPARQL feature gaps
- **Named graphs / GRAPH** — currently errors (`exec.rs:893`). Needs a quad store or graph-column.
- Property paths (`*`/`+`/`/`), SERVICE federation, remaining aggregates/expressions (the `M2:`
  markers at `exec.rs:2375/2640`). These touch `exec.rs` (conflict with T1).

### T10 — SPARQL 1.1/1.2 Update  *(store mutation; core cluster)*
The store is currently **immutable** (built once, queried). Update needs incremental insert/delete:
`INSERT DATA` / `DELETE DATA` / `DELETE…INSERT…WHERE` / `LOAD` / `CLEAR` / `CREATE` / `DROP` /
`COPY` / `MOVE` / `ADD`. Touches the **store + dict** (mutable permutation indexes + dict growth /
tombstones), and `exec.rs` (the update algebra; spargebra already parses Update). Architectural
decision required: mutate-in-place vs a delta/overlay layer merged at query time (the overlay
approach preserves the immutable-index fast path + the byte-identity invariant, and composes with
the out-of-core mmap store). **Conflicts with the core cluster** (T1/T3/T4 all touch store/dict).
**Prerequisite for T11's update endpoint** and a soft prerequisite for named-graph (T9) semantics.
Strongly interacts with T9 (GRAPH targets in `WITH`/`USING`/`GRAPH` update clauses) → do T9's
quad/graph-column model and T10 together.

### T11 — HTTP server, W3C-conformant  *(new `sparq-server` crate; mostly independent)*
Expose the engine over HTTP per the W3C specs:
- **SPARQL 1.1/1.2 Protocol** — `query` via GET + POST (`application/sparql-query` and
  url-encoded), `update` via POST (`application/sparql-update`); content negotiation for result
  formats (SPARQL Results **JSON/XML/CSV/TSV**, and RDF serializations for CONSTRUCT/DESCRIBE);
  correct HTTP status/error semantics; `default-graph-uri`/`named-graph-uri` params.
- **SPARQL 1.1 Graph Store HTTP Protocol** — `GET/PUT/POST/DELETE/HEAD` on graph resources
  (direct + indirect graph identification).
- Conformance: run the W3C SPARQL Protocol + GSP test suites. Async runtime (axum/hyper or similar,
  feature-gated; **not** in the wasm build).
- **Mostly independent** — a new crate wrapping the engine's public API. The **query + result-format
  + GSP-read side has no core-cluster conflicts** and can be built concurrently *now*. The
  **update + GSP-write side depends on T10**. So split: **T11a (query/protocol/formats/GSP-read,
  independent)** and **T11b (update endpoints, gated on T10)**.
- New result serializers (XML/CSV/TSV) are additive and reusable by the CLI.

---

## 2. Dependency & conflict map

Two clusters touch the same files and **must be serialised within the cluster** (one owner / one
branch at a time); everything else is **independent** and can run concurrently on its own
worktree/box.

```
 CORE HOT-PATH CLUSTER (exec.rs + dict.rs + store + lib.rs load)  — serialise internally
   T3 (u64 ids)  ──foundational──►  T1 (parallelism/NUMA)  ◄──shares merge_remap──  T2 (ingestion)
                         │                      ▲
                         └──── T4 (compressed index) ───┘   T6b/T6c, T9 also land here
   Recommended order inside the cluster:  T1.0 (+T2 core)  →  T1.1  →  T3  →  T4 / T6bc / T9 / T1.2 → T1.3

 INDEPENDENT THREADS (separate crates / additive — run fully concurrently, no merge conflicts)
   T5   inference       (sparq-reason)          ── no exec.rs / dict.rs edits
   T7   genai           (new sparq-* crates)    ── isolated; after T5+T6 per directive
   T6a  RDF1.2 parser   (sparq-core parser)     ── parser files only
   T8   scaling harness (sparq-bench)           ── additive instrument
   T11a HTTP server     (new sparq-server crate)── query/protocol/formats/GSP-read; wraps public API

 NEW STORE-MUTATION SUB-CLUSTER (joins the core cluster; do T9 quad-model + T10 together)
   T10  SPARQL Update   (store + dict + exec.rs) ──prereq──► T11b (update endpoints)
```

**Key dependency facts:**
- **T2 ⊂ T1.0** — the 24-min ingestion bottleneck (`merge_remap`) is the *same* serialization point
  as the query-time vocab fix. Do them as one piece of work; don't fork them.
- **T3 is foundational** to T4 and interacts with T1 (both edit `exec.rs`). Decide T3 early so T4 and
  T1.2/1.3 build on the final Id type — *but* T3's payoff is unproven (memory tradeoff), so gate it
  behind a measurement before committing the whole engine to u64.
- **T1, T2, T8 all need the NUMA/big box** to *measure* (not to write). T8 must land first.
- **T5, T7, T6a, T8 are conflict-free** with the core cluster → these are the threads to run **in
  parallel on cheap EC2 boxes** while the single core-cluster owner works the hot path locally.
- **T7 soft-depends on T1/optimisation** only via characteristic sets (T7's `sparq-introspect`
  doubles as a planner cardinality input) — a nice-to-have link, not a blocker.

---

## 3. Critical path & sequencing

1. **T8 first** (scaling harness) — cheap, unblocks all measurement. Do on M1 + a cheap Linux box.
2. **T1.0 + T2-core together** (allocator + thread-local vocab + sharded merge_remap) — biggest
   single scaling lever; fixes both query aggregation *and* ingestion throughput. Validate on the
   NUMA box.
3. **T1.1** (radix-partitioned operators) — direct GROUP BY/join/distinct scaling.
4. In parallel throughout: **T5** (inference) and **T8/T6a** run on their own boxes/worktrees with
   zero conflict.
5. **T3** (u64 ids) — gated on a measurement; once decided, **T4** (compressed index) follows.
6. **T1.2 → T1.3** (NUMA placement → morsel) — the deep ceiling-lifters, after T1.0/1.1 prove out.
7. **T6 (RDF/SPARQL 1.2)** then **T7 (GenAI)** — per the user's stated sequencing (after inference +
   the core engine are optimised).

---

## 4. EC2 budget-managed execution plan

Goal: run the independent threads concurrently without tripping over each other, **≤ $5/day** EC2
compute + **≤ $10 one-time** for the NUMA validation. Code isolation is via **git worktrees** (free,
local); EC2 is only for *running/measuring* Linux + many-core work.

**Cheap per-thread test boxes (the $5/day budget):**
- Use **one small Graviton/x86 box per active independent thread that needs Linux CI** — e.g.
  `c7g.large` (2 vCPU, ~$0.06/hr) or `t4g.medium` (~$0.03/hr) for inference/parser/genai test runs.
- **Terminate when idle** (each thread's box up only while actively benchmarking). 3 boxes × 4 h/day
  × $0.06 ≈ **$0.72/day** — well under $5. Even a c7g.2xlarge (8 vCPU, ~$0.24/hr) for ~8 h ≈ $1.9.
- **Network discipline:** generate datasets **on the box** (`sparq-bench dump`) — never transfer big
  data; only scp the **827 KB source tarball** in and tiny CSV results out. Egress on results is
  ~$0. This is what kept the whole hardware campaign under ~$2.
- Reuse the proven recipe (key+SG, Ubuntu 24.04 AMI, `aws-bootstrap.sh`/`hw-bench.sh`); always
  `terminate-instances` + delete key/SG at end of a session.

**The NUMA validation box (the ≤ $10 one-time):**
- Need ≥ 2 NUMA nodes to expose the dominant barrier. Target: a 2-socket EPYC (e.g.
  `m7a.48xlarge`, 192 vCPU). On-demand ~$11/hr would blow the cap in <1 h — so use a **spot
  instance** (~$3–4/hr typical) and run the **T8 scaling sweep + T1.0 A/B** in a **single ≤2 h
  session**, then terminate. That keeps it **≤ $8–10**. Capture all numbers in one run (sweep
  `RAYON_NUM_THREADS` 1→192 + `numactl` on/off) so the box is never up idle.
- A cheaper interim datapoint: memory-bandwidth scaling shows up even on a 1-socket high-core box
  (`c7g.16xlarge` spot, ~$1/hr); use it to dry-run the harness before the pricier 2-socket run.

**Concurrency rule to avoid thread collisions:** the **core hot-path cluster has exactly one
owner/branch at a time** (T1→T3→T4 serialised). The **independent threads (T5, T6a, T7, T8) each get
their own worktree + their own cheap box**, so up to ~4 threads progress simultaneously with zero
merge conflicts. Each lands via its own PR/commit; the core-cluster owner rebases as needed.

---

## 5. One-line summary per thread

| thread | what | files / crate | conflicts with | needs HW | independent? |
|---|---|---|---|---|---|
| T1 parallelism/NUMA | near-linear to 196c | exec.rs, lib.rs, dict.rs | T2,T3,T4,T6bc,T9 | yes (measure) | no — core cluster |
| T2 Wikidata 24-min | merge_remap parallel + zstd | dict.rs, lib.rs | T1 (same fix) | yes (big box) | folds into T1 |
| T3 u64 ValueIds | inline numerics/dates | dict.rs, exec.rs, store | T1,T4 | no | no — foundational |
| T4 compressed index | smaller disk @ billions | store, exec.rs | T1,T3 | partial | no — core cluster |
| T5 inference | OWL single-pass #27 + N3 | sparq-reason | — | no | **yes** |
| T6a RDF1.2 parser | triple terms | sparq-core parser | — | no | **yes** |
| T6b/c RDF/SPARQL1.2 | store + algebra | store, exec.rs | T1,T3 | no | no — core cluster |
| T7 GenAI | 4 feature-gated crates | new crates | — | no | **yes** (after T5/T6) |
| T8 scaling harness | thread-count sweep | sparq-bench | — | no | **yes** |
| T9 SPARQL gaps | GRAPH, paths, SERVICE | exec.rs | T1,T10 | no | no — core cluster |
| T10 SPARQL Update | INSERT/DELETE/LOAD/… | store, dict, exec.rs | T1,T3,T4,T9 | no | no — core cluster |
| T11a HTTP server | Protocol+formats+GSP-read | new sparq-server | — | no | **yes** |
| T11b update endpoints | Protocol update + GSP-write | sparq-server | T10 | no | gated on T10 |

---

# Roadmap extension (2026-06-10) — user-approved new threads

**Global constraint (hard rule):** every thread below is **opt-in** — a separate crate or a
non-default cargo feature, following the `sparq-reason` pattern — and must have **zero impact on
the WASM build's performance or bundle size** (and zero default-engine impact). Enforcement: the
wasm-build CI job stays mandatory, and T14 adds a tracked **bundle-size metric** to the benchmark
series so any size regression is visible per commit.

### T13 — W3C conformance suites in CI  *(correctness credibility; independent)*
Run the official `w3c/rdf-tests` suites (SPARQL 1.1 query evaluation + update + protocol; RDF 1.2
syntax) against the engine/server. Manifest-driven runner (parse `manifest.ttl`, evaluate, compare
against expected results with bag/set semantics). Report a pass-rate scoreboard in CI (informational
first, ratchet to a gate as coverage climbs). New dev-only crate `sparq-conformance`.

### T14 — RDF/JS bindings + npm  *(distribution; wasm)*
Expose sparq-wasm through the RDF/JS Dataset/Store/Query interfaces; TypeScript types; publish to
npm. Add a wasm **bundle-size tracking** metric to the benchmark CI. The natural channel for the
rdfjs ecosystem; benchmark vs Oxigraph-wasm + Comunica.

### T15 — Server production hardening  *(server; independent)*
Request timeouts + concurrency limits (tower layers), payload/result-size caps, cooperative query
cancellation (row-budget checks in the executor — the part needing engine support), structured
errors. Without this a public endpoint is one cross-product away from OOM.

### T16 — CONSTRUCT / DESCRIBE + streaming results  *(engine + server)*
An RDF-graph result API in the engine; CONSTRUCT/DESCRIBE evaluation; chunked/streaming
serialization on the server (and lazy SELECT streaming — pairs with the Tier-3 pipelined work).

### T17 — Incremental updates + durability  *(store)*
Delta-overlay updates (insert/delete sets merged at query time; periodic compaction) replacing the
O(n) rebuild; write-ahead log + recovery → sparq becomes a database, not just a query engine.
Readers keep snapshot isolation via the existing Arc-swap design.

### T18 — Incremental reasoning (DRed/counting)  *(sparq-reason)*
Maintain RDFS/OWL closures incrementally under INSERT/DELETE instead of full re-materialization
(RDFox's headline capability). Composes with T17; the materialization tests are the oracle.

### T19 — SHACL validation  *(new opt-in crate `sparq-shacl`)*
Core constraint components + targets evaluated via the query engine; W3C SHACL test suite as the
gate (fits the T13 conformance harness).

### T20 — Releases & packaging  *(infrastructure)*
crates.io publication, GitHub Releases wired to dist.yml binaries, Docker image for the server,
Homebrew formula. Versioning + changelog.

### T21 — Python bindings  *(adoption)*
pyo3 bindings + an rdflib Store backend; wheels via maturin in CI.

### T22 — EXPLAIN + observability  *(engine + server)* ✅
Query-plan introspection (EXPLAIN endpoint/CLI flag showing plan choice, cardinality estimates,
per-operator timings) + Prometheus metrics on the server.
**Done:** engine `explain()` — a planning-only dry run that replays the executor's own GOO
ordering/strategy helpers (shared functions, no logic duplication) with cardinality estimates,
filter-pushdown and binary-vs-WCOJ dispatch — and `explain_analyze()` (SELECT/ASK), which executes
under a thread-local per-operator trace (rows + wall time per `eval_graph_pattern` operator; one
flag check per operator entry when off). Server: `?explain=`/`Accept: text/x-sparq-explain` on
`/sparql`, plus a hand-rolled Prometheus `/metrics` (requests by endpoint/status, `/sparql`
latency histogram, subscription/triple gauges, update counter). See the server README.

### T23 — Subscription API (SPARQL subscriptions)  *(server; SEPA-inspired)*
Subscribe to a SPARQL query over WebSocket: initial result, then ADDED/REMOVED binding diffs after
each committed update. Model on the SEPA member submission (SPARQL 1.1 Subscribe Language /
Secure Event Protocol) — the spec lineage Blazegraph never shipped. v1: post-commit re-evaluation +
result diffing (hash rows; correct, simple) with debounce/coalescing; v2: incremental evaluation
reusing T17's deltas (only re-run when the delta's predicates intersect the query's). Adjacent to
Solid Notifications channels for that ecosystem.

### T24 — Previously-parked features, now unparked (all opt-in, wasm-neutral)
- **T24a HDT format** (`sparq-hdt`): read (and later write) HDT archives — HDT's dict+triples
  layout maps naturally onto sparq's model; big research-community win.
- **T24b GeoSPARQL** (`sparq-geo`): geometry literals, spatial functions + an R-tree index.
- **T24c RDF stream processing** (`sparq-stream`): windowed continuous queries (RSP-QL style);
  shares machinery with T23.
- **T24d GPU execution** (`sparq-gpu`): the g5 prototype — offload the embarrassingly-parallel
  inference sweep + large joins; measure PCIe-transfer break-even honestly before keeping.
  **DONE — measured-and-rejected (PARKED), 2026-06-11.** `crates/sparq-gpu` (opt-in, depended on
  by nothing, wgpu kept out of the wasm graph) landed with FILTER/hash-probe/GROUP-BY kernels,
  GPU-vs-CPU correctness tests (adapter-absent ⇒ skip) and an interleaved cpu1/cpuN/gpu-resident/
  gpu-e2e benchmark at 1M/10M/100M. Measured on M1: per-query transfer loses 9–25×; compute-light
  scans lose or tie the 8-core CPU *even resident*; hash-probe is the lone real win (1.7–3.2×
  resident, ~1.1× e2e) — not enough to pay for a residency cache + second backend. Verdict +
  re-open conditions (discrete ≥8 GB GPU, a resident-column tier landing anyway, or
  WebGPU-in-browser): `research/gpu-verdict.md`; harness stays in-tree as the re-test rig.

**Sequencing/dependencies:** T13/T15/T20/T21/T22 are independent → agent-parallel now. T14 after
the in-flight RDF-star merge (wasm API surface). T16 before T23-CONSTRUCT subscriptions. T17 → T18
→ T23-v2 chain. T24x each fully independent opt-in crates.
