// [FABLE-5] sq-pbz04.4.1 (epic sq-pbz04.4, design record
// research/owl2-direct-semantics-scoping.md §L1): the FAIL-CLOSED reverse RDF mapping.
//
// 🤖 SPARQ agent. Reads an OWL ontology in the ALCH fragment out of the `(Dict, [[Id; 3]])`
// substrate and returns a [`crate::model::Ontology`] — OR a typed [`ExtractError`] the moment
// it meets a triple it cannot map per the W3C OWL 2 *Mapping to RDF Graphs* tables restricted
// to ALCH.
//
// SOUNDNESS INVARIANT (the reason this is fail-closed, not skip-and-count):
// -------------------------------------------------------------------------
// The downstream consistency/satisfiability checker (L3/L4) must NEVER reason over a graph it
// only PARTIALLY understood. A single dropped axiom can be exactly the constraint that makes
// an ontology inconsistent (so silently dropping it yields an unsound "consistent" verdict) or
// exactly the assertion whose absence makes it consistent (an unsound "inconsistent" verdict).
// So — unlike `sparq-reason-el`/`-ql`, which extract what they can and TALLY the rest — this
// mapping aborts the WHOLE extraction with a structured error on the first out-of-fragment or
// malformed construct. "Understood in full, or refused" is the contract the checker relies on.
//
// The guarantee is realised by CLOSED classification: every triple is routed into exactly one
// of {recognised axiom, recognised class-expression backbone, ignorable declaration/annotation,
// ground assertion} or it produces an [`ExtractError`]. Nothing falls through silently.

use crate::model::{Axiom, ClassExpression, ObjectPropertyExpression, Ontology};
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{is_inline, Dict, Id, TermParts, NO_ID};

const OWL: &str = "http://www.w3.org/2002/07/owl#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// The structured, fail-closed rejection taxonomy of the reverse RDF mapping. Each variant is
/// a distinct reason the graph could not be mapped into the ALCH structural model; the payload
/// is a human-readable diagnostic (the offending IRI / triple), NOT part of the taxonomy —
/// callers and tests should match on the VARIANT.
///
/// Deferral rationale + unlock paths for every out-of-fragment construct: design record §5.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtractError {
    /// A recognised OWL/RDF construct that carries logical meaning OUTSIDE the ALCH fragment:
    /// cardinality / qualified cardinality, nominals (`owl:oneOf`/`owl:hasValue`),
    /// `owl:hasSelf`, inverse properties (`owl:inverseOf`), property characteristics
    /// (`owl:TransitiveProperty` / `owl:FunctionalProperty` / symmetry / reflexivity / …),
    /// `owl:sameAs` / `owl:differentFrom`, property chains, keys, `owl:disjointUnionOf`,
    /// equivalent / disjoint properties, and the n-ary `owl:AllDifferent` /
    /// `owl:AllDisjoint*` / `owl:NegativePropertyAssertion` collections.
    OutOfFragment(String),
    /// A datatype / data-property construct — the L1 ALCH fragment has NO concrete domain: a
    /// literal in a class or individual position, an `owl:DatatypeProperty` assertion, or a
    /// data-range restriction (`owl:onDataRange` / `owl:onDatatype` / `owl:withRestrictions`).
    DataConstruct(String),
    /// An RDF list backing a boolean class expression (`owl:intersectionOf` / `owl:unionOf`)
    /// is ill-formed: unterminated (no `rdf:nil`), cyclic, branching (a cell with two
    /// `rdf:first`/`rdf:rest` edges), or empty.
    MalformedList(String),
    /// A class-expression node is structurally ill-formed: an `owl:Restriction` missing its
    /// `owl:onProperty` or its filler, a `owl:complementOf` / restriction carrying two
    /// conflicting shape predicates, or an over-deep / cyclic class-expression nesting.
    MalformedClassExpression(String),
    /// A triple that cannot be mapped SOUNDLY without a declaration — e.g. an undeclared
    /// predicate whose object is an IRI/blank (indistinguishable between a role assertion and
    /// an annotation), an `rdf:type` whose object is neither a recognised class expression nor
    /// a known declaration meta-class, or an RDF 1.2 triple term (quoted triple) anywhere.
    /// Fail-closed: refused rather than guessed.
    Unclassifiable(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Positional args (not inline) — avoids the CodeQL rust/unused-variable false
            // positive on `format!("{x}")` (SPARQ agent CodeQL note).
            ExtractError::OutOfFragment(d) => {
                write!(f, "construct outside the ALCH fragment: {}", d)
            }
            ExtractError::DataConstruct(d) => {
                write!(f, "datatype / data-property construct (no concrete domain in L1): {}", d)
            }
            ExtractError::MalformedList(d) => write!(f, "malformed RDF list: {}", d),
            ExtractError::MalformedClassExpression(d) => {
                write!(f, "malformed class expression: {}", d)
            }
            ExtractError::Unclassifiable(d) => {
                write!(f, "triple not soundly classifiable in the ALCH fragment: {}", d)
            }
        }
    }
}

impl std::error::Error for ExtractError {}

