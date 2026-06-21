<!-- [OPUS-4.8] sq-2m6zm.4 (epic sq-2m6zm). 🤖 SPARQ agent — the cache-discounted
token A/B PRE-REGISTRATION. Written while Fable unavailable; flag for re-review when
Fable returns. Design record: research/dogfooding-sparq-knowledge-graph.md §5. -->

# PRE-REGISTRATION — PKG-query token A/B (sq-2m6zm.4)

This fixture **fixes the hypotheses, the arms, the metric, and the kill criteria
BEFORE any measurement is run**, per `research/dogfooding-sparq-knowledge-graph.md`
§5.1 ("declare BEFORE running"). Editing this file after a run, to fit the result,
is forbidden — the whole point of the pre-registration is that it is frozen.

> 🤖 **SPARQ agent.** This is the SCIENTIFIC GATE on dogfooding Phases 2–4. The
> verdict it produces decides whether the PKG-query approach (`introspect → ground
> → ask` via `pkg-query`) is adopted over plain `Read`/`Grep`/`bd` of the source
> documents. The verdict must be non-sycophantic: if PKG-query does NOT beat the
> status quo, that is the finding to report.

## Hypothesis

H1 (token): answering a project-knowledge lookup via `pkg-query` costs FEWER
cache-discounted *effective input tokens* than answering it by reading the source
document(s) — measured PAIRED, per task.

H2 (quality): the PKG-query answer is at least as CORRECT as the read-the-docs
answer (no accuracy regression, no increase in unresolvable/hallucinated claims).

## Arms (counterbalanced within task)

- **Arm A — read-the-docs (baseline):** answer using only the source document(s)
  read into context: `AGENTS.md` sections for Findings, `bd show` / `bd dep` /
  `.beads/issues.jsonl` for Tasks. Two A-variants are charged and BOTH reported:
  - `A_realistic` — the **cheapest plausible competent read**: the relevant
    AGENTS.md section (not the whole file), or `bd show <id>` (not the 1.6 MB
    jsonl). This is the HONEST baseline — a strawman "read the whole corpus" arm
    A would rig the comparison toward the PKG.
  - `A_naive` — the worst-case whole-document read (whole `AGENTS.md`, etc.). The
    PKG only "wins" against a naive agent if it loses against a competent one, so
    A_realistic is the binding baseline; A_naive is reported for context only.
