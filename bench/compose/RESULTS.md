<!-- [OPUS-4.8] sq-mztg8.1 (model-free fidelity/cost half) + sq-3pb7f (K4 real-model accuracy half)
(FO-LLM-bridge Phase 3; design research/fo-llm-bridge.md §2.3/§3.3/§4.3/§6, epic sq-mztg8).
🤖 SPARQ agent — the MEASURED record for the read-path URI-hiding A/B. This is the
AGENTS.md-sanctioned numeric home (bench/ is exempt from check-no-perf-numbers.py); no user-facing
markdown repeats these figures. Written while Fable unavailable; flag for re-review when Fable
returns. -->

# RESULTS — read-path URI-hiding compose A/B (FO-LLM-bridge Phase 3)

> 🤖 **SPARQ agent.** Measurement record for **bead sq-mztg8.1** — the read-path URI-hiding
> "compose" step + the closed NL→NL A/B over the real **Project-Knowledge-Graph (PKG)**.
> `bench/` is the AGENTS.md-sanctioned home for measured figures; the numbers below live HERE
> and are not repeated in any user-facing doc.

## The question (open question K4, design §7)

When a read-path serves an answer-row binding to the agent, should it show the **raw IRI**
(the NL→SPARQL incumbent) or **hide it behind a human label** (the §3.3 facade)? Does hiding
URIs **help or hurt**? K4 is pre-registered as a falsifiable kill-criterion: *if a real-model
NL→NL A/B over the hidden-URI loop does not match-or-beat the URI-visible path on answer
accuracy, URI-hiding is a no-op complexity cost and is dropped.*