/// Maximum class-expression nesting depth. A well-formed OWL ontology never approaches this;
/// the cap turns a maliciously deep or (via a non-list back-edge) cyclic class-expression
/// encoding into a [`ExtractError::MalformedClassExpression`] instead of a stack overflow.
const MAX_CE_DEPTH: usize = 512;

/// Reads the ALCH-fragment ontology encoded in `triples` (keyed through `dict`).
///
/// Returns the structural [`Ontology`] on success, or the FIRST [`ExtractError`] encountered —
/// the mapping is fail-closed (see the module-level soundness invariant): a graph is mapped in
/// full or refused, never mapped in part.
///
/// # Errors
/// Returns an [`ExtractError`] whenever the graph contains a construct outside the ALCH
/// fragment, a datatype / data-property construct, a malformed list or class expression, or a
/// triple that cannot be classified without a declaration.
pub fn extract(dict: &Dict, triples: &[[Id; 3]]) -> Result<Ontology, ExtractError> {
    let v = Vocab::intern(dict);
    // [OPUS-4.8] Fix 1 + Fix 3: Index::build now returns Err on a branching list,
    // duplicate backbone value, or cross-type punned declaration.
    let idx = Index::build(triples, &v)?;
    let mut onto = Ontology::new();

    for &[s, p, o] in triples {
        // Class-expression backbone predicates are consumed WHEN the enclosing class node is
        // decoded (via `Index`), and are inert on their own — skip them here. A dangling
        // backbone triple (a restriction/list cell never referenced by an axiom) carries no
        // logical import, so ignoring it is sound.
        if v.backbone.contains(&p) {
            continue;
        }
        // Recognised, mapped, or refused — closed classification (see module docs).
        classify_triple(dict, &v, &idx, s, p, o, &mut onto)?;
    }

    Ok(onto)
}

/// Routes one non-backbone triple: into an [`Axiom`] on `onto`, silently past an ignorable
/// declaration / annotation / header, or into an [`ExtractError`].
fn classify_triple(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    s: Id,
    p: Id,
    o: Id,
    onto: &mut Ontology,
) -> Result<(), ExtractError> {
    // --- Axiom predicates ---------------------------------------------------------------
    if p == v.sub_class_of {
        let sub = decode_class(dict, v, idx, s)?;
        let sup = decode_class(dict, v, idx, o)?;
        onto.axioms.push(Axiom::SubClassOf { sub, sup });
        return Ok(());
    }
    if p == v.equivalent_class {
        let left = decode_class(dict, v, idx, s)?;
        let right = decode_class(dict, v, idx, o)?;
        onto.axioms.push(Axiom::EquivalentClasses(left, right));
        return Ok(());
    }
    if p == v.disjoint_with {
        let left = decode_class(dict, v, idx, s)?;
        let right = decode_class(dict, v, idx, o)?;
        onto.axioms.push(Axiom::DisjointClasses(left, right));
        return Ok(());
    }
    if p == v.sub_property_of {
        let sub = decode_object_property(dict, idx, s)?;
        let sup = decode_object_property(dict, idx, o)?;
        onto.axioms.push(Axiom::SubObjectPropertyOf { sub, sup });
        return Ok(());
    }
    if p == v.domain {
        let property = decode_object_property(dict, idx, s)?;
        let domain = decode_class(dict, v, idx, o)?;
        onto.axioms.push(Axiom::ObjectPropertyDomain { property, domain });
        return Ok(());
    }
    if p == v.range {
        let property = decode_object_property(dict, idx, s)?;
        let range = decode_class(dict, v, idx, o)?;
        onto.axioms.push(Axiom::ObjectPropertyRange { property, range });
        return Ok(());
    }

    // --- rdf:type — declaration, class assertion, or out-of-fragment meta-class ----------
    if p == v.ty {
        return classify_type(dict, v, idx, s, o, onto);
    }

    // --- Recognised out-of-fragment logical / datatype predicates ------------------------
    if let Some(name) = v.out_of_fragment.get(&p) {
        return Err(ExtractError::OutOfFragment(format!(
            "predicate {} on {}",
            name,
            term_iri(dict, s)
        )));
    }
    if let Some(name) = v.data_predicate.get(&p) {
        return Err(ExtractError::DataConstruct(format!(
            "predicate {} on {}",
            name,
            term_iri(dict, s)
        )));
    }

    // --- Ignorable built-in annotations --------------------------------------------------
    if v.annotation.contains(&p) {
        return Ok(());
    }

    // --- Punned predicates: declared as multiple property types — fail-closed ------------
    // [OPUS-4.8] Fix 3: a predicate simultaneously declared as AnnotationProperty AND
    // ObjectProperty (or any other cross-type punning) is Unclassifiable — the downstream
    // checker must never reason over an axiom whose classification was ambiguous.
    if idx.punned_props.contains(&p) {
        return Err(ExtractError::Unclassifiable(format!(
            "predicate {} is declared as multiple property types (ambiguous classification)",
            term_iri(dict, p)
        )));
    }

    // --- Declared annotation properties (from the declaration pass) — ignorable ----------
    if idx.annotation_props.contains(&p) {
        return Ok(());
    }

    // --- Declared datatype properties — a data assertion, out of the concrete-domain-free
    //     ALCH fragment ----------------------------------------------------------------
    if idx.data_props.contains(&p) {
        return Err(ExtractError::DataConstruct(format!(
            "assertion using data property {}",
            term_iri(dict, p)
        )));
    }

    // --- Role assertion: a property known to be an object property, over two individuals --
    if idx.object_props.contains(&p) {
        return role_assertion(dict, idx, s, p, o, onto);
    }

    // --- Otherwise: cannot classify soundly without a declaration -------------------------
    // A literal object looks like a data-property assertion (no concrete domain in L1); an
    // IRI/blank object is ambiguous between a role assertion and an annotation. Either way we
    // refuse rather than guess (fail-closed).
    if is_literal(dict, o) {
        Err(ExtractError::DataConstruct(format!(
            "literal-valued triple with undeclared predicate {}",
            term_iri(dict, p)
        )))
    } else {
        Err(ExtractError::Unclassifiable(format!(
            "undeclared predicate {} (cannot tell a role assertion from an annotation)",
            term_iri(dict, p)
        )))
    }
}

