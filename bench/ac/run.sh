#!/usr/bin/env bash
# [OPUS-4.8] sq-i6du2.7 (epic sq-i6du2, #1613). [SONNET-4.6] sq-kvvcl (W2/W4 live lanes).
# 🤖 SPARQ agent — the AC-query benchmark harness runner.
# Written while Fable unavailable; flag for re-review when Fable returns.
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
#   3. builds the `ac-bench-live` live driver (bench/ac/live — its own [workspace]; it
#      links sparq-solid + the real engine so W2 live query_as + W4 concurrent-reader
#      lanes run against the actual authorization surface — see bead sq-kvvcl); and
#   4. propagates the FAIL-CLOSED exit code of BOTH drivers: any W1/W2/W3 oracle mismatch
#      OR any W2/W4 live OVER-SHARE (unauthorized data returned) => non-zero exit.
#
# HONESTY: the drivers print indicative wall-clock numbers, but every such number is
# advisory + NON-CANONICAL on this shared work box (see the QUIET-BOX convention in
# bench/CATALOG.md). The HARD, load-robust contract is the fail-closed exit code.
# AUTHORIZATION SURFACE: the live driver links sparq-solid + runs real WAC/ACP decisions.
#
# Usage: bench/ac/run.sh [--smoke] [--sf N]   (run from anywhere)
#   --smoke   quick fixed-seed smoke vector (GenParams::smoke) — the CI/per-commit tier.
#   --sf N    scale factor for a nightly/EC2 tier (default 1 when not --smoke).
# Env knobs:
#   ACBENCH_SMOKE=1    force --smoke even with no argument (per-commit default).
#   ACBENCH_NO_LIVE=1  skip the live driver (oracle-only run; e.g. offline, no sparq-solid).
# stdout: the drivers' per-suite TSV tables (# comment header + one row per lane) plus a
#         per-suite pass/fail/skip summary. Exit 0 iff every lane agreed (or skipped).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/bench/ac"
LIVE="$ROOT/bench/ac/live"
cd "$HERE"

# Default to the smoke tier for the per-commit lane; pass-through explicit args otherwise.
ARGS=("$@")
if [ ${#ARGS[@]} -eq 0 ] && [ "${ACBENCH_SMOKE:-0}" = "1" ]; then
  ARGS=(--smoke)
fi

# ── Oracle driver (bench/ac — links NO engine crate) ──────────────────────────────────
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

echo "[ac] running W1/W2-oracle/W3 fail-closed oracle lanes…" >&2
"$BIN" "${ARGS[@]}"
ORACLE_RC=$?

if [ "$ORACLE_RC" -eq 0 ]; then
  echo "[ac] oracle OK: every lane agreed with the by-construction oracle (fail-closed)." >&2
else
  echo "[ac] oracle FAILED: an oracle lane mismatched (rc=$ORACLE_RC)." >&2
fi

# ── Live driver (bench/ac/live — links sparq-solid, runs real WAC/ACP) ────────────────
LIVE_RC=0
if [ "${ACBENCH_NO_LIVE:-0}" = "1" ]; then
  echo "[ac] ACBENCH_NO_LIVE=1: skipping live driver (oracle-only run)." >&2
elif [ ! -d "$LIVE" ]; then
  echo "[ac] bench/ac/live/ not found — live W2/W4 lanes skipped (sq-kvvcl)." >&2
else
  echo "[ac] building the ac-bench-live driver (bench/ac/live; links sparq-solid)…" >&2
  cd "$LIVE"
  cargo build --release --locked >/dev/null 2>&1 || {
    echo "[ac] live --locked build failed; retrying without --locked…" >&2
    cargo build --release >/dev/null 2>&1
  }
  LIVE_BIN="$LIVE/target/release/ac-bench-live"
  if [ ! -x "$LIVE_BIN" ]; then
    echo "[ac] ERROR: ac-bench-live binary not found at $LIVE_BIN after build" >&2
    LIVE_RC=1
  else
    echo "[ac] running W2-live/W4-live fail-closed lanes (authorization surface)…" >&2
    "$LIVE_BIN" "${ARGS[@]}"
    LIVE_RC=$?
    if [ "$LIVE_RC" -eq 0 ]; then
      echo "[ac] live OK: no over-share detected (fail-closed security check passed)." >&2
    else
      echo "[ac] live FAILED: an over-share or concurrency mismatch was detected (rc=$LIVE_RC)." >&2
    fi
  fi
  cd "$HERE"
fi

# ── Aggregate exit code ────────────────────────────────────────────────────────────────
if [ "$ORACLE_RC" -ne 0 ] || [ "$LIVE_RC" -ne 0 ]; then
  exit 1
fi
exit 0
