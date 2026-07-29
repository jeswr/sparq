<!-- [OPUS-5] sq-2489d.6 (epic sq-2489d, issue #3244). 🤖 SPARQ agent — frozen
pre-registration for the Phase-7 end-to-end A/B of the provenance-driven KB. -->
# PRE-REGISTRATION — Phase 7: provenance-driven KB vs inert-PKG baseline

Frozen **before** any run, per the `research/agent-effectiveness-program.md` discipline.
**Editing this file to fit a result is forbidden.** The decision procedure is
[`analyze_prov.py`](./analyze_prov.py); the thresholds below are the ones it applies.

`research/provenance-driven-genai-kb.md` §5 Phase 7 is the honest payoff test for the
three capabilities Phases 1–5 landed. The governing rule of that record: **do not assume
provenance-weighting, answer-hedging or citations improve any outcome** — each ships
behind an on/off ablation and a pre-registered metric, and the deliverable is *a measured
verdict*, not "we built it". This pre-registration is that metric.

## 1. Hypotheses

- **H1 (citations).** Emitting provenance-derived citations reduces unresolvable claims
  in the agent's answer without degrading task outcome.
- **H2 (hedging).** Assurance-derived hedging + the abstention floor raises task outcome
  on the negative stratum (honest "no rows") without over-abstaining on answerable tasks.
- **H3 (provenance-weighting).** Weighting triples by `pkg:confidence`/`pkg:assurance`
  changes retrieval enough to move task outcome.

**Null in every case.** The literature reports *inconsistent* gains for confidence-weighted
KGE, and citation correctness ≠ faithfulness; a null result is the expected outcome for at
least one capability and is recorded as such, not buried.

## 2. Arms

All arms answer the **same frozen 30-task set** ([`tasks/abm_tasks.json`](./tasks/abm_tasks.json),
4 strata, PKG-answerable by construction, reused unchanged) over the **same graph**. Arms
differ only in which ablation switch is on.

| arm | capability | ablation switch |
|---|---|---|
| `base` | none — the **inert PKG** baseline | `citations` feature off, `NlqConfig::qualify = None`, `WeightMode::Uniform` |
| `cite` | citations | `sparq-nlq` feature `citations` (`Answer::citations`) |
| `hedge` | hedging + abstention | `NlqConfig::qualify = Some(..)` (`Nlq::ask_qualified`) |
| `weight` | provenance-weighting | `sparq-vectors` `WeightMode::Provenance` (feature `structure`) |

Each capability arm is compared **pairwise against `base` on the same task**, so task
difficulty cancels. One fresh sub-agent per `(task, arm)`; its brief opens with the
attribution tag `[ABM task=<id> arm=<base|cite|hedge|weight>]` that
[`tokens_real.py`](./tokens_real.py) mines. Arm order is counterbalanced across tasks.

## 3. Metrics

- **Tokens** — cache-discounted **effective input tokens** mined from the real
  transcripts (`1.0·input + 0.1·cache_read + 1.25·cache_creation`, the canonical §5.1
  multipliers). No `count_tokens`, no char proxy. Delta = `base − arm`, so a positive
  delta means the capability is cheaper.
- **Outcome** — the frozen `analyze3.grade` scorer: gold-key coverage on answerable
  strata, honest-abstention on the negative stratum. Delta = `arm − base`.
- **Capability honesty metrics** — see §4. These are the claims Phases 1–2 must not break,
  and they are pass/fail, not traded off against tokens.

## 4. Required instrumentation (per `(task, arm)` answer record)

A capability whose honesty claim cannot be checked is **not measured**. If the host does
not report the field below for *every* paired task, that capability's verdict returns
`honest = false` with a `blocked_reason` — the analyzer never defaults a missing field.

| capability | required fields | meaning |
|---|---|---|
| citations | `citations_emitted`, `citations_resolved` | count emitted vs count resolving to a real in-graph `prov:wasDerivedFrom` source |
| hedging | `abstained` | did the arm decline to assert an answer |
| provenance-weighting | — | outcome/tokens only |

## 5. Bars (frozen)

Imported unchanged from [`stats.py`](./stats.py) so this harness cannot silently weaken
the shared §5 bar it claims to reuse: `MIN_N = 30`, `MAX_P = 0.05`,
`MIN_RELATIVE_REDUCTION` (the `token_win` bar), Wilcoxon signed-rank + 10 000-iteration
bootstrap median CI.

- **`token_win`** — the shared §5 definition, unchanged: median relative reduction clears
  `MIN_RELATIVE_REDUCTION`, Wilcoxon `p < MAX_P`, `N ≥ MIN_N`, bootstrap CI lower bound > 0.
- **`token_ok`** — Phase-7-specific ADDITION. These capabilities are expected to *cost*
  tokens (a citation footnote is payload) and to pay on the outcome side, so a capability
  passes if it wins on tokens **or** its median overhead is **≤ 10 %** of the baseline's
  median effective input tokens (`MAX_TOKEN_OVERHEAD`).
- **`outcome_win`** — median outcome delta > 0, Wilcoxon `p < MAX_P`, `N ≥ MIN_N`, and
  bootstrap CI lower bound > 0.
- **`honesty_ok`** — citations: zero fabricated citations **and** resolution rate 1.0
  (Phase 1's own pre-registered metric). Hedging: the arm answers confidently-wrong no
  more often than `base` does.
- **`honest`** — no blocking reason: every relevant task has a complete `(base, arm)`
  transcript pair **and** a complete `(base, arm)` answer pair, `N ≥ MIN_N`, the required
  instrumentation is present, and the capability is on the answer path the task set
  exercises. A task missing an answer on either arm is dropped from the pairing and
  reported, never graded as 0.0 — otherwise an unanswered baseline would read as a
  perfectly-failed one and flatter the arm.

**`recommend_adopt = honest AND outcome_win AND token_ok AND honesty_ok`.** Decide on the
verdict object, never on one number.

## 6. Kill criteria

- A capability that fails `outcome_win` is **not adopted**, however cheap it is. "It built"
  is not a result.
- A capability that fabricates one citation fails outright — no token or outcome win
  redeems it.
- A capability whose token overhead exceeds the ceiling is not adopted even on an outcome
  win; re-pre-register a higher ceiling *before* re-running, never after seeing the number.

## 7. Scope limits recorded in advance (so they cannot be quietly dropped later)

- **Provenance-weighting is not on this task set's answer path.** `w(t)` feeds KGE training
  and per-`Block` fusion; the `pkg-query` path these 30 tasks exercise is canned SPARQL
  (`ORDER BY DESC(?conf)`), not vector retrieval. Only ~a third of the tasks use a
  confidence-ordered canned query, which is below `MIN_N` by construction. Its
  pre-registered metric therefore remains the **Phase-4 Hits@k/MRR ablation**
  (`crates/sparq-vectors/examples/kge_ablation.rs`); the analyzer emits `honest = false`
  with that reason rather than a flattering under-powered verdict.
- **`needs-maintainer-steer`: the canonical A/B host is not chosen.** Which model, on which
  box, with which capability build, is the maintainer's call and is not decided here.
- **Every verdict is model-dependent.** Issue #1111 requires a **re-run under Fable**
  before any verdict — adopt *or* abandon — is treated as final.
- **All numbers are NON-CANONICAL** (work-box, runtime-only). `verdict-phase7.json` and the
  token/answer artifacts are git-ignored; the committed artifacts are the harness code,
  the frozen task set, and this pre-registration.

## 8. Running it

```bash
# 1. a host runs 4 arms × 30 tasks, one fresh sub-agent each, tagged [ABM task= arm=]
python3 bench/pkg-dogfood/tokens_real.py <transcript-dir> tok_p7.json
python3 bench/pkg-dogfood/analyze_prov.py \
  --tokens tok_p7.json --answers answers-p7.json --out verdict-phase7.json
```
