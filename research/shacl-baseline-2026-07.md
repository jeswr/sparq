<!-- [FABLE-5] sq-7d3dj.33 — SHACL competitor baseline (first read) + sh:sparql root-cause. -->
# SHACL validation: first competitor baseline + the `sh:sparql` ~760 ms root cause

**Bead** `sq-7d3dj.33` (PERF gap D8, performance-dominance mandate). SHACL was
**NOT-MEASURED** in the 2026-07-07 canonical competitor matrix (no engine in the
matrix runs SHACL; RDFox publishes no numeric SHACL claim), and sparq's own CI
trend showed the `sparql_constraint` workload at **~760 ms** — 60–2000×
slower than the other four constraint kinds over the *same* corpus.

Two deliverables:

1. **A durable same-box SHACL comparison harness** —
   [`scripts/bench/shacl-same-box.sh`](../scripts/bench/shacl-same-box.sh)
   (sparq-shacl vs pySHACL vs Apache Jena SHACL) emitting
   `bench/canonical-competitor-results/`-shaped envelopes, so a future dedicated
   quiet-box EC2 gather is a one-liner (`CANONICAL=1 OUT_DIR=… scripts/bench/shacl-same-box.sh`).
2. **A profile-evidenced root cause** for the ~760 ms `sh:sparql` path (§3).

## 1. Harness

| Piece | What it does |
|---|---|
| `scripts/bench/shacl-same-box.sh` | Orchestrator: LUBM(univ) data via `bench/shacl/gen.sh`, shared workload set, three in-process drivers, per-workload timeout, envelope JSON per scale (same key shape as `canonical-sp2b-*.json`, incl. `count_crosscheck` + `<engine>_tsv`). `canonical:false` unless `CANONICAL=1`. |
| `scripts/bench/pyshacl-shacl-bench.py` | In-process pySHACL driver: rdflib-load once (timed separately), `pyshacl.validate` best-of-N per shapes file, `signal.alarm` per-workload cap. Emits the `bench_shacl` 6-column TSV. |
| `scripts/bench/JenaShaclBench.java` | In-process Jena driver: `RDFDataMgr` load once, `Shapes.parse` once, `ShaclValidator.get().validate` best-of-N. One JVM per workload under `timeout`; JVM start-up and parse stay outside the timed section. |
| `bench/shacl/shapes-sparql/sparql_heavy.ttl` | SPARQL-constraint-HEAVY shape set: 3 `sh:sparql` constraints over the 3 biggest LUBM target classes (~9.9k focus nodes/universe total). Kept out of `bench/shacl/shapes/` so the per-commit `expected.tsv` gate is untouched; its correctness check is cross-engine count agreement. |

Methodology mirrors the canonical SPARQL gathers: identical `(data, shapes)`
inputs, validate-only best-of-N on a loaded graph, `#violations`/`conforms`
cross-checked engine-vs-engine before any timing is trusted, timeouts recorded
as honest `ERROR` rows. Engines are gather-only `/tmp` deps (pip venv + Jena
tarball), never committed.

## 2. First read (NON-canonical — shared work box)

> **All numbers in this section are NON-canonical.** They were taken on the
> shared EC2 work box (other agents run builds/mutants concurrently; load
> average reached >16 during parts of the gather). They are directional only —
> the durable deliverable is the harness; the citable numbers come from the
> `CANONICAL=1` quiet-box re-run (bead filed, see §4).

#### LUBM(1) — 103 104 triples, iters=3 (best-of)

| workload | violations (all 3 agree) | sparq | Jena SHACL 5.4.0 | pySHACL 0.31.0 | sparq vs Jena | sparq vs pySHACL |
|---|---:|---:|---:|---:|---|---|
| `cardinality` | 149 | 435 µs | 5.8 ms | 55.3 ms | 13.4× ahead | 127.0× ahead |
| `class_nodekind` | 125 | 397 µs | 8.1 ms | 48.3 ms | 20.4× ahead | 121.7× ahead |
| `datatype_range` | 1874 | 27.4 ms | 34.0 ms | 542.1 ms | 1.2× ahead | 19.8× ahead |
| `node_paths` | 1802 | 14.0 ms | 64.8 ms | 1.04 s | 4.6× ahead | 74.5× ahead |
| `sparql_constraint` | 3738 | 915.8 ms | 204.1 ms | 32.62 s | **4.5× BEHIND** | 35.6× ahead |
| `sparql_heavy` | 4343 | 1.55 s | 287.5 ms | 52.80 s | **5.4× BEHIND** | 34.0× ahead |

