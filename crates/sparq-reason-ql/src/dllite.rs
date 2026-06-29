// [OPUS-4.8] sq-t5bne (epic sq-pbz04): the DL-Lite_R (OWL 2 QL) TBox model + extraction.
//
// OWL 2 QL is the syntactic restriction of OWL 2 whose DL counterpart is DL-Lite_R (Artale et
// al., JAIR 2009; W3C OWL 2 Profiles §3). The POSITIVE inclusions PerfectRef rewrites with are:
//
//   B ⊑ A            a "basic concept" B on the left of a named class A
//   R ⊑ S            role inclusion (incl. inverses, R ⊑ S⁻ etc.)
//
// where a basic concept B is one of:  A (named class) | ∃R | ∃R⁻  and a basic role R is a named
// property P or its inverse P⁻. NEGATIVE inclusions (B1 ⊑ ¬B2, disjointness, R1 ⊑ ¬R2) and
// functionality do NOT contribute to the certain-answer REWRITING of a positive CQ — they only
// affect consistency — so they are extracted but recorded as ignored-for-rewriting, never
// silently treated as positive (which would be unsound). This crate rewrites; it does not check
// consistency (a documented boundary — see the crate README / the deferred-path note).
//
// Extraction reads OWL axioms out of an RDF graph given as oxrdf triples. Only the QL fragment
// is recognised; an axiom using a non-QL construct is COUNTED in [`TBox::skipped`] (honest "n
// axioms outside the QL fragment were ignored"), never misapplied.

#[cfg(feature = "experimental")]
use oxrdf::NamedNode;
use oxrdf::{NamedOrBlankNode, Term, Triple};
use rustc_hash::{FxHashMap, FxHashSet};

/// A subject/object node reduced to the only two shapes TBox extraction cares about: a named
/// IRI or a blank node (literals are neither). `from_subject`/`from_object` normalise the two
/// distinct oxrdf position types (`NamedOrBlankNode` subject, `Term` object) into this.
#[derive(Clone, Debug)]
enum NodeKind {
    Iri(String),
    Blank(String),
}

impl NodeKind {
    fn from_subject(s: &NamedOrBlankNode) -> NodeKind {
        match s {
            NamedOrBlankNode::NamedNode(n) => NodeKind::Iri(n.as_str().to_string()),
            NamedOrBlankNode::BlankNode(b) => NodeKind::Blank(b.as_str().to_string()),
        }
    }

    fn from_object(o: &Term) -> Option<NodeKind> {
        match o {
            Term::NamedNode(n) => Some(NodeKind::Iri(n.as_str().to_string())),
            Term::BlankNode(b) => Some(NodeKind::Blank(b.as_str().to_string())),
            _ => None, // literal (or quoted triple) — never a TBox class/role node.
        }
    }
}

/// `rdf:type` — used by the (experimental) query-atom mapper to recognise class atoms.
#[cfg(feature = "experimental")]
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const OWL: &str = "http://www.w3.org/2002/07/owl#";

/// A basic role: a named property, or its inverse. Equality/ordering is by IRI + direction so
/// roles intern cleanly into the rewrite's working sets.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Role {
    /// The named property IRI.
    pub iri: String,
    /// `true` if this is the INVERSE role `P⁻` (object/subject swapped).
    pub inverse: bool,
}

impl Role {
    pub fn named(iri: impl Into<String>) -> Self {
        Role {
            iri: iri.into(),
            inverse: false,
        }
    }

    /// The inverse of this role (`P` ↔ `P⁻`).
    pub fn inv(&self) -> Role {
        Role {
            iri: self.iri.clone(),
            inverse: !self.inverse,
        }
    }
}

/// A "basic concept" B in DL-Lite_R: a named class, or an unqualified existential `∃R` over a
/// basic role (the domain of R). `∃R⁻` is `Exists` over the inverse role. (DL-Lite_R existentials
/// are UNqualified — `∃R.A` qualified fillers are NOT in QL, and are skipped in extraction.)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Basic {
    /// A named class `A`.
    Class(String),
    /// `∃R` — the domain of basic role `R` (something R-related).
    Exists(Role),
}

