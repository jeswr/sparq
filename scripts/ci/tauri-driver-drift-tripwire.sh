#!/usr/bin/env bash
# [FABLE-5] sq-t6492 — tauri-driver ↔ webkit2gtk DRIFT TRIPWIRE
#
# The native GUI smoke (nightly-full-sweep.yml `tauri-smoke`, gui.yml `tauri-e2e`) launches the
# Tauri shell through `tauri-driver`, which proxies to the runner's WebKitWebDriver (from the
# `webkit2gtk-driver` apt package). Both the WebDriver server AND the app's webview come from the
# SAME runner-image apt version of webkit2gtk, so they are internally consistent — the drift that
# breaks the lane is between the PINNED `tauri-driver` (crates.io) and the runner image's
# webkit2gtk, which GitHub bumps on ITS schedule, not ours.
#
# KNOWN-INCOMPATIBLE combination (issue #1740, bead sq-t6492):
#   tauri-driver 2.0.6 (newest published, May 2026)  +  webkit2gtk 2.52.x  (ubuntu-24.04 noble)
#   → POST /session fails with "session not created: Failed to match capabilities" DURING
#     capability negotiation, BEFORE WebKitWebDriver logs a session → the driver log is 0 bytes.
#   The webdriverio client already sends `browserName: "wry"` (the historic tauri#8828 fix), so
#   the surviving cause is a WebKitWebDriver 2.5x protocol/behaviour change (corroborated by
#   `WebKitWebDriver --version` printing its usage banner instead of a version on 2.52.x).
#
# PURPOSE: turn the NEXT drift from a 0-byte-log hunt into a one-line diagnosis. This script logs
# both versions and, when it sees the known-incompatible pair, emits a LOUD GitHub Actions
# annotation naming this bead + issue. It is DIAGNOSTIC, not a gate: it exits 0 by default so the
# advisory lane's pass/fail is still decided by the actual smoke run (research/web-gui-test-program.md
# §5 — this Linux smoke is advisory-forever and never gates the merge queue). Pass `--strict` to
# make a detected known-bad combination exit non-zero (reserved for if the lane is ever promoted
# to gating; unused today).
#
# Usage:
#   scripts/ci/tauri-driver-drift-tripwire.sh [--strict] [--tauri-driver-version X.Y.Z]
# If --tauri-driver-version is omitted it is read from the installed `tauri-driver` binary.

set -euo pipefail

STRICT=0
TAURI_DRIVER_VERSION=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    --tauri-driver-version) TAURI_DRIVER_VERSION="${2:-}"; shift 2 ;;
    --tauri-driver-version=*) TAURI_DRIVER_VERSION="${1#*=}"; shift ;;
    *) echo "tauri-drift-tripwire: unknown arg: $1" >&2; exit 2 ;;
  esac
done

# GitHub Actions annotation helpers degrade to plain echo when run locally (no $GITHUB_ACTIONS).
notice()  { if [ "${GITHUB_ACTIONS:-}" = "true" ]; then echo "::notice::$*";  else echo "NOTICE: $*";  fi; }
warning() { if [ "${GITHUB_ACTIONS:-}" = "true" ]; then echo "::warning::$*"; else echo "WARNING: $*"; fi; }
error()   { if [ "${GITHUB_ACTIONS:-}" = "true" ]; then echo "::error::$*";   else echo "ERROR: $*";   fi; }

# --- Gather versions ---------------------------------------------------------------------------

# webkit2gtk-driver apt version, e.g. "2.52.3-0ubuntu0.24.04.1" → we want the leading "2.52.3".
WEBKIT_APT_FULL="$(dpkg-query -W -f='${Version}' webkit2gtk-driver 2>/dev/null || echo "")"
WEBKIT_VER="$(printf '%s' "$WEBKIT_APT_FULL" | grep -oE '^[0-9]+\.[0-9]+\.[0-9]+' || echo "")"

# WebKitWebDriver banner (informational; on 2.52.x this prints a usage banner, not a version).
WKWD_BANNER="$(WebKitWebDriver --version 2>&1 | head -1 || echo "unknown")"

# tauri-driver version (from arg, else the installed binary's `--version`).
if [ -z "$TAURI_DRIVER_VERSION" ]; then
  TAURI_DRIVER_VERSION="$(tauri-driver --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "")"
fi

echo "── tauri-driver drift tripwire (bead sq-t6492 / issue #1740) ──────────────────────────"
echo "  tauri-driver version   : ${TAURI_DRIVER_VERSION:-unknown}"
echo "  webkit2gtk-driver apt  : ${WEBKIT_APT_FULL:-unknown}  (parsed: ${WEBKIT_VER:-unknown})"
echo "  WebKitWebDriver banner : ${WKWD_BANNER}"
echo "───────────────────────────────────────────────────────────────────────────────────────"

# --- Detect the known-incompatible combination ------------------------------------------------
# Known-bad: tauri-driver 2.0.x  AND  webkit2gtk 2.52.x (or any 2.5x ≥ 2.48 — the WebDriver
# behaviour change first ships in the 2.48 series). We flag 2.48/2.50/2.52 explicitly; anything
# newer than 2.52 is ALSO suspect and flagged so a future 2.54 bump does not slip past silently.

is_known_bad=0
case "$TAURI_DRIVER_VERSION" in
  2.0.*)
    case "$WEBKIT_VER" in
      2.4[89].*|2.5[0-9].*|2.[6-9][0-9].*|3.*)
        is_known_bad=1
        ;;
    esac
    ;;
esac

if [ "$is_known_bad" = "1" ]; then
  MSG="tauri-driver ${TAURI_DRIVER_VERSION} is KNOWN-INCOMPATIBLE with webkit2gtk ${WEBKIT_VER} \
(WebKitWebDriver capability negotiation fails: 'session not created: Failed to match capabilities'). \
This is the documented drift in bead sq-t6492 / issue #1740. The advisory GUI smoke is EXPECTED to \
be red on this combination. Fix path: build+run against a pinned older webkit2gtk (container, \
sq-t6492 follow-up) OR wait for a tauri-driver release that supports webkit2gtk 2.5x. Do NOT spend \
time on the 0-byte driver log — it is empty because the failure precedes any WebDriver session."
  if [ "$STRICT" = "1" ]; then
    error "$MSG"
    exit 1
  fi
  warning "$MSG"
  exit 0
fi

# Unknown-but-new: tauri-driver 2.0.x with a webkit2gtk we have not yet classified — surface it so
# the next drift is visible even if it turns out to be fine.
if [ "${TAURI_DRIVER_VERSION%%.*}" = "2" ] && [ -n "$WEBKIT_VER" ]; then
  notice "tauri-driver ${TAURI_DRIVER_VERSION} + webkit2gtk ${WEBKIT_VER}: not on the known-bad list. \
If the smoke fails on capability negotiation, extend the known-bad match in \
scripts/ci/tauri-driver-drift-tripwire.sh and update bead sq-t6492 / issue #1740."
fi

echo "tauri-driver drift tripwire: no known-incompatible combination detected."
exit 0
