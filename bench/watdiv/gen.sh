#!/usr/bin/env bash
# [OPUS-4.8] WatDiv (Waterloo SPARQL Diversity Benchmark) corpus generator —
# build-once-and-cache the REAL Waterloo DSG generator (`watdiv`), then emit a FIXED,
# deterministic WatDiv corpus (N-Triples).
#
#   bench/watdiv/gen.sh [scale-factor=1] [out=/tmp/watdiv/watdiv-sf<SF>.nt]
#
# Why the real generator (not a fallback): the official WatDiv C++ generator produces the
# canonical, semantics-pinned WatDiv corpus (the WSDBM e-commerce/social schema), so the
# 20 Basic-Testing query shapes (Linear/Star/Snowflake/Complex) exercise the genuine WatDiv
# structural diversity — a fake/approximate generator would defeat the point of adopting a
# well-known suite. We make it DETERMINISTIC (same row counts every run at a given SF) with
# two mechanical seed patches (below), so the corpus is reproducible per-commit.
#
# PINNED SOURCE: official WatDiv v0.6 source tarball from the University of Waterloo Data
# Systems Group (the canonical dsg-uwaterloo/watdiv GitHub repo is a docs-only Jekyll site;
# the buildable source ships as this tarball, linked from its README).
#   URL    : https://dsg.uwaterloo.ca/watdiv/watdiv_v06.tar
#   sha256 : fb8d930b74b3fbc8f948101bfaf658a90d2f74002f1fefda465c45ffd33a71d2
#
# HERMETICITY / CI NOTES:
#  - Network: one HTTPS GET of a ~300 KB tarball, pinned by sha256 (TARBALL_SHA256). The
#    binary + corpus are cached, so steady-state per-commit runs do NO network, NO rebuild.
#  - Toolchain: g++ + Boost only (apt: libboost-all-dev). NO cmake. The v0.6 source builds
#    CLEAN on g++ 13 / Boost 1.83 as-is (Makefile already pins -std=c++0x + -w; the code uses
#    no bare <cstdint> types, so the classic modern-GCC breakage does not bite). The only
#    edits we apply are the three patches below — none of which change the RDF semantics.
#  - Wordlist: upstream hard-codes /usr/share/dict/words (the `wamerican` package) for STRING
#    literals. To stay hermetic we VENDOR a fixed ASCII wordlist (bench/watdiv/files/words.txt)
#    and point the generator at it via the WATDIV_WORDS env var (patch 3). The per-query row
#    counts are INDEPENDENT of the wordlist CONTENT — they bind to typed/structural entities,
#    not literal text (verified by regenerating with an alternate wordlist: identical counts).
#    firstnames/lastnames ship inside the tarball; we use those.
#
# THREE SOURCE PATCHES (idempotent; mechanical; do NOT alter generation logic):
#   (1) src/model.cpp boost mt19937 seed  time(0)    -> 1u   (deterministic data RNG)
#   (2) src/model.cpp srand               time(NULL) -> 1u   (deterministic data RNG)
#   (3) src/model.cpp dictionary::init    read words/firstnames/lastnames paths from the
#       WATDIV_WORDS / WATDIV_FIRSTNAMES / WATDIV_LASTNAMES env vars (fallback = originals),
#       so we can feed the vendored wordlist without touching /usr/share/dict.
# Seed 1u is the PIN: the query constants in bench/watdiv/queries/*.rq and the counts in
# expected-rows.tsv were chosen against the seed-1 SF=1 corpus. Changing the seed changes
# which entities carry which properties and WILL break the expected-rows diff.
#
# Output is N-Triples (`<s>\t<p>\t<o> .` per line, one triple per line, tab-separated terms),
# so load it into sparq with format `ntriples`. Caller cleans up the corpus (gitignored,
# regenerable — see bench/CATALOG.md disk discipline).
set -euo pipefail

SF="${1:-1}"
CACHE="${WATDIV_CACHE:-/tmp/watdiv}"
OUT="${2:-$CACHE/watdiv-sf$SF.nt}"

