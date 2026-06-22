#!/usr/bin/env bash
# [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — one-command sparq-terse Phase-5 A/B.
# Written while Fable unavailable; flag for re-review when Fable returns.
# Spec: research/llm-ergonomic-sparql-surface.md §5 + bench/terse/PREREG.md.
#
# Builds the sparq-cli + the terse-expand / legend-card examples, generates the REAL per-arm context
# cards (arm A schema card, arm B/C legend card), grades every arm's reference query through the real
# transpiler + engine, accounts the cache-discounted effective input tokens (input-authoring proxy
# by default; pass --transcripts <dir> for the full-session real-token fan-out), then emits the §5.5
# PER-LEVER verdict object. Stdlib-Python + cargo only. Every number it prints is runtime-only /
# NON-CANONICAL (never frozen into committed markdown). Run from anywhere.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERE="$ROOT/bench/terse"
DATA="$ROOT/crates/sparq-kb/ingest/pkg-instances.ttl"
cd "$ROOT"
export PATH="$HOME/.cargo/bin:$PATH"

TRANSCRIPTS=""
if [[ "${1:-}" == "--transcripts" ]]; then TRANSCRIPTS="${2:?need a transcript dir}"; fi

echo "[run.sh] building sparq-cli + terse examples…"
cargo build -p sparq-cli --bin sparq-cli >/dev/null 2>&1
cargo build -p sparq-terse --features vectors --example terse-expand >/dev/null 2>&1
cargo build -p sparq-terse --example legend-card >/dev/null 2>&1
CLI="$ROOT/target/debug/sparq-cli"
EXPAND="$ROOT/target/debug/examples/terse-expand"
LEGEND="$ROOT/target/debug/examples/legend-card"

echo "[run.sh] generating the REAL per-arm context cards…"
# Arm B/C: the frozen legend card, verbatim from the crate.
"$LEGEND" > "$HERE/legend-card.txt"
# Arm A: a schema card listing the PKG prefixes + the class/property IRIs an agent needs. Built from
# the live PKG (the same vocabulary the legend abbreviates) so the comparison is apples-to-apples.
python3 "$HERE/make_schema_card.py" --data "$DATA" --cli "$CLI" --out "$HERE/schema-card.txt"

echo "[run.sh] grading quality (real transpiler + real engine, blind)…"
python3 "$HERE/grade.py" --data "$DATA" --tasks "$HERE/tasks/terse_tasks.json" \
  --refs "$HERE/tasks/reference_queries.json" --cli "$CLI" --expand "$EXPAND" \
  --out "$HERE/grades.json"

echo "[run.sh] accounting effective input tokens…"
if [[ -n "$TRANSCRIPTS" ]]; then
  python3 "$HERE/tokens.py" --transcripts "$TRANSCRIPTS" --out "$HERE/tokens.json"
else
  python3 "$HERE/tokens.py" --refs "$HERE/tasks/reference_queries.json" \
    --tasks "$HERE/tasks/terse_tasks.json" \
    --schema-card "$HERE/schema-card.txt" --legend-card "$HERE/legend-card.txt" \
    --out "$HERE/tokens.json"
fi

echo "[run.sh] verdict (Wilcoxon + bootstrap + cache-discount survival, frozen PREREG bar)…"
python3 "$HERE/verdict.py" --tokens "$HERE/tokens.json" --grades "$HERE/grades.json" \
  --tasks "$HERE/tasks/terse_tasks.json" --out "$HERE/verdict.json"

echo "[run.sh] done. Artifacts: grades.json, tokens.json, verdict.json (all NON-CANONICAL)."
