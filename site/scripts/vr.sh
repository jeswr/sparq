#!/usr/bin/env bash
# [FABLE-5] sq-ymr2e.10 — run the VISUAL-REGRESSION suite inside the digest-pinned official
# Playwright container (research/web-gui-test-program.md §4).
#
# WHY a container: screenshot baselines are only stable when the font rasterizer, antialiasing,
# and browser build are pinned — so the visual projects run ONLY here, and the committed
# baselines are Linux-only by policy. The digest below is the image the baselines were minted
# in; bumping @playwright/test means re-pinning this digest AND regenerating the baselines in
# the same PR (`npm run vr:update`).
#
# The IMAGE TAG must match the locked @playwright/test version (the image ships the matching
# browser builds); the DIGEST pins the exact rasterizer/OS layer. Verified at runtime below.
#
# PREREQS (host): `npm ci` at the repo root (linux-x64 node_modules are reused in-container) and
# the lean wasm bundle synced into site/public/wasm (js: `npm run build:wasm:lean`, then site:
# `npm run sync-wasm`) — the /try workbench capture waits on the real engine-ready state.
#
# Usage:  npm run vr            # compare against the committed baselines
#         npm run vr:update     # regenerate baselines (forces a rewrite of every snapshot)
#         bash scripts/vr.sh [extra playwright args]
set -euo pipefail

PLAYWRIGHT_VERSION="1.61.0"
IMAGE="mcr.microsoft.com/playwright:v${PLAYWRIGHT_VERSION}-noble@sha256:57b65fdc9ceabe0ef613124c7bbe2babcf9362c4d85e382fe3b03604e84b428a"

SITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${SITE_DIR}/.." && pwd)"

# Fail fast (and loudly) if the locked @playwright/test drifted from the pinned image: a version
# skew silently renders with a different browser build and every baseline diff becomes noise.
LOCKED="$(node -p "require('${REPO_ROOT}/node_modules/@playwright/test/package.json').version")"
if [ "${LOCKED}" != "${PLAYWRIGHT_VERSION}" ]; then
  echo "vr.sh: locked @playwright/test ${LOCKED} != pinned container ${PLAYWRIGHT_VERSION}." >&2
  echo "vr.sh: bump PLAYWRIGHT_VERSION + the image digest in scripts/vr.sh, then regenerate" >&2
  echo "vr.sh: the baselines (npm run vr:update) in the SAME PR." >&2
  exit 1
fi

if [ ! -f "${SITE_DIR}/public/wasm/sparq_wasm_bg.wasm" ]; then
  echo "vr.sh: site/public/wasm/sparq_wasm_bg.wasm missing — the /try capture needs the engine." >&2
  echo "vr.sh: build it:  (cd ../js && npm run build:wasm:lean) && npm run sync-wasm" >&2
  exit 1
fi

# --ipc=host: chromium's recommended shm setup under docker.
# -u host uid/gid: baselines/test-results/.next written back to the mount stay host-owned.
# HOME=/tmp: a writable HOME for the non-root uid (npm/next caches).
# CI is forwarded so the config's CI-only behaviour (retries/forbidOnly/reporter) matches CI.
exec docker run --rm --init --ipc=host \
  -u "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e SPARQ_VR=1 \
  -e SPARQ_NIGHTLY_VR="${SPARQ_NIGHTLY_VR:-}" \
  -e CI="${CI:-}" \
  -v "${REPO_ROOT}:/work" \
  -w /work/site \
  "${IMAGE}" \
  npx playwright test --project=visual-desktop --project=visual-mobile "$@"
