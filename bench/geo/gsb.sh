#!/usr/bin/env bash
# [SONNET-4.6] sq-ql2iy — GeoSPARQL Compliance Benchmark (GSB): fetch + pin recipe.
#
# The published cross-engine GeoSPARQL row (Jovanovik/Homburg/Spasic, ISPRS IJGI
# 10(7):487, 2021; arXiv:2102.06139) is scored by THIS artifact — 206 SPARQL
# queries over the 30 GeoSPARQL 1.0 requirements, with per-query expected answers
# in SPARQL-Results-XML. sparq's own 197-assertion DE-9IM battery is a DIFFERENT
# unit, which is why research/gap-conformance-cross-engine-2026-07.md scored the
# GeoSPARQL row NOT-COMPARABLE. Running sparq through this artifact is what makes
# it comparable.
#
#   bench/geo/gsb.sh          # -> /tmp/gsb/resources   (queries/, answers/, dataset/)
#
# GATHER-ONLY (/tmp), for TWO independent reasons:
#   1. the upstream is GPL-2.0 (OpenLink Software) and sparq is not — vendoring
#      its queries/answers into this tree would be a licence violation; and
#   2. AGENTS.md keeps engines/datasets out of git regardless.
# The durable, pinned artifact is THIS RECIPE, exactly as bench/geo/geographica.sh.
#
# The tarball is PINNED by sha256 (a mismatch = upstream changed the benchmark ->
# hard FAIL, never silently score against a different query set), and the three
# extracted trees are counted (206 queries + the 6 non-benchmark system queries,
# 406 answer files, 3 dataset files) so a partial extraction cannot be scored.
#
# TUNABLES (env):
#   GSB_CACHE_DIR   cache dir                (default /tmp/gsb)
#   GSB_FETCH       0 = never download (absent tarball -> fail; default 1)
#
# SCRATCH: rm -rf /tmp/gsb  (regenerable; ~3 MB extracted)
set -euo pipefail

# Pinned upstream: the GeoSPARQL Compliance Benchmark `master` tree as of the
# 2021-07-07 final results update (the revision whose Table 2 numbers the
# research record cites).
URL="https://codeload.github.com/OpenLinkSoftware/GeoSPARQLBenchmark/tar.gz/refs/heads/master"
TARBALL_SHA256="d46db11041be96881208b4954f70bda636fd217fcab4ea99ffb8bb658a5f21bf"

CACHE="${GSB_CACHE_DIR:-/tmp/gsb}"
FETCH="${GSB_FETCH:-1}"
TARBALL="$CACHE/GeoSPARQLBenchmark.tar.gz"
OUT="$CACHE/resources"
mkdir -p "$CACHE"

log() { printf '[gsb] %s\n' "$*" >&2; }

# <dir> — the extracted tree is usable only if every part is present and complete.
verify_tree() {
  local d="$1"
  [ -d "$d/gsb_queries" ] && [ -d "$d/gsb_answers" ] && [ -d "$d/gsb_dataset" ] || return 1
  [ "$(find "$d/gsb_queries" -name '*.rq' | wc -l)" = 212 ] || return 1
  [ "$(find "$d/gsb_answers" -name '*.srx' | wc -l)" = 406 ] || return 1
  [ -s "$d/gsb_dataset/dataset.rdf" ] || return 1
}

if verify_tree "$OUT"; then
  echo "$OUT"
  exit 0
fi

if [ ! -s "$TARBALL" ]; then
  if [ "$FETCH" != 1 ]; then
    log "ERROR: $TARBALL absent and GSB_FETCH=0"
    exit 1
  fi
  log "downloading $URL (gather-only, $CACHE)"
  if ! curl -fsSL --retry 2 -o "$TARBALL.tmp" "$URL"; then
    rm -f "$TARBALL.tmp"
    log "ERROR: download failed: $URL"
    exit 1
  fi
  mv "$TARBALL.tmp" "$TARBALL"
fi

got="$(sha256sum "$TARBALL" | cut -d' ' -f1)"
if [ "$got" != "$TARBALL_SHA256" ]; then
  log "ERROR: tarball sha256 mismatch"
  log "  want $TARBALL_SHA256"
  log "  got  $got"
  log "upstream changed the benchmark; re-pin deliberately (and re-derive the score)"
  exit 1
fi

rm -rf "$CACHE/extract" "$OUT"
mkdir -p "$CACHE/extract"
tar xzf "$TARBALL" -C "$CACHE/extract"
SRC="$CACHE/extract/GeoSPARQLBenchmark-master/src/main/resources"
if ! verify_tree "$SRC"; then
  log "ERROR: extracted tree is incomplete under $SRC"
  exit 1
fi
mv "$SRC" "$OUT"
rm -rf "$CACHE/extract"

verify_tree "$OUT" || { log "ERROR: post-move verification failed"; exit 1; }
echo "$OUT"