/// Handles an `rdf:type` triple `s a o`.
fn classify_type(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    s: Id,
    o: Id,
    onto: &mut Ontology,
) -> Result<(), ExtractError> {
    // Declaration / structural typings carry no ALCH-logical import — ignore.
    if v.declaration_types.contains(&o) {
        return Ok(());
    }
    // Property characteristics and n-ary collection meta-classes are logical but out of
    // fragment — refuse (they are not silently droppable).
    if let Some(name) = v.out_of_fragment_types.get(&o) {
        return Err(ExtractError::OutOfFragment(format!(
            "rdf:type {} on {}",
            name,
            term_iri(dict, s)
        )));
    }
    // owl:Thing / owl:Nothing as a type are legitimate (trivial / bottoming) class assertions.
    // Otherwise the object must decode as an ALCH class expression → ClassAssertion. A literal
    // object is malformed; an undeclared IRI/blank that does not decode is Unclassifiable.
    if is_literal(dict, o) {
        return Err(ExtractError::MalformedClassExpression(format!(
            "rdf:type with a literal class on {}",
            term_iri(dict, s)
        )));
    }
    let class = decode_class(dict, v, idx, o)?;
    onto.axioms.push(Axiom::ClassAssertion {
        class,
        individual: s,
    });
    Ok(())
}

/// Builds an [`Axiom::ObjectPropertyAssertion`] from `s p o`, or refuses.
fn role_assertion(
    dict: &Dict,
    idx: &Index,
    s: Id,
    p: Id,
    o: Id,
    onto: &mut Ontology,
) -> Result<(), ExtractError> {
    if is_literal(dict, o) {
        // An object property with a literal object is a data assertion / a punning clash — out
        // of the datatype-free ALCH fragment.
        return Err(ExtractError::DataConstruct(format!(
            "object property {} asserted with a literal value",
            term_iri(dict, p)
        )));
    }
    // The operands of a ground role assertion must be individuals, not class-expression
    // backbone nodes (a restriction / list cell used as an individual is malformed).
    if idx.structural_nodes.contains(&s) || idx.structural_nodes.contains(&o) {
        return Err(ExtractError::MalformedClassExpression(format!(
            "role assertion {} over a class-expression node",
            term_iri(dict, p)
        )));
    }
    if is_triple_term(dict, s) || is_triple_term(dict, o) {
        return Err(ExtractError::Unclassifiable(
            "RDF 1.2 triple term as a role-assertion individual".to_string(),
        ));
    }
    onto.axioms.push(Axiom::ObjectPropertyAssertion {
        property: ObjectPropertyExpression::ObjectProperty(p),
        source: s,
        target: o,
    });
    Ok(())
}

/// Decodes the dict node `id` into an ALCH [`ClassExpression`], or refuses. Cycle- and
/// depth-guarded (see [`MAX_CE_DEPTH`]).
fn decode_class(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    id: Id,
) -> Result<ClassExpression, ExtractError> {
    let mut visiting = FxHashSet::default();
    decode_class_inner(dict, v, idx, id, &mut visiting, 0)
}

