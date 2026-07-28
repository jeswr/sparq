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

#[test]
fn parse_failure_suggests_a_keyword_without_applying_it() {
    // Phase 4 (sq-h7zlx, design §3.2): the ONLY sliver of lever 2 — a did-you-mean on parse
    // failure. The public contract is that the call still returns `Err` and the query is
    // echoed unrepaired; a lenient parse that silently applied the fix could execute a
    // different, valid, wrong query.
    let query = "SELECT ?s WHERE { ?s ?p ?o } FLTR(?s > 1)";
    let error = terse_to_sparql(query).expect_err("a mistyped keyword must still fail");

    match error {
        TerseError::CanaryFailed { sparql, hints, .. } => {
            assert_eq!(sparql, query, "the failing query must be echoed verbatim");
            assert_eq!(hints.len(), 1, "expected one hint, got {hints:?}");
            assert_eq!(hints[0].token, "FLTR");
            assert!(
                hints[0].suggestions.contains(&"FILTER".to_string()),
                "expected FILTER among {:?}",
                hints[0].suggestions
            );
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

#[test]
fn a_structural_parse_error_gets_no_keyword_noise() {
    // The diagnostic is conservative: an unbalanced query has no mistyped keyword, so it
    // yields no suggestions rather than a plausible-looking guess.
    let error = terse_to_sparql("SELECT ?s WHERE { ?s ?p").expect_err("must fail the canary");

    match error {
        TerseError::CanaryFailed { hints, .. } => assert!(hints.is_empty(), "got {hints:?}"),
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

#[test]
fn keyword_hints_is_silent_on_valid_queries() {
    // The public diagnostic entry point never fires on a query that parses, so wiring it into
    // any other parse-failure path cannot perturb a working query.
    assert!(sparq_terse::keyword_hints("SELECT ?s WHERE { ?s a <http://ex/T> } LIMIT 1").is_empty());
}

#[test]
fn canonical_sparql_passes_through_byte_identically() {
    let query = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";
    let expansion = terse_to_sparql(query).expect("canonical SPARQL must pass through");

    assert_eq!(expansion.canonical_sparql, query);
    assert!(expansion.resolutions.is_empty());
    assert!(expansion.warnings.is_empty());
}
