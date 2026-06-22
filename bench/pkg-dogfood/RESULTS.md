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
