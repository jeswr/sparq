//! W3C SPARQL results serialisers that the engine does not already provide.
//!
//! The engine ships `sparq_engine::query_json` / `sparq_engine::json::to_sparql_json`
//! for **SPARQL Results JSON** (`application/sparql-results+json`). This module adds the
//! other standard SELECT/ASK result formats, all driven off the engine's public
//! [`sparq_engine::QueryResult`] (`{ vars, rows }`):
//!
//! * **XML**  — <https://www.w3.org/TR/rdf-sparql-XMLres/> (`application/sparql-results+xml`)
//! * **CSV**  — <https://www.w3.org/TR/sparql11-results-csv-tsv/> (`text/csv`)
//! * **TSV**  — <https://www.w3.org/TR/sparql11-results-csv-tsv/> (`text/tab-separated-values`)
//! * **ASK boolean** — JSON (`{"head":{},"boolean":true}`) and XML.
//!
//! These are deliberately pure (no async, no HTTP types) so they are unit-testable and
//! the `server` feature is not required to use them.

use oxrdf::Term;
use sparq_engine::QueryResult;

/// SPARQL Results XML media type.
pub const XML_MEDIA: &str = "application/sparql-results+xml";
/// SPARQL Results JSON media type.
pub const JSON_MEDIA: &str = "application/sparql-results+json";
/// SPARQL Results CSV media type.
pub const CSV_MEDIA: &str = "text/csv";
/// SPARQL Results TSV media type.
pub const TSV_MEDIA: &str = "text/tab-separated-values";

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// ---------------------------------------------------------------------------
// SELECT → XML  (https://www.w3.org/TR/rdf-sparql-XMLres/)
// ---------------------------------------------------------------------------

/// Serialises a SELECT result to the SPARQL Query Results XML Format.
pub fn select_to_xml(r: &QueryResult) -> String {
    let mut s = String::with_capacity(128 + r.rows.len() * 48);
    s.push_str("<?xml version=\"1.0\"?>\n");
    s.push_str("<sparql xmlns=\"http://www.w3.org/2005/sparql-results#\">\n");
    s.push_str("  <head>\n");
    for v in &r.vars {
        s.push_str("    <variable name=\"");
        xml_attr_escape(&mut s, v.as_str());
        s.push_str("\"/>\n");
    }
    s.push_str("  </head>\n");
    s.push_str("  <results>\n");
    for row in &r.rows {
        s.push_str("    <result>\n");
        for (vi, cell) in row.iter().enumerate() {
            if let Some(term) = cell {
                s.push_str("      <binding name=\"");
                xml_attr_escape(&mut s, r.vars[vi].as_str());
                s.push_str("\">");
                term_to_xml(&mut s, term);
                s.push_str("</binding>\n");
            }
        }
        s.push_str("    </result>\n");
    }
    s.push_str("  </results>\n");
    s.push_str("</sparql>\n");
    s
}

fn term_to_xml(s: &mut String, t: &Term) {
    match t {
        Term::NamedNode(n) => {
            s.push_str("<uri>");
            xml_text_escape(s, n.as_str());
            s.push_str("</uri>");
        }
        Term::BlankNode(b) => {
            s.push_str("<bnode>");
            xml_text_escape(s, b.as_str());
            s.push_str("</bnode>");
        }
        Term::Literal(l) => {
            if let Some(lang) = l.language() {
                s.push_str("<literal xml:lang=\"");
                xml_attr_escape(s, lang);
                s.push_str("\">");
                xml_text_escape(s, l.value());
                s.push_str("</literal>");
            } else {
                let dt = l.datatype();
                if dt.as_str() != XSD_STRING {
                    s.push_str("<literal datatype=\"");
                    xml_attr_escape(s, dt.as_str());
                    s.push_str("\">");
                } else {
                    s.push_str("<literal>");
                }
                xml_text_escape(s, l.value());
                s.push_str("</literal>");
            }
        }
        // RDF 1.2 triple term — the SPARQL 1.2 XML results encoding:
        // <triple><subject>…</subject><predicate>…</predicate><object>…</object></triple>.
        Term::Triple(t) => {
            s.push_str("<triple><subject>");
            match &t.subject {
                oxrdf::NamedOrBlankNode::NamedNode(n) => term_to_xml(s, &Term::NamedNode(n.clone())),
                oxrdf::NamedOrBlankNode::BlankNode(b) => term_to_xml(s, &Term::BlankNode(b.clone())),
            }
            s.push_str("</subject><predicate>");
            term_to_xml(s, &Term::NamedNode(t.predicate.clone()));
            s.push_str("</predicate><object>");
            term_to_xml(s, &t.object);
            s.push_str("</object></triple>");
        }
    }
}

