#!/usr/bin/env bash
# [OPUS-4.8] sq-9qz6 — wasm dependency-graph guard.
#
# The browser/wasm build must stay lean and pure-Rust: the native-only heavy deps
# (parallelism + the compression codecs + the parallel parser crate) must NEVER
# enter the `sparq-wasm` dependency graph for the wasm32 target. This was only an
# invariant stated in comments; this script makes it an enforced CI gate so an
# accidental `use` or a default-feature change that pulls one of them in fails fast
# (bundle bloat / broken browser build) instead of silently regressing.
#
# Run: scripts/wasm-deps-guard.sh   (exit 0 = clean, exit 1 = a forbidden crate is present)
set -euo pipefail

TARGET="wasm32-unknown-unknown"
# Crates that must be absent from the wasm32 graph (non-dev deps).
FORBIDDEN=(rayon flate2 zstd zstd-safe bzip2 sparq-parse mio tokio)

tree="$(cargo tree -p sparq-wasm --target "$TARGET" -e no-dev 2>/dev/null)"

fail=0
for crate in "${FORBIDDEN[@]}"; do
  # Match a tree line whose package name is exactly $crate (followed by a space+version
  # or end-of-line), ignoring the leading tree-drawing glyphs.
  if printf '%s\n' "$tree" | grep -qE "[^[:alnum:]_-]${crate}( v[0-9]| |\$)"; then
    echo "::error::forbidden crate '${crate}' is in the sparq-wasm ${TARGET} dependency graph"
    printf '%s\n' "$tree" | grep -E "[^[:alnum:]_-]${crate}( v[0-9]| |\$)" | head -3
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  echo "wasm-deps-guard: FAILED — a native-only dep leaked into the wasm graph (see above)."
  exit 1
fi
echo "wasm-deps-guard: OK — none of [${FORBIDDEN[*]}] are in the sparq-wasm ${TARGET} graph."
