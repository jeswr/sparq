#!/usr/bin/env bash
# [OPUS-4.8] sq-toze.27 (GS-3): dedicated CycloneDX SBOM for the PUBLISHED npm/JS
# surface — the WASM client `@jeswr/sparq` (the `js/` workspace).
#
# WHY: the Rust workspace is SBOM'd (scripts/gen-sbom-vex.sh, supply-chain.yml#sbom),
# but the JS side that ships to npm — the WASM client at `js/`, with its OWN
# package-lock.json dependency tree — was an un-SBOM'd supply-chain surface (gap GS-3).
# This generates a CycloneDX SBOM for that lockfile so the published npm package's
# dependency tree is enumerated alongside the Rust SBOMs.
#
# SCOPE DECISION (honest):
#   IN  — `js/` (`@jeswr/sparq`): the package actually PUBLISHED to npm + the WASM client
#         consumers depend on. Its only runtime dependency is `fzstd`; `typescript`,
#         `@types/node`, `@rdfjs/types` are devDependencies (build-time only, NOT in the
#         published tarball — `package.json` `files: [dist, wasm, README.md, …]`).
#   OUT — `site/` (`sparq-site`): `package.json` is `"private": true` — the GitHub-Pages
#         demo app, NEVER published and NOT a consumable artifact. Its (Next.js, React,
#         bb.js, …) tree is a DEV/showcase surface covered by npm-ecosystem Dependabot,
#         not part of any shipped product, so it is intentionally out of the release SBOM.
#         (If `site/` is ever shipped, add it here with the same `--omit dev` reasoning.)
#
#   We emit TWO views of `js/` and ship BOTH:
#     - sparq-js-<version>.sbom.cdx.json       runtime tree (`--omit dev`): what a
#                                              CONSUMER of @jeswr/sparq installs. This is
#                                              the primary released SBOM (matches the
#                                              published-tarball dependency surface).
#     - sparq-js-dev-<version>.sbom.cdx.json   full tree (incl. devDependencies): the
#                                              BUILD-time surface, for SSDF/supply-chain
#                                              completeness (parity with the Rust SBOM,
#                                              which enumerates the full build tree).
#
# CycloneDX 1.5 to match the Rust SBOM + the VEX. `--package-lock-only` so the tree is
# read from the committed package-lock.json without a network install (deterministic;
# the lockfile is the source of truth). `--validate` (cyclonedx-npm default) validates
# the output against the bundled CycloneDX 1.5 schema; we DON'T pass `--no-validate`.
#
# Requires: node, npx (cyclonedx-npm is run via npx at the pinned version). Usage:
#   VERSION=v1.2.3 scripts/gen-js-sbom.sh        # explicit version
#   scripts/gen-js-sbom.sh                       # derives version from git describe
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

VERSION="${VERSION:-${GITHUB_REF_NAME:-$(git describe --tags --always 2>/dev/null || echo dev)}}"
# [OPUS-4.8] sq-toze.27: SANITIZE the version before it becomes part of a FILENAME.
# On a pull_request event GITHUB_REF_NAME is "<pr-number>/merge" (e.g. "348/merge"); the
# embedded `/` made the --output-file path "sbom/sparq-js-348/merge.sbom.cdx.json", i.e.
# cyclonedx-npm silently wrote into a `sparq-js-348/` SUBDIRECTORY. The script still exited
# 0 (the file existed at its scattered path), but the workflow's non-recursive upload glob
# `sbom/sparq-js*.cdx.json` then matched nothing and `if-no-files-found: error` failed the
# job (exit 1). Collapse any character that is not a safe filename atom (so `/`, whitespace,
# etc.) to a single `-` so the SBOM always lands flat in $OUT_DIR where the glob expects it.
VERSION="$(printf '%s' "$VERSION" | tr -cs 'A-Za-z0-9._-' '-')"
VERSION="${VERSION:-dev}"
OUT_DIR="${OUT_DIR:-sbom}"
mkdir -p "$OUT_DIR"
# [OPUS-4.8] sq-toze.27: anchor OUT_DIR to an ABSOLUTE path. cyclonedx-npm runs inside
# `( cd "$JS_DIR" && … )`, so a RELATIVE --output-file would land under js/$OUT_DIR (the
# subshell cwd) instead of $REPO_ROOT/$OUT_DIR — which is exactly what made the validate +
# upload steps ENOENT on the path they expect. Resolving to absolute keeps writes and reads
# pointed at the same directory regardless of the working dir cyclonedx-npm runs in.
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

