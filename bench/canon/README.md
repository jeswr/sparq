<!-- [FABLE-5] sq-hmd7l.16 — RDFC-1.0 comparative canonicalization panel. -->
# RDFC-1.0 same-box panel — sparq-canon vs rdf-canonize (JS) / rdf-canon (Rust)

W3C **RDF Dataset Canonicalization** (RDFC-1.0) compared on two axes:
conformance parity (byte-identical canonical N-Quads) and **poison-graph
DoS-resistance** under a HARD per-graph wall-clock cap. Registered as
`canon-bench` in [`bench/benchmarks.toml`](../benchmarks.toml). The oracle is
the vendored W3C rdf-canon suite snapshot
(`crates/sparq-canon/tests/rdf-canon-testdata/`, see its PROVENANCE.md).

## The invariant: byte-identical canonical N-Quads BEFORE timing

Per engine, PHASE A canonicalizes every **non-pathological** eval fixture
(the 61 sane evals, incl. the SHA-384-parameterized case) and requires the
output to equal the vendored expected file **byte-for-byte**. A mismatch is
red: that engine's timing is skipped and the panel exits non-zero. Only the
pathological fixtures are allowed to produce non-`ok` outcomes — there they
are the *result* (PHASE B), never a harness failure:

| outcome | meaning |
|---|---|
| `ok` | canonicalized under the cap, byte-exact vs the W3C expected output |
| `guard` | engine refused fail-closed (its own complexity limit) — bounded, DoS-resistant |
| `capped` | blew the HARD wall-clock cap (`CANON_CAP_S`, default 10 s) — the DoS exposure |
| `wrong` | answered under the cap with non-matching bytes (soundness failure) |
| `accepted` | computed a must-fail (`RDFC10NegativeEvalTest`) case at default limits |

Poison fixtures: the suite's `poison – evil` evals (computable given defined
limits) + the negative 10-node blank-node clique, plus generated `gen-clique`
n-cliques (deterministic, never committed) to scale the sweep.

## Columns and honesty notes

| column | what it is | posture |
|---|---|---|
| sparq | `sparq-canon` public API via `crates/sparq-canon/examples/canon_bench.rs` | as shipped (HNDQ call limit) |
| rdf-canonize | digitalbazaar JS reference — the **independent implementation** | parity/timing at `maxWorkFactor=3` (its raised conformance posture); poison rows at BOTH `@default` and `@wf3` |
| rdf-canon | zkp-ld/yamdan Rust crate, driven directly | as shipped (HNDQ call limit) |

- **sparq-canon delegates its RDFC-1.0 algorithm to the `rdf-canon` crate** —
  at the default matching pin the rdf-canon column measures sparq's
  oxrdf-0.3↔0.2 bridge + guard configuration, *not* an independent
  implementation (override `CANON_RDF_CANON_VERSION` to compare a newer
  upstream). Cross-implementation confirmation comes from the JS column.
- **rdf-canonize's shipped default guard (`maxWorkFactor=1`) fail-closes on 18
  of the 64 approved eval cases** (15 sane + the 3 poison evals — measured by
  this panel), so its parity/timing posture is raised and every poison row
  carries an explicit posture suffix. sparq's shipped default passes all 64
  while still refusing the negative clique bounded — the panel records both
  trade-offs rather than declaring a single winner.
- Timings: `wall_us` includes process spawn (node startup dominates the JS
  column); `canon_us` is the engine-reported in-process time. Work-box numbers
  are NON-canonical (`bench/CATALOG.md` QUIET-BOX); nothing is committed.

## Run it

```sh
bash bench/canon/run.sh --smoke   # sparq only: 61-fixture byte-equality gate +
                                  # the vendored negative poison fixture under the cap

# full panel (peers are gather-time installs, never committed deps):
npm install rdf-canonize --prefix /tmp/canon-js
CANON_NODE_MODULES=/tmp/canon-js CANON_RDF_CANON_BIN=auto \
CANON_JSON_OUT=bench/competitor-results/canon-$(date -u +%Y%m%dT%H%M%SZ).json \
bash bench/canon/run.sh
```

Tunables: `CANON_CAP_S` (hard cap, s), `CANON_ITERS`, `CANON_CLIQUE_SIZES`,
`CANON_JS_MAX_WORK_FACTOR`, `CANON_RDF_CANON_VERSION`, `CANON_BENCH_BIN`.
Results land git-ignored under `bench/competitor-results/`. First-read gap
record: `research/gap-canon-2026-07.md`. Conformance itself is CI-gated by the
crate suite (`cargo test -p sparq-canon`); this panel adds the cross-engine and
DoS axes.
