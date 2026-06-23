<!-- [OPUS-4.8] sq-mztg8.1 (FO-LLM-bridge Phase 3; design research/fo-llm-bridge.md §2.3/§3.3/§6,
epic sq-mztg8). 🤖 SPARQ agent — the MEASURED record for the read-path URI-hiding A/B. This is the
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

## What was measured — and the honest split

The headline K4 question is an **answer-accuracy** question that needs a **real small model**
in a closed NL→NL loop. **That half is UNMEASURED** on the work box: there is no API key, and
only the `sparq-nlq` record/replay fixtures exist (the same unmeasured-accuracy gap the design
flags in §2.3). Reporting an accuracy delta from a model-free proxy would repeat the exact
mistake this project has reversed **three times** (the char/byte-proxy token claims). So this
record measures the two properties that ARE measurable deterministically, and registers the
accuracy verdict as **UNMEASURED**:

| Half | Status | Why |
|---|---|---|
| **Soundness** (does hiding preserve answer identity?) | **MEASURED** ✅ | model-free; a label-collision is a lost identity regardless of any model |
| **Presentation cost + coverage** (how the labelled view reads) | **MEASURED** ✅ | model-free; char counts + label-coverage over the real PKG |
| **Accuracy** (does hiding help a real model answer?) | **UNMEASURED** ⛔ | needs a real-model NL→NL fan-out; no API key on the work box |

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
2. **The accuracy half (K4) stays OPEN — registered UNMEASURED.** Whether the (sound, larger)
   labelled view helps or hurts a real small model's answer accuracy is the actual K4 verdict and
   is **not** decided here. The mechanism + the soundness gate are ready; the real-model NL→NL
   fan-out is the blocking dependency (tracked — see the follow-up bead).
3. **Soundness gate shipped.** `AbReport::fidelity_preserved()` is the hard gate: a hide that
   collides two answer IRIs onto one label is a lost identity and the harness exits non-zero. A
   future PKG that introduces a duplicate label will fail the bench loudly rather than silently
   serve a wrong answer.

## Pre-registered null (so the UNMEASURED accuracy verdict is honest)

Per design §5 (falsifiable, not assumed-better): the prior is **NEUTRAL** — URI-hiding is
*expected to make no difference* to answer accuracy until measured. The mechanism is built so the
null can be returned honestly; nothing here claims hiding helps. If/when the real-model A/B runs
and the hidden view does **not** match-or-beat the visible path on accuracy, K4 fires and the
read-path keeps **grounding only** (no hiding), per the design's kill-criterion.
