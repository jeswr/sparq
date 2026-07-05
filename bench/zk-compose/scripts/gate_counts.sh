#!/usr/bin/env bash
# [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
#
# Gate counts for the zk/compose circuit family (bb gates is ground truth,
# noir-optimisation SKILL §). Compiles every member and emits the
# `circuit_size` ultra_honk gate count in the sparq_ieee754 JSON convention
# (the `{circuit_size}` shape originated in the ieee754 lineage, now the
# sparq-org/noir_IEEE754 face repo — sq-5reoy/#1599).
#
# [OPUS-4.8] sq-5reoy: `nargo compile` of zk/compose now fetches the
# sparq_ieee754 dependency from the sparq-org/noir_IEEE754 git tag (v0.10.0),
# so this script needs network access to GitHub on first run of a fresh
# ~/.nargo cache (same as the existing poseidon v0.3.0 git dep it already
# fetches).
#
# The member list is read from the regression-gate snapshot
# (crates/sparq-zk-compose/tests/gate_count_snapshot.json) — the SAME source the
# in-crate gate_count_regression test baselines against — so this bench JSON
# always covers exactly the regression-gated set and the two can never drift.
# (sq-ifur: a hand-maintained MEMBERS array had silently dropped holder_pok,
# hidden_issuer_d4, revoke_unset_d10, and the join + scan-lattice members that
# the snapshot already gates, so re-baselining never regenerated their numbers.)
#
# Usage:  bench/zk-compose/scripts/gate_counts.sh > bench/zk-compose/gate_counts_latest.json
set -euo pipefail

COMPOSE_DIR="$(cd "$(dirname "$0")/../../../zk/compose" && pwd)"
SNAPSHOT_JSON="$(cd "$(dirname "$0")/../../../crates/sparq-zk-compose/tests" && pwd)/gate_count_snapshot.json"

# Drive the member set from the regression-gate snapshot's `members` keys (sorted
# for a stable diff), so re-baselining can never silently skip a gated member.
mapfile -t MEMBERS < <(python3 -c \
  "import json,sys; print('\n'.join(sorted(json.load(open(sys.argv[1]))['members'])))" \
  "$SNAPSHOT_JSON")

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
