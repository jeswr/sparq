#!/usr/bin/env bash
# [GPT-5.6] sq-xngad — mutation witnesses for the builtin-cost harness guards.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT/bench/builtins/run.sh"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/sparq-builtin-cost-test.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/fake-sparq-cli" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

[ "${1:-}" = bench ]
[ "${6:-}" = materialize ]
rows="${ROWS:?ROWS must be exported by the test}"
case "${FAKE_MUTATION:-none}" in
  none)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'replace_constant\t%s\t13.0\n' "$rows"
    ;;
  cardinality)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t0\t12.0\n'
    printf 'replace_constant\t%s\t13.0\n' "$rows"
    ;;
  timing)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\tNaN\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'replace_constant\t%s\t13.0\n' "$rows"
    ;;
  zero_timing)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'replace_constant\t%s\t13.0\n' "$rows"
    ;;
  unknown)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'unknown_probe\t%s\t13.0\n' "$rows"
    ;;
  duplicate)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'regex_constant\t%s\t13.0\n' "$rows"
    ;;
  missing)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    ;;
  extra_field)
    printf 'now_query_constant\t%s\t10.0\n' "$rows"
    printf 'rand_projection\t%s\t11.0\n' "$rows"
    printf 'regex_constant\t%s\t12.0\n' "$rows"
    printf 'replace_constant\t%s\t13.0\tunexpected\n' "$rows"
    ;;
  *) exit 64 ;;
esac
SH
chmod +x "$TMP/fake-sparq-cli"

run_case() {
  local mutation="$1"
  FAKE_MUTATION="$mutation" ROWS=7 ITERS=1 \
    CLI="$TMP/fake-sparq-cli" BUILTINS_CACHE="$TMP/cache-$mutation" \
    "$RUNNER"
}

expect_failure() {
  local mutation="$1"
  local message="$2"
  if run_case "$mutation" > "$TMP/$mutation.out" 2> "$TMP/$mutation.err"; then
    echo "ERROR: $mutation mutation escaped the harness" >&2
    exit 1
  fi
  grep -q "$message" "$TMP/$mutation.err"
}

run_case none > "$TMP/ok.out"
grep -q 'NON-CANONICAL' "$TMP/ok.out"
# shellcheck disable=SC2016 # literal Markdown code-span delimiters, not command substitution
grep -q '| `regex_constant` | 7 |' "$TMP/ok.out"

# Mutation witnesses: each result-integrity guard must make the harness red rather than emit a
# plausible-looking table for corrupted, incomplete, duplicated, or malformed CLI output.
expect_failure cardinality 'regex_constant returned 0 rows; expected 7'
expect_failure timing 'rand_projection emitted invalid microseconds: NaN'
expect_failure zero_timing 'rand_projection emitted invalid microseconds: 0'
expect_failure unknown 'unexpected result row: unknown_probe'
expect_failure duplicate 'duplicate result row: regex_constant'
expect_failure missing 'expected 4 probes, saw 3'
expect_failure extra_field 'malformed TSV result row:'

# Two runners sharing a generated-data cache must not clobber one another's raw result file.
FAKE_MUTATION=none ROWS=7 ITERS=1 CLI="$TMP/fake-sparq-cli" \
  BUILTINS_CACHE="$TMP/cache-concurrent" "$RUNNER" > "$TMP/concurrent-a.out" &
first_pid=$!
FAKE_MUTATION=none ROWS=7 ITERS=1 CLI="$TMP/fake-sparq-cli" \
  BUILTINS_CACHE="$TMP/cache-concurrent" "$RUNNER" > "$TMP/concurrent-b.out" &
second_pid=$!
wait "$first_pid"
wait "$second_pid"
grep -q 'NON-CANONICAL' "$TMP/concurrent-a.out"
grep -q 'NON-CANONICAL' "$TMP/concurrent-b.out"
if find "$TMP/cache-concurrent" -maxdepth 1 -name 'results.*.tsv' -print -quit | grep -q .; then
  echo "ERROR: concurrent runners left a raw result file behind" >&2
  exit 1
fi

echo "builtin-cost harness self-test: PASS"
