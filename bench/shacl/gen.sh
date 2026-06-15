#!/usr/bin/env bash
# [OPUS-4.8] (sq-7iai) SHACL benchmark corpus generator. The SHACL suite uses the LUBM
# ABox as its data substrate (exactly as Trav-SHACL / Re-SHACL do) x the 5 hand-authored
# shape graphs committed under shapes/. So this is a THIN wrapper over bench/lubm/gen.sh:
# it generates the deterministic LUBM(univ) ABox and emits two paths on stdout (mirroring
# bench/lubm/gen.sh's two-line convention):
#
#   bench/shacl/gen.sh [univ=1] [seed=0]
#
#   1. <data.nt>     the LUBM(univ) ABox (raw N-Triples; the SHACL DATA graph)
#   2. <shapes-dir>  bench/shacl/shapes/ (the 5 committed shape graphs; NOT generated)
#
# run.sh consumes both: validate <data.nt> against each shapes/*.ttl. The ontology TBox
# line that bench/lubm/gen.sh emits is intentionally NOT forwarded — the SHACL workloads
# validate the RAW ABox (no entailment), so their violation counts are deterministic
# constants at the pinned (-univ, -seed) corpus + fixed shapes (see expected.tsv).
#
# Toolchain + hermeticity are inherited from bench/lubm/gen.sh (javac/jar + rapper; one
# pinned shallow clone, then fully cached). The corpus is gitignored + regenerable.
set -euo pipefail

UNIV="${1:-1}"
SEED="${2:-0}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

# Reuse the LUBM generator for the deterministic ABox; it emits two lines (data, ontology).
mapfile -t LUBM_ARTS < <("$ROOT/bench/lubm/gen.sh" "$UNIV" "$SEED")
DATA="${LUBM_ARTS[0]}"

echo "$DATA"
echo "$HERE/shapes"
