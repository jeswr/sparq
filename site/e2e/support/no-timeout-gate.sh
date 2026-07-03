#!/usr/bin/env bash
# [OPUS-4.8] sq-ymr2e.1 — the determinism grep gate (research/web-gui-test-program.md §1.1).
#
# FAILS if any `page.waitForTimeout(...)` (or bare `waitForTimeout(...)`) call appears in the
# site's e2e TypeScript. Time-based sleeps are the #1 source of E2E flake; the foundation replaces
# them with web-first waiters (expectRunnerState / waitForAppReady). Prose mentions of the word in
# comments are fine — the gate matches only the CALL form `waitForTimeout(` in `*.ts` files.
#
# Run from site/:  npm run test:e2e:grep-gate
set -euo pipefail

# Resolve site/e2e relative to this script, so CWD does not matter.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
E2E_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# grep -r over *.ts only (excludes this .sh and the README.md). -F fixed string; the trailing "("
# means only the call form matches, never a prose mention like "no waitForTimeout".
if hits="$(grep -rInF --include='*.ts' 'waitForTimeout(' "${E2E_DIR}")"; then
  echo "✗ determinism gate: found time-based waitForTimeout call(s) under ${E2E_DIR} —" >&2
  echo "  replace with a web-first waiter (expectRunnerState / waitForAppReady). See" >&2
  echo "  e2e/support/README.md §doctrine rule 1." >&2
  echo "${hits}" >&2
  exit 1
fi

echo "✓ determinism gate: zero waitForTimeout calls under $(basename "${E2E_DIR}")/"
