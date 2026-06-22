#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-evb1: crate has zero `unsafe`.

//! See the crate README (rendered above) for the capability overview. The load-bearing
//! entry points are [`classify_graph`] (emit the complete subsumption lattice into a
//! `(Dict, triples)` pair) and [`Classifier`] (the typed [`ClassHierarchy`] API). The
//! algorithm lives in three private modules: `normal` (Baader–Brandt–Lutz normalization),
//! `classify` (CR1–CR5 saturation), and `extract` (RDF → EL+⊥ axioms).

use rustc_hash::FxHashMap;
use sparq_core::dict::{Dict, Id};

mod classify;
mod extract;
mod normal;

const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const OWL_NOTHING: &str = "http://www.w3.org/2002/07/owl#Nothing";

/// A tally of what the extractor read and what it could not (so an honest
/// "n axioms outside the EL fragment were ignored" can be surfaced).
///
/// The classifier is SOUND but only COMPLETE for the EL+⊥-minus-RBox fragment it recognises
/// (the MVP, Phase E1). Axioms using constructs outside that fragment — `owl:unionOf`,
/// `owl:complementOf`, `owl:allValuesFrom`, cardinality, `owl:hasValue`, nominals
/// (`owl:oneOf`), data properties, and RBox axioms (`owl:propertyChainAxiom`,
/// `owl:TransitiveProperty`, `rdfs:subPropertyOf` between object properties) — are NOT applied
/// and are counted in [`Report::skipped_axioms`]. RBox / property-chain saturation is Phase E2
/// (bead sq-xetf7); nominals + concrete domains are deferred further still.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Distinct named classes (and ⊤/⊥ if they occur) seen by the extractor.
    pub named_classes: usize,
    /// Class-axioms (`subClassOf`/`equivalentClass`/`disjointWith`) whose class expression
    /// used a construct OUTSIDE the recognised EL+⊥ fragment, and were therefore IGNORED.
    pub skipped_axioms: usize,
    /// New `rdfs:subClassOf` triples emitted by [`classify_graph`].
    pub emitted_subsumptions: usize,
    /// Named classes found unsatisfiable (`⊑ owl:Nothing`).
    pub unsatisfiable_classes: usize,
}

/// The complete class-subsumption lattice for the named classes of a TBox, as a typed view
/// over the source dict ids. Built by [`Classifier::classify`]; the triple-emitting
/// [`classify_graph`] wraps the same computation.
pub struct ClassHierarchy {
    /// `subClassOf[c]` = the named super-classes of dict class `c` (every class subsuming it,
    /// not just direct super-classes — transitive reduction is Phase E3, bead sq-s2nob).
    subsumers: FxHashMap<Id, Vec<Id>>,
    /// Dict ids of the unsatisfiable named classes (`⊑ owl:Nothing`).
    unsatisfiable: Vec<Id>,
    report: Report,
}