**Both halves are now MEASURED.** The model-free half (soundness + presentation cost) is below;
the **accuracy** half — the headline K4 question — is resolved in
[**§ K4 accuracy A/B**](#k4-accuracy-ab-real-model-resolves-the-headline-question) (sq-3pb7f),
the real-model NL→answer fan-out the original sq-mztg8.1 record left open.

## What was measured — and the honest split

The headline K4 question is an **answer-accuracy** question that needs a **real small model**
in a closed NL→NL loop. The original sq-mztg8.1 record measured only the two model-free
properties and registered accuracy UNMEASURED (no API key, only record/replay fixtures). The
follow-up **sq-3pb7f** ran the real-model NL→answer fan-out via tagged Claude Code sub-agents
(real-model telemetry, no API key — the pkg-dogfood mechanism), so **all three halves are now
MEASURED**:

| Half | Status | Why |
|---|---|---|
| **Soundness** (does hiding preserve answer identity?) | **MEASURED** ✅ | model-free; a label-collision is a lost identity regardless of any model |
| **Presentation cost + coverage** (how the labelled view reads) | **MEASURED** ✅ | model-free; char counts + label-coverage over the real PKG |
| **Accuracy** (does hiding help a real model answer?) | **MEASURED** ✅ (sq-3pb7f) | real-model NL→answer A/B, 35 paired questions, blinded answering + judging; see [§ K4 accuracy A/B](#k4-accuracy-ab-real-model-resolves-the-headline-question) |

## Method (deterministic, model-free)

`cargo run -p sparq-vectors --features compose --example compose_ab` loads the in-tree PKG
(`crates/sparq-kb/ingest/{pkg-instances,agents-findings}.ttl`), forms one answer table per
`rdf:type` class (the shape of a "list all X" agent question), and composes each binding under
**both** views (`UriVisible` = raw IRI, `UriHidden` = label + echo). It then measures, per class
and over the whole PKG:

- **round-trip fidelity** — does every shown label map back to exactly one IRI? (a collision is
  a lost identity that would silently change the answer);
- **coverage** — the fraction of answer IRIs resolved to a real/local-name label vs falling open
  to the raw IRI;
- **char_ratio** — `chars_hidden / chars_visible`, how much shorter/longer the labelled view is.

These figures are **MEASURED ON THIS WORK BOX, NON-CANONICAL** — deterministic over the
checked-in PKG, but a fidelity/cost sanity check, not a published accuracy benchmark.

## The measured result (work-box, NON-CANONICAL)

Over the checked-in PKG (**1487 typed answer IRIs** across 7 classes):

| Metric | Value | Reading |
|---|---|---|
| **fidelity preserved** | **true** (0 collisions / 1487) | URI-hiding is **SOUND** here — every label re-identifies to exactly one IRI |
| **coverage** | **1.000** | every typed entity has a real `rdfs:label`/`skos:prefLabel` — nothing falls open to a raw IRI |
| **char_ratio (whole PKG)** | **≈ 2.38** | the labelled view is **~2.4× LARGER to read** than the raw IRIs |

Per-class `char_ratio` splits the story:

| Class | n IRIs | char_ratio | note |
|---|---|---|---|
| Technique | 5 | ≈ 0.43 | short prefLabels ≪ IRIs → hiding is **more compact** |
| Concept | 3 | ≈ 0.50 | short prefLabels → more compact |
| Source / Document / Expression | 70 each | ≈ 0.89 | slightly more compact |
| Finding | 11 | ≈ 1.86 | finding labels are sentences → **larger** |
| Task | 1258 | ≈ 2.78 | task titles are long descriptions ≫ short `bd`-id IRIs → **much larger** |

## Verdict (NON-SYCOPHANTIC)

1. **URI-hiding is SOUND on this PKG, not a token-saver.** The hidden view loses **zero**
   answer identity (0/1487 collisions, coverage 1.0), so it is *safe* to A/B for accuracy. But
   it is **NOT** a token reduction: because PKG labels (Task/Finding descriptions) are far longer
   than the short opaque IRIs (`…#task-<id>`), hiding **inflates** the read size ~2.4× overall.
   The naive "hiding URIs makes the answer cheaper to read" assumption is **false here** — only
   the short-prefLabel classes (Concept/Technique) get more compact. This is exactly the kind of
   proxy-reversal the project keeps finding: measure, don't assume.
2. **The accuracy half (K4) is now MEASURED — see the next section.** Whether the (sound, larger)
   labelled view helps or hurts a real model's answer accuracy is the actual K4 verdict; sq-3pb7f
   resolves it with a real-model NL→answer A/B.
3. **Soundness gate shipped.** `AbReport::fidelity_preserved()` is the hard gate: a hide that
   collides two answer IRIs onto one label is a lost identity and the harness exits non-zero. A
   future PKG that introduces a duplicate label will fail the bench loudly rather than silently
   serve a wrong answer.

## K4 accuracy A/B (real-model) — resolves the headline question

> 🤖 **sq-3pb7f.** The accuracy half the sq-mztg8.1 record left open. Harness +
> reproduction: [`k4/README.md`](k4/README.md); raw evidence in `k4/answers_*.json` +
> `k4/scores_*.json`; computed result in `k4/k4_result.json`. **Work-box, NON-CANONICAL**
> (real-model sub-agent telemetry, single-run, N=35 paired). Flag for re-review when Fable
> returns.

**Method.** 35 frozen NL questions, each whose correct answer depends on grounding to the
right PKG entity, were rendered over the **real PKG** under both views by the real `compose`
module (`compose_k4` example). A blinded sub-agent answered each (question, arm) seeing ONLY
its arm's rows — never the arm name, the gold, or the word "URI-hiding" (35 × 2 = **70 answer
runs**). A second blinded sub-agent judge graded each answer vs a graph-derived gold
(1.0 correct / 0.5 partial / 0.0 incorrect), on a **shuffled** order so arm cannot leak. The
battery spans six kinds chosen to find where hiding helps OR is a no-op — including a
`dep-count` **null control** (the answer is a COUNT, identical info in both arms) and the
punctuation-heavy `ZK/MPC claim + circuit discipline` label (the known failure mode).

**The measured result (work-box, NON-CANONICAL):**

| Arm | Accuracy (mean score) | — |
|---|---|---|
| **`UriVisible` (RAW, raw IRIs)** | **0.643** | the incumbent |
| **`UriHidden` (labels + echo)** | **1.000** | the facade |

| Paired statistic | Value |
|---|---|
| mean Δ (HIDDEN − RAW) | **+0.357** |
| paired HIDDEN-wins / RAW-wins / ties | **16 / 0 / 19** |
| exact two-sided sign test (16–0) | **p ≈ 3.05e-5** |

**HIDDEN matches-or-beats RAW on every one of the 35 questions (never loses), and strictly
beats it on 16.** But the per-kind breakdown is the load-bearing, non-sycophantic part — the
win is **entirely concentrated where the IRI local name carries no meaning**:

| Kind | n | RAW acc | HIDDEN acc | Δ | reading |
|---|---|---|---|---|---|
| `task-title` (opaque `task-sq-XXXX` id) | 8 | **0.00** | 1.00 | **+1.00** | the whole win lives here |
| `disambiguate-finding` (verbatim req.) | 6 | 0.42 | 1.00 | +0.58 | RAW returns a slug/IRI, not the asked text |
| `punct-concept` (punctuation label) | 1 | 0.50 | 1.00 | +0.50 | RAW hedges the `+`/`/`-heavy label |
| `about-topic` (semi-informative `topic-…`) | 11 | **0.955** | 1.00 | +0.05 | RAW reconstructs the answer from the slug |
| `technique-name` (semi-informative `surface-…`) | 5 | **1.00** | 1.00 | **0.00** | slug ⇒ RAW already perfect |
| `dep-count` (**null control**, identical info) | 4 | **1.00** | 1.00 | **0.00** | calibrates judge bias — exactly 0, as it must |

## Verdict on K4 (NON-SYCOPHANTIC) — kept, conditionally

**K4 does NOT fire. URI-hiding is kept, not dropped — but the "why" is sharper than "hiding
helps."** Per the pre-registered kill-criterion (hide is dropped *only if* it fails to
match-or-beat visible on accuracy), the hidden view **matches-or-beats on 35/35** and **never
regresses a single question**, so the criterion is not met and hiding survives.

