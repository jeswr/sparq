//! [FABLE-5] sq-pbz04.1.2 (epic sq-pbz04.1, substrate seam 3) — the ORDERING-PARITY
//! oracle for the `substrate-compare` feature: the reasoner-side comparator
//! (`sparq_reason::compare::sort_ids`, the shared `sparq_substrate::compare` total order
//! over dictionary ids) must order an ENTAILED solution multiset BYTE-IDENTICALLY to a
//! REAL SPARQL-engine `ORDER BY` over the same materialised closure.
//!
//! Non-vacuous by construction: the fixture packs the pairs each comparator arm alone
//! decides — numeric-vs-lexical divergences (`9` < `10` by value, `"10"` < `"9"`
//! lexically), a cross-timezone `xsd:dateTime` pair whose TIMELINE order is the reverse
//! of its lexical order (pins `strict_cmp`), a beyond-2^53 integer pair sharing one f64
//! (pins the `exact_cmp` collapse-recheck), inline and stored integer ids, RDF 1.2 triple
//! terms (pins the component-wise recursion), plus blanks / IRIs / language-tagged /
//! boolean / date / gYear / unknown-datatype literals for the class ranks and the string
//! fallback. Mis-wire any arm and the two sequences diverge. All fixture terms are
//! pairwise DISTINCT under the total order, so the sorted sequence is unique and the
//! assertion is independent of either side's tie-breaking.
//!
//! (Compiled only with `--features substrate-compare` — a Cargo `required-features`
//! target. sparq-engine is a dev-dependency here purely as the parity oracle.)

use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_reason::compare::sort_ids;
use sparq_reason::{materialize, Profile};

/// Mixed-term fixture WITH an RDFS schema, so materialisation genuinely ENTAILS new
/// solutions (domain/range/subClassOf/subPropertyOf firings) that participate in the
/// ordered multiset.
const FIXTURE: &str = r#"
@prefix ex: <http://ex/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

ex:knows rdfs:domain ex:Person .
ex:knows rdfs:range ex:Agent .
ex:Person rdfs:subClassOf ex:LivingThing .
ex:knows rdfs:subPropertyOf ex:related .

_:alice ex:knows ex:bob .
ex:bob ex:name "bob" .
ex:bob ex:label "zebra"@en .
ex:bob ex:motto "bonjour"@fr .
ex:bob ex:age "42"^^xsd:integer .
ex:bob ex:debt "-3"^^xsd:integer .
ex:bob ex:tiny "9"^^xsd:integer .
ex:bob ex:score "9.5"^^xsd:decimal .
ex:bob ex:big1 "9007199254740992"^^xsd:integer .
ex:bob ex:big2 "9007199254740993"^^xsd:integer .
ex:bob ex:height "1.75E0"^^xsd:double .
ex:bob ex:ok "true"^^xsd:boolean .
ex:bob ex:no "false"^^xsd:boolean .
ex:bob ex:t1 "2024-03-15T14:00:00+01:00"^^xsd:dateTime .
ex:bob ex:t2 "2024-03-15T13:30:00Z"^^xsd:dateTime .
ex:bob ex:d "2024-03-14"^^xsd:date .
ex:bob ex:y "2020"^^xsd:gYear .
ex:bob ex:y2 "2021"^^xsd:gYear .
ex:bob ex:odd "weird"^^ex:custom .
"#;

/// The reasoner-side sort of the ENTAILED object multiset equals the engine's real
/// `ORDER BY ?o` over the identical materialised closure, term-for-term.
#[test]
fn entailed_solution_order_matches_engine_order_by() {
    let (mut dict, mut triples) = Graph::parse_to_triples(FIXTURE, "turtle").expect("fixture parses");

    // Two RDF 1.2 quoted-triple objects (structural dict terms; differ in their object
    // component, one of which is an inline-integer id) — interned exactly as a loader
    // would, so the reasoner sees them as ordinary opaque object ids.
    let quoted = |dict: &mut sparq_core::dict::Dict, o: &str| {
        dict.intern(&oxrdf::Term::Triple(Box::new(oxrdf::Triple::new(
            oxrdf::NamedNode::new_unchecked("http://ex/qa"),
            oxrdf::NamedNode::new_unchecked("http://ex/qp"),
            oxrdf::Term::Literal(oxrdf::Literal::new_typed_literal(
                o,
                oxrdf::NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer"),
            )),
        ))))
    };
    let bob = dict.intern_iri("http://ex/bob");
    let asserts = dict.intern_iri("http://ex/asserts");
    let tt1 = quoted(&mut dict, "1");
    let tt2 = quoted(&mut dict, "2");
    triples.push([bob, asserts, tt1]);
    triples.push([bob, asserts, tt2]);

    // Materialise the RDFS closure: the ordered answer set must contain ENTAILED rows.
    let added = materialize(Profile::Rdfs, &mut dict, &mut triples);
    assert!(added > 0, "the fixture must actually entail new solutions (got {} added)", added);

    // Reasoner side: order the full object multiset with the seam-3 comparator.
    let mut objects: Vec<Id> = triples.iter().map(|t| t[2]).collect();
    sort_ids(&dict, &mut objects);
    let reason_order: Vec<String> = objects.iter().map(|&id| dict.term(id).to_string()).collect();

    // Engine side: the SAME closure through the real engine, real ORDER BY.
    let graph = Graph::from_parts(dict, triples);
    let res = sparq_engine::query(&graph, "SELECT ?o WHERE { ?s ?p ?o } ORDER BY ?o").expect("engine query runs");
    let engine_order: Vec<String> = res
        .rows
        .iter()
        .map(|r| r[0].as_ref().expect("?o is always bound").to_string())
        .collect();

    assert_eq!(
        reason_order.len(),
        engine_order.len(),
        "both sides must order the same solution multiset"
    );
    assert_eq!(
        reason_order, engine_order,
        "entailed-solution ordering diverged from the engine's ORDER BY total order"
    );
}
