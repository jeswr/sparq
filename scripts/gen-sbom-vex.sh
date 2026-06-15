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
# cargo-cyclonedx processes the whole workspace and writes <crate>.cdx.json next to each
# member's Cargo.toml; we collect the two released-binary crates and discard the rest.
#
# Requires: cargo, cargo-cyclonedx, python3. Usage:
#   VERSION=v1.2.3 scripts/gen-sbom-vex.sh            # explicit version
#   scripts/gen-sbom-vex.sh                           # derives version from git describe
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

VERSION="${VERSION:-${GITHUB_REF_NAME:-$(git describe --tags --always 2>/dev/null || echo dev)}}"
OUT_DIR="${OUT_DIR:-sbom}"
mkdir -p "$OUT_DIR"

echo "==> generating CycloneDX SBOMs (whole workspace, sparq ${VERSION})"
cargo cyclonedx --format json --all-features

for crate in sparq-cli sparq-server; do
  src="crates/${crate}/${crate}.cdx.json"
  dst="$OUT_DIR/${crate}-${VERSION}.sbom.cdx.json"
  mv "$src" "$dst"
  echo "    -> $dst"
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
