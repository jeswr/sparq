<!-- [FABLE-5] sq-98w7z.4 (gh-2766, parent epic sq-98w7z / gh-2579). -->
# bench/pgo — rustc PGO (+ optional BOLT) evaluation for `sparq-cli` / `sparq-server`

**Evaluation harness only.** Nothing here changes the shipped release profile,
`dist.yml`, or any artifact — a follow-up bead decides adoption from the
canonical-box numbers plus dist-pipeline complexity.

## Why

The release profile is already saturated on the classic levers (fat LTO,
`codegen-units = 1`, `panic = "abort"`), and `-Ctarget-cpu` tiers measured zero
uplift on the canonical boxes (`research/hw-bench-results.md`). Profile feedback
— rustc PGO (`-Cprofile-generate` / `-Cprofile-use`), optionally stacked with
llvm-BOLT — is the remaining unexplored codegen lever
(`research/dependency-bottleneck-analysis-2026-07.md`, row 4). This harness
answers *"is it worth it?"* with a reproducible instrument → train → use A/B.

## Run

```sh
bench/pgo/run.sh            # all phases; each is cached/idempotent, so re-runs resume
bench/pgo/run.sh clean      # remove all scratch (three release target dirs — real disk)
bench/pgo/bolt.sh           # optional; self-gates on the >= 3% PGO verdict in the report
```

Phases (`run.sh <phase>...` to run a subset): `corpora`, `build-baseline`,
`build-instr`, `train`, `merge`, `build-pgo`, `measure`, `report`. Knobs
(`ITERS`, `PGO_SCRATCH`, `PGO_INGEST_SF`, `PGO_SKIP_SERVE=1`, …) are documented
in the `run.sh` header.

## Method

| stage | what | how |
|---|---|---|
| baseline | the shipped profile, untouched | `cargo build --release` into its own target dir |
| instrument | same profile + `-Cprofile-generate` | explicit `--target` keeps build scripts uninstrumented |
| train | real workloads, instrumented binaries | watdiv/sp2b/bsbm query mixes (`sparq-cli bench`), a decompress+parse+index `ingest full` leg, and the **`sparq-server` binary** on loopback HTTP (`serve_driver.py`; graceful `SIGTERM` so profiles flush) |
| merge | `llvm-profdata merge` | the rustup `llvm-tools` component (LLVM-matched) |
| use | same profile + `-Cprofile-use` | third target dir |
| measure | identical workloads on baseline vs PGO | queries: engine-internal min-of-`ITERS` (contention-robust); ingest: best wall-clock of `INGEST_ITERS`; serve: best-of-batches req/s + p50/p99 |
| report | `report.py` → `REPORT.md` + `summary.json` | per-query delta table, per-suite + overall geomean, **hard correctness differential** (every variant must carry every baseline suite with an identical query-name set, and row counts + serve response bytes must be identical — exit 1 and no `summary.json` otherwise; tested by `test_report.py`), and the BOLT verdict |

`bolt.sh` (optional) re-links the PGO build with `--emit-relocs`, profiles it
with `perf record` (LBR when available, `perf2bolt -nl` fallback), runs
`perf2bolt` + `llvm-bolt` (ext-tsp block reorder, function reorder + splitting,
ICF), then measures through the *same* `run.sh measure-variant` code path so the
`pgo-bolt` column is directly comparable. It refuses to run unless the PGO
geomean meets the bead's 3 % gate (`--force` to explore anyway) and skips
cleanly (exit 2) where `llvm-bolt`/`perf` are unavailable.

## Honesty rules

- **Work-box numbers are NON-canonical.** `report.py` stamps host/CPU/load into
  the report and labels it; adoption claims gate on the canonical quiet-box
  re-measure bead. Never copy numbers from a shared box into docs.
- The correctness differential is a gate: a PGO/BOLT build that changes any
  query's row count or the serve byte total fails the report, as does a variant
  with missing/extra suites or queries (no silent intersection); a failed report
  removes `summary.json` so `bolt.sh` can never gate on stale numbers.
- Corpora are the pinned deterministic generators already in-tree
  (`bench/{watdiv,sp2b,bsbm}/gen.sh`); this harness adds no new datasets.

## Disk hygiene

Everything regenerable lives under `PGO_SCRATCH` (default `/tmp/sparq-pgo`):
three (four with BOLT) release target dirs, profraw/profdata, perf data,
results, and the suite corpora caches (`WATDIV_CACHE`/`SP2B_CACHE`/`BSBM_CACHE`
are routed under the scratch root unless the caller sets them). `run.sh` aborts
below `PGO_MIN_FREE_GB` (default 15 GB) free and `run.sh clean` removes it all.
On boxes where `/tmp` is a small tmpfs, point `PGO_SCRATCH` at real disk.
Nothing is committed: this directory's `.gitignore` covers any locally copied
results.
