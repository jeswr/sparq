#!/usr/bin/env bash
# [OPUS-5] sq-i6du2.8 (epic sq-i6du2, #1613) — ODRE decision-agreement lane (ODRL).
# 🤖 SPARQ agent.
#
# The vehicle for the odrl-policy-bridge paper's specified-but-never-run §5.3 comparative
# decision-agreement study (research/ac-query-benchmark.md §4.2, disposition 2): run the
# benchmark's constraint-rich ODRL corpora (U3 financial + U4 research consortium) through
# BOTH sparq's ODRL evaluation path and the pinned ODRE reference implementation
# (arXiv:2409.17602), and diff the decisions.
#
# Pipeline (three stages, each refusing to fabricate):
#   1. exporter/   (Rust)   generate the U3/U4 ODRL corpora and record SPARQ's OWN decision
#                           per case, from the real sparq_policy evaluate path.
#   2. odre_adapter.py      run every case through pyodre under three input encodings
#                           (standard / odre-native / projected). Not importable => the
#                           whole lane reports SKIPPED, never a fabricated decision.
#   3. agreement.py         classify every divergence against known-divergences.json,
#                           validate the report against agreement-report.schema.json, and
#                           GATE: an unclassified divergence exits non-zero.
#
# HONESTY: this lane produces NO timing at all — its contract is a decision ledger, and
# the design record scopes ODRE to decision agreement first, timing second (§2.5). PASS
# means "every comparable case agreed, or every difference matched a sourced ledger
# entry"; it is NOT a correctness, conformance or security claim about either system, and
# the report says so in its own `claims` block. A run whose comparable set never exercised
# ODRE's constraint evaluation is reported as WEAK EVIDENCE by the report itself.
#
# Usage: bench/ac/odre/run.sh [--smoke] [--sf N] [--setup] [--self-test]  (run from anywhere)
#   --smoke      the per-commit tier: fixed seed, capped case count. Default when no args.
#   --sf N       scale factor for a nightly/EC2 tier (larger corpus, same protocol).
#   --setup      the ONE networked step: install the pinned ODRE reference implementation.
#                Every other invocation is offline.
#   --self-test  run the classifier's anti-vacuity self-test only (no corpus, no ODRE).
# Env knobs:
#   ODRE_PYTHON   python interpreter with pyodre installed (default: python3)
#   ODRE_OUT      output directory for the run artifacts (default: bench/ac/odre/out)
# Exit: 0 = report complete (or honestly skipped) with every divergence classified;
#       non-zero = unclassified divergence, incomplete report, or a stage failure.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
HERE="$ROOT/bench/ac/odre"
EXPORTER="$HERE/exporter"
PY="${ODRE_PYTHON:-python3}"
OUT="${ODRE_OUT:-$HERE/out}"

# rdflib resolves SPARQL results through hashed containers, and pyodre's `_action` takes
# an arbitrary FIRST row from that unordered set — so a run is only reproducible with the
# hash seed pinned. This pins the symptom, not the cause: see the
# `odre-arbitrary-action-selection` entry in known-divergences.json.
export PYTHONHASHSEED=0

SMOKE_ARGS=(--smoke)
SETUP=0
SELF_TEST=0
ARGS=()
for a in "$@"; do
  case "$a" in
    --setup) SETUP=1 ;;
    --self-test) SELF_TEST=1 ;;
    *) ARGS+=("$a") ;;
  esac
done
if [ ${#ARGS[@]} -gt 0 ]; then
  SMOKE_ARGS=("${ARGS[@]}")
fi

# ── --setup: the one networked step ───────────────────────────────────────────────────
if [ "$SETUP" = 1 ]; then
  echo "[odre] installing the pinned ODRE reference implementation (NETWORK REQUIRED)…" >&2
  echo "[odre] pins live in $HERE/requirements.txt; nothing is vendored into the repo." >&2
  "$PY" -m pip install -r "$HERE/requirements.txt"
  "$PY" - <<'PYEOF'
from importlib.metadata import version
print("[odre] installed pyodre %s" % version("pyodre"))
PYEOF
  echo "[odre] setup done — every later run is offline." >&2
  exit 0
fi

# ── --self-test: prove the gate is non-vacuous, no corpus needed ──────────────────────
if [ "$SELF_TEST" = 1 ]; then
  exec "$PY" "$HERE/agreement.py" --self-test
fi

mkdir -p "$OUT"

# ── 1. the sparq side ─────────────────────────────────────────────────────────────────
echo "[odre] building the sparq-side exporter (links sparq-policy — the system under test)…" >&2
cd "$EXPORTER"
cargo build --release --locked >/dev/null 2>&1 || {
  echo "[odre] --locked build failed; retrying without --locked (lockfile may need refresh)…" >&2
  cargo build --release >/dev/null 2>&1
}
BIN="$EXPORTER/target/release/ac-odre-export"
if [ ! -x "$BIN" ]; then
  echo "[odre] ERROR: ac-odre-export not found at $BIN after build" >&2
  exit 1
fi

echo "[odre] exporting the U3/U4 ODRL corpora + sparq's own decisions…" >&2
"$BIN" "${SMOKE_ARGS[@]}" --out "$OUT/cases.json"

# ── 2. the ODRE side ──────────────────────────────────────────────────────────────────
# The adapter itself decides present-vs-absent and reports honestly either way; a
# non-zero exit here is a real failure, not a missing dependency.
"$PY" "$HERE/odre_adapter.py" --cases "$OUT/cases.json" --out "$OUT/odre-decisions.json"

# ── 3. classify, validate, gate ───────────────────────────────────────────────────────
# The self-test runs on every invocation: it costs milliseconds and it is what stops the
# lane from reporting green with a gate that cannot fail.
"$PY" "$HERE/agreement.py" --self-test

set +e
"$PY" "$HERE/agreement.py" \
  --cases "$OUT/cases.json" \
  --odre "$OUT/odre-decisions.json" \
  --out "$OUT/agreement-report.json"
RC=$?
set -e

echo "" >&2
echo "[odre] artifacts in $OUT (git-ignored): cases.json, odre-decisions.json, agreement-report.json" >&2
case "$RC" in
  0) echo "[odre] OK: agreement report complete; every divergence classified." >&2 ;;
  3) echo "[odre] FAILED: UNCLASSIFIED DIVERGENCE — classify it in known-divergences.json (with a source) or fix the cause." >&2 ;;
  5) echo "[odre] FAILED: the produced report is not schema-complete (harness bug); nothing published." >&2 ;;
  *) echo "[odre] FAILED: agreement stage exited $RC." >&2 ;;
esac
exit "$RC"
