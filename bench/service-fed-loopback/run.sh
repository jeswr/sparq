#!/usr/bin/env bash
# [GPT-5.6] sq-139od — SERVICE-federation differential vs Comunica.
#
# The Rust driver links the existing opt-in sparq-conformance service-loopback
# harness, starts every endpoint in-process, and runs the committed fixtures through
# sparq and Comunica. compare.py asserts canonical solution-multiset equality before
# emitting the correctness-only JSON envelope.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/bench/service-fed-loopback"
COMUNICA_DIR="$ROOT/bench/federation"
COMUNICA_RUNNER="$COMUNICA_DIR/comunica_runner.mjs"
COMUNICA_SPEC="${SERVICE_FED_COMUNICA_SPEC:-@comunica/query-sparql@^4}"
OUT="${SERVICE_FED_ENVELOPE:-$ROOT/bench/competitor-results/federation-service-loopback-$(date -u +%Y%m%dT%H%M%SZ).json}"

log() { printf '[service-fed-loopback] %s\n' "$*" >&2; }
die() { printf '[service-fed-loopback] ERROR: %s\n' "$*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || die "cargo is required"
command -v node >/dev/null 2>&1 || die "node is required for the Comunica oracle"
command -v npm >/dev/null 2>&1 || die "npm is required for the gather-time Comunica install"
command -v python3 >/dev/null 2>&1 || die "python3 is required for the multiset oracle"

python3 "$HERE/compare.py" --self-test

if ! node -e "require.resolve('@comunica/query-sparql/package.json', {paths:['$COMUNICA_DIR']}); require.resolve('n3/package.json', {paths:['$COMUNICA_DIR']})" >/dev/null 2>&1; then
  log "installing $COMUNICA_SPEC + n3 into the existing federation gather directory"
  npm install --no-save --no-audit --no-fund --prefix "$COMUNICA_DIR" \
    "$COMUNICA_SPEC" n3 >/dev/null \
    || die "npm install failed (network is needed for the gather-time oracle)"
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/sparq-service-fed-loopback.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

log "running committed fixtures through in-process sparq endpoints and Comunica"
cargo run --quiet --locked --manifest-path "$HERE/Cargo.toml" -- \
  --manifest "$HERE/fixtures/manifest.json" \
  --comunica-runner "$COMUNICA_RUNNER" >"$WORK/raw.json"

python3 "$HERE/compare.py" --input "$WORK/raw.json" --envelope "$OUT"
log "all canonical solution multisets agree"
