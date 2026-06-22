<!-- [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — the sparq-terse Phase-5
A/B PRE-REGISTRATION. Written while Fable unavailable; flag for re-review when Fable
returns. Design record: research/llm-ergonomic-sparql-surface.md §5; phase plan §8.6. -->

# PRE-REGISTRATION — sparq-terse Phase-5 query-authoring A/B (sq-bzign)

This fixture **fixes the hypotheses, the arms, the metric, the quality scoring and the
kill criteria BEFORE any measurement is run**, per
`research/llm-ergonomic-sparql-surface.md` §5.4 ("declare BEFORE running"). Editing it
after a run, to fit the result, is forbidden — the point of pre-registration is that it
is frozen.

> 🤖 **SPARQ agent.** This is the ADOPTION GATE on the `sparq-terse` surface (the
> `K:<name>` keyword layer + the `V("phrase")` concept resolution). The verdict it
> produces decides whether either lever is adopted over plain SPARQL. The verdict must
> be **non-sycophantic**: if the terse surface does NOT save tokens, that is the finding
> to report. (Two proxy-based verdicts were reversed by real measurement this project —
> see `bench/EXPERIMENTS.md` — so the harness trusts the real numbers, not intuition.)

## What is measured (a query-AUTHORING task, not a doc-read task)

Each task gives an agent a natural-language question + a per-arm **context card**, and
asks it to author **one** SPARQL query over the *same* ingested PKG
(`crates/sparq-kb/ingest/pkg-instances.ttl`). The arms differ in the card carried and
the dialect written:

- **Arm A — plain SPARQL (baseline):** carries the **schema card** (PKG prefixes +
  class/property IRIs) and writes raw SPARQL with `PREFIX` lines and full prefixed names.
- **Arm B — terse keyword (lever 1):** carries the **legend card** (`legend_card()`,
  the frozen `K:<name>` legend) and writes `K:<name>` terse SPARQL.
- **Arm C — terse + `V()` (lever 3):** as B, plus may write `V("phrase")` for concept
  resolution (lexical-first, the no-model deterministic path).

## Hypotheses (frozen)

- **H1 (token):** authoring a PKG query in the terse dialect (arm B / C) costs FEWER
  cache-discounted *effective input tokens* than authoring it as plain SPARQL (arm A) —
  measured PAIRED, per task. The cost charged to each arm is the **context card it
  carries** (amortised over the task set, since the card sits behind the prompt-cache
  breakpoint) **plus** the **authored-query** tokens.
- **H2 (quality):** the terse-authored query is at least as CORRECT as the
  plain-SPARQL query: equal `parses` / `grounded` / `answer_correct` (answer-set F1 vs
  the gold query on the pinned PKG), and — for arm C — every `V()` binds the **gold**
  IRI (`resolution_correctness`, the silent-drift detector).

## Metric — cache-discounted effective input tokens (§5.1)

```text
effective_input = 1.0*fresh + 0.1*cache_read + 1.25*cache_write
```

(the canonical `scripts/agent-telemetry/metrics_row.py::effective_input_tokens`
multipliers). The paired statistic is the per-task delta `A − B` and `A − C` (positive
= terse cheaper).

The card is charged at its **true cache multiplier**: it sits behind the prompt-cache
breakpoint (the §1.6 "the win is a caching property" rule), so per task it is billed
once at `cache_read` (×0.1) and amortised — the harness reports the win **with and
without** the card so a "win that is purely the cache discount, or that ignores the
card tax" is flagged invalid (§5.3 / KILL-token).

### HONEST measurement-fidelity caveat (load-bearing — read this)

The §5 GOLD standard fans out one fresh Claude Code sub-agent per `(task, arm)`,
tagged `[TERSE task=<id> arm=<A|B|C>]`, and mines the REAL cache-discounted effective
input tokens straight from each transcript's `message.usage` (the
`bench/pkg-dogfood/tokens_real.py` mechanism — no `count_tokens` API, no proxy). The
harness `tokens.py` **is wired to consume those transcripts** (`--transcripts <dir>`):
when a fan-out exists, the verdict is computed on real tokens.

**This execution environment has no live sub-agent-dispatch tool and one session** — so
this run cannot fan out 90 sub-agents. It therefore measures the **deterministic,
model-independent component** the design itself names as the DOMINANT lever (§1.6, §7):
the **input-side authoring cost** — the per-arm context card + the authored query —
counted by a **documented char→token proxy** fed through the canonical effective-token
formula, plus the **deterministic quality** (the real transpiler + the real engine
grade every arm's reference query). This is **runtime-only, NON-CANONICAL, never frozen
into committed markdown** (`check-no-perf-numbers.py`); what is committed is this
protocol + the harness + the verdict schema. The full-session token A/B is a tracked
follow-up bead.

The proxy is **conservative toward the terse arms only where it must be**: arm B/C are
charged every byte of the legend card and every authored `K:`/`V()` token; arm A is
charged the schema card it genuinely needs and its `PREFIX` lines. The authored-query
token component is measured on each arm's **reference** query (the query a competent
agent writes — the gold query for A, its terse rewrite for B/C), so "fewer tokens by
writing a *wrong* query" cannot win: the quality gate runs on the SAME reference query.