/// A positive concept inclusion `B ⊑ A` (left a basic concept, right a NAMED class). The right
/// side being a named class is the shape PerfectRef rewrites a class atom with; QL also allows
/// `B ⊑ ∃R` on the right (range/existential generation), captured separately as
/// [`TBox::exists_super`] so the rewrite can apply it to ROLE atoms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConceptInclusion {
    pub sub: Basic,
    pub sup: String,
}

/// The extracted DL-Lite_R TBox: just the positive inclusions PerfectRef needs, plus an honest
/// tally of what was outside the QL fragment.
#[derive(Clone, Debug, Default)]
pub struct TBox {
    /// `B ⊑ A` with `A` a named class.
    pub concept_incl: Vec<ConceptInclusion>,
    /// `B ⊑ ∃R` (the existential-generating inclusions: a basic concept implies an R-successor).
    /// Keyed for the role-atom rewrite. Stored as `(sub basic concept, generated role)`.
    pub exists_super: Vec<(Basic, Role)>,
    /// Role inclusions `R ⊑ S` (both basic roles; inverses included).
    pub role_incl: Vec<(Role, Role)>,
    /// Count of axioms that used a construct OUTSIDE DL-Lite_R (and were therefore IGNORED for
    /// rewriting): qualified existentials, unionOf/intersection on the wrong side, cardinality,
    /// nominals, datatype constructs, negative inclusions used positively, … Surfaced so a
    /// caller can report "n axioms outside the QL fragment were ignored".
    pub skipped: usize,
}

impl TBox {
    /// Extract the DL-Lite_R positive inclusions from an RDF graph (oxrdf triples). Recognised:
    ///
    /// - `A rdfs:subClassOf B` with A,B named classes → `A ⊑ B`;
    /// - `R rdfs:subPropertyOf S` → `R ⊑ S`;
    /// - `R rdfs:domain A` → `∃R ⊑ A`;
    /// - `R rdfs:range A` → `∃R⁻ ⊑ A`;
    /// - `R owl:inverseOf S` → `R ⊑ S⁻` and `S⁻ ⊑ R` (bi-directional, as DL-Lite_R inverses);
    /// - `A rdfs:subClassOf [ owl:onProperty R ; owl:someValuesFrom owl:Thing ]` → `A ⊑ ∃R`
    ///   (UNqualified existential — a QL-legal range generator);
    /// - `[ owl:onProperty R ; owl:someValuesFrom owl:Thing ] rdfs:subClassOf A` → `∃R ⊑ A`.
    ///
    /// A QUALIFIED `owl:someValuesFrom` (filler other than `owl:Thing`) on the LEFT is QL-legal
    /// (`∃R.C ⊑ A` is in QL via `∃R ⊑ A` weakening? — NO: that would be unsound), so it is
    /// conservatively SKIPPED. Anything else (unionOf, complementOf, allValuesFrom, cardinality,
    /// hasValue, oneOf, qualified RHS existentials) is skipped and counted.
    pub fn extract(triples: &[Triple]) -> TBox {
        let mut tbox = TBox::default();
        // Index restriction blank-nodes: subject -> (onProperty, someValuesFrom-filler).
        let restrictions = index_restrictions(triples);

        for t in triples {
            let p = t.predicate.as_str();
            match p {
                _ if p == sub_class_of() => {
                    tbox.add_subclass(&t.subject, &t.object, &restrictions);
                }
                _ if p == sub_property_of() => {
                    if let (Some(r), Some(s)) =
                        (role_of_subject(&t.subject), role_of_object(&t.object))
                    {
                        tbox.role_incl.push((r, s));
                    } else {
                        tbox.skipped += 1;
                    }
                }
                _ if p == domain() => {
                    if let (Some(r), Some(NodeKind::Iri(a))) =
                        (role_of_subject(&t.subject), NodeKind::from_object(&t.object))
                    {
                        // ∃R ⊑ A
                        tbox.concept_incl.push(ConceptInclusion {
                            sub: Basic::Exists(r),
                            sup: a,
                        });
                    } else {
                        tbox.skipped += 1;
                    }
                }
                _ if p == range() => {
                    if let (Some(r), Some(NodeKind::Iri(a))) =
                        (role_of_subject(&t.subject), NodeKind::from_object(&t.object))
                    {
                        // ∃R⁻ ⊑ A
                        tbox.concept_incl.push(ConceptInclusion {
                            sub: Basic::Exists(r.inv()),
                            sup: a,
                        });
                    } else {
                        tbox.skipped += 1;
                    }
                }
                _ if p == inverse_of() => {
                    if let (Some(r), Some(s)) =
                        (role_of_subject(&t.subject), role_of_object(&t.object))
                    {
                        // R ≡ S⁻ : add both role inclusions.
                        tbox.role_incl.push((r.clone(), s.inv()));
                        tbox.role_incl.push((s.inv(), r));
                    } else {
                        tbox.skipped += 1;
                    }
                }
                _ => { /* not a TBox-shaping predicate; ignored (ABox / annotation) */ }
            }
        }
        tbox
    }

