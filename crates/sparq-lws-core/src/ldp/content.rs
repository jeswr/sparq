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
//! An honoured profile is wired through the read handlers (sq-10ty4): the response `Content-Type`
//! echoes it ([`NegotiatedFormat::content_type`]) and [`serialize_triples_negotiated`] emits the
//! requested document form — `expanded` byte-identically (the serialiser's default output IS the
//! expanded form), `compacted` via a local context-free compaction step that keeps the SSRF
//! posture (nothing is ever fetched).
//!
//! The three line/N3 READ formats land here too (gh-4879): `application/n-triples`,
//! `application/n-quads` and `text/n3` are negotiable on the read path and produced by
//! [`serialize_triples`], so an `Accept` naming one is SERVED rather than degrading to
//! `text/turtle`. They are **read-only** — [`classify`] still accepts only Turtle + JSON-LD on
//! write, so they are never a resource's stored format. Nothing is fetched to produce them (they
//! are rendered from the already-parsed triple set), so the SSRF posture is unchanged.

use oxjsonld::{JsonLdParser, JsonLdSerializer};
use oxrdf::{GraphNameRef, QuadRef, Triple};
use oxttl::{NQuadsSerializer, NTriplesSerializer, TurtleParser, TurtleSerializer};

use crate::error::ServerError;

/// A supported RDF media type.
///
/// [`Turtle`](Self::Turtle) and [`JsonLd`](Self::JsonLd) are the READ+WRITE formats: a resource can
/// be stored in either ([`classify`]) and served in either. The remaining three are **read-only** —
/// the read path can produce them ([`serialize_triples`], [`negotiate_accept_with_profile`]) but
/// [`classify`] never returns one, so they are never a stored format and never a parse input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdfFormat {
    /// `text/turtle` — the Solid default representation (read + write).
    Turtle,
    /// `application/ld+json` (read + write).
    JsonLd,
    /// `application/n-triples` (read-only).
    NTriples,
    /// `application/n-quads` (read-only) — for a single-graph resource the same document as
    /// [`NTriples`](Self::NTriples); see [`serialize_triples`].
    NQuads,
    /// `text/n3` (read-only) — Notation3, served as the Turtle syntax it subsumes; see
    /// [`serialize_triples`].
    N3,
}

impl RdfFormat {
    /// The canonical media type string.
    pub fn media_type(self) -> &'static str {
        match self {
            RdfFormat::Turtle => "text/turtle",
            RdfFormat::JsonLd => "application/ld+json",
            RdfFormat::NTriples => "application/n-triples",
            RdfFormat::NQuads => "application/n-quads",
            RdfFormat::N3 => "text/n3",
        }
    }
}

/// Every format the read path can produce, in the order a q-value tie between two of them is
/// broken. The resource's STORED format outranks all of these (cheapest, most faithful) and is
/// applied separately by [`negotiate_accept_with_profile`]; this order then decides among the rest,
/// keeping the two read+write formats ahead of the read-only ones.
const PRODUCIBLE: [RdfFormat; 5] = [
    RdfFormat::Turtle,
    RdfFormat::JsonLd,
    RdfFormat::NTriples,
    RdfFormat::NQuads,
    RdfFormat::N3,
];

