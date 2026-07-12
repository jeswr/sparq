# feat(sparq-vectors): read-only UFO/gUFO priors reader — wire the dormant `gufo_prior` axis with a UFO-provable, answer-safe disjointness + subsumption mask

**Agent:** Kern (Fable) · branch `kern/ufo-priors`

## Why

The structure-aware-vectorisation design (epic sq-0wo9e, `research/structure-aware-vectorisation.md` §2 row "gUFO rigidity and roles", §9.5) lists the gUFO prior as the *optional/last* prior; `taxonomy.rs` deferred it, and the eval harness has carried a hard-wired-false `AblationCell::gufo_prior` axis since P6. This PR wires that dormant axis with the narrowest sound slice: a **read-only** reader of UFO/gUFO structure whose only product is *provable* knowledge, fed into the existing answer-safe `DisjointnessOracle` seam.

## What

- **`crates/sparq-vectors/src/ufo_priors.rs` (new, `structure` feature — off by default).** Mines, without writing anything back: gUFO **meta-types** (`gufo:Kind/SubKind/Role/Phase/Category/RoleMixin/PhaseMixin/Mixin`) and their definitional **rigidity**; **identity providers** (the unique `gufo:Kind` in a class's reflexive `subClassOf` ancestry); **ontological natures** (`gufo:FunctionalComplex/Collection/Quantity/Relator/Quality/IntrinsicMode/ExtrinsicMode/Event/Situation` via `subClassOf` reachability); and instance-level **mediation/inherence** witnesses (`gufo:mediates` ⇒ relator, `gufo:inheresIn` ⇒ aspect — exactly gUFO's declared `rdfs:domain` entailments). Canonical namespace by default; non-canonical namespaces are an explicit caller declaration (no silent fallback).
- **UFO-provable disjointness** (three rules, all fail-closed): (1) two distinct kinds not related by `subClassOf` are disjoint (UFO's kind partition — every endurant instantiates exactly one kind); (2) propagation through unique identity providers (classes with zero/multiple kind ancestors join no propagation); (3) distinct leaf ontological natures are disjoint (UFO's taxonomy-of-individuals partition; a class reaching contradictory natures is excluded entirely). Deliberately **not** used: rigidity (not a disjointness), categories/mixins (classify across kinds by design), instance witnesses (never lifted to class level).
- **UFO-proven subsumption** (`proven_subsumptions()`): endurant-meta-typed classes ⊑ `gufo:Endurant`; nature-attached classes ⊑ their fixed gUFO upper chain — emitted only over gUFO terms already in the dictionary (read-only, no term minting).
- **`DisjointnessOracle::absorb_proven_pairs`** (taxonomy.rs): the explicit seam with a documented proven-only soundness contract; `UfoPriors::augment_oracle` feeds it, so the serve-time `mask_candidates` hard mask becomes UFO-aware while staying answer-safe.
- **The `gufo_prior` axis wired (eval.rs), DEFAULT OFF.** `EvalConfig::gufo_prior` (default `false`) + `EvalConfig::gufo_ns`. When ON, the UFO-augmented oracle is applied as the serve-time hard mask inside the filtered ranking: a candidate whose `rdf:type` is provably disjoint from the relation's declared `rdfs:domain`/`rdfs:range` class is dropped. Training is untouched (serve-time only; train-time repulsion stays a tracked follow-up). `AblationCell::gufo_prior` now reports the switch.

## Answer-safety & the default-off proof

- **Answer-safe:** the mask removes only candidates a UFO/OWL proof excludes; on a UFO-consistent graph a true answer's types can never be disjoint from the relation's declared signature, and structurally the held-out answer is skipped before the mask, so no metric can lose its true answer even on an inconsistent graph. End-to-end property test: with the same trained model, per-cell MRR/Hits@k(ON) ≥ (OFF) and query counts identical — and the mask must strictly bite on the gUFO slice.
- **Baselines byte-identical when OFF:** with `gufo_prior = false` (the default) the mask is *never constructed* — the candidate loop's only change is an `if let Some(..)` on a `None`. Tests assert (a) the default is off, (b) OFF runs are deterministic, (c) ON == OFF **byte-identically** (exact `f64` equality) on a gUFO-free graph (the honest no-op, mirroring the provenance-weighting convention). The default `sparq-vectors` build compiles zero UFO-prior code (`structure` feature, off by default) and gains no dependency.

## Tests / gates

- 9 new `ufo_priors` unit tests (kind partition, provider propagation, fail-closed guards: subClassOf-related kinds, ambiguous identity, contradictory natures; mediation/inherence witnesses; subsumption products; oracle augmentation + mask; empty-graph no-op; determinism).
- 1 new taxonomy seam test (`absorb_proven_pairs`), 3 new eval tests (byte-identical no-op, default-off determinism, answer-safety/monotonicity).
- `cargo build/test -p sparq-vectors --features kge`, `cargo fmt`, `cargo clippy -p sparq-vectors --features kge --all-targets -- -D warnings` — all green.

No benchmark numbers are stated anywhere: whether the mask *lifts* MRR on a real dataset is empirical and stays behind the (default-off) ablation axis, per the epic's measurement-gated-adoption posture.

---
*Review timing: this is genuine upstream sparq work — review on the normal upstream cadence; nothing downstream (Kern experiments) gates on it, and no Kern-side code depends on this landing.*

🤖 Generated with [Claude Code](https://claude.com/claude-code)
