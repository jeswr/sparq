#!/usr/bin/env bash
# [FABLE-5] sq-i858h — browser-SHACL same-runtime comparison entry point:
# sparq-shacl-wasm vs Zazuko rdf-validate-shacl in ONE Node process, over the
# SAME (data x shapes) workloads as bench/shacl (the committed shapes/*.ttl +
# shapes-sparql/sparql_heavy.ttl) on the small VENDORED micro-ABox
# (data/abox.ttl — no LUBM generator / Java needed).
#
#   bash run.sh --smoke     # 1 iteration; the ACCEPTANCE entry point (exit 0
#                           # iff the per-workload #violations+conforms
#                           # agreement gate is green for both engines)
#   bash run.sh             # best-of-$ITERS (default 3) + envelope
#   bash run.sh --bundle-only  # ONLY the bundle-byte comparison (no Node
#                           # runtime timing, no agreement gate)
#
# Stages:
#   1. wasm-pack build crates/sparq-shacl-wasm --target nodejs --release
#      (default features — no shacl-af; artifact bytes are the DETERMINISTIC
#      second column). Skipped when pkg/ already exists unless REBUILD=1.
#   2. npm-install the GATHER-ONLY peer deps (never committed; AGENTS.md —
#      engines stay out of git) into $MODULES_DIR (default /tmp scratch).
#      UNPINNED: bare package names, no committed lockfile — the resolved
#      versions are recorded as provenance, so stage 2b's peer bytes are NOT
#      reproducible by a later gather.
#   2b. [SONNET-4.6 sq-c6c2s] node bundle.mjs — the minified browser-bundle byte
#      comparison: esbuild-bundled (minified, tree-shaken) rdf-validate-shacl vs
#      the wasm artifact, gzip-9 wire bytes. Skipped WITH NOTICE when
#      esbuild/peer deps are absent (column ABSENT, never a 0).
#   3. node harness.mjs — the shared report->count reduction, the agreement
#      gate, timing rows ONLY after the gate, one canonical-competitor-results
#      envelope (canonical:false on the work box). Stage 2b's result is folded
#      into that envelope's `bundle_bytes.peer_minified_bundle`.
#
# TUNABLES (env):
#   ITERS         best-of-N               (default 3; --smoke forces 1)
#   MODULES_DIR   peer npm scratch        (default /tmp/shacl-wasm-deps)
#   OUT_DIR       envelope dir            (default ./results — git-ignored)
#   REBUILD       1 = force wasm rebuild  (default: reuse existing pkg/)
#   CANONICAL     1 = quiet-box run       (default 0: canonical:false)
#   FEATURES      extra cargo features for the wasm build (default: none —
#                 the canonical default-features artifact). FEATURES=stateful
#                 exports the sq-01xlp `ParsedGraph` handle => the harness adds
#                 the sparq validate-only column (sparq_validate_only_us); the
#                 envelope then flags bundle bytes as NON-canonical (feature
#                 build). pkg/ is auto-rebuilt when FEATURES changes.
#   ALLOW_MISSING_PEER  1 = degrade to sparq-only self-assert WITH NOTICE
#                 (peer column ABSENT — never fabricated). Default: the peer
#                 is REQUIRED (the whole point is the comparison).
#   NO_BUNDLE     1 = skip stage 2b entirely (byte column ABSENT).
#   TRIM_SWEEP    1 = stage 2b ALSO builds the `release-wasm` cargo-profile
#                 variant into pkg-release-wasm/ and measures it alongside the
#                 shipped `--release` artifact. That profile (workspace
#                 Cargo.toml) strips symbols and builds the cold oxttl/oxiri/
#                 spargebra parse crates at opt-level "z"; every OTHER sparq
#                 wasm bundle already ships under it, so it is the first
#                 size-trim lever to price. Measurement only — nothing is
#                 rebuilt or re-pinned on the strength of it.
#   WASM_OPT_PROBE 1 = stage 2b re-runs binaryen at max size settings over the
#                 shipped artifact to price the remaining binaryen-side
#                 headroom beyond the crate's packaged `wasm-opt = ["-Oz"]`.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

