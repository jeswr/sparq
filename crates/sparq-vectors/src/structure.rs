//! Structure-aware vectorisation **preprocessing + sampling-logic** layer (P0).
//!
//! [OPUS-4.8] sq-0wo9e.1 (epic sq-0wo9e; design `research/structure-aware-vectorisation.md`
//! §5.A + §2 row "domain/range"). This module is the **lowest-risk, additive** slice of the
//! structure-aware-vectorisation epic. It is **opt-in** (the `structure` cargo feature, off by
//! default) and changes **nothing** in the default `sparq-vectors` build or the core engine.
//!
//! # What this is — and, honestly, what it is NOT
//!
//! `sparq-vectors` is a vector-**search** + **import** surface: embeddings are produced
//! **out-of-process** (see [`crate::embed`]) and the crate has **no KGE training path** — no
//! TransE / ComplEx / RotatE, no triple-scoring, no learning loop, no negative-sampling consumer.
//! P0 of the design ("restrict KGE *corruptions* to type-valid entities") therefore presupposes a
//! trainer that does not exist in-tree. Rather than build a large new training subsystem blind
//! (the design review #996 may reshape it), this module ships the two **buildable, additive**
//! halves of P0 as reusable primitives an out-of-process trainer (or a future in-tree KGE block)
//! consumes:
//!
//! 1. **closure-before-vectorise** ([`close_for_vectorise`] / [`materialise_closure`]) — run the
//!    `sparq-reason` closure ([`Profile::Rdfs`] or [`Profile::OwlRl`]) over the graph **before** the
//!    vectoriser sees it, so entailed type / `subClassOf` / domain / range triples are *materialised
//!    facts* the encoder, the type extractor, and the negative sampler all read.
//! 2. **type-constrained negative sampling** ([`TypeConstraints`] + [`NegativeSampler`]) — the
//!    established Krompass et al. 2015 technique: corrupt a positive triple's head only to
//!    entities whose type lies in the predicate's **domain**, its tail only to entities in the
//!    predicate's **range**. The constraints are read from `sparq-introspect` (declared
//!    `rdfs:domain`/`rdfs:range` **and** observed domain/range histograms) over the **closed**
//!    graph.
//!
//! The KGE trainer that *consumes* these negatives is the deferred remainder (bead noted in the
//! crate README / SKILL). This module trains nothing and serves no exact answer.
//!
//! # The on/off ablation hook (load-bearing, design §6.B)
//!
//! Every prior in this epic ships **behind an on/off ablation** and is adopted **only on measured
//! lift** — no accuracy claim is made in advance. [`SamplingMode`] is that switch:
//! [`SamplingMode::Unconstrained`] reproduces vanilla uniform corruption; [`SamplingMode::TypeConstrained`]
//! applies the domain/range restriction. A harness runs the *same* sampler in both modes and
//! compares Hits@k / MRR downstream. **No benchmark numbers exist; none are claimed here.**

use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Id, TermParts};
use sparq_core::Graph;
use sparq_introspect::Introspection;
use sparq_reason::Profile;

/// `rdf:type` — the predicate whose objects are an entity's declared classes.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Outcome of [`materialise_closure`] / [`close_for_vectorise`]: the closed [`Graph`] plus a
/// small, **non-canonical** report of how many triples the closure added. The counts exist so a
/// caller can log / assert the closure ran (e.g. "every closed triple is visible to the encoder")
/// — they are *not* a performance or quality metric and must never be baked into docs.
pub struct ClosedGraph {
    /// The graph after materialisation: asserted triples **plus** every entailed triple of the
    /// chosen [`Profile`]. This is the graph the vectoriser, the type extractor, and the negative
    /// sampler should read.
    pub graph: Graph,
    /// Triples present before materialisation.
    pub asserted_triples: usize,
    /// Triples added by the closure (`entailed_triples` ⩾ 0; `0` ⇒ the graph was already closed
    /// under the profile, e.g. it has no schema axioms to fire on).
    pub entailed_triples: usize,
    /// The entailment regime that was materialised.
    pub profile: Profile,
}

