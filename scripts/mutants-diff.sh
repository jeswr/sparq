#!/usr/bin/env bash
# [SONNET-4.6] sq-qcnn.30 — bounded, changed-code-only mutation review signal.
set -euo pipefail

DIFF_FILE=${1:-mutants-pr.diff}
MUTANT_CAP=${MUTANTS_DIFF_CAP:-12}
MUTANT_TIMEOUT=${MUTANTS_DIFF_TIMEOUT_SECONDS:-120}
SUMMARY_FILE=${GITHUB_STEP_SUMMARY:-/dev/stdout}
LIST_FILE=${RUNNER_TEMP:-/tmp}/sparq-mutants-diff-list.json

case "$MUTANT_CAP" in
  ''|*[!0-9]*|0) echo "MUTANTS_DIFF_CAP must be a positive integer" >&2; exit 64 ;;
esac
case "$MUTANT_TIMEOUT" in
  ''|*[!0-9]*|0) echo "MUTANTS_DIFF_TIMEOUT_SECONDS must be a positive integer" >&2; exit 64 ;;
esac
if [[ ! -s "$DIFF_FILE" ]]; then
  {
    echo "### Changed-code mutation review"
    echo
    echo "No changed Rust code was present in the merge-base diff; no mutants were run."
  } >> "$SUMMARY_FILE"
  exit 0
fi

# List first so a large PR cannot silently turn this advisory signal into a full sweep.
# --in-diff is applied before sharding. With deterministic sharding, shard zero
# contains at most MUTANT_CAP entries when the denominator is ceil(total / cap).
if ! cargo mutants --in-diff "$DIFF_FILE" --list --json > "$LIST_FILE"; then
  {
    echo "### Changed-code mutation review"
    echo
    echo "cargo-mutants could not enumerate the changed-code mutants; inspect the advisory job log."
  } >> "$SUMMARY_FILE"
  echo "cargo-mutants could not enumerate the changed-code mutants" >&2
  exit 1
fi
TOTAL=$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1], encoding="utf-8"))))' "$LIST_FILE")

if (( TOTAL == 0 )); then
  {
    echo "### Changed-code mutation review"
    echo
    echo "The merge-base diff produced no Rust mutants."
  } >> "$SUMMARY_FILE"
  exit 0
fi

SHARDS=$(( (TOTAL + MUTANT_CAP - 1) / MUTANT_CAP ))
SELECTED=$(( (TOTAL + SHARDS - 1) / SHARDS ))
ARGS=(--in-diff "$DIFF_FILE" --no-shuffle --timeout "$MUTANT_TIMEOUT")
if (( SHARDS > 1 )); then
  ARGS+=(--shard "0/$SHARDS")
fi

set +e
cargo mutants "${ARGS[@]}"
STATUS=$?
set -e

SURVIVORS=0
if [[ -f mutants.out/missed.txt ]]; then
  SURVIVORS=$(grep -cve '^[[:space:]]*$' mutants.out/missed.txt || true)
fi
{
  echo "### Changed-code mutation review"
  echo
  echo "- Candidate mutants in the merge-base diff: $TOTAL"
  echo "- Mutants selected by the deterministic cap: $SELECTED (cap: $MUTANT_CAP)"
  echo "- Surviving mutants: $SURVIVORS"
  echo "- cargo-mutants exit status: $STATUS"
  echo
  if (( SURVIVORS > 0 )); then
    echo "Survivors are advisory review signals: tests for the changed code did not notice these mutations."
    echo
    echo '```text'
    sed -n '1,40p' mutants.out/missed.txt
    echo '```'
  elif (( STATUS == 0 )); then
    echo "All selected viable mutants were caught."
  else
    echo "The advisory run did not complete cleanly; inspect the job log and artifact."
  fi
} >> "$SUMMARY_FILE"

exit "$STATUS"
