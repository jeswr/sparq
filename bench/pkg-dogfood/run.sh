#!/usr/bin/env bash
# [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — one-command PKG token A/B.
# Written while Fable unavailable; flag for re-review when Fable returns.
# Spec: research/dogfooding-sparq-knowledge-graph.md §5.
#
# Builds the pkg-query helper, runs the counterbalanced read-payload A/B driver over
# the FROZEN task set (corpus sha256-verified), grades correctness PAIRED with a
# blind deterministic resolver, then emits the §5.6 verdict object. Stdlib-Python +
# cargo only. Every number it prints is runtime-only / NON-CANONICAL (never frozen
# into committed markdown). Run from anywhere.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/bench/pkg-dogfood"
cd "$ROOT"

echo "[run.sh] building pkg-query (sparq-kb --features query)…"
cargo build -p sparq-kb --features query --bin pkg-query >/dev/null 2>&1

echo "[run.sh] driving the A/B (read-payload effective tokens)…"
python3 "$HERE/run.py"   --out "$HERE/results.json"

echo "[run.sh] grading correctness (paired, deterministic, blind)…"
python3 "$HERE/grade.py" --results "$HERE/results.json" --out "$HERE/grades.json"

echo "[run.sh] stats + verdict (Wilcoxon + bootstrap + break-even, frozen PREREG bar)…"
python3 "$HERE/stats.py" --results "$HERE/results.json" --grades "$HERE/grades.json" --out "$HERE/verdict.json"

echo "[run.sh] done. Artifacts: results.json, grades.json, verdict.json (all NON-CANONICAL)."