impl ClosedGraph {
    /// Total triples after materialisation (`asserted_triples + entailed_triples`).
    pub fn closed_triples(&self) -> usize {
        self.asserted_triples + self.entailed_triples
    }
}

/// Materialise the `profile` closure over an **already-parsed** `(dict, triples)` pair, then
/// build the closed [`Graph`]. This is the closure-before-vectorise step (design §5.A): the
/// reasoner is sound and complete for its profile, so after this call every entailed type /
/// `subClassOf` / domain / range fact is a *real triple* the downstream vectorisation inputs see —
/// not just the asserted subset.
///
/// `dict`/`triples` come from [`Graph::parse_to_triples`]. Ownership is taken because the closed
/// graph is rebuilt from them via [`Graph::from_parts`].
///
/// ```
/// use sparq_core::Graph;
/// use sparq_reason::Profile;
/// use sparq_vectors::structure::materialise_closure;
///
/// let ttl = "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
///            @prefix ex: <http://ex/> .\n\
///            ex:Dog rdfs:subClassOf ex:Animal .\n\
///            ex:rex a ex:Dog .";
/// let (dict, triples) = Graph::parse_to_triples(ttl, "turtle").unwrap();
/// let closed = materialise_closure(dict, triples, Profile::Rdfs);
/// // `ex:rex a ex:Animal` is now an entailed, materialised triple.
/// assert!(closed.entailed_triples >= 1);
/// ```
pub fn materialise_closure(
    mut dict: sparq_core::dict::Dict,
    mut triples: Vec<[Id; 3]>,
    profile: Profile,
) -> ClosedGraph {
    let asserted_triples = triples.len();
    let entailed_triples = sparq_reason::materialize(profile, &mut dict, &mut triples);
    let graph = Graph::from_parts(dict, triples);
    ClosedGraph {
        graph,
        asserted_triples,
        entailed_triples,
        profile,
    }
}

/// Parse RDF `text` of the given `format`, then [`materialise_closure`] under `profile`
/// ([`Profile::Rdfs`] or [`Profile::OwlRl`]) — the convenience entry point for the whole
/// closure-before-vectorise step from a serialised source.
///
/// Returns `Err` with the parser's message on a parse failure (the closure step never fails:
/// materialisation only *adds* triples).
pub fn close_for_vectorise(
    text: &str,
    format: &str,
    profile: Profile,
) -> Result<ClosedGraph, String> {
    let (dict, triples) = Graph::parse_to_triples(text, format)?;
    Ok(materialise_closure(dict, triples, profile))
}

/// Per-predicate **type constraints** mined from a (closed) graph via `sparq-introspect`:
/// the set of class IRIs admissible as a predicate's subject (domain) and object (range), and a
/// per-entity type membership map. This is the input to [`NegativeSampler`].
///
/// "Admissible domain/range" unions the **declared** `rdfs:domain`/`rdfs:range` with the
/// **observed** domain/range histograms `sparq-introspect` mines from usage — declared axioms are
/// often absent or wrong in real data, so the observed classes broaden coverage. A predicate with
/// *no* known domain (or range) classes is treated as **unconstrained** on that side (every
/// entity is a valid corruption), so the sampler never deadlocks on a missing schema — it simply
/// degrades to uniform corruption for that side, which is the honest fail-open behaviour.
pub struct TypeConstraints {
    /// `rdf:type` of each entity id, as the **interned class ids** that entity carries. Entities
    /// with no `rdf:type` are absent (their lookup yields an empty set → unconstrained).
    entity_types: FxHashMap<Id, FxHashSet<Id>>,
    /// For each predicate id: the class ids admissible as its **subject** (domain). Empty ⇒
    /// unconstrained domain.
    predicate_domain: FxHashMap<Id, FxHashSet<Id>>,
    /// For each predicate id: the class ids admissible as its **object** (range). Empty ⇒
    /// unconstrained range.
    predicate_range: FxHashMap<Id, FxHashSet<Id>>,
}