impl ClassHierarchy {
    /// All (transitive) named super-classes of `class`, by dict id (empty if `class` is not a
    /// classified named class or has no proper super-class besides ⊤). Excludes the class
    /// itself, `owl:Thing`, and `owl:Nothing`.
    pub fn super_classes(&self, class: Id) -> &[Id] {
        self.subsumers.get(&class).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Whether `sub` is subsumed by `sup` (`sub ⊑ sup`) under the classified TBox. Reflexive
    /// (`sub == sup`) and `sup == owl:Thing` are NOT reported here — this answers the proper
    /// derived subsumptions only, matching [`super_classes`](Self::super_classes).
    pub fn is_subclass_of(&self, sub: Id, sup: Id) -> bool {
        self.subsumers.get(&sub).is_some_and(|s| s.contains(&sup))
    }

    /// Dict ids of the named classes found unsatisfiable (`⊑ owl:Nothing`). A non-empty list
    /// means the TBox forces those classes to be empty (an EL+⊥ clash, e.g. via
    /// `owl:disjointWith`); it does NOT by itself make the whole ontology inconsistent (that
    /// needs an asserted instance of an unsatisfiable class — ABox reasoning, out of scope).
    pub fn unsatisfiable_classes(&self) -> &[Id] {
        &self.unsatisfiable
    }

    /// The extraction/classification [`Report`] (named-class count, skipped non-EL axioms,
    /// unsatisfiable-class count). `emitted_subsumptions` is zero here — it is set by
    /// [`classify_graph`], which materializes the lattice as triples.
    pub fn report(&self) -> Report {
        self.report
    }
}

/// The OWL 2 EL consequence-based classifier. Stateless wrapper — [`Classifier::classify`]
/// runs the whole normalize → saturate → project pipeline over a borrowed graph.
///
/// ```
/// use sparq_core::Graph;
/// use sparq_reason_el::Classifier;
///
/// // A ⊑ ∃r.B, B ⊑ C, ∃r.C ⊑ D  ⊨  A ⊑ D  (the CR4 subsumption OWL 2 RL cannot derive).
/// let ttl = r#"
///   @prefix : <http://ex/> .
///   @prefix owl: <http://www.w3.org/2002/07/owl#> .
///   @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
///   :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
///   :B rdfs:subClassOf :C .
///   [ owl:onProperty :r ; owl:someValuesFrom :C ] rdfs:subClassOf :D .
/// "#;
/// let (dict, triples) = Graph::parse_to_triples(ttl, "turtle").unwrap();
/// let h = Classifier::classify(&dict, &triples);
/// let a = dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://ex/A").unwrap()));
/// let d = dict.lookup(&oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://ex/D").unwrap()));
/// assert!(h.is_subclass_of(a, d));
/// ```
pub struct Classifier;

impl Classifier {
    /// Classifies the EL+⊥ TBox embedded in `triples`, returning the complete named-class
    /// subsumption [`ClassHierarchy`]. Does NOT mutate the graph; use [`classify_graph`] to
    /// materialize the lattice as triples. Single-threaded.
    pub fn classify(dict: &Dict, triples: &[[Id; 3]]) -> ClassHierarchy {
        let (axioms, names, mut report) = extract::extract(dict, triples);
        let sat = classify::saturate(&axioms, names.concept_count());
        let cls = classify::classify(&sat, &names);
        // Re-key the named-concept lattice onto dict ids for the public view.
        let mut subsumers: FxHashMap<Id, Vec<Id>> = FxHashMap::default();
        for (&c, sups) in &cls.subsumers {
            let Some(cid) = names.dict_of(c) else {
                continue;
            };
            let row: Vec<Id> = sups.iter().filter_map(|&d| names.dict_of(d)).collect();
            if !row.is_empty() {
                subsumers.insert(cid, row);
            }
        }
        let unsatisfiable: Vec<Id> = cls
            .unsatisfiable
            .iter()
            .filter_map(|&c| names.dict_of(c))
            .collect();
        report.unsatisfiable_classes = unsatisfiable.len();
        ClassHierarchy {
            subsumers,
            unsatisfiable,
            report,
        }
    }
}

/// Classifies the EL+⊥ TBox in `triples` and MATERIALIZES the complete subsumption lattice
/// IN PLACE: appends every derived `(C rdfs:subClassOf D)` not already present (interning the
/// `rdfs:subClassOf` predicate into `dict` if absent), plus `(C rdf:type owl:Nothing)`-style
/// clashes are surfaced via [`Report::unsatisfiable_classes`] (not emitted as triples — the
/// caller decides whether to flag them, matching the RL `inconsistencies()` reporting shape).
///
/// Returns the [`Report`] with `emitted_subsumptions` set to the number of NEW triples added.
/// Idempotent: a second call adds nothing. This is the seam the CLI/`Graph` integration uses —
/// the emitted edges are ordinary triples queryable by plain BGP eval, exactly like the RL
/// `scm-*` output.
pub fn classify_graph(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> Report {
    let hierarchy = Classifier::classify(dict, triples);
    let sub_class_of = dict.intern_iri(RDFS_SUB_CLASS_OF);
    let _ = dict.intern_iri(OWL_NOTHING); // ensure the term exists for downstream clash checks.

    let mut present: rustc_hash::FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut emitted = 0usize;
    // Deterministic order: by subclass dict id, then super-class dict id.
    let mut subs: Vec<(&Id, &Vec<Id>)> = hierarchy.subsumers.iter().collect();
    subs.sort_unstable_by_key(|(c, _)| **c);
    for (&c, sups) in subs {
        let mut sups = sups.clone();
        sups.sort_unstable();
        for d in sups {
            let t = [c, sub_class_of, d];
            if present.insert(t) {
                triples.push(t);
                emitted += 1;
            }
        }
    }
    let mut report = hierarchy.report;
    report.emitted_subsumptions = emitted;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{NamedNode, Term as OTerm};
    use sparq_core::Graph;

    fn iri(dict: &Dict, s: &str) -> Id {
        dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(s.to_string())))
    }

    fn parse(ttl: &str) -> (Dict, Vec<[Id; 3]>) {
        Graph::parse_to_triples(ttl, "turtle").expect("parse")
    }

    const PRE: &str = r#"
        @prefix : <http://ex/> .
        @prefix owl: <http://www.w3.org/2002/07/owl#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    "#;

