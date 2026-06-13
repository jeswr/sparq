#!/usr/bin/env bash
# [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
#
# Gate counts for the zk/compose circuit family (bb gates is ground truth,
# noir-optimisation SKILL §). Compiles every member and emits the
# `circuit_size` ultra_honk gate count in the zk/ieee754 JSON convention.
#
# Usage:  bench/zk-compose/scripts/gate_counts.sh > bench/zk-compose/gate_counts_latest.json
set -euo pipefail

COMPOSE_DIR="$(cd "$(dirname "$0")/../../../zk/compose" && pwd)"
MEMBERS=(scan_k1_n16_r4 scan_k2_n16_r8 scan_k2_n64_r8 filter_int_d1 filter_int_d2 filter_int_d4 filter_f64)

cd "$COMPOSE_DIR"
nargo compile --workspace >/dev/null 2>&1

printf '{\n'
printf '  "timestamp": "%s",\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  "tool": "bb gates -s ultra_honk",\n'
printf '  "bb_version": "%s",\n' "$(bb --version 2>/dev/null | head -1)"
printf '  "nargo_version": "%s",\n' "$(nargo --version 2>/dev/null | grep -o 'nargo version = .*' | head -1)"
printf '  "benchmarks": {\n'
last=$(( ${#MEMBERS[@]} - 1 ))
for i in "${!MEMBERS[@]}"; do
  pkg="${MEMBERS[$i]}"
  size=$(bb gates -s ultra_honk -b "target/$pkg.json" 2>/dev/null \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['functions'][0]['circuit_size'])")
  comma=","; [ "$i" -eq "$last" ] && comma=""
  printf '    "%s": { "circuit_size": %s }%s\n' "$pkg" "$size" "$comma"
done
printf '  }\n'
printf '}\n'
