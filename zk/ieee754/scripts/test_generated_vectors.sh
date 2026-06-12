#!/usr/bin/env zsh
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
fixture="$repo_root/tests/generated_arithmetic/src/lib.nr"

if [[ ! -f "$fixture" ]]; then
  python3 "$repo_root/scripts/generate_float_vectors.py" --output "$fixture"
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sparq-ieee754-generated-vectors.XXXXXX")"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

package_dir="$tmp_dir/generated_vectors"
mkdir -p "$package_dir/src"

cat > "$package_dir/Nargo.toml" <<EOF
[package]
name = "sparq_ieee754_generated_vectors"
type = "lib"
authors = [""]

[dependencies]
sparq_ieee754 = { path = "$repo_root" }
EOF

cp "$fixture" "$package_dir/src/lib.nr"

cd "$package_dir"
nargo test