fn decode_class_inner(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    id: Id,
    visiting: &mut FxHashSet<Id>,
    depth: usize,
) -> Result<ClassExpression, ExtractError> {
    if depth > MAX_CE_DEPTH {
        return Err(ExtractError::MalformedClassExpression(
            "class expression nested beyond the depth cap (cyclic or adversarial)".to_string(),
        ));
    }
    if is_literal(dict, id) {
        return Err(ExtractError::DataConstruct(
            "literal in a class-expression position".to_string(),
        ));
    }
    if is_triple_term(dict, id) {
        return Err(ExtractError::Unclassifiable(
            "RDF 1.2 triple term in a class-expression position".to_string(),
        ));
    }
    if id == v.thing {
        return Ok(ClassExpression::Thing);
    }
    if id == v.nothing {
        return Ok(ClassExpression::Nothing);
    }

    // Reject a node carrying an out-of-fragment class shape (nominal / cardinality / hasSelf /
    // data-range restriction) BEFORE treating it as a named class, so `∃r.{a}` or a cardinality
    // restriction is never mistaken for an opaque named class (which would drop its meaning).
    if let Some(name) = idx.out_of_fragment_shape.get(&id) {
        return Err(ExtractError::OutOfFragment(format!(
            "class node using {}",
            name
        )));
    }
    if let Some(name) = idx.data_shape.get(&id) {
        return Err(ExtractError::DataConstruct(format!(
            "class node using {}",
            name
        )));
    }

    // Boolean / quantifier shapes, detected by the presence of their defining predicate. This
    // fires for BLANK anonymous nodes AND for a NAMED class that carries an inline definition
    // (`:A owl:intersectionOf (…)`): `owl:intersectionOf`/`unionOf`/`complementOf`/a
    // `owl:Restriction` on a class node denotes a COMPLETE equivalence, so inlining the decoded
    // expression at every occurrence of the node is model-preserving (sound). Decoding is
    // deterministic, so a named node inlines to the same expression consistently everywhere.
    let is_inter = idx.intersection_of.contains_key(&id);
    let is_union = idx.union_of.contains_key(&id);
    let is_compl = idx.complement_of.contains_key(&id);
    let is_restr = idx.is_restriction.contains(&id)
        || idx.some_values_from.contains_key(&id)
        || idx.all_values_from.contains_key(&id);
    let shape_count =
        usize::from(is_inter) + usize::from(is_union) + usize::from(is_compl) + usize::from(is_restr);
    if shape_count > 1 {
        return Err(ExtractError::MalformedClassExpression(format!(
            "class node combines multiple class-expression shapes: {}",
            term_iri(dict, id)
        )));
    }
    if shape_count == 0 {
        // A bare IRI/blank with no recognised shape is a NAMED class (an anonymous blank with
        // no class predicates is still a leaf; it carries meaning only through a referencing
        // axiom, which is where soundness is enforced).
        return Ok(ClassExpression::Class(id));
    }

    // Composite node — guard direct/indirect self-reference (e.g. `_:x complementOf _:x`,
    // `:A owl:intersectionOf (:A …)`) so a cyclic encoding fails closed rather than recursing.
    if !visiting.insert(id) {
        return Err(ExtractError::MalformedClassExpression(format!(
            "cyclic class expression at {}",
            term_iri(dict, id)
        )));
    }
    let result = if is_inter {
        let head = idx.intersection_of[&id];
        decode_class_list(dict, v, idx, head, visiting, depth)
            .map(ClassExpression::ObjectIntersectionOf)
    } else if is_union {
        let head = idx.union_of[&id];
        decode_class_list(dict, v, idx, head, visiting, depth).map(ClassExpression::ObjectUnionOf)
    } else if is_compl {
        let operand = idx.complement_of[&id];
        decode_class_inner(dict, v, idx, operand, visiting, depth + 1)
            .map(|inner| ClassExpression::ObjectComplementOf(Box::new(inner)))
    } else {
        decode_restriction(dict, v, idx, id, visiting, depth)
    };
    visiting.remove(&id);
    result
}

/// Decodes an `owl:Restriction` node (`∃R.C` / `∀R.C`) or refuses.
fn decode_restriction(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    id: Id,
    visiting: &mut FxHashSet<Id>,
    depth: usize,
) -> Result<ClassExpression, ExtractError> {
    let prop = idx.on_property.get(&id).copied().ok_or_else(|| {
        ExtractError::MalformedClassExpression(format!(
            "restriction {} missing owl:onProperty",
            term_iri(dict, id)
        ))
    })?;
    let property = decode_object_property(dict, idx, prop)?;
    let some = idx.some_values_from.get(&id).copied();
    let all = idx.all_values_from.get(&id).copied();
    match (some, all) {
        (Some(_), Some(_)) => Err(ExtractError::MalformedClassExpression(format!(
            "restriction {} has both someValuesFrom and allValuesFrom",
            term_iri(dict, id)
        ))),
        (Some(filler), None) => {
            let inner = decode_class_inner(dict, v, idx, filler, visiting, depth + 1)?;
            Ok(ClassExpression::ObjectSomeValuesFrom(property, Box::new(inner)))
        }
        (None, Some(filler)) => {
            let inner = decode_class_inner(dict, v, idx, filler, visiting, depth + 1)?;
            Ok(ClassExpression::ObjectAllValuesFrom(property, Box::new(inner)))
        }
        (None, None) => Err(ExtractError::MalformedClassExpression(format!(
            "restriction {} missing its someValuesFrom/allValuesFrom filler",
            term_iri(dict, id)
        ))),
    }
}

