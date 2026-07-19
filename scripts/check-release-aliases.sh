#!/usr/bin/env bash
# [FABLE-5] sq-vw3ax.11.3 — fail-closed release alias contract check.
#
# The /download page (site/src/app/download/download-client.tsx) renders direct-download
# buttons against version-STABLE alias assets served at
# https://github.com/<repo>/releases/latest/download/<alias>. release.yml stages alias
# COPIES of the canonical versioned assets (research/site-home-app-download-residuals.md §3);
# this script derives the alias set the site ACTUALLY references and verifies every one of
# them appears in the staged release asset list.
#
# FAIL-CLOSED: any site-referenced alias missing from the asset list exits non-zero, which
# blocks the release — a stable release must never ship silent-404 download buttons. EXTRA
# assets are always allowed (the normative table publishes more than the site references,
# e.g. the arm64-linux GUI/CLI aliases).
#
# The site-referenced set is extracted from the component source, so the SITE is the source
# of truth (renaming an alias there automatically retargets this check):
#   * GUI aliases:  every `file: "<name>"` string literal (primary + `secondary.file`).
#   * CLI aliases:  every `token: "<t>", ext: "<e>"` pair returned by cliTarget(), composed
#                   to `sparq-cli-<t>.<e>` (the component builds `sparq-cli-${token}.${ext}`).
# Extraction is guarded non-vacuous: if a refactor of download-client.tsx breaks either
# pattern, the shrunken contract fails this check loudly instead of passing emptily.
#
# Usage: check-release-aliases.sh <asset-list-file> [download-client.tsx]
#   asset-list-file      one staged release asset filename per line (e.g. `ls -1 assets`)
#   download-client.tsx  path to the site component (defaults to the in-repo path)

set -euo pipefail

ASSET_LIST="${1:?usage: check-release-aliases.sh <asset-list-file> [download-client.tsx]}"
CLIENT="${2:-site/src/app/download/download-client.tsx}"

if [ ! -f "$ASSET_LIST" ]; then
  echo "ERROR: asset list file not found: $ASSET_LIST" >&2
  exit 2
fi
if [ ! -f "$CLIENT" ]; then
  echo "ERROR: download client source not found: $CLIENT" >&2
  exit 2
fi

# GUI aliases: `file: "sparq-gui-....ext"` literals. The quote after the colon restricts the
# match to string literals (the `file: string;` interface field and doc comments never match).
# `|| true`: a zero-match grep must reach the non-vacuity guard below, not die under pipefail.
gui_aliases="$(grep -oE 'file: "[^"]+"' "$CLIENT" | sed -E 's/^file: "([^"]+)"$/\1/' | sort -u || true)"

# CLI aliases: cliTarget() returns `{ token: "<t>", ext: "<e>", label: ... }` per platform;
# the component downloads `sparq-cli-<t>.<e>`. Compose the same name per distinct pair.
cli_aliases="$(grep -oE 'token: "[^"]+", ext: "[^"]+"' "$CLIENT" \
  | sed -E 's/^token: "([^"]+)", ext: "([^"]+)"$/sparq-cli-\1.\2/' | sort -u || true)"

gui_count=$(printf '%s' "$gui_aliases" | grep -c . || true)
cli_count=$(printf '%s' "$cli_aliases" | grep -c . || true)

# Non-vacuity guard: the current component references 5 GUI + 4 CLI aliases. A collapse below
# these floors means the extraction patterns no longer match the source — fail loudly rather
# than approving an empty (or hollowed-out) contract.
if [ "$gui_count" -lt 2 ] || [ "$cli_count" -lt 2 ]; then
  echo "ERROR: alias extraction from $CLIENT looks broken (gui=$gui_count, cli=$cli_count)." >&2
  echo "       The component was likely refactored; update the extraction patterns in $0." >&2
  exit 2
fi

echo "site-referenced alias contract ($((gui_count + cli_count)) aliases) from $CLIENT:"
printf '%s\n%s\n' "$gui_aliases" "$cli_aliases" | sed 's/^/  - /'

missing=0
while IFS= read -r alias; do
  [ -n "$alias" ] || continue
  if ! grep -Fxq -- "$alias" "$ASSET_LIST"; then
    echo "MISSING: site-referenced alias '$alias' is not in the staged asset list" >&2
    missing=$((missing + 1))
  fi
done < <(printf '%s\n%s\n' "$gui_aliases" "$cli_aliases")

if [ "$missing" -gt 0 ]; then
  echo "FAIL: $missing site-referenced alias(es) missing — refusing to release (fail-closed)." >&2
  exit 1
fi

echo "OK: every site-referenced alias is present in the staged asset list."
