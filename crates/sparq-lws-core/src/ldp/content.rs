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

/// Negotiate the response RDF format from an `Accept` header against the formats this server can
/// produce (Turtle + JSON-LD).
///
/// Returns the best acceptable [`RdfFormat`], or `None` when the client explicitly accepts neither
/// (the caller then responds 406). An ABSENT or `*/*` Accept defaults to the resource's stored
/// format (`stored`) — the most faithful, zero-cost response. Quality values (`q=`) are honoured:
/// the highest-q acceptable type wins, ties broken in the header's order.
///
/// This is a deliberately small, dependency-free `Accept` parser sufficient for the two RDF media
/// types the server serves; it is NOT a general RFC 7231 content-negotiation engine.
pub fn negotiate_accept(accept: Option<&str>, stored: RdfFormat) -> Option<RdfFormat> {
    let raw = match accept {
        None => return Some(stored),
        Some(s) if s.trim().is_empty() => return Some(stored),
        Some(s) => s,
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

    fn bump(slot: &mut Option<f32>, q: f32) {
        *slot = Some(slot.unwrap_or(0.0).max(q));
    }

    for part in raw.split(',') {
        let mut it = part.split(';');
        let media = it.next().unwrap_or("").trim().to_ascii_lowercase();
        // Parse an optional q-value; default 1.0; clamp to [0,1]; a malformed q is treated as 0
        // (RFC 7231 §5.3.1 — an unparseable weight is not "accepted").
        let mut q: f32 = 1.0;
        for param in it {
            let p = param.trim();
            if let Some(v) = p.strip_prefix("q=").or_else(|| p.strip_prefix("Q=")) {
                q = v.trim().parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0);
            }
        }
        match media.as_str() {
            "text/turtle" => bump(&mut q_turtle, q),
            "application/ld+json" => bump(&mut q_jsonld, q),
            "text/*" => bump(&mut q_text_star, q),
            "application/*" => bump(&mut q_app_star, q),
            "*/*" => bump(&mut q_any, q),
            _ => {}
        }
    }

    // Resolve each concrete type's effective weight: an explicit q wins; else the most specific
    // applicable wildcard (`type/*`), else `*/*`. A type with no applicable range is not accepted.
    let turtle = q_turtle.or(q_text_star).or(q_any).unwrap_or(0.0);
    let jsonld = q_jsonld.or(q_app_star).or(q_any).unwrap_or(0.0);

    if turtle <= 0.0 && jsonld <= 0.0 {
        return None; // 406 — the client accepts neither producible type.
    }
    // Highest q wins; on a tie prefer the resource's stored format (cheapest, most faithful).
    Some(match stored {
        RdfFormat::Turtle if turtle >= jsonld => RdfFormat::Turtle,
        RdfFormat::JsonLd if jsonld >= turtle => RdfFormat::JsonLd,
        _ if turtle >= jsonld => RdfFormat::Turtle,
        _ => RdfFormat::JsonLd,
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
            negotiate_accept(Some(""), RdfFormat::JsonLd),
            Some(RdfFormat::JsonLd)
        );
        assert_eq!(
            negotiate_accept(Some("*/*"), RdfFormat::Turtle),
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
    fn unacceptable_accept_is_none_406() {
        assert_eq!(
            negotiate_accept(Some("application/xml"), RdfFormat::Turtle),
            None
        );
        assert_eq!(negotiate_accept(Some("text/html"), RdfFormat::JsonLd), None);
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
