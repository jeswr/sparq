#!/usr/bin/env bash
# [OPUS-4.8] (sq-ustq) Full-text-search per-commit runner — the self-asserting entry point
# CI calls, mirroring bench/lubm/run.sh + bench/shacl/run.sh. Self-contained:
#   1. read the pinned corpus parameters (N, seed) from gen.sh.
#   2. run examples/bench_text on that corpus (deterministic counts + advisory timing).
#   3. ASSERT each workload's `count` vs expected.tsv (exit 1 on any drift — the HARD
#      correctness/structure gate lives here, so the harvested timings stay advisory).
#      This is the FTS analogue of LUBM's count diff: the query hit counts + the integer
#      index bytes-per-doc.
#   4. emit on STDOUT the 3-column `<workload>\t<count>\t<us>` contract the ci-bench hook
#      consumes (count = the asserted value; us = the advisory timing). Correctness lives in
#      expected.tsv, NOT in extra stdout columns — exactly like LUBM / SHACL.
#
# Usage: bench/fts/run.sh   (run from anywhere; honours $BENCH_TEXT / $ITERS / $FTS_N
#                            / $FTS_SEED overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
ITERS="${ITERS:-3}"
GEN="$ROOT/bench/fts/gen.sh"
EXP="$ROOT/bench/fts/expected.tsv"
# The G1 TSV-emitting runner: a sparq-text example (the crate is isolated — not a sparq-cli
# dependency). Override with $BENCH_TEXT for a custom build.
BIN="${BENCH_TEXT:-$ROOT/target/release/examples/bench_text}"

if [ ! -x "$BIN" ]; then
  echo "[fts] ERROR: bench_text not found at $BIN" >&2
  echo "[fts]   build: cargo build --release -p sparq-text --example bench_text" >&2
  exit 1
fi

# ---- 1. resolve the pinned corpus parameters (gen.sh emits two lines: N then seed) -------
mapfile -t PARAMS < <("$GEN" "${FTS_N:-}" "${FTS_SEED:-}" 2>/dev/null || "$GEN")
N="${PARAMS[0]}"
SEED="${PARAMS[1]}"
echo "[fts] corpus: N=$N seed=$SEED iters=$ITERS" >&2

# ---- 2. run bench_text (TSV: name count us) ----------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
"$BIN" "$N" "$SEED" "$ITERS" > "$TMP/fts.tsv"

# ---- 3. assert the DETERMINISTIC count column vs expected.tsv (exit 1 on any mismatch) ---
fail=0
seen=0
while IFS=$'\t' read -r name count us; do
  [ -n "$name" ] || continue
  exp="$(awk -F'\t' -v k="$name" '$1==k{print $2}' "$EXP")"
  if [ -z "$exp" ]; then
    echo "[fts] ERROR: $name has no expected.tsv entry" >&2; fail=1; continue
  fi
  if [ "$count" != "$exp" ]; then
    echo "[fts] ERROR: $name count=$count expected=$exp (correctness/structure regression)" >&2; fail=1
  fi
  seen=$((seen+1))
  # ---- 4. forward the 3-column hook contract on STDOUT (count = the asserted value) ----
  printf '%s\t%s\t%s\n' "$name" "$count" "$us"
done < "$TMP/fts.tsv"

# Every expected workload must have run (a vanished workload is itself a regression).
exp_workloads=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_workloads" ]; then
  echo "[fts] ERROR: ran $seen workloads, expected $exp_workloads (a workload vanished)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[fts] OK: all $seen workloads match expected.tsv (hit counts + bytes_per_doc)" >&2
else
  echo "[fts] FAILED: one or more values diverged from expected.tsv" >&2
fi
exit "$fail"