# Pin the generator so a silent upstream bump can't change the output shape.
CDX_NPM="@cyclonedx/cyclonedx-npm@5.0.0"

JS_DIR="$REPO_ROOT/js"
LOCKFILE="$JS_DIR/package-lock.json"

# [OPUS-4.8] sq-jpki.1: repo-root npm-workspaces migration — the per-member
# `js/package-lock.json` was DROPPED as redundant; the single source of truth is the repo-root
# `package-lock.json`. cyclonedx-npm's `--package-lock-only` reads a lock CO-LOCATED with the
# manifest it scans, so we DERIVE a transient, self-contained `js/package-lock.json` out of the
# root lock (preserving the EXACT pinned versions; no `^`-range re-resolution, no network — the
# js-sbom lane is deliberately install-free + deterministic). We pass `--no-workspaces` to
# cyclonedx so it does NOT climb to the workspace root + report `sparq-monorepo` as the root
# component; the SBOM stays anchored at the published client `@jeswr/sparq` (root component
# name `sparq`), exactly as before the migration. The derived lock is scratch — removed on exit.
ROOT_LOCK="$REPO_ROOT/package-lock.json"
if [ ! -f "$ROOT_LOCK" ]; then
  echo "ERROR: repo-root $ROOT_LOCK not found — cannot derive the JS client lock for SBOM" >&2
  exit 1
fi
DERIVED_LOCK=0
if [ ! -f "$LOCKFILE" ]; then
  node "$REPO_ROOT/scripts/derive-workspace-member-lock.mjs" js "$LOCKFILE"
  DERIVED_LOCK=1
  # Remove the transient derived lock on ANY exit so it can never be staged/committed and the
  # working tree is left as it was found (single root lock; no per-member lock).
  trap 'if [ "$DERIVED_LOCK" -eq 1 ]; then rm -f "$LOCKFILE"; fi' EXIT
fi
if [ ! -f "$LOCKFILE" ]; then
  echo "ERROR: $LOCKFILE not found and could not be derived — cannot SBOM the JS client" >&2
  exit 1
fi

runtime_out="$OUT_DIR/sparq-js-${VERSION}.sbom.cdx.json"
dev_out="$OUT_DIR/sparq-js-dev-${VERSION}.sbom.cdx.json"

echo "==> generating CycloneDX SBOM for the npm client @jeswr/sparq (${VERSION})"

# Runtime tree (what a consumer installs) — the primary released JS SBOM.
echo "    -> runtime tree (--omit dev): $runtime_out"
( cd "$JS_DIR" && npx --yes "$CDX_NPM" \
    --spec-version 1.5 \
    --output-format JSON \
    --package-lock-only \
    --no-workspaces \
    --omit dev \
    --mc-type library \
    --output-file "$runtime_out" )

# [OPUS-4.8] sq-pl1p (sq-f04e/#887 + #922 class): full build-time tree (incl. devDependencies)
# — parity with the Rust full-tree SBOM. `--ignore-npm-errors` is REQUIRED here, and ONLY here.
#
# WHY (the exact #922 break mechanism): cyclonedx-npm shells out to
# `npm ls --json --long --all --package-lock-only` (NO `--omit`) to enumerate the FULL tree, then
# THROWS (exit 254) if that npm ls exits non-zero AND --ignore-npm-errors is unset (its default;
# cyclonedx-npm builders.js fetchNpmLs). When a PUBLISHED member (js/ = @jeswr/sparq) declares an
# EXTERNAL (registry) devDependency, derive-workspace-member-lock.mjs projects that dep's full
# transitive closure into the standalone lock. A real dev-dep closure (e.g. @solid/acl-check ->
# rdflib -> undici/cross-fetch, or n3) routinely carries UNSATISFIED `peerDependencies` and
# version-overlap that `npm ls --package-lock-only` validates and flags as `missing`/`invalid`
# (ELSPROBLEMS). That is INSTALL-FREE lock-VALIDATION noise — npm never actually installs here —
# not a real defect, but the FULL view trips it (the runtime `--omit dev` view does NOT: it prunes
# the entire dev subtree BEFORE validation, so it stays strict, see above). #922 worked around
# this by relocating 3 test-only dev-deps to the repo-root package.json; this is the PROPER fix so
# any future external dev-dep on a published member can't re-trip exit 254.
#
# HONESTY: --ignore-npm-errors only suppresses npm ls's NON-ZERO EXIT; cyclonedx still consumes
# npm ls's STDOUT, so the FULL component set (every dev dep + its closure) is still enumerated —
# nothing is dropped from the build-time SBOM. The runtime SBOM above keeps STRICT validation (NO
# --ignore-npm-errors) so a genuine runtime-graph version conflict still reds the lane. And the
# superset guard below asserts the dev SBOM still contains every runtime component, so this flag
# can never silently hide a real runtime supply-chain surface.
echo "    -> full build tree (incl. dev): $dev_out"
( cd "$JS_DIR" && npx --yes "$CDX_NPM" \
    --spec-version 1.5 \
    --output-format JSON \
    --package-lock-only \
    --no-workspaces \
    --ignore-npm-errors \
    --mc-type library \
    --output-file "$dev_out" )