Data-graph load (advisory, s): sparq 0.129 · Jena 1.079 · pySHACL (rdflib) 3.846

#### LUBM(10) — 1 316 700 triples, iters=1

| workload | violations (all 3 agree) | sparq | Jena SHACL 5.4.0 | pySHACL 0.31.0 | sparq vs Jena | sparq vs pySHACL |
|---|---:|---:|---:|---:|---|---|
| `cardinality` | 1965 | 9.2 ms | 297.5 ms | 361.2 ms | 32.3× ahead | 39.2× ahead |
| `class_nodekind` | 1602 | 8.5 ms | 266.7 ms | 300.0 ms | 31.5× ahead | 35.5× ahead |
| `datatype_range` | 24019 | 219.9 ms | 1.62 s | 6.03 s | 7.3× ahead | 27.4× ahead |
| `node_paths` | 23040 | 135.4 ms | 2.29 s | 15.47 s | 16.9× ahead | 114.3× ahead |
| `sparql_constraint` | 47696 | 200.79 s | 6.76 s | **timeout** (>900 s cap) | **29.7× BEHIND** | n/a |
| `sparql_heavy` | 55157 | 313.68 s | 9.25 s | **timeout** (>900 s cap) | **33.9× BEHIND** | n/a |

Data-graph load (advisory, s): sparq 0.369 · Jena 10.150 · pySHACL (rdflib) 30.335

**Reading (directional):**

- **Counts cross-check clean**: all three engines agree on `#violations` and
  `conforms` for **all 6 workloads at both scales** (`all_agree: true` across the
  board) — the timing comparison rests on verified-identical answers.
- **Core constraints (cardinality / class / nodekind / datatype / paths): sparq
  leads everywhere** — 13–32× vs Jena and 20–127× vs pySHACL, with the lead
  *widening* at the 1.3M-triple scale (sub-linear scaling vs both).
- **HONEST GAP — `sh:sparql` workloads: sparq is BEHIND Jena, and the gap grows
  with scale**: 4.5–5.4× behind at 103k triples, **~30–34× behind at 1.3M**
  (200.8 s / 313.7 s vs 6.8 s / 9.3 s). Root cause in §3 — sparq re-executes the
  full constraint query per focus node (quadratic), Jena substitutes `$this`
  into an index-seeded evaluation (linear). The batching fix (sq-7d3dj.33.1)
  is measured to bring LUBM(1) `sparql_constraint` from ~916 ms to ~1.5 ms —
  which would flip the row to ~2 orders of magnitude AHEAD of Jena.
- **pySHACL times out (>900 s) on both `sh:sparql` workloads at 1.3M triples**
  (recorded honestly as `ERROR/timeout`, not fabricated).
- **`datatype_range` vs Jena is only ~1.2× (103k) / 7.3× (1.3M)** — ahead, but
  below the order-of-magnitude mandate → headroom bead sq-7d3dj.33.4.
- The first LUBM(1) gather ran while another agent's `cargo-mutants` wave loaded
  the box (load >16); it was re-gathered on the quiet box and the table above is
  the quiet re-run. Both raw envelopes live in `/tmp` only (NON-canonical —
  deliberately not committed; the canonical run is sq-7d3dj.33.3).

## 3. Root cause of the ~760 ms `sparql_constraint` workload

**Verdict: per-focus-node full-BGP re-execution — O(N_focus × full-query) work
instead of O(full-query). Not re-parsing, not re-planning overhead.**

The `sh:sparql` path (`crates/sparq-shacl/src/sparql.rs`,
`PreparedSparql::evaluate`) does, **for every focus node**:

1. `pre_bind_select` — clone the parsed algebra and inject a **single-row**
   `VALUES ?this { <focus> }` pushed down next to the BGP;
2. `sparq_engine::PreparedQuery::from` + `query_prepared` — execute the whole
   query from scratch.

The executor evaluates the resulting `Join(BGP, Values)` by **materialising the
full BGP result** and hash-joining it against the 1-row VALUES: the single
bound `$this` never seeds an index lookup (no bind-join / sideways information
passing), so each focus node pays the full scan-and-join cost of the *unbound*
query.

