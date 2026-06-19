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
if [ ! -f "$LOCKFILE" ]; then
  echo "ERROR: $LOCKFILE not found — cannot SBOM the JS client" >&2
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
    --omit dev \
    --mc-type library \
    --output-file "$runtime_out" )

# Full build-time tree (incl. devDependencies) — parity with the Rust full-tree SBOM.
echo "    -> full build tree (incl. dev): $dev_out"
( cd "$JS_DIR" && npx --yes "$CDX_NPM" \
    --spec-version 1.5 \
    --output-format JSON \
    --package-lock-only \
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

echo "==> done. JS SBOMs in $OUT_DIR/:"
for f in "$OUT_DIR"/sparq-js*.cdx.json; do
  [ -e "$f" ] && echo "    $f"
done
