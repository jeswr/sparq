> 🤖 SPARQ agent — landing Kern PR, hygiene by **GPT-5.6**

# vectors: RDF 1.2 quoted-triple visibility as an off-by-default ablation axis; fix empty-string verbalisation of quoted triples

## What

Three changes to the opt-in `structure`/`kge` measurement stack (default build untouched, zero new
dependencies; nothing outside `sparq-vectors` changes):

1. **Bugfix (unflagged):** `grounding::render_object` rendered an RDF 1.2 quoted-triple object as
   the **empty string** — silent data loss in every NL-string / subgraph-text grounding over RDF
   1.2 data. It now renders the reconstructed `<<( s p o )>>` term (nested terms included,
   depth-capped by the existing dict reconstruction; oxrdf's `Display` provides the RDF 1.2
   triple-term syntax). Regression-tested: no grounding fact may have an empty object. Happy to
   split this into its own PR if preferred.

2. **`TermScope` — the quoted-terms ablation axis (default OFF, byte-stable):** the trainer, eval,
   and negative sampler each carried a private `is_entity` that excluded `TermParts::Triple`, so a
   graph's statement-level structure (`rdf:reifies` edges, content-addressed shared quoted-term
   nodes) was structurally invisible to the KGE embedding — the axis could not even be measured.
   The reifier node itself (an IRI/blank node) was already embeddable; what was dropped is the
   `rdf:reifies` edge to the quoted term, leaving the reifier a disconnected metadata stub.
   - One shared `is_embeddable(graph, id, scope)` replaces the three copies.
     `TermScope::IriBlank` — the default in **every** constructor and preset — reduces to the
     identical match, so all existing baselines are **byte-identical** (see below).
   - `TermScope::Embeddable` (opt-in per ablation arm via `TrainConfig::term_scope`) admits quoted
     terms to entity space, with **sort-preserving negative corruption**: a quoted-term slot is
     corrupted only from the quoted pool and an atomic slot only from the atomic pool (a
     cross-sort negative is detectable from term class alone and would pollute the training
     margin). Quoted candidates bypass the type-constraint class filter — a quoted term carries no
     `rdf:type`, and statements have no class discipline yet.
   - **Split membership and the ranking pool stay atomic under BOTH scopes** — the eval population
     is scope-invariant, so paired ON/OFF deltas isolate the training-side visibility effect
     rather than comparing rankings over different candidate sets.

3. **Measurement plumbing:** a `synthetic_rdf12` eval slice (deterministic in seed, N-Triples,
   honesty-guarded like the existing gUFO/relational/provenance slices: shared quoted-term
   reifier hubs, noise reifications, and ≥15% overlapping-but-uncorroborated decoy source pairs so
   claim overlap alone cannot separate the `ex:corroborates` target) and `run_quoted_ablation`
   (paired per-seed ON−OFF deltas with common random numbers, mirroring `run_weight_ablation`;
   exact-zero delta on quote-free graphs). `AblationCell` gains a `quoted_terms` field so the 2×2
   matrix reports the axis explicitly (always `false` under the presets, mirroring the dormant
   `gufo_prior` axis).

## Baseline safety (the default-off byte-stability guarantee)

Three independent layers:

1. **Structural:** `TermScope::IriBlank` reduces `is_embeddable` to the exact former match arms;
   the sampler's quoted pool is empty under it, so the draw loop, PRNG stream, and rejection
   sequence are bit-identical; `Splits` and the ranking pool are untouched. No float path, PRNG
   constant, or iteration order changes when OFF. (`grep -n "fn is_entity"
   crates/sparq-vectors/src` now returns nothing; `TermScope::Embeddable` is constructed only in
   `run_quoted_ablation` and tests.)
2. **In-tree regression tests, bracketing the change from both directions:**
   - `invisible_reifications_change_nothing_when_scope_is_off` — adding `rdf:reifies` lines to a
     graph under the default scope leaves splits, ranking pool, **model bytes (bit-equal
     `entity_emb`/`rel_emb`)**, loss curve, and filtered metrics identical (the two variants share
     one parsed dictionary, so the comparison is exact and parser-independent).
   - `quoted_ablation_is_exactly_zero_on_quote_free_graphs` — the ON arm on a quote-free graph is
     bit-equal to OFF and the paired delta is exactly `0.0`.
3. **Cross-commit pin (`examples/kge_pin.rs`):** prints a deterministic digest over
   `(epoch_loss, entity_emb, rel_emb, ablation metrics)` for the three pre-existing quote-free
   slices at pinned seeds {1,2,3}. Run on the merge-base and this head (same box/toolchain/thread
   count): the outputs must be — and are — identical (`diff` empty; verified before opening this
   PR).

## What this PR deliberately does NOT do

- **No compositional statement encoder:** the ON arm embeds a quoted term as a *node* (its
  `rdf:reifies` edges and content-addressed hub-sharing become visible structure); access to the
  quoted `(s, p, o)` content itself is a separate, measurement-gated follow-up PR.
- **No widening of the eval ranking pool** (rejected: incomparable filtered ranks across arms).
- **`vec:` VALUES rows still drop triple-term neighbours** (`rewrite::term_to_ground`); the
  vendored spargebra exposes `GroundTerm::Triple`, so this is mappable, but it is query-surface
  behaviour outside this PR's measurement scope — deferred to a follow-up.
- **No accuracy claim and no committed numbers.** This PR makes the quoted-terms axis measurable;
  whether visibility lifts anything is exactly what the paired ablation exists to measure, and
  adoption of any default stays gated on a measured, multi-seed, paired-delta result on the
  asymmetric model — the crate's established `SamplingMode`/`WeightMode` discipline.

## Context: why

This comes out of the **Kern (kernel-of-truth) research programme** — an agent-driven effort
evaluating whether ontological discipline (including statements-about-statements held without
assertion, i.e. RDF 1.2 reification) earns measurable lift in KGE link prediction on this stack.
Today `sparq-vectors` structurally cannot see quoted triples, so that question cannot be asked.
The change is useful to the vectoriser independently of that programme (RDF 1.2 data is silently
truncated today — the grounding bugfix corrects real data loss), which is why it is proposed
upstream rather than kept on a fork. Authored by an AI agent (Fable) under the programme's
review-everything-before-commit workflow; treat with the usual reviewer skepticism and feel free
to request splits or renames.

---

*@jeswr will review; expect a delay — active review Wed–Fri.*
