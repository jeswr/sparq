# Coverage & benchmark expansion — audit + design (2026-06-14)

Design record from a read-only audit workflow (6 auditors: benchmark inventory, well-known
suites, dashboard, test-coverage llvm-cov, test-gaps). Point-in-time measurements; the
**generated** structured data (CI perf JSON, llvm-cov output) is the live source of truth —
the numbers here are an audit snapshot, not a hard-coded perf claim. Work items are tracked
in **beads** (see `bd list -l area:bench` / `-l area:test`).

<!-- [OPUS-4.8] sq-knl8 (2026-06-17): total-crate count synced 25 → 31 to track the
     current workspace (`ls crates/`); this is the TOTAL count, distinct from the
     unsafe-posture forbid/unsafe split tracked separately in the memsafety audit. -->

## 1. Benchmark coverage

### 1.1 Package coverage — already strong
Of 31 crates, ~20 have a perf bench (registered in `bench/benchmarks.toml` or an `examples/`
harness). Genuine gaps / actions:

- **sparq-vectors** — has `tests/throughput.rs` (HNSW build + exact-vs-HNSW query) but it is
  NOT registered. Add a `vectors-throughput` registry entry
  (`cargo test -p sparq-vectors --release --test throughput -- --ignored --nocapture`).
- **sparq-serve / sparq-server** — only the research `bench/serve` loadgen *spike*. Promote
  ONE deterministic micro-metric to per-commit ci-bench: arc-swap publish latency /
  generation-ring rebuild cost (serve), and an in-process req/s on the count-all query (server).
- **sparq-shacl** — no bench. Add `examples/bench_shacl.rs` (`shacl-validate-bench`):
  Core node/property constraints + one `sh:sparql` constraint over the synthetic social graph;
  emit per-constraint-class validation latency (min-of-K).
- **sparq-py** — add a pytest-timed FFI-overhead micro-bench (`tests/bench_py.py`).
- **sparq-wasm** — tracked only by the `wasm_bundle_bytes` SIZE gate; add a node/wasmtime
  query micro-bench emitting `wasm_query_us` (keep the size gate).
- **sparq-nlq / sparq-mpc** — defer (LLM-dominated / crypto-scaffold); first new bench when
  the local hot path / crypto lands. Capture as P3/P4 beads.

### 1.2 Operator coverage — the one real engine-bench gap
Today ci-bench exercises: 1-pattern scan, star-3, chain-2, triangle (cyclic), FILTER(num>),
COUNT(*), OPTIONAL. Design: a SINGLE **operator-coverage** `.rq` suite under
`bench/operators/queries/`, driven by the EXISTING `sparq-cli bench <data> <fmt> <dir> <iters>
<count|materialize|json>` harness over the synthetic dump, one ci-bench latency line per
operator family so a regression in one operator is visible. Cover the families the engine
implements and the zk-trace claims: BGP shapes, all join shapes, UNION, OPTIONAL/!bound,
MINUS, FILTER (numeric/string/regex/IN/EXISTS), BIND, VALUES, aggregates (COUNT/SUM/AVG/
GROUP_CONCAT/SAMPLE + GROUP BY + HAVING), DISTINCT/REDUCED, ORDER BY + LIMIT/OFFSET, property
paths (`*`/`+`/`?`/seq/alt/inv/negated), subqueries, ASK/CONSTRUCT/DESCRIBE, SERVICE (local).

### 1.3 Well-known suites — tiered (CI vs EC2/nightly)
Recommended order: **WatDiv → SP2Bench → BSBM → DBPSB → LUBM**. [SONNET-4.6]
FedBench and LargeRDFBench remain roadmap-only until dedicated adapters, local fixtures,
and reproducible result oracles cover their multi-endpoint workloads. [SONNET-4.6] The
current `federation-fedshop` suite is a smaller, local FedShop-shaped comparison and does
not claim coverage of either suite.

