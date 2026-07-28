// [SONNET-4.6] sq-pbz04.4.19 (epic sq-pbz04.4) — DIFFERENTIAL PARITY for the
// concrete-domain oracle's datatype lattice, over the XSD literal matrix.
//
// 🤖 SPARQ agent. `sparq_reason_dl::cdomain` decides datatype value-space intersection and
// containment STRUCTURALLY (families, tiers, integer intervals — see its module docs §1). A
// wrong entry in that lattice would silently manufacture tableau verdicts, so this lane
// cross-checks EVERY pair against an INDEPENDENT source of truth: `sparq_reason::dtype`, the
// repo's existing D-entailment value seam, which maps a `(lexical, datatype)` pair to a
// canonical `DValue` key (and which itself delegates its numeric parsing to the shared
// `sparq_substrate::numeric` tower). Two datatypes' value spaces INTERSECT exactly when some
// literal keys into both.
//
// Four assertions per ordered pair, so both soundness directions are pinned:
//   (1) no false UNSAT — a shared witness forces the oracle to say "intersects";
//   (2) no false SAT   — "intersects" forces a shared witness to exist in the matrix;
//   (3) containment    — `V(A) ⊆ V(B)` forces every A-key to be a B-key;
//   (4) non-containment — `V(A) ⊄ V(B)` forces some A-key not to be a B-key.
//
// SCOPE. `rdfs:Literal`, `owl:real` and `owl:rational` are EXCLUDED: they have no lexical
// space in the OWL 2 datatype map (no literal is ever written with them as its datatype), so
// no witness search can speak about them. Their lattice positions are argued in the
// `cdomain` module docs (witnesses W1/W2/W5) and pinned by that module's own unit tests.
#![cfg(feature = "dl_datatypes")]

use sparq_reason::dtype::{d_value_key, DValue};
use sparq_reason_dl::cdomain::{satisfiable, Datatype, ALL_DATATYPES};
use std::collections::BTreeSet;

/// Datatypes with no lexical space — the witness search cannot reach them (see SCOPE above).
fn has_lexical_space(d: Datatype) -> bool {
    !matches!(
        d,
        Datatype::RdfsLiteral | Datatype::OwlReal | Datatype::OwlRational
    )
}

/// The lexical matrix. Chosen to place a witness in EVERY non-empty pairwise intersection
/// and difference of the admitted lattice: the integer bounds of every derived type and the
/// values just outside them, plus one representative per non-numeric family.
const LEXICALS: &[&str] = &[
    // Integer anchors — every admitted integer interval either contains one of these or is
    // disjoint from the other's (cdomain module docs §1).
    "-1", "0", "1",
    // Just outside each bounded type's range, so `V(A) ∖ V(B)` has a witness.
    "127", "128", "255", "256", "-128", "-129", "32767", "32768", "65535", "65536", "-32768",
    "-32769", "2147483647", "2147483648", "4294967295", "4294967296", "-2147483648",
    "-2147483649", "9223372036854775807", "9223372036854775808", "18446744073709551615",
    "18446744073709551616", "-9223372036854775808", "-9223372036854775809",
    // Non-integral decimals (`xsd:decimal ∖ ℤ`).
    "0.5", "-0.5", "1.25",
    // Booleans, strings, URIs, timestamps.
    "true", "false", "hello", "http://example.org/x", "2020-01-01T00:00:00Z",
    "2020-01-01T00:00:00", "2020-06-30T12:34:56+01:00",
];

/// Every `DValue` key the datatype accepts over [`LEXICALS`].
fn keys(d: Datatype) -> BTreeSet<String> {
    let iri = d.iri();
    LEXICALS
        .iter()
        .filter_map(|lex| d_value_key(lex, &iri).map(|k| render(&k)))
        .collect()
}

/// A stable string rendering of a `DValue` (it is not `Ord`, and the variant identity is
/// what carries the value-space partition).
fn render(v: &DValue) -> String {
    match v {
        DValue::Decimal(s) => format!("dec:{}", s),
        DValue::F64(b) => format!("f64:{}", b),
        DValue::F32(b) => format!("f32:{}", b),
        DValue::Bool(b) => format!("bool:{}", b),
        DValue::Str(s) => format!("str:{}", s),
        DValue::Temporal(i, tz, date) => format!("time:{}:{}:{}", i, tz, date),
        DValue::Uri(s) => format!("uri:{}", s),
        DValue::Octets(o) => format!("oct:{:?}", o),
    }
}

/// The lexical matrix must actually reach every in-scope datatype; otherwise the "no false
/// SAT" direction below would pass vacuously for an unwitnessed type.
#[test]
fn every_in_scope_datatype_has_at_least_one_witness() {
    for &d in ALL_DATATYPES {
        if !has_lexical_space(d) {
            continue;
        }
        assert!(
            !keys(d).is_empty(),
            "{} has no witness in the lexical matrix — the differential would be vacuous",
            d.iri()
        );
    }
}

/// The headline differential: intersection and containment agree EXACTLY with the value
/// seam over the whole XSD matrix.
#[test]
fn lattice_matches_the_engine_value_semantics() {
    let in_scope: Vec<Datatype> = ALL_DATATYPES
        .iter()
        .copied()
        .filter(|&d| has_lexical_space(d))
        .collect();
    let all_keys: Vec<(Datatype, BTreeSet<String>)> =
        in_scope.iter().map(|&d| (d, keys(d))).collect();

    for (a, ka) in &all_keys {
        for (b, kb) in &all_keys {
            let shared_witness = ka.intersection(kb).next().is_some();
            let oracle_intersects = satisfiable(&[*a, *b], &[]);
            // (1) no false UNSAT — the direction that would make the tableau report a
            //     wrong `Unsatisfiable`.
            assert!(
                !shared_witness || oracle_intersects,
                "FALSE UNSAT: {} and {} share a value witness but the oracle calls them disjoint",
                a.iri(),
                b.iri()
            );
            // (2) no false SAT — the direction that would make the tableau report a wrong
            //     `Satisfiable`.
            assert!(
                !oracle_intersects || shared_witness,
                "FALSE SAT: the oracle intersects {} with {} but no literal keys into both",
                a.iri(),
                b.iri()
            );

            let oracle_contains = !satisfiable(&[*a], &[*b]); // V(A) ∖ V(B) = ∅
            let witness_outside = ka.difference(kb).next().is_some();
            // (3) containment ⇒ every A-witness is a B-witness.
            assert!(
                !oracle_contains || !witness_outside,
                "FALSE CONTAINMENT: the oracle says {} ⊆ {} but a witness of the former is \
                 not one of the latter",
                a.iri(),
                b.iri()
            );
            // (4) non-containment ⇒ some A-witness is outside B.
            assert!(
                oracle_contains || witness_outside,
                "MISSED CONTAINMENT: the oracle says {} ⊄ {} but every witness of the former \
                 is also one of the latter",
                a.iri(),
                b.iri()
            );
        }
    }
}

/// The single case the differential exists for: `xsd:integer` and `xsd:string` are disjoint
/// (WebOnt-I5.3-015), and no literal in the matrix witnesses otherwise.
#[test]
fn the_webont_i53_015_pair_is_disjoint_under_both_sources() {
    let int_keys = keys(Datatype::XsdInteger);
    let str_keys = keys(Datatype::XsdString);
    assert!(int_keys.intersection(&str_keys).next().is_none());
    assert!(!satisfiable(&[Datatype::XsdInteger, Datatype::XsdString], &[]));
}
