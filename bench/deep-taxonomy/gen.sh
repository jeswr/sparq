#!/usr/bin/env bash
# [OPUS-4.8] Deep Taxonomy (DT) corpus generator (sq-1hgz; parent sq-i0nm "design D"). A thin,
# DETERMINISTIC wrapper around the EXISTING generator bench/inference/gen_deeptaxonomy.py — this
# suite REUSES that generator (it is NOT rewritten here), it just promotes it to a first-class,
# cache-backed bench/<suite>/gen.sh entry point mirroring bench/sp2b/gen.sh + bench/lubm/gen.sh.
#
#   bench/deep-taxonomy/gen.sh [depth=1000] [out=/tmp/deep-taxonomy/dt-<depth>.n3]
#
# Emits ONE path on stdout: the generated Notation3 document for the requested DEPTH tier:
#   :i a :n0 .                                       (one instance fact)
#   { ?x a ?c . ?c :sc ?d } => { ?x a ?d } .         (one transitivity meta-rule)
#   :n0 :sc :n1 . :n1 :sc :n2 . ... :n{depth-1} :sc :n{depth} .   (a depth-deep :sc chain)
#
# run.sh consumes it: materialize the N3 forward closure with `sparq-cli reason … n3`, then run
# query.rq over the closure and ASSERT both the closure triple-count and the query row-count
# against expected.tsv (deterministic — see run.sh + README.md).
#
# WHY REUSE gen_deeptaxonomy.py (no rewrite): it is the canonical DT generator already cited by
# bench/inference/eye-comparison.md (the EYE head-to-head), so the closure sizes it produces are
# the SAME ones the EYE comparison + the published RR-2023 DeepTaxonomy literature use. The output
# is FULLY DETERMINISTIC (pure function of `depth`, no RNG, byte-identical every run), so the
# corpus — and thus the closure size — is reproducible per-commit.
#
# HERMETICITY / CI NOTES:
#  - Network: NONE. The generator is in-repo pure Python (stdlib only); nothing is fetched.
#  - Toolchain: python3 ONLY (already required by the bench harness). No g++/javac/rapper/java,
#    so unlike lubm/watdiv/bsbm this suite has NO heavyweight toolchain guard — it runs on the
#    per-commit tier by default (see scripts/ci-bench.sh deeptax hook).
#  - Output is cached under $DEEPTAX_CACHE (default /tmp/deep-taxonomy), gitignored + regenerable
#    (see bench/CATALOG.md disk discipline). Caller cleans up.
#
# Output is Notation3; load into sparq with format `n3` (it carries the `{…}=>{…}` rule). Caller
# cleans up the corpus.
set -euo pipefail

DEPTH="${1:-1000}"
CACHE="${DEEPTAX_CACHE:-/tmp/deep-taxonomy}"
OUT="${2:-$CACHE/dt-${DEPTH}.n3}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GEN="$ROOT/bench/inference/gen_deeptaxonomy.py"

if ! command -v python3 >/dev/null 2>&1; then
  echo "[deeptax] ERROR: 'python3' not found (needed to run gen_deeptaxonomy.py)." >&2
  exit 1
fi
if [ ! -f "$GEN" ]; then
  echo "[deeptax] ERROR: generator missing at $GEN" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"
# Deterministic: same bytes every run for a given depth. Regenerate only if absent so the cache
# (like lubm/sp2b) makes steady-state per-commit runs skip the (cheap) generation step too.
if [ ! -s "$OUT" ]; then
  python3 "$GEN" "$DEPTH" > "$OUT.tmp"
  mv "$OUT.tmp" "$OUT"
fi

echo "$OUT"
