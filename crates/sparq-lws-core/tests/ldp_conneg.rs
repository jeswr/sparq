// AUTHORED-BY Claude Fable 5
//! Acceptance tests for the hardened Solid read content negotiation (bead sq-ah3oj):
//! `negotiate_accept` / `negotiate_accept_with_profile` in `ldp::content`.
//!
//! Covers, per the bead's acceptance criteria:
//! - q-value ordering picks the highest acceptable format;
//! - the `application/ld+json` `profile` parameter deterministically selects expanded vs
//!   compacted (surfaced in [`NegotiatedFormat::jsonld_profile`]);
//! - a blank/unknown `Accept` falls back to `text/turtle` (the Solid default), never an error;
//! - the JSON-LD path stays LOCAL-ONLY: a document whose `@context` is a remote IRI fails to
//!   parse (no remote context loader is reachable from any code path).

use sparq_lws_core::ldp::content::{
    negotiate_accept, negotiate_accept_with_profile, parse_to_triples, serialize_triples,
    JsonLdProfileParam, RdfFormat,
};

const EXPANDED: &str = "http://www.w3.org/ns/json-ld#expanded";
const COMPACTED: &str = "http://www.w3.org/ns/json-ld#compacted";

/// Shorthand: the negotiated format for `accept` over a stored-Turtle resource.
fn fmt(accept: &str) -> Option<RdfFormat> {
    negotiate_accept(Some(accept), RdfFormat::Turtle)
}

/// Shorthand: the negotiated JSON-LD profile for `accept` over a stored-Turtle resource.
fn profile(accept: &str) -> Option<JsonLdProfileParam> {
    negotiate_accept_with_profile(Some(accept), RdfFormat::Turtle)
        .expect("acceptable")
        .jsonld_profile
}

// --- q-value ordering ----------------------------------------------------------------------

#[test]
fn q_value_ordering_picks_the_highest_acceptable_format() {
    // JSON-LD outweighs Turtle although listed second…
    assert_eq!(
        fmt("text/turtle;q=0.5, application/ld+json;q=0.9"),
        Some(RdfFormat::JsonLd)
    );
    // …and vice versa.
    assert_eq!(
        fmt("application/ld+json;q=0.2, text/turtle;q=0.8"),
        Some(RdfFormat::Turtle)
    );
    // An explicit type outweighs a wildcard at lower q.
    assert_eq!(
        fmt("*/*;q=0.1, application/ld+json;q=0.9"),
        Some(RdfFormat::JsonLd)
    );
    // A q-tie prefers the STORED format (cheapest, most faithful).
    assert_eq!(
        negotiate_accept(
            Some("text/turtle;q=0.7, application/ld+json;q=0.7"),
            RdfFormat::JsonLd
        ),
        Some(RdfFormat::JsonLd)
    );
}

#[test]
fn q_zero_excludes_a_type_and_the_other_still_wins() {
    assert_eq!(
        fmt("text/turtle;q=0, application/ld+json;q=0.1"),
        Some(RdfFormat::JsonLd)
    );
}

// --- the ld+json `profile` parameter --------------------------------------------------------

#[test]
fn profile_param_selects_expanded() {
    assert_eq!(
        profile(&format!("application/ld+json;profile=\"{EXPANDED}\"")),
        Some(JsonLdProfileParam::Expanded)
    );
}

#[test]
fn profile_param_selects_compacted() {
    assert_eq!(
        profile(&format!("application/ld+json;profile=\"{COMPACTED}\"")),
        Some(JsonLdProfileParam::Compacted)
    );
}

#[test]
fn profile_param_accepts_unquoted_value_and_case_insensitive_name() {
    assert_eq!(
        profile(&format!("application/ld+json;PROFILE={COMPACTED}")),
        Some(JsonLdProfileParam::Compacted)
    );
}

#[test]
fn profile_value_first_honoured_iri_wins_deterministically() {
    // A multi-IRI profile value: the FIRST honoured document form wins (client order).
    assert_eq!(
        profile(&format!(
            "application/ld+json;profile=\"http://www.w3.org/ns/json-ld#flattened {COMPACTED} {EXPANDED}\""
        )),
        Some(JsonLdProfileParam::Compacted)
    );
}

#[test]
fn unknown_profile_iris_are_ignored_not_an_error() {
    assert_eq!(
        profile("application/ld+json;profile=\"http://www.w3.org/ns/json-ld#framed\""),
        None
    );
    assert_eq!(profile("application/ld+json"), None);
}

