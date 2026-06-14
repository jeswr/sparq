#!/usr/bin/env bash
# [OPUS-4.8] SP2Bench corpus generator — build-once-and-cache the REAL Freiburg DBIS
# generator (`sp2b_gen`), then emit a FIXED, deterministic DBLP-in-RDF corpus.
#
#   bench/sp2b/gen.sh [triples=250000] [out=/tmp/sp2b/sp2b-<triples>.ttl]
#
# Why the real generator (not a fallback): the official `sp2b_gen` is small, BSD-licensed
# C++ with a PLAIN Makefile (no cmake), and builds hermetically from source in ~30s given
# only g++. It produces the canonical, semantics-pinned SP2Bench corpus, so query result
# sizes match the published reference — a fake/approximate generator would defeat the point
# of adopting a well-known suite. The generator is DETERMINISTIC (same bytes every run at a
# given -t, verified by sha256), so the corpus is reproducible per-commit.
#
# HERMETICITY / CI NOTES:
#  - Network: one HTTPS GET of a 2.7 MB tarball from dbis.informatik.uni-freiburg.de,
#    pinned by sha256 (TARBALL_SHA256). The binary + corpus are cached, so steady-state
#    per-commit runs do NO network and NO rebuild.
#  - Toolchain: g++ only (already required to build the Rust toolchain's C deps). NO cmake.
#  - Build flags: -O2, NOT -O3 — the upstream Makefile WARNS that -O3 is not
#    semantics-preserving (FP reassociation) and changes the generated document. We keep -O2.
#  - Two source PATCHES are applied (idempotently) for modern GCC + non-x86:
#      (1) add <cstring>/<cstdlib> includes (2008 code predates the stricter header rules);
#      (2) namepoolfile.cpp: `char c = fgetc(...)` -> `int c`. fgetc returns int and EOF is
#          -1; on aarch64 `char` is UNSIGNED so the original `while (c != EOF)` never
#          terminated (infinite loop on `reading name files...`). int c fixes it WITHOUT
#          touching generation logic, so output stays byte-identical to the x86 reference.
#    These patches are mechanical portability fixes; they do not alter the RDF produced.
#
# Output is Turtle/N3 (the generator emits @prefix + prefixed names + ^^xsd:string), so load
# it into sparq with format `turtle` (NOT ntriples). Caller cleans up the corpus (gitignored,
# regenerable — see bench/CATALOG.md disk discipline).
set -euo pipefail

TRIPLES="${1:-250000}"
CACHE="${SP2B_CACHE:-/tmp/sp2b}"
OUT="${2:-$CACHE/sp2b-$TRIPLES.ttl}"

TARBALL_URL="https://dbis.informatik.uni-freiburg.de/content/projects/SP2B/docs/sp2b-v1_01-full.tar.gz"
# Pinned: official sp2b v1.01 full distribution (source + queries + name files).
TARBALL_SHA256="6640506cf2e2d142ea3435b1b8cc3999879349ebcd1a3cdc601db831f5cb3d73"

BIN="$CACHE/sp2b_gen"
SRC="$CACHE/src"
mkdir -p "$CACHE"

# ---- 1. fetch + verify (skipped if the built binary is already cached) -------------------
if [ ! -x "$BIN" ]; then
  TAR="$CACHE/sp2b-v1_01-full.tar.gz"
  if [ ! -f "$TAR" ]; then
    echo "[sp2b] downloading generator source ($TARBALL_URL)..." >&2
    curl -fsSL -o "$TAR" "$TARBALL_URL"
  fi
  echo "$TARBALL_SHA256  $TAR" | sha256sum -c - >&2

  # ---- 2. unpack + apply portability patches --------------------------------------------
  rm -rf "$CACHE/sp2b"
  tar xzf "$TAR" -C "$CACHE"
  rm -rf "$SRC"; mkdir -p "$SRC"
  cp -r "$CACHE/sp2b/src/." "$SRC/"
  # name pools live next to the binary (read from cwd at generation time)
  cp "$CACHE/sp2b/bin/"*.txt "$CACHE/" 2>/dev/null || cp "$SRC/"*.txt "$CACHE/"

  cd "$SRC"
  # (a) missing <cstring>/<cstdlib> includes (idempotent: only prepend if absent)
  for f in namepoolfile.cpp missingcites.cpp namepoolmgr.cpp; do
    head -1 "$f" | grep -q '<cstring>' || { printf '#include <cstring>\n%s' "$(cat "$f")" > "$f.t" && mv "$f.t" "$f"; }
  done
  head -1 volumemgr.cpp | grep -q '<cstdlib>' || { printf '#include <cstdlib>\n%s' "$(cat volumemgr.cpp)" > volumemgr.t && mv volumemgr.t volumemgr.cpp; }
  # (b) unsigned-char EOF infinite loop (aarch64) — fgetc returns int.
  sed -i 's/^    char c;$/    int c; \/\/ [PORT] fgetc->int; EOF unrepresentable in unsigned char (aarch64)/' namepoolfile.cpp
  # (c) drop the hardcoded -m32 (the upstream Makefile targets 2008 x86); build native.
  #     KEEP -O2 (NOT -O3) per the upstream determinism warning.
  sed -i 's/g++ -Wall -m32 -O2/g++ -w -O2/' Makefile

  # ---- 3. build (no cmake; ~30s) --------------------------------------------------------
  echo "[sp2b] building sp2b_gen (g++ -O2, native)..." >&2
  make >/dev/null
  cp "$SRC/sp2b_gen" "$BIN"
  echo "[sp2b] cached generator: $BIN" >&2
fi

# ---- 4. generate the fixed corpus (deterministic; cached) --------------------------------
if [ ! -s "$OUT" ]; then
  echo "[sp2b] generating $TRIPLES triples -> $OUT ..." >&2
  # generator reads its name files (givennames/familynames/titlewords.txt) from cwd, and
  # writes its output relative to cwd; run from the cache dir then move into place.
  ( cd "$CACHE" && "$BIN" -t "$TRIPLES" "sp2b-$TRIPLES.ttl.tmp" >/dev/null )
  mv "$CACHE/sp2b-$TRIPLES.ttl.tmp" "$OUT"
fi
echo "$OUT"
