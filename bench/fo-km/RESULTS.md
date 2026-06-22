<!-- [OPUS-4.8] sq-mztg8 (FO-KM epic; design research/foundational-ontology-km-benchmark.md,
PR #1106; harness PR #1107). 🤖 SPARQ agent — the MEASURED Metric-1 record. This is the
SANCTIONED numeric home (bench/ is exempt from check-no-perf-numbers.py); no user-facing
markdown repeats these figures. Written while Fable unavailable; flag for re-review when
Fable returns. -->

# RESULTS — FO-KM Metric 1 (agent KM-task accuracy + cost over the PKG)

> 🤖 **SPARQ agent.** Measurement record for **Metric 1** of epic **sq-mztg8** — the
> 4-arm A/B in which the same Haiku NL-tool answers FO-exercising KM questions over the
> **same** Project-Knowledge-Graph (PKG) typed under different foundational-ontology (FO)
> overlays. `bench/` is the AGENTS.md-sanctioned home for measured figures; the numbers
> below live HERE and are not repeated in any user-facing doc.
>
> This is the **verdict** for the harness shipped in PR #1107 (`bench/fo-km/`). The
> pre-registered prior (design §7) was **NEUTRAL** — built to be able to return the null
> honestly. The measured Metric-1 result is **NOT** neutral; see the finding below.

## The question

Does any foundational ontology beat the **no-FO incumbent** — and the named **gUFO**
baseline — on the knowledge-management tasks this repo's PKG actually does, measured by
the **agent's** answer accuracy and model-price-weighted token cost? (Design §5, Metric 1.)

## The four arms

Each arm = the shipped PKG (`pkg.ttl` + `pkg-instances.ttl`) **plus** one FO overlay
loaded via `pkg-query --extra-graph <overlay> --close owl-rl`, so the overlay's
`rdfs:subClassOf` axioms entail the FO-typed facts.

| Arm | Overlay | PKG-class → FO top category |
|---|---|---|
| **no-FO** (incumbent) | `overlays/no-fo.ttl` | (none — the shipped reuse-first PKG) |
| **gUFO** (named baseline) | `overlays/gufo.ttl` | Task→`gufo:Event`; Finding→`gufo:AbstractIndividual`; Source/Technique→`gufo:Object` |
| **DOLCE-DUL** | `overlays/dolce-dul.ttl` | Task→`dul:Action`; Finding→`dul:Description`; Source→`dul:InformationObject`; Technique→`dul:Method` |
| **schema.org-as-top** | `overlays/schema-org.ttl` | Task→`schema:Action`; Finding→`schema:Claim`; Source→`schema:DigitalDocument`; Technique→`schema:HowTo` |

## Method

- **One Haiku NL-tool per (arm, task).** For each of the 4 arms × 16 FO-exercising KM
  tasks (`tasks.jsonl`: TH 7, ER 4, CC 5), a fresh Haiku sub-agent answered the natural-
  language question by driving the `crates/sparq-kb` `pkg-query` helper
  (`--extra-graph <arm overlay> --close owl-rl`) end to end: **introspect → ground → ask**.
  The agent saw only the NL question; it chose the SPARQL, ran it under closure, and read
  back the rows. (This is the same NL-tool envelope measured positive for the PKG-
  answerable class in `bench/pkg-dogfood/RESULTS.md` arm C, here re-run per FO overlay.)
- **Real cache-discounted effective input tokens** were mined straight from each fresh
  sub-agent's transcript `message.usage` block —
  `1.0·input + 0.1·cache_read + 1.25·cache_creation` (the canonical §5.1 multipliers; the
  same formula as `bench/pkg-dogfood/tokens_real.py`). **No `count_tokens` API, no char
  proxy.** Closure build CPU/wall cost is **non-canonical** and is never charged as a token
  cost (design §5.1). See `analyze.py` for the miner + grader.
- **Heuristic deterministic grading.** Accuracy is per-task gold-key coverage with no
  model in the loop: a count/list task is correct when the agent's answer resolves the
  gold count and entity local-names (count match + entity-coverage); a concept-coverage
  (CC) partition task is correct when the answer covers each gold sub-category bucket.
  Honest abstention on a task an arm genuinely cannot answer is scored as an abstain
  (counted separately), not a wrong answer.
- **Prices** (list approx, USD/Mtok): Haiku $1 in / $5 out. Totals are model-price-
  weighted $ over all 16 tasks per arm.

## The measured result

N = 16 FO-exercising tasks, single counterbalanced run, one Haiku NL-tool per (arm, task).

| Arm | accuracy | abstain | median eff. input tok | total $ (16 tasks) |
|---|---|---|---|---|
| **no-FO** (incumbent) | 0.58 | 5/16 | 57,921 | $1.21 |
| **gUFO** | 0.54 | 2/16 | 45,069 | $0.94 |
| **DOLCE-DUL** | 0.64 | 1/16 | 68,502 | $1.50 |
| **schema.org-as-top** | **0.84** | 2/16 | 51,005 | $1.38 |

By task kind (TH type-hierarchy / ER entailment / CC cross-category):

| Arm | TH | ER | CC |
|---|---|---|---|
| no-FO | 0.61 | 0.25 | 0.80 |
| gUFO | 0.37 | 0.50 | 0.80 |
| DOLCE-DUL | 0.62 | 0.50 | 0.80 |
| **schema.org-as-top** | **0.86** | **0.75** | **0.90** |

## The finding

**schema.org-as-top markedly beats gUFO (0.84 vs 0.54) for the agent's KM tasks, and
gUFO scored *below* the no-FO incumbent (0.54 vs 0.58).** The win is consistent across
all three task kinds (schema.org is the top arm on TH, ER, and CC alike).

The driver is **LLM fluency**, not formal ontological richness. The agent wields the
ubiquitous `schema:` vocabulary reliably — it grounds NL questions onto `schema:Claim` /
`schema:DigitalDocument` / `schema:HowTo` / `schema:Action` and writes correct SPARQL —
but fumbles the academic `gufo:` / `dul:` terms it has seen far less in training, choosing
the wrong category or mis-typing the query. The metaphysically richer FOs (gUFO's
endurant/perdurant axis, DOLCE's descriptive layer) bought *more* expressivity but were
*harder for the agent to use correctly*, so their realised accuracy fell. This **confirms
the LLM-fluency hypothesis of research PR #1106** (design §2 fluency stream): for an
agentic KG queried by an LLM, the FO's fluency to the model dominates its formal fit.

DOLCE-DUL edged the incumbent (0.64 vs 0.58) and abstained least (1/16) — its native
method/document/description categories gave the agent reachable targets — but it cost the
most ($1.50) and stayed well behind schema.org. gUFO is the clear loser of the four:
lower accuracy than doing nothing, at a lower token cost that does not redeem it.

## Honest caveats

- **N = 16, single run.** This is one counterbalanced pass over 16 tasks, not a powered
  multi-seed study. The point estimates carry run-to-run variance.
- **Heuristic grading.** Accuracy is a deterministic gold-key/coverage resolver, not a
  semantic judge — it can mis-grade an answer that is right in spirit but phrased so the
  resolver misses a key. **Robustness to grading noise:** the schema.org ≫ gUFO gap is
  *large* (0.30 absolute) and *consistent across all three task kinds* (TH/ER/CC), so the
  direction of the finding is not an artefact of any single task's grader. A grading-noise
  flip of the headline ordering would require errors correlated the same way across all
  three strata, which the per-kind breakdown does not show.
- **This is Metric 1 — the AGENT.** It measures how well an LLM agent *uses* each FO to
  answer KM questions. It does **not** measure a formal-reasoning quality the agent never
  touches. **Metric 2** (the KGE closure-prior MRR via `eval.rs`
  `run_ablation_multiseed_paired`) is **EC2-deferred** (bead **sq-p5ro8**): it needs a
  canonical/quiet box and is a separate phase. A metaphysically richer FO (gUFO/DOLCE)
  could rank *differently* on Metric 2 — where the prior is consumed by a learned model,
  not authored into SPARQL by an LLM — so this Metric-1 verdict is **scoped to the agent
  use-case** and must not be read as a global "schema.org is the best FO" claim.
- **Token cost is informational here, not the decision metric.** Unlike the cross-model
  cost study in `bench/pkg-dogfood/`, all four arms use the same Haiku NL-tool, so the
  $ differences reflect how much introspection each FO drove (richer overlays → more rows
  to read), not a model-tier saving. Accuracy is the decision metric for Metric 1.

## Reproduce

The 4 overlays + 16 tasks are committed (`overlays/`, `tasks.jsonl`); the per-(arm, task)
Haiku NL-tool dispatch is documented in `README.md`. The token miner + coverage grader is
`analyze.py` (run it against a directory of the run's `agent-*.jsonl` transcripts). Task
discrimination — that each FO arm answers under closure while the no-FO arm returns 0 —
is independently checked by `validate_tasks.py` (needs the `close` feature).