#[test]
fn profile_of_the_best_q_ldjson_range_wins() {
    // The higher-q explicit ld+json range's profile is the one surfaced…
    assert_eq!(
        profile(&format!(
            "application/ld+json;profile=\"{COMPACTED}\";q=0.4, application/ld+json;profile=\"{EXPANDED}\";q=0.9"
        )),
        Some(JsonLdProfileParam::Expanded)
    );
    // …and a q-tie keeps the FIRST-listed range's profile (deterministic).
    assert_eq!(
        profile(&format!(
            "application/ld+json;profile=\"{COMPACTED}\";q=0.9, application/ld+json;profile=\"{EXPANDED}\";q=0.9"
        )),
        Some(JsonLdProfileParam::Compacted)
    );
}

#[test]
fn profile_is_none_when_turtle_wins_or_jsonld_won_via_wildcard() {
    // Turtle wins ⇒ no JSON-LD profile, even though one was sent.
    let n = negotiate_accept_with_profile(
        Some(&format!(
            "text/turtle;q=0.9, application/ld+json;profile=\"{EXPANDED}\";q=0.4"
        )),
        RdfFormat::Turtle,
    )
    .expect("acceptable");
    assert_eq!(n.format, RdfFormat::Turtle);
    assert_eq!(n.jsonld_profile, None);

    // JSON-LD reached ONLY via a wildcard range carries no honoured profile.
    let n = negotiate_accept_with_profile(Some("application/*"), RdfFormat::Turtle)
        .expect("acceptable");
    assert_eq!(n.format, RdfFormat::JsonLd);
    assert_eq!(n.jsonld_profile, None);
}

// --- Solid-default fallback + the remaining 406 ---------------------------------------------

#[test]
fn blank_accept_falls_back_to_turtle() {
    assert_eq!(
        negotiate_accept(Some(""), RdfFormat::JsonLd),
        Some(RdfFormat::Turtle)
    );
    assert_eq!(
        negotiate_accept(Some("   "), RdfFormat::JsonLd),
        Some(RdfFormat::Turtle)
    );
}

#[test]
fn unknown_accept_falls_back_to_turtle_never_an_error() {
    for accept in ["text/html", "image/png", "application/xml, text/html;q=0.5"] {
        assert_eq!(
            negotiate_accept(Some(accept), RdfFormat::JsonLd),
            Some(RdfFormat::Turtle),
            "Accept: {accept}"
        );
    }
}

#[test]
fn absent_accept_keeps_the_stored_format() {
    // The absent-Accept path is unchanged (byte-identical invariant): stored format, verbatim.
    assert_eq!(
        negotiate_accept(None, RdfFormat::JsonLd),
        Some(RdfFormat::JsonLd)
    );
    assert_eq!(
        negotiate_accept(None, RdfFormat::Turtle),
        Some(RdfFormat::Turtle)
    );
}

#[test]
fn only_an_explicit_q_zero_refusal_of_every_covered_type_is_406() {
    assert_eq!(fmt("text/turtle;q=0, application/ld+json;q=0"), None);
    assert_eq!(fmt("*/*;q=0"), None);
}

// --- SSRF posture: JSON-LD stays local-only --------------------------------------------------

#[test]
fn jsonld_remote_context_is_rejected_not_fetched() {
    // A JSON-LD body whose @context is a REMOTE IRI must fail to parse: the parser has no remote
    // document loader (local-only by construction), so no code path can be induced to fetch it.
    let body = br#"{
        "@context": "https://schema.org",
        "@id": "https://pod.example/alice/data#me",
        "name": "Alice"
    }"#;
    let err = parse_to_triples(RdfFormat::JsonLd, body, "https://pod.example/alice/data");
    assert!(
        err.is_err(),
        "a remote @context must be rejected, got: {err:?}"
    );
}

#[test]
fn jsonld_serialisation_round_trips_locally() {
    // The re-serialisation path emits self-contained (context-free) JSON-LD that parses back
    // locally — no context resolution, remote or otherwise, is involved.
    let turtle =
        b"<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";
    let triples =
        parse_to_triples(RdfFormat::Turtle, turtle, "https://pod.example/alice/data").unwrap();
    let jsonld = serialize_triples(RdfFormat::JsonLd, &triples).unwrap();
    let reparsed =
        parse_to_triples(RdfFormat::JsonLd, &jsonld, "https://pod.example/alice/data").unwrap();
    assert_eq!(reparsed, triples);
}
