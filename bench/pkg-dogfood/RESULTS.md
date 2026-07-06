<!-- [OPUS-4.8] sq-ve5dy / sq-zbyo7 (epic sq-2m6zm). 🤖 SPARQ agent — the REAL
cache-discounted 3-arm token A/B measurement record. This is the SANCTIONED home for
the measured numbers (bench/ is exempt from check-no-perf-numbers.py); no other
markdown repeats them. Written while Fable unavailable; flag for re-review when Fable
returns. -->

# RESULTS — PKG-query 3-arm token A/B (real transcript telemetry)

> 🤖 **SPARQ agent.** This is the measurement record for bead **sq-ve5dy** (agent
> flavour) — proven by **sq-zbyo7**. It **supersedes the #1078 char-proxy** A/B,
> whose lower-fidelity estimate inverted this verdict (see the CRITICAL LESSON below).
> `bench/` is the AGENTS.md-sanctioned home for measured figures; the numbers below
> live HERE and are not repeated in any user-facing doc.

## The question

Is it cheaper, **at equal answer quality**, to let a cheap model (Haiku) answer a
PKG-answerable project-knowledge question as a natural-language tool call than to have
the expensive orchestrator (Opus) read the source documents — and is it cheaper than
having Opus run `pkg-query` itself? The decision metric is **model-price-weighted $**
(not raw tokens), because the whole point of the cheap-model NL-tool is to push the
verbose NL→SPARQL→run→NL middle onto a model ~15× cheaper per token.

## The three arms

| Arm | Who answers | How |
|---|---|---|
| **A** | Opus | reads the source docs (AGENTS.md section / `bd show`) to answer |
| **B** | Opus | runs `pkg-query` itself (introspect → ground → ask) |
| **C** | **Haiku sub-agent** | runs `pkg-query` as an NL tool; Opus only emits the question and reads the NL answer + the executed SPARQL + provenance |

## The measured result

N = 30 frozen tasks (`tasks/abm_tasks.json`), single counterbalanced run. Tokens are
the **real cache-discounted effective input tokens** mined straight from each fresh
sub-agent's transcript `message.usage` (`1.0·input + 0.1·cache_read + 1.25·cache_creation`);
**no `count_tokens` API, no char proxy**. Quality is gold-key coverage per task
(negative-stratum tasks graded by honest-abstention). Prices (list approx, USD/Mtok):
Opus $15 in / $75 out; Haiku $1 in / $5 out.

| Arm | median eff. input tok | total $ (30 tasks) | quality |
|---|---|---|---|
| **A** — Opus read-docs | 96,411 | $47.19 | 0.92 |
| **B** — Opus pkg-query | 49,426 | $24.79 | 1.00 |
| **C** — Haiku pkg-query (NL-tool) | 38,199 | **$1.57** | **1.00** |

**$-cost ratios (total over the 30 tasks):**

- A read-docs is **≈ 30× the cost of C** (the Haiku NL-tool) — **at equal or better quality**.
- B (Opus pkg-query) is **≈ 16× the cost of C**.
- A is ≈ 1.9× the cost of B.

Quality was **1.00 across all strata** for arms B and C; arm A (read-docs) was 0.92,
losing the most on multi-hop and negative tasks (it has to read more to confirm an
absence, and is likelier to miss a join key). So the cheap-model NL-tool is the
cheapest arm **and** ties the best quality.

## Method (how this was measured / how to reproduce)

1. **Frozen task set** — `tasks/abm_tasks.json`: 30 PKG-answerable questions across 4
   strata (point-lookup / multi-hop / synthesis / negative), each with `gold_keys` and
   the canned `pkg-query` (`armB`) that answers it. Frozen before the run.
2. **Fresh sub-agent per (task, arm)** — each of the 30×3 = 90 cells is answered by a
   brand-new sub-agent whose brief opens with the tag `[ABM task=<id> arm=<A|B|C>]`.
   Arms A and B run on Opus; arm C dispatches a `model:haiku` sub-agent that does the
   whole NL→SPARQL→run→NL round-trip and returns the answer + the executed SPARQL +
   provenance. This was driven as **two `pkg-token-ab` workflow runs** (one for A+B,
   one for C), each fanning out fresh sub-agents so no context bleeds between cells.
3. **Mine real tokens** — `tokens_real.py <transcript-dir>` reads every `agent-*.jsonl`
   transcript, attributes it by its `[ABM …]` tag, and sums the cache-discounted
   effective input + output tokens from `message.usage`. One JSON row per cell.
4. **3-arm $-weighted verdict** — `analyze3.py --tasks tasks/abm_tasks.json --tokens
   <tok_AB.json> <tok_C.json> --answers <answers.json> --out verdict.json` grades each
   answer against its gold keys and reports median effective tokens, total $, $/task,
   and quality per arm + per stratum, with the A/B/C $-ratios.

