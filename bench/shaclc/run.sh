#!/usr/bin/env bash
# [GPT-5.6] PR #2136: sparq-shaclc committed-corpus parse-throughput runner.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
passes="${SHACLC_PASSES:-20}"
samples="${SHACLC_SAMPLES:-5}"

case "${1:-}" in
  "") ;;
  --smoke) passes=1; samples=1 ;;
  -h|--help)
    sed -n '2,12p' "${BASH_SOURCE[0]}"
    exit 0
    ;;
  *) echo "usage: bench/shaclc/run.sh [--smoke]" >&2; exit 2 ;;
esac

cd "$root"
cargo run --release -p sparq-shaclc --example bench_parse -- "$passes" "$samples"
