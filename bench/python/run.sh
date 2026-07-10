#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.18 — python-bindings-bench runner: sparq-py vs pyoxigraph vs rdflib.
#
# Drives scripts/bench-adapters/python_rdf_adapter.py over the SP2Bench tiny tier and
# enforces the catalog invariant: NO timing row without cross-engine row-count agreement
# (the adapter's --compare stage exits 1 on any disagreement, which fails this script).
# The primary metric is BINDING OVERHEAD — sparq-py's per-query time minus the
# engine-internal time of the SAME queries on the SAME corpus from `sparq-cli bench`
# (materialize mode, matched codegen profile) — reported alongside the absolute columns.
#
# Engines are GATHER-TIME dependencies only (never committed — bench/CATALOG.md):
#   pip install pyoxigraph rdflib
#   (cd crates/sparq-py && maturin develop --profile python-release)   # import name: sparq
# An engine that is not importable is SKIPPED gracefully (bead sq-hmd7l.18); the
# comparison runs over whichever engines are present.
#
# Usage:
#   bash bench/python/run.sh --smoke     # tiny corpus, 3 queries, workload+gate only
#   bash bench/python/run.sh             # workload + floor + slope micro-benchmarks
# Env knobs:
#   PYBENCH_PYTHON  python with the engines installed (default: python3)
#   SP2B_T          corpus triple count (default 10000 — the tiny tier)
#   ITERS           warm query iterations, min-of-K (default 5)
#   QUERIES         SP2B SELECT query stems (default: q01 q02 q03a q03b q03c q05b q10)
#   CLI             sparq-cli binary for the native reference (default: auto-detect;
#                   absent => the binding-overhead column is NOT-RUN, absolutes still print)
#
# Results: bench/competitor-results/python-bindings-<engine>-<UTC>.json (git-ignored).
# Wall-clock caveat: quiet_box_sensitive — treat absolute numbers from a busy box as
# indicative only; ratios and the overhead delta are the robust read.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ADAPTER="$ROOT/scripts/bench-adapters/python_rdf_adapter.py"
PY="${PYBENCH_PYTHON:-python3}"

SMOKE=0
if [ "${1:-}" = "--smoke" ]; then SMOKE=1; fi

if [ "$SMOKE" = 1 ]; then
  T="${SP2B_T:-10000}"; ITERS=3; LOAD_ITERS=2; QUERIES="${QUERIES:-q01 q03a q10}"; MICRO=0
else
  T="${SP2B_T:-10000}"; ITERS="${ITERS:-5}"; LOAD_ITERS="${LOAD_ITERS:-3}"
  QUERIES="${QUERIES:-q01 q02 q03a q03b q03c q05b q10}"; MICRO=1
fi

echo "[pybench] corpus: SP2Bench t=$T (deterministic); queries: $QUERIES; iters=$ITERS" >&2
CORPUS="$(bash "$ROOT/bench/sp2b/gen.sh" "$T")"

SCRATCH="$(mktemp -d /tmp/pybench.XXXXXX)"
trap 'rm -rf "$SCRATCH"' EXIT
QDIR="$SCRATCH/queries"; mkdir -p "$QDIR"
for q in $QUERIES; do cp "$ROOT/bench/sp2b/queries/$q.rq" "$QDIR/"; done

OUTDIR="$ROOT/bench/competitor-results"; mkdir -p "$OUTDIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

# ---- per-engine workload gather (graceful skip when not importable) ---------------
PRESENT=()
JSONS=()
for e in sparq pyoxigraph rdflib; do
  if "$PY" -c "import $e" >/dev/null 2>&1; then
    echo "[pybench] workload: $e ..." >&2
    "$PY" "$ADAPTER" --engine "$e" --mode workload --input "$CORPUS" --format turtle \
      --queries "$QDIR" --iters "$ITERS" --load-iters "$LOAD_ITERS" \
      --json "$SCRATCH/$e.json" >/dev/null
    cp "$SCRATCH/$e.json" "$OUTDIR/python-bindings-$e-$STAMP.json"
    PRESENT+=("$e"); JSONS+=("$SCRATCH/$e.json")
  else
    echo "[pybench] SKIP $e: not importable via $PY (see install notes in this script's header)" >&2
  fi
done

if [ "${#PRESENT[@]}" = 0 ]; then
  echo "[pybench] no Python RDF engine importable — nothing to compare (graceful skip)" >&2
  exit 0
fi

# ---- native engine-internal reference (sparq-cli bench, materialize mode) ---------
CLIARG=()
CLI="${CLI:-}"
if [ -z "$CLI" ]; then
  for cand in "$ROOT/target/python-release/sparq-cli" "$ROOT/target/release/sparq-cli"; do
    if [ -x "$cand" ]; then CLI="$cand"; break; fi
  done
fi
if [ -n "$CLI" ] && [ -x "$CLI" ]; then
  echo "[pybench] native reference: $CLI bench (materialize, $ITERS iters)" >&2
  "$CLI" bench "$CORPUS" turtle "$QDIR" "$ITERS" materialize --json "$SCRATCH/cli.json" >/dev/null
  cp "$SCRATCH/cli.json" "$OUTDIR/python-bindings-sparq-cli-$STAMP.json"
  CLIARG=(--cli "$SCRATCH/cli.json")
else
  echo "[pybench] NOT-RUN native reference: no sparq-cli binary found (build with" >&2
  echo "          cargo build --profile python-release -p sparq-cli); absolutes only" >&2
fi

# ---- agreement gate + combined report (exits 1 on any row-count disagreement) -----
"$PY" "$ADAPTER" --compare "${JSONS[@]}" "${CLIARG[@]}"

# ---- binding-overhead isolation micro-benchmarks (full mode only) -----------------
if [ "$MICRO" = 1 ]; then
  for e in "${PRESENT[@]}"; do
    echo "[pybench] floor: $e ..." >&2
    "$PY" "$ADAPTER" --engine "$e" --mode floor --json "$OUTDIR/python-bindings-floor-$e-$STAMP.json"
    echo "[pybench] slope: $e ..." >&2
    "$PY" "$ADAPTER" --engine "$e" --mode slope --json "$OUTDIR/python-bindings-slope-$e-$STAMP.json"
  done
fi

echo "[pybench] done; results under bench/competitor-results/ (git-ignored)" >&2
