#!/usr/bin/env bash
# [SONNET-4.6] sq-ymr2e.5 — determinism gate: forbids waitForTimeout / sleep in specs + support.
#
# Run before (or as part of) the CI test step to enforce the no-sleep doctrine
# (research/web-gui-test-program.md §1.1 determinism rules): every wait must be a web-first
# assertion on a real UI state, never a fixed-delay sleep.
#
# Usage:  bash support/no-timeout-gate.sh
#         (run from gui/e2e-playwright/)
set -euo pipefail

# Match only CALLS (with a parenthesis), not comments that document the rule.
# This catches `page.waitForTimeout(` / `waitForTimeout(` while ignoring
# comment lines like "// NO waitForTimeout — use web-first assertions".
FOUND=$(grep -rn --include="*.ts" "waitForTimeout(" specs/ support/ 2>/dev/null || true)

if [ -n "$FOUND" ]; then
  echo "ERROR: waitForTimeout() call found — use web-first assertions only (design §1.1):" >&2
  echo "$FOUND" >&2
  exit 1
fi

echo "OK: no waitForTimeout found in specs/ or support/"
