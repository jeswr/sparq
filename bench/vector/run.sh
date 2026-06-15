#!/usr/bin/env bash
# [OPUS-4.8] (sq-v02y) Vector / ANN per-commit runner — the self-asserting entry point CI
# calls, mirroring bench/lubm/run.sh + bench/shacl/run.sh + bench/fts/run.sh. Self-contained:
#   1. read the pinned corpus parameters (N, seed) from gen.sh.
#   2. run examples/bench_vectors on that corpus (deterministic recall DEFICITS + advisory us).
#   3. GATE each recall workload's `count` (the integer recall deficit). The deterministic
#      workloads (diskann/pq — single-threaded, fixed-seed builds) are EXACT-gated vs
#      expected.tsv (exit 1 on any drift). The HNSW workload is FLOOR-gated only: its
#      instant-distance build is rayon-PARALLEL, so float-sum reduction order can flip a
#      single boundary neighbour (+/-1 deficit) — exact equality would be a FLAKY (dishonest)
#      gate, so HNSW is asserted < its floor (recall@10 >= 0.95) instead. ALL workloads also
#      assert floor headroom. This is the ANN analogue of LUBM's row-count diff / FTS's
#      hit-count diff, adapted for an APPROXIMATE surface (recall vs exact-kNN as a deficit, G4).
#   4. emit on STDOUT the 3-column `<workload>\t<count>\t<us>` contract the ci-bench hook
#      consumes (count = the asserted recall deficit; us = the advisory query latency). The
#      advisory build_s row is forwarded with its sentinel-0 count (no expected.tsv entry).
#
# Usage: bench/vector/run.sh   (run from anywhere; honours $BENCH_VECTORS / $ITERS / $VEC_N
#                               / $VEC_SEED overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
ITERS="${ITERS:-3}"
GEN="$ROOT/bench/vector/gen.sh"
EXP="$ROOT/bench/vector/expected.tsv"
# The G1 TSV-emitting runner: a sparq-vectors example (the crate is isolated — not a sparq-cli
# dependency). Override with $BENCH_VECTORS for a custom build.
BIN="${BENCH_VECTORS:-$ROOT/target/release/examples/bench_vectors}"

if [ ! -x "$BIN" ]; then
  echo "[vector] ERROR: bench_vectors not found at $BIN" >&2
  echo "[vector]   build: cargo build --release -p sparq-vectors --example bench_vectors" >&2
  exit 1
fi

# ---- 1. resolve the pinned corpus parameters (gen.sh emits two lines: N then seed) ----------
mapfile -t PARAMS < <("$GEN" "${VEC_N:-}" "${VEC_SEED:-}" 2>/dev/null || "$GEN")
N="${PARAMS[0]}"
SEED="${PARAMS[1]}"
echo "[vector] corpus: N=$N seed=$SEED iters=$ITERS" >&2

# ---- 2. run bench_vectors (TSV: name deficit us) --------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
"$BIN" "$N" "$SEED" "$ITERS" > "$TMP/vector.tsv"

# Per-workload floor headroom: the recall deficit must stay STRICTLY below these (= the crate
# gate-test floors expressed as deficits: 1-0.95=0.05=>50, 1-0.90=0.10=>100).
floor_for() {
  case "$1" in
    hnsw_recall_at10) echo 50 ;;
    diskann_recall_at10) echo 100 ;;
    pq_recall_at10) echo 50 ;;
    *) echo "" ;;
  esac
}

# ---- 3. assert the DETERMINISTIC deficit column vs expected.tsv (exit 1 on any mismatch) ----
fail=0
seen=0
while IFS=$'\t' read -r name count us; do
  [ -n "$name" ] || continue
  # The advisory build_s row has no expected.tsv entry — forward it but don't gate it.
  if [ "$name" = "build_s" ]; then
    printf '%s\t%s\t%s\n' "$name" "$count" "$us"
    continue
  fi
  exp="$(awk -F'\t' -v k="$name" '$1==k{print $2}' "$EXP")"
  if [ -z "$exp" ]; then
    echo "[vector] ERROR: $name has no expected.tsv entry" >&2; fail=1; continue
  fi
  # EXACT-gate the deterministic workloads (single-threaded, fixed-seed builds). HNSW is
  # rayon-parallel (+/-1 deficit jitter) so it is FLOOR-gated only — exact equality would flake.
  case "$name" in
    hnsw_recall_at10) : ;;  # floor-only (handled below); no exact-equality assert
    *)
      if [ "$count" != "$exp" ]; then
        echo "[vector] ERROR: $name deficit=$count expected=$exp (recall regression)" >&2; fail=1
      fi
      ;;
  esac
  # Floor headroom: every workload's deficit must stay below the crate gate-test floor — this is
  # the HARD gate for HNSW (it has no exact assert) and a second guard for diskann/pq.
  floor="$(floor_for "$name")"
  if [ -n "$floor" ] && [ "$count" -ge "$floor" ]; then
    echo "[vector] ERROR: $name deficit=$count >= floor $floor (recall below the crate gate)" >&2; fail=1
  fi
  seen=$((seen+1))
  # ---- 4. forward the 3-column hook contract on STDOUT (count = the asserted deficit) ----
  printf '%s\t%s\t%s\n' "$name" "$count" "$us"
done < "$TMP/vector.tsv"

# Every expected workload must have run (a vanished workload is itself a regression).
exp_workloads=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_workloads" ]; then
  echo "[vector] ERROR: ran $seen recall workloads, expected $exp_workloads (a workload vanished)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[vector] OK: all $seen recall workloads match expected.tsv (deficits) + clear their floors" >&2
else
  echo "[vector] FAILED: one or more recall deficits diverged from expected.tsv / breached a floor" >&2
fi
exit "$fail"
