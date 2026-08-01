#!/usr/bin/env bash
# [SONNET-4.6] Cost-capped SP2Bench heavy/full-scale runner for EC2/nightly.
set -euo pipefail
cd "$(dirname "$0")/../.."
CLI="${SP2B_CLI:-target/release/sparq-cli}"
GEN="${SP2B_GEN:-bench/sp2b/gen.sh}"
EXPECTED="${SP2B_EC2_EXPECTED:-bench/sp2b/expected-rows-ec2.tsv}"
TIMEOUT_SECONDS="${SP2B_QUERY_TIMEOUT_SECONDS:-900}"
SCALES="${SP2B_FULL_SCALES:-5000000 25000000 100000000}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

run_query() {
  local scale="$1" corpus="$2" query="$3" source="$4"
  local query_dir="$TMP/query-$scale-$query" output="$TMP/out-$scale-$query.tsv"
  mkdir -p "$query_dir"; cp "$source" "$query_dir/$query.rq"
  if ! timeout --signal=TERM --kill-after=30s "$TIMEOUT_SECONDS" \
      "$CLI" bench "$corpus" turtle "$query_dir" 1 count > "$output"; then
    echo "ERROR: sp2b scale=$scale query=$query exceeded ${TIMEOUT_SECONDS}s or failed" >&2; return 1
  fi
  local got us expected
  got="$(awk -F '\t' -v query="$query" '$1 == query { print $2 }' "$output")"
  us="$(awk -F '\t' -v query="$query" '$1 == query { print $3 }' "$output")"
  expected="$(awk -F '\t' -v scale="$scale" -v query="$query" '$1 == scale && $2 == query { print $3 }' "$EXPECTED")"
  if [ -z "$got" ] || [ "$got" = ERROR ] || [ -z "$us" ]; then
    echo "ERROR: sp2b scale=$scale query=$query produced malformed output" >&2; return 1
  fi
  if [ -z "$expected" ] || [ "$got" != "$expected" ]; then
    echo "ERROR: sp2b scale=$scale query=$query rows=$got expected=${expected:-MISSING}" >&2; return 1
  fi
  printf 'sp2b_%s_%s_count_us\t%s\t%s\n' "$scale" "$query" "$got" "$us"
}

corpus="$($GEN 250000)"
for source in bench/sp2b/queries-heavy/*.rq; do
  query="$(basename "$source" .rq)"; run_query 250000 "$corpus" "$query" "$source"
done
for scale in $SCALES; do
  corpus="$($GEN "$scale")"
  while IFS=$'\t' read -r expected_scale query _; do
    case "$expected_scale" in \#*|"") continue ;; esac
    [ "$expected_scale" = "$scale" ] || continue
    source="bench/sp2b/queries/$query.rq"; [ -f "$source" ] || source="bench/sp2b/queries-heavy/$query.rq"
    [ -f "$source" ] || { echo "ERROR: missing SP2Bench query $query" >&2; exit 1; }
    run_query "$scale" "$corpus" "$query" "$source"
  done < "$EXPECTED"
done
