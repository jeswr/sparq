#!/usr/bin/env bash
# [OPUS-4.8] LUBM (Lehigh University Benchmark) corpus generator — build-once-and-cache the
# real UBA (Univ-Bench Artificial) Java generator, then emit a FIXED, deterministic
# LUBM(1) corpus (~100k triples) PLUS the Univ-Bench OWL ontology, both as N-Triples.
#
#   bench/lubm/gen.sh [univ=1] [seed=0]
#
# Emits two paths on stdout (one per line):
#   1. <data.nt>           the LUBM(univ) ABox      (raw, NO ontology, NO entailment)
#   2. <ontology.nt>       the Univ-Bench OWL TBox   (subClassOf/subPropertyOf/inverseOf/
#                                                      TransitiveProperty axioms)
# run.sh consumes BOTH: extensional queries run on <data.nt>; entailed queries run on the
# OWL-RL closure of (<data.nt> + <ontology.nt>) — see run.sh and README.md.
#
# WHY THE REAL GENERATOR (not a fallback): the UBA generator is the canonical, semantics-
# pinned source of LUBM data, so the query answer sizes match the published benchmark. It is
# tiny pure-Java (5 source files, JDK-only — no external deps), DETERMINISTIC at a fixed
# (-univ, -seed): the RDF/XML files are byte-identical across runs (sha256-verified during
# development), so the corpus is reproducible per-commit.
#
# WHY javac/jar (NOT Maven): upstream ships a Maven pom, but the sources import only
# java.io.* / java.util.* — no third-party jars. So we compile with the JDK's own
# `javac` + `jar` (already required toolchain) and SKIP the heavyweight Maven download.
# This is the lighter, hermetic path: no network beyond the one pinned git clone.
#
# HERMETICITY / CI NOTES:
#  - Network: ONE shallow `git clone` of the pinned UBA commit (a few hundred KB). The built
#    jar + the generated corpus are cached under $LUBM_CACHE, so steady-state per-commit runs
#    do NO network and NO rebuild.
#  - Toolchain: JDK (javac/jar/java) + raptor2-utils (`rapper`) for RDF/XML -> N-Triples.
#    rapper is a small, fast, reliable C converter (apt: raptor2-utils). gen.sh errors with a
#    clear install hint if it is absent.
#  - Ontology: upstream's committed univ-bench.owl is wrapped in an Apple Safari Webarchive
#    (a bplist container) — rapper cannot parse it directly. We extract the embedded RDF/XML
#    payload (the bytes from the first `<?xml` to the last `</rdf:RDF>`) with a 3-line awk/sed;
#    the extracted TBox is triple-for-triple identical to the canonical copy served at
#    http://swat.cse.lehigh.edu/onto/univ-bench.owl (verified during development). Extracting
#    from the PINNED commit keeps us independent of the live SWAT server.
#
# Output is N-Triples; load into sparq with format `ntriples`. Caller cleans up the corpus
# (gitignored, regenerable — see bench/CATALOG.md disk discipline).
set -euo pipefail

UNIV="${1:-1}"
SEED="${2:-0}"
CACHE="${LUBM_CACHE:-/tmp/lubm}"

# Pinned UBA source: github.com/nandana/LUBM-UBA (a maintained, build-from-source mirror of
# Lehigh's UBA 1.7). Apache-2.0 (per its pom.xml <licenses>). Pin the exact commit so the
# generator — and thus the corpus — is reproducible.
UBA_REPO="https://github.com/nandana/LUBM-UBA.git"
UBA_COMMIT="05c3f3a30b8d1872faad8a40e29b8e81daa15894"
ONTO_URL="http://swat.cse.lehigh.edu/onto/univ-bench.owl"

SRC="$CACHE/src"
JAR="$CACHE/uba.jar"
ONTO_NT="$CACHE/univ-bench.nt"
DATA_NT="$CACHE/lubm-univ${UNIV}-seed${SEED}.nt"
mkdir -p "$CACHE"

# ---- 0. tool checks ----------------------------------------------------------------------
if ! command -v rapper >/dev/null 2>&1; then
  echo "[lubm] ERROR: 'rapper' not found. Install raptor2-utils (sudo apt-get install -y raptor2-utils)." >&2
  exit 1