| suite | per-commit feasibility | data | queries |
|---|---|---|---|
| **SP2Bench** | **now** (smallest, deterministic, operator-focused) | `sp2b_gen -t 250000` (build-once-cache) or tiny fallback | 17 canonical Q1-Q12+Qa/Qb/Qc committed verbatim |
| WatDiv | needs-work (C++ gen + libboost) | `watdiv -d model SF=1` ~100k | 20 basic-testing (L/S/F/C) pre-instantiated + committed |
| BSBM | needs-work (Java bsbmtools) | `Generator -pc 300` ~100k | Explore mix committed |
| DBPSB/FEASIBLE | needs-work (fetch+cache) | pinned DBpedia Databus slice (sha256), ~500k-1M cut | curated 10-15 FEASIBLE/DBPSB `.rq` |
| LUBM | needs-work (Java UBA + RDF/XML→NT) | `uba.jar -univ 1 -seed 0` ~100k | 14 Q; split extensional vs **entailed** (run after `sparq-cli reason`) |

Per-commit tier commits the **queries** + a gen/fetch script (cache the generator binary /
pinned data); per-commit data stays ~100-250k triples & hermetic. Full scales (10M-1B) →
`bench-ec2.yml` / nightly with per-query timeout + result-size assertion vs reference numbers
and the stored QLever baselines (`bench/qlever-baselines.md`). LUBM Q4/Q6/Q8/Q10/Q13 require
OWL/RDFS reasoning — run the closure first; clearly label reasoning-dependent queries.

### 1.4 Dashboard (prettier, latest-commit summary table)
**Approach: custom index, NOT a separate site.** `github-action-benchmark` only writes its
default `index.html` when one does NOT exist (`fs.stat` early-return in write.ts) but
OVERWRITES `data.js` (`window.BENCHMARK_DATA`) every run. So commit ONE custom page to
`benchmark-data:dev/bench/` and the action preserves it forever while refreshing the data.

Files (on the `benchmark-data` branch): `index.html` (loads `data.js` then Chart.js v2.9.2
CDN then `dashboard.js`), `dashboard.js` (~150 LOC vanilla: latest-commit summary table +
grouped trend charts; `series=window.BENCHMARK_DATA.entries["sparq engine"]`,
`latest=series.at(-1)`, `bestOf(metric)=min over all entries`, `Δ=(latest-best)/best`),
`dashboard.css` (~80 LOC, eye-js/Jelly style, light+auto-dark, sticky header, tabular-nums,
green/red Δ pills). Install via a self-healing "seed dashboard if `dashboard.js` absent" step
in `bench.yml` (mirrors the orphan-branch bootstrap) — preferred over a manual script.
**Summary table** (top of page): one row per `latest.benches` metric, grouped by family
(regex `^q\d+_(.+)_(count|materialize|json)_us$`; memory/size + pipeline buckets), columns
Metric / Latest / Unit / Δ-vs-best (colored pill).

## 2. Test coverage

### 2.1 State — already well-tested
Real llvm-cov (cargo-llvm-cov 0.8.7, per-crate): 14/23 crates ≥~85% lines. Strong adversarial
estates: sparq-zk 90.8% (forge-and-verify per soundness gate + differential prove→verify→
cleartext fuzzer), sparq-reason 83% (incremental==from-scratch closure property tests + EYE
differential), sparq-engine 80.75% (208 tests), introspect 96.6%, sim 96.9%.

**Honest ceiling:** 100% line/region coverage is neither achievable nor desirable here —
GPU kernels need hardware, MPC malicious-security is deferred-by-design, the NLQ live-LLM and
WASM-in-browser paths aren't exercised offline. The goal is a **ratcheted floor that only
rises** + a **per-crate test-presence gate**, not a literal 100%.