/// Decodes an `owl:intersectionOf`/`owl:unionOf` operand list into a non-empty vector of
/// class expressions, or refuses (malformed list / member).
fn decode_class_list(
    dict: &Dict,
    v: &Vocab,
    idx: &Index,
    head: Id,
    visiting: &mut FxHashSet<Id>,
    depth: usize,
) -> Result<Vec<ClassExpression>, ExtractError> {
    let mut out = Vec::new();
    let mut cursor = head;
    let mut seen = FxHashSet::default();
    loop {
        if cursor == v.rdf_nil {
            break;
        }
        if !seen.insert(cursor) {
            return Err(ExtractError::MalformedList(
                "cyclic rdf:rest chain".to_string(),
            ));
        }
        if seen.len() > MAX_CE_DEPTH {
            return Err(ExtractError::MalformedList(
                "list longer than the depth cap".to_string(),
            ));
        }
        let first = idx.first.get(&cursor).copied().ok_or_else(|| {
            ExtractError::MalformedList(format!(
                "list cell {} missing rdf:first",
                term_iri(dict, cursor)
            ))
        })?;
        let member = decode_class_inner(dict, v, idx, first, visiting, depth + 1)?;
        out.push(member);
        cursor = idx.rest.get(&cursor).copied().ok_or_else(|| {
            ExtractError::MalformedList(format!(
                "list cell {} missing rdf:rest (unterminated list)",
                term_iri(dict, cursor)
            ))
        })?;
    }
    if out.is_empty() {
        return Err(ExtractError::MalformedList(
            "empty operand list (owl:intersectionOf/owl:unionOf require at least one member)"
                .to_string(),
        ));
    }
    Ok(out)
}

/// Decodes a node used in an object-property position. In L1 only a NAMED object property is
/// admitted; an `owl:inverseOf` node (an inverse property expression) is refused
/// (`OutOfFragment`), a literal is refused (`MalformedClassExpression`), and — per
/// model.rs:20-33 — a blank node or RDF 1.2 triple term is refused (`Unclassifiable`).
fn decode_object_property(
    dict: &Dict,
    idx: &Index,
    id: Id,
) -> Result<ObjectPropertyExpression, ExtractError> {
    if is_literal(dict, id) {
        return Err(ExtractError::MalformedClassExpression(
            "literal in an object-property position".to_string(),
        ));
    }
    // [OPUS-4.8] Fix 2: blank nodes and triple terms are not named object properties.
    // The ALCH fragment admits ONLY named (IRI) object properties — model.rs:20-33.
    if is_triple_term(dict, id) {
        return Err(ExtractError::Unclassifiable(
            "RDF 1.2 triple term in an object-property position".to_string(),
        ));
    }
    if !is_inline(id) && matches!(dict.term_parts(id), TermParts::Blank(_)) {
        return Err(ExtractError::Unclassifiable(format!(
            "blank node {} in an object-property position (named properties only in L1)",
            term_iri(dict, id)
        )));
    }
    if idx.inverse_of.contains(&id) {
        return Err(ExtractError::OutOfFragment(format!(
            "inverse object property expression {}",
            term_iri(dict, id)
        )));
    }
    Ok(ObjectPropertyExpression::ObjectProperty(id))
}

// -------------------------------------------------------------------------------------------
// Vocabulary + structural index
// -------------------------------------------------------------------------------------------

/// Interned dict ids for the fixed OWL/RDFS/rdf vocabulary the mapping matches on. A term
/// absent from the store gets [`NO_ID`] from `lookup` (matching nothing), exactly like the
/// `sparq-reason` path.
struct Vocab {
    ty: Id,
    sub_class_of: Id,
    equivalent_class: Id,
    disjoint_with: Id,
    sub_property_of: Id,
    domain: Id,
    range: Id,
    thing: Id,
    nothing: Id,
    rdf_nil: Id,
    /// Class-expression / list backbone predicates (inert on their own; consumed in-context).
    backbone: FxHashSet<Id>,
    /// Ignorable built-in annotation / ontology-header predicates.
    annotation: FxHashSet<Id>,
    /// `rdf:type` objects that are ignorable declaration / structural meta-classes.
    declaration_types: FxHashSet<Id>,
    /// `rdf:type` objects that are out-of-fragment meta-classes (property characteristics,
    /// n-ary collections), mapped to a display name.
    out_of_fragment_types: FxHashMap<Id, &'static str>,
    /// Predicates recognised as out-of-fragment LOGICAL constructs, mapped to a display name.
    out_of_fragment: FxHashMap<Id, &'static str>,
    /// Predicates recognised as datatype / data-range constructs, mapped to a display name.
    data_predicate: FxHashMap<Id, &'static str>,
    /// `owl:onProperty` id (used both as backbone and to build the restriction index).
    on_property: Id,
    some_values_from: Id,
    all_values_from: Id,
    intersection_of: Id,
    union_of: Id,
    complement_of: Id,
    inverse_of: Id,
    rdf_first: Id,
    rdf_rest: Id,
    restriction_ty: Id,
    /// Declaration-typing objects that *classify entities* for the disambiguation pass.
    owl_object_property: Id,
    owl_datatype_property: Id,
    owl_annotation_property: Id,
}

/// OWL predicates whose presence is an out-of-fragment LOGICAL construct (design record §5).
const OWL_OUT_OF_FRAGMENT_PREDS: &[&str] = &[
    "inverseOf",
    "sameAs",
    "differentFrom",
    "hasValue",
    "oneOf",
    "hasSelf",
    "onClass",
    "onProperties",
    "propertyChainAxiom",
    "hasKey",
    "disjointUnionOf",
    "equivalentProperty",
    "propertyDisjointWith",
    "members",
    "distinctMembers",
    "sourceIndividual",
    "assertionProperty",
    "targetIndividual",
    "targetValue",
    "maxCardinality",
    "minCardinality",
    "cardinality",
    "maxQualifiedCardinality",
    "minQualifiedCardinality",
    "qualifiedCardinality",
];

