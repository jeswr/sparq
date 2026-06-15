#!/usr/bin/env bash
# [OPUS-4.8] (sq-tf8n) GeoSPARQL benchmark corpus generator. The geo suite uses a FIXED
# CRS84 point corpus (~100k seeded random POINT(lon lat) literals over an 8°x8° window)
# as its data substrate. The corpus is generated DETERMINISTICALLY by shelling out to the
# sparq-geo `bench_geo gen` example — so the committed expected.tsv counts and the
# bench_geo in-process fallback corpus are byte-identical (no f64-formatting drift between
# Rust and shell). Mirrors bench/shacl/gen.sh / bench/lubm/gen.sh:
#
#   bench/geo/gen.sh [n=100000]
#
# emits ONE path on stdout: the cached corpus N-Triples file under /tmp/geo. (Unlike LUBM
# there is no second TBox line — the geo workloads validate the raw point corpus.) The
# corpus is gitignored + regenerable; CI may key actions/cache on /tmp/geo.
#
# Toolchain: just cargo (no javac/rapper/Docker) — the corpus is pure Rust. The bench_geo
# example is built on demand if absent.
set -euo pipefail

N="${1:-100000}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
CACHE="${GEO_CACHE_DIR:-/tmp/geo}"
mkdir -p "$CACHE"
OUT="$CACHE/points-${N}.nt"
BIN="${BENCH_GEO:-$ROOT/target/release/examples/bench_geo}"

# Build the corpus generator on demand (sparq-geo is isolated — its own --example).
if [ ! -x "$BIN" ]; then
  ( cd "$ROOT" && cargo build --release -q -p sparq-geo --example bench_geo ) >&2
  BIN="$ROOT/target/release/examples/bench_geo"
fi

# Deterministic: regenerate only if missing (the seed/window are pinned in the binary).
if [ ! -s "$OUT" ]; then
  "$BIN" gen "$N" > "$OUT.tmp"
  mv "$OUT.tmp" "$OUT"
fi

echo "$OUT"