# [OPUS-4.8] sq-toze.27: fail LOUDLY (not with a bare downstream ENOENT) if cyclonedx-npm
# exited 0 but did not actually emit the file we expect at the path we expect.
for f in "$runtime_out" "$dev_out"; do
  if [ ! -f "$f" ]; then
    echo "ERROR: expected CycloneDX SBOM was not produced at: $f" >&2
    echo "       (cyclonedx-npm returned success but wrote nothing here — check OUT_DIR/cwd)" >&2
    exit 1
  fi
done

# Belt-and-braces sanity: both are valid-shaped CycloneDX JSON for the right component.
for f in "$runtime_out" "$dev_out"; do
  node -e '
    const fs = require("fs");
    const d = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    if (d.bomFormat !== "CycloneDX") { console.error("not CycloneDX:", process.argv[1]); process.exit(1); }
    if (d.specVersion !== "1.5") { console.error("not specVersion 1.5:", process.argv[1]); process.exit(1); }
    const name = d.metadata && d.metadata.component && d.metadata.component.name;
    if (name !== "sparq") { console.error("unexpected root component", name, "in", process.argv[1]); process.exit(1); }
    const purls = (d.components || []).map(c => c.purl).filter(Boolean);
    if (purls.some(p => !p.startsWith("pkg:npm/"))) { console.error("non-npm purl in", process.argv[1]); process.exit(1); }
    console.log("    OK", process.argv[1], "—", (d.components||[]).length, "component(s), specVersion", d.specVersion);
  ' "$f"
done

# [OPUS-4.8] sq-pl1p: HONESTY GUARD for the --ignore-npm-errors on the dev SBOM. Assert the dev
# (full) SBOM is a SUPERSET of the runtime SBOM by purl — i.e. the relaxed dev pass can never
# silently DROP a real runtime supply-chain component (which would understate the shipped surface).
# If a runtime purl is missing from the dev tree, fail LOUDLY rather than emit a dishonest SBOM.
echo "    -> honesty guard: dev SBOM must contain every runtime component"
node -e '
  const fs = require("fs");
  const purls = (p) => new Set((JSON.parse(fs.readFileSync(p, "utf8")).components || [])
    .map(c => c.purl).filter(Boolean));
  const runtime = purls(process.argv[1]);
  const dev = purls(process.argv[2]);
  const missing = [...runtime].filter(p => !dev.has(p));
  if (missing.length) {
    console.error("ERROR: dev SBOM is NOT a superset of the runtime SBOM — runtime purl(s) absent");
    console.error("       from the dev tree (the --ignore-npm-errors pass dropped a real runtime");
    console.error("       component): " + JSON.stringify(missing));
    process.exit(1);
  }
  console.log("    OK runtime SBOM (" + runtime.size + " purl) is a subset of the dev SBOM (" + dev.size + " purl)");
' "$runtime_out" "$dev_out"

echo "==> done. JS SBOMs in $OUT_DIR/:"
for f in "$OUT_DIR"/sparq-js*.cdx.json; do
  [ -e "$f" ] && echo "    $f"
done
