#!/usr/bin/env bash
# Regenerate the checked-in rdf-shuttle-generated parsers (sq-tonhr.6).
#
# Node lives ONLY here (and in any CI drift lane that calls this) — never in
# `cargo build` or the test suite. The generated .rs artifacts, the vendored
# grammar, and the provenance pins are committed together; the pure-Rust
# tests/provenance_drift.rs test keeps them from drifting apart.
#
# usage: scripts/regen-shuttle-parsers.sh [path-to-rdf-shuttle-checkout]
#   (default: clones jeswr/rdf-shuttle to a temp dir at origin/main)
set -euo pipefail
cd "$(dirname "$0")/.."

SHUTTLE="${1:-}"
CLEANUP=""
if [ -z "$SHUTTLE" ]; then
  SHUTTLE=$(mktemp -d)
  CLEANUP="$SHUTTLE"
  git clone --quiet --depth 1 https://github.com/jeswr/rdf-shuttle.git "$SHUTTLE"
fi
trap '[ -n "$CLEANUP" ] && rm -rf "$CLEANUP"' EXIT

COMMIT=$(git -C "$SHUTTLE" rev-parse HEAD)
CRATE=crates/sparq-shaclc

(cd "$SHUTTLE/packages/gen-rs" \
  && node src/cli.js ../../grammars/shaclc12ext.shuttle -o /tmp/shaclc12.gen.rs --profile rdf12 \
  && node src/cli.js ../../grammars/shaclc12ext.shuttle -o /tmp/shaclc12ext.gen.rs --profile rdf12,ext)

cp /tmp/shaclc12.gen.rs    "$CRATE/src/raw/shaclc12.rs"
cp /tmp/shaclc12ext.gen.rs "$CRATE/src/raw/shaclc12ext.rs"
cp "$SHUTTLE/grammars/shaclc12ext.shuttle" "$CRATE/grammar/shaclc12ext.shuttle"
rm -f /tmp/shaclc12.gen.rs /tmp/shaclc12ext.gen.rs

HASH=$(sha256sum "$CRATE/grammar/shaclc12ext.shuttle" | cut -d' ' -f1)

cat > "$CRATE/src/provenance.rs" <<EOF
//! GENERATED provenance pins — written by \`scripts/regen-shuttle-parsers.sh\`.
//! DO NOT EDIT by hand: regenerate alongside the artifacts in \`src/raw/\`.

/// The \`jeswr/rdf-shuttle\` commit whose \`gen-rs\` backend generated the
/// artifacts in \`src/raw/\` (and whose grammar is vendored in \`grammar/\`).
pub const RDF_SHUTTLE_COMMIT: &str = "$COMMIT";

/// SHA-256 of the vendored \`grammar/shaclc12ext.shuttle\` at generation time.
/// \`tests/provenance_drift.rs\` recomputes this from the vendored file, so a
/// grammar edit without regeneration turns the suite red.
pub const GRAMMAR_SHA256: &str = "$HASH";

/// The gen-rs CLI invocations that produced the two artifacts.
pub const REGEN_COMMANDS: &[&str] = &[
    "shuttle-gen-rs grammars/shaclc12ext.shuttle -o shaclc12.rs --profile rdf12",
    "shuttle-gen-rs grammars/shaclc12ext.shuttle -o shaclc12ext.rs --profile rdf12,ext",
];
EOF

echo "regenerated from rdf-shuttle $COMMIT (grammar sha256 $HASH)"
echo "now run: cargo test -p sparq-shaclc && cargo clippy -p sparq-shaclc --all-targets -- -D warnings"