## Pre-registered significance bar (frozen — §5.4)

A lever's **token win** (`token_win = true`) requires the paired-median effective-input
reduction of `A − {B,C}` to exceed **BOTH**:

1. **≥ 20 % relative reduction** of the paired median (below ~20 % is inside the noise
   of the one independent 15–28 % memory benchmark — not worth the surface's
   maintenance), **AND**
2. **p < 0.05** via the Wilcoxon signed-rank test over **≥ 30 tasks**, with a bootstrap
   95 % CI on the median delta whose **lower bound is also > 0**, **AND**
3. the win **survives the cache discount** (is not solely the ×0.1 card component) AND
   the card tax is charged.

> **N + fidelity caveat, declared up front.** The frozen task set is **N = 30**
> (stratified, ≥ 4 strata) — it MEETS the §5.4 task-count bar, so the Wilcoxon /
> bootstrap test is run and reported. BUT this run's tokens are the **input-authoring
> proxy**, not the full-session transcript A/B (no fan-out available here). So a
> `token_win = true` from THIS run is a **proxy win on the dominant input-side lever**,
> explicitly flagged as such in the verdict (`fidelity: "input-authoring-proxy"`); it
> is upgraded to a full verdict only when the transcript fan-out runs. Reporting the
> proxy honestly — neither over- nor under-claiming — is the §5 discipline.

## Stratification (§5.4 anti-selection-bias)

The 30-task set spans **4 strata**, each reported separately, so a point-lookup-heavy
set does not trivially favour the terse surface:

- `point-lookup` — one class+property pattern ("identifiers of every Open Task").
- `multi-hop` — a join/traversal ("Closed tasks some Task dependsOn"; Finding→topic).
- `synthesis` — aggregation / GROUP BY / ordering / UNION over several patterns.
- `negative` — a deliberately empty answer-set (a status/topic/priority absent from the
  PKG) — tests that the terse arm does not *manufacture* rows and that `V()` over an
  absent phrase **loud-fails** rather than silently mis-binds.

## Quality scoring (deterministic, blind — §5.3)

Each arm's reference query is graded by the REAL toolchain, never self-reported:

- `parses` — the (transpiled, for B/C) query parses under the vendored `spargebra`
  (the `terse-expand` example runs the silent-rewrite canary; arm A is parsed by the
  engine).
- `grounded` — every predicate/class IRI in the canonical query is present in the PKG
  dictionary (no out-of-schema term). Computed against the schema term-set.
- `answer_correct` — answer-set **F1** of the arm's canonical query result rows vs the
  **gold** query result rows on the pinned PKG snapshot (`sparq-cli query`).
- `resolution_correctness` (arm C only) — every `V("phrase")` resolved to the **gold**
  IRI for that phrase (from the task's `concepts` map). A `V()` that binds the wrong
  IRI but still yields a tolerant answer-set is caught here.

## Kill criteria (frozen — §5.4)

- **KILL-token:** paired-median effective reduction `< 20 %`, OR Wilcoxon `p ≥ 0.05`,
  OR the bootstrap median-delta CI includes 0, OR the saving is entirely the
  cache-discount component, OR (arm B) it inverts once the legend-card tax is charged at
  its production cache multiplier (the prefix-tax trap). → do not adopt for token reasons.
- **KILL-quality:** the terse arm raises the out-of-schema (`grounded`) rate, OR drops
  `answer_correct` below arm A, OR (arm C) `resolution_correctness < 1.0` on the frozen
  set (a single silent mis-bind fails the lever). → reject regardless of token saving.

## Verdict object (§5.5) — emitted PER LEVER

```json
{ "lever": "keyword|V",
  "fidelity": "input-authoring-proxy|full-session-transcript",
  "token_delta_median_pct": float, "token_delta_ci": [lo, hi], "token_win": bool,
  "card_tax_charged": true, "win_survives_cache_discount": bool,
  "first_shot": { "parses": float, "grounded": float, "answer_correct_f1": float,
                  "resolution_correctness": float|null },
  "honest": bool, "recommend_adopt": bool }
```

`recommend_adopt = true` requires `token_win AND win_survives_cache_discount AND
card_tax_charged AND quality non-regression AND (lever V ⇒ resolution_correctness == 1.0)`
**AND** `fidelity == "full-session-transcript"` for an UNCONDITIONAL adopt — a
proxy-fidelity run can at most return a **conditional** recommend pending the fan-out.
The decision is made on the verdict OBJECT, never on a single number.