### 2.2 Coverage artifacts / fixes
- **sparq-zk-compose/e2e.rs** — was broken (stale vs 9-arg `verify_manifest` +
  `ProofManifest.derivation_steps`), which broke the whole-workspace test build and disabled
  18 `#[ignore]` bb e2e tests. **FIXED 2026-06-14** (commit reconciling xxg/ayv) — coverage
  for this crate will rise once measured with the e2e suite compiling.
- **sparq-cli** shows 0% (subprocess `CARGO_BIN_EXE` artifact, not a real gap) but most
  subcommands genuinely lack tests — refactor command bodies into testable fns OR add
  per-subcommand `assert_cmd` golden/exit-code tests.
- **sparq-core dict-spill** — `dictspill.rs`/`extsort.rs` are 0% in the default run (feature
  off → 80% with `--features dict-spill`). CI coverage must run with the feature on.

### 2.3 Coverage gate design
Two new `ci.yml` jobs mirroring the conformance-ratchet idiom (checked-in threshold that may
only rise):
1. **Line-coverage ratchet** via `cargo-llvm-cov` with a committed per-crate floor JSON.
2. **Per-crate test-presence gate** — fails if a crate drops below its recorded test count /
   loses its integration-test dir (catches whole under-tested crates, not just %).

KEY CONSTRAINT: the heaviest tests are NOT plain `cargo test` — the two W3C suites run as the
`sparq-conformance` / `sparq-inference-conformance` BINARIES (`cargo run`), and SHACL/reason
`explain_*`/EYE/rdf-turtle suites SKIP when fixtures aren't fetched. The gate MUST fetch
fixtures + include the conformance binaries in the measurement, else the number is
misleadingly low and flaky.

### 2.4 Prioritised test gaps
- **P0 result-format serializer oracle** (sparq-server XML/CSV/TSV): nothing round-trips OUR
  serialized output through a reference parser (conformance only parses W3C expecteds). Build
  a `QueryResult → our XML/CSV/TSV → oxigraph sparesults → result-set` equality oracle over
  the existing random-data generator. (model: chunked-vs-serial parser oracle)
- **P0 per-builtin error-path table** (sparq-engine): every `Function` variant is dispatched
  but only happy paths pinned; add a variant-indexed table test (one row per builtin ×
  {valid, type-error, boundary} → `Value::Error`). (model: forge_gates gate map +
  relational_type_error_semantics)
- **P0 stand up the coverage gate** (§2.3) WITH fixtures + conformance binaries included.
- **P1 UPDATE SILENT/non-SILENT outcome table + request-level atomicity rollback**
  (sparq-engine). (model: WAL all-or-nothing tests)
- **P1 RDF 1.2 triple terms in UPDATE, CONSTRUCT templates, and the PARALLEL Turtle chunk path**
  (engine+core): SELECT side strong; ingest/template/serialize + quoted-triple-at-chunk-
  boundary are gaps.
- **P1 MPC adversarial-share negative suite + "no fake crypto" stub gate** (sparq-mpc): assert
  tampered shares are detected and every `NotYetImplemented` stays a typed error.
- **P2 GeoSPARQL OGC compliance ratchet + DE-9IM differential vs the `geo` crate**.
- **P2 HDT load==N-Triples-load differential + truncated-archive rejection oracle**.
- **P2 RSP-QL streamed-equals-batch window oracle**.
- **P2 SHACL property-path coverage (path.rs 36%) + W3C SHACL suite ratchet**.
- **P3 WASM headless (wasm-bindgen-test) + `cargo tree --target wasm32` dep-graph guard**
  (flate2/zstd/rayon/sparq-parse must be absent); sparq-py result-parity + panic-surface tests.
- **P3 GPU CPU-vs-GPU differential oracle** (skipped when no device).
- **P3 full-text brute-force inverted-index oracle; reason always-on per-regime
  non-entailment assertions**.

(Models cited above are existing in-repo patterns — reuse them, don't invent new harness shapes.)
