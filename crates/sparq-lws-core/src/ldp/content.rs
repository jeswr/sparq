// AUTHORED-BY Claude Opus 4.8
//! RDF content-type handling for the LDP path.
//!
//! M1 supports the two RDF formats the production server allows: **Turtle** and **JSON-LD** (the
//! house rule — `oxttl` for Turtle, `oxjsonld` for JSON-LD, per the spike §4). This module classifies
//! a media type and validates that a body parses as RDF in that format, returning the parsed quad
//! count (proof the body is well-formed RDF) without retaining the graph — the slice stores bytes
//! verbatim and lets SPARQ be authoritative for the triples.
//!
//! M2: content negotiation (an `Accept`-driven serialisation choice) + re-serialisation between the
//! two RDF formats now land here ([`negotiate_accept`] + [`serialize_triples`]). The JSON-LD
//! `noRemoteContextLoader` SSRF posture (oxjsonld is local-only by construction) ports favourably.
//! The Solid read path is hardened via [`negotiate_accept_with_profile`]: q-values, the JSON-LD
//! `profile` parameter (expanded vs compacted, surfaced in [`NegotiatedFormat`]), and an `Accept`
//! naming no producible type degrading to `text/turtle` (the Solid default) instead of a 406.
//! Still M2-next: N-Triples/N-Quads/N3 read formats.

use oxjsonld::{JsonLdParser, JsonLdSerializer};
use oxrdf::{GraphNameRef, QuadRef, Triple};
use oxttl::{TurtleParser, TurtleSerializer};

use crate::error::ServerError;

/// A supported RDF media type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdfFormat {
    Turtle,
    JsonLd,
}

impl RdfFormat {
    /// The canonical media type string.
    pub fn media_type(self) -> &'static str {
        match self {
            RdfFormat::Turtle => "text/turtle",
            RdfFormat::JsonLd => "application/ld+json",
        }
    }
}

/// Classify a `Content-Type` header value into a supported [`RdfFormat`].
///
/// The media-type is matched case-insensitively and any parameters (e.g. `; charset=utf-8`) are
/// ignored. An unsupported or absent type is an [`ServerError::UnsupportedMediaType`] — the LDP
/// surface accepts only RDF on this slice's single-resource PUT path.
pub fn classify(content_type: Option<&str>) -> Result<RdfFormat, ServerError> {
    let raw = content_type.ok_or_else(|| ServerError::UnsupportedMediaType("missing".into()))?;
    let essence = raw
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match essence.as_str() {
        "text/turtle" => Ok(RdfFormat::Turtle),
        "application/ld+json" => Ok(RdfFormat::JsonLd),
        other => Err(ServerError::UnsupportedMediaType(other.to_string())),
    }
}

/// Validate that `body` parses as RDF in `format`, returning the number of triples/quads parsed.
///
/// `base_iri` is the resource's own IRI: per the LDP/RDF convention a server resolves relative IRIs
/// in a submitted document against the request URI, so a document that uses relative IRIs is valid.
/// This proves the body is well-formed RDF before the slice stores it. The parsed graph is NOT
/// retained — SPARQ is the authoritative triple store; the blob store keeps the bytes verbatim.
pub fn validate_rdf(format: RdfFormat, body: &[u8], base_iri: &str) -> Result<usize, ServerError> {
    Ok(parse_to_triples(format, body, base_iri)?.len())
}

