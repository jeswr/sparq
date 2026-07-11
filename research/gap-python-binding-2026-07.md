<!-- [FABLE-5] sq-hmd7l.18 first-read gap record. ALL numbers in this file are
NON-CANONICAL (shared aarch64 work box, frequently busy). Ratios and the
matched-config deltas are the robust reads; absolute wall-clock is indicative. -->

# Python binding overhead — sparq-py vs pyoxigraph vs rdflib (first read, 2026-07-10)

**Status:** first read complete; binding layer measured THIN, but a structural
**feature-parity gap in the shipped wheel** was found and root-caused → **P1 bead
`sq-11664`**. **Bead:** sq-hmd7l.18. **Epic:** sq-hmd7l. **Registry suite:**
`python-bindings-bench` (bench/benchmarks.toml).
**Box:** shared work box — EC2 **r7g.2xlarge** (Graviton, aarch64, 8 vCPU, 61 GiB),
NOT a quiet bench box; numbers are **non-canonical**.

## Harness + provenance

- **Harness:** `bench/python/run.sh` → `scripts/bench-adapters/python_rdf_adapter.py`
  (modes `workload` / `floor` / `slope` / `compare`); unit tests for the agreement
  gate in `scripts/bench-adapters/test_adapters.py`.
- **Git:** `6df375096` (origin/main, 2026-07-10). **Python:** 3.12.3.
- **Engines:** sparq-py `sparq-rdf 0.1.0` (this rev, `maturin develop --profile
  python-release`, pyo3 0.29 abi3); **pyoxigraph 0.5.9** (PyPI wheel, in-memory
  `Store`); **rdflib 7.6.0** (PyPI, pure Python).
- **Native reference:** `sparq-cli bench <corpus> turtle <queries> 5 materialize`
  built at the same rev with the SAME `--profile python-release` codegen
  (`materialize` = engine-internal full row materialisation in Rust, no serialisation).
- **Corpus:** SP2Bench deterministic generator, tiny tier `bench/sp2b/gen.sh 10000`
  (10 303 triples) — small so the rdflib column stays tractable.
- **Regime:** load = fresh store per iteration, min-of-3; queries = warm
  (load once), min-of-5, **every bound cell materialised into a Python object**
  for all three engines (matched work: sparq-py's `Graph.query` builds `Term`
  objects eagerly; the pyoxigraph adapter touches `sol[var]` for every var;
  rdflib rows already hold Python objects). GC paused during timing windows.
- **Row-count agreement gate:** every query's solution count agreed across
  sparq-py / pyoxigraph / rdflib / sparq-cli **before** any timing row (adapter
  `--compare`, exit 1 on disagreement). All 7 queries agreed at t=10k.
- Raw JSON envelopes (versions + env + timings): `bench/competitor-results/
  python-bindings-*-20260710T151508Z.json` (git-ignored, regenerable).

## 1. Whole-call workload (absolute columns — engine + binding combined, honest label)

`SELECT` queries, min-of-5 warm µs, rows agreed 4-way. rdflib's column is
**engine-bound, not binding-bound** (no FFI boundary) — it is the ecosystem
reference, not a binding-overhead comparison.

| query | rows | sparq-py µs | pyoxigraph µs | rdflib µs | sparq-cli engine µs |
|---|---|---|---|---|---|
| q01 | 1 | 15.1 | 67.4 | 3 803.9 | 20.5 |
| q02 | 147 | 1 092.4 | 12 416.7 | 46 139.3 | 1 384.3 |
| q03a | 846 | 1 933.5 | 12 260.1 | 327 989.6 | 188.0 |
| q03b | 9 | 1 298.8 | 5 116.2 | 475 468.8 | 13.4 |
| q03c | 0 | 1 195.3 | 6 437.9 | 304 346.7 | 11.5 |
| q05b | 155 | 1 375.2 | 50 253.1 | 4 352 068.6 | 729.5 |
| q10 | 166 | 155.3 | 939.7 | 4 762.8 | 22.6 |

Load (10 303 triples, min-of-3, fresh store each): **sparq-py 40.9 ms** ·
pyoxigraph 76.4 ms (`bulk_load`; plain `load` 80.3 ms) · rdflib 879.7 ms.

Whole-call, sparq-py is **4.1–38× faster than pyoxigraph on every query** and
42–3 300× faster than rdflib (engine-bound). But the CLI column exposes an
anomaly: q03b/q03c run **~100× slower via the wheel than engine-internal**, far
beyond any plausible boundary cost for 0–9 rows. That is NOT binding overhead —
see §3.

## 2. Binding-overhead isolation (the honest primary metric)

### 2a. Floor — calls where engine work is ~nil (fixed 8-triple graph, min over ~3k iters)

| op | sparq-py µs | pyoxigraph µs | rdflib µs | sparq-py vs pyoxigraph |
|---|---|---|---|---|
| `len(g)` (pure boundary crossing) | **0.18** | 2.06 | 0.85 (no FFI) | **11.4×** |
| `ASK` hit (boundary + query parse) | **3.03** | 23.29 | 1 392.91 | **7.7×** |
| SELECT, 0 rows | **3.52** | 24.02 | 1 282.48 | **6.8×** |
| SELECT `LIMIT 1` | **4.91** | 32.02 | 1 478.75 | **6.5×** |
| SELECT, 8 rows | **12.26** | 81.75 | 1 569.45 | **6.7×** |