    #[test]
    fn transitive_subclass() {
        let ttl = format!("{PRE} :A rdfs:subClassOf :B . :B rdfs:subClassOf :C .");
        let (dict, triples) = parse(&ttl);
        let h = Classifier::classify(&dict, &triples);
        let (a, c) = (iri(&dict, "http://ex/A"), iri(&dict, "http://ex/C"));
        assert!(h.is_subclass_of(a, c), "A ⊑ C must be derived transitively");
    }

    #[test]
    fn cr4_existential_traversal() {
        // The spike §1.3 counterexample: A ⊑ ∃r.B, B ⊑ C, ∃r.C ⊑ D ⊨ A ⊑ D. RL CANNOT do this.
        let ttl = format!(
            "{PRE}
             :A rdfs:subClassOf [ owl:onProperty :r ; owl:someValuesFrom :B ] .
             :B rdfs:subClassOf :C .
             [ owl:onProperty :r ; owl:someValuesFrom :C ] rdfs:subClassOf :D ."
        );
        let (dict, triples) = parse(&ttl);
        let h = Classifier::classify(&dict, &triples);
        let (a, d) = (iri(&dict, "http://ex/A"), iri(&dict, "http://ex/D"));
        assert!(
            h.is_subclass_of(a, d),
            "CR4 must derive A ⊑ D through the r-successor"
        );
    }

    #[test]
    fn conjunction_subsumption() {
        // (A ⊓ B) ⊑ C, X ⊑ A, X ⊑ B ⊨ X ⊑ C.
        let ttl = format!(
            "{PRE}
             [ owl:intersectionOf ( :A :B ) ] rdfs:subClassOf :C .
             :X rdfs:subClassOf :A . :X rdfs:subClassOf :B ."
        );
        let (dict, triples) = parse(&ttl);
        let h = Classifier::classify(&dict, &triples);
        let (x, c) = (iri(&dict, "http://ex/X"), iri(&dict, "http://ex/C"));
        assert!(
            h.is_subclass_of(x, c),
            "CR2 must derive X ⊑ C from the conjunction"
        );
    }

    #[test]
    fn disjoint_makes_unsatisfiable() {
        // A disjointWith B, X ⊑ A, X ⊑ B ⊨ X ⊑ ⊥.
        let ttl = format!(
            "{PRE}
             :A owl:disjointWith :B .
             :X rdfs:subClassOf :A . :X rdfs:subClassOf :B ."
        );
        let (dict, triples) = parse(&ttl);
        let h = Classifier::classify(&dict, &triples);
        let x = iri(&dict, "http://ex/X");
        assert!(
            h.unsatisfiable_classes().contains(&x),
            "X must be unsatisfiable (A ⊓ B ⊑ ⊥)"
        );
    }

    #[test]
    fn classify_graph_materializes_and_is_idempotent() {
        let ttl = format!("{PRE} :A rdfs:subClassOf :B . :B rdfs:subClassOf :C .");
        let (mut dict, mut triples) = parse(&ttl);
        let before = triples.len();
        let r1 = classify_graph(&mut dict, &mut triples);
        // A ⊑ C is the one NEW edge (A⊑B, B⊑C are asserted).
        assert_eq!(r1.emitted_subsumptions, 1, "exactly the A ⊑ C edge is new");
        assert_eq!(triples.len(), before + 1);
        let r2 = classify_graph(&mut dict, &mut triples);
        assert_eq!(r2.emitted_subsumptions, 0, "second call is idempotent");
    }

    #[test]
    fn non_el_axioms_are_skipped_not_misapplied() {
        // owl:unionOf is outside EL+⊥: the axiom must be skipped, not crash or mis-derive.
        let ttl = format!(
            "{PRE}
             [ owl:unionOf ( :A :B ) ] rdfs:subClassOf :C .
             :A rdfs:subClassOf :D ."
        );
        let (dict, triples) = parse(&ttl);
        let h = Classifier::classify(&dict, &triples);
        assert!(
            h.report().skipped_axioms >= 1,
            "the unionOf axiom must be recorded as skipped"
        );
        let (a, d) = (iri(&dict, "http://ex/A"), iri(&dict, "http://ex/D"));
        assert!(
            h.is_subclass_of(a, d),
            "the EL part (A ⊑ D) must still classify"
        );
    }

    #[test]
    fn empty_graph_is_safe() {
        let (dict, triples) = (Dict::new(), Vec::<[Id; 3]>::new());
        let h = Classifier::classify(&dict, &triples);
        assert_eq!(h.report().emitted_subsumptions, 0);
        assert!(h.unsatisfiable_classes().is_empty());
    }
}