```bash
# reproduce from a pair of workflow transcript dirs:
python3 bench/pkg-dogfood/tokens_real.py <dir-with-A+B-transcripts> tok_AB.json
python3 bench/pkg-dogfood/tokens_real.py <dir-with-C-transcripts>   tok_C.json
python3 bench/pkg-dogfood/analyze3.py \
    --tasks   bench/pkg-dogfood/tasks/abm_tasks.json \
    --tokens  tok_AB.json tok_C.json \
    --answers answers.json \
    --out     verdict.json
```

`tok_*.json` / `answers.json` / `verdict.json` are runtime artifacts (git-ignored) —
regenerate them; the committed artifacts are the harness CODE + the frozen tasks.

## Honest caveats (load-bearing)

- **Tasks are PKG-answerable by construction.** The 30 questions were chosen to have a
  PKG answer; they do **not** sample questions where the fact is outside the head-slice
  PKG (those force a fallback to Read/Grep on every arm). The win generalises only to
  the PKG-answerable question class — which is exactly the class the `query-pkg` skill
  scopes itself to.
- **The gold keys mildly favour pkg-query on quality.** They reward the structured,
  row-shaped answer the canned queries return; a doc read that paraphrases can miss an
  exact key. The quality gap (0.92 vs 1.00) is therefore a *modest* over-statement of
  A's real-world miss rate — but the **$ gap is an order of magnitude** and is the
  load-bearing result, not the quality tie.
- **Arm C's $ omits a small Opus delegation cost.** The orchestrator still spends a few
  Opus tokens to emit the question and read the NL answer back. That delegation
  overhead (question + short answer, no schema card / SPARQL / rows in Opus context) is
  small relative to the ≈ 30× gap and does not change the verdict's direction, but the
  headline $1.57 is the **Haiku-side** cost only.
- **N = 30, single run, directional.** This is one counterbalanced run, not a
  multi-run significance study. It is enough to establish the **direction and order of
  magnitude** of the win; it is not a p-value. Treat the ratios as "≈ 30× / ≈ 16×",
  not as a measured constant.
- **All numbers are runtime / NON-CANONICAL** (work-box transcripts, list-price
  approximations). The prices are stated openly so the ratio is auditable and easy to
  re-point if list prices move.

## CRITICAL LESSON — prefer REAL measurement over a proxy

The earlier **#1078 char-proxy** A/B (the `tokens.py` PoC + `PREREG.md`, which honestly
flagged itself as a lower-fidelity proxy) estimated read-payload bytes via a char→token
ratio and an *idealised minimal scoped read* for arm A. Under that idealisation arm A
looked competitive — the proxy **inverted this verdict**. The real-transcript
measurement here shows the read-docs arm actually consumes far more effective input
tokens than the idealised scoped read assumed (the agent reads more than the minimal
section, re-reads on multi-hop, and pays the warm-prefix dynamics the proxy excluded).

**Takeaway:** a char/byte proxy with a charitable read-scope assumption is not a
substitute for mining the tokens the model actually consumed. When the decision is
load-bearing (adopt-or-not), measure the real transcripts. The proxy was useful as a
direction-finder, but the proxy's verdict must not be trusted over the measurement.

Licensed MIT (repo default).


---

## RE-RUN — Fable subject (sq-2m6zm.9, 2026-07-05): the A-vs-B verdict SHIFTS

> 🤖 **SPARQ agent** [FABLE-5]. Append-only re-run record for bead **sq-2m6zm.9**
> (#1111 re-attempt program, thread A rung 1; design record
> `research/neurosymbolic-fable-program.md`). Everything above this line is the
> original **Opus-subject** record, unchanged. Harness **byte-unchanged**: frozen
> `tasks/abm_tasks.json`, `tokens_real.py`, `analyze3.py` (verified `git diff` clean
> against `origin/main`). Subject substitution only: **Fable (`claude-fable-5`)
> replaces Opus in arms A and B**; arm C stays a Haiku NL-tool.

### Method delta (vs the original run)

- Same 30 frozen tasks × 3 arms = 90 cells; **one fresh headless `claude -p` session
  per cell** (`--model claude-fable-5` for A/B, `claude-haiku-4-5` for C;
  `--allowedTools Bash Read Grep Glob Skill`; brief opens with the `[ABM task= arm=]`
  tag), transcripts mined by the frozen `tokens_real.py`.
- **Serving-model gate (bead invariant).** Fable sessions can silently serve a
  different model mid-run, so each transcript's `message.model` was mined **per
  assistant line** (`model_ids.py`, committed alongside this record). A cell is valid
  only if *every* assistant line was served by the expected subject; mixed cells are
  excluded, never counted. **Result: 90/90 cells VALID, 0 excluded** — arms A/B:
  811/811 assistant lines `claude-fable-5`; arm C: 329/329 lines
  `claude-haiku-4-5-20251001`. Per-task evidence table below.
- **Operational note (recorded for honesty).** The first dispatch wave
  (2026-07-05 ≈ 18:07–18:25 UTC) hit the account session limit mid-window; 12 of the
  90 cells (t27–t30 × 3 arms) returned 429 error results. Those cells were **discarded
  entirely and re-run after the window reset** (≈ 20:10 UTC) under the same briefs —
  no partial or mixed transcript was counted.

