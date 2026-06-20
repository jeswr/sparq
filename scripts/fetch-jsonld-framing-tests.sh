#!/usr/bin/env bash
# [OPUS-4.8] sq-oy1f.19 — fetches the official W3C JSON-LD 1.1 Framing test suite
# (github.com/w3c/json-ld-framing) at a PINNED commit into the gitignored
# tests/w3c/json-ld-framing directory. Framing is a SEPARATE W3C repo from the
# JSON-LD API suite (toRdf/fromRdf/compact live in w3c/json-ld-api, fetched by
# scripts/fetch-jsonld-tests.sh); the frame manifest + fixtures live here.
# Mirrors scripts/fetch-jsonld-tests.sh: test data is never committed — run this
# before the JSON-LD `frame` conformance runner. The runner SKIPS itself when the
# suite is absent, so a fresh checkout stays green offline.
set -euo pipefail

# Pinned w3c/json-ld-framing commit (main, 2026-06). Bump deliberately: the
# JSON-LD framing pass-count floor (FRAME_FLOOR) in
# crates/sparq-conformance/tests/jsonld_suite.rs is calibrated against THIS suite
# revision — pass counts are only comparable across runs when the suite is fixed.
PIN="3bf782ba9a40dd1b143435abe386d38df64f2b47"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/tests/w3c/json-ld-framing"

if [ -d "$DEST/.git" ]; then
    HAVE="$(git -C "$DEST" rev-parse HEAD)"
    if [ "$HAVE" = "$PIN" ]; then
        echo "json-ld-framing already at pinned commit $PIN — nothing to do."
        exit 0
    fi
    echo "json-ld-framing present at $HAVE, re-pinning to $PIN…"
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
    exit 0
fi

mkdir -p "$(dirname "$DEST")"
echo "Cloning w3c/json-ld-framing (shallow) into tests/w3c/json-ld-framing…"
git clone --depth 1 https://github.com/w3c/json-ld-framing "$DEST"
if [ "$(git -C "$DEST" rev-parse HEAD)" != "$PIN" ]; then
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
fi
echo "json-ld-framing pinned at $PIN."
