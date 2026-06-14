#!/usr/bin/env bash
# [OPUS-4.8] LUBM per-commit runner — the entry point CI calls. Self-contained:
#   1. ensure the LUBM(1) corpus + Univ-Bench ontology exist (gen.sh).
#   2. EXTENSIONAL tier: run queries-extensional/ directly against the RAW ABox.
#   3. ENTAILED tier: materialize the OWL-RL closure of (ABox + TBox) with `sparq-cli reason`,
#      then run queries-entailed/ against that closure.
#   4. assert both tiers' solution counts against expected-rows.tsv (exit 1 on any mismatch).
#
# The split is the point of the suite: the entailed queries return 0 on the raw data and their
# correct counts only after reasoning (see bench/lubm/README.md + expected-rows.tsv).
#
# Usage: bench/lubm/run.sh   (run from the repo root; honours $CLI / $ITERS overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
CLI="${CLI:-$ROOT/target/release/sparq-cli}"
ITERS="${ITERS:-3}"
GEN="$ROOT/bench/lubm/gen.sh"
Q_EXT="$ROOT/bench/lubm/queries-extensional"
Q_ENT="$ROOT/bench/lubm/queries-entailed"
EXP="$ROOT/bench/lubm/expected-rows.tsv"
CACHE="${LUBM_CACHE:-/tmp/lubm}"

if [ ! -x "$CLI" ]; then
  echo "[lubm] ERROR: sparq-cli not found at $CLI (build: cargo build --release -p sparq-cli)" >&2
  exit 1
fi

# ---- 1. ensure data (gen.sh emits two lines: data.nt then ontology.nt) -------------------
mapfile -t ARTS < <("$GEN" 1 0)
DATA="${ARTS[0]}"
ONTO="${ARTS[1]}"
echo "[lubm] data=$DATA  ontology=$ONTO" >&2

# ---- 2. materialize the OWL-RL closure of (ABox + TBox) for the entailed tier ------------
COMBINED="$CACHE/lubm-with-onto.nt"
CLOSURE="$CACHE/lubm-owl-closure.nt"
cat "$DATA" "$ONTO" > "$COMBINED"
# Exact invocation: `sparq-cli reason <data> <format> owl <out.nt>` writes the full OWL-RL
# closure (ABox + TBox + all entailed triples) as N-Triples. (owl == owl-rl; see main.rs.)
echo "[lubm] materializing OWL-RL closure ..." >&2
"$CLI" reason "$COMBINED" ntriples owl "$CLOSURE" 2>&1 | sed 's/^/[lubm] /' >&2

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- 3. run both tiers (count mode; ITERS iters) -----------------------------------------
echo "[lubm] === extensional tier (raw ABox) ===" >&2
"$CLI" bench "$DATA"    ntriples "$Q_EXT" "$ITERS" count > "$TMP/ext.tsv"
echo "[lubm] === entailed tier (OWL-RL closure) ===" >&2
"$CLI" bench "$CLOSURE" ntriples "$Q_ENT" "$ITERS" count > "$TMP/ent.tsv"
cat "$TMP/ext.tsv" "$TMP/ent.tsv"

# ---- 4. assert solution counts vs expected-rows.tsv --------------------------------------
fail=0
check() {  # $1 = results tsv  $2 = regime label
  while IFS=$'\t' read -r q rows _us; do
    [ -n "$q" ] || continue
    exp="$(awk -F'\t' -v k="$q" '$1==k{print $2}' "$EXP")"
    if [ -z "$exp" ]; then
      echo "[lubm] ERROR: $q ($2) has no expected-rows entry" >&2; fail=1; continue
    fi
    if [ "$rows" != "$exp" ]; then
      echo "[lubm] ERROR: $q ($2) rows=$rows expected=$exp (correctness regression)" >&2; fail=1
    fi
  done < "$1"
}
check "$TMP/ext.tsv" extensional
check "$TMP/ent.tsv" entailed

if [ "$fail" = 0 ]; then
  n=$(( $(wc -l < "$TMP/ext.tsv") + $(wc -l < "$TMP/ent.tsv") ))
  echo "[lubm] OK: all $n query counts match expected-rows.tsv" >&2
else
  echo "[lubm] FAILED: one or more counts diverged from expected-rows.tsv" >&2
fi
exit "$fail"
