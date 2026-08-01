#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.24 — RIF conformance pass-rate runner (bench registry id
# `rif-conformance`). CONFORMANCE-ONLY: the RIF axis verdict is NOT-COMPARABLE (no
# runnable open-source RIF peer exists to race — fuxi dead, Jena dropped RIF, RIFle
# unmaintained; RDFox consumes datalog, not RIF), so this harness emits a W3C
# test-suite pass-rate and NO wall-clock. Perf of RIF-translated rules is the
# materialization axis (sq-hmd7l.7), never this one. See research/gap-rif-2026-07.md.
#
# ACCEPTANCE (sq-hmd7l.24):
#   bash bench/rif/run.sh --smoke   # exit 0, asserting pinned pass/fail on the
#                                   # vendored subset (bench/rif/expected.tsv)
#
# Usage:
#   bench/rif/run.sh --smoke   # self-asserting vendored subset (exit 1 on drift)
#   bench/rif/run.sh           # full-suite per-dialect pass-rate over the FETCHED
#                              # W3C archive (scripts/fetch-inference-suites.sh);
#                              # graceful skip (exit 0) when the archive is absent;
#                              # tees to bench/rif/results.tsv (gitignored)
#
# The `--features rif-xml` flag is REQUIRED: RIF is an opt-in surface — the default
# sparq-reason build carries zero RIF code by design (the lean-core posture).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MODE="full"
case "${1:-}" in
  --smoke) MODE="smoke" ;;
  "") MODE="full" ;;
  *)
    echo "[rif] ERROR: unknown argument '${1:-}' — usage: bench/rif/run.sh [--smoke]" >&2
    exit 2
    ;;
esac

if [ "$MODE" = "smoke" ]; then
  exec cargo run -p sparq-reason --release --example rif_conformance \
    --features rif-xml -- --smoke
else
  # Full mode: pass-rate over the fetched W3C archive; results (pass/fail counts,
  # NOT perf numbers) land in the gitignored bench/rif/results.tsv.
  cargo run -p sparq-reason --release --example rif_conformance \
    --features rif-xml | tee "$ROOT/bench/rif/results.tsv"
fi
