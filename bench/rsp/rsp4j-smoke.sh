#!/usr/bin/env bash
# [FABLE-5] (sq-hmd7l.20) Bounded count-matched-replay protocol SMOKE — fast, no JVM.
#
# What it proves (the sq-hmd7l.20 acceptance):
#   1. the comparator unit tests pass, INCLUDING the fidelity guard that re-derives
#      every scenario's per-window counts from the pinned replay exports and asserts
#      them against bench/rsp/expected.tsv (the chain to the in-code oracle script),
#   2. the replay-adapter path count-matches one scenario (srbench_join)
#      window-for-window against the oracle: an independently derived competitor
#      count stream flows through the REAL count-match gate + envelope emitter,
#   3. the gate INVARIANT holds in the negative direction: a corrupted count stream
#      is refused (exit 3, verdict NOT-COUNT-MATCHED, zero timing rows).
#
# The REAL-engine leg (RSP4J/YASPER driven with the same replay) is gather-time:
# bench/rsp/gather-rsp4j.sh. This smoke needs only python3 + the committed files.
#
# Usage: bench/rsp/rsp4j-smoke.sh   (run from anywhere)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "[rsp4j-smoke] 1/3 unit tests + replay-fidelity guard" >&2
python3 "$HERE/test_rsp4j_compare.py" >/dev/null

echo "[rsp4j-smoke] 2/3 end-to-end count-match (srbench_join, window-for-window)" >&2
python3 "$HERE/rsp4j_compare.py" ref-counts \
  --replay "$HERE/replay/srbench.ts.tsv" --scenario srbench_join \
  --raw-reports > "$TMP/competitor.tsv"
python3 "$HERE/rsp4j_compare.py" count-match \
  --scenario srbench_join \
  --competitor "$TMP/competitor.tsv" \
  --replay "$HERE/replay/srbench.ts.tsv" \
  --out "$TMP/envelope.json"
python3 - "$TMP/envelope.json" <<'EOF'
import json, sys
e = json.load(open(sys.argv[1]))
assert e["verdict"] == "COUNT-MATCHED", e["verdict"]
assert e["count_match"]["matched"] == 4 and e["count_match"]["mismatched"] == 0
assert e["time_model_caveat"] is True
EOF

echo "[rsp4j-smoke] 3/3 negative path: corrupted counts are refused" >&2
sed 's/^report\t10001\t3$/report\t10001\t4/' "$TMP/competitor.tsv" > "$TMP/corrupted.tsv"
if cmp -s "$TMP/competitor.tsv" "$TMP/corrupted.tsv"; then
  echo "[rsp4j-smoke] ERROR: corruption did not apply (smoke would be vacuous)" >&2
  exit 1
fi
if python3 "$HERE/rsp4j_compare.py" count-match \
    --scenario srbench_join \
    --competitor "$TMP/corrupted.tsv" \
    --replay "$HERE/replay/srbench.ts.tsv" \
    --out "$TMP/bad-envelope.json" 2>/dev/null; then
  echo "[rsp4j-smoke] ERROR: gate accepted a corrupted count stream" >&2
  exit 1
fi
python3 - "$TMP/bad-envelope.json" <<'EOF'
import json, sys
e = json.load(open(sys.argv[1]))
assert e["verdict"] == "NOT-COUNT-MATCHED", e["verdict"]
assert e["rows"] == [], "timing rows admitted past a failed gate"
EOF

echo "[rsp4j-smoke] OK: srbench_join count-matched window-for-window; corrupted stream refused" >&2