/// Classify a `Content-Type` header value into a supported [`RdfFormat`].
///
/// The media-type is matched case-insensitively and any parameters (e.g. `; charset=utf-8`) are
/// ignored. An unsupported or absent type is an [`ServerError::UnsupportedMediaType`] — the LDP
/// surface accepts only RDF on this slice's single-resource PUT path.
///
/// This is the WRITE side, and it is deliberately narrower than the read side: only the two
/// read+write formats classify as RDF. The read-only formats ([`RdfFormat::NTriples`],
/// [`RdfFormat::NQuads`], [`RdfFormat::N3`]) are producible on GET but are not RDF *inputs* — a
/// body declared as one is not parse-validated, and the LDP write path (`validate_writable`) keeps
/// it verbatim as an opaque binary resource, exactly as it does any other non-RDF type. So a
/// resource's stored RDF format is always Turtle or JSON-LD.
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
///
/// Only the two INPUT formats parse. The read-only formats ([`RdfFormat::NTriples`],
/// [`RdfFormat::NQuads`], [`RdfFormat::N3`]) are an [`ServerError::UnsupportedMediaType`]: they are
/// produced by the read path but never accepted as a body, so the parse surface is kept equal to
/// [`classify`]'s accepted set rather than growing an input path no request can reach.
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
        // Read-only formats: producible, never accepted as input (see the doc above).
        RdfFormat::NTriples | RdfFormat::NQuads | RdfFormat::N3 => Err(
            ServerError::UnsupportedMediaType(format.media_type().to_string()),
        ),
    }
}

