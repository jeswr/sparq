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
    serialize_triples_negotiated, JsonLdProfileParam, NegotiatedFormat, RdfFormat,
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

// --- honouring the profile: Content-Type echo + document form (bead sq-10ty4) ----------------

/// The negotiated outcome for an explicit `application/ld+json` range carrying `profile`.
fn negotiated(profile_iri: &str) -> NegotiatedFormat {
    negotiate_accept_with_profile(
        Some(&format!("application/ld+json;profile=\"{profile_iri}\"")),
        RdfFormat::Turtle,
    )
    .expect("acceptable")
}

#[test]
fn content_type_echoes_the_honoured_profile_per_the_iana_registration() {
    assert_eq!(
        negotiated(COMPACTED).content_type(),
        format!("application/ld+json;profile=\"{COMPACTED}\"")
    );
    assert_eq!(
        negotiated(EXPANDED).content_type(),
        format!("application/ld+json;profile=\"{EXPANDED}\"")
    );
    // No honoured profile ⇒ the bare media type (both formats).
    let plain = negotiate_accept_with_profile(Some("application/ld+json"), RdfFormat::Turtle)
        .expect("acceptable");
    assert_eq!(plain.content_type(), "application/ld+json");
    let turtle =
        negotiate_accept_with_profile(Some("text/turtle"), RdfFormat::Turtle).expect("acceptable");
    assert_eq!(turtle.content_type(), "text/turtle");
}

#[test]
fn expanded_profile_output_is_the_default_serialisation_byte_for_byte() {
    // The serialiser's default output IS the expanded document form (top-level array, no
    // `@context`), so the expanded profile is honoured with byte-identical output — which is why
    // it shares the plain-JSON-LD variant ETag.
    let turtle = b"<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";
    let triples =
        parse_to_triples(RdfFormat::Turtle, turtle, "https://pod.example/alice/data").unwrap();
    let default = serialize_triples(RdfFormat::JsonLd, &triples).unwrap();
    let expanded = serialize_triples_negotiated(negotiated(EXPANDED), &triples).unwrap();
    assert_eq!(expanded, default);
    // And it really is the expanded form: a top-level ARRAY of node objects.
    let doc: serde_json::Value = serde_json::from_slice(&expanded).unwrap();
    assert!(doc.is_array(), "expanded form is a top-level array: {doc}");
}

#[test]
fn compacted_profile_output_is_genuinely_compacted_and_round_trips() {
    // A resource with a plain literal, a typed literal, a language-tagged literal, an IRI object
    // and a multi-valued property — covering every value-compaction rule.
    let turtle = br#"@prefix foaf: <http://xmlns.com/foaf/0.1/> .
<https://pod.example/alice/data#me>
    foaf:name "Alice" ;
    foaf:age "30"^^<http://www.w3.org/2001/XMLSchema#integer> ;
    foaf:label "Alice"@en ;
    foaf:knows <https://pod.example/bob#me>, <https://pod.example/carol#me> .
"#;
    let triples =
        parse_to_triples(RdfFormat::Turtle, turtle, "https://pod.example/alice/data").unwrap();
    let bytes = serialize_triples_negotiated(negotiated(COMPACTED), &triples).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // Single subject ⇒ the top-level value is the node object itself (no @graph wrapper).
    let node = doc
        .as_object()
        .expect("a single-subject document compacts to the node object");
    assert_eq!(
        node.get("@id").and_then(|v| v.as_str()),
        Some("https://pod.example/alice/data#me")
    );
    // A lone plain-string @value compacts to the bare string (no array, no value object)…
    assert_eq!(
        node.get("http://xmlns.com/foaf/0.1/name"),
        Some(&serde_json::json!("Alice"))
    );
    // …a typed literal keeps its value object (the @type must survive)…
    assert_eq!(
        node.get("http://xmlns.com/foaf/0.1/age"),
        Some(&serde_json::json!({
            "@type": "http://www.w3.org/2001/XMLSchema#integer",
            "@value": "30"
        }))
    );
    // …a language-tagged literal keeps its value object…
    assert_eq!(
        node.get("http://xmlns.com/foaf/0.1/label"),
        Some(&serde_json::json!({"@language": "en", "@value": "Alice"}))
    );
    // …and a multi-valued property stays an array (of compacted node references).
    assert_eq!(
        node.get("http://xmlns.com/foaf/0.1/knows"),
        Some(&serde_json::json!([
            {"@id": "https://pod.example/bob#me"},
            {"@id": "https://pod.example/carol#me"}
        ]))
    );

    // The compacted document still parses back (locally — no context, nothing to fetch) to the
    // SAME triples, so compaction preserves both the content and the SSRF posture.
    let mut reparsed =
        parse_to_triples(RdfFormat::JsonLd, &bytes, "https://pod.example/alice/data").unwrap();
    let mut expected = triples.clone();
    reparsed.sort_by_key(|t| t.to_string());
    expected.sort_by_key(|t| t.to_string());
    assert_eq!(reparsed, expected);
}

#[test]
fn compacted_multi_subject_document_wraps_in_graph() {
    let turtle = br#"<https://pod.example/a> <http://xmlns.com/foaf/0.1/name> "A" .
<https://pod.example/b> <http://xmlns.com/foaf/0.1/name> "B" .
"#;
    let triples =
        parse_to_triples(RdfFormat::Turtle, turtle, "https://pod.example/alice/data").unwrap();
    let bytes = serialize_triples_negotiated(negotiated(COMPACTED), &triples).unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let graph = doc
        .get("@graph")
        .and_then(|v| v.as_array())
        .expect("a multi-subject document compacts to {\"@graph\": [...]}");
    assert_eq!(graph.len(), 2);
    // Each member is a compacted node object.
    assert_eq!(
        graph[0].get("http://xmlns.com/foaf/0.1/name"),
        Some(&serde_json::json!("A"))
    );
}