/// OWL predicates whose presence is a datatype / data-range construct (no concrete domain in
/// L1 — design record §5).
const OWL_DATA_PREDS: &[&str] = &[
    "onDataRange",
    "onDatatype",
    "withRestrictions",
    "datatypeComplementOf",
];

/// `rdf:type` objects that are ignorable declaration / structural meta-classes.
const DECLARATION_TYPE_LOCALS: &[(&str, &str)] = &[
    (OWL, "Class"),
    (OWL, "ObjectProperty"),
    (OWL, "DatatypeProperty"),
    (OWL, "AnnotationProperty"),
    (OWL, "NamedIndividual"),
    (OWL, "Ontology"),
    (OWL, "Restriction"),
    (OWL, "OntologyProperty"),
    // [OPUS-4.8] Fix 4: owl:DeprecatedClass and owl:DeprecatedProperty are structural
    // meta-classes (OWL 2 §11.2) with no logical content in ALCH — treat as ignorable
    // declarations rather than producing a spurious ClassAssertion.
    (OWL, "DeprecatedClass"),
    (OWL, "DeprecatedProperty"),
    (RDFS, "Class"),
    (RDFS, "Datatype"),
    (RDF, "List"),
    (RDF, "Property"),
];

/// `rdf:type` objects that are out-of-fragment meta-classes (property characteristics / n-ary
/// collections).
const OUT_OF_FRAGMENT_TYPE_LOCALS: &[&str] = &[
    "TransitiveProperty",
    "FunctionalProperty",
    "InverseFunctionalProperty",
    "SymmetricProperty",
    "AsymmetricProperty",
    "ReflexiveProperty",
    "IrreflexiveProperty",
    "AllDifferent",
    "AllDisjointClasses",
    "AllDisjointProperties",
    "NegativePropertyAssertion",
];

/// Ignorable built-in annotation / ontology-header predicates.
const ANNOTATION_LOCALS: &[(&str, &str)] = &[
    (RDFS, "label"),
    (RDFS, "comment"),
    (RDFS, "seeAlso"),
    (RDFS, "isDefinedBy"),
    (OWL, "versionInfo"),
    (OWL, "versionIRI"),
    (OWL, "priorVersion"),
    (OWL, "backwardCompatibleWith"),
    (OWL, "incompatibleWith"),
    (OWL, "deprecated"),
    (OWL, "imports"),
];

impl Vocab {
    fn intern(dict: &Dict) -> Vocab {
        use oxrdf::{NamedNode, Term as OTerm};
        let look = |iri: String| dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(iri)));
        let owl = |local: &str| look(format!("{}{}", OWL, local));

        let on_property = owl("onProperty");
        let some_values_from = owl("someValuesFrom");
        let all_values_from = owl("allValuesFrom");
        let intersection_of = owl("intersectionOf");
        let union_of = owl("unionOf");
        let complement_of = owl("complementOf");
        let inverse_of = owl("inverseOf");
        let rdf_first = look(format!("{}first", RDF));
        let rdf_rest = look(format!("{}rest", RDF));

        let mut backbone = FxHashSet::default();
        for id in [
            on_property,
            some_values_from,
            all_values_from,
            intersection_of,
            union_of,
            complement_of,
            rdf_first,
            rdf_rest,
        ] {
            if id != NO_ID {
                backbone.insert(id);
            }
        }

        let mut annotation = FxHashSet::default();
        for &(ns, local) in ANNOTATION_LOCALS {
            let id = look(format!("{}{}", ns, local));
            if id != NO_ID {
                annotation.insert(id);
            }
        }

        let mut declaration_types = FxHashSet::default();
        for &(ns, local) in DECLARATION_TYPE_LOCALS {
            let id = look(format!("{}{}", ns, local));
            if id != NO_ID {
                declaration_types.insert(id);
            }
        }

        let mut out_of_fragment_types = FxHashMap::default();
        for &local in OUT_OF_FRAGMENT_TYPE_LOCALS {
            let id = owl(local);
            if id != NO_ID {
                out_of_fragment_types.insert(id, local);
            }
        }

        let mut out_of_fragment = FxHashMap::default();
        for &local in OWL_OUT_OF_FRAGMENT_PREDS {
            let id = owl(local);
            if id != NO_ID {
                out_of_fragment.insert(id, local);
            }
        }

        let mut data_predicate = FxHashMap::default();
        for &local in OWL_DATA_PREDS {
            let id = owl(local);
            if id != NO_ID {
                data_predicate.insert(id, local);
            }
        }

        Vocab {
            ty: look(format!("{}type", RDF)),
            sub_class_of: look(format!("{}subClassOf", RDFS)),
            equivalent_class: owl("equivalentClass"),
            disjoint_with: owl("disjointWith"),
            sub_property_of: look(format!("{}subPropertyOf", RDFS)),
            domain: look(format!("{}domain", RDFS)),
            range: look(format!("{}range", RDFS)),
            thing: owl("Thing"),
            nothing: owl("Nothing"),
            rdf_nil: look(format!("{}nil", RDF)),
            backbone,
            annotation,
            declaration_types,
            out_of_fragment_types,
            out_of_fragment,
            data_predicate,
            on_property,
            some_values_from,
            all_values_from,
            intersection_of,
            union_of,
            complement_of,
            inverse_of,
            rdf_first,
            rdf_rest,
            restriction_ty: owl("Restriction"),
            owl_object_property: owl("ObjectProperty"),
            owl_datatype_property: owl("DatatypeProperty"),
            owl_annotation_property: owl("AnnotationProperty"),
        }
    }
}

