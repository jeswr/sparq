// [GPT-5.6] sq-bif.33 — pin the public loud-failure and pass-through contracts.

use sparq_terse::{terse_to_sparql, TerseError, LEGEND_VERSION};

// [SONNET-4.6] sq-h7zlx — Phase 4: the did-you-mean diagnostic is a HINT on parse failure,
// never a rewrite (design §3.2).

#[test]
fn parse_failure_hints_at_the_misspelled_keyword_without_applying_it() {
    let query = "SELECT ?s WHERE { ?s ?p ?o FLTR(?o > 1) }";
    let error = terse_to_sparql(query).expect_err("a misspelled keyword must still fail loudly");

    match error {
        TerseError::CanaryFailed {
            sparql,
            suggestions,
            ..
        } => {
            // The emission is untouched — the suggestion was NOT applied.
            assert_eq!(sparql, query);
            assert!(!sparql.contains("FILTER"), "the hint must never be applied");
            let hint = suggestions
                .iter()
                .find(|s| s.token == "FLTR")
                .unwrap_or_else(|| panic!("expected a FLTR hint, got {suggestions:?}"));
            assert!(
                hint.suggestions.contains(&"FILTER".to_string()),
                "expected FILTER among {:?}",
                hint.suggestions
            );
            // And it reaches the agent through the rendered error.
            let rendered = TerseError::CanaryFailed {
                sparql,
                parse_error: "unbalanced".to_string(),
                suggestions,
            }
            .to_string();
            assert!(rendered.contains("did you mean FILTER"), "got {rendered}");
            assert!(rendered.contains("(not applied)"), "got {rendered}");
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

#[test]
fn a_comparison_operator_does_not_hide_a_later_misspelling() {
    // `<` is less-than as well as an IRI opener: a query shaped like this one used to skip
    // from the operator to end of input, losing the FLTR hint entirely.
    for query in [
        "SELECT ?s WHERE { ?s ?p ?o FILTER(?o < 1) FLTR(?o) }",
        "SELECT ?s WHERE { ?s ?p ?o FILTER(?a < 1 && ?b > 2) FLTR(?o) }",
        // Compact: SPARQL punctuation delimits these tokens without whitespace, so every
        // byte from `<` to `>` is IRIREF-legal and only the lexical position tells them
        // apart. A comparison sits on each side of the misspelling.
        "SELECT ?s WHERE { ?s ?p ?o FILTER(?a<1&&FLTR(?o)&&?b>2) }",
    ] {
        let error = terse_to_sparql(query).expect_err("a misspelled keyword must fail loudly");

        match error {
            TerseError::CanaryFailed { suggestions, .. } => {
                let hint = suggestions
                    .iter()
                    .find(|s| s.token == "FLTR")
                    .unwrap_or_else(|| panic!("expected a FLTR hint for {query:?}, got {suggestions:?}"));
                assert!(
                    hint.suggestions.contains(&"FILTER".to_string()),
                    "expected FILTER among {:?}",
                    hint.suggestions
                );
            }
            other => panic!("expected CanaryFailed, got {other:?}"),
        }
    }
}

#[test]
fn a_genuine_iri_is_still_ignored_beside_a_compact_comparison() {
    // The other half of the contract: reading a compact `?a<1` as a comparison must not
    // start reporting the words inside real IRI refs. `SELCT` sits in an IRI (data, never a
    // hint); `FLTR` sits in keyword position between two compact comparisons.
    let query = "SELECT ?s WHERE { ?s <http://ex/SELCT> ?o FILTER(?a<1&&FLTR(?o)&&?b>2) }";
    let error = terse_to_sparql(query).expect_err("a misspelled keyword must fail loudly");

    match error {
        TerseError::CanaryFailed { suggestions, .. } => {
            let tokens: Vec<&str> = suggestions.iter().map(|s| s.token.as_str()).collect();
            assert_eq!(tokens, ["FLTR"], "expected only the FLTR hint, got {suggestions:?}");
            assert!(
                suggestions[0].suggestions.contains(&"FILTER".to_string()),
                "expected FILTER among {:?}",
                suggestions[0].suggestions
            );
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

#[test]
fn a_parse_failure_with_no_near_keyword_carries_no_hint() {
    // Unbalanced but correctly spelled: nothing to suggest, so the error stays terse.
    let error = terse_to_sparql("SELECT ?s WHERE { ?s ?p").expect_err("invalid SPARQL must fail");

    match error {
        TerseError::CanaryFailed { suggestions, .. } => {
            assert!(suggestions.is_empty(), "expected no hint, got {suggestions:?}");
        }
        other => panic!("expected CanaryFailed, got {other:?}"),
    }
}

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
fn canonical_sparql_passes_through_byte_identically() {
    let query = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";
    let expansion = terse_to_sparql(query).expect("canonical SPARQL must pass through");

    assert_eq!(expansion.canonical_sparql, query);
    assert!(expansion.resolutions.is_empty());
    assert!(expansion.warnings.is_empty());
}