ITERS="${ITERS:-3}"
MODULES_DIR="${MODULES_DIR:-/tmp/shacl-wasm-deps}"
OUT_DIR="${OUT_DIR:-$HERE/results}"
CANONICAL="${CANONICAL:-0}"

BUNDLE_ONLY=0

for a in "$@"; do
  case "$a" in
    --smoke) ITERS=1 ;;
    --bundle-only) BUNDLE_ONLY=1 ;;
    -h|--help) sed -n '2,59p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "error: unknown argument '$a' (see --help)" >&2; exit 2 ;;
  esac
done

log() { printf '[shacl-wasm] %s\n' "$*" >&2; }

# ---- 1. the wasm artifact (deterministic-bytes column source) ---------------
# A features marker forces a rebuild whenever FEATURES changes, so a stale pkg/
# can never silently drop (or smuggle in) the stateful column.
FEATURES="${FEATURES:-}"
PKG="$HERE/pkg"
MARKER="$PKG/.features"
if [ ! -f "$PKG/sparq_shacl_wasm.js" ] || [ "${REBUILD:-0}" = "1" ] \
   || [ "$(cat "$MARKER" 2>/dev/null || true)" != "$FEATURES" ]; then
  log "wasm-pack build crates/sparq-shacl-wasm (nodejs, release, features: ${FEATURES:-default})"
  wasm-pack build "$ROOT/crates/sparq-shacl-wasm" --target nodejs --release \
    --out-dir "$PKG" ${FEATURES:+--features "$FEATURES"} >&2
  printf '%s' "$FEATURES" > "$MARKER"
else
  log "reusing existing $PKG (REBUILD=1 to force)"
fi
WASM_PACK_VERSION="$(wasm-pack --version 2>/dev/null || echo unknown)"

# ---- 2. gather-only peer deps (rdf-validate-shacl + its RDF/JS stack) -------
# The exact package set from bench/competitors.json's install_recipe, PLUS
# esbuild — the bundler that turns that stack into the minified, tree-shaken
# browser bundle stage 2b weighs. All gather-only; none of it is committed.
# UNPINNED ON PURPOSE (for now): bare names, no committed manifest/lockfile, so
# top-level AND transitive versions float. The resolved versions are recorded in
# the envelope as PROVENANCE — they do NOT make stage 2b's peer bytes
# reproducible, which is why that column is labelled non-canonical. Pinning it
# properly means a committed package.json + package-lock.json installed with
# `npm ci` (the posture bench/wasm-compare/bundle.mjs already has for oxigraph).
PEER_PKGS=(rdf-validate-shacl @rdfjs/data-model @rdfjs/dataset @rdfjs/term-map
           @rdfjs/namespace @rdfjs/parser-n3 @rdfjs/environment clownface esbuild)
if [ ! -d "$MODULES_DIR/node_modules/rdf-validate-shacl" ] \
   || { [ "${NO_BUNDLE:-0}" != "1" ] && [ ! -d "$MODULES_DIR/node_modules/esbuild" ]; }; then
  if [ "${ALLOW_MISSING_PEER:-0}" = "1" ]; then
    log "NOTICE: peer deps missing and ALLOW_MISSING_PEER=1 — sparq-only self-assert"
  else
    log "installing gather-only peer deps into $MODULES_DIR (scratch, never committed)"
    mkdir -p "$MODULES_DIR"
    ( cd "$MODULES_DIR" && \
      { [ -f package.json ] || echo '{"private":true}' > package.json; } && \
      npm install --no-save --no-audit --no-fund "${PEER_PKGS[@]}" >&2 )
  fi
fi

mkdir -p "$OUT_DIR"
GIT_COMMIT="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$OUT_DIR/shacl-wasm-vendored-${TS}.json"