/// The one-pass structural index the extractor builds over the triples: the class-expression
/// backbone edges, the restriction typing, the entity declarations used to disambiguate
/// assertions, and the per-node out-of-fragment / datatype shape markers.
#[derive(Default)]
struct Index {
    on_property: FxHashMap<Id, Id>,
    some_values_from: FxHashMap<Id, Id>,
    all_values_from: FxHashMap<Id, Id>,
    intersection_of: FxHashMap<Id, Id>,
    union_of: FxHashMap<Id, Id>,
    complement_of: FxHashMap<Id, Id>,
    first: FxHashMap<Id, Id>,
    rest: FxHashMap<Id, Id>,
    is_restriction: FxHashSet<Id>,
    /// Nodes that back a class expression / list — never valid as a ground individual.
    structural_nodes: FxHashSet<Id>,
    /// Nodes carrying an out-of-fragment class shape predicate (`owl:oneOf`, cardinality, …),
    /// mapped to a display name.
    out_of_fragment_shape: FxHashMap<Id, &'static str>,
    /// Nodes carrying a datatype / data-range shape predicate, mapped to a display name.
    data_shape: FxHashMap<Id, &'static str>,
    /// Nodes appearing as the object of `owl:inverseOf` — inverse property expressions.
    inverse_of: FxHashSet<Id>,
    /// Entities declared (or used) as object properties — enables sound role-assertion mapping.
    object_props: FxHashSet<Id>,
    /// Entities declared as datatype properties.
    data_props: FxHashSet<Id>,
    /// Entities declared as annotation properties.
    annotation_props: FxHashSet<Id>,
    /// Entities declared as MORE THAN ONE property type (e.g. both AnnotationProperty and
    /// ObjectProperty) — a punning clash that makes their use in triples Unclassifiable.
    /// [OPUS-4.8] Fix 3.
    punned_props: FxHashSet<Id>,
}

