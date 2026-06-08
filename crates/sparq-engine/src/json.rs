//! SPARQL 1.1 JSON results serialisation (`application/sparql-results+json`),
//! hand-written (no serde) so it is shared by the CLI and the wasm build with no
//! extra dependencies.

use crate::QueryResult;
use oxrdf::Term;
use sparq_core::dict::TermParts;

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// Serialises a term's JSON binding object directly from its dictionary components,
/// with no intermediate `oxrdf::Term` allocation (the materialise hot path).
pub(crate) fn parts_to_json(s: &mut String, parts: TermParts<'_>) {
    match parts {
        TermParts::Iri { prefix, suffix } => {
            s.push_str("{\"type\":\"uri\",\"value\":\"");
            escape_into(s, prefix);
            escape_into(s, suffix);
            s.push_str("\"}");
        }
        TermParts::Blank(b) => {
            s.push_str("{\"type\":\"bnode\",\"value\":\"");
            escape_into(s, b);
            s.push_str("\"}");
        }
        TermParts::Lit { value, datatype, lang } => {
            s.push_str("{\"type\":\"literal\",\"value\":\"");
            escape_into(s, value);
            s.push('"');
            if let Some(lang) = lang {
                s.push_str(",\"xml:lang\":\"");
                escape_into(s, lang);
                s.push('"');
            } else if datatype != XSD_STRING {
                s.push_str(",\"datatype\":\"");
                escape_into(s, datatype);
                s.push('"');
            }
            s.push('}');
        }
    }
}

/// JSON for an inline-integer value id (an `xsd:integer`), heap-free.
pub(crate) fn inline_int_json(s: &mut String, mut v: u32) {
    s.push_str("{\"type\":\"literal\",\"value\":\"");
    if v == 0 {
        s.push('0');
    } else {
        let mut digits = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            digits[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        for &d in digits[..n].iter().rev() {
            s.push(d as char);
        }
    }
    s.push_str("\",\"datatype\":\"http://www.w3.org/2001/XMLSchema#integer\"}");
}

/// Serialises a query result to a SPARQL 1.1 JSON results string.
pub fn to_sparql_json(r: &QueryResult) -> String {
    let mut s = String::with_capacity(64 + r.rows.len() * 32);
    s.push_str("{\"head\":{\"vars\":[");
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        escape_into(&mut s, v.as_str());
        s.push('"');
    }
    s.push_str("]},\"results\":{\"bindings\":[");
    for (ri, row) in r.rows.iter().enumerate() {
        if ri > 0 {
            s.push(',');
        }
        s.push('{');
        let mut first = true;
        for (vi, cell) in row.iter().enumerate() {
            if let Some(term) = cell {
                if !first {
                    s.push(',');
                }
                first = false;
                s.push('"');
                escape_into(&mut s, r.vars[vi].as_str());
                s.push_str("\":");
                term_to_json(&mut s, term);
            }
        }
        s.push('}');
    }
    s.push_str("]}}");
    s
}

pub(crate) fn term_to_json(s: &mut String, t: &Term) {
    match t {
        Term::NamedNode(n) => {
            s.push_str("{\"type\":\"uri\",\"value\":\"");
            escape_into(s, n.as_str());
            s.push_str("\"}");
        }
        Term::BlankNode(b) => {
            s.push_str("{\"type\":\"bnode\",\"value\":\"");
            escape_into(s, b.as_str());
            s.push_str("\"}");
        }
        Term::Literal(l) => {
            s.push_str("{\"type\":\"literal\",\"value\":\"");
            escape_into(s, l.value());
            s.push('"');
            if let Some(lang) = l.language() {
                s.push_str(",\"xml:lang\":\"");
                escape_into(s, lang);
                s.push('"');
            } else {
                let dt = l.datatype();
                if dt.as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                    s.push_str(",\"datatype\":\"");
                    escape_into(s, dt.as_str());
                    s.push('"');
                }
            }
            s.push('}');
        }
        // RDF-star quoted triple: fall back to its N-Triples form as a literal.
        other => {
            s.push_str("{\"type\":\"literal\",\"value\":\"");
            escape_into(s, &other.to_string());
            s.push_str("\"}");
        }
    }
}

/// Appends `s` to `out` with JSON string escaping.
pub(crate) fn escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}