# ---- 2b. [SONNET-4.6 sq-c6c2s] minified-bundle byte comparison ---------------
# The peer's UNPACKED npm footprint is not a wire size, so the first-read record
# (research/gap-shacl-wasm-2026-07.md) made no byte-ratio claim. This stage
# builds the comparable number: what a bundler actually emits for a browser app.
# Bytes, no wall clock => no quiet box needed; but the peer install above is
# unpinned, so these bytes are point-in-time, not reproducible/canonical. Its
# result is folded into the stage-3 envelope; a failure here NEVER fails the run
# — the column goes ABSENT rather than fabricated, like the peer timing column.
BUNDLE_OUT=""
if [ "${NO_BUNDLE:-0}" = "1" ]; then
  log "NOTICE: NO_BUNDLE=1 — minified-bundle byte column ABSENT"
else
  BUNDLE_PKGS=(--pkg "shipped=$PKG")
  if [ "${TRIM_SWEEP:-0}" = "1" ]; then
    # Size-trim lever #1: the workspace `release-wasm` cargo profile (strip =
    # symbols; oxttl/oxiri/spargebra at opt-level "z"). Every OTHER sparq wasm
    # bundle already ships under it — this suite's artifact is built with plain
    # `--release`, so the profile is free headroom IF the bytes say so.
    TRIM_PKG="$HERE/pkg-release-wasm"
    if [ ! -f "$TRIM_PKG/sparq_shacl_wasm.js" ] || [ "${REBUILD:-0}" = "1" ] \
       || [ "$(cat "$TRIM_PKG/.features" 2>/dev/null || true)" != "$FEATURES" ]; then
      log "TRIM_SWEEP: wasm-pack build --profile release-wasm -> $TRIM_PKG"
      wasm-pack build "$ROOT/crates/sparq-shacl-wasm" --target nodejs \
        --profile release-wasm --out-dir "$TRIM_PKG" \
        ${FEATURES:+--features "$FEATURES"} >&2
      printf '%s' "$FEATURES" > "$TRIM_PKG/.features"
    fi
    BUNDLE_PKGS+=(--pkg "release-wasm-profile=$TRIM_PKG")
  fi
  BUNDLE_OUT="$OUT_DIR/shacl-wasm-bundle-${TS}.json"
  BUNDLE_EXTRA=()
  [ "${WASM_OPT_PROBE:-0}" = "1" ] && BUNDLE_EXTRA+=(--wasm-opt-probe)
  if ! WASM_PACK_VERSION="$WASM_PACK_VERSION" node "$HERE/bundle.mjs" \
      "${BUNDLE_PKGS[@]}" \
      --modules-dir "$MODULES_DIR" \
      --git-commit "$GIT_COMMIT" \
      --out "$BUNDLE_OUT" \
      ${BUNDLE_EXTRA[@]:+"${BUNDLE_EXTRA[@]}"}; then
    log "NOTICE: bundle.mjs failed — minified-bundle byte column ABSENT (never fabricated)"
    BUNDLE_OUT=""
  fi
fi

if [ "$BUNDLE_ONLY" = "1" ]; then
  [ -n "$BUNDLE_OUT" ] && log "bundle envelope: $BUNDLE_OUT"
  log "--bundle-only: skipping the Node-runtime harness (no gate, no timings)"
  exit 0
fi

# ---- 3. the harness (gate -> timing -> envelope) -----------------------------
EXTRA=()
[ "${ALLOW_MISSING_PEER:-0}" = "1" ] && EXTRA+=(--allow-missing-peer)
[ -n "$BUNDLE_OUT" ] && EXTRA+=(--peer-bundle "$BUNDLE_OUT")

WASM_PACK_VERSION="$WASM_PACK_VERSION" node "$HERE/harness.mjs" \
  --pkg "$PKG" \
  --modules-dir "$MODULES_DIR" \
  --data "$HERE/data/abox.ttl" \
  --shapes-dir "$ROOT/bench/shacl/shapes" \
  --shapes-dir "$ROOT/bench/shacl/shapes-sparql" \
  --expected "$HERE/expected.tsv" \
  --iters "$ITERS" \
  --canonical "$CANONICAL" \
  --git-commit "$GIT_COMMIT" \
  --sparq-features "$FEATURES" \
  --out "$OUT" \
  ${EXTRA[@]:+"${EXTRA[@]}"}

log "envelope: $OUT"
log "scratch peer deps live in $MODULES_DIR (delete when finished)"