impl TypeConstraints {
    /// Mine type constraints from `graph` (which should already be **closed** — call
    /// [`materialise_closure`] first so entailed `subClassOf`/type/domain/range facts are present).
    ///
    /// Builds the `sparq-introspect` [`Introspection`] once and reads its declared + observed
    /// domain/range histograms; the per-entity `rdf:type` map is read directly from the graph's
    /// `rdf:type` triples (so it includes every entailed type, not just a sampled subset).
    pub fn mine(graph: &Graph) -> TypeConstraints {
        let introspection = Introspection::build(graph);
        Self::from_introspection(graph, &introspection)
    }

    /// Mine type constraints from a graph and a **pre-built** [`Introspection`] (avoids a second
    /// `Introspection::build` when the caller already has one). The introspection must have been
    /// built from the *same* `graph` — class/predicate IRIs are resolved back to ids via the
    /// graph's dictionary.
    pub fn from_introspection(graph: &Graph, introspection: &Introspection) -> TypeConstraints {
        let dict = &graph.dict;

        // Resolve the rdf:type predicate id (NO_ID-equivalent if the graph never uses it → the
        // entity_types map is simply empty and every entity is unconstrained).
        let rdf_type_id = graph.id_of(&type_term());

        // Per-entity rdf:type set, read from the (closed) graph's rdf:type triples directly so it
        // reflects every entailed type. iter_ids is subject-sorted; we just scan once.
        let mut entity_types: FxHashMap<Id, FxHashSet<Id>> = FxHashMap::default();
        if let Some(type_pred) = rdf_type_id {
            for [s, p, o] in graph.iter_ids() {
                if p == type_pred {
                    entity_types.entry(s).or_default().insert(o);
                }
            }
        }

        // Resolve a class IRI string to its interned id; classes absent from the dict are skipped
        // (they can never be the type of any entity in this graph, so they cannot constrain).
        let class_id = |iri: &str| -> Option<Id> {
            let id = dict_iri_id(dict, iri);
            id.filter(|&i| i != sparq_core::dict::NO_ID)
        };

        let mut predicate_domain: FxHashMap<Id, FxHashSet<Id>> = FxHashMap::default();
        let mut predicate_range: FxHashMap<Id, FxHashSet<Id>> = FxHashMap::default();

        for pred in &introspection.predicates {
            let Some(pred_id) = class_id(&pred.predicate) else {
                continue;
            };

            let domain = predicate_domain.entry(pred_id).or_default();
            for c in &pred.declared_domains {
                if let Some(cid) = class_id(c) {
                    domain.insert(cid);
                }
            }
            for c in &pred.inferred_domains {
                if let Some(cid) = class_id(&c.iri) {
                    domain.insert(cid);
                }
            }

            let range = predicate_range.entry(pred_id).or_default();
            for c in &pred.declared_ranges {
                if let Some(cid) = class_id(c) {
                    range.insert(cid);
                }
            }
            for c in &pred.inferred_ranges {
                if let Some(cid) = class_id(&c.iri) {
                    range.insert(cid);
                }
            }
        }

        TypeConstraints {
            entity_types,
            predicate_domain,
            predicate_range,
        }
    }

    /// The interned `rdf:type` class ids of `entity` (empty if it has none).
    pub fn types_of(&self, entity: Id) -> Option<&FxHashSet<Id>> {
        self.entity_types.get(&entity)
    }

    /// Is `entity` admissible as the **subject** of `predicate`? `true` when the predicate has no
    /// known domain (unconstrained), or `entity` carries at least one type in the domain.
    pub fn admissible_subject(&self, predicate: Id, entity: Id) -> bool {
        self.admissible(&self.predicate_domain, predicate, entity)
    }

