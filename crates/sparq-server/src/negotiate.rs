//! `Accept`-header content negotiation for SPARQL results.
//!
//! Maps an HTTP `Accept` header to the result format the server will produce. We support
//! the four W3C result serialisations for SELECT (JSON/XML/CSV/TSV) and the boolean
//! serialisations for ASK (JSON/XML). The parsing is a pragmatic q-value-aware scan: it
//! collects acceptable media ranges with their q-values and picks the highest-q supported
//! format, defaulting to JSON (the engine's native, fastest path) when nothing matches or
//! the header is absent.

/// The negotiated SELECT/ASK result format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Xml,
    Csv,
    Tsv,
}

impl Format {
    /// The `Content-Type` for a SELECT result in this format (charset annotated where the
    /// W3C registration specifies one).
    pub fn select_content_type(self) -> &'static str {
        match self {
            Format::Json => "application/sparql-results+json",
            Format::Xml => "application/sparql-results+xml",
            Format::Csv => "text/csv; charset=utf-8",
            Format::Tsv => "text/tab-separated-values; charset=utf-8",
        }
    }

    /// The `Content-Type` for an ASK boolean. CSV/TSV have no boolean form, so they fall
    /// back to JSON per common practice.
    pub fn ask_content_type(self) -> &'static str {
        match self {
            Format::Xml => "application/sparql-results+xml",
            _ => "application/sparql-results+json",
        }
    }
}

/// The negotiated RDF graph serialisation for CONSTRUCT / DESCRIBE + Graph Store reads.
/// `Turtle` is now a real prefix-compacting Turtle document, and `RdfXml` is a genuine
/// RDF/XML document ([OPUS-4.8] sq-rt6v) — distinct serialisations, not all-N-Triples.
/// Negotiation picks the format (and its `Content-Type`) from the `Accept` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFormat {
    NTriples,
    Turtle,
    /// `application/rdf+xml` — RDF/XML. [OPUS-4.8] sq-rt6v.
    RdfXml,
    /// `application/ld+json` — JSON-LD 1.1 (the engine's flattened serialisation).
    /// [OPUS-4.8] sq-oy1f.1. OPT-IN: only a participant in negotiation when the server is
    /// built with the `jsonld` feature (which links the engine's `serialize-rdf` writer);
    /// without it the variant does not exist, so `application/ld+json` is unrecognised and
    /// negotiation never selects it (it falls through to the default, exactly like any other
    /// unsupported media type).
    #[cfg(feature = "jsonld")]
    JsonLd,
}

impl GraphFormat {
    pub fn content_type(self) -> &'static str {
        match self {
            GraphFormat::NTriples => "application/n-triples; charset=utf-8",
            GraphFormat::Turtle => "text/turtle; charset=utf-8",
            // [OPUS-4.8] sq-rt6v: RDF/XML registers `charset=utf-8` like the others; the
            // document's own XML declaration also pins UTF-8.
            GraphFormat::RdfXml => "application/rdf+xml; charset=utf-8",
            // [OPUS-4.8] sq-oy1f.1: JSON-LD is UTF-8 JSON; the IANA `application/ld+json`
            // registration carries no charset parameter (JSON is UTF-8 by RFC 8259), but we
            // annotate it like the others so a strict client never mis-decodes.
            #[cfg(feature = "jsonld")]
            GraphFormat::JsonLd => "application/ld+json; charset=utf-8",
        }
    }
}