/// Parse `body` (in `format`) into its default-graph triples, resolving relative IRIs against
/// `base_iri`.
///
/// Both source formats are reduced to a flat `Vec<Triple>`: the Turtle parser already yields the
/// default graph; the JSON-LD parser yields quads, of which only the default graph is retained (a
/// Solid RDF *resource* is a single graph — named graphs in a submitted JSON-LD document are not
/// part of the resource's triples). This is the shared parse step behind both validation and content
/// negotiation, so an unparseable body is rejected (a 400) before any storage or re-serialisation.
pub fn parse_to_triples(
    format: RdfFormat,
    body: &[u8],
    base_iri: &str,
) -> Result<Vec<Triple>, ServerError> {
    match format {
        RdfFormat::Turtle => {
            let parser = TurtleParser::new()
                .with_base_iri(base_iri)
                .map_err(|e| ServerError::BadRequest(format!("invalid base IRI: {e}")))?;
            let mut triples = Vec::new();
            for triple in parser.for_slice(body) {
                let t =
                    triple.map_err(|e| ServerError::BadRequest(format!("invalid Turtle: {e}")))?;
                triples.push(t);
            }
            Ok(triples)
        }
        RdfFormat::JsonLd => {
            // oxjsonld is local-only by construction (no remote context loader) — the SSRF-safe
            // posture the production server enforces explicitly is the default here.
            let parser = JsonLdParser::new()
                .with_base_iri(base_iri)
                .map_err(|e| ServerError::BadRequest(format!("invalid base IRI: {e}")))?;
            let mut triples = Vec::new();
            for quad in parser.for_slice(body) {
                let q =
                    quad.map_err(|e| ServerError::BadRequest(format!("invalid JSON-LD: {e}")))?;
                // A resource is a single (default) graph; ignore any named-graph quads.
                if q.graph_name == oxrdf::GraphName::DefaultGraph {
                    triples.push(Triple::new(q.subject, q.predicate, q.object));
                }
            }
            Ok(triples)
        }
    }
}

/// Serialise a triple set into `format`, returning the bytes.
///
/// The serialisation is unconditioned (no base-IRI abbreviation) so the output is self-contained and
/// stable. Used by content negotiation on read (re-render the stored Turtle as JSON-LD or vice
/// versa) and after a PATCH (re-serialise the patched graph for storage).
pub fn serialize_triples(format: RdfFormat, triples: &[Triple]) -> Result<Vec<u8>, ServerError> {
    match format {
        RdfFormat::Turtle => {
            let mut ser = TurtleSerializer::new().for_writer(Vec::new());
            for t in triples {
                ser.serialize_triple(t)
                    .map_err(|e| ServerError::Storage(format!("turtle serialise: {e}")))?;
            }
            ser.finish()
                .map_err(|e| ServerError::Storage(format!("turtle serialise: {e}")))
        }
        RdfFormat::JsonLd => {
            let mut ser = JsonLdSerializer::new().for_writer(Vec::new());
            for t in triples {
                let q = QuadRef::new(
                    t.subject.as_ref(),
                    t.predicate.as_ref(),
                    t.object.as_ref(),
                    GraphNameRef::DefaultGraph,
                );
                ser.serialize_quad(q)
                    .map_err(|e| ServerError::Storage(format!("json-ld serialise: {e}")))?;
            }
            ser.finish()
                .map_err(|e| ServerError::Storage(format!("json-ld serialise: {e}")))
        }
    }
}

/// A JSON-LD document form requested via the `Accept` header's `profile` media-type parameter
/// ([JSON-LD 1.1 IANA registration](https://www.w3.org/TR/json-ld11/#iana-considerations)).
///
/// Only the two READ-relevant document forms are honoured: `expanded` and `compacted`. Other
/// registered profile IRIs (`flattened`, `framed`, …) are ignored — per the registration a profile
/// parameter is a client *preference* a server may decline, never an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonLdProfileParam {
    /// `http://www.w3.org/ns/json-ld#expanded`
    Expanded,
    /// `http://www.w3.org/ns/json-ld#compacted`
    Compacted,
}

impl JsonLdProfileParam {
    /// Extract the first honoured document form from a `profile` parameter VALUE (the part after
    /// `profile=`), deterministically.
    ///
    /// The value is a (usually quoted) whitespace-separated list of profile IRIs; the FIRST one
    /// that names an honoured form wins (client order = client preference). Unknown IRIs are
    /// skipped. IRIs match the canonical registration exactly (IRIs are case-sensitive) — matched
    /// via [`oxjsonld::JsonLdProfile::from_iri`] so the accepted spellings stay pinned to the
    /// vendored JSON-LD implementation, not a hand-copied string.
    fn from_param_value(value: &str) -> Option<Self> {
        value
            .trim()
            .trim_matches('"')
            .split_ascii_whitespace()
            .find_map(|iri| match oxjsonld::JsonLdProfile::from_iri(iri) {
                Some(oxjsonld::JsonLdProfile::Expanded) => Some(Self::Expanded),
                Some(oxjsonld::JsonLdProfile::Compacted) => Some(Self::Compacted),
                _ => None,
            })
    }
}

