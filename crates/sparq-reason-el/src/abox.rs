// [OPUS-4.8] sq-pbz04.2.5 (epic sq-pbz04.2): ABox internalization + realisation + whole-ontology
// consistency — the opt-in `abox` feature.
//
// The TBox classifier (`Classifier::classify` / `classify_graph`) is instance-agnostic by
// contract. This module adds the ABox layer WITHOUT disturbing it: it runs the SAME
// normalize → CR1–CR6 saturation over the TBox axioms PLUS the ABox assertions internalized as
// safe-nominal axioms (`extract::extract` with `ExtractOpts { abox: true }` — the internalization
// lives in `extract.rs` where it reuses the restriction/intersection `decode` machinery), then
// READS OFF the saturation:
//
//   {a} ⊑ C   (C a NAMED class, incl. owl:Thing)  ⇒  `a rdf:type C`
//   {a} ⊑ {b} (a derived nominal subsumption)      ⇒  `a owl:sameAs b`
//   {a} ⊑ ⊥   OR  ⊤ ⊑ ⊥                            ⇒  a whole-ontology `inconsistent` verdict
//
// SOUNDNESS (the load-bearing invariant — soundness over completeness). The saturation maintains
// `D ∈ S(C) ⟹ T ⊨ C ⊑ D`. A nominal `{a}` denotes the singleton `{a^I}`, so:
//   * `C ∈ S({a})` with C named ⟹ `T ⊨ {a} ⊑ C` ⟹ `a^I ∈ C^I` — the typing holds in EVERY model.
//   * `{b} ∈ S({a})` ⟹ `T ⊨ {a} ⊑ {b}` ⟹ `a^I ∈ {b^I}` ⟹ `a^I = b^I` — `owl:sameAs` holds.
//   * `⊥ ∈ S({a})` ⟹ `{a}` is empty, but `a` witnesses it non-empty — a contradiction: the whole
//     ontology has no model. `⊥ ∈ S(⊤)` is the global `⊤ ⊑ ⊥` case.
// The CR6 reachability side-condition is untouched (this module only READS the saturation the
// existing `classify::saturate` produced). Unsupported assertion shapes stay counted skips
// (`Report::skipped_assertions`, fail-closed) — never a guessed typing.

use crate::extract::{self, ExtractOpts};
use crate::normal::{Concept, Names, BOTTOM, TOP};
use crate::{classify, ClassHierarchy, Report};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id, NO_ID};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";

/// A list of `(individual dict id, class-or-individual dict id)` readoff rows.
type Pairs = Vec<(Id, Id)>;

fn lookup(dict: &Dict, iri: &str) -> Id {
    use oxrdf::{NamedNode, Term as OTerm};
    dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(iri.to_string())))
}

/// The result of ABox-aware classification (the `abox` feature): the TBox subsumption
/// [`ClassHierarchy`] PLUS the realised instance facts and the whole-ontology consistency verdict.
///
/// Every emitted typing / `owl:sameAs` holds in EVERY model of the input (soundness over
/// completeness); when the ontology is [inconsistent](Self::is_inconsistent) the realised facts
/// are EMPTY (an inconsistent ontology entails everything — the honest surface is the verdict
/// alone, not a flood of derived rows).
pub struct Realization {
    hierarchy: ClassHierarchy,
    types: Vec<(Id, Id)>,
    same_as: Vec<(Id, Id)>,
}

impl Realization {
    /// The TBox class-subsumption lattice (unchanged by the ABox layer — nominals project out).
    pub fn class_hierarchy(&self) -> &ClassHierarchy {
        &self.hierarchy
    }

    /// Realised `(individual dict id, class dict id)` type assertions — the complete derived
    /// typing of every asserted individual (including `owl:Thing` when it is interned), sorted +
    /// deduplicated. EMPTY when the ontology is inconsistent.
    pub fn type_assertions(&self) -> &[(Id, Id)] {
        &self.types
    }

