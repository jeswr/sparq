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
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_DECIMAL: &str = "http://www.w3.org/2001/XMLSchema#decimal";
const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";

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
                oxrdf::NamedOrBlankNode::NamedNode(n) => {
                    term_to_xml(s, &Term::NamedNode(n.clone()))
                }
                oxrdf::NamedOrBlankNode::BlankNode(b) => {
                    term_to_xml(s, &Term::BlankNode(b.clone()))
                }
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

/// Flush threshold for [`select_to_csv_chunks`] / [`select_to_tsv_chunks`]: mirrors the
/// engine's `JSON_CHUNK_BYTES` constant (64 KiB) so all three SELECT formats share the same
/// chunking granularity — large enough that per-chunk overhead is negligible, small enough
/// that a streamed body never holds a second whole-result copy in memory. [SONNET-4.6]
const CSV_TSV_CHUNK_BYTES: usize = 64 * 1024;

/// [`select_to_csv`] as an ordered sequence of string chunks whose concatenation is
/// **byte-identical** to the single-string result.
///
/// Row-oriented chunking: rows are accumulated into the current chunk until the chunk
/// exceeds `CSV_TSV_CHUNK_BYTES`, at which point the chunk is flushed and a new one
/// starts. The HTTP server streams these via `chunked_response` (the same path as JSON /
/// T16) so peak memory never holds a second full-result copy. [SONNET-4.6]
pub fn select_to_csv_chunks(r: &QueryResult) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    // Header row (same logic as select_to_csv).
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            current.push(',');
        }
        csv_field(&mut current, v.as_str());
    }
    current.push_str("\r\n");
    // Data rows — flush when the current chunk reaches the threshold.
    for row in &r.rows {
        for (vi, cell) in row.iter().enumerate() {
            if vi > 0 {
                current.push(',');
            }
            if let Some(term) = cell {
                let mut buf = String::new();
                term_lexical_csv(&mut buf, term);
                csv_field(&mut current, &buf);
            }
        }
        current.push_str("\r\n");
        if current.len() >= CSV_TSV_CHUNK_BYTES {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// [`select_to_tsv`] as an ordered sequence of string chunks whose concatenation is
/// **byte-identical** to the single-string result.
///
/// Row-oriented chunking: mirrors [`select_to_csv_chunks`] — rows are accumulated until
/// the current chunk exceeds `CSV_TSV_CHUNK_BYTES`. [SONNET-4.6]
pub fn select_to_tsv_chunks(r: &QueryResult) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    // Header row (same logic as select_to_tsv).
    for (i, v) in r.vars.iter().enumerate() {
        if i > 0 {
            current.push('\t');
        }
        current.push('?');
        current.push_str(v.as_str());
    }
    current.push('\n');
    // Data rows — flush when the current chunk reaches the threshold.
    for row in &r.rows {
        for (vi, cell) in row.iter().enumerate() {
            if vi > 0 {
                current.push('\t');
            }
            if let Some(term) = cell {
                term_to_tsv(&mut current, term);
            }
        }
        current.push('\n');
        if current.len() >= CSV_TSV_CHUNK_BYTES {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
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
///
/// Per the SPARQL 1.1 Query Results TSV format (`§ Encoding terms`, and matching the
/// W3C `csv-tsv-res` `csvtsv01`/`csvtsv03` expected files and the oxigraph reference
/// serialiser), an `xsd:integer` / `xsd:decimal` / `xsd:double` / `xsd:boolean` literal
/// whose lexical form is already a valid Turtle numeric/boolean token is written **bare**
/// (no quotes, no `^^datatype`) — e.g. `"30"^^xsd:integer` → `30`, `"2.2"^^xsd:decimal` →
/// `2.2`, `"1.0E6"^^xsd:double` → `1.0E6`. Crucially the bare token is the literal's OWN
/// lexical form, NOT a canonicalised one: a data-sourced literal round-trips its original
/// spelling (sq-u79ee / survey §C1 / FINDINGS F21 — preserve data, canonical for computed;
/// computed numerics already arrive in canonical form from the engine and are valid tokens
/// too, so they abbreviate canonically). `xsd:string` and `rdf:langString` keep the
/// implicit-datatype short forms; every other datatype (incl. integer/decimal SUBTYPES like
/// `xsd:negativeInteger`, and custom datatypes) is quoted + typed.
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
            // Numeric / boolean abbreviation: write the bare lexical form when its datatype
            // is exactly integer/decimal/double/boolean AND the form is a valid Turtle token.
            if l.language().is_none() {
                let value = l.value();
                let dt = l.datatype();
                let bare = match dt.as_str() {
                    XSD_INTEGER => is_turtle_integer(value),
                    XSD_DECIMAL => is_turtle_decimal(value),
                    XSD_DOUBLE => is_turtle_double(value),
                    XSD_BOOLEAN => is_turtle_boolean(value),
                    _ => false,
                };
                if bare {
                    // Tokens contain no TAB/newline/quote/backslash, so no escaping needed.
                    s.push_str(value);
                    return;
                }
            }
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

// --- Turtle numeric / boolean token recognisers (SPARQL Results TSV abbreviation) ---
//
// Grammar productions mirror the Turtle spec (and oxigraph's `sparesults` reference
// serialiser) so a literal abbreviated here parses BACK to the same RDF term:
//   [19]   INTEGER  ::= [+-]? [0-9]+
//   [20]   DECIMAL  ::= [+-]? [0-9]* '.' [0-9]+
//   [21]   DOUBLE   ::= [+-]? ([0-9]+ '.' [0-9]* EXPONENT | '.' [0-9]+ EXPONENT | [0-9]+ EXPONENT)
//   [154s] EXPONENT ::= [eE] [+-]? [0-9]+
//   [133s] BooleanLiteral ::= 'true' | 'false'

fn is_turtle_boolean(value: &str) -> bool {
    matches!(value, "true" | "false")
}

fn is_turtle_integer(value: &str) -> bool {
    let value = strip_sign(value.as_bytes());
    !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

fn is_turtle_decimal(value: &str) -> bool {
    let mut value = strip_sign(value.as_bytes());
    while value.first().is_some_and(u8::is_ascii_digit) {
        value = &value[1..];
    }
    let Some(value) = value.strip_prefix(b".") else {
        return false;
    };
    !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

fn is_turtle_double(value: &str) -> bool {
    let mut value = strip_sign(value.as_bytes());
    let mut with_before = false;
    while value.first().is_some_and(u8::is_ascii_digit) {
        value = &value[1..];
        with_before = true;
    }
    let mut with_after = false;
    if let Some(v) = value.strip_prefix(b".") {
        value = v;
        while value.first().is_some_and(u8::is_ascii_digit) {
            value = &value[1..];
            with_after = true;
        }
    }
    // EXPONENT is mandatory for a DOUBLE token (a form with no exponent is a DECIMAL).
    value = match value.split_first() {
        Some((b'e' | b'E', rest)) => rest,
        _ => return false,
    };
    value = strip_sign(value);
    (with_before || with_after) && !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

/// Strips a single leading `+`/`-` sign, returning the remaining bytes.
fn strip_sign(value: &[u8]) -> &[u8] {
    match value.split_first() {
        Some((b'+' | b'-', rest)) => rest,
        _ => value,
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
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",
        )
        .unwrap();
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
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?n WHERE { ?s ex:name ?n }",
        )
        .unwrap();
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
        assert!(
            xml.contains("<literal>a &lt; b &amp; c &gt; d</literal>"),
            "unescaped markup: {xml}"
        );
        // The raw, unescaped sequence must NOT appear anywhere in the document.
        assert!(
            !xml.contains("a < b & c > d"),
            "literal leaked raw markup: {xml}"
        );
    }

    #[test]
    fn xml_triple_term_with_a_blank_node_subject() {
        // [OPUS-4.8] sq-4vao: a triple term whose SUBJECT is a blank node exercises the
        // `NamedOrBlankNode::BlankNode` subject arm of the `<triple>` XML encoder (the existing
        // triple-term test only covers a named-node subject). The bnode label is data-dependent,
        // so assert on the structural `<bnode>` element, not its id.
        let g = Graph::load_str(
            "PREFIX : <http://ex/>\n<< _:s :b :c >> :certainty 0.9 .",
            "turtle",
        )
        .unwrap();
        let r = query(
            &g,
            "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }",
        )
        .unwrap();
        let xml = select_to_xml(&r);
        assert!(
            xml.contains("<triple><subject><bnode>"),
            "expected a bnode subject in the triple term: {xml}"
        );
        assert!(
            xml.contains("<predicate><uri>http://ex/b</uri></predicate>"),
            "got: {xml}"
        );
        assert!(
            xml.contains("<object><uri>http://ex/c</uri></object>"),
            "got: {xml}"
        );
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
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",
        )
        .unwrap();
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
        let g = Graph::load_str(
            "PREFIX : <http://ex/>\n<< :a :b :c >> :certainty 0.9 .",
            "turtle",
        )
        .unwrap();
        let r = query(
            &g,
            "SELECT ?t WHERE { ?r <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?t }",
        )
        .unwrap();
        let xml = select_to_xml(&r);
        assert!(
            xml.contains(
                "<triple><subject><uri>http://ex/a</uri></subject><predicate><uri>http://ex/b</uri></predicate><object><uri>http://ex/c</uri></object></triple>"
            ),
            "got: {xml}"
        );
        let tsv = select_to_tsv(&r);
        assert!(
            tsv.contains("<<( <http://ex/a> <http://ex/b> <http://ex/c> )>>"),
            "got: {tsv}"
        );
    }

    #[test]
    fn tsv_header_and_term_syntax() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a ?n WHERE { ?s ex:age ?a . ?s ex:name ?n }",
        )
        .unwrap();
        let tsv = select_to_tsv(&r);
        let header = tsv.lines().next().unwrap();
        assert_eq!(header, "?s\t?a\t?n");
        assert!(tsv.contains("<http://ex/alice>"));
        // xsd:integer is abbreviated to its bare Turtle token (sq-u79ee / TSV §Encoding terms).
        assert!(
            tsv.contains("\t30\t") || tsv.contains("\t30\n"),
            "integer not abbreviated: {tsv}"
        );
        assert!(
            !tsv.contains("\"30\"^^"),
            "integer should not be quoted+typed: {tsv}"
        );
        assert!(tsv.contains("\"Bob\"@en"));
        // plain string: just quoted, no datatype
        assert!(tsv.contains("\"Alice\""));
    }

    /// sq-u79ee / survey §C1 / FINDINGS F21: the SPARQL Results TSV serialiser abbreviates
    /// `xsd:integer` / `xsd:decimal` / `xsd:double` / `xsd:boolean` literals to their bare
    /// Turtle token, PRESERVING the data-sourced literal's own lexical form, and quotes every
    /// other typed literal. Mirrors the W3C `csv-tsv-res/csvtsv03` expected file.
    #[test]
    fn tsv_numeric_abbreviation_preserves_data_lexical_form() {
        // Mirrors data2.ttl from the W3C csv-tsv-res suite (tsv03), with explicit lexical forms.
        let data = r#"@prefix : <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:s1 :p1 "1"^^xsd:string .
:s2 :p2 "2.2"^^xsd:decimal .
:s3 :p3 "-3"^^xsd:negativeInteger .
:s4 :p4 "4,4"^^xsd:string .
:s5 :p5 "5,5"^^:myCustomDatatype .
:s6 :p6 "1.0E6"^^xsd:double .
:s7 :p7 "a7"^^xsd:hexBinary ."#;
        let g = Graph::load_str(data, "turtle").unwrap();
        let r = query(&g, "SELECT ?s ?o WHERE { ?s ?p ?o } ORDER BY ?s").unwrap();
        let tsv = select_to_tsv(&r);

        // Bare numeric tokens (the data's OWN lexical form is preserved — NOT canonicalised:
        // the double stays "1.0E6", it is not rewritten to "1.0e6" or "1000000").
        assert!(
            line_ends_with(&tsv, "\t2.2"),
            "decimal not abbreviated: {tsv}"
        );
        assert!(
            line_ends_with(&tsv, "\t1.0E6"),
            "double lexical form not preserved: {tsv}"
        );
        assert!(
            !tsv.contains("1.0e6"),
            "double must not be canonicalised: {tsv}"
        );
        assert!(
            !tsv.contains("1000000"),
            "double must not be canonicalised: {tsv}"
        );
        // xsd:string keeps the quoted short form (no datatype) even when it looks numeric.
        assert!(
            line_ends_with(&tsv, "\t\"1\""),
            "xsd:string `1` should stay quoted: {tsv}"
        );
        assert!(
            line_ends_with(&tsv, "\t\"4,4\""),
            "xsd:string `4,4` should stay quoted: {tsv}"
        );
        // Integer/decimal SUBTYPES and custom datatypes stay quoted + typed.
        assert!(
            line_ends_with(
                &tsv,
                "\t\"-3\"^^<http://www.w3.org/2001/XMLSchema#negativeInteger>"
            ),
            "negativeInteger should not be abbreviated: {tsv}"
        );
        assert!(
            line_ends_with(&tsv, "\t\"5,5\"^^<http://example.org/myCustomDatatype>"),
            "custom datatype should stay quoted+typed: {tsv}"
        );
        assert!(
            line_ends_with(
                &tsv,
                "\t\"a7\"^^<http://www.w3.org/2001/XMLSchema#hexBinary>"
            ),
            "hexBinary should stay quoted+typed: {tsv}"
        );
    }

    /// sq-u79ee: a COMPUTED double (from an arithmetic expression) carries the engine's
    /// canonical lexical form — the "canonical for computed" half of the preserve-data /
    /// canonical-computed contract. The engine's integral-double convention spells
    /// `1.0E6 + 0.0` as the plain `1000000` (no exponent); since that is NOT a valid Turtle
    /// DOUBLE token, the TSV serialiser keeps it quoted + `^^xsd:double` rather than emitting
    /// a bare `1000000` that would round-trip to `xsd:integer`. Abbreviation never silently
    /// changes a term's datatype.
    #[test]
    fn tsv_computed_double_stays_canonical() {
        let g = Graph::load_str(
            "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             <http://e/s> <http://e/p> \"1.0E6\"^^xsd:double .",
            "turtle",
        )
        .unwrap();
        let r = query(
            &g,
            "SELECT (?o + \"0.0\"^^<http://www.w3.org/2001/XMLSchema#double> AS ?c) WHERE { ?s ?p ?o }",
        )
        .unwrap();
        let tsv = select_to_tsv(&r);
        // Canonical engine form, datatype preserved (not abbreviated to a bare integer token).
        assert!(
            cell_is(
                &tsv,
                "\"1000000\"^^<http://www.w3.org/2001/XMLSchema#double>"
            ),
            "computed double should keep its canonical form + datatype: {tsv}"
        );
        // The data spelling 1.0E6 is NOT echoed for a computed value.
        assert!(
            !tsv.contains("1.0E6"),
            "computed value should not carry the data lexical form: {tsv}"
        );
    }

    /// sq-u79ee: a COMPUTED double whose canonical form lands in scientific notation IS a
    /// valid Turtle DOUBLE token, so it abbreviates bare — confirming computed doubles still
    /// serialise in the engine's canonical (mantissa-E-exponent) spelling, never the data's.
    #[test]
    fn tsv_computed_fractional_double_is_canonical_scientific() {
        let g = Graph::load_str(
            "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             <http://e/s> <http://e/p> \"3.0\"^^xsd:double .",
            "turtle",
        )
        .unwrap();
        // 3.0 / 8.0 = 0.375 → canonical xsd:double "3.75E-1".
        let r = query(
            &g,
            "SELECT (?o / \"8.0\"^^<http://www.w3.org/2001/XMLSchema#double> AS ?c) WHERE { ?s ?p ?o }",
        )
        .unwrap();
        let tsv = select_to_tsv(&r);
        assert!(
            cell_is(&tsv, "3.75E-1"),
            "computed double not canonical scientific + bare: {tsv}"
        );
        assert!(
            !tsv.contains("\"3.75E-1\"^^"),
            "valid Turtle double token should be bare: {tsv}"
        );
    }

    /// sq-u79ee: integer/decimal/double subtype + token edge cases for the TSV abbreviation
    /// recognisers — exercises the bare-vs-quoted decision directly.
    #[test]
    fn tsv_abbreviation_edge_cases() {
        let data = r#"@prefix : <http://e/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:a :p "+42"^^xsd:integer .
:b :p "007"^^xsd:integer .
:c :p "-0.5"^^xsd:decimal .
:d :p ".5"^^xsd:decimal .
:e :p "1e10"^^xsd:double .
:f :p ".5e3"^^xsd:double .
:g :p "true"^^xsd:boolean .
:h :p "1"^^xsd:double .
:i :p "NaN"^^xsd:double ."#;
        let g = Graph::load_str(data, "turtle").unwrap();
        let r = query(&g, "SELECT ?s ?o WHERE { ?s ?p ?o } ORDER BY ?s").unwrap();
        let tsv = select_to_tsv(&r);
        // Signed/leading-zero integers abbreviate, preserving spelling.
        assert!(line_ends_with(&tsv, "\t+42"), "signed integer: {tsv}");
        assert!(
            line_ends_with(&tsv, "\t007"),
            "leading-zero integer preserved: {tsv}"
        );
        // Decimal forms.
        assert!(line_ends_with(&tsv, "\t-0.5"), "signed decimal: {tsv}");
        assert!(line_ends_with(&tsv, "\t.5"), "leading-dot decimal: {tsv}");
        // Doubles require an exponent.
        assert!(
            line_ends_with(&tsv, "\t1e10"),
            "double with exponent: {tsv}"
        );
        assert!(line_ends_with(&tsv, "\t.5e3"), "leading-dot double: {tsv}");
        // boolean true.
        assert!(line_ends_with(&tsv, "\ttrue"), "boolean: {tsv}");
        // "1"^^xsd:double has no exponent → NOT a valid Turtle DOUBLE token → quoted+typed.
        assert!(
            line_ends_with(&tsv, "\t\"1\"^^<http://www.w3.org/2001/XMLSchema#double>"),
            "double without exponent must stay quoted+typed: {tsv}"
        );
        // "NaN"^^xsd:double is a legal XSD double VALUE but not a Turtle DOUBLE token → quoted.
        assert!(
            line_ends_with(&tsv, "\t\"NaN\"^^<http://www.w3.org/2001/XMLSchema#double>"),
            "NaN must stay quoted+typed: {tsv}"
        );
    }

    // -----------------------------------------------------------------------
    // Chunked serialiser tests (sq-7d3dj.12) [SONNET-4.6]
    // -----------------------------------------------------------------------

    /// select_to_csv_chunks concatenates to exactly select_to_csv for a typical small result.
    #[test]
    fn csv_chunks_byte_identical_small() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",
        )
        .unwrap();
        let single = select_to_csv(&r);
        let chunks = select_to_csv_chunks(&r);
        assert!(!chunks.is_empty(), "chunks must not be empty");
        assert_eq!(
            chunks.concat(),
            single,
            "CSV chunks must concatenate to the single-string form"
        );
    }

    /// select_to_tsv_chunks concatenates to exactly select_to_tsv for a typical small result.
    #[test]
    fn tsv_chunks_byte_identical_small() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s ?a WHERE { ?s ex:age ?a }",
        )
        .unwrap();
        let single = select_to_tsv(&r);
        let chunks = select_to_tsv_chunks(&r);
        assert!(!chunks.is_empty(), "chunks must not be empty");
        assert_eq!(
            chunks.concat(),
            single,
            "TSV chunks must concatenate to the single-string form"
        );
    }

    /// An empty result (zero rows) produces exactly the header row as a single chunk.
    #[test]
    fn csv_chunks_empty_result() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age 9999 }",
        )
        .unwrap();
        assert!(r.rows.is_empty(), "expected no rows");
        let single = select_to_csv(&r);
        let chunks = select_to_csv_chunks(&r);
        assert_eq!(
            chunks.concat(),
            single,
            "empty CSV must still produce a header chunk"
        );
    }

    /// An empty result (zero rows) for TSV produces exactly the header row as a single chunk.
    #[test]
    fn tsv_chunks_empty_result() {
        let r = query(
            &g(),
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age 9999 }",
        )
        .unwrap();
        assert!(r.rows.is_empty(), "expected no rows");
        let single = select_to_tsv(&r);
        let chunks = select_to_tsv_chunks(&r);
        assert_eq!(
            chunks.concat(),
            single,
            "empty TSV must still produce a header chunk"
        );
    }

    /// A large result (> CSV_TSV_CHUNK_BYTES bytes) produces multiple CSV chunks, all
    /// concatenating byte-identically to the single-string form. [SONNET-4.6]
    #[test]
    fn csv_chunks_splits_large_result() {
        // ~2 000 rows × ~80 bytes/row ≈ 160 KiB > 64 KiB chunk threshold.
        let mut data = String::new();
        for i in 0..2000_u32 {
            data.push_str(&format!(
                "<http://ex/s{}> <http://ex/p> \"{:0>60}\" .\n",
                i, i
            ));
        }
        let g = Graph::load_str(&data, "ntriples").unwrap();
        let r = query(&g, "SELECT ?s ?o WHERE { ?s ?p ?o }").unwrap();
        let single = select_to_csv(&r);
        let chunks = select_to_csv_chunks(&r);
        assert!(
            chunks.len() > 1,
            "expected multiple CSV chunks for a large result; got {} chunk(s)",
            chunks.len()
        );
        assert_eq!(
            chunks.concat(),
            single,
            "large CSV chunk concat must equal single-string form"
        );
    }

    /// A large result (> CSV_TSV_CHUNK_BYTES bytes) produces multiple TSV chunks, all
    /// concatenating byte-identically to the single-string form. [SONNET-4.6]
    #[test]
    fn tsv_chunks_splits_large_result() {
        let mut data = String::new();
        for i in 0..2000_u32 {
            data.push_str(&format!(
                "<http://ex/s{}> <http://ex/p> \"{:0>60}\" .\n",
                i, i
            ));
        }
        let g = Graph::load_str(&data, "ntriples").unwrap();
        let r = query(&g, "SELECT ?s ?o WHERE { ?s ?p ?o }").unwrap();
        let single = select_to_tsv(&r);
        let chunks = select_to_tsv_chunks(&r);
        assert!(
            chunks.len() > 1,
            "expected multiple TSV chunks for a large result; got {} chunk(s)",
            chunks.len()
        );
        assert_eq!(
            chunks.concat(),
            single,
            "large TSV chunk concat must equal single-string form"
        );
    }

    /// Helper: true iff some line of `tsv` ends with `suffix` (TSV rows are LF-terminated).
    fn line_ends_with(tsv: &str, suffix: &str) -> bool {
        tsv.lines().any(|l| l.ends_with(suffix))
    }

    /// Helper: true iff some TAB-separated CELL (in any data row) equals `value` exactly —
    /// robust for single-column results where the cell is the whole line.
    fn cell_is(tsv: &str, value: &str) -> bool {
        tsv.lines()
            .skip(1)
            .flat_map(|l| l.split('\t'))
            .any(|c| c == value)
    }

    // [OPUS-4.8] sq-qcnn.37: direct unit tests for the pure XML/TSV encoding helpers that the
    // integration path leaves uncovered (the specific special-char arms and the decimal-no-dot
    // false path). These exercise the REAL encoding logic, not a proxy.

    #[test]
    fn xml_attr_escape_encodes_xml_special_characters() {
        // Each XML special character must be encoded to its entity reference.
        let mut s = String::new();
        xml_attr_escape(&mut s, "&<>\"");
        assert_eq!(s, "&amp;&lt;&gt;&quot;");
        // Plain characters pass through unchanged.
        let mut t = String::new();
        xml_attr_escape(&mut t, "hello");
        assert_eq!(t, "hello");
    }

    #[test]
    fn push_ws_charref_encodes_tab_and_carriage_return() {
        // \t must map to &#9; (the XML numeric character reference for HT).
        let mut s = String::new();
        push_ws_charref(&mut s, '\t');
        assert_eq!(s, "&#9;");
        // \r must map to &#13; (CR — used in some OS line endings).
        let mut r = String::new();
        push_ws_charref(&mut r, '\r');
        assert_eq!(r, "&#13;");
    }

    #[test]
    fn is_turtle_decimal_returns_false_when_no_dot_present() {
        // An integer-only token (no '.') is NOT a Turtle DECIMAL — it is a Turtle INTEGER.
        assert!(!is_turtle_decimal("123"));
        assert!(!is_turtle_decimal("-456"));
        assert!(!is_turtle_decimal("+7"));
        // With a dot it IS a decimal (regression-guard for the passing path too).
        assert!(is_turtle_decimal("12.3"));
        assert!(is_turtle_decimal(".5"));
    }

    #[test]
    fn push_ws_charref_covers_newline_space_and_passthrough_arms() {
        // [OPUS-4.8] sq-qcnn.37: cover the two arms not exercised by the tab/CR test above.
        // '\n' must map to &#10;
        let mut nl = String::new();
        push_ws_charref(&mut nl, '\n');
        assert_eq!(nl, "&#10;");
        // ' ' must map to &#32;
        let mut sp = String::new();
        push_ws_charref(&mut sp, ' ');
        assert_eq!(sp, "&#32;");
        // Any other character passes through verbatim (the defensive `_` arm that the
        // comment notes is unreachable in normal use but that the match must have).
        let mut pt = String::new();
        push_ws_charref(&mut pt, 'a');
        assert_eq!(pt, "a");
    }

    #[test]
    fn is_turtle_boolean_returns_false_for_non_boolean_lexical_form() {
        // [OPUS-4.8] sq-qcnn.37: exercise the non-matching (returns-false) arm of
        // is_turtle_boolean. The matching arms are already exercised indirectly through
        // the `tsv_abbreviation_edge_cases` test ("true"^^xsd:boolean).
        assert!(!is_turtle_boolean("yes"));
        assert!(!is_turtle_boolean("1"));
        assert!(!is_turtle_boolean("True")); // case-sensitive
                                             // Both canonical boolean values must still return true (regression guard).
        assert!(is_turtle_boolean("true"));
        assert!(is_turtle_boolean("false"));
    }
}