/// Negotiates the CONSTRUCT/DESCRIBE + GSP-read graph serialisation from an `Accept` header:
/// `text/turtle`, `application/n-triples` and — [OPUS-4.8] sq-rt6v — `application/rdf+xml`
/// (q-value aware), default N-Triples.
pub fn negotiate_graph(accept: Option<&str>) -> GraphFormat {
    let accept = match accept {
        Some(a) if !a.trim().is_empty() => a,
        _ => return GraphFormat::NTriples,
    };
    let mut best: Option<(GraphFormat, f32, usize)> = None;
    for (media, q) in media_ranges(accept) {
        let (fmt, spec) = match media.as_str() {
            "application/n-triples" => (Some(GraphFormat::NTriples), 2),
            "text/turtle" => (Some(GraphFormat::Turtle), 2),
            // [OPUS-4.8] sq-rt6v: RDF/XML (and the older `application/xml`/`text/xml` aliases
            // some clients send for RDF/XML — kept lower-specificity so an exact match wins).
            "application/rdf+xml" => (Some(GraphFormat::RdfXml), 2),
            "application/xml" | "text/xml" => (Some(GraphFormat::RdfXml), 1),
            // [OPUS-4.8] sq-oy1f.1: JSON-LD, OPT-IN behind the `jsonld` feature. When the
            // feature is off this arm is compiled out, so `application/ld+json` matches no arm
            // and falls through to the default (N-Triples) like any unsupported type.
            #[cfg(feature = "jsonld")]
            "application/ld+json" => (Some(GraphFormat::JsonLd), 2),
            "*/*" => (Some(GraphFormat::NTriples), 0),
            _ => (None, 0),
        };
        if let Some(fmt) = fmt {
            if q <= 0.0 {
                continue; // q=0 explicitly rejects
            }
            let better = match best {
                None => true,
                Some((_, bq, bspec)) => q > bq || (q == bq && spec > bspec),
            };
            if better {
                best = Some((fmt, q, spec));
            }
        }
    }
    best.map(|(f, _, _)| f).unwrap_or(GraphFormat::NTriples)
}

/// Splits an `Accept` header into (lowercased media range, q-value) pairs.
fn media_ranges(accept: &str) -> impl Iterator<Item = (String, f32)> + '_ {
    accept.split(',').map(|part| {
        let mut it = part.split(';');
        let media = it.next().unwrap_or("").trim().to_ascii_lowercase();
        let mut q = 1.0f32;
        for param in it {
            if let Some(v) = param.trim().strip_prefix("q=") {
                q = v.parse().unwrap_or(1.0);
            }
        }
        (media, q)
    })
}