    /// Realised `(individual, individual)` `owl:sameAs` pairs derived from nominal equality,
    /// sorted + deduplicated. EMPTY when the ontology is inconsistent.
    pub fn same_as(&self) -> &[(Id, Id)] {
        &self.same_as
    }

    /// Whether the WHOLE ontology is inconsistent (`{a} ⊑ ⊥` for an asserted individual, or a
    /// global `⊤ ⊑ ⊥`). See [`ClassHierarchy::is_inconsistent`].
    pub fn is_inconsistent(&self) -> bool {
        self.hierarchy.is_inconsistent()
    }

    /// The extraction/classification [`Report`] — `skipped_assertions` is the count of
    /// data-property / non-EL assertions DEFERRED (fail-closed), `unsatisfiable_classes` the TBox
    /// named-class clashes.
    pub fn report(&self) -> Report {
        self.hierarchy.report()
    }
}

/// Classifies the EL+⊥ TBox in `triples` AND internalizes its ABox (`ClassAssertion` /
/// `ObjectPropertyAssertion`) as safe-nominal axioms, returning the realised typings,
/// `owl:sameAs` pairs, and whole-ontology consistency verdict. Does NOT mutate the graph; use
/// [`realize_graph`] to materialize the readoff as triples. Single-threaded.
///
/// ```
/// use sparq_core::Graph;
/// use sparq_reason_el::realize;
///
/// // Two disjoint ClassAssertions on one individual ⇒ the ontology is inconsistent
/// // (the OWL WG `DisjointClasses-002` shape — an ABox clash the TBox classifier cannot see).
/// let ttl = r#"
///   @prefix : <http://ex/> .
///   @prefix owl: <http://www.w3.org/2002/07/owl#> .
///   :Boy owl:disjointWith :Girl .
///   :Stewie a :Boy , :Girl .
/// "#;
/// let (dict, triples) = Graph::parse_to_triples(ttl, "turtle").unwrap();
/// let r = realize(&dict, &triples);
/// assert!(r.is_inconsistent());
/// ```
pub fn realize(dict: &Dict, triples: &[[Id; 3]]) -> Realization {
    let ex = extract::extract(dict, triples, ExtractOpts { abox: true });
    let sat = crate::saturate_extracted(&ex);
    let cls = classify::classify(&sat, &ex.names);
    let mut hierarchy = crate::hierarchy_from(cls, &ex.names, ex.report);

    // Whole-ontology inconsistency: an asserted individual forced into ⊥, or a global ⊤ ⊑ ⊥.
    let inconsistent = sat.s[TOP as usize].contains(&BOTTOM)
        || ex
            .names
            .nominals()
            .any(|(_, c)| sat.s[c as usize].contains(&BOTTOM));
    hierarchy.inconsistent = inconsistent;

    let (types, same_as) = if inconsistent {
        // An inconsistent ontology entails everything — surface ONLY the verdict, not a flood.
        (Vec::new(), Vec::new())
    } else {
        readoff(dict, &sat, &ex.names)
    };
    Realization {
        hierarchy,
        types,
        same_as,
    }
}