/// The outcome of [`negotiate_accept_with_profile`]: the chosen response format, plus — when that
/// format is JSON-LD and the winning explicit `application/ld+json` range carried a recognised
/// `profile` parameter — the requested document form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedFormat {
    /// The RDF format the response body should be serialised in.
    pub format: RdfFormat,
    /// The JSON-LD document form the client asked for, when [`Self::format`] is
    /// [`RdfFormat::JsonLd`] and an explicit `application/ld+json` range carried one. `None`
    /// otherwise (including a JSON-LD choice reached via an `application/*` / `*/*` wildcard —
    /// wildcard ranges carry no honoured profile). The serialiser currently emits the streaming
    /// (expanded-compatible) document form regardless; this field lets the handler echo the
    /// selected profile and honour `compacted` output once the serialiser can produce it.
    pub jsonld_profile: Option<JsonLdProfileParam>,
}

/// Negotiate the response RDF format from an `Accept` header against the formats this server can
/// produce (Turtle + JSON-LD). Thin wrapper over [`negotiate_accept_with_profile`] (the single
/// `Accept` decision point) that drops the JSON-LD profile detail.
pub fn negotiate_accept(accept: Option<&str>, stored: RdfFormat) -> Option<RdfFormat> {
    negotiate_accept_with_profile(accept, stored).map(|n| n.format)
}

