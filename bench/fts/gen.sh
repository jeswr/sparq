#!/usr/bin/env bash
# [OPUS-4.8] (sq-ustq) Full-text-search benchmark corpus parameters. Unlike LUBM/SHACL
# (which need an external generator + rapper), the FTS corpus is SYNTHETIC and generated
# IN-PROCESS by examples/bench_text from a deterministic seed — there is no file to
# materialise and no toolchain to install. So gen.sh is a thin parameter source: it echoes
# the two pinned corpus parameters on stdout (mirroring the two-line gen.sh convention),
# which run.sh + the ci-bench hook consume to drive bench_text:
#
#   bench/fts/gen.sh [N] [seed]
#
#   1. <N>      number of synthetic 8-word literals (the per-commit corpus size)
#   2. <seed>   the corpus RNG seed
#
# At a fixed (N, seed) the corpus — and therefore every hit count and the index
# bytes-per-doc — is a deterministic constant (see expected.tsv). The FIXED query set is
# drawn from an INDEPENDENT seed inside bench_text, so the asserted counts shift only when
# search SEMANTICS change, never as an artefact of corpus-gen RNG consumption.
#
# Per-commit tier: N=100000 (sub-second build, ~100k docs). Heavy/latency tier: N=1000000
# (the 1M-literal axis in the design); the latency numbers there are ADVISORY only and the
# dev box is non-canonical, so they are never gated.
set -euo pipefail

N="${1:-100000}"
SEED="${2:-0}"

echo "$N"
echo "$SEED"