# Vendored inputs live next to this script (bench/watdiv/files/).
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VWORDS="$HERE/files/words.txt"

TARBALL_URL="https://dsg.uwaterloo.ca/watdiv/watdiv_v06.tar"
TARBALL_SHA256="fb8d930b74b3fbc8f948101bfaf658a90d2f74002f1fefda465c45ffd33a71d2"

BIN="$CACHE/watdiv"
SRC="$CACHE/src"
SEED="1u"           # PIN — see header. Do NOT change without regenerating expected-rows.tsv.
mkdir -p "$CACHE"

# ---- 1. fetch + verify + build (skipped if the built binary is already cached) -----------
if [ ! -x "$BIN" ]; then
  TAR="$CACHE/watdiv_v06.tar"
  if [ ! -f "$TAR" ]; then
    echo "[watdiv] downloading generator source ($TARBALL_URL)..." >&2
    curl -fsSL -o "$TAR" "$TARBALL_URL"
  fi
  echo "$TARBALL_SHA256  $TAR" | sha256sum -c - >&2

  # ---- 2. unpack + apply the three deterministic/hermeticity patches --------------------
  rm -rf "$CACHE/watdiv-src"
  tar xf "$TAR" -C "$CACHE" --one-top-level=watdiv-src --strip-components=1
  rm -rf "$SRC"; mkdir -p "$SRC"
  cp -r "$CACHE/watdiv-src/." "$SRC/"

  cd "$SRC"
  # (1)+(2): pin the two data-path RNG seeds (idempotent: only rewrites the time-seeded form).
  sed -i "s/boost::mt19937(static_cast<unsigned> (time(0)))/boost::mt19937(static_cast<unsigned>($SEED))/" src/model.cpp
  sed -i "s/^    srand (time(NULL));\$/    srand ($SEED);/" src/model.cpp
  # (3): env-overridable dictionary paths (idempotent: only if not already patched).
  if ! grep -q 'WATDIV_WORDS' src/model.cpp; then
    python3 - <<'PY'
p = "src/model.cpp"
s = open(p).read()
old = '        dict->init("/usr/share/dict/words", "../../files/firstnames.txt", "../../files/lastnames.txt");'
new = ('        const char * env_words = getenv("WATDIV_WORDS");\n'
       '        const char * env_first = getenv("WATDIV_FIRSTNAMES");\n'
       '        const char * env_last  = getenv("WATDIV_LASTNAMES");\n'
       '        dict->init(env_words ? env_words : "/usr/share/dict/words",\n'
       '                   env_first ? env_first : "../../files/firstnames.txt",\n'
       '                   env_last  ? env_last  : "../../files/lastnames.txt");')
if old not in s:
    raise SystemExit("[watdiv] FATAL: dictionary::init line not found — upstream source changed?")
open(p, "w").write(s.replace(old, new))
PY
  fi

  # ---- 3. build (no cmake; ~30s) -------------------------------------------------------
  echo "[watdiv] building watdiv (g++, Boost; BOOST_HOME=${BOOST_HOME:-/usr})..." >&2
  BOOST_HOME="${BOOST_HOME:-/usr}" make release >/dev/null
  cp "$SRC/bin/Release/watdiv" "$BIN"
  echo "[watdiv] cached generator: $BIN" >&2
fi

# ---- 4. generate the fixed corpus (deterministic; cached) -------------------------------
if [ ! -s "$OUT" ]; then
  echo "[watdiv] generating SF=$SF -> $OUT ..." >&2
  # The generator reads its name files from cwd-relative ../../files and writes triples to
  # stdout. We run from the cached src tree so ../../files resolves, point STRING literals at
  # the vendored wordlist, and redirect stdout into place. It also drops a `saved.txt` state
  # file in cwd (only needed for the -q/-s modes) — harmless; left in the cache dir.
  ( cd "$SRC/bin/Release" \
    && WATDIV_WORDS="$VWORDS" "$BIN" -d "$SRC/model/wsdbm-data-model.txt" "$SF" ) > "$OUT.tmp" 2>/dev/null
  mv "$OUT.tmp" "$OUT"
fi
echo "$OUT"
