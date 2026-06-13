#!/usr/bin/env bash
# [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
#
# Wall-clock bb prove/verify for one circuit-family member. Generates a witness
# from the member's current Prover.toml (write one first, e.g. via the crate's
# e2e tests, or by hand), then times prove (--write_vk) and verify.
#
# Usage:  bench/zk-compose/scripts/prove_verify.sh <member>
#   e.g.  bench/zk-compose/scripts/prove_verify.sh filter_int_d1
set -euo pipefail

PKG="${1:?usage: prove_verify.sh <member>}"
COMPOSE_DIR="$(cd "$(dirname "$0")/../../../zk/compose" && pwd)"
OUT="$(mktemp -d)"
TARGET="noir-recursive"

cd "$COMPOSE_DIR"
nargo compile --package "$PKG" >/dev/null 2>&1
nargo execute "${PKG}_bench" --package "$PKG" >/dev/null 2>&1

echo "== prove ($PKG) =="
time bb prove -b "target/$PKG.json" -w "target/${PKG}_bench.gz" -o "$OUT" --write_vk -t "$TARGET"
echo "proof bytes: $(wc -c < "$OUT/proof"), vk bytes: $(wc -c < "$OUT/vk")"

echo "== verify ($PKG) =="
time bb verify -p "$OUT/proof" -i "$OUT/public_inputs" -k "$OUT/vk" -t "$TARGET"
echo "verify exit: $?"
rm -rf "$OUT"
