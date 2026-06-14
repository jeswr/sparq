#!/usr/bin/env bash
# [OPUS-4.8] BSBM Explore-mix per-commit runner. Self-contained: ensures the fixed -pc 300
# corpus exists (gen.sh, fetch+cache), runs the instantiated Explore queries through the
# `sparq-cli bench` harness in `materialize` mode (the mix includes a CONSTRUCT (query12) and a
# DESCRIBE (query09) — graph-valued forms report produced-triple counts), then diffs every
# query's result size against bench/bsbm/expected-rows.tsv. Exits 1 on any mismatch / error.
# This is what CI calls. Scratch lives in /tmp (gitignored, regenerable).
#
#   bench/bsbm/run.sh                 # build sparq-cli first: cargo build --release -p sparq-cli
#   BSBM_PC=300 ITERS=3 bench/bsbm/run.sh
set -euo pipefail
cd "$(dirname "$0")/../.."   # repo root

PC="${BSBM_PC:-300}"
ITERS="${ITERS:-3}"
CLI="${SPARQ_CLI:-target/release/sparq-cli}"
GEN=bench/bsbm/gen.sh
QDIR=bench/bsbm/queries
EXP=bench/bsbm/expected-rows.tsv

[ -x "$CLI" ] || { echo "ERROR: $CLI not found — run: cargo build --release -p sparq-cli" >&2; exit 2; }

echo "[bsbm] free space on /tmp before gen:" >&2; df -h /tmp 2>/dev/null | tail -1 >&2 || true

CORPUS="$(bash "$GEN" "$PC")"
[ -s "$CORPUS" ] || { echo "ERROR: corpus generation failed (empty $CORPUS)" >&2; exit 1; }
echo "[bsbm] corpus: $CORPUS ($(wc -l < "$CORPUS") triples)" >&2

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$CLI" bench "$CORPUS" ntriples "$QDIR" "$ITERS" materialize | tee "$TMP/out.tsv"

# Differential: every query's result size must equal the committed expected size.
fail=0
while IFS=$'\t' read -r q exp; do
  case "$q" in \#*|"") continue;; esac
  got="$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$TMP/out.tsv")"
  if [ "${got:-MISSING}" != "$exp" ]; then
    echo "ERROR: bsbm $q rows=${got:-MISSING} expected=$exp (correctness regression)" >&2
    fail=1
  fi
done < "$EXP"

# Also fail if any query errored (rows column == ERROR).
if awk -F'\t' '$2=="ERROR"{exit 0} END{exit 1}' "$TMP/out.tsv"; then
  echo "ERROR: one or more bsbm queries returned ERROR" >&2
  awk -F'\t' '$2=="ERROR"{print "  "$0}' "$TMP/out.tsv" >&2
  fail=1
fi

echo "[bsbm] free space on /tmp after run:" >&2; df -h /tmp 2>/dev/null | tail -1 >&2 || true
[ "$fail" = 0 ] && echo "[bsbm] OK — all $(grep -cvE '^#|^$' "$EXP") Explore queries match expected sizes." >&2
exit "$fail"