/// ASK boolean as SPARQL Results XML.
pub fn ask_to_xml(value: bool) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n<sparql xmlns=\"http://www.w3.org/2005/sparql-results#\">\n  <head/>\n  <boolean>{}</boolean>\n</sparql>\n",
        value
    )
}

/// ASK boolean as SPARQL Results JSON.
pub fn ask_to_json(value: bool) -> String {
    format!("{{\"head\":{{}},\"boolean\":{}}}", value)
}

// ---------------------------------------------------------------------------
// SELECT → CSV / TSV  (https://www.w3.org/TR/sparql11-results-csv-tsv/)
// ---------------------------------------------------------------------------

/// Serialises a SELECT result to SPARQL Results CSV.
///
/// Per the spec: header is the variable names (no leading `?`); each binding is rendered
/// by its lexical form (IRIs verbatim, bnodes as `_:label`, literals as their string with
/// NO datatype/lang), unbound = empty; CSV-quote a field iff it contains `"`, `,`, CR or LF;
/// line terminator is CRLF.
pub fn select_to_csv(r: &QueryResult) -> String {
    let mut s = String::with_capacity(64 + r.rows.len() * 32);
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        csv_field(&mut s, v.as_str());
    }
    s.push_str("\r\n");
    for row in &r.rows {
        for (vi, cell) in row.iter().enumerate() {
            if vi > 0 {
                s.push(',');
            }
            if let Some(term) = cell {
                let mut buf = String::new();
                term_lexical_csv(&mut buf, term);
                csv_field(&mut s, &buf);
            }
        }
        s.push_str("\r\n");
    }
    s
}

/// Serialises a SELECT result to SPARQL Results TSV.
///
/// Per the spec: header variables are written WITH the leading `?`; values use the
/// SPARQL term syntax (IRIs as `<...>`, literals with quotes + datatype/lang, bnodes as
/// `_:label`); TAB/newline/CR/`"`/`\` inside literals are escaped; unbound = empty;
/// fields are TAB-separated, rows LF-terminated.
pub fn select_to_tsv(r: &QueryResult) -> String {
    let mut s = String::with_capacity(64 + r.rows.len() * 32);
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            s.push('\t');
        }
        s.push('?');
        s.push_str(v.as_str());
    }
    s.push('\n');
    for row in &r.rows {
        for (vi, cell) in row.iter().enumerate() {
            if vi > 0 {
                s.push('\t');
            }
            if let Some(term) = cell {
                term_to_tsv(&mut s, term);
            }
        }
        s.push('\n');
    }
    s
}

/// CSV value lexical form: IRI verbatim, bnode `_:label`, literal = its string value only.
fn term_lexical_csv(s: &mut String, t: &Term) {
    match t {
        Term::NamedNode(n) => s.push_str(n.as_str()),
        Term::BlankNode(b) => {
            s.push_str("_:");
            s.push_str(b.as_str());
        }
        Term::Literal(l) => s.push_str(l.value()),
        other => s.push_str(&other.to_string()),
    }
}

/// Writes one CSV field, quoting per RFC 4180 only when required.
fn csv_field(out: &mut String, v: &str) {
    let needs_quote = v.bytes().any(|b| matches!(b, b'"' | b',' | b'\n' | b'\r'));
    if !needs_quote {
        out.push_str(v);
        return;
    }
    out.push('"');
    for ch in v.chars() {
        if ch == '"' {
            out.push('"'); // doubled per RFC 4180
        }
        out.push(ch);
    }
    out.push('"');
}

/// TSV value: full SPARQL term syntax with TSV escaping inside literals.
fn term_to_tsv(s: &mut String, t: &Term) {
    match t {
        Term::NamedNode(n) => {
            s.push('<');
            s.push_str(n.as_str());
            s.push('>');
        }
        Term::BlankNode(b) => {
            s.push_str("_:");
            s.push_str(b.as_str());
        }
        Term::Literal(l) => {
            s.push('"');
            tsv_escape(s, l.value());
            s.push('"');
            if let Some(lang) = l.language() {
                s.push('@');
                s.push_str(lang);
            } else {
                let dt = l.datatype();
                if dt.as_str() != XSD_STRING {
                    s.push_str("^^<");
                    s.push_str(dt.as_str());
                    s.push('>');
                }
            }
        }
        other => s.push_str(&other.to_string()),
    }
}