/// Serialise a triple set into `format`, returning the bytes.
///
/// The serialisation is unconditioned (no base-IRI abbreviation) so the output is self-contained and
/// stable. Used by content negotiation on read (re-render the stored Turtle as JSON-LD or vice
/// versa) and after a PATCH (re-serialise the patched graph for storage).
///
/// The three read-only formats are rendered from the same triple set, without any new dependency:
///
/// - **N-Triples and N-Quads each use their own `oxttl` serialiser**, the resource's triples being
///   emitted as default-graph statements. Their outputs coincide byte for byte here (a
///   default-graph statement carries no graph label), which the tests pin as an observed property —
///   no code depends on it.
/// - **N3 is emitted as Turtle.** Notation3 subsumes the Turtle grammar, so a Turtle document is
///   already a valid N3 document, and no N3-only construct (formula, variable, rule) can arise from
///   a triple set. `oxttl` ships an N3 *parser* only — there is no N3 serialiser to call — which is
///   consistent with N3 being read-only here. One divergence to be honest about: the workspace
///   enables `oxttl`'s `rdf-12`, so a triple whose object is an RDF 1.2 triple term serialises as
///   `<<( … )>>` — syntax N3 does not define.
pub fn serialize_triples(format: RdfFormat, triples: &[Triple]) -> Result<Vec<u8>, ServerError> {
    match format {
        // N3 is served as the Turtle syntax it subsumes (see the doc above).
        RdfFormat::Turtle | RdfFormat::N3 => serialize_turtle(triples),
        RdfFormat::NTriples => serialize_ntriples(triples),
        RdfFormat::NQuads => serialize_nquads(triples),
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

/// Serialise a triple set as Turtle — also the document served for [`RdfFormat::N3`], whose grammar
/// subsumes Turtle's (see [`serialize_triples`]).
fn serialize_turtle(triples: &[Triple]) -> Result<Vec<u8>, ServerError> {
    let mut ser = TurtleSerializer::new().for_writer(Vec::new());
    for t in triples {
        ser.serialize_triple(t)
            .map_err(|e| ServerError::Storage(format!("turtle serialise: {}", e)))?;
    }
    ser.finish()
        .map_err(|e| ServerError::Storage(format!("turtle serialise: {}", e)))
}

/// Serialise a triple set as N-Triples (see [`serialize_triples`]).
///
/// The low-level serialiser is stateless (one canonical line per triple, no document prologue or
/// terminator), so a single instance writes the whole set into the output buffer.
fn serialize_ntriples(triples: &[Triple]) -> Result<Vec<u8>, ServerError> {
    let mut ser = NTriplesSerializer::new().low_level();
    let mut out = Vec::new();
    for t in triples {
        ser.serialize_triple(t.as_ref(), &mut out)
            .map_err(|e| ServerError::Storage(format!("n-triples serialise: {}", e)))?;
    }
    Ok(out)
}

/// Serialise a triple set as N-Quads (see [`serialize_triples`]).
///
/// Each triple is emitted as a DEFAULT-GRAPH statement — a Solid RDF resource is a single graph
/// (see [`parse_to_triples`]), and there is no named graph to label it with.
fn serialize_nquads(triples: &[Triple]) -> Result<Vec<u8>, ServerError> {
    let mut ser = NQuadsSerializer::new().for_writer(Vec::new());
    for t in triples {
        let q = QuadRef::new(
            t.subject.as_ref(),
            t.predicate.as_ref(),
            t.object.as_ref(),
            GraphNameRef::DefaultGraph,
        );
        ser.serialize_quad(q)
            .map_err(|e| ServerError::Storage(format!("n-quads serialise: {}", e)))?;
    }
    Ok(ser.finish())
}

/// Serialise a triple set into the negotiated format AND document form, returning the bytes.
///
/// Same as [`serialize_triples`] for every format except JSON-LD with an honoured profile — the
/// serialiser's default output (a top-level array of node objects, full IRIs, no `@context`) is
/// ALREADY the [expanded document form](https://www.w3.org/TR/json-ld11/#expanded-document-form),
/// so the `expanded` profile is honoured byte-identically. The `compacted` profile applies
/// `compact_jsonld`, the local (context-free) compaction step — no context is used or fetched,
/// preserving the local-only no-remote-context SSRF posture. The `profile` parameter is
/// JSON-LD-only, so the Turtle / N-Triples / N-Quads / N3 outputs are the plain serialisations.
pub fn serialize_triples_negotiated(
    negotiated: NegotiatedFormat,
    triples: &[Triple],
) -> Result<Vec<u8>, ServerError> {
    let bytes = serialize_triples(negotiated.format, triples)?;
    match (negotiated.format, negotiated.jsonld_profile) {
        (RdfFormat::JsonLd, Some(JsonLdProfileParam::Compacted)) => compact_jsonld(&bytes),
        _ => Ok(bytes),
    }
}

/// Compact the serialiser's own expanded JSON-LD output with an EMPTY context — the JSON-LD 1.1
/// compaction of the document against no context, scoped to the shape [`serialize_triples`]
/// produces (a top-level array of flat node objects whose non-`@id` values are arrays of node
/// references / value objects; no `@list`, no nesting, no named graphs — the serialiser emits
/// none of those for a default-graph triple set):
///
/// - a single-element property array collapses to its (compacted) single value;
/// - a value object holding ONLY `@value` collapses to the bare scalar (there is no default
///   language, so a plain string never needs its object kept); one carrying `@type` / `@language`
///   (/ `@direction`) keeps its object — those entries must survive;
/// - a node reference (`{"@id": …}`) stays an object (an empty context maps no term to `@id`);
/// - a single top-level node object becomes the document itself, an empty document becomes `{}`,
///   and multiple node objects wrap in `{"@graph": […]}` — the `JsonLdProcessor.compact` API's
///   top-level rule.
///
/// Purely local structural rewriting over our own serialiser's bytes: no context document exists,
/// so nothing can be fetched (the SSRF posture is preserved by construction). Key order follows
/// `serde_json`'s deterministic map, so equal inputs give equal bytes (the ETag contract).
fn compact_jsonld(bytes: &[u8]) -> Result<Vec<u8>, ServerError> {
    let doc: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| ServerError::Storage(format!("json-ld compaction parse: {e}")))?;
    // The serialiser's context-free output is always a top-level array; anything else means the
    // input was not ours — fail loudly rather than mislabel a non-compacted body.
    let serde_json::Value::Array(nodes) = doc else {
        return Err(ServerError::Storage(
            "json-ld compaction: unexpected serialiser output shape".into(),
        ));
    };
    let mut compacted: Vec<serde_json::Value> = nodes.into_iter().map(compact_node).collect();
    let out = match compacted.len() {
        0 => serde_json::Value::Object(serde_json::Map::new()),
        1 => compacted.pop().unwrap_or_default(),
        _ => {
            let mut map = serde_json::Map::with_capacity(1);
            map.insert("@graph".into(), serde_json::Value::Array(compacted));
            serde_json::Value::Object(map)
        }
    };
    serde_json::to_vec(&out)
        .map_err(|e| ServerError::Storage(format!("json-ld compaction serialise: {e}")))
}

/// Compact one node object: every entry except `@id` (whose value is a plain string) is a
/// property whose array value compacts per the rules on [`compact_jsonld`].
fn compact_node(node: serde_json::Value) -> serde_json::Value {
    let serde_json::Value::Object(map) = node else {
        // Not a node object — never produced by our serialiser; pass through unchanged.
        return node;
    };
    let compacted = map
        .into_iter()
        .map(|(key, value)| {
            let value = if key == "@id" {
                value
            } else {
                compact_property_values(value)
            };
            (key, value)
        })
        .collect();
    serde_json::Value::Object(compacted)
}

/// Compact a property's expanded value array: each member value-compacts (a lone-`@value` object
/// to its bare scalar), and a single-member array collapses to that member (`compactArrays`).
fn compact_property_values(value: serde_json::Value) -> serde_json::Value {
    let serde_json::Value::Array(members) = value else {
        return compact_value(value);
    };
    let mut members: Vec<serde_json::Value> = members.into_iter().map(compact_value).collect();
    if members.len() == 1 {
        members.pop().unwrap_or_default()
    } else {
        serde_json::Value::Array(members)
    }
}

/// Value-compact one expanded object: a value object holding ONLY `@value` becomes the bare
/// scalar; everything else (a `@type`/`@language`-carrying value object, a node reference) is
/// kept as-is.
fn compact_value(value: serde_json::Value) -> serde_json::Value {
    let serde_json::Value::Object(map) = value else {
        return value;
    };
    if map.len() == 1 && map.contains_key("@value") {
        return map
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap_or_default();
    }
    serde_json::Value::Object(map)
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
    /// The canonical profile IRI (the JSON-LD 1.1 IANA registration), via
    /// [`oxjsonld::JsonLdProfile::iri`] so the spelling stays pinned to the vendored JSON-LD
    /// implementation. Used to echo the honoured profile back in the response `Content-Type`.
    pub fn iri(self) -> &'static str {
        match self {
            Self::Expanded => oxjsonld::JsonLdProfile::Expanded.iri(),
            Self::Compacted => oxjsonld::JsonLdProfile::Compacted.iri(),
        }
    }

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
    /// wildcard ranges carry no honoured profile). An honoured profile is echoed back in the
    /// response `Content-Type` ([`Self::content_type`]) and drives the document form
    /// ([`serialize_triples_negotiated`]): the serialiser's default output IS the expanded form,
    /// and `compacted` applies the local (context-free, nothing-fetched) compaction step.
    pub jsonld_profile: Option<JsonLdProfileParam>,
}

