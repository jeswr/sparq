#!/usr/bin/env bash
# [OPUS-4.8] sq-tmsd6: Fetches the SolidLab ODRL-Test-Suite at a PINNED commit into
# the gitignored tests/odrl-test-suite/ directory. Test data is FETCHED, never
# committed to this repo (mirrors scripts/fetch-conformance.sh / the W3C suites) —
# run this before `cargo test -p sparq-policy --test odrl_test_suite`.
#
# Suite: github.com/SolidLabResearch/ODRL-Test-Suite (MIT, LICENSE.md). Its
# self-describing Turtle cases under data/ are the differential oracle the
# crate-local runner evaluates through sparq-policy's real evaluate() path.
set -euo pipefail

# Pinned SolidLabResearch/ODRL-Test-Suite commit (main, 2025-10-15). Bump
# deliberately: pass-rates are only comparable across runs when the suite
# revision is fixed (same discipline as the W3C suite pin).
PIN="7958238e72511059478e43ec9e57b053504cfd2c"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/tests/odrl-test-suite"

if [ -d "$DEST/.git" ]; then
    HAVE="$(git -C "$DEST" rev-parse HEAD)"
    if [ "$HAVE" = "$PIN" ]; then
        echo "ODRL-Test-Suite already at pinned commit $PIN — nothing to do."
        exit 0
    fi
    echo "ODRL-Test-Suite present at $HAVE, re-pinning to $PIN…"
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
    exit 0
fi

mkdir -p "$(dirname "$DEST")"
echo "Cloning SolidLabResearch/ODRL-Test-Suite (shallow) into tests/odrl-test-suite…"
git clone --depth 1 https://github.com/SolidLabResearch/ODRL-Test-Suite "$DEST"
if [ "$(git -C "$DEST" rev-parse HEAD)" != "$PIN" ]; then
    git -C "$DEST" fetch --depth 1 origin "$PIN"
    git -C "$DEST" checkout --detach "$PIN"
fi
echo "ODRL-Test-Suite pinned at $PIN."
