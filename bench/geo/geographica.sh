#!/usr/bin/env bash
# [FABLE-5] sq-hmd7l.29 — Geographica real-world subset: fetch + pin + normalise
# recipe for the LGD/GeoNames slices of the Geographica benchmark (Garbis,
# Kyzirakos, Koubarakis — ISWC 2013; the reviewer-recognised real-world
# GeoSPARQL suite). Mirrors bench/geo/gen.sh's contract: emit ONE path on
# stdout — the merged, normalised N-Triples corpus under the cache dir.
#
#   bench/geo/geographica.sh          # -> /tmp/geographica/lgd-geonames.nt
#
# GATHER-ONLY (/tmp): the upstream tarballs and the derived corpus are NEVER
# committed (engines/datasets stay out of git per AGENTS.md); the recipe is the
# durable, pinned artifact. Everything is verified before use:
#
#   * upstream tarballs are PINNED by sha256 (a hash mismatch = upstream
#     changed the data -> hard FAIL, never silently benchmark different data);
#   * the merged corpus is PINNED by sha256 too (triple-check: the whole
#     fetch->normalise->merge pipeline is deterministic).
#
# NORMALISATION (documented, deterministic): every wktLiteral in the upstream
# slices carries the anchor `<http://www.opengis.net/def/crs/EPSG/4326> ` with
# coordinates written LON/LAT — a Strabon-era artifact (spec-correct EPSG:4326
# is LAT/LON, and the OGC form of the IRI has a `/0/` version segment). Left
# as-is, jena-geosparql would axis-swap the literals per the EPSG registry
# (geometries land in the Indian Ocean) while sparq-geo classifies the
# non-OGC-form IRI as an opaque `Crs::Other` (CRS-mismatch vs the queries'
# bare-CRS84 constants). The recipe strips the anchor, making every literal
# bare CRS84 (long/lat as written) — the semantics Geographica/Strabon
# intended — so BOTH engines interpret the coordinates identically and the
# result-set-size cross-check is meaningful. Query geometry constants
# (bench/geo/queries-geographica/*.rq) are bare CRS84 upstream already.
#
# TUNABLES (env):
#   GEOGRAPHICA_CACHE_DIR   cache dir                 (default /tmp/geographica)
#   GEO_FETCH_GEOGRAPHICA   0 = never download (absent tarball -> fail; default 1)
#
# SCRATCH: rm -rf /tmp/geographica  (regenerable; ~160 MB extracted)
set -euo pipefail

BASE_URL="https://geographica2.di.uoa.gr/datasets"
# Pinned upstream tarballs (Last-Modified 2022-06-28 on geographica2.di.uoa.gr).
GEONAMES_SHA256="73c589112d876259f7dffa5d4a0764ff99df1f9aac23ba85a96b5cf16336aee2"
LGD_SHA256="c13e0594707d28fa2d0c9da9819991ed71dd4700f9e7049126278d2d0fb1aefd"
# The merged normalised corpus (geonames.nt then linkedgeodata.nt, anchors
# stripped): 563 885 triples / 34 087 wktLiterals (21 990 POINT + 12 097
# LINESTRING + …).
MERGED_SHA256_EXPECTED="48810b8d2be084c9757157d4e444b1b77da3a79a27f6ee9d9d38f747c0cdd6d7"

CACHE="${GEOGRAPHICA_CACHE_DIR:-/tmp/geographica}"
FETCH="${GEO_FETCH_GEOGRAPHICA:-1}"
OUT="$CACHE/lgd-geonames.nt"
mkdir -p "$CACHE"

log() { printf '[geographica] %s\n' "$*" >&2; }

# Deterministic + verified: reuse the cached corpus only if its pin matches.
if [ -s "$OUT" ]; then
  got="$(sha256sum "$OUT" | cut -d' ' -f1)"
  if [ "$got" = "$MERGED_SHA256_EXPECTED" ]; then
    echo "$OUT"
    exit 0
  fi
  log "cached corpus sha256 mismatch (got $got); rebuilding"
  rm -f "$OUT"
fi

fetch_verify() { # <name.tar.xz> <sha256>
  local tarball="$CACHE/$1" want="$2" got
  if [ ! -s "$tarball" ]; then
    if [ "$FETCH" != 1 ]; then
      log "ERROR: $tarball absent and GEO_FETCH_GEOGRAPHICA=0"
      return 1
    fi
    log "downloading $BASE_URL/$1 (gather-only, /tmp)"
    if ! curl -fsSL --retry 2 -o "$tarball.tmp" "$BASE_URL/$1"; then
      rm -f "$tarball.tmp"
      log "ERROR: download failed: $BASE_URL/$1"
      return 1
    fi
    mv "$tarball.tmp" "$tarball"
  fi
  got="$(sha256sum "$tarball" | cut -d' ' -f1)"
  if [ "$got" != "$want" ]; then
    log "ERROR: $1 sha256 mismatch: got $got want $want (upstream changed; re-pin deliberately or stop)"
    return 1
  fi
}

fetch_verify geonames.tar.xz "$GEONAMES_SHA256"
fetch_verify linkedgeodata.tar.xz "$LGD_SHA256"

log "extracting + normalising (strip the EPSG/4326 lon-lat anchor -> bare CRS84)"
for name in geonames linkedgeodata; do
  if [ ! -s "$CACHE/$name.nt" ]; then
    tar -xJf "$CACHE/$name.tar.xz" -C "$CACHE" "$name.nt"
  fi
done
# Merge order is PINNED (geonames then linkedgeodata); the anchor substitution
# is anchored to the literal-opening quote so nothing else can match.
sed 's|"<http://www.opengis.net/def/crs/EPSG/4326> |"|g' \
  "$CACHE/geonames.nt" "$CACHE/linkedgeodata.nt" > "$OUT.tmp"

got="$(sha256sum "$OUT.tmp" | cut -d' ' -f1)"
if [ "$got" != "$MERGED_SHA256_EXPECTED" ]; then
  rm -f "$OUT.tmp"
  log "ERROR: merged corpus sha256 mismatch: got $got want $MERGED_SHA256_EXPECTED"
  exit 1
fi
mv "$OUT.tmp" "$OUT"
echo "$OUT"