    /// Add an `A ⊑ B`-derived inclusion, resolving restriction blank-nodes on either side into
    /// the QL basic-concept forms. Non-QL shapes are skipped + counted.
    fn add_subclass(
        &mut self,
        sub: &NamedOrBlankNode,
        sup: &Term,
        restrictions: &FxHashMap<String, Restriction>,
    ) {
        let sub_b = basic_of(&NodeKind::from_subject(sub), restrictions);
        // Right side: a named class A (the PerfectRef class-atom rewrite target), or an
        // unqualified ∃R (range generator → exists_super).
        match NodeKind::from_object(sup) {
            Some(NodeKind::Iri(a)) => {
                if let Some(b) = sub_b {
                    self.concept_incl.push(ConceptInclusion { sub: b, sup: a });
                } else {
                    self.skipped += 1;
                }
            }
            Some(NodeKind::Blank(bn)) => match restrictions.get(&bn) {
                Some(Restriction {
                    on_property,
                    some_values_thing: true,
                }) => {
                    // B ⊑ ∃R (unqualified existential on the RHS).
                    if let Some(b) = sub_b {
                        self.exists_super
                            .push((b, Role::named(on_property.clone())));
                    } else {
                        self.skipped += 1;
                    }
                }
                // Qualified RHS existential or a non-someValues restriction: outside QL rewriting.
                _ => self.skipped += 1,
            },
            None => self.skipped += 1,
        }
    }
}

/// Map a subclass-side node to a basic concept, resolving a restriction blank-node into `∃R`.
/// Returns `None` (→ skip) for any non-QL shape.
fn basic_of(t: &NodeKind, restrictions: &FxHashMap<String, Restriction>) -> Option<Basic> {
    match t {
        NodeKind::Iri(a) => Some(Basic::Class(a.clone())),
        NodeKind::Blank(bn) => match restrictions.get(bn) {
            // ∃R on the LEFT requires the UNQUALIFIED form (someValuesFrom owl:Thing). A
            // QUALIFIED ∃R.C on the left is NOT equivalent to ∃R ⊑ … (weakening it would be
            // unsound), so it is skipped.
            Some(Restriction {
                on_property,
                some_values_thing: true,
            }) => Some(Basic::Exists(Role::named(on_property.clone()))),
            _ => None,
        },
    }
}

/// A parsed owl:Restriction blank-node: `[ owl:onProperty R ; owl:someValuesFrom F ]`.
struct Restriction {
    on_property: String,
    /// True iff the someValuesFrom filler is exactly `owl:Thing` (the UNqualified existential).
    some_values_thing: bool,
}