Both Rust engines parse SPARQL natively, so the ASK/SELECT floors are
boundary + query-parse + ~zero eval; `len()` is the pure FFI crossing. rdflib's
~1.3–1.6 ms floor is its pure-Python SPARQL machinery (engine, not binding).

### 2b. Slope — per-row result materialisation (`SELECT ?s ?p ?o`, 64 vs 8192 triples)

Per-row cost (3 cells/row): **sparq-py ≈ 1.4–2.1 µs/row** (1 380 ns/row on the
matched-config wheel, 2 114 ns/row on the shipped wheel — the spread is box +
config variance) vs **pyoxigraph ≈ 7.3 µs/row** vs rdflib ≈ 20.7 µs/row.
Caveat stated: the slope includes each engine's per-row scan, not just the
binding — for the two Rust engines Python-object construction dominates.

### 2c. sparq-py vs its own native engine (matched-config wheel, §3) — the direct split

With the wheel built at **engine-feature parity with sparq-cli** (temporary local
probe, not shipped): binding overhead = whole-call − engine-internal:

| query | rows (cells) | overhead µs | per-cell |
|---|---|---|---|
| q01 | 1 (1) | +4.1 | call floor |
| q03c | 0 (0) | +2.1 | call floor |
| q03b | 9 (9) | +6.2 | ~0.7 µs |
| q03a | 846 (846) | +446.3 | ~0.53 µs |
| q05b | 155 (310) | +169.8 | ~0.55 µs |
| q02 | 147 (1 470) | +1 033.5 | ~0.70 µs |
| q10 | 166 (332) | +150.9 | ~0.45 µs |

Two independent instruments agree: **sparq-py's pyo3 boundary costs ~2–4 µs per
call plus ~0.5–0.9 µs per materialised result cell** (dict + `Term` object
construction). Consistent with the floor (§2a) and slope (§2b).

## 3. Structural finding — the shipped wheel is rewrite-dark (P1 `sq-11664`)

The q03b/q03c ~100× whole-call-vs-CLI anomaly was root-caused this session
(perf profiles + a plain-Rust reproducer of the byte-identical
`load_dataset`+`query` path + a feature-resolution diff):

- `sparq-cli` builds `sparq-engine` with the opt-in planner features
  **`dp-planner`, `algebra-rewrite`, `antijoin-static-decline`**
  (crates/sparq-cli/Cargo.toml:124); `sparq-server` defaults `algebra-rewrite`
  ON for the same reason (sq-7d3dj.30.13: the shipped binary should execute the
  plans the canonical benchmarks measure).
- **`sparq-py` enables none of them** — the pip wheel plans without the
  `FILTER (?p = <const>)` constant-substitution rewrite, so the q03 family takes
  the generic `bind_join` + per-row `apply_filter_scalar` scan
  (~1.2 ms) instead of the rewritten ~11 µs path. perf: the wheel burns in
  `bind_join`/`eval_compiled`/`values_equal`+malloc; the plain-Rust process
  burns only in the spargebra query parser.
- A matched-config wheel (this probe) closes q03c from 1 195 µs to **13.2 µs**
  (engine 11.1 µs) and q03b from 1 299 µs to **19.6 µs**.

**Consequence:** PyPI users currently get an engine configuration that is up to
~100× slower than sparq-cli on rewrite-dependent shapes. Fix bead **`sq-11664`
(P1)**: give the wheel the CLI's planner feature trio (leaf/binary-crate
rationale — the engine LIBRARY default stays lean), decide sparq-server's
dp-planner/antijoin parity, and add a feature-parity drift guard; then re-run
this suite.

## 4. Verdicts (fixed vocabulary, §0 of research/perf-dominance-gap-2026-07.md)

| axis | verdict | evidence |
|---|---|---|
| Binding-call floor vs pyoxigraph | **AHEAD-BUT-NOT-OOM** (6.5–11×) | §2a, five ops |
| Per-row materialisation vs pyoxigraph | **AHEAD-BUT-NOT-OOM** (~3.5–5×) | §2b (includes engine scan, stated) |
| Whole-call SP2B tiny tier vs pyoxigraph | **AHEAD-BUT-NOT-OOM → CLEARLY-AHEAD** per query (4.1–38×) | §1 (engine+binding combined, stated) |
| Load from Python vs pyoxigraph | **AHEAD-BUT-NOT-OOM** (1.9×) | §1 |
| vs rdflib (whole-call) | **CLEARLY-AHEAD** (42–3 300×) — but as a *binding* comparison **NOT-COMPARABLE** (rdflib has no FFI boundary; its cost is the pure-Python engine, honestly labelled the ecosystem reference) | §1 |
| sparq-py binding layer vs native sparq-cli | **thin — ~2–4 µs/call + ~0.5–0.9 µs/cell** (PARITY with the engine at matched config) | §2c |
| **Shipped wheel engine config vs sparq-cli** | **BEHIND (up to ~100× on rewrite-dependent queries)** — root-caused, not spun; **P1 `sq-11664`** | §3 |

Honesty notes: single box, single corpus tier, min-of-K on a busy machine —
per-query CLI-deltas at ms scale have low SNR (medians ran up to 4× the mins
during contention; one shipped-config q02 sample even beat the CLI's min).
The floor/slope instruments and the matched-config deltas, which agree with each
other, are the load-robust reads. Canonical quiet-box re-run belongs to the
epic's canonical wave.
