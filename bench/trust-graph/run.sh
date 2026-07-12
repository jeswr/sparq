#!/usr/bin/env bash
# [FABLE-5] sq-on7r4. 🤖 SPARQ agent — certification-edge closure benchmark runner.
#
# Self-contained, self-asserting entry point (mirrors bench/ac/run.sh): builds the
# standalone `trust-graph-bench` driver (bench/trust-graph — its own [workspace], like
# bench/ac + bench/parse; it depends on sparq-trust with the default-OFF `cert-graph`
# feature ON, in this detached project ONLY) and runs its fail-closed lanes:
#   - the strict-additivity ENVELOPE (load-bearing): zero (surviving) certifications ⇒
#     derive_effective_rules output byte-EQUAL to the input direct_rules;
#   - closure-overhead timing over (anchors × certifications × scope kind), each lane
#     correctness-gated (expected admitted/rejected count + verbatim anchors prefix).
#
# HONESTY: the driver prints indicative wall-clock numbers, but every such number is
# advisory + NON-CANONICAL on this shared work box (the QUIET-BOX convention in
# bench/CATALOG.md); no number is committed to markdown. The HARD, load-robust contract
# is the fail-closed envelope exit code, which this script propagates.
#
# Usage: bench/trust-graph/run.sh [--smoke] [--sf N]   (run from anywhere)
#   --smoke  quick fixed-size tier (the CI/per-commit tier).
#   --sf N   scale factor for a nightly/EC2 tier (default 1 when not --smoke).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

echo "[trust-graph] building the standalone trust-graph-bench driver (detached workspace; cert-graph ON here only)…" >&2
# Standalone workspace root — resolve against bench/trust-graph/Cargo.lock. --locked
# keeps the run reproducible (fail if the lockfile is stale).
cargo build --release --locked >/dev/null 2>&1 || {
  # A first-time checkout (or a dependency bump) may need to refresh the lockfile; retry
  # once without --locked so a fresh clone is not blocked, then report honestly.
  echo "[trust-graph] --locked build failed; retrying without --locked (lockfile may need refresh)…" >&2
  cargo build --release >/dev/null 2>&1
}

BIN="$HERE/target/release/trust-graph-bench"
if [ ! -x "$BIN" ]; then
  echo "[trust-graph] ERROR: trust-graph-bench binary not found at $BIN after build" >&2
  exit 1
fi

echo "[trust-graph] running the fail-closed additivity-envelope + overhead lanes…" >&2
"$BIN" "$@"
RC=$?
if [ "$RC" -eq 0 ]; then
  echo "[trust-graph] OK: the strict-additivity envelope held on every lane (fail-closed)." >&2
else
  echo "[trust-graph] FAILED: an envelope/correctness lane mismatched (rc=$RC)." >&2
fi
exit "$RC"
