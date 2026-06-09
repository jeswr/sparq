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

/// Negotiates the result format from an optional `Accept` header value.
pub fn negotiate(accept: Option<&str>) -> Format {
    let accept = match accept {
        Some(a) if !a.trim().is_empty() => a,
        _ => return Format::Json,
    };

    let mut best: Option<(Format, f32, usize)> = None; // (format, q, specificity)
    for part in accept.split(',') {
        let mut it = part.split(';');
        let media = it.next().unwrap_or("").trim().to_ascii_lowercase();
        let mut q = 1.0f32;
        for param in it {
            let param = param.trim();
            if let Some(v) = param.strip_prefix("q=") {
                q = v.parse().unwrap_or(1.0);
            }
        }
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
}