- **Arm B — query-the-PKG:** answer via `pkg-query` (the §6.4 / PR #1075 helper).
  Charged ALL of arm B's costs (§5.1 "charge arm B all its costs"):
  - the one-time **schema card** (introspect: `schema-classes` + `schema-properties`)
    that the agent reads once before grounding — billed ONCE, then amortised over
    the task set (`schema_card / N`), because it persists per the design's
    "mine once, summarise forever";
  - the **executed SPARQL** the helper prints (the design's "surface the SPARQL"
    rule — the agent must read it to verify);
  - the **answer rows** the helper returns;
  - the **deferred-tool-definition tax**: the `query-pkg` SKILL description that
    must sit in the prompt for the agent to know the tool exists. Charged once,
    amortised (`tool_def / N`) because it is deferred-loaded behind the cache
    breakpoint (§5.1 "the deferred-tool-definition tokens actually pulled in").
  - the **amortised one-time ingestion** slice `(ingest_build) / N` (§5.1).

## Metric — cache-discounted effective input tokens (§5.1)

```
effective_input = 1.0*fresh + 0.1*cache_read + 1.25*cache_write
```

reusing `scripts/agent-telemetry/metrics_row.py::effective_input_tokens` — the
canonical implementation. The paired statistic is the per-task delta
`A_realistic − B` (positive = PKG cheaper).

### HONEST measurement-fidelity caveat (load-bearing — read this)

The §5 GOLD standard runs each arm as a **separate Claude Code session** and diffs
two `agent_telemetry.py` transcript reports, with model completions pinned via
record/replay. That requires orchestrating ≥8 isolated sessions and an
`ANTHROPIC_API_KEY` for `count_tokens` (the only exact Claude tokenizer; `tiktoken`
is wrong for Claude). **Neither is available in this execution environment** (no
key; one session). So this harness measures the **READ-PAYLOAD effective tokens**:
the input bytes each arm forces into context, converted to a token estimate by a
**documented char/token proxy**, fed through the canonical effective-token formula.

This captures the DOMINANT driver of the delta (§1.1: "the saving scales with
`corpus_size / answer_size`") but it is NOT the full-session A/B. It deliberately
EXCLUDES the model round-trip / repair-loop output tokens and the per-turn cached-
prefix dynamics — those need the multi-session harness. The numbers are therefore
a **lower-fidelity proxy**, runtime-only, NON-CANONICAL, and never frozen into
committed markdown (`check-no-perf-numbers.py`). What is committed is this protocol
+ the code + the verdict schema. The full-session A/B is tracked as a follow-up.

The char/token proxy is **conservative against the PKG** by construction: it
charges arm B every byte it prints (schema card, SPARQL, rows) at the same rate as
arm A's prose, and amortises arm B's one-time costs only over the small frozen N
(a larger real task corpus would amortise them further, helping B). If the proxy
shows B losing, the real tokenizer would not rescue it.

## Pre-registered significance bar (frozen — §5.1)

A token WIN (`token_win = true`) requires the paired-median effective-input
reduction of `A_realistic − B` to exceed **BOTH**:

1. **≥ 20 % relative reduction** of the paired median (below ~20 % is inside the
   noise of the one independent 15–28 % memory-saving benchmark — not worth the
   maintenance), **AND**
2. **p < 0.05** via the Wilcoxon signed-rank test over **≥ 30 tasks**, with a
   bootstrap 95 % CI on the median delta whose **lower bound is also > 0**.

> **N caveat, declared up front.** The frozen task set in this PoC is **N = 12**
> (stratified, see below), NOT ≥ 30. With N < 30 the §5.1 significance bar
> **cannot be met by construction** — so this run **cannot return
> `token_win = true`** under the frozen rule, regardless of the deltas. It can only
> (a) report the per-task deltas + spread descriptively, and (b) return
> `token_win = false` with an honest "underpowered — needs ≥30 tasks + full-session
> harness" note. This is intentional: it is more honest to under-claim from a small
> PoC than to lower the bar to manufacture a positive. Scaling to ≥30 tasks + the
> full-session harness is the follow-up.

## Stratification (§5.5 anti-selection-bias)

The task set spans **≥ 4 strata** so a point-lookup-heavy set does not trivially
favour the KG. Each stratum is reported separately:

- `point-lookup` — one fact from one place ("status of bead X", "what does the
  merge-discipline finding say").
- `multi-hop` — a join/traversal ("what does bead X depend on, and is each done").
- `synthesis` — the union of several facts ("all the sub-agent standing rules").
- `negative` — a deliberately out-of-KG / empty-result question (tests honest
  abstention; the PKG returns 0 rows, the docs require a read to confirm absence).

## Kill criteria (frozen — §5.4)

- **KILL 1 (token):** paired-median effective reduction `< 20 %`, OR Wilcoxon
  `p ≥ 0.05`, OR the bootstrap median-delta CI includes 0, OR the saving is entirely
  the cache-discount component. → do not adopt for token reasons.
- **KILL 2 (break-even):** `N*` infinite (`C_docread − C_query ≤ 0`), OR `N*`
  exceeds the realistic recurrence of a question class before the docs change. →
  net loss; kill.
- **KILL 3 (quality):** PKG arm increases unresolvable-claim rate, OR drops paired
  answer correctness below the docs arm. → kill.

## Verdict object (§5.6)

`recommend_adopt = true` requires `token_win` **AND** quality non-regression
**AND** a finite `break_even_N` within a realistic horizon. The decision is made on
the verdict OBJECT, never on a single number.