impl NegotiatedFormat {
    /// The response `Content-Type` value: the negotiated media type, with the honoured JSON-LD
    /// `profile` parameter echoed back (quoted, the canonical IRI) per the JSON-LD 1.1 IANA
    /// registration — so a client can see which document form the server actually honoured.
    pub fn content_type(&self) -> String {
        match self.jsonld_profile {
            Some(profile) => format!(
                "{};profile=\"{}\"",
                self.format.media_type(),
                profile.iri()
            ),
            None => self.format.media_type().to_string(),
        }
    }

    /// Whether the stored bytes of a resource stored in `stored` format ARE this negotiation's
    /// representation (the verbatim fast path): same format AND no honoured JSON-LD profile. Any
    /// honoured profile forces a re-serialisation even for a stored-JSON-LD resource — the stored
    /// bytes are whatever document form the client originally wrote, so serving them under a
    /// `profile`-carrying `Content-Type` would be dishonest. The single shared rule keeps the body
    /// path ([`serialize_triples_negotiated`] callers) and the validator path in agreement.
    pub fn serves_stored_verbatim(&self, stored: RdfFormat) -> bool {
        self.format == stored && self.jsonld_profile.is_none()
    }

    /// The short, `+`-free `+<variant>` ETag suffix token for this negotiated representation
    /// (consumed by `conditional::variant_etag`; shared by the LDP and identity read paths so a
    /// variant tag is derived identically on both surfaces).
    ///
    /// Every MEDIA TYPE gets its own token, because a representation is bytes *plus* their
    /// `Content-Type` (RFC 9110 §8.8.3): N-Quads and N-Triples (like N3 and Turtle) serialise to
    /// the same bytes, but a shared token would let a client holding one 304 the other and label
    /// those bytes with the wrong type. WITHIN a media type the token splits only when the bytes
    /// differ: `compacted` output differs from the default JSON-LD serialisation ⇒ a distinct
    /// token, while `expanded` output is byte-identical to it (the serialiser's default output IS
    /// the expanded document form — [`serialize_triples_negotiated`]) ⇒ the same token as plain
    /// JSON-LD.
    pub fn variant_suffix(&self) -> &'static str {
        match (self.format, self.jsonld_profile) {
            (RdfFormat::Turtle, _) => "ttl",
            (RdfFormat::JsonLd, Some(JsonLdProfileParam::Compacted)) => "jsonld-c",
            (RdfFormat::JsonLd, _) => "jsonld",
            (RdfFormat::NTriples, _) => "nt",
            (RdfFormat::NQuads, _) => "nq",
            (RdfFormat::N3, _) => "n3",
        }
    }
}

