#!/usr/bin/env bash
# [GPT-5.6] (sq-wzzxg) Self-relative HDT decode A/B. The Rust example performs
# the hard direct-vs-upstream graph-agreement assertion before printing timings;
# this wrapper preserves its nonzero exit and exposes the benchmark TSV contract.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

triples=1000000
if [ "${1:-}" = "--smoke" ]; then
  triples=100
elif [ "$#" -ne 0 ]; then
  echo "usage: $0 [--smoke]" >&2
  exit 2
fi

BIN="${HDT_DIRECT_AB_BIN:-$ROOT/target/release/examples/bench_direct_vs_upstream}"
if [ -z "${HDT_DIRECT_AB_BIN:-}" ]; then
  cargo build --release -p sparq-hdt --example bench_direct_vs_upstream >&2
fi
if [ ! -x "$BIN" ]; then
  echo "[hdt-direct-ab] ERROR: runner is not executable: $BIN" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# A panic or any other failure (including graph divergence) stops here before TSV
# is emitted. Thus machine-sensitive timings can never be reported after a failed
# correctness gate.
if ! "$BIN" "$triples" >"$TMP/raw.stdout" 2>"$TMP/raw.stderr"; then
  cat "$TMP/raw.stderr" >&2
  echo "[hdt-direct-ab] ERROR: graph-agreement gate or benchmark run failed" >&2
  exit 1
fi
cat "$TMP/raw.stderr" >&2

loaded="$(awk '/^loaded [0-9]+ triples / { print $2 }' "$TMP/raw.stdout")"
direct_s="$(awk '/^  direct / { print $3 }' "$TMP/raw.stdout" | sed 's/s$//')"
upstream_s="$(awk '/^  upstream / { print $3 }' "$TMP/raw.stdout" | sed 's/s$//')"

if ! [[ "$loaded" =~ ^[0-9]+$ ]] || [ "$loaded" != "$triples" ]; then
  echo "[hdt-direct-ab] ERROR: expected $triples agreed triples, got '${loaded:-missing}'" >&2
  exit 1
fi
if ! [[ "$direct_s" =~ ^[0-9]+([.][0-9]+)?$ ]] || ! [[ "$upstream_s" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "[hdt-direct-ab] ERROR: benchmark timing rows were missing or malformed" >&2
  exit 1
fi

direct_us="$(awk -v seconds="$direct_s" 'BEGIN { printf "%.0f", seconds * 1000000 }')"
upstream_us="$(awk -v seconds="$upstream_s" 'BEGIN { printf "%.0f", seconds * 1000000 }')"
printf 'direct_decode\t%s\t%s\n' "$loaded" "$direct_us"
printf 'upstream_decode\t%s\t%s\n' "$loaded" "$upstream_us"
echo "[hdt-direct-ab] OK: direct and upstream paths produced agreeing graphs" >&2
