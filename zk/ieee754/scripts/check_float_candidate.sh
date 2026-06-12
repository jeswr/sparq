#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
candidate_output="${1:-/tmp/candidate_float_ops.json}"

cd "$repo_root"

echo "==> Running Noir tests"
nargo test --silence-warnings

echo "==> Checking generated vectors"
bash ./scripts/test_generated_vectors.sh

echo "==> Checking public API"
bash ./scripts/test_public_api.sh

echo "==> Linting private helper usage"
python3 scripts/lint_private_function_usage.py

echo "==> Checking diff whitespace"
git diff --check

echo "==> Benchmarking float operations"
python3 scripts/benchmark_float_ops.py --output "$candidate_output"

echo "==> Comparing gate counts against baseline"
python3 scripts/compare_float_benchmarks.py "$candidate_output" --baseline bench/float_ops_latest.json --max-regression 1

echo "==> Candidate checks passed"