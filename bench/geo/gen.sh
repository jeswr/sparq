#!/usr/bin/env bash
# [OPUS-4.8] (sq-tf8n) GeoSPARQL benchmark corpus generator. The geo suite uses a FIXED
# CRS84 point corpus (~100k seeded random POINT(lon lat) literals over an 8°x8° window)
# as its data substrate. The corpus is generated DETERMINISTICALLY by shelling out to the
# sparq-geo `bench_geo gen` example — so the committed expected.tsv counts and the
# bench_geo in-process fallback corpus are byte-identical (no f64-formatting drift between
# Rust and shell). Mirrors bench/shacl/gen.sh / bench/lubm/gen.sh:
#
#   bench/geo/gen.sh [n=100000] [direct|feature]
#
# emits ONE path on stdout: the cached corpus N-Triples file under /tmp/geo. (Unlike LUBM
# there is no second TBox line — the geo workloads validate the raw point corpus.) The
# corpus is gitignored + regenerable; CI may key actions/cache on /tmp/geo.
#
# `feature` emits a Jena-indexable variant without changing the fixed corpus:
# each entity points to a geometry node via geo:hasGeometry, and the geometry
# node carries the same geo:asWKT literal. [SONNET-4.6] (sq-enfy3)
#
# Toolchain: just cargo (no javac/rapper/Docker) — the corpus is pure Rust. The bench_geo
# example is built on demand if absent.
set -euo pipefail

N="${1:-100000}"
MODEL="${2:-direct}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
CACHE="${GEO_CACHE_DIR:-/tmp/geo}"
mkdir -p "$CACHE"
OUT="$CACHE/points-${N}.nt"
FEATURE_OUT="$CACHE/points-feature-${N}.nt"
BIN="${BENCH_GEO:-$ROOT/target/release/examples/bench_geo}"

if [ "$MODEL" != direct ] && [ "$MODEL" != feature ]; then
  echo "usage: $0 [n] [direct|feature]" >&2
  exit 2
fi

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

if [ "$MODEL" = feature ]; then
  if [ ! -s "$FEATURE_OUT" ] || [ "$OUT" -nt "$FEATURE_OUT" ]; then
    if ! awk '
      BEGIN { OFS = " " }
      {
        if (NF != 5 || $1 !~ /^<.*>$/ ||
            $2 != "<http://www.opengis.net/ont/geosparql#asWKT>") {
          print "gen.sh: unexpected corpus shape at line " NR > "/dev/stderr"
          exit 1
        }
        entity = $1
        geometry = substr(entity, 1, length(entity) - 1) "/geometry>"
        $1 = geometry
        print entity, "<http://www.opengis.net/ont/geosparql#hasGeometry>", geometry, "."
        print
      }
    ' "$OUT" > "$FEATURE_OUT.tmp"; then
      rm -f "$FEATURE_OUT.tmp"
      exit 1
    fi
    EXPECTED_LINES=$((2 * $(wc -l < "$OUT")))
    if [ "$(wc -l < "$FEATURE_OUT.tmp")" -ne "$EXPECTED_LINES" ]; then
      echo "feature transform produced unexpected line count" >&2
      rm -f "$FEATURE_OUT.tmp"
      exit 1
    fi
    mv "$FEATURE_OUT.tmp" "$FEATURE_OUT"
  fi
  echo "$FEATURE_OUT"
else
  echo "$OUT"
fi
