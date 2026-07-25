#!/usr/bin/env bash
# [GPT-5.6] sq-xngad — reproducible scalar-builtin cost probes.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="${CLI:-$ROOT/target/release/sparq-cli}"
ROWS="${ROWS:-100000}"
ITERS="${ITERS:-5}"
CACHE="${BUILTINS_CACHE:-/tmp/sparq-builtin-cost}"
DATA="$CACHE/rows-${ROWS}.nt"
RAW=""
QUERIES="${QUERY_DIR:-$ROOT/bench/builtins/queries}"

case "$ROWS:$ITERS" in
  *[!0-9:]*|0:*|*:0) echo "ERROR: ROWS and ITERS must be positive integers" >&2; exit 2 ;;
esac
[ -x "$CLI" ] || {
  echo "ERROR: sparq-cli not found at $CLI; run: cargo build --release -p sparq-cli" >&2
  exit 2
}

mkdir -p "$CACHE"
RAW="$(mktemp "$CACHE/results.XXXXXX.tsv")"
cleanup() {
  rm -f "${tmp:-}" "$RAW"
}
trap cleanup EXIT
if [ ! -f "$DATA" ] || [ "$(wc -l < "$DATA")" -ne "$ROWS" ]; then
  tmp="$DATA.tmp.$$"
  awk -v n="$ROWS" 'BEGIN {
    for (i = 0; i < n; i++) {
      printf "<urn:row:%d> <urn:value> \"row-%08d-alpha\" .\n", i, i
    }
  }' > "$tmp"
  mv "$tmp" "$DATA"
  tmp=""
fi

# Materialize bindings so projection expressions (especially RAND) cannot be optimized away.
"$CLI" bench "$DATA" ntriples "$QUERIES" "$ITERS" materialize > "$RAW"

expected_for() {
  case "$1" in
    regex_constant|replace_constant|rand_projection|now_query_constant) printf '%s\n' "$ROWS" ;;
    *) return 1 ;;
  esac
}

fail=0
seen=0
declare -A seen_names=()
while IFS= read -r line || [ -n "$line" ]; do
  if [[ "$line" != *$'\t'* ]]; then
    echo "ERROR: malformed TSV result row: $line" >&2
    fail=1
    continue
  fi
  name="${line%%$'\t'*}"
  remainder="${line#*$'\t'}"
  if [[ "$remainder" != *$'\t'* ]]; then
    echo "ERROR: malformed TSV result row: $line" >&2
    fail=1
    continue
  fi
  rows="${remainder%%$'\t'*}"
  micros="${remainder#*$'\t'}"
  if [ -z "$name" ] || [ -z "$rows" ] || [ -z "$micros" ] || [[ "$micros" == *$'\t'* ]]; then
    echo "ERROR: malformed TSV result row: $line" >&2
    fail=1
    continue
  fi
  expected="$(expected_for "$name")" || {
    echo "ERROR: unexpected result row: $name" >&2; fail=1; continue
  }
  if [ -n "${seen_names[$name]:-}" ]; then
    echo "ERROR: duplicate result row: $name" >&2
    fail=1
  fi
  seen_names[$name]=1
  seen=$((seen + 1))
  if [ "$rows" != "$expected" ]; then
    echo "ERROR: $name returned $rows rows; expected $expected" >&2
    fail=1
  fi
  if ! awk -v value="$micros" 'BEGIN {
    exit !(value ~ /^[0-9]+([.][0-9]+)?$/ && value + 0 > 0)
  }'; then
    echo "ERROR: $name emitted invalid microseconds: $micros" >&2
    fail=1
  fi
done < "$RAW"
[ "$seen" -eq 4 ] || { echo "ERROR: expected 4 probes, saw $seen" >&2; fail=1; }
[ "$fail" -eq 0 ] || exit 1

printf '%s\n\n' '> Work-box timings — NON-CANONICAL; canonical rerun: sq-98w7z.9.'
printf '| probe | result rows | min us | rows/s |\n'
printf '|---|---:|---:|---:|\n'
awk -F'\t' '{
  rate = ($3 > 0) ? ($2 * 1000000 / $3) : 0
  printf "| `%s` | %d | %.3f | %.0f |\n", $1, $2, $3, rate
}' "$RAW"
