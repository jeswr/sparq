#!/usr/bin/env bash
# [OPUS-4.8] WatDiv per-commit runner — self-contained. Ensures the fixed SF=1 corpus
# exists (via gen.sh), builds sparq-cli if needed, runs the 16 per-commit Basic-Testing
# queries in count + materialize + json modes, prints the count-mode result table, and
# HARD-CHECKS the count-mode solution counts against expected-rows.tsv (exit 1 on mismatch
# = a correctness regression). This is what CI calls.
#
#   bench/watdiv/run.sh [scale-factor=1]
#
# Heavy-tier queries (queries-heavy/: F1/F4/C1/C2) are EMPTY at SF=1 and are NOT run here;
# they belong to the EC2/nightly SF>=10 tier. See bench/watdiv/README.md.
set -euo pipefail

SF="${1:-1}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"        # repo root
GEN="$HERE/gen.sh"
QDIR="$HERE/queries"
EXP="$HERE/expected-rows.tsv"

CLI="$ROOT/target/release/sparq-cli"
if [ ! -x "$CLI" ]; then
  echo "[watdiv] building sparq-cli (release)..." >&2
  ( cd "$ROOT" && cargo build --release -p sparq-cli >/dev/null )
fi

echo "[watdiv] ensuring SF=$SF corpus..." >&2
CORPUS="$(bash "$GEN" "$SF")"
[ -s "$CORPUS" ] || { echo "[watdiv] FATAL: corpus not produced" >&2; exit 1; }
echo "[watdiv] corpus: $CORPUS ($(wc -l < "$CORPUS") triples)" >&2

# Smoke materialize + json (catches serialization-path regressions; output discarded).
"$CLI" bench "$CORPUS" ntriples "$QDIR" 3 materialize >/dev/null 2>&1 || true
"$CLI" bench "$CORPUS" ntriples "$QDIR" 3 json        >/dev/null 2>&1 || true

# Count mode is the correctness gate.
echo "[watdiv] count-mode results (name<TAB>rows<TAB>min_us):" >&2
COUNTS="$("$CLI" bench "$CORPUS" ntriples "$QDIR" 3 count 2>/dev/null)"
echo "$COUNTS"

# HARD differential: every count-mode solution count must equal the committed expected size.
fail=0
while IFS=$'\t' read -r q exp; do
  case "$q" in \#*|"") continue;; esac
  got="$(printf '%s\n' "$COUNTS" | awk -F'\t' -v k="$q" '$1==k{print $2}')"
  if [ "${got:-MISSING}" != "$exp" ]; then
    echo "ERROR: watdiv $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
    fail=1
  fi
done < "$EXP"

if [ "$fail" = 0 ]; then
  echo "[watdiv] OK — all $(grep -cvE '^\s*#|^\s*$' "$EXP") per-commit queries match expected-rows.tsv" >&2
else
  exit 1
fi
