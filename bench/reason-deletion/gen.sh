#!/usr/bin/env bash
# [FABLE-5] bench/reason-deletion corpus wrapper (bead sq-31fza) — cache-backed entry point
# mirroring bench/deep-taxonomy/gen.sh + bench/lubm/gen.sh.
#
#   bench/reason-deletion/gen.sh [units=15000] [out=$REASONDEL_CACHE/rd-<units>.nt]
#
# Emits ONE path on stdout: the deterministic N-Triples ABox for the requested UNITS tier
# (1 unit = 1 athlete + 1 result = 8 triples; see gen_reason_deletion.py for the schema).
#
# HERMETICITY: python3 stdlib only, no network. Output cached under $REASONDEL_CACHE
# (default /tmp/reason-deletion), gitignored + regenerable; byte-identical per <units>,
# so the cache makes steady-state runs skip generation. Caller cleans up.
set -euo pipefail

UNITS="${1:-15000}"
CACHE="${REASONDEL_CACHE:-/tmp/reason-deletion}"
OUT="${2:-$CACHE/rd-${UNITS}.nt}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GEN="$ROOT/bench/reason-deletion/gen_reason_deletion.py"

if ! command -v python3 >/dev/null 2>&1; then
  echo "[reasondel] ERROR: 'python3' not found (needed to run gen_reason_deletion.py)." >&2
  exit 1
fi
if [ ! -f "$GEN" ]; then
  echo "[reasondel] ERROR: generator missing at $GEN" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"
# Deterministic: same bytes every run for a given <units>; regenerate only if absent.
if [ ! -s "$OUT" ]; then
  python3 "$GEN" "$UNITS" > "$OUT.tmp"
  mv "$OUT.tmp" "$OUT"
fi

echo "$OUT"