/// Negotiate the response RDF format from an `Accept` header against the formats this server can
/// produce. Thin wrapper over [`negotiate_accept_with_profile`] (the single
/// `Accept` decision point) that drops the JSON-LD profile detail.
pub fn negotiate_accept(accept: Option<&str>, stored: RdfFormat) -> Option<RdfFormat> {
    negotiate_accept_with_profile(accept, stored).map(|n| n.format)
}

/// Negotiate the response RDF format (and JSON-LD document form) from an `Accept` header against
/// the formats this server can produce: the read+write pair (Turtle + JSON-LD) plus the read-only
/// N-Triples / N-Quads / N3 ([`RdfFormat`]).
///
/// - An ABSENT `Accept` means "no preference": the resource's stored format (`stored`) — the most
///   faithful, zero-cost response (as does `*/*`, which covers every producible type).
/// - Quality values (`q=`) are honoured: the highest-q acceptable type wins; a q-tie prefers the
///   stored format (cheapest), then the fixed producible order (Turtle, JSON-LD, N-Triples,
///   N-Quads, N3), so the two read+write formats keep winning the ties they won before the
///   read-only formats existed.
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
/// This is a deliberately small, dependency-free `Accept` parser sufficient for the RDF media
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
    // range covers only the `text/…` producible types (`text/turtle`, `text/n3`); an
    // `application/*` range only the `application/…` ones (`application/ld+json`,
    // `application/n-triples`, `application/n-quads`); `*/*` covers all five. Each is kept
    // separately so a `text/*` request never yields an `application/…` type (the bug roborev
    // flagged).
    let mut q_turtle: Option<f32> = None;
    let mut q_jsonld: Option<f32> = None;
    let mut q_ntriples: Option<f32> = None;
    let mut q_nquads: Option<f32> = None;
    let mut q_n3: Option<f32> = None;
    let mut q_text_star: Option<f32> = None; // covers Turtle + N3
    let mut q_app_star: Option<f32> = None; // covers JSON-LD + N-Triples + N-Quads
    let mut q_any: Option<f32> = None; // covers every producible type

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
            "application/n-triples" => bump(&mut q_ntriples, q),
            "application/n-quads" => bump(&mut q_nquads, q),
            "text/n3" => bump(&mut q_n3, q),
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
        && q_ntriples.is_none()
        && q_nquads.is_none()
        && q_n3.is_none()
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
    let weight = |format: RdfFormat| -> f32 {
        match format {
            RdfFormat::Turtle => q_turtle.or(q_text_star).or(q_any),
            RdfFormat::N3 => q_n3.or(q_text_star).or(q_any),
            RdfFormat::JsonLd => q_jsonld.or(q_app_star).or(q_any),
            RdfFormat::NTriples => q_ntriples.or(q_app_star).or(q_any),
            RdfFormat::NQuads => q_nquads.or(q_app_star).or(q_any),
        }
        .unwrap_or(0.0)
    };

    // Highest q wins. The stored format is the incumbent (cheapest, most faithful) and only a
    // STRICTLY greater weight displaces it, so it takes every tie; among the rest the scan order —
    // the fixed producible order — breaks ties, keeping Turtle/JSON-LD ahead of the read-only
    // formats a wildcard range now also covers.
    let mut format = stored;
    let mut best = weight(stored);
    for candidate in PRODUCIBLE {
        let q = weight(candidate);
        if q > best {
            best = q;
            format = candidate;
        }
    }

    if best <= 0.0 {
        return None; // 406 — the client EXPLICITLY refused (q=0) every covered producible type.
    }
    Some(NegotiatedFormat {
        format,
        // The profile is only meaningful for a JSON-LD response, and only when JSON-LD won via an
        // explicit `application/ld+json` range (a wildcard win carries no honoured profile).
        jsonld_profile: match format {
            RdfFormat::JsonLd => jsonld_profile,
            _ => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const IRI: &str = "https://pod.example/alice/data";
    const TURTLE: &str =
        "<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";
    /// Two triples sharing a subject — the shape whose Turtle serialisation (subject grouped with
    /// `;`) DIFFERS from its N-Triples serialisation (subject repeated per line).
    const TWO_TRIPLES: &str = "<https://pod.example/alice/data#me> \
        <http://xmlns.com/foaf/0.1/name> \"Alice\" ; \
        <http://xmlns.com/foaf/0.1/nick> \"Al\" .";

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

    /// The negotiated outcome for an explicit `application/ld+json` range carrying `profile`.
    fn jsonld_with_profile(profile_iri: &str) -> NegotiatedFormat {
        negotiate_accept_with_profile(
            Some(&format!("application/ld+json;profile=\"{profile_iri}\"")),
            RdfFormat::Turtle,
        )
        .expect("acceptable")
    }

    #[test]
    fn profile_iris_come_from_the_vendored_implementation() {
        assert_eq!(
            JsonLdProfileParam::Expanded.iri(),
            "http://www.w3.org/ns/json-ld#expanded"
        );
        assert_eq!(
            JsonLdProfileParam::Compacted.iri(),
            "http://www.w3.org/ns/json-ld#compacted"
        );
    }

    #[test]
    fn serves_stored_verbatim_only_without_an_honoured_profile() {
        let plain = negotiate_accept_with_profile(Some("application/ld+json"), RdfFormat::JsonLd)
            .expect("acceptable");
        assert!(plain.serves_stored_verbatim(RdfFormat::JsonLd));
        assert!(!plain.serves_stored_verbatim(RdfFormat::Turtle));
        // An honoured profile always re-serialises, even from a stored-JSON-LD resource.
        let compacted = jsonld_with_profile("http://www.w3.org/ns/json-ld#compacted");
        assert!(!compacted.serves_stored_verbatim(RdfFormat::JsonLd));
        let expanded = jsonld_with_profile("http://www.w3.org/ns/json-ld#expanded");
        assert!(!expanded.serves_stored_verbatim(RdfFormat::JsonLd));
    }

    #[test]
    fn compacting_an_empty_document_yields_the_empty_object() {
        let compacted = jsonld_with_profile("http://www.w3.org/ns/json-ld#compacted");
        let bytes = serialize_triples_negotiated(compacted, &[]).unwrap();
        assert_eq!(bytes, b"{}");
    }

    #[test]
    fn compaction_rejects_a_non_array_document_shape() {
        // Defensive: the serialiser's context-free output is always a top-level array; anything
        // else must fail loudly rather than be mislabelled as compacted.
        assert!(compact_jsonld(b"{\"@graph\":[]}").is_err());
        assert!(compact_jsonld(b"not json").is_err());
    }

    #[test]
    fn serialise_to_turtle_is_reparseable() {
        let triples = parse_to_triples(RdfFormat::Turtle, TURTLE.as_bytes(), IRI).unwrap();
        let ttl = serialize_triples(RdfFormat::Turtle, &triples).unwrap();
        let reparsed = parse_to_triples(RdfFormat::Turtle, &ttl, IRI).unwrap();
        assert_eq!(reparsed, triples);
    }

    // --- the read-only line/N3 formats (gh-4879) ---------------------------------------------

    #[test]
    fn read_only_formats_carry_their_canonical_media_types() {
        assert_eq!(RdfFormat::NTriples.media_type(), "application/n-triples");
        assert_eq!(RdfFormat::NQuads.media_type(), "application/n-quads");
        assert_eq!(RdfFormat::N3.media_type(), "text/n3");
    }

    #[test]
    fn an_accept_naming_a_read_format_is_served_not_degraded_to_turtle() {
        // The gh-4879 behaviour: these used to be unrecognised, so the read degraded to Turtle.
        assert_eq!(
            negotiate_accept(Some("application/n-triples"), RdfFormat::Turtle),
            Some(RdfFormat::NTriples)
        );
        assert_eq!(
            negotiate_accept(Some("application/n-quads"), RdfFormat::Turtle),
            Some(RdfFormat::NQuads)
        );
        assert_eq!(
            negotiate_accept(Some("text/n3"), RdfFormat::JsonLd),
            Some(RdfFormat::N3)
        );
        // …and they lose to a higher-q producible type, like any other range.
        assert_eq!(
            negotiate_accept(
                Some("application/n-triples;q=0.4, text/turtle;q=0.8"),
                RdfFormat::JsonLd
            ),
            Some(RdfFormat::Turtle)
        );
    }

    #[test]
    fn wildcards_keep_preferring_the_read_write_formats() {
        // The read-only formats now fall under the wildcards too, but the fixed producible order
        // keeps the pre-existing winners: `application/*` ⇒ JSON-LD (not N-Triples/N-Quads),
        // `text/*` ⇒ Turtle (not N3), `*/*` ⇒ the stored format.
        assert_eq!(
            negotiate_accept(Some("application/*"), RdfFormat::Turtle),
            Some(RdfFormat::JsonLd)
        );
        assert_eq!(
            negotiate_accept(Some("text/*"), RdfFormat::JsonLd),
            Some(RdfFormat::Turtle)
        );
        assert_eq!(
            negotiate_accept(Some("*/*"), RdfFormat::JsonLd),
            Some(RdfFormat::JsonLd)
        );
        // A `text/*` range still never yields an `application/…` type, and vice versa.
        assert_eq!(
            negotiate_accept(Some("text/*;q=0.9, application/n-quads;q=1"), RdfFormat::Turtle),
            Some(RdfFormat::NQuads)
        );
    }

    #[test]
    fn read_only_formats_are_not_accepted_as_input() {
        // The WRITE surface is unchanged: none of them classifies as an RDF input (so the write
        // path keeps treating such a body as an opaque binary, never as RDF)…
        assert!(classify(Some("application/n-triples")).is_err());
        assert!(classify(Some("application/n-quads")).is_err());
        assert!(classify(Some("text/n3")).is_err());
        // …and the parse surface matches the accepted set, so they never become a stored format.
        for format in [RdfFormat::NTriples, RdfFormat::NQuads, RdfFormat::N3] {
            assert!(matches!(
                parse_to_triples(format, TURTLE.as_bytes(), IRI),
                Err(ServerError::UnsupportedMediaType(_))
            ));
        }
    }

    #[test]
    fn ntriples_output_is_line_based_and_is_the_nquads_document() {
        // TWO triples on ONE subject: Turtle groups them under a single subject with `;`, whereas
        // N-Triples repeats the subject on its own line — so this fixture (unlike a lone triple,
        // whose two serialisations coincide) actually pins the line format.
        let triples = parse_to_triples(RdfFormat::Turtle, TWO_TRIPLES.as_bytes(), IRI).unwrap();
        let nt = serialize_triples(RdfFormat::NTriples, &triples).unwrap();
        assert_eq!(
            String::from_utf8(nt.clone()).unwrap(),
            "<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .\n\
             <https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/nick> \"Al\" .\n"
        );
        assert_ne!(
            nt,
            serialize_triples(RdfFormat::Turtle, &triples).unwrap(),
            "the line format must not be the Turtle serialisation"
        );
        // The N-Quads serialiser emits the same bytes for this set: a default-graph statement
        // carries no graph label. Pinned as an observed property — nothing in the code assumes it.
        let nq = serialize_triples(RdfFormat::NQuads, &triples).unwrap();
        assert_eq!(nq, nt);
        // The lines really are the triple set: one per triple, and they reparse as Turtle (which
        // subsumes the N-Triples grammar) back to the same triples.
        assert_eq!(nt.iter().filter(|b| **b == b'\n').count(), triples.len());
        assert_eq!(parse_to_triples(RdfFormat::Turtle, &nt, IRI).unwrap(), triples);
    }

    #[test]
    fn n3_is_served_as_the_turtle_syntax_it_subsumes() {
        // Again the two-triple fixture, so "N3 is Turtle" is distinguishable from "N3 is the line
        // format" — the two coincide for a single triple.
        let triples = parse_to_triples(RdfFormat::Turtle, TWO_TRIPLES.as_bytes(), IRI).unwrap();
        let n3 = serialize_triples(RdfFormat::N3, &triples).unwrap();
        assert_eq!(n3, serialize_triples(RdfFormat::Turtle, &triples).unwrap());
        assert_ne!(n3, serialize_triples(RdfFormat::NTriples, &triples).unwrap());
    }

    #[test]
    fn every_media_type_gets_its_own_variant_suffix() {
        // Distinct media types ⇒ distinct ETag variant tokens even where the bytes coincide
        // (N-Triples/N-Quads, Turtle/N3), so a 304 can never mislabel a cached representation.
        let suffix = |format| {
            NegotiatedFormat {
                format,
                jsonld_profile: None,
            }
            .variant_suffix()
        };
        let tokens: Vec<&str> = PRODUCIBLE.iter().map(|f| suffix(*f)).collect();
        assert_eq!(tokens, vec!["ttl", "jsonld", "nt", "nq", "n3"]);
        let mut sorted = tokens.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), tokens.len(), "variant tokens must be unique");
    }

    #[test]
    fn a_read_only_format_never_serves_stored_bytes_verbatim() {
        // The stored format is always Turtle or JSON-LD, so a read-only negotiation always
        // re-serialises — and its Content-Type is the bare media type (profile is JSON-LD-only).
        for format in [RdfFormat::NTriples, RdfFormat::NQuads, RdfFormat::N3] {
            let negotiated = NegotiatedFormat {
                format,
                jsonld_profile: None,
            };
            assert!(!negotiated.serves_stored_verbatim(RdfFormat::Turtle));
            assert!(!negotiated.serves_stored_verbatim(RdfFormat::JsonLd));
            assert_eq!(negotiated.content_type(), format.media_type());
        }
    }
}