/// Reads the realised type assertions + `owl:sameAs` pairs off a saturated (CONSISTENT)
/// classification. `owl:Thing` typing is emitted per individual only when `owl:Thing` is interned
/// (it is a trivially-true type; [`realize_graph`] interns it so it is always emitted there).
fn readoff(dict: &Dict, sat: &classify::Saturation, names: &Names) -> (Pairs, Pairs) {
    let owl_thing = lookup(dict, OWL_THING);
    // concept → individual dict id, for the sameAs reverse mapping.
    let rev: FxHashMap<Concept, Id> = names.nominals().map(|(id, c)| (c, id)).collect();
    let mut types: Pairs = Vec::new();
    let mut same_as: Pairs = Vec::new();
    for (ind, nc) in names.nominals() {
        if owl_thing != NO_ID {
            types.push((ind, owl_thing));
        }
        for &d in &sat.s[nc as usize] {
            if d == nc || d == TOP || d == BOTTOM {
                continue;
            }
            if names.is_nominal(d) {
                if let Some(&other) = rev.get(&d) {
                    if other != ind {
                        // `{other} ∈ S({ind})` ⟹ `ind = other`; equality is symmetric, so both
                        // directions are sound entailments.
                        same_as.push((ind, other));
                        same_as.push((other, ind));
                    }
                }
            } else if let Some(cd) = names.dict_of(d) {
                types.push((ind, cd));
            }
        }
    }
    types.sort_unstable();
    types.dedup();
    same_as.sort_unstable();
    same_as.dedup();
    (types, same_as)
}

/// The report of a materializing [`realize_graph`] run: the base [`Report`] (skipped assertions,
/// unsatisfiable classes), the whole-ontology `inconsistent` verdict, and how many NEW readoff
/// rows were appended. When `inconsistent`, NO realisation rows are emitted (honest — the verdict
/// is the surface, not an everything-entailed flood).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AboxReport {
    /// The extraction/classification report (`skipped_assertions`, `unsatisfiable_classes`, …).
    pub report: Report,
    /// Whole-ontology inconsistency (`{a} ⊑ ⊥` or `⊤ ⊑ ⊥`).
    pub inconsistent: bool,
    /// NEW `a rdf:type C` triples appended.
    pub emitted_type_assertions: usize,
    /// NEW `a owl:sameAs b` triples appended.
    pub emitted_same_as: usize,
}

