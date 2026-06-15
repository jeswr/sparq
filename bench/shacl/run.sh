#!/usr/bin/env bash
# [OPUS-4.8] (sq-7iai) SHACL per-commit runner — the self-asserting entry point CI calls,
# mirroring bench/lubm/run.sh. Self-contained:
#   1. ensure the LUBM(1) ABox exists (gen.sh) + locate the shapes dir.
#   2. run examples/bench_shacl over the data x shapes/*.ttl (count + advisory timing).
#   3. ASSERT each workload's conforms/violations/focus_nodes vs expected.tsv (exit 1 on
#      any drift — the HARD correctness gate lives here, so the harvested timings stay
#      advisory). This is the SHACL analogue of LUBM's count diff.
#   4. emit on STDOUT the 3-column `<workload>\t<violations>\t<validate_us>` contract the
#      ci-bench hook consumes (count = the asserted #violations; us = validate time).
#      Correctness lives in expected.tsv, NOT in extra stdout columns — exactly like LUBM.
#
# Usage: bench/shacl/run.sh   (run from anywhere; honours $BENCH_SHACL / $ITERS overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
ITERS="${ITERS:-3}"
GEN="$ROOT/bench/shacl/gen.sh"
EXP="$ROOT/bench/shacl/expected.tsv"
# The G1 TSV-emitting runner: a sparq-shacl example (the crate stays isolated — it is not a
# dependency of sparq-cli). Override with $BENCH_SHACL for a custom build.
BIN="${BENCH_SHACL:-$ROOT/target/release/examples/bench_shacl}"

if [ ! -x "$BIN" ]; then
  echo "[shacl] ERROR: bench_shacl not found at $BIN" >&2
  echo "[shacl]   build: cargo build --release -p sparq-shacl --example bench_shacl" >&2
  exit 1
fi

# ---- 1. ensure data + locate shapes (gen.sh emits two lines: data.nt then shapes-dir) ----
mapfile -t ARTS < <("$GEN" 1 0)
DATA="${ARTS[0]}"
SHAPES="${ARTS[1]}"
echo "[shacl] data=$DATA  shapes=$SHAPES" >&2

# ---- 2. validate (full 6-column TSV: name violations validate_us conforms focus_nodes load_us)
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
"$BIN" "$DATA" ntriples "$SHAPES" "$ITERS" > "$TMP/shacl.tsv"

# ---- 3. assert the DETERMINISTIC columns vs expected.tsv (exit 1 on any mismatch) --------
fail=0
seen=0
while IFS=$'\t' read -r name violations validate_us conforms focus_nodes _load_us; do
  [ -n "$name" ] || continue
  # expected.tsv: <workload>\t<conforms>\t<violations>\t<focus_nodes>
  exp="$(awk -F'\t' -v k="$name" '$1==k{print $2"\t"$3"\t"$4}' "$EXP")"
  if [ -z "$exp" ]; then
    echo "[shacl] ERROR: $name has no expected.tsv entry" >&2; fail=1; continue
  fi
  IFS=$'\t' read -r exp_conforms exp_violations exp_focus <<< "$exp"
  if [ "$violations" != "$exp_violations" ]; then
    echo "[shacl] ERROR: $name violations=$violations expected=$exp_violations (correctness regression)" >&2; fail=1
  fi
  if [ "$conforms" != "$exp_conforms" ]; then
    echo "[shacl] ERROR: $name conforms=$conforms expected=$exp_conforms (correctness regression)" >&2; fail=1
  fi
  if [ "$focus_nodes" != "$exp_focus" ]; then
    echo "[shacl] ERROR: $name focus_nodes=$focus_nodes expected=$exp_focus (target-selection regression)" >&2; fail=1
  fi
  seen=$((seen+1))
  # ---- 4. forward the 3-column hook contract on STDOUT (count = #violations) ----
  printf '%s\t%s\t%s\n' "$name" "$violations" "$validate_us"
done < "$TMP/shacl.tsv"

# Every committed workload must have run (a vanished workload is itself a regression).
exp_workloads=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_workloads" ]; then
  echo "[shacl] ERROR: ran $seen workloads, expected $exp_workloads (a workload vanished)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[shacl] OK: all $seen workloads match expected.tsv (violations + conforms + focus_nodes)" >&2
else
  echo "[shacl] FAILED: one or more values diverged from expected.tsv" >&2
fi
exit "$fail"
