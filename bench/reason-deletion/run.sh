#!/usr/bin/env bash
# [FABLE-5] Deletion-workload correctness + timing runner for incremental reasoning
# (bead sq-31fza; parent sq-6tykl.4 "FBF-grade retraction benchmark"; axis under sq-hmd7l).
# A self-contained, self-asserting entry point mirroring bench/owl-sameas/run.sh /
# bench/deep-taxonomy/run.sh. For EACH requested TIER it:
#
#   1. ensures the corpus exists (gen.sh -> gen_reason_deletion.py): a deterministic
#      LUBM-class synthetic ABox at the tier's unit count (units column of expected.tsv).
#   2. runs the EXISTING, UNMODIFIED driver example
#        crates/sparq-reason/examples/incremental_olympics_bench.rs
#      on that corpus. The example itself performs the randomized insert/delete batches
#      (fixed-seed xorshift; ABox-only sampling) across three workloads — RDFS
#      (MaterializedGraph), OWL 2 RL mono + OWL 2 RL fixpoint (MaterializedOwlGraph) —
#      and carries the LOAD-BEARING built-in asserts: after the randomized
#      insert/delete/restore sequence the incrementally-maintained closure must be
#      set-EQUAL to a from-scratch re-materialization, and ABox deltas must never have
#      triggered a full rebuild. A violated assert panics -> non-zero exit -> this
#      runner fails LOUDLY. (Differential correctness is the POINT of the suite.)
#   3. parses the closure sizes and ASSERTS them against expected.tsv (deterministic
#      structural counts — catches silent under/over-derivation AND generator drift).
#   4. emits timing metric TSV lines and wraps everything in the standard JSON results
#      envelope (host block marks work-box timings NON-canonical; quiet_box=false).
#
# DELETION RATIOS: the driver's delta sizes are fixed ({1, 100, 10000} triples), so the
# suite gets its ratio axis by varying the BASE size per tier — e.g. a 10,000-triple
# delete batch is a large fraction of the small tier's ABox and a small fraction of the
# large tier's. The envelope records delete_ratio = delta / abox_triples per cell.
#
# Usage: bench/reason-deletion/run.sh    (run from anywhere; honours these env knobs)
#   REASONDEL_TIERS="small medium"  tier names (rows of expected.tsv; default per-commit
#                                   pair; add "large" for the nightly/EC2 tier)
#   BIN=<path>                      driver binary (default:
#                                   target/release/examples/incremental_olympics_bench;
#                                   built automatically if absent)
#   REASONDEL_CACHE=<dir>           corpus + envelope cache (default /tmp/reason-deletion)
#   REASONDEL_JSON_OUT=<path>       envelope path (default $REASONDEL_CACHE/
#                                   reason-deletion-<UTC>.json; git-ignored territory)
#
# stdout: one TSV line per metric:  <metric_name>\t<value>\t<unit>
#   reasondel_<tier>_<wl>_closure_triples   incremental closure size (DETERMINISTIC)
#   reasondel_<tier>_<wl>_full_s            from-scratch re-materialization seconds
#   reasondel_<tier>_<wl>_ins<delta>_s      incremental insert-batch seconds
#   reasondel_<tier>_<wl>_del<delta>_s      incremental delete-batch seconds
#   (<wl> in {rdfs, owlmono, owlfix}; timings are wall-clock trend-only — NOT perf-gated)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
BIN="${BIN:-$ROOT/target/release/examples/incremental_olympics_bench}"
GEN="$ROOT/bench/reason-deletion/gen.sh"
EXP="$ROOT/bench/reason-deletion/expected.tsv"
CACHE="${REASONDEL_CACHE:-/tmp/reason-deletion}"
TIERS="${REASONDEL_TIERS:-small medium}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
JSON_OUT="${REASONDEL_JSON_OUT:-$CACHE/reason-deletion-$STAMP.json}"

if [ ! -x "$BIN" ]; then
  echo "[reasondel] driver not found at $BIN — building (release)..." >&2
  cargo build --release -p sparq-reason --example incremental_olympics_bench >&2
fi
if [ ! -x "$BIN" ]; then
  echo "[reasondel] ERROR: driver still missing at $BIN" >&2
  exit 1
fi

mkdir -p "$CACHE"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail=0
ran_tiers=""
for TIER in $TIERS; do
  UNITS="$(awk -F'\t' -v t="$TIER" '$1==t{print $2}' "$EXP")"
  if [ -z "$UNITS" ]; then
    echo "[reasondel] ERROR: tier '$TIER' has no expected.tsv row" >&2
    fail=1; continue
  fi
  echo "[reasondel] === tier $TIER (units=$UNITS) ===" >&2
  CORPUS="$("$GEN" "$UNITS")"

  # Run the driver. A built-in differential-correctness assert failing PANICS the
  # example -> non-zero exit here -> the suite is RED. That exit-code coupling is the
  # acceptance-critical wiring: run.sh green <=> incremental==from-scratch held on the
  # whole randomized sequence in all three workloads (plus the N3/WAC section's own
  # asserts, which the example always runs).
  if ! "$BIN" "$CORPUS" > "$TMP/$TIER.out" 2> "$TMP/$TIER.err"; then
    echo "[reasondel] ERROR: driver FAILED on tier $TIER (differential-correctness" >&2
    echo "[reasondel]        assert or crash). Last lines:" >&2
    tail -n 25 "$TMP/$TIER.out" "$TMP/$TIER.err" >&2 || true
    fail=1; continue
  fi
  if grep -q "skipping RDFS/OWL workloads" "$TMP/$TIER.out"; then
    echo "[reasondel] ERROR: driver skipped the corpus (parse/path problem?)" >&2
    fail=1; continue
  fi
  ran_tiers="$ran_tiers $TIER"
done

# Parse every tier's output, assert closure fixtures, emit metric TSV + the envelope.
if ! RD_TMP="$TMP" RD_EXP="$EXP" RD_TIERS="$ran_tiers" RD_JSON_OUT="$JSON_OUT" \
     RD_SPARQ_REV="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)" \
     python3 "$ROOT/bench/reason-deletion/parse_and_assert.py"; then
  fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[reasondel] OK: differential correctness + closure fixtures green ($ran_tiers )" >&2
  echo "[reasondel] envelope: $JSON_OUT" >&2
else
  echo "[reasondel] FAILED: see errors above" >&2
fi
exit "$fail"
