#!/usr/bin/env bash
# [OPUS-4.8] OWL sameAs equality micro-suite corpus generator (sq-msl6, design A3). A thin,
# DETERMINISTIC, cache-backed wrapper around bench/owl-sameas/gen_sameas.py — mirroring
# bench/deep-taxonomy/gen.sh + bench/lubm/gen.sh as a first-class bench/<suite>/gen.sh entry point.
#
#   bench/owl-sameas/gen.sh [tier=8] [out=/tmp/owl-sameas/sameas-<tier>.nt]
#
# A TIER is the per-class size N; the suite fixes the class count K and the data-triples-per-class M
# (see TIER_K / TIER_M below) so a single numeric tier knob drives the dashboard scaling axis. Emits
# ONE path on stdout: the generated N-Triples corpus for the requested tier:
#   K independent owl:sameAs equivalence classes, each N individuals (a STAR of N-1 sameAs edges to
#   the class anchor) + M data triples on the anchor. See gen_sameas.py for the exact shape.
#
# run.sh consumes it: materialise the OWL-RL closure with `sparq-cli reason <f> ntriples owl`, run
# query.rq over the closure, and ASSERT both the closure triple-count (K*N*(N+M)) and the query
# row-count (K*N) against expected.tsv (deterministic — see run.sh + README.md).
#
# HERMETICITY / CI NOTES (identical posture to bench/deep-taxonomy/gen.sh):
#  - Network: NONE. gen_sameas.py is in-repo pure Python (stdlib only); nothing is fetched.
#  - Toolchain: python3 ONLY (already required by the bench harness). NO javac/rapper/java guard,
#    so unlike lubm this suite runs on the per-commit tier by default (scripts/ci-bench.sh sameas hook).
#  - Output is FULLY DETERMINISTIC (pure function of K,N,M — no RNG, byte-identical every run), so the
#    corpus — and thus the closed-form closure size — is reproducible per-commit.
#  - Output cached under $SAMEAS_CACHE (default /tmp/owl-sameas), gitignored + regenerable
#    (bench/CATALOG.md disk discipline). Caller cleans up.
#
# Output is N-Triples; load into sparq with format `ntriples`.
set -euo pipefail

TIER="${1:-8}"          # per-class size N
TIER_K=4                # equivalence classes (fixed across tiers)
TIER_M=3                # data triples per class (fixed across tiers)
CACHE="${SAMEAS_CACHE:-/tmp/owl-sameas}"
OUT="${2:-$CACHE/sameas-${TIER}.nt}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GEN="$ROOT/bench/owl-sameas/gen_sameas.py"

if ! command -v python3 >/dev/null 2>&1; then
  echo "[sameas] ERROR: 'python3' not found (needed to run gen_sameas.py)." >&2
  exit 1
fi
if [ ! -f "$GEN" ]; then
  echo "[sameas] ERROR: generator missing at $GEN" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"
# Deterministic: same bytes every run for a given (K,N,M). Regenerate only if absent so the cache
# (like lubm/deep-taxonomy) makes steady-state per-commit runs skip the (cheap) generation step too.
if [ ! -s "$OUT" ]; then
  python3 "$GEN" "$TIER_K" "$TIER" "$TIER_M" > "$OUT.tmp"
  mv "$OUT.tmp" "$OUT"
fi

echo "$OUT"