/// Negotiates the result format from an optional `Accept` header value.
pub fn negotiate(accept: Option<&str>) -> Format {
    let accept = match accept {
        Some(a) if !a.trim().is_empty() => a,
        _ => return Format::Json,
    };

    let mut best: Option<(Format, f32, usize)> = None; // (format, q, specificity)
    for (media, q) in media_ranges(accept) {
        // specificity: an exact supported type beats a wildcard at equal q; the first
        // listed wins remaining ties (the `better` test below keeps the earlier match).
        let (fmt, spec) = match media.as_str() {
            "application/sparql-results+json" | "application/json" => (Some(Format::Json), 2),
            "application/sparql-results+xml" | "application/xml" | "text/xml" => (Some(Format::Xml), 2),
            "text/csv" => (Some(Format::Csv), 2),
            "text/tab-separated-values" => (Some(Format::Tsv), 2),
            "*/*" => (Some(Format::Json), 0),
            _ => (None, 0),
        };
        if let Some(fmt) = fmt {
            if q <= 0.0 {
                continue; // q=0 explicitly rejects
            }
            let better = match best {
                None => true,
                Some((_, bq, bspec)) => q > bq || (q == bq && spec > bspec),
            };
            if better {
                best = Some((fmt, q, spec));
            }
        }
    }
    best.map(|(f, _, _)| f).unwrap_or(Format::Json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_json() {
        assert_eq!(negotiate(None), Format::Json);
        assert_eq!(negotiate(Some("")), Format::Json);
        assert_eq!(negotiate(Some("*/*")), Format::Json);
    }

    #[test]
    fn exact_types() {
        assert_eq!(negotiate(Some("application/sparql-results+json")), Format::Json);
        assert_eq!(negotiate(Some("application/sparql-results+xml")), Format::Xml);
        assert_eq!(negotiate(Some("text/csv")), Format::Csv);
        assert_eq!(negotiate(Some("text/tab-separated-values")), Format::Tsv);
    }

    #[test]
    fn q_values_pick_highest() {
        assert_eq!(
            negotiate(Some("application/sparql-results+json;q=0.5, text/csv;q=0.9")),
            Format::Csv
        );
        // q=0 rejects json, so xml wins
        assert_eq!(
            negotiate(Some("application/sparql-results+json;q=0, application/sparql-results+xml")),
            Format::Xml
        );
    }

    #[test]
    fn unsupported_falls_back_to_json() {
        assert_eq!(negotiate(Some("application/pdf")), Format::Json);
    }

    #[test]
    fn exact_beats_wildcard() {
        assert_eq!(negotiate(Some("text/csv, */*")), Format::Csv);
    }

    #[test]
    fn graph_formats() {
        assert_eq!(negotiate_graph(None), GraphFormat::NTriples);
        assert_eq!(negotiate_graph(Some("")), GraphFormat::NTriples);
        assert_eq!(negotiate_graph(Some("*/*")), GraphFormat::NTriples);
        assert_eq!(negotiate_graph(Some("text/turtle")), GraphFormat::Turtle);
        assert_eq!(negotiate_graph(Some("application/n-triples")), GraphFormat::NTriples);
        // q-values decide; q=0 rejects.
        assert_eq!(
            negotiate_graph(Some("application/n-triples;q=0.5, text/turtle;q=0.9")),
            GraphFormat::Turtle
        );
        assert_eq!(
            negotiate_graph(Some("application/n-triples;q=0, text/turtle")),
            GraphFormat::Turtle
        );
        // exact beats wildcard; unsupported falls back to N-Triples.
        assert_eq!(negotiate_graph(Some("text/turtle, */*")), GraphFormat::Turtle);
        // [OPUS-4.8] sq-rt6v: RDF/XML is now a first-class graph format.
        assert_eq!(negotiate_graph(Some("application/rdf+xml")), GraphFormat::RdfXml);
        // The `application/xml`/`text/xml` aliases also map to RDF/XML, but at lower
        // specificity, so an exact `application/rdf+xml` (or turtle/n-triples) wins a tie.
        assert_eq!(negotiate_graph(Some("application/xml")), GraphFormat::RdfXml);
        assert_eq!(negotiate_graph(Some("text/xml")), GraphFormat::RdfXml);
        assert_eq!(negotiate_graph(Some("text/turtle, application/xml")), GraphFormat::Turtle);
        // q=0 rejects RDF/XML, so Turtle wins.
        assert_eq!(negotiate_graph(Some("application/rdf+xml;q=0, text/turtle")), GraphFormat::Turtle);
        // An unsupported type still falls back to N-Triples.
        assert_eq!(negotiate_graph(Some("application/pdf")), GraphFormat::NTriples);
    }

    // [OPUS-4.8] sq-oy1f.1: JSON-LD negotiation is only compiled with the `jsonld` feature.
    #[cfg(feature = "jsonld")]
    #[test]
    fn graph_format_jsonld() {
        assert_eq!(negotiate_graph(Some("application/ld+json")), GraphFormat::JsonLd);
        assert_eq!(GraphFormat::JsonLd.content_type(), "application/ld+json; charset=utf-8");
        // q-values decide between JSON-LD and the other graph formats.
        assert_eq!(
            negotiate_graph(Some("text/turtle;q=0.5, application/ld+json;q=0.9")),
            GraphFormat::JsonLd
        );
        // q=0 rejects JSON-LD, so Turtle wins.
        assert_eq!(
            negotiate_graph(Some("application/ld+json;q=0, text/turtle")),
            GraphFormat::Turtle
        );
        // Exact JSON-LD beats a wildcard.
        assert_eq!(negotiate_graph(Some("application/ld+json, */*")), GraphFormat::JsonLd);
    }

    // [OPUS-4.8] sq-oy1f.1: WITHOUT the feature, `application/ld+json` is just another
    // unsupported type — negotiation must fall through to the default (N-Triples).
    #[cfg(not(feature = "jsonld"))]
    #[test]
    fn graph_format_jsonld_unsupported_without_feature() {
        assert_eq!(negotiate_graph(Some("application/ld+json")), GraphFormat::NTriples);
        // Even alongside a supported type, the JSON-LD range is ignored and Turtle is picked.
        assert_eq!(
            negotiate_graph(Some("application/ld+json;q=0.9, text/turtle;q=0.5")),
            GraphFormat::Turtle
        );
    }
}
