#!/usr/bin/env bash
# detect-tier.sh — [OPUS-4.8] Fable context-broker: post-hoc serving-tier observer.
#
# Given an agent transcript (JSONL, one event per line, carrying the served
# "model" ids), report which tier ACTUALLY served the run and whether/where a
# claude-fable-5 -> claude-opus-4-8 downgrade occurred (the empirical dual-use
# safeguard characterised in research/fable-context-broker.md). This formalises
# the manual `grep | sort | uniq -c` the orchestrator has been running by hand.
#
# Usage:   scripts/fable/detect-tier.sh <transcript.jsonl>
# Output:  per-tier tally, the dominant serving tier, and a downgrade verdict.
# Exit:    0 = clean read (regardless of tier); 2 = usage / no-model-data error.
#
# Advisory tooling only — deliberately NOT wired into the CI gate.
set -euo pipefail

FABLE_ID="claude-fable-5"
OPUS_ID="claude-opus-4-8"
MATCH="claude-(fable-5|opus-4-8)"

usage() {
  echo "usage: $0 <transcript.jsonl>" >&2
  exit 2
}

[ "$#" -eq 1 ] || usage
file="$1"
[ -f "$file" ] || { echo "error: no such file: $file" >&2; exit 2; }

# Ordered sequence of served tiers (one per model-id occurrence, in file order).
# grep -oE prints each matched id on its own line; the prefix match normalises
# dated ids (e.g. claude-fable-5-20250514 -> claude-fable-5). Collapse to a bare
# fable|opus token for the ordering pass.
mapfile -t tiers < <(grep -oE "$MATCH" "$file" \
  | sed -e "s/${FABLE_ID}/fable/" -e "s/${OPUS_ID}/opus/")

if [ "${#tiers[@]}" -eq 0 ]; then
  echo "no ${FABLE_ID} / ${OPUS_ID} model ids found in: $file" >&2
  exit 2
fi

echo "== per-tier tally ($file) =="
grep -oE "$MATCH" "$file" | sort | uniq -c

# Dominant (most-served) tier.
fable_n=0
opus_n=0
for t in "${tiers[@]}"; do
  case "$t" in
    fable) fable_n=$((fable_n + 1)) ;;
    opus)  opus_n=$((opus_n + 1)) ;;
  esac
done
if [ "$fable_n" -ge "$opus_n" ]; then
  dominant="$FABLE_ID"
else
  dominant="$OPUS_ID"
fi
echo
echo "dominant serving tier: $dominant  (fable=$fable_n opus=$opus_n)"

# Downgrade detection: the first opus occurrence that follows >=1 fable one.
seen_fable=0
downgrade_at=""
idx=0
for t in "${tiers[@]}"; do
  idx=$((idx + 1))
  if [ "$t" = "fable" ]; then
    seen_fable=1
  elif [ "$t" = "opus" ] && [ "$seen_fable" -eq 1 ] && [ -z "$downgrade_at" ]; then
    downgrade_at="$idx"
  fi
done

echo
# [FABLE] is safe ONLY when opus never served the run (opus_n == 0); any opus
# occurrence — before OR after fable — means opus served part of the run, so the
# run can never be stamped [FABLE].
if [ "$opus_n" -eq 0 ]; then
  echo "NO DOWNGRADE: run stayed on ${FABLE_ID} throughout — safe to stamp [FABLE]."
elif [ "$fable_n" -eq 0 ]; then
  echo "NO FABLE: run served entirely by ${OPUS_ID} (Fable never engaged)."
elif [ -n "$downgrade_at" ]; then
  echo "DOWNGRADE: fable -> opus at model-occurrence #${downgrade_at} of ${#tiers[@]}"
  echo "verdict: stamp this run with its ACTUAL tier (${dominant}), NEVER [FABLE]."
else
  echo "MIXED: both tiers served this run (fable=$fable_n opus=$opus_n) with no fable->opus"
  echo "downgrade transition (opus preceded fable); opus served part of the run — NEVER [FABLE]."
fi
