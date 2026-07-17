//! Regression checks for committed `canonicalize_nquads` fuzz seeds. [GPT-5.6]

use std::collections::HashSet;

/// [GPT-5.6] sq-40apk: RDFC-1.0 canonicalization applies RDF dataset set
/// semantics, so the duplicate quad in the CI reproducer must collapse.
#[test]
fn duplicate_quad_reproducer_preserves_distinct_count() {
    let input =
        include_str!("../seeds/canonicalize_nquads/24694f1d7701250710a0fec7432dc1b8c562ee99");
    let input_quads = sparq_canon::parse_nquads(input).expect("reproducer must parse");
    let distinct_input_quads = input_quads
        .iter()
        .map(ToString::to_string)
        .collect::<HashSet<_>>();

    assert_eq!(
        input_quads.len(),
        5,
        "reproducer must retain its duplicate on parse"
    );
    assert_eq!(
        distinct_input_quads.len(),
        4,
        "one repeated quad must collapse under dataset set semantics"
    );

    let canonical = sparq_canon::canonicalize_nquads(input).expect("reproducer must canonicalize");
    let canonical_quads =
        sparq_canon::parse_nquads(&canonical).expect("canonical output must parse");
    assert_eq!(
        canonical_quads.len(),
        distinct_input_quads.len(),
        "canonicalization must preserve the deduplicated quad count"
    );
}
