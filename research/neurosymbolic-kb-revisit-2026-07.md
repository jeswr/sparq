# Neurosymbolic self-built-KB — fresh Fable-tier verdict (2026-07)

<!-- 🤖 SPARQ agent (Claude Fable 5) — fresh design-for-review verdict record for
bead sq-7g7ox. [FABLE-5] -->

**Status:** research record, design-for-review. Re-derivation of the self-built-KB
direction from first principles, NOT anchored to the prior model's verdicts.
**Bead:** `sq-7g7ox` (P2) · **GitHub:** #1111 (`revisit-with-fable`), #1110 (PROV-O
load-bearing).
**Builds on (does not restate the numbers of):**
`research/dogfooding-sparq-knowledge-graph.md`,
`research/provenance-driven-genai-kb.md`,
`research/research-kb-program.md`,
`research/neurosymbolic-fable-program.md`,
`research/fo-llm-bridge.md`,
`research/foundational-ontology-km-benchmark.md`.
Measured figures live only in `bench/pkg-dogfood/RESULTS.md` and
`bench/fo-km/RESULTS.md` (the AGENTS.md-sanctioned numeric homes); this record cites
them by direction and points at those files rather than repeating the figures.

## 0. Correction to the brief's premise (read this first)

The brief asks for a *fresh Fable-tier revisit* of the self-built-KB direction
"because prior verdicts are model-dependent (e.g. the pkg-dogfood verdict REVERSED to
adopt at real fidelity)". Two parts of that framing are stale against `origin/main` and
must be corrected before any honest re-derivation:

