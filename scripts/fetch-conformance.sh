#!/usr/bin/env bash
# Fetches the official W3C SPARQL test suites (github.com/w3c/rdf-tests) at a
# PINNED commit into the gitignored tests/w3c/ directory. Test data is never
# committed to this repo — run this script before `cargo run -p sparq-conformance`.
set -euo pipefail

# Pinned w3c/rdf-tests commit (main, 2026-06). Bump deliberately: pass-rates are
# only comparable across runs when the suite revision is fixed.
PIN="f25dbc092c654d792974848e81bb519d7328f0e8"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/tests/w3c/rdf-tests"

if [ -d "$DEST/.git" ]; then
    HAVE="$(git -C "$DEST" rev-parse HEAD)"
    if [ "$HAVE" = "$PIN" ]; then
        echo "rdf-tests already at pinned commit $PIN — nothing to do."
        exit 0
    fi
    echo "rdf-tests present at $HAVE, re-pinning to $PIN…"
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
    exit 0
fi

mkdir -p "$(dirname "$DEST")"
echo "Cloning w3c/rdf-tests (shallow) into tests/w3c/rdf-tests…"
git clone --depth 1 https://github.com/w3c/rdf-tests "$DEST"
if [ "$(git -C "$DEST" rev-parse HEAD)" != "$PIN" ]; then
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
fi
echo "rdf-tests pinned at $PIN."
