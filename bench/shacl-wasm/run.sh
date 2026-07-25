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
#
# Stages:
#   1. wasm-pack build crates/sparq-shacl-wasm --target nodejs --release
#      (default features — no shacl-af; artifact bytes are the DETERMINISTIC
#      second column). Skipped when pkg/ already exists unless REBUILD=1.
#   2. npm-install the GATHER-ONLY peer deps (never committed; AGENTS.md —
#      engines stay out of git) into $MODULES_DIR (default /tmp scratch).
#   3. node harness.mjs — the shared report->count reduction, the agreement
#      gate, timing rows ONLY after the gate, one canonical-competitor-results
#      envelope (canonical:false on the work box).
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
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

ITERS="${ITERS:-3}"
MODULES_DIR="${MODULES_DIR:-/tmp/shacl-wasm-deps}"
OUT_DIR="${OUT_DIR:-$HERE/results}"
CANONICAL="${CANONICAL:-0}"

for a in "$@"; do
  case "$a" in
    --smoke) ITERS=1 ;;
    -h|--help) sed -n '2,32p' "${BASH_SOURCE[0]}"; exit 0 ;;
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
# The exact package set from bench/competitors.json's install_recipe.
PEER_PKGS=(rdf-validate-shacl @rdfjs/data-model @rdfjs/dataset @rdfjs/term-map
           @rdfjs/namespace @rdfjs/parser-n3 @rdfjs/environment clownface)
if [ ! -d "$MODULES_DIR/node_modules/rdf-validate-shacl" ]; then
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

# ---- 3. the harness (gate -> timing -> envelope) -----------------------------
mkdir -p "$OUT_DIR"
GIT_COMMIT="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$OUT_DIR/shacl-wasm-vendored-${TS}.json"

EXTRA=()
[ "${ALLOW_MISSING_PEER:-0}" = "1" ] && EXTRA+=(--allow-missing-peer)

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
