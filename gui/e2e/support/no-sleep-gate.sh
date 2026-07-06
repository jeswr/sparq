#!/usr/bin/env bash
# [SONNET-4.6] sq-ymr2e.6 — determinism gate for the tauri-driver smoke harness.
#
# Forbids arbitrary time-based waits (await sleep / await setTimeout / waitForTimeout)
# in the harness source files (*.mjs, *.js, *.ts). Polling MUST use event-driven APIs
# (webdriverio waitForExist / waitUntil with an explicit condition), never a fixed delay.
#
# Mirrors the no-timeout-gate.sh used by the Playwright mock-IPC lane (gui/e2e-playwright/).
# Scope: all .mjs/.js/.ts files under gui/e2e/ (excluding node_modules/).
#
# Exit 0 = clean; exit 1 = violation found.

set -euo pipefail

HARNESS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
echo "[no-sleep-gate] scanning $HARNESS_DIR for arbitrary time-based waits…"

FOUND=0

while IFS= read -r -d $'\0' f; do
  # Forbidden patterns:
  #   await sleep(        — node:timers/promises sleep alias
  #   await setTimeout(   — direct awaited timer (not a callback-style retry)
  #   waitForTimeout(     — WebdriverIO / Playwright explicit timeout wait
  if grep -En 'await sleep\(|await setTimeout\(|waitForTimeout\(' "$f" >/dev/null 2>&1; then
    echo "FAIL: arbitrary time-based wait found in $f:"
    grep -En 'await sleep\(|await setTimeout\(|waitForTimeout\(' "$f"
    FOUND=1
  fi
done < <(find "$HARNESS_DIR" \( -name "*.mjs" -o -name "*.js" -o -name "*.ts" \) \
  ! -path "*/node_modules/*" -print0)

if [ "$FOUND" -eq 1 ]; then
  echo ""
  echo "[no-sleep-gate] FAILED — remove arbitrary sleeps; use waitForExist / waitUntil instead"
  exit 1
fi

echo "[no-sleep-gate] clean — no arbitrary time-based waits found"