impl Index {
    // [OPUS-4.8] Fix 1 + Fix 3: returns Err on a branching list / duplicate backbone
    // value (different value for same functional-position predicate on same node), or on a
    // cross-type punned declaration. Identical re-assertion (same value) is silently allowed.
    fn build(triples: &[[Id; 3]], v: &Vocab) -> Result<Index, ExtractError> {
        let mut idx = Index::default();
        for &[s, p, o] in triples {
            if p == v.on_property {
                // owl:onProperty is functional on a restriction node — exactly one value.
                if let Some(prev) = idx.on_property.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "restriction node has two conflicting owl:onProperty values"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
                idx.object_props.insert(o); // usage-based typing
            } else if p == v.some_values_from {
                if let Some(prev) = idx.some_values_from.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "restriction node has two conflicting owl:someValuesFrom values"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.all_values_from {
                if let Some(prev) = idx.all_values_from.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "restriction node has two conflicting owl:allValuesFrom values"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.intersection_of {
                if let Some(prev) = idx.intersection_of.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "class node has two conflicting owl:intersectionOf values"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.union_of {
                if let Some(prev) = idx.union_of.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "class node has two conflicting owl:unionOf values".to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.complement_of {
                if let Some(prev) = idx.complement_of.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedClassExpression(
                            "class node has two conflicting owl:complementOf values".to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.inverse_of {
                idx.inverse_of.insert(s);
            } else if p == v.rdf_first {
                // rdf:first is functional — each list cell carries exactly one member.
                if let Some(prev) = idx.first.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedList(
                            "list cell has two conflicting rdf:first values (branching list)"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.rdf_rest {
                // rdf:rest is functional — each list cell carries exactly one continuation.
                if let Some(prev) = idx.rest.insert(s, o) {
                    if prev != o {
                        return Err(ExtractError::MalformedList(
                            "list cell has two conflicting rdf:rest values (branching list)"
                                .to_string(),
                        ));
                    }
                }
                idx.structural_nodes.insert(s);
            } else if p == v.ty {
                if o == v.restriction_ty {
                    idx.is_restriction.insert(s);
                    idx.structural_nodes.insert(s);
                } else if o == v.owl_object_property {
                    // Fix 3: track cross-type punning on declaration.
                    if idx.data_props.contains(&s) || idx.annotation_props.contains(&s) {
                        idx.punned_props.insert(s);
                    }
                    idx.object_props.insert(s);
                } else if o == v.owl_datatype_property {
                    if idx.object_props.contains(&s) || idx.annotation_props.contains(&s) {
                        idx.punned_props.insert(s);
                    }
                    idx.data_props.insert(s);
                } else if o == v.owl_annotation_property {
                    if idx.object_props.contains(&s) || idx.data_props.contains(&s) {
                        idx.punned_props.insert(s);
                    }
                    idx.annotation_props.insert(s);
                }
            } else if let Some(name) = v.out_of_fragment.get(&p) {
                // An out-of-fragment predicate makes its subject an out-of-fragment class
                // shape (so a `∃r.{a}` node is refused, not read as a named class), AND is
                // caught again standalone in `classify_triple` (so a top-level `owl:sameAs`
                // is refused even if its subject is never referenced as a class).
                idx.out_of_fragment_shape.insert(s, name);
            } else if let Some(name) = v.data_predicate.get(&p) {
                idx.data_shape.insert(s, name);
            } else if p == v.sub_property_of || p == v.domain || p == v.range {
                // A property used in an object-property axiom position is known to be an
                // object property (usage-based typing) — but only the subject/relevant operand.
                if p == v.sub_property_of {
                    idx.object_props.insert(s);
                    idx.object_props.insert(o);
                } else {
                    idx.object_props.insert(s);
                }
            }
        }
        Ok(idx)
    }
}

// -------------------------------------------------------------------------------------------
// Node-kind helpers
// -------------------------------------------------------------------------------------------

/// `true` iff `id` denotes an RDF literal (inline integer or an interned literal).
fn is_literal(dict: &Dict, id: Id) -> bool {
    is_inline(id) || matches!(dict.term_parts(id), TermParts::Lit { .. })
}

/// `true` iff `id` denotes an RDF 1.2 triple term (quoted triple).
fn is_triple_term(dict: &Dict, id: Id) -> bool {
    !is_inline(id) && matches!(dict.term_parts(id), TermParts::Triple(_))
}

/// Renders `id` as a short diagnostic string (the IRI/blank/literal text). Error-path only.
fn term_iri(dict: &Dict, id: Id) -> String {
    if is_inline(id) {
        return format!("(inline literal id {})", id);
    }
    match dict.term_parts(id) {
        TermParts::Iri { prefix, suffix } => format!("<{}{}>", prefix, suffix),
        TermParts::Lit { value, .. } => format!("\"{}\"", value),
        TermParts::Blank(label) => format!("_:{}", label),
        TermParts::Triple(_) => "<<triple term>>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Axiom;
    use sparq_core::Graph;

    /// The five taxonomy arms each render a non-empty diagnostic (direct coverage of the
    /// `Display` impl, which the fail-closed error paths rely on for operator diagnostics).
    #[test]
    fn extract_error_display_covers_every_arm() {
        let arms = [
            ExtractError::OutOfFragment("x".into()),
            ExtractError::DataConstruct("x".into()),
            ExtractError::MalformedList("x".into()),
            ExtractError::MalformedClassExpression("x".into()),
            ExtractError::Unclassifiable("x".into()),
        ];
        for e in arms {
            let s = format!("{}", e);
            assert!(s.contains('x'), "diagnostic should carry the detail: {}", s);
            // Exercise the std::error::Error impl too.
            let _dyn: &dyn std::error::Error = &e;
        }
    }

    /// Direct unit test of the public `extract` entry point: a one-axiom accept and a
    /// fail-closed reject, so the entry point is covered independently of the integration suite.
    #[test]
    fn extract_direct_accept_and_reject() {
        let (dict, triples) = Graph::parse_to_triples(
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <http://ex/A> rdfs:subClassOf <http://ex/B> .",
            "turtle",
        )
        .expect("parse");
        let onto = extract(&dict, &triples).expect("accept");
        assert_eq!(onto.len(), 1);
        assert!(matches!(onto.axioms[0], Axiom::SubClassOf { .. }));

        let (dict, triples) = Graph::parse_to_triples(
            "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             <http://ex/a> owl:sameAs <http://ex/b> .",
            "turtle",
        )
        .expect("parse");
        assert!(matches!(
            extract(&dict, &triples),
            Err(ExtractError::OutOfFragment(_))
        ));
    }

    /// Direct coverage of the node-kind helpers over a graph with an IRI, a literal, and a
    /// blank node.
    #[test]
    fn node_kind_helpers() {
        let (dict, _t) = Graph::parse_to_triples(
            "<http://ex/s> <http://ex/p> \"lit\" .\n<http://ex/s> <http://ex/q> _:b .",
            "turtle",
        )
        .expect("parse");
        use oxrdf::{Literal, NamedNode, Term as OTerm};
        let s = dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked("http://ex/s")));
        let lit = dict.lookup(&OTerm::Literal(Literal::new_simple_literal("lit")));
        assert!(!is_literal(&dict, s));
        assert!(is_literal(&dict, lit));
        assert!(!is_triple_term(&dict, s));
        assert!(term_iri(&dict, s).contains("ex/s"));
        assert!(term_iri(&dict, lit).contains("lit"));
    }
}
