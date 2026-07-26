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
# A CACHED extraction is never accepted on those counts alone — counting files
# cannot see a stale or locally edited corpus, and scoring one would make the
# recorded result irreproducible while the header still claimed a hard pin. So
# extraction also records a PROOF next to the tree ($CACHE/.verified): the
# tarball digest it came from, plus a digest over the CONTENT of every benchmark
# file. The cache is reused only when that proof still matches — i.e. only when
# the tree provably is the unmodified extraction of a tarball matching the pin
# ABOVE. Anything else (no proof, an edited file, a re-pin) re-extracts, which
# re-checks the tarball. `bench/geo/gsb.sh --self-test` pins that rule.
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
MARKER="$CACHE/.verified"

log() { printf '[gsb] %s\n' "$*" >&2; }

# <dir> — the extracted tree is usable only if every part is present and complete.
verify_tree() {
  local d="$1"
  [ -d "$d/gsb_queries" ] && [ -d "$d/gsb_answers" ] && [ -d "$d/gsb_dataset" ] || return 1
  [ "$(find "$d/gsb_queries" -name '*.rq' | wc -l)" = 212 ] || return 1
  [ "$(find "$d/gsb_answers" -name '*.srx' | wc -l)" = 406 ] || return 1
  [ -s "$d/gsb_dataset/dataset.rdf" ] || return 1
}

# <dir> — a digest over the CONTENT (and path) of every benchmark file, so a
# count-preserving edit changes it. `sha256sum` output already carries the path;
# the file list is sorted so the digest does not depend on readdir order.
tree_digest() {
  ( cd "$1" && find gsb_queries gsb_answers gsb_dataset -type f -print0 \
      | LC_ALL=C sort -z | xargs -0 sha256sum ) | sha256sum | cut -d' ' -f1
}

# <dir> — the proof recorded beside an extraction: the PINNED tarball digest it
# was extracted from (so re-pinning invalidates every cache) and the tree's own
# content digest (so editing the corpus does).
marker_line() { printf '%s %s' "$TARBALL_SHA256" "$(tree_digest "$1")"; }

self_test() {  # hermetic: fabricates a correctly-shaped tree, no network, no corpus
  local me out want pass=0 fail=0 i
  me="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  # NOT `local`: the EXIT trap below runs after this function has returned.
  sandbox="$(mktemp -d)"
  trap 'rm -rf "$sandbox"' EXIT
  out="$sandbox/resources"
  mkdir -p "$out/gsb_queries" "$out/gsb_answers" "$out/gsb_dataset"
  for ((i = 1; i <= 212; i++)); do printf 'SELECT * {}\n' > "$out/gsb_queries/q$i.rq"; done
  for ((i = 1; i <= 406; i++)); do printf '<sparql/>\n' > "$out/gsb_answers/a$i.srx"; done
  printf '<rdf:RDF/>\n' > "$out/gsb_dataset/dataset.rdf"

  # Each case runs the REAL script over the sandbox with fetching disabled and no
  # tarball present, so "accepted from cache" is exactly "exit 0", and any
  # rejection shows up as the re-extract path failing for want of a tarball.
  probe() {  # <case> <accept|reject>
    local rc=0 got
    got="$(GSB_CACHE_DIR="$sandbox" GSB_FETCH=0 bash "$me" 2>/dev/null)" || rc=$?
    if [ "$2" = accept ] && [ "$rc" = 0 ] && [ "$got" = "$out" ]; then
      pass=$((pass + 1))
    elif [ "$2" = reject ] && [ "$rc" != 0 ]; then
      pass=$((pass + 1))
    else
      fail=$((fail + 1)); printf 'CASE FAILED: %s (rc=%s out=%s)\n' "$1" "$rc" "$got"
    fi
  }

  marker_line "$out" > "$sandbox/.verified"
  probe "an unmodified tree with its proof is reused" accept

  want="$(cat "$sandbox/.verified")"
  # THE REGRESSION: an edit that preserves every file count. Counting files
  # cannot see it; the content digest must.
  printf 'SELECT ?tampered {}\n' > "$out/gsb_queries/q7.rq"
  probe "a count-preserving edit is not accepted from cache" reject
  printf 'SELECT * {}\n' > "$out/gsb_queries/q7.rq"

  rm -f "$sandbox/.verified"
  probe "a tree with no recorded proof is not accepted" reject

  printf '%s %s' "0000000000000000000000000000000000000000000000000000000000000000" \
    "${want##* }" > "$sandbox/.verified"
  probe "a proof naming a different tarball digest is not accepted" reject

  printf '%s' "$want" > "$sandbox/.verified"
  probe "restoring the tree and its proof makes it reusable again" accept

  printf 'gsb.sh --self-test: %s passed, %s failed.\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

if [ "${1:-}" = "--self-test" ]; then
  self_test  # before the cache dir is created: the self-test is hermetic
  exit
fi
mkdir -p "$CACHE"

# Reuse a cached extraction only against its recorded proof (see the header):
# complete, provably unedited, and provably from a tarball matching the pin.
if verify_tree "$OUT" && [ -s "$MARKER" ] &&
   [ "$(cat "$MARKER")" = "$(marker_line "$OUT")" ]; then
  echo "$OUT"
  exit 0
fi
if [ -e "$MARKER" ]; then
  log "cached tree does not match its recorded proof (edited, incomplete, or re-pinned); re-extracting"
  rm -f "$MARKER"
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
# Record the proof LAST: it exists only for a tree that reached here, i.e. one
# extracted from a tarball whose sha256 matched the pin above.
marker_line "$OUT" > "$MARKER"
echo "$OUT"
