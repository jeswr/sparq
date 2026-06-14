#!/usr/bin/env bash
# [OPUS-4.8] BSBM (Berlin SPARQL Benchmark) corpus generator — fetch-once-and-cache the
# canonical prebuilt `bsbmtools` distribution, then emit a FIXED, deterministic e-commerce
# corpus (products / vendors / offers / reviews) as N-Triples.
#
#   bench/bsbm/gen.sh [product_count=300] [out=/tmp/bsbm/bsbm-pc<N>.nt]
#
# Why the real bsbmtools (not a fallback): bsbmtools is the canonical BSBM data generator
# (Bizer/Schultz). The v0.2 SourceForge distribution ships a PREBUILT lib/bsbm.jar plus its
# dependency jars (ssj, log4j, jdom), so we need only a JRE (JDK 21 is installed) — NO Maven,
# NO Ant, NO compile step. It is DETERMINISTIC: at a given -pc the generator emits byte-
# identical N-Triples every run (seeded RNG; sha256-verified below), so the corpus is
# reproducible per-commit and the instantiated Explore queries have stable result sizes.
#
# Scale knob: -pc <product count>. -pc 300 ~= 115,987 triples (the design's ~100k per-commit
# target). -fc switches on forward chaining (materialises the rdfs:subClassOf product-type
# closure into the data) so the Explore queries' `?p a <leaf-type>` patterns match without a
# reasoner — this matches how the instantiated bench/bsbm/queries are written.
#
# HERMETICITY / CI NOTES:
#  - Network: one HTTPS GET of a 2.4 MB zip from SourceForge, pinned by sha256 (ZIP_SHA256).
#    The jars + corpus are cached, so steady-state per-commit runs do NO network.
#  - Toolchain: a JRE only (`java`). The distribution is prebuilt — no javac/mvn/ant.
#  - Output is N-Triples (`-s nt`); load it into sparq with format `ntriples`.
#  - Pinned source: SourceForge bsbmtools v0.2 (mirror: github.com/afs/BSBM, the Explore query
#    TEMPLATES the bench/bsbm/queries are instantiated from live in queries/explore/ there).
# Caller cleans up the corpus (gitignored, regenerable — see bench/CATALOG.md disk discipline).
set -euo pipefail

PC="${1:-300}"
CACHE="${BSBM_CACHE:-/tmp/bsbm}"
OUT="${2:-$CACHE/bsbm-pc$PC.nt}"

# Pinned: canonical bsbmtools v0.2 full distribution (prebuilt jars + query templates + name
# files). SourceForge file name is bsbmtools-v0.2.zip (note the 'v').
ZIP_URL="https://sourceforge.net/projects/bsbmtools/files/bsbmtools/bsbmtools-0.2/bsbmtools-v0.2.zip/download"
ZIP_SHA256="40f5e59baadec3af0014b7647989d3e0fc0476af25e84a4bc9d7f8cd81520aaa"

TOOLS="$CACHE/bsbmtools-0.2"
mkdir -p "$CACHE"

# ---- 1. fetch + verify (skipped if the distribution is already unpacked) -----------------
if [ ! -f "$TOOLS/lib/bsbm.jar" ]; then
  ZIP="$CACHE/bsbmtools-v0.2.zip"
  if [ ! -f "$ZIP" ]; then
    echo "[bsbm] downloading bsbmtools v0.2 ($ZIP_URL)..." >&2
    curl -fsSL -o "$ZIP" "$ZIP_URL"
  fi
  echo "$ZIP_SHA256  $ZIP" | sha256sum -c - >&2
  echo "[bsbm] unpacking prebuilt bsbmtools..." >&2
  rm -rf "$TOOLS"
  unzip -q -o "$ZIP" -d "$CACHE"
  [ -f "$TOOLS/lib/bsbm.jar" ] || { echo "[bsbm] ERROR: lib/bsbm.jar missing after unpack" >&2; exit 1; }
fi

# ---- 2. generate the fixed corpus (deterministic; cached) --------------------------------
# The generator writes <fn>.nt relative to CWD and reads its name pools (titlewords.txt /
# givennames.txt) from CWD, so run it from inside the tools dir, then move the result.
if [ ! -s "$OUT" ]; then
  echo "[bsbm] generating -fc -pc $PC -s nt -> $OUT ..." >&2
  CP="."; for j in "$TOOLS"/lib/*.jar; do CP="$CP:$j"; done
  ( cd "$TOOLS" \
      && rm -f "bsbm-pc$PC.nt" \
      && java -cp "$CP" -Xmx512M benchmark.generator.Generator \
           -fc -pc "$PC" -s nt -fn "bsbm-pc$PC" >/dev/null )
  mv "$TOOLS/bsbm-pc$PC.nt" "$OUT"
fi
echo "$OUT"