#[test]
fn variant_suffix_is_profile_specific_only_when_the_bytes_differ() {
    // Compacted output differs from the default serialisation ⇒ a DISTINCT variant suffix;
    // expanded output is byte-identical to it ⇒ the SAME suffix as plain JSON-LD.
    let plain = negotiate_accept_with_profile(Some("application/ld+json"), RdfFormat::Turtle)
        .expect("acceptable");
    assert_eq!(plain.variant_suffix(), "jsonld");
    assert_eq!(negotiated(EXPANDED).variant_suffix(), "jsonld");
    assert_eq!(negotiated(COMPACTED).variant_suffix(), "jsonld-c");
    let turtle =
        negotiate_accept_with_profile(Some("text/turtle"), RdfFormat::JsonLd).expect("acceptable");
    assert_eq!(turtle.variant_suffix(), "ttl");
}

// --- the read-only line/N3 formats (gh-4879) -------------------------------------------------

#[test]
fn an_accept_naming_a_line_or_n3_format_is_served() {
    // Each of the three read formats is now negotiable in its own right, instead of degrading to
    // the Solid default (`text/turtle`) because the media type was unrecognised.
    assert_eq!(fmt("application/n-triples"), Some(RdfFormat::NTriples));
    assert_eq!(fmt("application/n-quads"), Some(RdfFormat::NQuads));
    assert_eq!(fmt("text/n3"), Some(RdfFormat::N3));
    // Case-insensitively on the media type, and with parameters ignored.
    assert_eq!(fmt("Application/N-Triples;q=1"), Some(RdfFormat::NTriples));
    // q-values order them against the read+write formats like any other range.
    assert_eq!(
        fmt("text/turtle;q=0.3, application/n-quads;q=0.9"),
        Some(RdfFormat::NQuads)
    );
    assert_eq!(
        fmt("text/n3;q=0.2, application/ld+json;q=0.7"),
        Some(RdfFormat::JsonLd)
    );
}

#[test]
fn the_read_formats_do_not_disturb_the_existing_wildcard_outcomes() {
    // `text/*` now also covers `text/n3` and `application/*` the two `application/…` line formats,
    // but the fixed producible order keeps the previous winners.
    assert_eq!(fmt("text/*"), Some(RdfFormat::Turtle));
    assert_eq!(fmt("application/*"), Some(RdfFormat::JsonLd));
    assert_eq!(
        negotiate_accept(Some("application/*"), RdfFormat::JsonLd),
        Some(RdfFormat::JsonLd)
    );
    assert_eq!(fmt("*/*"), Some(RdfFormat::Turtle));
    // An unknown type still degrades to Turtle rather than to a read-only format.
    assert_eq!(fmt("image/png"), Some(RdfFormat::Turtle));
}

#[test]
fn each_read_format_serialises_to_a_reparseable_document() {
    // TWO triples on ONE subject: Turtle groups them with `;` while N-Triples repeats the subject
    // per line, so the two serialisations are distinguishable here (they coincide for one triple).
    let turtle = b"<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" ; \
        <http://xmlns.com/foaf/0.1/nick> \"Al\" .";
    let triples =
        parse_to_triples(RdfFormat::Turtle, turtle, "https://pod.example/alice/data").unwrap();
    let ttl = serialize_triples(RdfFormat::Turtle, &triples).unwrap();

    // N-Triples: one canonical line per triple, and the N-Quads document of the same (default)
    // graph is byte-identical — a default-graph quad carries no graph label.
    let nt = serialize_triples(RdfFormat::NTriples, &triples).unwrap();
    let nq = serialize_triples(RdfFormat::NQuads, &triples).unwrap();
    assert_eq!(nt, nq);
    assert_ne!(nt, ttl, "the line format is not the Turtle serialisation");
    assert_eq!(
        String::from_utf8(nt.clone()).unwrap().lines().count(),
        triples.len()
    );

    // N3 is served as the Turtle syntax it subsumes.
    let n3 = serialize_triples(RdfFormat::N3, &triples).unwrap();
    assert_eq!(n3, ttl);
    assert_ne!(n3, nt);

    // All three documents parse back to the same triple set through the Turtle grammar (which
    // subsumes N-Triples), proving the bytes really carry the resource's triples.
    for bytes in [&nt, &nq, &n3] {
        let reparsed =
            parse_to_triples(RdfFormat::Turtle, bytes, "https://pod.example/alice/data").unwrap();
        assert_eq!(reparsed, triples);
    }
}

#[test]
fn each_read_format_gets_its_own_content_type_and_variant_tag() {
    for (accept, media_type, suffix) in [
        ("application/n-triples", "application/n-triples", "nt"),
        ("application/n-quads", "application/n-quads", "nq"),
        ("text/n3", "text/n3", "n3"),
    ] {
        let negotiated =
            negotiate_accept_with_profile(Some(accept), RdfFormat::Turtle).expect("acceptable");
        assert_eq!(negotiated.content_type(), media_type);
        assert_eq!(negotiated.variant_suffix(), suffix);
        // A read-only format is never the stored format, so it always re-serialises.
        assert!(!negotiated.serves_stored_verbatim(RdfFormat::Turtle));
        assert!(!negotiated.serves_stored_verbatim(RdfFormat::JsonLd));
    }
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
