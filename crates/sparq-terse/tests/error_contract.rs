// [GPT-5.6] sq-bif.33 — pin the public loud-failure and pass-through contracts.

use sparq_terse::{terse_to_sparql, TerseError, LEGEND_VERSION};

#[test]
fn unknown_keyword_exposes_versioned_suggestions() {
    let query = "SELECT ?f WHERE { ?f K:derived ?s }";
    let error = terse_to_sparql(query).expect_err("an unknown keyword must fail loudly");

    match error {
        TerseError::UnknownKeyword {
            keyword,
            legend_version,
            suggestions,
        } => {
            assert_eq!(keyword, "derived");
            assert_eq!(legend_version, LEGEND_VERSION);
            assert!(
                suggestions.contains(&"derivedFrom".to_string()),
                "expected derivedFrom among {suggestions:?}"
            );
        }
        other => panic!("expected UnknownKeyword, got {other:?}"),
    }
}

#[test]
fn real_prefix_k_collides_with_keyword_sigil() {
    let query = "PREFIX K: <http://ex/> SELECT ?s WHERE { ?s K:type ?o }";
    let error = terse_to_sparql(query).expect_err("PREFIX K: must collide with the K: sigil");

    match error {
        TerseError::KeywordPrefixCollision { keyword } => assert_eq!(keyword, "type"),
        other => panic!("expected KeywordPrefixCollision, got {other:?}"),
    }
}

#[test]
fn keyword_text_inside_literal_is_data() {
    let query = "SELECT ?s WHERE { ?s <http://ex/p> \"K:label\" }";
    let expansion = terse_to_sparql(query).expect("K: text inside a literal is data");

    assert_eq!(expansion.canonical_sparql, query);
    assert!(expansion.keywords.is_empty());
}

#[test]
fn vector_phrase_requires_opt_in_feature() {
    let query = "SELECT ?f WHERE { ?f <http://ex/about> V(\"cardinality estimation\") }";
    let error = terse_to_sparql(query).expect_err("V() requires the opt-in vectors surface");

    match error {
        TerseError::FeatureRequired { phrase, .. } => {
            assert_eq!(phrase, "cardinality estimation");
        }
        other => panic!("expected FeatureRequired, got {other:?}"),
    }
}

// [OPUS-5] sq-h7zlx — Phase 4, design §3.2: the did-you-mean diagnostic SUGGESTS on a parse
// failure and NEVER applies the suggestion (that would be the rejected lenient-parse mode).
#[test]
fn parse_failure_suggests_the_nearest_keyword_without_applying_it() {
    let query = "SELECT ?s WHERE { ?s ?p ?o FLTR(?o > 1) }";
    let error = terse_to_sparql(query).expect_err("a mistyped keyword must still fail loudly");

    match error {
        TerseError::CanaryFailed {
            sparql,
            suggestions,
            ..
        } => {
            // The suggestion is DATA, not a repair: the text handed back is the query that
            // failed, byte-for-byte — no `FILTER` was substituted anywhere.
            assert_eq!(sparql, query, "the failing query must be echoed untouched");
            assert!(!sparql.contains("FILTER"), "the suggestion must never be applied");
            let hint = suggestions.first().expect("expected a did-you-mean hint");
            assert_eq!(hint.token, "FLTR");
            assert_eq!(hint.suggestion, "FILTER");
            // ... and it is visible in the message the agent reads.
            let rendered = TerseError::CanaryFailed {
                sparql,
                parse_error: "x".to_string(),
                suggestions,
            }
            .to_string();
            assert!(rendered.contains("did you mean FILTER?"), "got {rendered}");
            assert!(rendered.contains("not applied"), "got {rendered}");
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

#[test]
fn parse_failure_without_a_keyword_typo_suggests_nothing() {
    // The hint fires only on a keyword-shaped word: an ordinary syntax error stays a plain,
    // loud parse failure rather than acquiring a speculative suggestion.
    let error = terse_to_sparql("SELECT ?s WHERE { ?s ?p").expect_err("unbalanced query");

    match error {
        TerseError::CanaryFailed { suggestions, .. } => {
            assert!(suggestions.is_empty(), "expected no hints, got {suggestions:?}");
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

// [SONNET-4.6] Review round 1 on #4847: the local part of a prefixed name is DATA, and the
// scanner must hold that line across the whole `PN_LOCAL` grammar — not just its ASCII subset.
// Each query below carries a VALID prefixed name whose local part ends in a typo-shaped ASCII
// suffix, and fails to parse for an UNRELATED reason (the `WHERE` block is never closed), so
// the did-you-mean scanner does run. It must stay silent: an ASCII-only skip stopped inside
// the name and hinted `FLTR -> FILTER` at a term the user spelled correctly.
#[test]
fn valid_prefixed_names_never_become_keyword_hints() {
    for query in [
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:\u{e9}FLTR ?o",
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:a%C3%A9FLTR ?o",
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:a\\~FLTR ?o",
        "PREFIX : <http://ex/> SELECT ?s WHERE { ?s :FLTR ?o",
    ] {
        let error = terse_to_sparql(query).expect_err("the unclosed block must fail to parse");

        match error {
            TerseError::CanaryFailed { suggestions, .. } => assert!(
                suggestions.is_empty(),
                "hinted at a correctly-spelled prefixed name in {query}: {suggestions:?}"
            ),
            other => panic!("expected CanaryFailed, got {other:?}"),
        }
    }
}

#[test]
fn canonical_sparql_passes_through_byte_identically() {
    let query = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";
    let expansion = terse_to_sparql(query).expect("canonical SPARQL must pass through");

    assert_eq!(expansion.canonical_sparql, query);
    assert!(expansion.resolutions.is_empty());
    assert!(expansion.warnings.is_empty());
}