1. **The Fable-subject re-run already happened.** Bead `sq-2m6zm.9` is CLOSED
   (merged PR #1603, 2026-07-06). Both prior benchmarks — the PKG token A/B and the
   FO-KM accuracy A/B — were re-run with Fable (`claude-fable-5`) as the subject model,
   with per-cell model-id verification (no silent substitution: 811/811 and 329/329
   and 64/64 model-id-checked lines respectively; see the RE-RUN sections of both
   `bench/*/RESULTS.md`). So this is not a *first* Fable revisit — it is a **second
   opinion on an already-executed Fable re-run**, plus a first-principles verdict on
   the components the re-run did not settle.

2. **The pkg-dogfood "reversed to adopt" story is a two-stage history, and the second
   stage partly walked it back.** The reversal the brief alludes to was the
   *char-proxy → real-token* correction (the #1078 proxy had inverted the verdict; the
   real-transcript N=30 measurement flipped it back to "adopt the cheap-model NL-tool").
   That reversal was measured with **Opus** as the expensive orchestrator. The **Fable**
   re-run then found the direction *holds but compresses*, and surfaced a **new**
   finding the brief does not mention: the "Opus `pkg-query` roughly halves read-docs
   tokens" sub-claim (arm B vs arm A) **does not hold for Fable** — a strong
   orchestrator reads docs efficiently and the introspect→ground→ask loop costs it
   *more*, not less. The headline delegation win survives; one of its two legs did not.

The consequence for this record: the interesting open questions are no longer "does the
prior verdict survive a stronger model" (it was tested) but "given what the Fable re-run
actually showed, which components are worth *building further* vs freezing vs dropping".
That is what §2–§7 answer.

## 1. What is already built (verified against the code, not the docs)

Read the actual crate before deciding anything. `crates/sparq-kb` is a real,
compiling, tested crate — this direction is **not greenfield**. Verified state:

| Component | State | Evidence |
|---|---|---|
| PKG ontology + SHACL shapes | IMPLEMENTED | `crates/sparq-kb/ontology/pkg/pkg.ttl`, `shapes/pkg.shapes.ttl`; every `Finding` requires `prov:wasDerivedFrom` (`sh:minCount 1`), a `pkg:confidence` in `[0,1]`, and a `pkg:assurance` from `{Proven, Claimed, Conjectured}` |
| Provenance emission (PROV-O) | IMPLEMENTED | pipeline emits `prov:wasDerivedFrom` / `wasGeneratedBy` / `wasAttributedTo` / `generatedAtTime` per finding; `sparq-prov` produces per-fact lineage |
| DQV alignment | IMPLEMENTED | `sq-2489d.3` merged; `pkg:confidence rdfs:subPropertyOf dqv:value`, `pkg:assurance` a `dqv:Metric` |
| Canned NL-tool (introspect→ground→ask) | IMPLEMENTED | `src/query/nl_tool.rs`, `canned.rs`, `bin/pkg_query.rs`; deterministic query + grounding envelope (executed SPARQL + resolved/ungrounded IRIs + confidence tag). NOT an LLM NL→SPARQL translator; the cheap-model wrapper is external |
| Literature pipeline (fixture path) | IMPLEMENTED (replay-only) | `src/literature/*.rs`; connector→normalise→extract(replay)→ground→emit-TTL→sidecar over committed fixtures, zero network, zero live model in CI |
| Literature live connector + extractor | WIRED-BUT-INERT | `literature-live` feature (default-OFF), `LiveExtractor` over an injectable runner; maintainer-gated OFF (`sq-tzars.9`, PR #1609); never driven in CI |
| FO typing (schema.org-as-top) | RATIFIED (soft alignment) | `pkg.ttl` `skos:closeMatch`/`rdfs:subClassOf schema:*`; ratified in `sq-mztg8.5` (PR #1611). gUFO/DOLCE exist only as benchmark *overlays*, not in the PKG |
| UNCALIBRATED disclaimer | IMPLEMENTED | `sq-2489d.9` merged (PR #1594) — confidence-bearing answers surface the hand-authored-not-calibrated caveat |
| First KB dump | DONE (manual) | saved to `sparq-org/research-kb`; automated push ABORTED per #1552 (`sq-yfh5d`) |

The crate is opt-in by strict additivity (`default = []`; `validate`/`query`/`close`/
`literature`/`literature-live` all default-OFF; `publish = false`). Core stays lean.
This satisfies the opt-in-feature architecture rule.

**What is designed-but-not-consumed (the honest gap):** provenance and confidence are
*recorded and validated* but **inert** — nothing reads them back to weight embeddings
(USE 1), hedge answers beyond the static disclaimer (USE 2), or render citations from
lineage (USE 3). That inertness is the substance of the remaining decision, not the
substrate.

## 2. First-principles framing: is a self-built project KG worth it at all?

Strip the sunk cost and ask the question cold. An LLM coding agent working on a 37-crate
repo needs three things a raw document corpus serves badly:

- **Answer-sized, sourced facts** ("where was decision X made / what is the status of
  bead Y / which sources are unexplored") without paying tokens to read whole documents.
- **Durable, queryable memory** that survives context compaction and is shared across
  agents on one account (the maintainer runs several).
- **Provenance** so a retrieved fact carries where it came from and how much to trust
  it — the failure mode of flat agent memory is a confident un-sourced claim.

External prior art (surveyed 2026-07) confirms this is a live, *not-yet-settled*
research frontier, which supports the maintainer's "novel" flag rather than undercutting
it: graph-based agent memory that the agent *builds and reorganises* is the stated
2025–2026 frontier (e.g. graph-memory taxonomies and self-evolving graph-memory engines
such as SAGE/MAGMA lines of work), and provenance-bearing memory + anchor-constrained
grounding + provenance-based hallucination detection are being proposed exactly as
reliability levers (e.g. ProVe-style provenance verification, AEVS-style character-level
provenance for KG extraction). sparq is unusual in that it *is* an RDF/SPARQL/SHACL/
reasoner/vector engine, so the substrate is free — the KG is dogfood, not a new
dependency. That is a genuine and defensible differentiator.

**First-principles verdict on the direction:** worth pursuing, but the value is
concentrated in the *retrieval-economics* leg (answer-sized sourced facts), which is
measured-positive and already shipped, and in *provenance as a correctness lever*, which
is designed but unproven. The "self-evolving from literature-trawled facts" leg is the
speculative, cost-exposed part and should stay gated until the cheaper legs pay off.
This is a PARTIAL adopt at the direction level, decomposed per component below.

## 3. Component: PKG NL-tool routing economics — **ADOPT (with a scoped caveat)**

**Claim under test:** letting a cheap model (Haiku) answer a PKG-answerable question as
a natural-language tool call is cheaper, at equal quality, than the expensive
orchestrator reading source docs.

**Evidence (see `bench/pkg-dogfood/RESULTS.md`, do not restate the figures here):**

- **Opus-subject, real-transcript N=30:** the cheap-model NL-tool is the cheapest arm
  by a large multiple and ties the best quality. Robust.
- **Fable-subject re-run (`sq-2m6zm.9`):** the *direction holds* — the cheap NL-tool is
  still cheaper than a strong orchestrator reading docs by an order of magnitude — but
  the *magnitude compresses* (a strong orchestrator reads docs efficiently) **and a
  small quality gap appears** (the cheap tool loses a little on multi-hop tasks this
  run). So "equal-or-better quality" weakens to "slightly below, at a fraction of the
  cost".
- **New, load-bearing sub-finding:** the "Opus `pkg-query`-in-context halves read-docs
  tokens" leg (arm B < arm A) **does not reproduce for Fable** — B is *more* expensive
  than A for a strong orchestrator. The introspect→ground→ask loop is many turns; a
  strong model burns fewer turns just reading.

**Verdict: ADOPT the cheap-model NL-tool** as the default retrieval path, with two
honest scoping notes:

1. Route the round-trip to a *cheap* model. Do **not** have a strong orchestrator run
   `pkg-query` in-context expecting a token saving over reading docs — that saving is
   Opus-era and does not survive a strong orchestrator.
2. On multi-hop questions the cheap tool has a measured small accuracy deficit; the
   answer envelope (executed SPARQL + resolved/ungrounded IRIs + row count) is the
   mitigation — the orchestrator can see *whether* the answer was computed and
   re-ask/verify cheaply. Keep that envelope non-optional.

**What would settle the residual:** the end-to-end exec-accuracy of a *real* cheap-model
endpoint (not the canned router) is still open (`sq-2m6zm.7`, in progress). That is the
right place to firm the multi-hop gap; no new bead needed.

## 4. Component: FO typing choice (schema.org 0.84) — **PARTIAL: keep schema.org default; the 0.84 dominance is Haiku-scoped**

**Claim under test:** schema.org-as-top is the right foundational-ontology typing for the
PKG, on the strength of a 0.84 KM-task accuracy vs gUFO/DOLCE/no-FO.

**Evidence (see `bench/fo-km/RESULTS.md`):**

- **Haiku subject:** schema.org clearly leads; gUFO is the weakest formal FO; DOLCE-DUL
  edges the incumbent. This is the 0.84-vs-rest result the brief cites — it is real, and
  it is **scoped to the Haiku (cheap-tier) agent**.
- **Fable subject re-run (`sq-2m6zm.9`):** the headline dominance **does not reproduce**
  — schema.org and DOLCE-DUL *tie*; schema.org keeps only an expressibility-subset edge;
  gUFO stays clearly behind; and the no-FO incumbent becomes the *worst* arm. So the
  #1111 "fluency unlocks richer FOs" thesis is **half-confirmed**: true for DOLCE-DUL,
  false for gUFO.
- **Cross-run caveat, stated honestly:** absolute levels are not comparable across the
  two runs (Fable abstains far less and the frozen grader rewards a verbal style the
  cheap tier produced and Fable does not) — **only within-run ordering is valid**. The
  valid Fable-tier ordering is `schema.org = DOLCE-DUL > gUFO > no-FO`.

**Verdict: PARTIAL.** Keep **schema.org-as-top as the default** — it is never dominated,
it is the cheapest to maintain (soft `skos:closeMatch`, no reasoner dependency, SHACL-
and DL-safe), and the PKG is queried by *cheaper* tiers too, where the 0.84 dominance
stands. But **retire the claim that schema.org is uniquely best**: at Fable fluency it
merely ties DOLCE-DUL, and *any* FO overlay beats no-FO. This is already how
`sq-mztg8.5` resolved (ratify with a per-tier note) — this record ratifies that
resolution rather than reopening it.

**What would settle the residual:** the closure-prior (KGE) metric that would give FO a
*symbolic* payoff (not just prompt fluency) is **sign-unstable on synthetic slices**
(`sq-p5ro8` / `sq-0wo9e.9`) and needs a real schema-bearing KG on a canonical machine to
settle — deferred, correctly. Round-2 arm expansion (gist as the pragmatic middle, BFO)
is `sq-givgo`, correctly P3. No new FO bead is warranted; the existing two suffice.

## 5. Component: PROV-O as load-bearing provenance — **PARTIAL: substrate ADOPT, consumption UNPROVEN → build one thin consumer + measure**

**Claim under test (#1110):** provenance should be *load-bearing* — driving embedding
weights (USE 1), answer hedging (USE 2), and citations (USE 3), not just decorative.

**Evidence:** the substrate is real and enforced (§1): PROV-O lineage on every finding,
SHACL-mandatory source + confidence + assurance, machine-tier trust caps, DQV alignment.
External prior art (2026) independently converges on this being the right shape —
provenance-bearing memory and provenance verification are the current reliability levers
for KG-augmented LLMs. **But** all three *uses* are currently inert: nothing reads
provenance back. The load-bearing claim is therefore **designed, not demonstrated**.

**Verdict: PARTIAL.**

- **ADOPT the substrate** unconditionally — it is cheap, already shipped, SHACL-gated,
  and is the precondition for every reliability claim this direction could make. Keeping
  facts sourced-or-rejected is worth it on its own as an anti-hallucination discipline
  even before any consumer exists.
- **Do NOT yet claim provenance is load-bearing for output quality.** It is
  infrastructure. The honest status is "recorded and validated, not yet consumed".
- **Build exactly one thin consumer and measure it**, rather than all three at once. The
  cheapest, most falsifiable consumer is **USE 3 citations for the PKG-native tier**:
  render `[source]` provenance beside a canned-query answer. It needs only a renderer
  (the join is already in the canned queries) and it has a crisp, un-gameable metric —
  citation-resolution rate and count of fabricated citations, which should be 1.0 and 0
  by construction because SHACL already forbids dangling `cito:citesAsEvidence`. That
  turns "provenance is load-bearing" from an aspiration into a shipped, measured feature
  with the smallest possible surface. USE 1 (weight embeddings) is the most expensive
  and least certain (no in-tree KGE trainer, the closure-prior lift is itself
  sign-unstable per §4) — defer it. USE 2 beyond the static disclaimer needs the
  calibration harness (`sq-2489d.10`) as input and should wait for it.

**What would settle the residual:** the end-to-end token/outcome A/B of a
provenance-consuming KB vs the inert baseline (`sq-2489d.6`, Phase 7, open) is the honest
final arbiter of the whole load-bearing claim — but it is premature until at least one
consumer exists to measure. Ship the citation renderer first, then that A/B has something
to compare.

## 6. Component: literature-trawling to self-populate the KB — **PARTIAL / HOLD: keep the fixture pipeline; keep the live pilot maintainer-gated**

**Claim under test:** the agent should self-populate the KB by trawling literature
(CORE v3 / OpenAlex), extracting findings with a cheap model, grounding them to PKG
IRIs, and SHACL-gating the result.

**Evidence:** the fixture pipeline is real and correct (connector→extract-replay→
ground→emit→sidecar, SHACL-gated, machine-tier trust-capped). The *live* path
(`literature-live`, `LiveExtractor`) is wired but inert and maintainer-gated OFF
(`sq-tzars.9`, PR #1609). Grounding is deterministic (substring-of-abstract check +
DOI-resolves-to-`pkg:Source` check) — a genuine anti-hallucination guard, matching the
anchor-constrained-provenance shape the external prior art recommends.

**Verdict: PARTIAL / HOLD, and this is the honest place to *not* manufacture
enthusiasm.**

- The engineering is sound and the trust discipline (fail-closed licence capture,
  machine-tier confidence ceiling, `secx:Conjectured` cap, no dangling citations) is
  exactly right. **Keep the fixture pipeline** — it is the reproducible, CI-safe half.
- But the *value* of live trawling to a coding agent is **unquantified and its cost is
  real** (per-paper Haiku Batches spend; the automated public dump was already ABORTED
  per #1552 for good reasons). Self-populating the KB with paper findings is the
  speculative leg of this whole direction; it should not run at scale until the cheaper
  legs (§3 retrieval, §5 citations) have demonstrated the KB pays for itself.
- **Keep the live pilot maintainer-gated.** Do not un-gate it as part of this program.
  The right unlock condition is a measured answer to "does a literature-derived fact in
  the KB change an agent's answer for the better", which is downstream of the §5
  consumer and the `sq-2489d.6` A/B. Until then, live trawling is cost without a proven
  payoff — HOLD is the honest call, not REJECT (the machinery is valuable and correct),
  and not ADOPT (the benefit is unproven).

## 7. Component: confidence calibration — **PARTIAL: disclaimer shipped, calibration harness is the real gate**

The static UNCALIBRATED disclaimer is shipped (`sq-2489d.9`) — correct and honest, since
the confidences are hand-authored, not empirically calibrated. Promoting any answer to
"calibrated" requires the reliability-diagram harness (`sq-2489d.10`, OPEN, P2), which in
turn needs machine-tier findings to be non-vacuous — i.e. it depends on the §6 pilot
producing data. **Verdict: PARTIAL** — keep the disclaimer; do not build the calibration
harness yet, because with only hand-authored confidences it would be a vacuous
measurement. Order it *after* the pilot produces machine-tier findings. This matches the
existing rung-C decision; no change needed.

## 8. Summary of verdicts

| Component | Verdict | One-line reason |
|---|---|---|
| Self-built project KG (direction) | **PARTIAL ADOPT** | Retrieval-economics leg is measured-positive + shipped; self-evolving leg is speculative, gate it |
| PKG NL-tool routing | **ADOPT** | Cheap-model delegation still wins by an order of magnitude under Fable; route to a *cheap* model, not a strong one in-context |
| FO typing (schema.org 0.84) | **PARTIAL** | Keep schema.org default; the 0.84 dominance is Haiku-scoped — at Fable fluency it ties DOLCE-DUL; retire the uniqueness claim |
| PROV-O load-bearing | **PARTIAL** | Substrate ADOPT (shipped, SHACL-gated); consumption UNPROVEN — build one thin citation consumer + measure |
| Literature-trawling | **PARTIAL / HOLD** | Keep the fixture pipeline; keep the live pilot maintainer-gated until a cheaper leg proves the KB pays off |
| Confidence calibration | **PARTIAL** | Disclaimer shipped; calibration harness must wait for pilot data or it is vacuous |

The through-line: **the cheap, measured legs are worth adopting now; the expensive,
unproven legs (self-population, embedding-weighting) should stay gated behind a measured
payoff.** No component is a clean REJECT — the substrate is genuinely good — but two
components (literature-trawling at scale, embedding-weighting) are HOLDs whose enthusiasm
would be manufactured if presented as ADOPTs today.

## 9. Phased plan (each phase is a future bead for the orchestrator)

The prior program already spent most of the buildable beads (the Fable re-run, the FO
ratification, the DQV adoption, the disclaimer, the fixture pilot are all CLOSED). This
plan adds only what this fresh verdict newly justifies, and explicitly *reuses* the
existing open beads rather than duplicating them.

1. **[NEW] USE-3 citation renderer for the PKG-native tier** — render provenance
   `[source]` beside canned-query answers; metric = citation-resolution rate (target 1.0)
   and fabricated-citation count (target 0, by SHACL construction). Smallest falsifiable
   step that turns "provenance is load-bearing" into a shipped feature. Filed as a new
   bead under `sq-2489d`.
2. **[REUSE — existing, open] `sq-2m6zm.7`** — end-to-end exec-accuracy of a real
   cheap-model NL endpoint; the right place to firm the multi-hop accuracy gap from §3.
   Already in progress; no new bead.
3. **[REUSE — existing, open, do NOT start yet] `sq-2489d.6`** — end-to-end token/outcome
   A/B of the provenance-consuming KB vs the inert baseline. Gate on phase 1 landing so
   it has a consumer to measure. No new bead; add a dependency note.
4. **[REUSE — existing, open, gated] `sq-2489d.10`** — confidence-calibration
   reliability-diagram harness; keep gated on machine-tier pilot data (would be vacuous
   otherwise). No new bead.
5. **[HOLD — maintainer decision] live literature-trawling at scale** — remains
   maintainer-gated (`sq-tzars.9` shipped OFF); un-gate only after phases 1 + 3 show the
   KB pays off. No new bead; recorded as a documented HOLD.
6. **[REUSE — existing, deferred] `sq-p5ro8` / `sq-givgo`** — FO closure-prior on a real
   KG and FO round-2 arm expansion; correctly P3, needs a canonical machine. No new bead.

Net new build surface from this fresh verdict: **one bead** (phase 1). Everything else is
already tracked; the value of this record is the *honest scoping* — telling the
orchestrator which of the existing beads to advance, which to gate, and which claim to
retire.

## 10. Open questions that genuinely need the maintainer

1. **Does a self-populated KB actually change an agent's output for the better?** This is
   the load-bearing unknown for §5–§6. It is answerable only by `sq-2489d.6` *after* a
   consumer exists — but the maintainer may have a prior on whether it is worth the
   Batches spend to find out, given the automated-dump abort (#1552) already signalled
   caution.
2. **Per-tier FO policy.** This record and `sq-mztg8.5` keep schema.org default because
   the PKG serves cheap tiers too. If the maintainer expects the PKG to be queried
   *predominantly* by Fable-tier agents going forward, DOLCE-DUL becomes an equal
   contender and the "second arm to watch" (`sq-givgo`). This is a routing-mix judgment
   only the maintainer can make.
3. **Is the frozen deterministic grader the right instrument at Fable fluency?** The
   FO-KM absolute scores fell across all arms substantially because the grader rewards a
   verbal style the cheap tier produced and Fable does not (§4). Within-run ordering is
   valid, but if future re-runs are Fable-subject, a grader that scores *semantic*
   correctness (not verbatim enumeration) may be worth the maintainer's call before
   spending on more arms.