/// Escapes a literal lexical form for the quoted TSV string production.
fn tsv_escape(out: &mut String, v: &str) {
    for ch in v.chars() {
        match ch {
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
}

// ---------------------------------------------------------------------------
// XML escaping
// ---------------------------------------------------------------------------

fn xml_text_escape(out: &mut String, s: &str) {
    // [OPUS-4.8] Leading/trailing whitespace in element TEXT content is silently dropped by
    // any XML parser that trims text events (the default for quick-xml / the `sparesults`
    // reference parser, and most SAX-style consumers) — so a literal `"  pad  "` written as
    // `<literal>  pad  </literal>` round-trips to `"pad"`. (Found by the serializer oracle in
    // tests/serializer_oracle.rs.) Escape BOUNDARY whitespace (space/tab/CR/LF at the start
    // or end) as numeric character references so it survives the round trip; interior
    // whitespace is preserved by parsers and is left verbatim. Mirrors the reference
    // serializer's `escape_including_bound_whitespaces` (sparesults' xml.rs).
    // The whitespace set XML parsers trim from text events.
    const XML_BOUND_WS: [char; 4] = ['\t', '\n', '\r', ' '];
    let trimmed = s.trim_matches(XML_BOUND_WS);
    if trimmed.len() == s.len() {
        // No boundary whitespace — the common path.
        xml_text_escape_body(out, s);
        return;
    }
    let prefix_len = s.len() - s.trim_start_matches(XML_BOUND_WS).len();
    for ch in s[..prefix_len].chars() {
        push_ws_charref(out, ch);
    }
    xml_text_escape_body(out, trimmed);
    for ch in s[prefix_len + trimmed.len()..].chars() {
        push_ws_charref(out, ch);
    }
}

/// Escapes the XML-significant characters (`&`, `<`, `>`) in text content; boundary
/// whitespace handling is done by [`xml_text_escape`]. [OPUS-4.8]
fn xml_text_escape_body(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

/// Writes a whitespace character as its XML numeric character reference. [OPUS-4.8]
fn push_ws_charref(out: &mut String, ch: char) {
    match ch {
        '\t' => out.push_str("&#9;"),
        '\n' => out.push_str("&#10;"),
        '\r' => out.push_str("&#13;"),
        ' ' => out.push_str("&#32;"),
        // `trim_matches` above strips only these four, so nothing else reaches here.
        _ => out.push(ch),
    }
}

fn xml_attr_escape(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;
    use sparq_engine::query;

    const DATA: &str = r#"
        @prefix ex: <http://ex/> .
        ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
        ex:bob   ex:age 25 ; ex:name "Bob"@en .
    "#;

    fn g() -> Graph {
        Graph::load_str(DATA, "turtle").unwrap()
    }

    #[test]
    fn xml_has_header_and_results() {
        let r = query(&g(), "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }").unwrap();
        let xml = select_to_xml(&r);
        assert!(xml.starts_with("<?xml version=\"1.0\"?>"));
        assert!(xml.contains("xmlns=\"http://www.w3.org/2005/sparql-results#\""));
        assert!(xml.contains("<variable name=\"s\"/>"));
        assert!(xml.contains("<variable name=\"a\"/>"));
        assert!(xml.contains("<uri>http://ex/alice</uri>"));
        // inline integer literal carries its datatype
        assert!(xml.contains("datatype=\"http://www.w3.org/2001/XMLSchema#integer\""));
    }

    #[test]
    fn xml_escapes_and_lang() {
        let r = query(&g(), "PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }").unwrap();
        let xml = select_to_xml(&r);
        assert!(xml.contains("<literal xml:lang=\"en\">Bob</literal>"));
        // plain xsd:string literal => no datatype attribute
        assert!(xml.contains("<literal>Alice</literal>"));
    }

    #[test]
    fn xml_text_body_escapes_markup_characters() {
        // [OPUS-4.8] sq-4vao: a literal value containing XML-significant characters must be
        // escaped in the `<literal>` text body, NOT emitted verbatim — otherwise a literal like
        // `<script>` would break the results document (and be an injection vector). Exercises the
        // `<` and `>` body arms (the `&` arm too) of `xml_text_escape_body`.
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:v \"a < b & c > d\" .",
            "turtle",
        )
        .unwrap();
        let r = query(&g, "PREFIX ex: <http://ex/> SELECT ?v WHERE { ?s ex:v ?v }").unwrap();
        let xml = select_to_xml(&r);
        assert!(xml.contains("<literal>a &lt; b &amp; c &gt; d</literal>"), "unescaped markup: {xml}");
        // The raw, unescaped sequence must NOT appear anywhere in the document.
        assert!(!xml.contains("a < b & c > d"), "literal leaked raw markup: {xml}");
    }

    #[test]
    fn xml_triple_term_with_a_blank_node_subject() {
        // [OPUS-4.8] sq-4vao: a triple term whose SUBJECT is a blank node exercises the
        // `NamedOrBlankNode::BlankNode` subject arm of the `<triple>` XML encoder (the existing
        // triple-term test only covers a named-node subject). The bnode label is data-dependent,
        // so assert on the structural `<bnode>` element, not its id.
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< _:s :b :c >> :certainty 0.9 .", "turtle").unwrap();
        let r = query(&g, "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }").unwrap();
        let xml = select_to_xml(&r);
        assert!(xml.contains("<triple><subject><bnode>"), "expected a bnode subject in the triple term: {xml}");
        assert!(xml.contains("<predicate><uri>http://ex/b</uri></predicate>"), "got: {xml}");
        assert!(xml.contains("<object><uri>http://ex/c</uri></object>"), "got: {xml}");
    }

    #[test]
    fn ask_serialisers() {
        assert_eq!(ask_to_json(true), "{\"head\":{},\"boolean\":true}");
        assert_eq!(ask_to_json(false), "{\"head\":{},\"boolean\":false}");
        assert!(ask_to_xml(true).contains("<boolean>true</boolean>"));
        assert!(ask_to_xml(false).contains("<boolean>false</boolean>"));
    }

    #[test]
    fn csv_header_and_lexical() {
        let r = query(&g(), "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }").unwrap();
        let csv = select_to_csv(&r);
        let mut lines = csv.split("\r\n");
        assert_eq!(lines.next().unwrap(), "s,a");
        // IRIs verbatim, integer literal as bare lexical
        assert!(csv.contains("http://ex/alice,30"));
        assert!(csv.ends_with("\r\n"));
    }

    #[test]
    fn csv_quotes_when_needed() {
        let g = Graph::load_str(
            "@prefix ex: <http://ex/> . ex:a ex:v \"a,b\\\"c\" .",
            "turtle",
        )
        .unwrap();
        let r = query(&g, "PREFIX ex: <http://ex/> SELECT ?v WHERE { ?s ex:v ?v }").unwrap();
        let csv = select_to_csv(&r);
        // field with comma + quote must be quoted, inner quote doubled
        assert!(csv.contains("\"a,b\"\"c\""), "got: {csv}");
    }

    #[test]
    fn xml_and_tsv_triple_terms() {
        // RDF 1.2 triple term: SPARQL 1.2 XML <triple> element; TSV falls back to the
        // SPARQL 1.2 `<<( … )>>` term syntax via oxrdf's Display.
        let g = Graph::load_str("PREFIX : <http://ex/>\n<< :a :b :c >> :certainty 0.9 .", "turtle").unwrap();
        let r = query(&g, "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }").unwrap();
        let xml = select_to_xml(&r);
        assert!(
            xml.contains(
                "<triple><subject><uri>http://ex/a</uri></subject><predicate><uri>http://ex/b</uri></predicate><object><uri>http://ex/c</uri></object></triple>"
            ),
            "got: {xml}"
        );
        let tsv = select_to_tsv(&r);
        assert!(tsv.contains("<<( <http://ex/a> <http://ex/b> <http://ex/c> )>>"), "got: {tsv}");
    }

    #[test]
    fn tsv_header_and_term_syntax() {
        let r = query(&g(), "PREFIX ex: <http://ex/> SELECT ?s ?a ?n WHERE { ?s ex:age ?a . ?s ex:name ?n }").unwrap();
        let tsv = select_to_tsv(&r);
        let header = tsv.lines().next().unwrap();
        assert_eq!(header, "?s\t?a\t?n");
        assert!(tsv.contains("<http://ex/alice>"));
        assert!(tsv.contains("\"30\"^^<http://www.w3.org/2001/XMLSchema#integer>"));
        assert!(tsv.contains("\"Bob\"@en"));
        // plain string: just quoted, no datatype
        assert!(tsv.contains("\"Alice\""));
    }
}