### The measured result (verbatim `analyze3.py` output)

```
=== 3-ARM VERDICT (N=30 tasks, real cache-discounted tokens) ===
prices/Mtok (list approx): Opus $15 in/$75 out  Haiku $1 in/$5 out

arm                             med eff_in  med out   total $  med $/task  quality
A=Opus read-docs                   154,440    2,516    91.246      2.4592     0.96
B=Opus pkg-query                   185,678    7,551   103.651      3.3475     0.97
C=Haiku pkg-query (NL-tool)        131,532    3,439     5.209      0.1589     0.92

=== $-COST RATIOS (total over the task set) ===
  A read-docs total: $91.246
  B pkg-query total: $103.651   -> A is 0.9x B
  C Haiku NL-tool:   $5.209   -> A is 17.5x C ; B is 19.9x C

=== QUALITY per arm per stratum (mean gold-coverage) ===
  stratum           A      B      C
  point-lookup   0.88   1.00   0.94
  multi-hop      0.97   0.89   0.78
  synthesis      1.00   1.00   1.00
  negative       1.00   1.00   1.00

wrote /tmp/sq2m6zm9/verdict_fable.json
```

`analyze3.py` prices arms A/B at its frozen **Opus-era $15/$75** — kept verbatim per
the re-run-not-rebuild invariant. Re-priced at the **actual subject-model list prices**
(Fable $10 in / $50 out; Haiku $1 / $5 per Mtok — same token rows, reporting script
`fable_dollars.py` in the run scratch, frozen analyzer untouched):

```
arm A: n=30 total=$60.831 median/task=$1.6395
arm B: n=30 total=$69.101 median/task=$2.2317
arm C: n=30 total=$5.209 median/task=$0.1589
A/C=11.7x  B/C=13.3x  A/B=0.9x
```

Per-task pairing on the same frozen rows: **B beats A on effective input tokens on
only 12/30 tasks; median B/A eff-token ratio 1.11** (median assistant turns: A 7.5,
B 15).

### Verdict vs the Opus-era record — SHIFTED on A-vs-B; delegation holds in direction

| Opus-era claim (record above) | Fable-subject outcome |
|---|---|
| **B (pkg-query) ≈ halves A (read-docs) eff tokens** (B cheaper on 29/30 tasks) | **Does NOT hold.** B is *more* expensive than A: median B/A ratio **1.11**, B cheaper on **12/30**; A ≈ 0.9× B in $ under both pricings. Fable reads docs efficiently (7.5 median turns) but drives a much longer introspect→ground→ask loop (15 median turns). |
| **C (Haiku NL-tool) ≈ 30× cheaper than A at equal-or-better quality — ADOPTED** | **Direction holds; magnitude compresses; a quality gap appears.** C ≈ **11.7×** cheaper than A at actual subject prices (17.5× at the analyzer's frozen Opus-era prices). Quality: C 0.92 vs A 0.96 / B 0.97 — C loses on multi-hop (0.78) this run, so "equal or better" weakens to "slightly below, at ≈ 1/12 the cost". |

The model-dependence #1111 predicted is real, and it points the **opposite way** on
this benchmark: with Opus, read-docs was the *worst*-quality arm (0.92 vs 1.00); with
Fable, read-docs is cheap **and** near-top quality (0.96) while the pkg-query middle
spends twice the turns for no quality gain. **A stronger orchestrator needs the
symbolic middle less.** The cheap path for a Fable orchestrator is: read the docs, or
delegate to the Haiku NL-tool — not run `pkg-query` itself. (The decision consequence
belongs to the `query-pkg` skill owner / epic `sq-2m6zm`, not this record.)

### Per-task serving-model ids (validity evidence)

<details>
<summary>90-cell model-id table (mined from transcript <code>message.model</code>; all VALID)</summary>

| task | arm A | arm B | arm C |
|---|---|---|---|
| t01 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t02 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t03 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t04 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t05 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t06 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t07 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t08 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t09 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t10 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t11 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t12 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t13 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t14 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t15 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t16 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t17 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t18 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t19 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t20 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t21 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t22 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t23 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t24 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t25 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t26 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t27 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t28 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t29 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |
| t30 | claude-fable-5 | claude-fable-5 | claude-haiku-4-5-20251001 |

</details>

### Honest caveats

- **N = 30, single counterbalanced run per subject.** Cross-run deltas (e.g. arm C
  quality 1.00 → 0.92) carry run-to-run and grader-style variance; the load-bearing
  signal is the **within-run** arm ordering, measured by the same frozen grader.
- **Answer-style sensitivity.** The frozen gold-key grader rewards verbatim,
  row-shaped answers; Fable's synthesizing style can under-resolve keys a row-reading
  agent surfaces verbatim (same caveat class as the original record's "gold keys
  mildly favour pkg-query").
- **All numbers runtime / NON-CANONICAL** (work-box transcripts, list-price
  approximations, stated openly). Run artifacts are regenerable and git-ignored per
  the existing policy; the committed artifacts are this record + `model_ids.py`.