The honest mechanism, though, is **not** that labels are intrinsically better to reason over —
it is that **a raw IRI whose local name is an opaque id (`kb:task-sq-XXXX`) starves the agent of
the answer, and the label restores it.** Where the local name is already semi-informative
(`topic-merge-discipline`, `surface-genai-retrieval`), the agent reconstructs the answer from
the slug and RAW ties HIDDEN (Δ ≈ 0). Where the binding is a pure-id node, RAW scores **0.00**
and hiding is decisive. So:

- **Hiding is worth pursuing on the read-path** — it is a real, significant, never-harmful
  accuracy gain (+0.357 overall, p ≈ 3e-5), *not* the "no-op complexity cost" the null
  predicted, and **not** a dead end.
- **The gain is conditional on label informativeness, not universal.** It is large exactly for
  the PKG's opaque-id entities (Tasks — the bulk of the graph) and ~zero for the well-named
  ones. The right framing for the SKILL is therefore "**hiding rescues opaque-id bindings**,"
  not "labels beat IRIs everywhere."
- **It is a legibility/accuracy tool, still not a token-saver.** This accuracy win comes at the
  measured ~2.4× read-cost (above): the very `task-title` class that drives the accuracy gain is
  also the one whose long titles inflate `char_ratio` to ~2.78. Hiding **buys accuracy on
  opaque ids with tokens**, it does not save them — consistent with the model-free half and with
  the project's prior token-proxy reversals.

This resolves K4 (`research/fo-llm-bridge.md` §4.3): the read-path **keeps the URI-hiding
facade**, scoped to the case it demonstrably helps (opaque-id bindings), and the design's
"grounding only" fallback is not triggered.