/// Negotiate the response RDF format (and JSON-LD document form) from an `Accept` header against
/// the formats this server can produce (Turtle + JSON-LD).
///
/// - An ABSENT `Accept` means "no preference": the resource's stored format (`stored`) — the most
///   faithful, zero-cost response (as does `*/*`, which covers both producible types).
/// - Quality values (`q=`) are honoured: the highest-q acceptable type wins; a q-tie prefers the
///   stored format (cheapest); range ties break in the header's order.
/// - The `application/ld+json` `profile` parameter is honoured deterministically: the profile of
///   the best-q explicit `ld+json` range is surfaced (first-listed range wins q-ties), reduced to
///   the first honoured document form in its value.
/// - A PRESENT header that names/covers NO producible type (blank, or only unrecognised types like
///   `text/html`) falls back to `text/turtle` — the Solid default representation — rather than
///   failing the read (never a 406, never a 500).
/// - `None` (⇒ the caller's 406) is returned ONLY when the client EXPLICITLY refused every
///   producible type it covered (all applicable ranges at `q=0`) — serving a representation the
///   client q=0-refused would violate RFC 7231 §5.3.1.
///
/// This is a deliberately small, dependency-free `Accept` parser sufficient for the two RDF media
/// types the server serves; it is NOT a general RFC 7231 content-negotiation engine (in
/// particular, parameters are split on `;`, so a quoted parameter value containing `;` — which no
/// registered profile IRI does — would mis-parse).
pub fn negotiate_accept_with_profile(
    accept: Option<&str>,
    stored: RdfFormat,
) -> Option<NegotiatedFormat> {
    // ABSENT header — "no preference": the stored format, byte-identical to the stored bytes.
    let Some(raw) = accept else {
        return Some(NegotiatedFormat {
            format: stored,
            jsonld_profile: None,
        });
    };

    // Track the best q for each producible type, plus the matching type-range wildcards. A `text/*`
    // range can only cover Turtle (`text/turtle`); an `application/*` range only JSON-LD
    // (`application/ld+json`); `*/*` covers both. Each is kept separately so a `text/*` request never
    // yields JSON-LD (the bug roborev flagged).
    let mut q_turtle: Option<f32> = None;
    let mut q_jsonld: Option<f32> = None;
    let mut q_text_star: Option<f32> = None; // covers Turtle only
    let mut q_app_star: Option<f32> = None; // covers JSON-LD only
    let mut q_any: Option<f32> = None; // covers both

    // The profile carried by the BEST explicit `application/ld+json` range seen so far. Updated
    // only on a STRICTLY greater q, so the first-listed range wins q-ties — deterministic, and
    // consistent with `bump`, under which a later equal-q range changes nothing.
    let mut jsonld_profile: Option<JsonLdProfileParam> = None;

    fn bump(slot: &mut Option<f32>, q: f32) {
        *slot = Some(slot.unwrap_or(0.0).max(q));
    }

    for part in raw.split(',') {
        let mut it = part.split(';');
        let media = it.next().unwrap_or("").trim().to_ascii_lowercase();
        // Parse the parameters: an optional q-value (default 1.0; clamp to [0,1]; a malformed q is
        // treated as 0 — RFC 7231 §5.3.1, an unparseable weight is not "accepted") and an optional
        // `profile` (media-type parameter names are case-insensitive; the IRI value is not).
        let mut q: f32 = 1.0;
        let mut profile: Option<JsonLdProfileParam> = None;
        for param in it {
            let Some((name, value)) = param.split_once('=') else {
                continue;
            };
            match name.trim().to_ascii_lowercase().as_str() {
                "q" => q = value.trim().parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0),
                "profile" => profile = JsonLdProfileParam::from_param_value(value),
                _ => {}
            }
        }
        match media.as_str() {
            "text/turtle" => bump(&mut q_turtle, q),
            "application/ld+json" => {
                if q_jsonld.is_none_or(|cur| q > cur) {
                    jsonld_profile = profile;
                }
                bump(&mut q_jsonld, q);
            }
            "text/*" => bump(&mut q_text_star, q),
            "application/*" => bump(&mut q_app_star, q),
            "*/*" => bump(&mut q_any, q),
            _ => {}
        }
    }

    // A header that named/covered NO producible type — blank, or only unrecognised media types
    // (`text/html`, `application/xml`, …). Solid default: degrade to Turtle, don't fail the read.
    if q_turtle.is_none()
        && q_jsonld.is_none()
        && q_text_star.is_none()
        && q_app_star.is_none()
        && q_any.is_none()
    {
        return Some(NegotiatedFormat {
            format: RdfFormat::Turtle,
            jsonld_profile: None,
        });
    }

    // Resolve each concrete type's effective weight: an explicit q wins; else the most specific
    // applicable wildcard (`type/*`), else `*/*`. A type with no applicable range is not accepted.
    let turtle = q_turtle.or(q_text_star).or(q_any).unwrap_or(0.0);
    let jsonld = q_jsonld.or(q_app_star).or(q_any).unwrap_or(0.0);

    if turtle <= 0.0 && jsonld <= 0.0 {
        return None; // 406 — the client EXPLICITLY refused (q=0) every covered producible type.
    }
    // Highest q wins; on a tie prefer the resource's stored format (cheapest, most faithful).
    let format = match stored {
        RdfFormat::Turtle if turtle >= jsonld => RdfFormat::Turtle,
        RdfFormat::JsonLd if jsonld >= turtle => RdfFormat::JsonLd,
        _ if turtle >= jsonld => RdfFormat::Turtle,
        _ => RdfFormat::JsonLd,
    };
    Some(NegotiatedFormat {
        format,
        // The profile is only meaningful for a JSON-LD response, and only when JSON-LD won via an
        // explicit `application/ld+json` range (a wildcard win carries no honoured profile).
        jsonld_profile: match format {
            RdfFormat::JsonLd => jsonld_profile,
            RdfFormat::Turtle => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const IRI: &str = "https://pod.example/alice/data";
    const TURTLE: &str =
        "<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

    #[test]
    fn absent_or_wildcard_accept_keeps_stored_format() {
        assert_eq!(
            negotiate_accept(None, RdfFormat::Turtle),
            Some(RdfFormat::Turtle)
        );
        assert_eq!(
            negotiate_accept(None, RdfFormat::JsonLd),
            Some(RdfFormat::JsonLd)
        );
        assert_eq!(
            negotiate_accept(Some("*/*"), RdfFormat::Turtle),
            Some(RdfFormat::Turtle)
        );
    }

    #[test]
    fn blank_accept_falls_back_to_turtle_the_solid_default() {
        // A PRESENT-but-blank Accept is treated as naming no producible type: the Solid default
        // (text/turtle), even when the stored format is JSON-LD.
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
    fn explicit_jsonld_wins_over_stored_turtle() {
        assert_eq!(
            negotiate_accept(Some("application/ld+json"), RdfFormat::Turtle),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn explicit_turtle_wins_over_stored_jsonld() {
        assert_eq!(
            negotiate_accept(Some("text/turtle"), RdfFormat::JsonLd),
            Some(RdfFormat::Turtle)
        );
    }

    #[test]
    fn q_values_are_honoured() {
        // JSON-LD preferred by weight even though Turtle is listed first.
        assert_eq!(
            negotiate_accept(
                Some("text/turtle;q=0.5, application/ld+json;q=0.9"),
                RdfFormat::Turtle
            ),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn q_zero_excludes_a_type() {
        // Turtle explicitly refused (q=0); JSON-LD acceptable ⇒ JSON-LD.
        assert_eq!(
            negotiate_accept(
                Some("text/turtle;q=0, application/ld+json"),
                RdfFormat::Turtle
            ),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn unrecognised_accept_falls_back_to_turtle_not_406() {
        // An Accept naming only types the server can't produce degrades to the Solid default
        // (text/turtle) — the read never fails over an exotic Accept.
        assert_eq!(
            negotiate_accept(Some("application/xml"), RdfFormat::Turtle),
            Some(RdfFormat::Turtle)
        );
        assert_eq!(
            negotiate_accept(Some("text/html"), RdfFormat::JsonLd),
            Some(RdfFormat::Turtle)
        );
    }

    #[test]
    fn explicit_q_zero_refusal_of_every_covered_type_is_none_406() {
        // The ONLY remaining 406: the client covered our producible types and explicitly refused
        // them all with q=0 — serving one anyway would violate the client's stated refusal.
        assert_eq!(
            negotiate_accept(
                Some("text/turtle;q=0, application/ld+json;q=0"),
                RdfFormat::Turtle
            ),
            None
        );
        assert_eq!(negotiate_accept(Some("*/*;q=0"), RdfFormat::JsonLd), None);
    }

    #[test]
    fn text_star_covers_only_turtle() {
        // `text/*` maps to Turtle, never JSON-LD — even when the stored format is JSON-LD.
        assert_eq!(
            negotiate_accept(Some("text/*"), RdfFormat::JsonLd),
            Some(RdfFormat::Turtle)
        );
        assert_eq!(
            negotiate_accept(Some("text/*"), RdfFormat::Turtle),
            Some(RdfFormat::Turtle)
        );
    }

    #[test]
    fn application_star_covers_only_jsonld() {
        // `application/*` maps to JSON-LD, never Turtle — even when the stored format is Turtle.
        assert_eq!(
            negotiate_accept(Some("application/*"), RdfFormat::Turtle),
            Some(RdfFormat::JsonLd)
        );
        assert_eq!(
            negotiate_accept(Some("application/*"), RdfFormat::JsonLd),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn any_wildcard_covers_both_and_keeps_stored() {
        assert_eq!(
            negotiate_accept(Some("*/*"), RdfFormat::Turtle),
            Some(RdfFormat::Turtle)
        );
        assert_eq!(
            negotiate_accept(Some("*/*"), RdfFormat::JsonLd),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn explicit_beats_wildcard() {
        // An explicit application/ld+json at higher q wins over a text/* range.
        assert_eq!(
            negotiate_accept(
                Some("text/*;q=0.3, application/ld+json;q=0.9"),
                RdfFormat::Turtle
            ),
            Some(RdfFormat::JsonLd)
        );
    }

    #[test]
    fn turtle_round_trips_through_jsonld_and_back() {
        // Parse Turtle → serialise JSON-LD → parse JSON-LD → same single triple.
        let triples = parse_to_triples(RdfFormat::Turtle, TURTLE.as_bytes(), IRI).unwrap();
        assert_eq!(triples.len(), 1);
        let jsonld = serialize_triples(RdfFormat::JsonLd, &triples).unwrap();
        let reparsed = parse_to_triples(RdfFormat::JsonLd, &jsonld, IRI).unwrap();
        assert_eq!(reparsed, triples);
    }

    #[test]
    fn serialise_to_turtle_is_reparseable() {
        let triples = parse_to_triples(RdfFormat::Turtle, TURTLE.as_bytes(), IRI).unwrap();
        let ttl = serialize_triples(RdfFormat::Turtle, &triples).unwrap();
        let reparsed = parse_to_triples(RdfFormat::Turtle, &ttl, IRI).unwrap();
        assert_eq!(reparsed, triples);
    }
}
