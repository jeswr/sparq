#!/usr/bin/env bash
# [SONNET-4.6] sq-ymr2e.5 — determinism gate: forbids fixed-delay waits in specs + support.
#
# Run before (or as part of) the CI test step to enforce the no-sleep doctrine
# (research/web-gui-test-program.md §1.1 determinism rules): every wait must be a web-first
# assertion on a real UI state, never a fixed-delay sleep.
#
# Patterns caught (all must be CALLS — have a parenthesis — so doc-comments are not flagged):
#   waitForTimeout(   — Playwright's fixed-delay API (page.waitForTimeout / locator.waitForTimeout)
#   sleep(            — any user-defined or imported sleep helper
#   setTimeout(       — raw JS timer used as a sleep substitute
#
# Each grep runs with --include="*.ts" so only TypeScript spec/support files are scanned;
# shell scripts (*.sh) and config (*.json, *.ts outside these dirs) are not in scope.
#
# Usage:  bash support/no-timeout-gate.sh
#         (run from gui/e2e-playwright/)
set -euo pipefail

FAIL=0

# ── 1. waitForTimeout ─────────────────────────────────────────────────────────────────────────
# Catches `page.waitForTimeout(` / `waitForTimeout(` while ignoring comment-doc lines like
# "// NO waitForTimeout — use web-first assertions" (no parenthesis in those comments).
FOUND=$(grep -rn --include="*.ts" "waitForTimeout(" specs/ support/ 2>/dev/null || true)
if [ -n "$FOUND" ]; then
  echo "ERROR: waitForTimeout() call found — use web-first assertions only (design §1.1):" >&2
  echo "$FOUND" >&2
  FAIL=1
fi

# ── 2. sleep( ─────────────────────────────────────────────────────────────────────────────────
# Catches any call to a helper named sleep (a common anti-pattern wrapper around setTimeout/delay).
FOUND=$(grep -rn --include="*.ts" "sleep(" specs/ support/ 2>/dev/null || true)
if [ -n "$FOUND" ]; then
  echo "ERROR: sleep() call found — use web-first assertions only (design §1.1):" >&2
  echo "$FOUND" >&2
  FAIL=1
fi

# ── 3. setTimeout( ───────────────────────────────────────────────────────────────────────────
# Catches raw JS timer calls used as a sleep substitute (setTimeout(fn, N)).
# In Playwright test specs and support helpers there is no legitimate use of setTimeout;
# any occurrence means a fixed-delay wait that violates the determinism doctrine.
FOUND=$(grep -rn --include="*.ts" "setTimeout(" specs/ support/ 2>/dev/null || true)
if [ -n "$FOUND" ]; then
  echo "ERROR: setTimeout() call found — use web-first assertions only (design §1.1):" >&2
  echo "$FOUND" >&2
  FAIL=1
fi

if [ "$FAIL" -ne 0 ]; then
  exit 1
fi

echo "OK: no fixed-delay waits (waitForTimeout / sleep / setTimeout) found in specs/ or support/"
