//! Opt-in reasoning for sparq: forward-chaining **materialization** of the deductive
//! closure over dictionary-encoded triples.
//!
//! Materialization (compute all entailed triples and add them to the store, à la RDFox's
//! datalog approach) is the right fit for a dictionary-encoded triplestore: rules are joins
//! over fixed-width integer ids, and the closure is computed once at load/build time — so
//! querying stays exactly as fast as before and the default (no-reason) build is completely
//! unaffected (this crate is not even compiled unless depended upon).
//!
//! Profiles:
//! - [`Profile::Rdfs`] — RDFS entailment (the non-explosive subset rdfs2,3,5,7,9,11), DONE.
//! - `Profile::OwlRl` — OWL 2 RL property/class axioms (sameAs, inverseOf, Transitive/
//!   Symmetric, equivalentClass/Property, …) — roadmap, builds on the same fixpoint engine.
//! - Notation3 / EYE-style rules (`{ … } => { … }` with builtins) — a separate, larger
//!   subsystem (see `n3` module, roadmap).
//!
//! Beyond batch materialization, [`MaterializedGraph`] maintains the RDFS closure
//! **incrementally** under inserts/deletes (exact derivation counting; see the
//! `incremental` module docs) so a data update costs time proportional to the change, not
//! a full re-materialization.

use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

mod incremental;
pub mod n3;
mod owl;
mod rdfs;
pub use incremental::MaterializedGraph;
pub use n3::{reason_n3, reason_n3_proof, ProofStep};
pub use owl::{inconsistencies, materialize_owl_rl};
pub use rdfs::materialize_rdfs;
pub(crate) use rdfs::RdfsIndex;

/// Which entailment regime to materialize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// RDFS (subset rdfs2,3,5,7,9,11 — subclass/subproperty/domain/range; no axiomatic or
    /// reflexive triples, which add no useful inferences and explode the store).
    Rdfs,
    /// OWL 2 RL (a useful subset: equality/sameAs, inverseOf, Symmetric/Transitive/
    /// Functional/InverseFunctional properties, equivalentClass/Property) — includes RDFS.
    OwlRl,
}

impl Profile {
    /// Parse a CLI/profile name (`"rdfs"`, `"owl"`/`"owl-rl"`).
    pub fn parse(s: &str) -> Option<Profile> {
        match s.to_ascii_lowercase().as_str() {
            "rdfs" => Some(Profile::Rdfs),
            "owl" | "owl-rl" | "owlrl" => Some(Profile::OwlRl),
            _ => None,
        }
    }
}

/// Materialize the closure for `profile` in place: expands `triples` with every entailed
/// triple (deduplicated against the input), interning any vocabulary terms it needs into
/// `dict`. Returns the number of NEW triples added. Idempotent: a second call adds nothing.
pub fn materialize(profile: Profile, dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> usize {
    match profile {
        Profile::Rdfs => materialize_rdfs(dict, triples),
        Profile::OwlRl => materialize_owl_rl(dict, triples),
    }
}

/// Standard RDF/RDFS vocabulary ids, interned once. Interning is idempotent — it returns
/// the id the data already uses for e.g. `rdf:type`, or assigns a fresh one if the term is
/// absent (in which case it simply matches nothing). Predicates are never inline ids.
pub(crate) struct Vocab {
    pub ty: Id,
    pub sub_class: Id,
    pub sub_prop: Id,
    pub domain: Id,
    pub range: Id,
}

impl Vocab {
    pub fn intern(dict: &mut Dict) -> Vocab {
        use oxrdf::vocab::{rdf, rdfs};
        Vocab {
            ty: dict.intern_iri(rdf::TYPE.as_str()),
            sub_class: dict.intern_iri(rdfs::SUB_CLASS_OF.as_str()),
            sub_prop: dict.intern_iri(rdfs::SUB_PROPERTY_OF.as_str()),
            domain: dict.intern_iri(rdfs::DOMAIN.as_str()),
            range: dict.intern_iri(rdfs::RANGE.as_str()),
        }
    }
}

/// A small typed view over the schema (TBox) edges, rebuilt from the current triple set
/// each fixpoint round. Shared by the RDFS rules (and, later, OWL-RL).
#[derive(Default)]
pub(crate) struct Schema {
    /// `c -> direct super-classes d` from `(c rdfs:subClassOf d)`.
    pub sub_class: FxHashMap<Id, Vec<Id>>,
    /// `p -> direct super-properties q` from `(p rdfs:subPropertyOf q)`.
    pub sub_prop: FxHashMap<Id, Vec<Id>>,
    /// `p -> domain classes` from `(p rdfs:domain c)`.
    pub domain: FxHashMap<Id, Vec<Id>>,
    /// `p -> range classes` from `(p rdfs:range c)`.
    pub range: FxHashMap<Id, Vec<Id>>,
}

impl Schema {
    pub fn build(all: &FxHashSet<[Id; 3]>, v: &Vocab) -> Schema {
        let mut s = Schema::default();
        for &[su, p, o] in all {
            if p == v.sub_class {
                s.sub_class.entry(su).or_default().push(o);
            } else if p == v.sub_prop {
                s.sub_prop.entry(su).or_default().push(o);
            } else if p == v.domain {
                s.domain.entry(su).or_default().push(o);
            } else if p == v.range {
                s.range.entry(su).or_default().push(o);
            }
        }
        s
    }
}