    /// Is `entity` admissible as the **object** of `predicate`? `true` when the predicate has no
    /// known range (unconstrained), or `entity` carries at least one type in the range.
    pub fn admissible_object(&self, predicate: Id, entity: Id) -> bool {
        self.admissible(&self.predicate_range, predicate, entity)
    }

    fn admissible(&self, side: &FxHashMap<Id, FxHashSet<Id>>, predicate: Id, entity: Id) -> bool {
        match side.get(&predicate) {
            // Unconstrained side (no declared/observed classes) → every entity is admissible.
            None => true,
            Some(classes) if classes.is_empty() => true,
            Some(classes) => match self.entity_types.get(&entity) {
                // The entity has no type → cannot be proven admissible against a constrained
                // side; treat it as inadmissible so type-constrained mode is strictly the
                // restriction the literature describes (Unconstrained mode bypasses this).
                None => false,
                Some(types) => types.iter().any(|t| classes.contains(t)),
            },
        }
    }
}

/// Which negative-sampling regime to apply — the **on/off ablation switch** (design §6.B). The
/// sampler runs identically under both modes except for the admissibility filter, so a harness can
/// measure type-constrained vs unconstrained on the *same* seed and positives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplingMode {
    /// Vanilla uniform corruption: any entity is a candidate (the ablation **off** baseline).
    Unconstrained,
    /// Krompass et al. 2015 type-constrained corruption: head corruptions restricted to the
    /// predicate's domain types, tail corruptions to its range types (the ablation **on** arm).
    TypeConstrained,
}

/// Which position of a positive triple a negative corrupts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corrupt {
    /// Replace the head (subject), keeping `(?, r, t)`.
    Head,
    /// Replace the tail (object), keeping `(h, r, ?)`.
    Tail,
}

/// A deterministic, type-aware **negative sampler** for KGE training. It does **not** train — it
/// only *emits* corrupted triples a trainer scores. This is the reusable P0(b) primitive.
///
/// The entity pool is the graph's distinct subjects-and-objects of object-property triples (the
/// nodes a KGE places in entity space). Corruptions are drawn deterministically from a SplitMix64
/// stream seeded per call, so a fixed `(seed, mode)` is reproducible across runs and platforms —
/// the on/off ablation compares like with like.
pub struct NegativeSampler<'a> {
    constraints: &'a TypeConstraints,
    /// All candidate entity ids (sorted, deduplicated) — the corruption pool.
    entities: Vec<Id>,
    /// Positive `(h, r, t)` triples, used to reject a corruption that accidentally reproduces a
    /// true triple (the standard "filtered" negative-sampling guard).
    positives: FxHashSet<[Id; 3]>,
    mode: SamplingMode,
}

impl<'a> NegativeSampler<'a> {
    /// Build a sampler over `graph`'s object-property triples (literal objects are skipped — a KGE
    /// places only entities in entity space; literals are handled by the typed-literal encoders of
    /// design Phase 1, not here). `constraints` should be mined from the **same, closed** graph.
    ///
    /// `mode` is the ablation switch ([`SamplingMode`]).
    pub fn new(
        graph: &Graph,
        constraints: &'a TypeConstraints,
        mode: SamplingMode,
    ) -> NegativeSampler<'a> {
        let mut entity_set: FxHashSet<Id> = FxHashSet::default();
        let mut positives: FxHashSet<[Id; 3]> = FxHashSet::default();

        for [s, p, o] in graph.iter_ids() {
            // Only object-property triples (object is an entity, not a literal/inline value).
            if is_entity(graph, o) && is_entity(graph, s) {
                entity_set.insert(s);
                entity_set.insert(o);
                positives.insert([s, p, o]);
            }
        }

        // Deterministic order: sort the dedup'd entity ids ascending, independent of the
        // FxHashSet iteration order, so the sampler is reproducible across runs.
        let mut entities: Vec<Id> = entity_set.into_iter().collect();
        entities.sort_unstable();

