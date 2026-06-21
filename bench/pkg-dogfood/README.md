<!-- [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — the PKG-query token A/B
harness. Written while Fable unavailable; flag for re-review when Fable returns. -->
# PKG-query token A/B (`sq-2m6zm.4`)

The **scientific gate** on dogfooding sparq as a project knowledge graph (epic
`sq-2m6zm`, Phases 2–4). It measures whether answering a project-knowledge lookup
via the **`pkg-query`** skill (`introspect → ground → ask`, PR #1075) costs fewer
**cache-discounted effective input tokens** than reading the source document(s) —
and whether answer **quality** holds — then emits the `§5.6` verdict object.
Protocol + kill-criteria: `research/dogfooding-sparq-knowledge-graph.md` §5.

## What it does

- **`PREREG.md`** — frozen pre-registration: hypotheses, arms, the metric, and the
  kill criteria, fixed BEFORE any run (editing it to fit a result is forbidden).
- **`tasks/tasks.jsonl`** — frozen, **stratified** task set (point-lookup /
  multi-hop / synthesis / negative) answerable BOTH ways, each with a gold key.
- **`corpus-manifest.json`** — sha256 of the pinned corpus; the driver aborts on drift.
- **`run.py`** — counterbalanced A/B driver. Arm A = realistic source read
  (AGENTS.md section / `bd show`, NOT the whole corpus) + a naive whole-doc variant;
  Arm B = `pkg-query`, **charged all its costs** (schema card + executed SPARQL +
  rows + deferred-tool-def tax + amortised ingestion).
- **`tokens.py`** — effective tokens, reusing the canonical
  `scripts/agent-telemetry/metrics_row.py::effective_input_tokens` (the §5.1 formula).
- **`grade.py`** — blind deterministic PAIRED grader (no LLM judges its own KG).
- **`stats.py`** — Wilcoxon signed-rank + bootstrap median CI + break-even, applies
  the frozen bar + kill criteria, emits `verdict.json`.

```bash
bench/pkg-dogfood/run.sh   # build → drive → grade → verdict
```

## Honesty (load-bearing)

- **Fidelity caveat.** This measures the **read-payload** effective tokens via a
  documented char/token PROXY — it is **not** the full §5 multi-session Claude Code
  A/B with `count_tokens` (the exact Claude tokenizer; `tiktoken` is wrong for
  Claude) over per-session transcripts. Neither an API key nor isolated sessions
  are available where this was written. The proxy captures the dominant delta
  driver and is conservative against the PKG; the full-session A/B is the follow-up
  (`bd` bead). See `PREREG.md` "measurement-fidelity caveat".
- **All numbers are runtime-only / NON-CANONICAL** (work-box + proxy). `run.sh`
  prints them; nothing is frozen into committed markdown (`check-no-perf-numbers.py`).
  `results.json` / `grades.json` / `verdict.json` are git-ignored — regenerate them.
- **Underpowered by construction.** The PoC task set is N=12 < the pre-registered
  ≥30, so the frozen bar CANNOT return `token_win=true`. It reports per-task deltas
  + spread + per-stratum descriptively and returns an honest `recommend_adopt`.
- **Non-sycophantic verdict.** If PKG-query does not beat the realistic baseline,
  the harness says so. The realistic Arm A (scoped read / `bd show`) is the binding
  baseline precisely so a strawman "read the whole corpus" arm cannot rig the win.

Licensed MIT (repo default).
