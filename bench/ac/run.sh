#!/usr/bin/env bash
# [OPUS-4.8] sq-i6du2.7 (epic sq-i6du2, #1613). 🤖 SPARQ agent — the AC-query benchmark
# harness runner. Written while Fable unavailable; flag for re-review when Fable returns.
# Spec: research/ac-query-benchmark.md (§2–§3). Substrate: crates/sparq-acbench (#1627).
#
# The self-contained, self-asserting entry point CI + scripts/bench/run-all-benchmarks.sh
# call (mirrors bench/deep-taxonomy/run.sh + bench/parse/run.sh). It:
#   1. builds the standalone `ac-bench` driver (bench/ac — its own [workspace], like
#      bench/parse + bench/dict; it path-depends on the dev-only crate sparq-acbench and
#      links NO engine crate, so the by-construction oracle is structurally independent of
#      the system under test);
#   2. runs the W1 (decision) / W2-oracle (result-set) / W3 (ACL-write churn) oracle lanes
#      over every USE-CASE generator that is PRESENT (a not-yet-implemented generator is
#      Skipped-with-reason, never a fabricated pass);
#   3. propagates the driver's FAIL-CLOSED exit code: any W1/W2/W3 mismatch against the
#      by-construction oracle => the driver exits non-zero => this script exits non-zero.
#
# SCOPE BOUNDARY (bead sq-i6du2.7): scripts + registry ONLY. This runs the oracle lanes
# that need NO engine link. The W2 live `query_as` sub-lane and the W4 concurrency QUERY
# sub-lane require linking `sparq-solid` / the real engine and belong to the separate
# live-driver bead sq-kvvcl; the driver records them Skipped-with-reason (visible, never
# a silent drop). This script does NOT link sparq-solid.
#
# HONESTY: the driver prints an indicative wall-clock per lane, but every such number is
# advisory + NON-CANONICAL on this shared work box (see the QUIET-BOX convention in
# bench/CATALOG.md). The HARD, load-robust contract is the fail-closed oracle exit code —
# a deterministic pass/fail, not a timing.
#
# Usage: bench/ac/run.sh [--smoke] [--sf N]   (run from anywhere)
#   --smoke   quick fixed-seed smoke vector (GenParams::smoke) — the CI/per-commit tier.
#   --sf N    scale factor for a nightly/EC2 tier (default 1 when not --smoke).
# Env knobs:
#   ACBENCH_SMOKE=1   force --smoke even with no argument (per-commit default).
# stdout: the driver's per-suite TSV table (# comment header + one row per lane) plus a
#         per-suite pass/fail/skip summary. Exit 0 iff every oracle lane agreed.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/bench/ac"
cd "$HERE"

# Default to the smoke tier for the per-commit lane; pass-through explicit args otherwise.
ARGS=("$@")
if [ ${#ARGS[@]} -eq 0 ] && [ "${ACBENCH_SMOKE:-0}" = "1" ]; then
  ARGS=(--smoke)
fi

echo "[ac] building the standalone ac-bench driver (bench/ac; links no engine crate)…" >&2
# Standalone workspace root — resolve against bench/ac/Cargo.lock, exactly like bench/parse
# and bench/dict. --locked keeps the run reproducible (fail if the lockfile is stale).
cargo build --release --locked >/dev/null 2>&1 || {
  # A first-time checkout (or a dependency bump) may need to refresh the lockfile; retry
  # once without --locked so a fresh clone is not blocked, then report honestly.
  echo "[ac] --locked build failed; retrying without --locked (lockfile may need refresh)…" >&2
  cargo build --release >/dev/null 2>&1
}

BIN="$HERE/target/release/ac-bench"
if [ ! -x "$BIN" ]; then
  echo "[ac] ERROR: ac-bench binary not found at $BIN after build" >&2
  exit 1
fi

echo "[ac] running the W1/W2-oracle/W3 fail-closed oracle lanes…" >&2
# The driver streams its own TSV table to stdout and exits non-zero on any oracle mismatch.
"$BIN" "${ARGS[@]}"
rc=$?

if [ "$rc" -eq 0 ]; then
  echo "[ac] OK: every oracle lane agreed with the by-construction oracle (fail-closed)." >&2
else
  echo "[ac] FAILED: an oracle lane mismatched the by-construction oracle (rc=$rc)." >&2
fi
exit "$rc"