        NegativeSampler {
            constraints,
            entities,
            positives,
            mode,
        }
    }

    /// The number of candidate entities in the corruption pool.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Sample up to `n` negatives for the positive triple `(h, r, t)` corrupting `side`, using
    /// `seed` to drive a deterministic stream. Returns fewer than `n` only when the admissible
    /// pool is exhausted (e.g. a tightly-typed predicate with few range entities).
    ///
    /// A candidate is rejected when it (a) equals the original entity, (b) reproduces a true
    /// positive triple (filtered guard), or (c) under [`SamplingMode::TypeConstrained`] is not
    /// admissible for the corrupted side. Under [`SamplingMode::Unconstrained`] only (a) and (b)
    /// apply — that is the ablation baseline.
    pub fn sample(&self, triple: [Id; 3], side: Corrupt, n: usize, seed: u64) -> Vec<[Id; 3]> {
        let [h, r, t] = triple;
        if self.entities.is_empty() || n == 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(n.min(self.entities.len()));
        let mut emitted: FxHashSet<Id> = FxHashSet::default();
        let mut state = seed ^ mix_triple(triple) ^ (side_salt(side));

        // Bounded attempts: at most a small multiple of the pool, so a near-empty admissible set
        // terminates instead of spinning. The cap is generous enough that a healthy pool fills `n`.
        let max_attempts = self.entities.len().saturating_mul(8).max(64);
        for _ in 0..max_attempts {
            if out.len() >= n {
                break;
            }
            let idx = (splitmix64(&mut state) as usize) % self.entities.len();
            let candidate = self.entities[idx];

            let (nh, nt) = match side {
                Corrupt::Head => (candidate, t),
                Corrupt::Tail => (h, candidate),
            };
            let original = match side {
                Corrupt::Head => h,
                Corrupt::Tail => t,
            };

            if candidate == original || emitted.contains(&candidate) {
                continue;
            }
            // Filtered guard: never emit a corruption that is itself a true triple.
            if self.positives.contains(&[nh, r, nt]) {
                continue;
            }
            // Type-constraint filter — the only behavioural difference between the two modes.
            if self.mode == SamplingMode::TypeConstrained {
                let ok = match side {
                    Corrupt::Head => self.constraints.admissible_subject(r, candidate),
                    Corrupt::Tail => self.constraints.admissible_object(r, candidate),
                };
                if !ok {
                    continue;
                }
            }

            emitted.insert(candidate);
            out.push([nh, r, nt]);
        }
        out
    }
}

// ---- helpers ----------------------------------------------------------------------------------

/// The `rdf:type` named-node term.
fn type_term() -> oxrdf::Term {
    oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(RDF_TYPE))
}

/// Resolve an IRI string to its interned dictionary id, or `None` if absent.
fn dict_iri_id(dict: &sparq_core::dict::Dict, iri: &str) -> Option<Id> {
    let id = dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
        iri,
    )));
    (id != sparq_core::dict::NO_ID).then_some(id)
}

/// Is the term id an **entity** (a named node or blank node), as opposed to a literal / inline
/// value? Only entities go in KGE entity space.
fn is_entity(graph: &Graph, id: Id) -> bool {
    matches!(
        graph.dict.term_parts(id),
        TermParts::Iri { .. } | TermParts::Blank(_)
    )
}

/// Mix a triple into a 64-bit value so the per-triple stream differs for each positive (the
/// sampler is otherwise deterministic for a fixed seed).
fn mix_triple([s, p, o]: [Id; 3]) -> u64 {
    let mut v = (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    v ^= (p as u64)
        .rotate_left(21)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9);
    v ^= (o as u64)
        .rotate_left(42)
        .wrapping_mul(0x94D0_49BB_1331_11EB);
    v
}

/// A small per-side salt so head and tail streams of the same triple diverge.
fn side_salt(side: Corrupt) -> u64 {
    match side {
        Corrupt::Head => 0x1111_2222_3333_4444,
        Corrupt::Tail => 0x5555_6666_7777_8888,
    }
}

