#!/usr/bin/env bash
# detect-tier.sh — [OPUS-4.8] Fable context-broker: post-hoc serving-tier observer.
# Updated for the Opus 5 rollout (maintainer directive 2026-07-24): claude-opus-5
# is the PRIMARY top-tier model, replacing both claude-fable-5 and claude-opus-4-8.
#
# Given an agent transcript (JSONL, one event per line, carrying the served
# "model" ids), report which tier ACTUALLY served the run and whether/where a
# claude-opus-5 -> (claude-fable-5 | claude-opus-4-8) downgrade occurred (the
# empirical dual-use safeguard characterised in research/fable-context-broker.md,
# originally for the fable->opus pair). This formalises the manual
# `grep | sort | uniq -c` the orchestrator has been running by hand.
#
# Usage:   scripts/fable/detect-tier.sh <transcript.jsonl>
# Output:  per-tier tally, the dominant serving tier, and a downgrade verdict.
# Exit:    0 = clean read (regardless of tier); 2 = usage / no-model-data error.
#
# Advisory tooling only — deliberately NOT wired into the CI gate.
set -euo pipefail

PRIMARY_ID="claude-opus-5"
FABLE_ID="claude-fable-5"
OPUS48_ID="claude-opus-4-8"
MATCH="claude-(opus-5|fable-5|opus-4-8)"

usage() {
  echo "usage: $0 <transcript.jsonl>" >&2
  exit 2
}

[ "$#" -eq 1 ] || usage
file="$1"
[ -f "$file" ] || { echo "error: no such file: $file" >&2; exit 2; }

# Ordered sequence of served tiers (one per model-id occurrence, in file order).
# grep -oE prints each matched id on its own line; the prefix match normalises
# dated ids (e.g. claude-opus-5-20260701 -> claude-opus-5). Collapse to a bare
# primary|downgrade token for the ordering pass (both claude-fable-5 and
# claude-opus-4-8 are downgrade tiers under the Opus 5 primary).
mapfile -t tiers < <(grep -oE "$MATCH" "$file" \
  | sed -e "s/${PRIMARY_ID}/primary/" -e "s/${FABLE_ID}/downgrade/" -e "s/${OPUS48_ID}/downgrade/")

if [ "${#tiers[@]}" -eq 0 ]; then
  echo "no ${PRIMARY_ID} / ${FABLE_ID} / ${OPUS48_ID} model ids found in: $file" >&2
  exit 2
fi

echo "== per-tier tally ($file) =="
grep -oE "$MATCH" "$file" | sort | uniq -c

# Dominant (most-served) tier bucket.
primary_n=0
downgrade_n=0
for t in "${tiers[@]}"; do
  case "$t" in
    primary)   primary_n=$((primary_n + 1)) ;;
    downgrade) downgrade_n=$((downgrade_n + 1)) ;;
  esac
done
if [ "$primary_n" -ge "$downgrade_n" ]; then
  dominant="$PRIMARY_ID"
else
  dominant="downgrade (${FABLE_ID}/${OPUS48_ID} — see tally)"
fi
echo
echo "dominant serving tier: $dominant  (primary=$primary_n downgrade=$downgrade_n)"

# Downgrade detection: the first downgrade occurrence that follows >=1 primary one.
seen_primary=0
downgrade_at=""
idx=0
for t in "${tiers[@]}"; do
  idx=$((idx + 1))
  if [ "$t" = "primary" ]; then
    seen_primary=1
  elif [ "$t" = "downgrade" ] && [ "$seen_primary" -eq 1 ] && [ -z "$downgrade_at" ]; then
    downgrade_at="$idx"
  fi
done

echo
# [OPUS-5] is safe ONLY when no downgrade model ever served the run
# (downgrade_n == 0); any downgrade occurrence — before OR after the primary —
# means a downgrade model served part of the run, so the run can never be
# stamped [OPUS-5]. Stamp the ACTUAL serving model(s) and flag the run for
# re-review under Opus 5 (AGENTS.md § Model provenance).
if [ "$downgrade_n" -eq 0 ]; then
  echo "NO DOWNGRADE: run stayed on ${PRIMARY_ID} throughout — safe to stamp [OPUS-5]."
elif [ "$primary_n" -eq 0 ]; then
  echo "NO PRIMARY: run served entirely by downgrade tiers (${PRIMARY_ID} never engaged);"
  echo "stamp the ACTUAL serving model(s) ([FABLE-5]/[OPUS-4.8]) and flag for re-review under Opus 5."
elif [ -n "$downgrade_at" ]; then
  echo "DOWNGRADE: primary -> downgrade at model-occurrence #${downgrade_at} of ${#tiers[@]}"
  echo "verdict: stamp this run with its ACTUAL tier(s) (see tally), NEVER [OPUS-5];"
  echo "flag downgrade-authored spans for re-review under Opus 5."
else
  echo "MIXED: both buckets served this run (primary=$primary_n downgrade=$downgrade_n) with no"
  echo "primary->downgrade transition (a downgrade tier preceded the primary); a downgrade model"
  echo "served part of the run — NEVER [OPUS-5]; flag those spans for re-review under Opus 5."
fi