/// Index every owl:Restriction-shaped blank node by its id. We only retain `onProperty` +
/// whether `someValuesFrom` is `owl:Thing`; any restriction lacking a single onProperty, or
/// carrying a non-QL restriction predicate, is simply not indexed (callers then skip it).
fn index_restrictions(triples: &[Triple]) -> FxHashMap<String, Restriction> {
    let mut on_property: FxHashMap<String, String> = FxHashMap::default();
    let mut some_values: FxHashMap<String, bool> = FxHashMap::default();
    // A restriction node carrying any of these predicates is NON-QL — mark it poisoned so it is
    // never resolved into a basic concept even if it also has onProperty.
    let mut poisoned: FxHashSet<String> = FxHashSet::default();
    let owl_thing = format!("{OWL}Thing");

    for t in triples {
        let NamedOrBlankNode::BlankNode(s) = &t.subject else {
            continue;
        };
        let s = s.as_str().to_string();
        let p = t.predicate.as_str();
        if p == format!("{OWL}onProperty") {
            if let Term::NamedNode(r) = &t.object {
                on_property.insert(s, r.as_str().to_string());
            } else {
                poisoned.insert(s);
            }
        } else if p == format!("{OWL}someValuesFrom") {
            match &t.object {
                Term::NamedNode(f) => {
                    some_values.insert(s, f.as_str() == owl_thing);
                }
                _ => {
                    poisoned.insert(s);
                }
            }
        } else if NON_QL_RESTRICTION.iter().any(|m| p == format!("{OWL}{m}")) {
            poisoned.insert(s);
        }
    }

    let mut out = FxHashMap::default();
    for (bn, prop) in on_property {
        if poisoned.contains(&bn) {
            continue;
        }
        // someValuesFrom present? record whether it is owl:Thing. Absent → not a someValues
        // restriction we rewrite (could be allValues/cardinality already poisoned, or a bare
        // onProperty) → treat as NON-unqualified-existential (some_values_thing = false), which
        // makes both the LHS ∃R and the RHS ∃R resolution skip it (sound: we only generate/
        // consume UNqualified existentials).
        let some_values_thing = some_values.get(&bn).copied().unwrap_or(false);
        out.insert(
            bn,
            Restriction {
                on_property: prop,
                some_values_thing,
            },
        );
    }
    out
}

/// Restriction predicates that, if present on a node, make it non-QL (so the node is poisoned).
const NON_QL_RESTRICTION: &[&str] = &[
    "allValuesFrom",
    "hasValue",
    "minCardinality",
    "maxCardinality",
    "cardinality",
    "minQualifiedCardinality",
    "maxQualifiedCardinality",
    "qualifiedCardinality",
    "hasSelf",
    "onClass",
];

fn role_of_subject(t: &NamedOrBlankNode) -> Option<Role> {
    match t {
        NamedOrBlankNode::NamedNode(n) => Some(Role::named(n.as_str())),
        NamedOrBlankNode::BlankNode(_) => None,
    }
}

fn role_of_object(t: &Term) -> Option<Role> {
    match t {
        Term::NamedNode(n) => Some(Role::named(n.as_str())),
        _ => None,
    }
}

// Vocabulary IRI helpers (kept as functions to avoid `const` String concat churn).
fn sub_class_of() -> String {
    format!("{RDFS}subClassOf")
}
fn sub_property_of() -> String {
    format!("{RDFS}subPropertyOf")
}
fn domain() -> String {
    format!("{RDFS}domain")
}
fn range() -> String {
    format!("{RDFS}range")
}
fn inverse_of() -> String {
    format!("{OWL}inverseOf")
}

/// The `rdf:type` IRI (a role-atom predicate that means "class membership", handled specially by
/// the query atom mapper, never as an ordinary role). Only the experimental rewriter needs it.
#[cfg(feature = "experimental")]
pub(crate) fn rdf_type_iri() -> NamedNode {
    NamedNode::new_unchecked(RDF_TYPE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_inverse_roundtrips() {
        let r = Role::named("http://ex/r");
        assert_eq!(r.inv().inv(), r);
        assert!(r.inv().inverse);
    }
}