/// SplitMix64 — a small, fast, well-distributed deterministic PRNG step. Reused (not imported)
/// from the same algorithm `embed.rs` uses; kept local so `structure` carries no cross-module
/// `pub(crate)` coupling beyond the feature gate.
#[inline]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small schema-rich graph: a subclass hierarchy + typed entities + an object property with a
    // declared domain/range, plus a "wrong-typed" entity that type-constrained sampling must
    // exclude as a corruption.
    const TTL: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <http://ex/> .

ex:Dog   rdfs:subClassOf ex:Animal .
ex:Cat   rdfs:subClassOf ex:Animal .
ex:Owner rdfs:subClassOf ex:Person .

ex:owns rdfs:domain ex:Person ; rdfs:range ex:Animal .

ex:alice a ex:Owner .
ex:bob   a ex:Owner .
ex:rex   a ex:Dog .
ex:tom   a ex:Cat .
ex:car   a ex:Vehicle .

ex:alice ex:owns ex:rex .
ex:bob   ex:owns ex:tom .
"#;

    fn closed() -> ClosedGraph {
        close_for_vectorise(TTL, "turtle", Profile::Rdfs).unwrap()
    }

    #[test]
    fn closure_materialises_entailed_types() {
        let c = closed();
        // RDFS must add at least: rex a Animal, tom a Animal, alice a Person, bob a Person (rdfs9
        // via subClassOf), and the domain/range entailments (rdfs2/rdfs3).
        assert!(
            c.entailed_triples > 0,
            "closure added nothing: {}",
            c.entailed_triples
        );
        assert_eq!(c.closed_triples(), c.asserted_triples + c.entailed_triples);

        // The entailed `ex:rex a ex:Animal` must now be a real triple visible to the encoder.
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let animal = c.graph.id_of(&iri("http://ex/Animal")).unwrap();
        let type_p = c.graph.id_of(&type_term()).unwrap();
        let has = c
            .graph
            .iter_ids()
            .any(|[s, p, o]| s == rex && p == type_p && o == animal);
        assert!(
            has,
            "closure-before-vectorise did not materialise ex:rex a ex:Animal"
        );
    }

    #[test]
    fn type_constraints_read_domain_and_range() {
        let c = closed();
        let tc = TypeConstraints::mine(&c.graph);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();

        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap(); // Owner ⊑ Person
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap(); // Dog ⊑ Animal
        let car = c.graph.id_of(&iri("http://ex/car")).unwrap(); // Vehicle — neither

        // Subject (domain = Person): a Person is admissible; an Animal/Vehicle is not.
        assert!(
            tc.admissible_subject(owns, alice),
            "Owner⊑Person must satisfy domain Person"
        );
        assert!(
            !tc.admissible_subject(owns, car),
            "Vehicle must not satisfy domain Person"
        );

        // Object (range = Animal): an Animal is admissible; a Person/Vehicle is not.
        assert!(
            tc.admissible_object(owns, rex),
            "Dog⊑Animal must satisfy range Animal"
        );
        assert!(
            !tc.admissible_object(owns, car),
            "Vehicle must not satisfy range Animal"
        );
        assert!(
            !tc.admissible_object(owns, alice),
            "Person must not satisfy range Animal"
        );
    }

    #[test]
    fn type_constrained_sampler_excludes_wrong_types() {
        let c = closed();
        let tc = TypeConstraints::mine(&c.graph);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();
        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap();
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let car = c.graph.id_of(&iri("http://ex/car")).unwrap();

        let sampler = NegativeSampler::new(&c.graph, &tc, SamplingMode::TypeConstrained);

        // Corrupt the TAIL of (alice owns rex): every emitted object must be range-admissible
        // (an Animal), so `ex:car` (Vehicle) can NEVER appear.
        let negs = sampler.sample([alice, owns, rex], Corrupt::Tail, 16, 42);
        assert!(
            !negs.is_empty(),
            "expected some type-valid tail corruptions"
        );
        for [h, r, t] in &negs {
            assert_eq!(*h, alice);
            assert_eq!(*r, owns);
            assert!(tc.admissible_object(owns, *t), "emitted non-range tail {t}");
            assert_ne!(
                *t, car,
                "Vehicle must never be a type-constrained range corruption"
            );
            assert_ne!(*t, rex, "must not reproduce the original tail");
        }
    }

    #[test]
    fn ablation_off_admits_wrong_types() {
        let c = closed();
        let tc = TypeConstraints::mine(&c.graph);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();
        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap();
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let car = c.graph.id_of(&iri("http://ex/car")).unwrap();

        // Unconstrained mode is the ablation OFF baseline: the type filter is bypassed, so the
        // wrong-typed `ex:car` is reachable as a tail corruption (it would never be under ON).
        let off = NegativeSampler::new(&c.graph, &tc, SamplingMode::Unconstrained);
        let mut saw_car = false;
        // Several seeds, since any single draw is random; the pool is tiny so this is reliable.
        for seed in 0..32u64 {
            let negs = off.sample([alice, owns, rex], Corrupt::Tail, 8, seed);
            if negs.iter().any(|[_, _, t]| *t == car) {
                saw_car = true;
                break;
            }
        }
        assert!(
            saw_car,
            "Unconstrained ablation arm must be able to emit the wrong-typed entity"
        );
        let _ = rex;
    }

    #[test]
    fn sampler_is_deterministic_for_fixed_seed() {
        let c = closed();
        let tc = TypeConstraints::mine(&c.graph);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();
        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap();
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let s = NegativeSampler::new(&c.graph, &tc, SamplingMode::TypeConstrained);
        let a = s.sample([alice, owns, rex], Corrupt::Tail, 4, 7);
        let b = s.sample([alice, owns, rex], Corrupt::Tail, 4, 7);
        assert_eq!(
            a, b,
            "fixed (seed, mode, triple) must reproduce identical negatives"
        );
    }

    #[test]
    fn sampler_never_reproduces_a_true_triple() {
        let c = closed();
        let tc = TypeConstraints::mine(&c.graph);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();
        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap();
        let bob = c.graph.id_of(&iri("http://ex/bob")).unwrap();
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let tom = c.graph.id_of(&iri("http://ex/tom")).unwrap();
        // (bob owns tom) is a true triple; corrupting (alice owns rex)'s head to `bob` together
        // with tail `tom` is not what `sample` does (it fixes the tail), but corrupting the tail of
        // (bob owns ?) must never re-emit `tom`.
        let s = NegativeSampler::new(&c.graph, &tc, SamplingMode::TypeConstrained);
        for seed in 0..32 {
            let negs = s.sample([bob, owns, tom], Corrupt::Tail, 8, seed);
            assert!(
                !negs.contains(&[bob, owns, tom]),
                "filtered guard must drop the true triple"
            );
        }
        let _ = (alice, rex);
    }

    #[test]
    fn unconstrained_predicate_is_fail_open() {
        // A predicate with no declared/observed range still samples (degrades to uniform), so a
        // missing schema never deadlocks the trainer.
        let ttl = r#"
@prefix ex: <http://ex/> .
ex:a ex:rel ex:b .
ex:b ex:rel ex:c .
ex:c ex:rel ex:a .
"#;
        let c = close_for_vectorise(ttl, "turtle", Profile::Rdfs).unwrap();
        let tc = TypeConstraints::mine(&c.graph);
        let rel = c.graph.id_of(&iri("http://ex/rel")).unwrap();
        let a = c.graph.id_of(&iri("http://ex/a")).unwrap();
        let b = c.graph.id_of(&iri("http://ex/b")).unwrap();
        let s = NegativeSampler::new(&c.graph, &tc, SamplingMode::TypeConstrained);
        let negs = s.sample([a, rel, b], Corrupt::Tail, 4, 1);
        assert!(
            !negs.is_empty(),
            "unconstrained predicate must still produce negatives"
        );
    }

    fn iri(s: &str) -> oxrdf::Term {
        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
    }
}
