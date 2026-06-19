#!/usr/bin/env bash
# [OPUS-4.8] (sq-k0km) sparq-nlq OFFLINE NL->SPARQL per-commit runner — the self-asserting
# entry point CI calls, mirroring bench/solid/run.sh + bench/hdt/run.sh + bench/rsp/run.sh.
# Self-contained + FULLY OFFLINE (no network, no `live` feature, no Anthropic round-trip):
#   1. run the sparq-nlq `bench` EXAMPLE at a PINNED entity count (the example takes N as its
#      first arg; pin it so the counts are comparable across commits). The LLM is a tiny
#      in-example fixed-completion stub, so the example measures the crate's DETERMINISTIC
#      core — schema grounding (sparq-introspect scan + token-budgeted summary render),
#      prompt construction, and the extract->spargebra-validate->budgeted-execute loop — on a
#      synthetic typed graph generated in-example from a fixed shape (NO external dataset).
#   2. PARSE the example's human-readable stdout into the DETERMINISTIC STRUCTURAL COUNTS
#      (synthetic triple count, grounded-prompt char length, ask result-row count, ask repair
#      rounds — all a pure function of the FIXED N + synthetic schema, so byte-stable across
#      machines: NOT wall-clock) and ASSERT each vs expected.tsv (exit 1 on any drift — the
#      HARD correctness gate). The example does NOT emit a TSV itself, so the parse lives here
#      (no crate change needed). The us timings the example also prints are NON-CANONICAL
#      (dev/gh-runner box) and are NOT harvested.
#   3. forward on STDOUT the 3-column `<metric>\t<value>\t<unit>` contract the ci-bench `nlq`
#      hook consumes. The prompt-char length is a real token-budget proxy (smaller = leaner
#      grounded prompt); the rest are structural counts. Correctness lives in expected.tsv,
#      exactly like LUBM / FTS / HDT / RSP / Solid.
#
# These light up the dashboard's GenAI family row (familyOf: `nlq_*` -> genai). HONESTY: this
# is the OFFLINE deterministic core only; LIVE NL->SPARQL exec-accuracy on a canonical host is
# a SEPARATE concern (sq-qidj / sq-g0lw, EC2-blocked) and is NOT claimed here. The sibling
# GenAI suites sim-olympics-eval + introspect-olympics need the gitignored 1.78M-triple
# olympics.nt dataset (NOT in CI) and so are NOT wired here — they are EC2/dataset-gathered.
# This is a TREND/coverage feed, NOT a head-to-head competitor card (featured = false).
#
# Usage: bench/nlq/run.sh   (run from anywhere; honours $NLQ_BIN / $NLQ_N overrides)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
EXP="$ROOT/bench/nlq/expected.tsv"
# Pinned entity count — the counts (triples = 3*N, prompt chars, rows) are a function of N,
# so it MUST match the value baked into expected.tsv. Override with $NLQ_N only when also
# re-deriving expected.tsv.
N="${NLQ_N:-2000}"
# The runner: a sparq-nlq example (the crate is isolated — not a sparq-cli dependency).
# Override with $NLQ_BIN for a custom build. NB the example's target name is `bench`, which
# collides with other crates' `--example bench` binaries at target/release/examples/bench;
# the ci-bench hook builds THIS crate's example immediately before invoking run.sh.
BIN="${NLQ_BIN:-$ROOT/target/release/examples/bench}"

if [ ! -x "$BIN" ]; then
  echo "[nlq] ERROR: sparq-nlq bench example not found at $BIN" >&2
  echo "[nlq]   build: cargo build --release -p sparq-nlq --example bench" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# ---- 1. run the example OFFLINE at the pinned N (human-readable stdout) --------------------
"$BIN" "$N" > "$TMP/nlq.out" 2>"$TMP/nlq.err" || {
  echo "[nlq] ERROR: sparq-nlq bench example exited non-zero" >&2
  cat "$TMP/nlq.err" >&2 || true
  exit 1
}

# ---- 2. extract the DETERMINISTIC counts (pure function of N + the synthetic schema) -------
emit() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$TMP/nlq.tsv"; }
: > "$TMP/nlq.tsv"

# "synthetic graph: <N> entities, <T> triples"
triples=$(grep -oE 'synthetic graph: [0-9]+ entities, [0-9]+ triples' "$TMP/nlq.out" | grep -oE '[0-9]+ triples' | grep -oE '[0-9]+' | head -1)
# "  prompt: <C> chars"
prompt_chars=$(grep -oE 'prompt: [0-9]+ chars' "$TMP/nlq.out" | grep -oE '[0-9]+' | head -1)
# "  answered in <R> repair round(s); result rows = <X>"
repairs=$(grep -oE 'answered in [0-9]+ repair round' "$TMP/nlq.out" | grep -oE '[0-9]+' | head -1)
result_rows=$(grep -oE 'result rows = [0-9]+' "$TMP/nlq.out" | grep -oE '[0-9]+' | head -1)

[ -n "$triples" ]      && emit nlq_synth_triples   "$triples"      count
[ -n "$prompt_chars" ] && emit nlq_prompt_chars    "$prompt_chars" chars
[ -n "$repairs" ]      && emit nlq_ask_repairs     "$repairs"      count
[ -n "$result_rows" ]  && emit nlq_ask_result_rows "$result_rows"  count

# ---- 3. assert vs expected.tsv (exit 1 on any drift) + forward the hook contract ----------
fail=0
seen=0
while IFS=$'\t' read -r metric value unit; do
  [ -n "$metric" ] || continue
  exp="$(awk -F'\t' -v k="$metric" '$1==k{print $2}' "$EXP")"
  if [ -z "$exp" ]; then
    echo "[nlq] ERROR: $metric has no expected.tsv entry" >&2; fail=1; continue
  fi
  if [ "$value" != "$exp" ]; then
    echo "[nlq] ERROR: $metric=$value expected=$exp (NLQ offline-loop count regression)" >&2; fail=1
  fi
  seen=$((seen+1))
  printf '%s\t%s\t%s\n' "$metric" "$value" "$unit"
done < "$TMP/nlq.tsv"

# Every expected metric must have been emitted (a vanished count = a parse break or regression).
exp_rows=$(grep -cvE '^#|^$' "$EXP")
if [ "$seen" != "$exp_rows" ]; then
  echo "[nlq] ERROR: emitted $seen metrics, expected $exp_rows (a metric vanished — parse break or schema change)" >&2; fail=1
fi

if [ "$fail" = 0 ]; then
  echo "[nlq] OK: all $seen offline NL->SPARQL counts match expected.tsv (triples/prompt-chars/rows/repairs at N=$N)" >&2
else
  echo "[nlq] FAILED: one or more offline-loop counts diverged from expected.tsv" >&2
fi
exit "$fail"
