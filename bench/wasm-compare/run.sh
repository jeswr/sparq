#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.17 — wasm-compare suite entry point.
#
#   bash run.sh --bundle-only          # stage (a): DETERMINISTIC bundle bytes only
#   bash run.sh                        # bundle bytes + Node-runtime latency comparison
#   bash run.sh --browser              # + in-browser (headless Chromium) comparison
#   bash run.sh --quick                # smoke tier for the latency stages
#   bash run.sh --corpus sp2b|watdiv   # OPT-IN (sq-hmd7l.40): well-known-suite corpus
#                                      # at its native per-commit tier, gated on the
#                                      # suite's own expected-rows.tsv (default
#                                      # workload unchanged when the flag is absent)
#
# Stage (a) — bundle.mjs — is deterministic (pinned immutable npm artifact vs
# the shipped sparq bundle) and needs no quiet box. Stage (b) — the
# cross-library latency comparison layered on the sq-3ul2n.1 browser harness
# (browser/compare.mjs) — is ADVISORY / NON-CANONICAL wherever it runs.
#
# Competitor npm packages are GATHER-ONLY (never committed): compare.mjs
# skips missing libraries WITH NOTICE; install them in browser/ first — the
# exact pinned command is printed by any skip (see compare-workload.mjs).
#
# This suite deliberately does NOT touch scripts/ci-bench.sh or
# scripts/introspect-wasm-size.sh (their wasm_bundle_bytes gate stanzas are
# the pre-bindgen ratchet; this compares SHIPPED artifacts).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BUNDLE_ONLY=0
BROWSER=0
QUICK=()
CORPUS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --bundle-only) BUNDLE_ONLY=1 ;;
    --browser) BROWSER=1 ;;
    --quick) QUICK=(--quick) ;;
    --corpus)
      shift
      CORPUS=(--corpus "${1:?error: --corpus requires a value (sp2b|watdiv)}")
      ;;
    --corpus=*) CORPUS=(--corpus "${1#*=}") ;;
    -h|--help)
      sed -n '2,21p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1' (see --help)" >&2
      exit 2
      ;;
  esac
  shift
done

node "$HERE/bundle.mjs"

if [ "$BUNDLE_ONLY" -eq 1 ]; then
  exit 0
fi

RUNTIME=node
if [ "$BROWSER" -eq 1 ]; then
  RUNTIME=node,chromium
fi
node "$HERE/browser/compare.mjs" --runtime "$RUNTIME" ${QUICK[@]:+"${QUICK[@]}"} ${CORPUS[@]:+"${CORPUS[@]}"}
