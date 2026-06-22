#!/usr/bin/env bash
# [OPUS-4.8] (sq-lrp9) HDT per-commit runner — the self-asserting entry point CI calls,
# mirroring bench/fts/run.sh + bench/rsp/run.sh. Self-contained:
#   1. run the sparq-hdt `bench_oracle` EXAMPLE (the crate is isolated — not a sparq-cli
#      dependency, so the runner is a crate example, like FTS's bench_text / RSP's
#      rsp_oracle). It loads the VENDORED real-world snikmeta.hdt and decodes it to a
#      native sparq Graph, then resolves triple patterns over that graph.
#   2. ASSERT every DETERMINISTIC `count`-unit metric vs expected.tsv (exit 1 on any drift
#      — the HARD correctness gate). This is the LOAD-AND-DECODE-TO-NATIVE + triple-pattern
#      -resolution gate (design §3.7) plus the direct-vs-upstream id-translation oracle.
#   3. forward on STDOUT the 3-column `<metric>\t<value>\t<unit>` contract the ci-bench
#      `hdt` hook consumes: the asserted DETERMINISTIC counts plus the advisory
#      hdt_load_s / hdt_vs_ntgz_load_s (trend-only — NOT gated, machine-sensitive; the
#      RATIO survives box contention per bench/CATALOG.md). Correctness lives in
#      expected.tsv, exactly like LUBM / FTS / SHACL / RSP.
#
# Load-and-decode-to-native is the ONLY like-for-like axis vs hdt-cpp / hdt-java
# (sparq-hdt decodes HDT into sparq's own Dict/Graph then queries its own indexes;
# hdt-cpp/java query the compressed BitmapTriples in place — different model). A
# "query over HDT" head-to-head is explicitly OUT OF SCOPE (see README.md).
#
# Usage: bench/hdt/run.sh   (run from anywhere; honours $HDT_BIN / $HDT_BENCH_N overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
EXP="$ROOT/bench/hdt/expected.tsv"
# The G1 TSV-emitting runner: a sparq-hdt example (the crate is isolated — not a sparq-cli
# dependency). Override with $HDT_BIN for a custom build.
BIN="${HDT_BIN:-$ROOT/target/release/examples/bench_oracle}"

if [ ! -x "$BIN" ]; then
  echo "[hdt] ERROR: bench_oracle not found at $BIN" >&2
  echo "[hdt]   build: cargo build --release -p sparq-hdt --example bench_oracle" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. run the oracle (TSV: metric value unit) -----------------------------------------
"$BIN" > "$TMP/hdt.tsv"

# ---- 2. assert the DETERMINISTIC count-unit metrics vs expected.tsv (exit 1 on mismatch) -
# ---- 3. forward the 3-column hook contract on STDOUT (advisory rows pass through) --------
fail=0
seen=0
while IFS=$'\t' read -r metric value unit; do
  [ -n "$metric" ] || continue
  case "$unit" in
    count)
      exp="$(awk -F'\t' -v k="$metric" '$1==k{print $2}' "$EXP")"
      if [ -z "$exp" ]; then
        echo "[hdt] ERROR: $metric has no expected.tsv entry" >&2; fail=1; continue
      fi
      if [ "$value" != "$exp" ]; then
        echo "[hdt] ERROR: $metric=$value expected=$exp (HDT load-decode correctness regression)" >&2; fail=1
      fi
      seen=$((seen+1))
      ;;
  esac
  # Forward EVERY metric (deterministic counts + advisory timings) to the hook.
  printf '%s\t%s\t%s\n' "$metric" "$value" "$unit"
done < "$TMP/hdt.tsv"

# Every expected metric must have been emitted (a vanished gate metric is a regression).
exp_rows=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_rows" ]; then
  echo "[hdt] ERROR: emitted $seen count metrics, expected $exp_rows (a gate metric vanished)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[hdt] OK: all $seen load-and-decode counts match expected.tsv (snikmeta triples/terms/predicates + direct==upstream oracle)" >&2
else
  echo "[hdt] FAILED: one or more load-and-decode counts diverged from expected.tsv" >&2
fi
exit "$fail"