fi
if ! command -v javac >/dev/null 2>&1; then
  echo "[lubm] ERROR: 'javac' not found. Install a JDK (e.g. default-jdk)." >&2
  exit 1
fi
if ! command -v perl >/dev/null 2>&1; then
  echo "[lubm] ERROR: 'perl' not found (used to unwrap the ontology from its webarchive)." >&2
  exit 1
fi

# ---- 1. fetch + build the jar (skipped if cached) ----------------------------------------
if [ ! -f "$JAR" ]; then
  if [ ! -d "$SRC/.git" ]; then
    echo "[lubm] cloning UBA @ ${UBA_COMMIT:0:12} ..." >&2
    rm -rf "$SRC"
    git clone -q "$UBA_REPO" "$SRC"
    ( cd "$SRC" && git checkout -q "$UBA_COMMIT" )
  fi
  echo "[lubm] compiling UBA generator (javac, JDK-only)..." >&2
  rm -rf "$CACHE/classes"; mkdir -p "$CACHE/classes"
  javac -nowarn -d "$CACHE/classes" "$SRC"/src/main/java/edu/lehigh/swat/bench/uba/*.java 2>/dev/null
  printf 'Main-Class: edu.lehigh.swat.bench.uba.Generator\n' > "$CACHE/manifest.txt"
  jar cfm "$JAR" "$CACHE/manifest.txt" -C "$CACHE/classes" .
  echo "[lubm] cached jar: $JAR" >&2
fi

# ---- 2. extract + convert the ontology (TBox) once ---------------------------------------
if [ ! -s "$ONTO_NT" ]; then
  if [ ! -d "$SRC/.git" ]; then
    rm -rf "$SRC"; git clone -q "$UBA_REPO" "$SRC"; ( cd "$SRC" && git checkout -q "$UBA_COMMIT" )
  fi
  echo "[lubm] extracting + converting ontology -> N-Triples ..." >&2
  # Strip the Safari Webarchive bplist wrapper: keep the BYTES from the first `<?xml` to the
  # end of `</rdf:RDF>` (the binary bplist header/trailer share those lines, so a line-based
  # sed/grep slice leaves binary garbage that breaks the XML parser — a byte slice is exact).
  perl -0777 -ne 'print $1 if /(<\?xml.*<\/rdf:RDF>)/s' "$SRC/univ-bench.owl" > "$CACHE/univ-bench.owl"
  rapper -q -i rdfxml -o ntriples "$CACHE/univ-bench.owl" "$ONTO_URL" > "$ONTO_NT.tmp"
  mv "$ONTO_NT.tmp" "$ONTO_NT"
  echo "[lubm] ontology: $(wc -l < "$ONTO_NT") triples -> $ONTO_NT" >&2
fi

# ---- 3. generate the fixed ABox (deterministic; cached) ----------------------------------
if [ ! -s "$DATA_NT" ]; then
  echo "[lubm] generating LUBM($UNIV) seed=$SEED ..." >&2
  GEN="$CACHE/gen-univ${UNIV}-seed${SEED}"
  rm -rf "$GEN"; mkdir -p "$GEN"
  # UBA writes University<N>_<i>.owl (RDF/XML) to its cwd; run from a scratch dir.
  ( cd "$GEN" && java -jar "$JAR" -univ "$UNIV" -seed "$SEED" -onto "$ONTO_URL" >/dev/null )
  # Convert every department file to N-Triples and concatenate. The base URI only affects the
  # ABox header's owl:imports (the only relative ref); all individuals use absolute rdf:about.
  : > "$DATA_NT.tmp"
  for f in "$GEN"/University*_*.owl; do
    rapper -q -i rdfxml -o ntriples "$f" "http://www.lehigh.edu/" >> "$DATA_NT.tmp"
  done
  mv "$DATA_NT.tmp" "$DATA_NT"
  rm -rf "$GEN"   # scratch RDF/XML no longer needed; the NT is the artifact
  echo "[lubm] data: $(wc -l < "$DATA_NT") triples (pre-dedup) -> $DATA_NT" >&2
fi

# Emit the two artifact paths (data first, ontology second).
echo "$DATA_NT"
echo "$ONTO_NT"
