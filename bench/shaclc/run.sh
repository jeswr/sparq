#!/usr/bin/env bash
# [GPT-5.6] PR #2136: sparq-shaclc parse/write-throughput runner.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
shapes="${SHACLC_SHAPES:-2000}"

case "${1:-}" in
  "") ;;
  --smoke) shapes=10 ;;
  -h|--help)
    sed -n '2,12p' "${BASH_SOURCE[0]}"
    exit 0
    ;;
  *) echo "usage: bench/shaclc/run.sh [--smoke]" >&2; exit 2 ;;
esac

cd "$root"
cargo run --release -p sparq-shaclc --example bench_shaclc -- "$shapes"