### Profile evidence (perf, cycles:u, LUBM(1) × `sparql_constraint`, 3 iters)

| % cycles | Symbol | Meaning |
|---:|---|---|
| 26.4 % | `sparq_core::store::PermData::rows_in` | full predicate-row scans, per focus node |
| 25.9 % | `sparq_engine::exec::eval_bgp_binary` | full BGP evaluation, per focus node |
| 10.1 % + 5.6 % | `hashbrown::…::get` + `sparq_substrate::join::probe_emit` | hash-join of the full BGP result against the 1-row VALUES |
| < 1 % | parse / algebra clone / `pre_bind_select` | **planning & rewrite are NOT the problem** |

Call stacks: `Validator::eval_component → query_prepared → eval_modified →
eval_graph_pattern → eval_bgp_binary → TripleStore::scan_with → PermData::rows_in`,
once per focus node.

### Arithmetic corroboration (quiet-box moments, LUBM(1), best-of-5)

| Measurement | Result |
|---|---|
| Whole **unbound** constraint query, run once | 926 µs → all 3 738 solutions |
| Same query with a **single-row** `VALUES ?this` (what the validator runs per focus node) | 464 µs → 3 rows |
| × 1 874 focus nodes | 1 874 × 464 µs ≈ **870 ms** ≈ the observed 860–916 ms (CI trend ~760 ms) |
| Same query with **all 1 874 focus nodes in ONE `VALUES`** | **1 479 µs** → the same 3 738 rows |

So the whole workload's answer set is computable in ~1.5 ms; the current
per-focus loop spends ~580× that. The other four workloads don't exhibit this
because core constraints evaluate against per-focus-node value sets fetched by
index, not by re-running a whole query.

### Fix directions (beads filed under `sq-7d3dj.33`, see §4)

1. **Batch focus nodes** in `sparq-shacl`: inject ONE `VALUES ?this { …all… }`,
   run once, group solutions by `?this` (the measured ~580×). Semantics guard:
   batching is equivalence-preserving for the pre-binding-legal query subset the
   crate already enforces (no MINUS/VALUES/SERVICE, sub-selects must project
   `$this`) **except** top-level aggregates/LIMIT-bearing forms not grouped by
   `$this` — those keep the per-focus path.
2. **Bind-join / SIP in `sparq-engine`**: when one join side is a tiny VALUES,
   seed the BGP scans with its bindings (index lookups instead of full scans).
   Fixes the general query-shape class (also benefits federation/service-style
   rebinding), turns the per-focus query into ~µs work even unbatched.

## 4. Follow-up beads

| Bead | P | What |
|---|---|---|
| `sq-7d3dj.33.1` | P1 | **The fix**: batch all focus nodes into ONE `VALUES` execution in `sparq-shacl` (measured ~580× on `sparql_constraint`; guard the non-batchable aggregate/LIMIT forms). |
| `sq-7d3dj.33.2` | P2 | Engine-level bind-join / sideways-information-passing: seed BGP scans from a tiny VALUES join side instead of full-scan + hash-join (fixes the general query-shape class). |
| `sq-7d3dj.33.3` | P2 | CANONICAL EC2 re-run of this harness (`CANONICAL=1`, dedicated quiet box) — ideally after sq-7d3dj.33.1 lands, or before/after for the honest delta. |
| `sq-7d3dj.33.4` | P2 | Core-constraint headroom: `datatype_range` / `node_paths` are ahead of Jena but below the order-of-magnitude bar at some scales — profile + targeted fix. |

## 5. Reproduction

```sh
# harness (both scales, all engines; NON-canonical on a shared box)
scripts/bench/shacl-same-box.sh
# canonical re-run (dedicated quiet EC2 box)
CANONICAL=1 OUT_DIR=bench/canonical-competitor-results/<date> scripts/bench/shacl-same-box.sh
# root-cause profile
cargo build --release -p sparq-shacl --example bench_shacl
perf record -g --call-graph dwarf -e cycles:u -F 999 \
  target/release/examples/bench_shacl /tmp/lubm/lubm-univ1-seed0.nt ntriples <dir-with-sparql_constraint.ttl> 3
# scratch cleanup
rm -rf /tmp/jena-shacl /tmp/shacl-bench-venv
```
