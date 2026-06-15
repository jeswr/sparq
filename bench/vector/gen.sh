#!/usr/bin/env bash
# [OPUS-4.8] (sq-v02y) Vector / ANN benchmark corpus parameters. Like the FTS suite (and
# unlike LUBM/SHACL, which need an external generator + rapper), the ANN corpus is SYNTHETIC
# and generated IN-PROCESS by examples/bench_vectors from a deterministic splitmix64 seed —
# there is no file to materialise and no toolchain to install. So gen.sh is a thin parameter
# source: it echoes the two pinned corpus parameters on stdout (mirroring the two-line gen.sh
# convention), which run.sh + the ci-bench hook consume to drive bench_vectors:
#
#   bench/vector/gen.sh [N] [seed]
#
#   1. <N>      number of synthetic 32-d vectors in the HNSW/DiskANN substrate
#   2. <seed>   the corpus RNG seed
#
# At a fixed (N, seed) the corpus, the index parameters, and the FIXED query set are all
# deterministic, so recall@10 vs the nearest_exact brute-force ground truth — and therefore
# the integer recall DEFICITS in expected.tsv — are deterministic constants. The PQ workload
# uses a FIXED 10k clustered set (independent of N — N scales only the HNSW/DiskANN substrate),
# matching the regime crates/sparq-vectors/tests/quant.rs gates.
#
# Per-commit tier: N=50000 (the same 50k x 32 set the crate recall gate tests use; sub-second
# at release opt-level). The big published ANN datasets (SIFT1M / GloVe-100-angular) drive the
# gather-tier ann-benchmarks recall-QPS Pareto harness (a follow-up bead — too heavy per-commit).
set -euo pipefail

N="${1:-50000}"
SEED="${2:-0}"

echo "$N"
echo "$SEED"
