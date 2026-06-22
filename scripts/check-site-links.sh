#!/usr/bin/env bash
# [OPUS-4.8] sq-d8or — anti-drift CI glue: offline link-check over the BUILT docs-site HTML.
#
# WHY: the docs-quality `internal-links` job (bead sq-5fd1) runs lychee --offline over the
# repo's MARKDOWN, but the published GitHub Pages site (the Next.js static export under
# site/out/, plus the in-site rendered papers) is HTML — its relative links + #anchors were
# never link-checked, so a renamed route or a stale cross-link could ship broken to the live
# site. sq-d8or extends the lychee scope to that built HTML. This script is the shared,
# self-testable core (CLI; no Action) so the same check runs in CI (pages.yml, after
# `next build`) and locally (`scripts/check-site-links.sh site/out`).
#
# WHAT: lychee --offline (no network, deterministic) over every *.html under the export,
# validating relative links AND heading anchors (--include-fragments). The static export is
# served under the `/sparq` GitHub Pages base path (site/next.config.ts: basePath:"/sparq",
# trailingSlash:true), so every INTERNAL link in the emitted HTML is an absolute path like
#   /sparq/about/   (and the bare root link /sparq/).
# On disk those resolve to <out>/about/index.html and <out>/index.html — i.e. the `/sparq`
# prefix must be stripped to land in the export root. Three --remap rules do exactly that:
#   0. file://<out>/sparq/<path>#<frag> -> file://<out>/<path>/index.html#<frag>
#                                       (a CROSS-PAGE path+fragment link, see [OPUS-4.8] sq-bpoey)
#   1. file://<out>/sparq/(.*)  -> file://<out>/$1          (every /sparq/<path> link)
#   2. file://<out>/sparq       -> file://<out>/index.html  (the bare /sparq root link)
#
# [OPUS-4.8] sq-bpoey — RULE 0 is load-bearing for cross-page anchor links like
# `/sparq/capabilities/#privacy` (homepage theme grid + the removed /surface/* redirect stubs
# point at /capabilities/#<theme>). With `--root-dir`, lychee resolves a *directory* link that
# carries a fragment to the bare directory path (e.g. file://<out>/capabilities) and then,
# crucially, does NOT fall through to that directory's index.html when locating the `#fragment`
# heading — so it reports "Cannot find fragment" for EVERY such link. (Verified empirically
# against the CI lychee 0.23.0: a `/dir/#frag`, a `/dir#frag`, AND a `/dir/` form all fail; only
# an explicit `/dir/index.html#frag` resolves.) Adding a trailing slash before the fragment in
# the emitted link — as one might expect under trailingSlash:true — does NOT help; the fix must
# rewrite the directory to its index.html for the fragment lookup. Rule 0 runs FIRST so it owns
# the path+fragment case; rule 1 then handles every remaining (fragment-free) /sparq/<path> link.
# This is the regression that reddened the pages.yml link-check after PR #1003 introduced the
# first cross-page fragment links on the site (the self-test below only covered same-page #anchors).
#
# (lychee normalises a trailing-slash dir link to the slashless path before matching, so the
# two rules together cover both forms — verified empirically: 0 errors over the full export,
# and an injected dead link is caught.)
#
# EXIT: non-zero iff lychee finds a broken internal link/anchor (CI-gating). A self-test of
# the remap + teeth lives in scripts/tests/test_check_site_links.sh.
set -euo pipefail

OUT_DIR="${1:-site/out}"

if ! command -v lychee >/dev/null 2>&1; then
  echo "check-site-links: 'lychee' not found on PATH." >&2
  echo "  Install it (cargo install lychee) or run via the pinned lycheeverse action in CI." >&2
  exit 127
fi

if [ ! -d "$OUT_DIR" ]; then
  echo "check-site-links: export directory '$OUT_DIR' not found." >&2
  echo "  Build it first:  (cd site && npm ci && npm run build)" >&2
  exit 1
fi

# Absolute path so the file:// remap patterns are unambiguous regardless of CWD.
OUT_ABS="$(cd "$OUT_DIR" && pwd)"

echo "check-site-links: lychee --offline over ${OUT_ABS}/**/*.html (basePath /sparq remapped)"

# --root-dir lets lychee resolve absolute (/...) links against the export root; the two
# --remap rules strip the /sparq Pages base path. `dev/` (the overlaid benchmark dashboard,
# a separate first-party artifact written by bench.yml onto benchmark-data) is excluded:
# it is not part of THIS site's source and carries its own link surface.
lychee \
  --offline \
  --include-fragments \
  --no-progress \
  --root-dir "$OUT_ABS" \
  --remap "file://${OUT_ABS}/sparq/([^#]+)#(.+) file://${OUT_ABS}/\$1/index.html#\$2" \
  --remap "file://${OUT_ABS}/sparq/(.*) file://${OUT_ABS}/\$1" \
  --remap "file://${OUT_ABS}/sparq file://${OUT_ABS}/index.html" \
  --exclude-path "${OUT_ABS}/dev" \
  "${OUT_ABS}/**/*.html"