/// Classifies + realises the ABox and MATERIALIZES the readoff IN PLACE: appends every derived
/// `(a rdf:type C)` (including `a rdf:type owl:Thing`, which is interned so it is always emitted)
/// and `(a owl:sameAs b)` not already present, interning `rdf:type` / `owl:sameAs` / `owl:Thing`
/// as needed. Whole-ontology inconsistency is surfaced via [`AboxReport::inconsistent`] (NOT as a
/// triple — the caller decides how to flag it), matching the TBox `classify_graph` convention of
/// reporting clashes rather than emitting them. When inconsistent, NO rows are appended.
///
/// Idempotent: a second call adds nothing. The emitted edges are ordinary triples queryable by
/// plain BGP eval — the same `(Dict, triples)` seam the RL `scm-*` / EL `classify_graph` output
/// uses.
pub fn realize_graph(dict: &mut Dict, triples: &mut Vec<[Id; 3]>) -> AboxReport {
    // Intern the readoff vocabulary FIRST so `realize` sees `owl:Thing` and emits its typing for
    // every individual (a trivially-true type the WG `*-Ontology-001` conclusions assert).
    let rdf_type = dict.intern_iri(RDF_TYPE);
    let _owl_thing = dict.intern_iri(OWL_THING);
    let owl_same_as = dict.intern_iri(OWL_SAME_AS);

    let r = realize(dict, triples);
    let inconsistent = r.is_inconsistent();
    let report = r.report();

    let mut present: FxHashSet<[Id; 3]> = triples.iter().copied().collect();
    let mut emitted_type_assertions = 0usize;
    let mut emitted_same_as = 0usize;
    if !inconsistent {
        for &(ind, cls) in r.type_assertions() {
            let t = [ind, rdf_type, cls];
            if present.insert(t) {
                triples.push(t);
                emitted_type_assertions += 1;
            }
        }
        for &(a, b) in r.same_as() {
            let t = [a, owl_same_as, b];
            if present.insert(t) {
                triples.push(t);
                emitted_same_as += 1;
            }
        }
    }
    AboxReport {
        report,
        inconsistent,
        emitted_type_assertions,
        emitted_same_as,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Classifier;
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
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    "#;

    // --- WebOnt-Ontology-001: rdf:type owl:Thing (+ equivalentClass) realisation readoff. -----
    #[test]
    fn webont_ontology_001_thing_typing_and_equivalence() {
        // Car ≡ Automobile; car : Car, owl:Thing ; auto : Automobile, owl:Thing.
        let ttl = format!(
            "{PRE}
             :Car owl:equivalentClass :Automobile .
             :car a :Car , owl:Thing .
             :auto a :Automobile , owl:Thing ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(!r.is_inconsistent());
        let (car, auto) = (iri(&dict, "http://ex/car"), iri(&dict, "http://ex/auto"));
        let (thing, cls_car, cls_auto) = (
            iri(&dict, "http://www.w3.org/2002/07/owl#Thing"),
            iri(&dict, "http://ex/Car"),
            iri(&dict, "http://ex/Automobile"),
        );
        let ty = r.type_assertions();
        // The conclusion: both individuals are instances of owl:Thing ...
        assert!(ty.contains(&(car, thing)), "car rdf:type owl:Thing");
        assert!(ty.contains(&(auto, thing)), "auto rdf:type owl:Thing");
        // ... and Car ≡ Automobile makes each an instance of BOTH named classes.
        assert!(ty.contains(&(car, cls_car)) && ty.contains(&(car, cls_auto)));
        assert!(ty.contains(&(auto, cls_car)) && ty.contains(&(auto, cls_auto)));
    }

    // --- DisjointClasses-002: an ABox-driven inconsistency (disjoint dual ClassAssertion). -----
    #[test]
    fn disjoint_classes_002_abox_inconsistency() {
        let ttl = format!(
            "{PRE}
             :Boy owl:disjointWith :Girl .
             :Stewie a :Boy , :Girl ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(r.is_inconsistent(), "Stewie ∈ Boy ⊓ Girl ⊑ ⊥ ⇒ inconsistent");
        assert!(r.type_assertions().is_empty(), "no realisation for an inconsistent ontology");
        // The load-bearing invariant: Boy/Girl are each individually SATISFIABLE — the clash is
        // ABox-only, so the TBox classifier (byte-identical here) flags NO unsatisfiable class.
        let h = Classifier::classify(&dict, &triples);
        assert!(h.unsatisfiable_classes().is_empty());
        assert!(!h.is_inconsistent(), "TBox path never decides whole-ontology consistency");
    }

    // --- New-Feature-BottomObjectProperty-001: an ABox instance of ∃⊥.⊤ ⊑ ⊥. -----------------
    #[test]
    fn bottom_object_property_001_empty_role_clash() {
        // i : [ ∃owl:bottomObjectProperty.owl:Thing ] — owl:bottomObjectProperty is empty, so
        // the existential is unsatisfiable and the asserted instance forces inconsistency.
        let ttl = format!(
            "{PRE}
             :i a [ a owl:Restriction ;
                    owl:onProperty owl:bottomObjectProperty ;
                    owl:someValuesFrom owl:Thing ] ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(r.is_inconsistent(), "∃⊥.⊤ is empty ⇒ {{i}} ⊑ ⊥ ⇒ inconsistent");
    }

    // --- WebOnt-Restriction-001: an ABox instance of ∃op.owl:Nothing (CR5 clash). -------------
    #[test]
    fn webont_restriction_001_existential_bottom_filler() {
        // a, b : [ ∃op.owl:Nothing ] — a someValuesFrom into ⊥ is empty (CR5), so the asserted
        // instances force inconsistency, yet op / the anonymous class name no NAMED unsat class.
        let ttl = format!(
            "{PRE}
             :op a owl:ObjectProperty .
             :a a [ a owl:Restriction ; owl:onProperty :op ; owl:someValuesFrom owl:Nothing ] .
             :b a [ a owl:Restriction ; owl:onProperty :op ; owl:someValuesFrom owl:Nothing ] ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(r.is_inconsistent(), "∃op.⊥ ⊑ ⊥ (CR5) ⇒ asserted instances ⇒ inconsistent");
        let h = Classifier::classify(&dict, &triples);
        assert!(h.unsatisfiable_classes().is_empty(), "no NAMED class is unsatisfiable");
    }

    // --- WebOnt-Thing-003: a global ⊤ ⊑ ⊥ inconsistency, no individuals needed. ----------------
    #[test]
    fn webont_thing_003_global_top_bottom() {
        let ttl = format!("{PRE} owl:Thing owl:equivalentClass owl:Nothing .");
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(r.is_inconsistent(), "⊤ ≡ ⊥ ⇒ ⊥ ∈ S(⊤) ⇒ global inconsistency");
        // The TBox classifier, by its documented contract, decides only named-class satisfiability
        // and flags nothing (⊤/⊥ are excluded from that surface).
        let h = Classifier::classify(&dict, &triples);
        assert!(!h.is_inconsistent());
    }

    // --- ObjectPropertyAssertion internalization: {a} ⊑ ∃p.{b}, threaded through a TBox axiom. -
    #[test]
    fn object_property_assertion_types_through_existential() {
        // p some Parent ⊑ Fond ; alice p bob, bob : Parent  ⊨  alice : Fond  (CR4 through {bob}).
        let ttl = format!(
            "{PRE}
             [ a owl:Restriction ; owl:onProperty :p ; owl:someValuesFrom :Parent ]
                 rdfs:subClassOf :Fond .
             :bob a :Parent .
             :alice :p :bob ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(!r.is_inconsistent());
        let (alice, fond) = (iri(&dict, "http://ex/alice"), iri(&dict, "http://ex/Fond"));
        assert!(
            r.type_assertions().contains(&(alice, fond)),
            "alice ∈ ∃p.Parent ⊑ Fond via the asserted p-edge to bob : Parent"
        );
    }

    // --- DataPropertyAssertion is a fail-closed counted skip (cdomain rescue deferred). --------
    #[test]
    fn data_property_assertion_is_a_counted_skip() {
        let ttl = format!(
            "{PRE}
             :alice a :Person .
             :alice :age 30 ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(!r.is_inconsistent());
        assert_eq!(r.report().skipped_assertions, 1, "the data-property assertion is skipped");
        // The class assertion still classifies the individual.
        let (alice, person) = (iri(&dict, "http://ex/alice"), iri(&dict, "http://ex/Person"));
        assert!(r.type_assertions().contains(&(alice, person)));
    }

    // --- A non-EL ClassAssertion class expression stays a counted skip (fail-closed). ----------
    #[test]
    fn non_el_class_assertion_is_skipped() {
        let ttl = format!(
            "{PRE}
             :x a [ owl:unionOf ( :A :B ) ] .
             :y a :C ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(r.report().skipped_assertions >= 1, "the unionOf class assertion is skipped");
        let (y, c) = (iri(&dict, "http://ex/y"), iri(&dict, "http://ex/C"));
        assert!(r.type_assertions().contains(&(y, c)), "the EL class assertion still classifies");
    }

    // --- ObjectPropertyAssertion with a structural bnode object is a counted skip. -------------
    // [OPUS-4.8] sq-pbz04.2.5: previously the case was sound-and-fail-closed but untallied;
    // this test pins that it is NOW counted in `skipped_assertions` rather than silently dropped.
    #[test]
    fn object_property_assertion_structural_bnode_object_is_counted_skip() {
        // :alice :p [ a owl:Restriction ; owl:onProperty :q ; owl:someValuesFrom :D ] .
        // The object blank node is structural (typed owl:Restriction, has onProperty/svf edges).
        // The subject is a plain IRI individual, so the assertion is an OPA — but the object
        // cannot be interned as a safe nominal. Fail-closed: counted as a skip, never guessed.
        let ttl = format!(
            "{PRE}
             :alice :p [ a owl:Restriction ;
                         owl:onProperty :q ;
                         owl:someValuesFrom :D ] .
             :alice a :Person ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert_eq!(
            r.report().skipped_assertions,
            1,
            "the OPA with a structural-bnode object must be counted as a skip"
        );
        // The ClassAssertion still classifies alice correctly.
        let (alice, person) = (iri(&dict, "http://ex/alice"), iri(&dict, "http://ex/Person"));
        assert!(
            r.type_assertions().contains(&(alice, person)),
            "alice rdf:type Person still realized"
        );
    }

    // --- realize_graph materializes the readoff and is idempotent. -----------------------------
    #[test]
    fn realize_graph_materializes_typings_and_is_idempotent() {
        let ttl = format!(
            "{PRE}
             :A rdfs:subClassOf :B .
             :i a :A ."
        );
        let (mut dict, mut triples) = parse(&ttl);
        let before = triples.len();
        let rep = realize_graph(&mut dict, &mut triples);
        assert!(!rep.inconsistent);
        assert!(rep.emitted_type_assertions >= 1, "at least i rdf:type A/B/owl:Thing emitted");
        assert!(triples.len() > before);
        let (i, a, b) = (
            iri(&dict, "http://ex/i"),
            iri(&dict, "http://ex/A"),
            iri(&dict, "http://ex/B"),
        );
        let rdf_type = iri(&dict, RDF_TYPE);
        assert!(triples.contains(&[i, rdf_type, a]), "i rdf:type A materialized");
        assert!(triples.contains(&[i, rdf_type, b]), "i rdf:type B (derived) materialized");
        let rep2 = realize_graph(&mut dict, &mut triples);
        assert_eq!(rep2.emitted_type_assertions, 0, "second call is idempotent");
        assert_eq!(rep2.emitted_same_as, 0);
    }

    // --- realize_graph emits NOTHING for an inconsistent ontology (verdict only). --------------
    #[test]
    fn realize_graph_emits_nothing_when_inconsistent() {
        let ttl = format!(
            "{PRE}
             :Boy owl:disjointWith :Girl .
             :Stewie a :Boy , :Girl ."
        );
        let (mut dict, mut triples) = parse(&ttl);
        let before = triples.len();
        let rep = realize_graph(&mut dict, &mut triples);
        assert!(rep.inconsistent);
        assert_eq!(rep.emitted_type_assertions, 0);
        assert_eq!(rep.emitted_same_as, 0);
        assert_eq!(triples.len(), before, "no rows appended for an inconsistent ontology");
    }

    // --- owl:sameAs readoff from a derived nominal subsumption (singleton oneOf ClassAssertion). -
    #[test]
    fn same_as_readoff_from_singleton_one_of_assertion() {
        // `a rdf:type { b }` (a singleton `owl:oneOf` enumeration) ⇒ `{a} ⊑ {b}` ⇒ a = b.
        let ttl = format!(
            "{PRE}
             :a a [ owl:oneOf ( :b ) ] ."
        );
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(!r.is_inconsistent());
        let (a, b) = (iri(&dict, "http://ex/a"), iri(&dict, "http://ex/b"));
        let sa = r.same_as();
        assert!(sa.contains(&(a, b)), "a owl:sameAs b");
        assert!(sa.contains(&(b, a)), "owl:sameAs is emitted symmetrically");
    }

    // --- An empty / TBox-only graph realises to nothing and is consistent. ----------------------
    #[test]
    fn tbox_only_graph_is_consistent_with_empty_realisation() {
        let ttl = format!("{PRE} :A rdfs:subClassOf :B .");
        let (dict, triples) = parse(&ttl);
        let r = realize(&dict, &triples);
        assert!(!r.is_inconsistent());
        assert!(r.type_assertions().is_empty());
        assert!(r.same_as().is_empty());
    }
}
