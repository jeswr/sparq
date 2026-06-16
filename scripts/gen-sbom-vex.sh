#!/usr/bin/env bash
# [OPUS-4.8] sq-toze.3 (GX-2): per-release CycloneDX SBOM + VEX generator.
#
# Produces, into $OUT_DIR (default: ./sbom):
#   - sparq-cli-<version>.sbom.cdx.json     CycloneDX SBOM for the released sparq-cli binary
#   - sparq-server-<version>.sbom.cdx.json  CycloneDX SBOM for the released sparq-server binary
#   - sparq-<version>.vex.cdx.json          the checked-in VEX (supply-chain/vex.cdx.json) with
#                                           the released version + a generated timestamp stamped in
#
# All three are attached to the GitHub Release by .github/workflows/release.yml (and SLSA-
# attested there). Each SBOM enumerates that binary's full dependency tree (NTIA minimum
# elements: supplier, component name, version, unique purl, dependency relationships, author,
# timestamp). The VEX states the exploitability of every advisory cargo-deny is configured to
# ignore (kept 1:1 in sync with deny.toml [advisories].ignore).
#
# cargo-cyclonedx (`--all`) processes every workspace member and writes <crate>.cdx.json
# next to each member's Cargo.toml; we collect the two released-binary crates and discard the
# rest. `--all` is required here: the root is a virtual workspace (no [package]), so a bare
# `cargo cyclonedx` has no package to resolve. We use the **default** feature set (no
# --all-features) so each SBOM matches the actually-shipped binary — the release builds
# (release.yml / Dockerfile) build `-p sparq-cli`/`-p sparq-server` without --all-features,
# and the CI SBOM job (supply-chain.yml) likewise uses `cargo cyclonedx --all`.
#
# Requires: cargo, cargo-cyclonedx, jq, python3. Usage:
#   VERSION=v1.2.3 scripts/gen-sbom-vex.sh            # explicit version
#   scripts/gen-sbom-vex.sh                           # derives version from git describe
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

VERSION="${VERSION:-${GITHUB_REF_NAME:-$(git describe --tags --always 2>/dev/null || echo dev)}}"
OUT_DIR="${OUT_DIR:-sbom}"
mkdir -p "$OUT_DIR"

echo "==> generating CycloneDX SBOMs (whole workspace, sparq ${VERSION})"
cargo cyclonedx --all --format json

# [OPUS-4.8] sq-toze.30 (GS-6 / F-6): cargo-cyclonedx 0.5.9 stamps the absolute build dir
# into every workspace/path-dependency bom-ref (path+file:///abs/...#ver) and purl
# (?download_url=file://...), which would leak the CI runner's filesystem layout into the
# PUBLISHED SBOM. Normalize each shipped SBOM through scripts/sbom-normalize.jq, which
# rewrites those refs to the canonical, host-independent pkg:cargo/<name>@<version> form,
# rewriting the dependency graph (component bom-ref + every ref/dependsOn edge) in lock-step
# so internal references stay consistent. The transform is deterministic and idempotent.
NORMALIZE_JQ="$REPO_ROOT/scripts/sbom-normalize.jq"
for crate in sparq-cli sparq-server; do
  src="crates/${crate}/${crate}.cdx.json"
  dst="$OUT_DIR/${crate}-${VERSION}.sbom.cdx.json"
  jq -f "$NORMALIZE_JQ" "$src" > "$dst"
  rm -f "$src"
  # Belt-and-braces: fail loudly if any host-revealing absolute path survived.
  if grep -qE 'path\+file://|download_url=file://|/home/' "$dst"; then
    echo "ERROR: host path leaked into $dst after normalization" >&2
    exit 1
  fi
  echo "    -> $dst (abs-path-normalized)"
done

# Discard the per-member SBOMs we don't ship (keeps the worktree clean for `git status`).
find crates -name '*.cdx.json' -delete

echo "==> stamping VEX (sparq ${VERSION})"
VEX_OUT="$OUT_DIR/sparq-${VERSION}.vex.cdx.json"
python3 - "$VERSION" "$VEX_OUT" <<'PY'
import json, sys, datetime, pathlib
version, out = sys.argv[1], sys.argv[2]
ver = version.lstrip("v")
src = pathlib.Path("supply-chain/vex.cdx.json")
doc = json.loads(src.read_text())
meta = doc.setdefault("metadata", {})
meta["timestamp"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
comp = meta.setdefault("component", {})
comp["version"] = ver
comp["bom-ref"] = f"pkg:cargo/sparq@{ver}"
for v in doc.get("vulnerabilities", []):
    for a in v.get("affects", []):
        a["ref"] = comp["bom-ref"]
pathlib.Path(out).write_text(json.dumps(doc, indent=2) + "\n")
print(f"    -> {out}")
PY

echo "==> done. artifacts in $OUT_DIR/:"
ls -1 "$OUT_DIR/"
