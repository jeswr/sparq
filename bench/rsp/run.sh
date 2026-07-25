#!/usr/bin/env bash
# [OPUS-4.8] (sq-b1hn) RSP-QL per-commit runner — the self-asserting entry point CI calls,
# mirroring bench/fts/run.sh + bench/deep-taxonomy/run.sh. Self-contained:
#   1. run the sparq-rsp `rsp_oracle` EXAMPLE (the crate is isolated — not a sparq-cli
#      dependency, so the runner is a crate example, like bench/fts's bench_text).
#   2. ASSERT every DETERMINISTIC per-window result-row count vs expected.tsv (exit 1 on
#      any drift — the HARD correctness gate). sparq-rsp is CLOCK-FREE, so a replay over a
#      fixed (triple, ts) script is a pure function: the per-window row counts are fully
#      deterministic, a STRONGER gate than any wall-clock RSP benchmark. This covers the
#      single-window scenarios (tumbling/sliding/GROUP-BY × three EvalModes — the
#      three-EvalMode-equivalence is encoded as identical expected counts) AND the SRBench
#      correctness ORACLE (multi-window observation ⋈ station-metadata join).
#   3. forward on STDOUT the 3-column `<metric>\t<value>\t<unit>` contract the ci-bench
#      `rsp` hook consumes: the asserted per-window `*_rows` counts (DETERMINISTIC) plus the
#      single advisory `rsp_persistentdict_triples_per_s` (trend-only — NOT gated, machine-
#      sensitive). Correctness lives in expected.tsv, exactly like LUBM / FTS / SHACL.
#
# Raw perf head-to-head vs C-SPARQL / CQELS / RSP4J stays OUT OF SCOPE here (wall-clock
# service engines — a different time model); the BOUNDED count-matched-replay comparison
# (design record sec 5.2) lives beside this gate: see README.md + rsp4j-smoke.sh /
# gather-rsp4j.sh (sq-hmd7l.20).
#
# Usage: bench/rsp/run.sh   (run from anywhere; honours $RSP_BIN override)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
EXP="$ROOT/bench/rsp/expected.tsv"
# The G1 TSV-emitting runner: a sparq-rsp example (the crate is isolated — not a sparq-cli
# dependency). Override with $RSP_BIN for a custom build.
BIN="${RSP_BIN:-$ROOT/target/release/examples/rsp_oracle}"

if [ ! -x "$BIN" ]; then
  echo "[rsp] ERROR: rsp_oracle not found at $BIN" >&2
  echo "[rsp]   build: cargo build --release -p sparq-rsp --example rsp_oracle" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. run the oracle (TSV: metric value unit) -----------------------------------------
"$BIN" > "$TMP/rsp.tsv"

# ---- 2. assert the DETERMINISTIC *_rows metrics vs expected.tsv (exit 1 on any mismatch) -
# ---- 3. forward the 3-column hook contract on STDOUT (advisory metric passes through) ----
fail=0
seen=0
while IFS=$'\t' read -r metric value unit; do
  [ -n "$metric" ] || continue
  case "$metric" in
    *_rows)
      exp="$(awk -F'\t' -v k="$metric" '$1==k{print $2}' "$EXP")"
      if [ -z "$exp" ]; then
        echo "[rsp] ERROR: $metric has no expected.tsv entry" >&2; fail=1; continue
      fi
      if [ "$value" != "$exp" ]; then
        echo "[rsp] ERROR: $metric rows=$value expected=$exp (RSP correctness regression)" >&2; fail=1
      fi
      seen=$((seen+1))
      ;;
  esac
  # Forward EVERY metric (deterministic counts + advisory throughput) to the hook.
  printf '%s\t%s\t%s\n' "$metric" "$value" "$unit"
done < "$TMP/rsp.tsv"

# Every expected per-window metric must have been emitted (a vanished window is a regression).
exp_rows=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_rows" ]; then
  echo "[rsp] ERROR: emitted $seen *_rows metrics, expected $exp_rows (a window/scenario vanished)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[rsp] OK: all $seen per-window row counts match expected.tsv (single-window x 3 EvalModes + SRBench oracle)" >&2
else
  echo "[rsp] FAILED: one or more per-window row counts diverged from expected.tsv" >&2
fi
exit "$fail"
