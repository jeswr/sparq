#!/usr/bin/env zsh
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sparq-ieee754-public-api.XXXXXX")"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

write_manifest() {
  local package_dir="$1"
  local package_name="$2"

  mkdir -p "$package_dir/src"

  cat > "$package_dir/Nargo.toml" <<EOF
[package]
name = "$package_name"
type = "lib"
authors = [""]

[dependencies]
sparq_ieee754 = { path = "$repo_root" }
EOF
}

run_expected_failure() {
  local package_dir="$1"
  local expected_text="$2"

  cd "$package_dir"

  if nargo test > output.txt 2>&1; then
    cat output.txt
    echo "expected public API rejection, but package compiled"
    exit 1
  fi

  if ! grep -Eq "$expected_text" output.txt; then
    cat output.txt
    echo "expected compiler output to contain: $expected_text"
    exit 1
  fi
}

run_hidden_symbol_check() {
  local symbol="$1"
  local package_name="$2"
  local package_dir="$tmp_dir/$package_name"

  write_manifest "$package_dir" "$package_name"

  cat > "$package_dir/src/lib.nr" <<EOF
use sparq_ieee754::$symbol;

#[test]
fn symbol_is_not_public_api() {}
EOF

  run_expected_failure "$package_dir" "Could not resolve|private"
}

positive_dir="$tmp_dir/positive"
private_methods_dir="$tmp_dir/private_methods"

write_manifest "$positive_dir" "sparq_ieee754_public_api"
write_manifest "$private_methods_dir" "sparq_ieee754_private_methods"

cp "$repo_root/tests/public_api/src/lib.nr" "$positive_dir/src/lib.nr"
cp "$repo_root/tests/private_methods/src/lib.nr" "$private_methods_dir/src/lib.nr"

cd "$positive_dir"
nargo test

hidden_symbols=(
  FloatParts
  ops
  ops::add
  ops::add_wide
  ops::sub
  ops::mul
  ops::div
  ops::uint_to_float_parts_u64
  ops::signed_magnitude_i8
  ops::compare_parts
  ops::round_to_integral
  ops::float_to_u64
  ops::float_to_i64_wide
  ops::sqrt_op
  uint_type
  exponent_size
  mantissa_size
  float_type_from_params
  uint_type_from_float_params
  generate_float_type
  codegen
  parts
  sizing
)

index=0
for symbol in "${hidden_symbols[@]}"; do
  run_hidden_symbol_check "$symbol" "sparq_ieee754_hidden_symbol_$index"
  index=$((index + 1))
done

private_fields=(
  sign
  exponent
  mantissa
)

for field in "${private_fields[@]}"; do
  private_field_dir="$tmp_dir/private_field_$field"
  write_manifest "$private_field_dir" "sparq_ieee754_private_field_$field"
  cp "$repo_root/tests/private_fields/$field/src/lib.nr" "$private_field_dir/src/lib.nr"
  run_expected_failure "$private_field_dir" "private"
done

run_expected_failure "$private_methods_dir" "private"