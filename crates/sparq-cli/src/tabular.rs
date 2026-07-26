//! [FABLE-5] (sq-lsp7k.8, epic sq-lsp7k) MATERIALIZING tabular→RDF import — the `tabular`
//! subcommand: CSV **direct mapping** + the **R2RML materializing subset** over CSV logical
//! tables. sparq's counter-story to SQL virtualization platforms is *import fast, reload fast*:
//! CSV rows stream through a first-party RFC-4180 reader, are mapped row-by-row to N-Triples
//! chunks, and feed the existing parallel N-Triples ingest (`Graph::load_reader_parallel`) —
//! no whole-file buffering at any stage, and `.csv.gz`/`.csv.zst`/`.csv.bz2` reuse
//! `open_reader`'s transparent streaming decompression.
//!
//! **Direct mapping** (W3C Direct Mapping flavor, IRI rows instead of blank nodes):
//! - table name = the file stem (`people.csv.gz` → `people`), percent-encoded where needed;
//! - subject   = the `--template` IRI template (`{col}` + `{_row}` placeholders; default
//!   `<base><table>/row/{_row}`, `{_row}` = 1-based data-row number);
//! - predicate = `<base><table>#<column>`;
//! - object    = the cell as a literal, with datatype inference (`xsd:integer` / `xsd:decimal`
//!   / `xsd:double` / `xsd:boolean`, else a plain string; `--no-infer` keeps everything a
//!   plain string). An EMPTY cell is NULL: no triple.
//! - each row gets `rdf:type <base><table>` (override with `--class <iri>`, drop with
//!   `--class none`).
//!
//! **R2RML subset** (`--mapping <r2rml.ttl>`): logical tables are CSV files bound by
//! `rr:tableName` = file stem (or an explicit `name=path` positional). Supported:
//! `rr:logicalTable`/`rr:tableName`, `rr:subjectMap`/`rr:subject`, `rr:class`,
//! `rr:predicateObjectMap`, `rr:predicateMap`/`rr:predicate`, `rr:objectMap`/`rr:object`,
//! `rr:template` (IRI-safe percent-encoding of substituted values in IRI maps),
//! `rr:column`, `rr:constant`, `rr:termType` (IRI/Literal/BlankNode), `rr:datatype`,
//! `rr:language`, `rr:parentTriplesMap`/`rr:joinCondition` (+ `rr:child`/`rr:parent`), and
//! `rr:graphMap`/`rr:graph`. Everything else in the `rr:` namespace — notably `rr:sqlQuery`,
//! `rr:sqlVersion`, `rr:inverseExpression` — is a LOUD error (fail-closed), never a silent
//! skip. An empty CSV cell is NULL per R2RML: the term map generates no term and the triple
//! (or the whole row, for a subject map) is skipped. Column-valued literals stay plain
//! strings (CSV's natural datatype) unless the mapping says `rr:datatype`/`rr:language` —
//! datatype inference is a direct-mapping convenience only, never applied to R2RML output.
//!
//! **Joins** ([SONNET-4.6] sq-u1z86): a referencing object map (`rr:objectMap [
//! rr:parentTriplesMap <#Parent> ; rr:joinCondition [ rr:child "c" ; rr:parent "p" ] ]`)
//! emits the PARENT triples map's subject for every parent row whose `rr:parent` cells equal
//! the child row's `rr:child` cells. HONEST COST: the parent table is read once up front and
//! its generated subjects are held in an in-memory keyed index (only the subject terms, never
//! the rows) — the CHILD table still streams. A NULL (empty) cell on either side never joins,
//! per SQL/R2RML NULL semantics; with ZERO join conditions the spec's cross product applies,
//! so every parent subject is emitted for every child row. Only the parent's SUBJECT map is
//! evaluated (never its predicate-object maps), so indexes cannot recurse.
//!
//! **Named graphs** ([SONNET-4.6] sq-u1z86): `rr:graphMap`/`rr:graph` on a subject map or a
//! predicate-object map route statements to named graphs. A statement's graph set is the
//! union of its subject map's and its predicate-object map's graph maps; an empty union — or
//! the explicit `rr:defaultGraph` constant — means the default graph. When a mapping uses any
//! graph map the emitter switches from N-Triples to **N-Quads** and the in-process load goes
//! through `Graph::load_dataset` so `GRAPH ?g { … }` sees the named graphs. `--out` stays
//! fully streaming; the `--query` dataset path buffers the generated N-Quads, because
//! sparq-core's dataset loader is whole-text (the N-Triples path is unaffected).
//!
//! **Row provenance** (`--provenance`, both modes): each generated row subject also gets
//! `<sparq:row> "<n>"^^xsd:integer` and `<sparq:table> "<table>"` in the subject's graph(s),
//! under sparq's own [`TABULAR_NS`] vocabulary (deliberately not `--base`, so provenance is
//! always distinguishable from mapped data).
//!
//! SQL-connection R2RML is explicitly OUT of scope — virtualization is a non-goal; sparq's
//! counter-story is materializing import. A GUI import wizard lives on the site surface, not
//! here.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::time::Instant;

const RR: &str = "http://www.w3.org/ns/r2rml#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
/// `rr:defaultGraph` — the graph-map constant that names the DEFAULT graph rather than a
/// named one (R2RML §9); it is the one `rr:` IRI that is legal as a graph term.
const RR_DEFAULT_GRAPH: &str = "http://www.w3.org/ns/r2rml#defaultGraph";
/// [SONNET-4.6] (sq-u1z86) Row-provenance vocabulary for `--provenance`. Deliberately NOT the
/// mapping's `--base`: provenance triples must stay distinguishable from mapped data.
const TABULAR_NS: &str = "https://sparq.dev/ns/tabular#";

// ---------------------------------------------------------------------------------------------
// Streaming RFC-4180 CSV reader
// ---------------------------------------------------------------------------------------------

/// Streaming RFC-4180 CSV record reader over any `Read` — quoted fields (embedded separators,
/// quotes-as-`""`, embedded newlines), CRLF or LF record ends, optional UTF-8 BOM, and a
/// configurable single-byte separator. STRICT + loud: a stray quote, a bare CR, an unterminated
/// quoted field, or non-UTF-8 bytes are errors, never silent coercions. Fully-empty lines are
/// skipped (they carry no record). Never buffers more than the internal 64 KiB block + the
/// current record.
struct CsvRows<R: Read> {
    src: R,
    buf: Vec<u8>,
    pos: usize,
    len: usize,
    sep: u8,
    bom_done: bool,
    pushback: Vec<u8>,
    eof: bool,
    failed: bool,
}

impl<R: Read> CsvRows<R> {
    fn new(src: R, sep: u8) -> Self {
        CsvRows {
            src,
            buf: vec![0u8; 64 * 1024],
            pos: 0,
            len: 0,
            sep,
            bom_done: false,
            pushback: Vec::new(),
            eof: false,
            failed: false,
        }
    }

    /// Next raw byte from the buffered source (`None` = EOF).
    fn raw_byte(&mut self) -> Result<Option<u8>, String> {
        loop {
            if self.pos < self.len {
                let b = self.buf[self.pos];
                self.pos += 1;
                return Ok(Some(b));
            }
            if self.eof {
                return Ok(None);
            }
            self.len = self.src.read(&mut self.buf).map_err(|e| format!("CSV read error: {e}"))?;
            self.pos = 0;
            if self.len == 0 {
                self.eof = true;
            }
        }
    }

    /// Next byte after one-shot UTF-8 BOM stripping at stream start.
    fn next_byte(&mut self) -> Result<Option<u8>, String> {
        if !self.bom_done {
            self.bom_done = true;
            let mut head: Vec<u8> = Vec::with_capacity(3);
            while head.len() < 3 {
                match self.raw_byte()? {
                    Some(b) => head.push(b),
                    None => break,
                }
            }
            if head.as_slice() != [0xEF, 0xBB, 0xBF] {
                self.pushback = head;
            }
        }
        if !self.pushback.is_empty() {
            return Ok(Some(self.pushback.remove(0)));
        }
        self.raw_byte()
    }

    /// `\r` must be followed by `\n` (RFC 4180 CRLF); a bare CR is a loud error.
    fn expect_lf(&mut self) -> Result<(), String> {
        match self.next_byte()? {
            Some(b'\n') => Ok(()),
            _ => Err("bare CR (a \\r not followed by \\n) in CSV input".into()),
        }
    }

    /// Parse one record. `Ok(None)` = clean EOF.
    fn record(&mut self) -> Result<Option<Vec<String>>, String> {
        #[derive(PartialEq)]
        enum S {
            Start,
            Unquoted,
            Quoted,
            QuoteQuote,
        }
        fn finish(fields: &mut Vec<String>, field: &mut Vec<u8>) -> Result<(), String> {
            let bytes = std::mem::take(field);
            fields
                .push(String::from_utf8(bytes).map_err(|_| "CSV field is not valid UTF-8".to_string())?);
            Ok(())
        }
        let mut fields: Vec<String> = Vec::new();
        let mut field: Vec<u8> = Vec::new();
        let mut any = false;
        let mut s = S::Start;
        loop {
            let Some(b) = self.next_byte()? else {
                return match s {
                    S::Quoted => Err("unterminated quoted CSV field at end of input".into()),
                    _ => {
                        if !any {
                            return Ok(None);
                        }
                        finish(&mut fields, &mut field)?;
                        Ok(Some(fields))
                    }
                };
            };
            any = true;
            match s {
                S::Start => {
                    if b == self.sep {
                        finish(&mut fields, &mut field)?;
                    } else {
                        match b {
                            b'"' => s = S::Quoted,
                            b'\n' => {
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            b'\r' => {
                                self.expect_lf()?;
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            _ => {
                                field.push(b);
                                s = S::Unquoted;
                            }
                        }
                    }
                }
                S::Unquoted => {
                    if b == self.sep {
                        finish(&mut fields, &mut field)?;
                        s = S::Start;
                    } else {
                        match b {
                            b'\n' => {
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            b'\r' => {
                                self.expect_lf()?;
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            b'"' => return Err("stray '\"' inside an unquoted CSV field".into()),
                            _ => field.push(b),
                        }
                    }
                }
                S::Quoted => match b {
                    b'"' => s = S::QuoteQuote,
                    _ => field.push(b),
                },
                S::QuoteQuote => {
                    if b == self.sep {
                        finish(&mut fields, &mut field)?;
                        s = S::Start;
                    } else {
                        match b {
                            b'"' => {
                                field.push(b'"');
                                s = S::Quoted;
                            }
                            b'\n' => {
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            b'\r' => {
                                self.expect_lf()?;
                                finish(&mut fields, &mut field)?;
                                return Ok(Some(fields));
                            }
                            _ => {
                                return Err(format!(
                                    "unexpected byte 0x{b:02x} after the closing '\"' of a quoted CSV field"
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<R: Read> Iterator for CsvRows<R> {
    type Item = Result<Vec<String>, String>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        loop {
            match self.record() {
                Ok(Some(rec)) => {
                    // A fully-empty line is not a record.
                    if rec.len() == 1 && rec[0].is_empty() {
                        continue;
                    }
                    return Some(Ok(rec));
                }
                Ok(None) => return None,
                Err(e) => {
                    self.failed = true;
                    return Some(Err(e));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// RDF term generation
// ---------------------------------------------------------------------------------------------

/// A generated RDF term, ready for N-Triples serialisation.
#[derive(Clone, Debug, PartialEq)]
enum GenTerm {
    Iri(String),
    Blank(String),
    Lit { value: String, datatype: Option<String>, lang: Option<String> },
}

/// N-Triples serialisation of one term.
fn nt_term(t: &GenTerm) -> String {
    match t {
        GenTerm::Iri(i) => format!("<{i}>"),
        GenTerm::Blank(l) => format!("_:{l}"),
        GenTerm::Lit { value, datatype, lang } => {
            let mut s = format!("\"{}\"", escape_literal(value));
            if let Some(l) = lang {
                s.push('@');
                s.push_str(l);
            } else if let Some(dt) = datatype {
                s.push_str("^^<");
                s.push_str(dt);
                s.push('>');
            }
            s
        }
    }
}

/// N-Triples literal escaping: `\` `"` `\n` `\r` `\t` plus `\u00XX` for other C0 controls.
fn escape_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// R2RML "IRI-safe" percent-encoding of a template-substituted value: keep iunreserved
/// (ALPHA / DIGIT / `-` `.` `_` `~` and non-ASCII), percent-encode every other byte.
fn iri_safe(v: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(v.len());
    for &b in v.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') || b >= 0x80 {
            out.push(b);
        } else {
            out.extend_from_slice(format!("%{b:02X}").as_bytes());
        }
    }
    // Kept bytes are the original (valid) UTF-8 sequences; escapes are ASCII.
    String::from_utf8(out).expect("iri_safe preserves UTF-8 validity")
}

/// Minimal absolute-IRI sanity check: a scheme, and none of the characters N-Triples /
/// RFC 3987 forbid raw inside `<...>`. Loud error, never a silently-broken output file.
fn check_iri(iri: &str) -> Result<(), String> {
    let absolute = iri
        .split_once(':')
        .map(|(scheme, _)| {
            !scheme.is_empty()
                && scheme.as_bytes()[0].is_ascii_alphabetic()
                && scheme.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        })
        .unwrap_or(false);
    if !absolute {
        return Err(format!("generated IRI {iri:?} is not absolute (no scheme; pass --base to resolve relative IRIs)"));
    }
    if let Some(bad) = iri.chars().find(|&c| (c as u32) <= 0x20 || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\')) {
        return Err(format!("generated IRI {iri:?} contains forbidden character {bad:?}"));
    }
    Ok(())
}

/// Deterministic, injective blank-node label from a value: ASCII alphanumerics kept,
/// every other byte (including `_`) becomes `_HH` (hex) — two distinct values can never
/// collide on a label.
fn blank_label(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for &b in v.as_bytes() {
        if b.is_ascii_alphanumeric() {
            out.push(b as char);
        } else {
            out.push_str(&format!("_{b:02X}"));
        }
    }
    if out.is_empty() {
        out.push_str("_empty");
    }
    out
}

/// Direct-mapping datatype inference over a cell's lexical form. Returns the XSD datatype
/// IRI, or `None` for a plain string. Deliberately conservative: `INF`/`NaN` and exotic
/// numeric forms stay strings.
fn infer_datatype(v: &str) -> Option<&'static str> {
    if v == "true" || v == "false" {
        return Some("http://www.w3.org/2001/XMLSchema#boolean");
    }
    let unsigned = v.strip_prefix(['+', '-']).unwrap_or(v);
    if !unsigned.is_empty() && unsigned.bytes().all(|b| b.is_ascii_digit()) {
        return Some("http://www.w3.org/2001/XMLSchema#integer");
    }
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (unsigned, None),
    };
    let mantissa_ok = match mantissa.split_once('.') {
        Some((i, f)) => {
            (!i.is_empty() || !f.is_empty())
                && i.bytes().all(|b| b.is_ascii_digit())
                && f.bytes().all(|b| b.is_ascii_digit())
        }
        None => !mantissa.is_empty() && mantissa.bytes().all(|b| b.is_ascii_digit()),
    };
    if !mantissa_ok {
        return None;
    }
    match exponent {
        None => {
            // No exponent: mantissa_ok + a '.' means xsd:decimal (pure digits handled above).
            if mantissa.contains('.') {
                Some("http://www.w3.org/2001/XMLSchema#decimal")
            } else {
                None
            }
        }
        Some(e) => {
            let e = e.strip_prefix(['+', '-']).unwrap_or(e);
            if !e.is_empty() && e.bytes().all(|b| b.is_ascii_digit()) {
                Some("http://www.w3.org/2001/XMLSchema#double")
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Compiled term maps
// ---------------------------------------------------------------------------------------------

/// One template piece: literal text, a column reference (by resolved index), or the
/// 1-based data-row number (`{_row}`, direct mapping only).
#[derive(Clone, Debug)]
enum TplPart {
    Text(String),
    Col(usize),
    Row,
}

/// Where a term's lexical value comes from.
#[derive(Clone, Debug)]
enum ValueSpec {
    Constant(GenTerm),
    Column(usize),
    Template(Vec<TplPart>),
}

/// What kind of term the value becomes.
#[derive(Clone, Debug, PartialEq)]
enum TermKind {
    Iri,
    Blank,
    LitPlain,
    LitTyped(String),
    LitLang(String),
    /// Direct-mapping convenience: per-cell datatype inference. Never produced from R2RML.
    LitInfer,
}

#[derive(Clone, Debug)]
struct TermSpec {
    value: ValueSpec,
    kind: TermKind,
}

/// [SONNET-4.6] (sq-u1z86) A compiled `rr:parentTriplesMap` reference (an R2RML *referencing
/// object map*): the parent triples map's generated SUBJECT terms, indexed by the
/// `rr:joinCondition` key so the child table can still stream row-by-row. Only the parent's
/// subject map is ever evaluated — never its predicate-object maps — so index construction
/// cannot recurse, even for a self-join.
#[derive(Clone, Debug)]
struct JoinIndex {
    /// Child-side key columns (indices into the CHILD header), in `rr:joinCondition` order.
    child_cols: Vec<usize>,
    /// Join key → the parent subjects it selects, deduplicated in first-seen order.
    keyed: HashMap<Vec<String>, Vec<GenTerm>>,
    /// With ZERO join conditions R2RML specifies the full cross product: EVERY parent subject
    /// joins every child row. Only populated in that case (`child_cols` empty).
    all: Vec<GenTerm>,
}

impl JoinIndex {
    /// The parent subjects one child row joins to (empty = this row contributes no triple).
    fn lookup(&self, row: &[String]) -> &[GenTerm] {
        if self.child_cols.is_empty() {
            return &self.all;
        }
        let mut key: Vec<String> = Vec::with_capacity(self.child_cols.len());
        for &i in &self.child_cols {
            // A NULL (empty) cell never joins — SQL/R2RML NULL semantics.
            if row[i].is_empty() {
                return &[];
            }
            key.push(row[i].clone());
        }
        self.keyed.get(&key).map_or(&[][..], Vec::as_slice)
    }
}

/// The object side of a predicate-object map: an ordinary term map, or a referencing object
/// map that resolves through a [`JoinIndex`] into zero or more parent subjects.
#[derive(Clone, Debug)]
enum ObjectSpec {
    Term(TermSpec),
    Join(JoinIndex),
}

/// One `rr:predicateObjectMap`: every predicate × every object (the R2RML cartesian rule).
#[derive(Clone, Debug)]
struct CompiledPom {
    predicates: Vec<TermSpec>,
    objects: Vec<ObjectSpec>,
    /// `rr:graphMap`/`rr:graph` on this predicate-object map; unioned with the subject map's.
    graphs: Vec<TermSpec>,
}

/// A fully header-resolved mapping for ONE logical table (one CSV file).
#[derive(Clone, Debug)]
struct CompiledMap {
    subject: TermSpec,
    classes: Vec<String>,
    /// `rr:graphMap`/`rr:graph` on the subject map: applies to the `rr:class` triples, the
    /// row-provenance triples, and (unioned) to every predicate-object map's statements.
    subject_graphs: Vec<TermSpec>,
    poms: Vec<CompiledPom>,
    /// Resolve relative generated IRIs against this (simple prefix concatenation).
    base: Option<String>,
    /// Expected record width (= header width); a ragged row is a loud error.
    width: usize,
    /// `Some(table name)` under `--provenance`: emit the row/table provenance triples.
    provenance: Option<String>,
}

impl CompiledMap {
    /// Whether this map can route a statement to a NAMED graph — i.e. whether the emitted
    /// document is N-Quads rather than N-Triples.
    fn emits_quads(&self) -> bool {
        !self.subject_graphs.is_empty() || self.poms.iter().any(|p| !p.graphs.is_empty())
    }
}

/// Resolve a column reference against the header: a `"quoted"` (delimited) name matches
/// exactly after unquoting; a bare name matches exactly first, then case-insensitively
/// (SQL unquoted-identifier semantics) — ambiguity is an error.
fn resolve_col(name: &str, header: &[String]) -> Result<usize, String> {
    let delimited = name.len() >= 2 && name.starts_with('"') && name.ends_with('"');
    let bare = if delimited { &name[1..name.len() - 1] } else { name };
    if let Some(i) = header.iter().position(|h| h == bare) {
        return Ok(i);
    }
    if !delimited {
        let ci: Vec<usize> =
            (0..header.len()).filter(|&i| header[i].eq_ignore_ascii_case(bare)).collect();
        match ci.len() {
            1 => return Ok(ci[0]),
            n if n > 1 => return Err(format!("column reference {name:?} is ambiguous (case-insensitive) in header {header:?}")),
            _ => {}
        }
    }
    Err(format!("column {name:?} not found in CSV header {header:?}"))
}

/// Parse an R2RML-style string template: `{column}` references, `\{` `\}` `\\` escapes.
/// `{_row}` (the 1-based data-row number) is only legal where `allow_row` is set (the
/// direct-mapping subject template) — R2RML templates stay spec-clean.
fn parse_template(t: &str, header: &[String], allow_row: bool) -> Result<Vec<TplPart>, String> {
    let mut parts: Vec<TplPart> = Vec::new();
    let mut text = String::new();
    let mut chars = t.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(e @ ('{' | '}' | '\\')) => text.push(e),
                Some(other) => return Err(format!("invalid escape '\\{other}' in template {t:?}")),
                None => return Err(format!("dangling '\\' at end of template {t:?}")),
            },
            '{' => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(ch) => name.push(ch),
                        None => return Err(format!("unclosed '{{' in template {t:?}")),
                    }
                }
                if !text.is_empty() {
                    parts.push(TplPart::Text(std::mem::take(&mut text)));
                }
                if name == "_row" {
                    if !allow_row {
                        return Err("{_row} is a direct-mapping placeholder; it is not available in R2RML templates".into());
                    }
                    parts.push(TplPart::Row);
                } else {
                    parts.push(TplPart::Col(resolve_col(&name, header)?));
                }
            }
            '}' => return Err(format!("unbalanced '}}' in template {t:?}")),
            c => text.push(c),
        }
    }
    if !text.is_empty() {
        parts.push(TplPart::Text(text));
    }
    if parts.is_empty() {
        return Err("empty template".into());
    }
    Ok(parts)
}

/// Generate the term a spec produces for one row. `Ok(None)` = NULL (an empty referenced
/// cell): the triple — or the whole row, for a subject — is skipped, per R2RML.
fn gen_term(
    spec: &TermSpec,
    row: &[String],
    row_num: u64,
    base: Option<&str>,
) -> Result<Option<GenTerm>, String> {
    let raw = match &spec.value {
        ValueSpec::Constant(t) => return Ok(Some(t.clone())),
        ValueSpec::Column(i) => {
            let v = &row[*i];
            if v.is_empty() {
                return Ok(None);
            }
            v.clone()
        }
        ValueSpec::Template(parts) => {
            // IRI term maps percent-encode each SUBSTITUTED value (never the fixed text);
            // literal/blank-node maps substitute raw values.
            let encode = spec.kind == TermKind::Iri;
            let mut out = String::new();
            for p in parts {
                match p {
                    TplPart::Text(t) => out.push_str(t),
                    TplPart::Row => out.push_str(&row_num.to_string()),
                    TplPart::Col(i) => {
                        let v = &row[*i];
                        if v.is_empty() {
                            return Ok(None);
                        }
                        if encode {
                            out.push_str(&iri_safe(v));
                        } else {
                            out.push_str(v);
                        }
                    }
                }
            }
            out
        }
    };
    Ok(Some(match &spec.kind {
        TermKind::Iri => {
            let iri = if raw.contains(':') || base.is_none() {
                raw
            } else {
                format!("{}{raw}", base.unwrap_or_default())
            };
            check_iri(&iri)?;
            GenTerm::Iri(iri)
        }
        TermKind::Blank => GenTerm::Blank(blank_label(&raw)),
        TermKind::LitPlain => GenTerm::Lit { value: raw, datatype: None, lang: None },
        TermKind::LitTyped(dt) => GenTerm::Lit { value: raw, datatype: Some(dt.clone()), lang: None },
        TermKind::LitLang(l) => GenTerm::Lit { value: raw, datatype: None, lang: Some(l.clone()) },
        TermKind::LitInfer => {
            let dt = infer_datatype(&raw).map(str::to_owned);
            GenTerm::Lit { value: raw, datatype: dt, lang: None }
        }
    }))
}

/// [SONNET-4.6] (sq-u1z86) Resolve one row's graph targets for a set of graph maps. Each entry
/// is an N-Triples `<iri>` form, or the EMPTY string standing for the default graph (what
/// `rr:defaultGraph` names). `Ok(Some(set))` with an EMPTY set means "no graph maps at this
/// position at all". `Ok(None)` means graph maps ARE present here but every one of them
/// generated NULL, so this position contributes no graph term (R2RML NULL semantics). The two
/// cases are distinct and [`graph_union`] — not this function — decides what the difference
/// means for a given statement.
fn gen_graphs(
    specs: &[TermSpec],
    row: &[String],
    row_num: u64,
    base: Option<&str>,
) -> Result<Option<Vec<String>>, String> {
    if specs.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let mut out: Vec<String> = Vec::with_capacity(specs.len());
    for spec in specs {
        let Some(t) = gen_term(spec, row, row_num, base)? else { continue };
        let g = match t {
            GenTerm::Iri(iri) if iri == RR_DEFAULT_GRAPH => String::new(),
            GenTerm::Iri(iri) => format!("<{iri}>"),
            other => {
                return Err(format!("graph map generated a non-IRI term {other:?} on row {row_num}"))
            }
        };
        if !out.contains(&g) {
            out.push(g);
        }
    }
    if out.is_empty() {
        return Ok(None); // every graph map was NULL
    }
    Ok(Some(out))
}

/// Append one statement to each of `graphs` (an empty graph string = the default graph, which
/// is exactly the N-Triples line an N-Quads document also accepts).
fn write_stmt(out: &mut String, s_nt: &str, p_nt: &str, o_nt: &str, graphs: &[String]) {
    for g in graphs {
        out.push_str(s_nt);
        out.push(' ');
        out.push_str(p_nt);
        out.push(' ');
        out.push_str(o_nt);
        if !g.is_empty() {
            out.push(' ');
            out.push_str(g);
        }
        out.push_str(" .\n");
    }
}

/// The graph set of one statement: the union of the subject-map graph terms and the
/// predicate-object-map graph terms (R2RML §11 step 8), where `None` on either side is
/// [`gen_graphs`]'s "graph maps present here, but all of them generated NULL".
///
/// NULL is PER-POSITION: it removes only that position's contribution, so a NULL subject graph
/// map can never discard a graph the predicate-object map generated, nor the other way round.
/// `None` is returned — the statement is not generated at all — only when the union is empty
/// *and* at least one position carried a graph map, i.e. every graph map that applies to this
/// statement generated NULL. The default graph is reached only when NEITHER position carries a
/// graph map.
fn graph_union(subject: Option<&[String]>, pom: Option<&[String]>) -> Option<Vec<String>> {
    let mut gs: Vec<String> = subject.unwrap_or_default().to_vec();
    for g in pom.unwrap_or_default() {
        if !gs.contains(g) {
            gs.push(g.clone());
        }
    }
    if !gs.is_empty() {
        return Some(gs);
    }
    if subject.is_none() || pom.is_none() {
        return None; // a graph map applied and generated NULL: drop rather than default.
    }
    Some(vec![String::new()])
}

/// Emit the N-Triples (or, with graph maps, N-Quads) lines one CSV row generates under a
/// compiled map.
fn emit_row(map: &CompiledMap, row: &[String], row_num: u64, out: &mut String) -> Result<(), String> {
    if row.len() != map.width {
        return Err(format!(
            "ragged CSV row {row_num}: {} field(s), header has {}",
            row.len(),
            map.width
        ));
    }
    let base = map.base.as_deref();
    let Some(subj) = gen_term(&map.subject, row, row_num, base)? else {
        return Ok(()); // NULL subject: the whole row generates nothing.
    };
    if matches!(subj, GenTerm::Lit { .. }) {
        return Err(format!("subject map generated a literal on row {row_num}"));
    }
    let s_nt = nt_term(&subj);
    let subject_graphs = gen_graphs(&map.subject_graphs, row, row_num, base)?;
    // The `rr:class` and provenance statements live in the subject map's graphs alone, so they
    // are dropped when those all generated NULL — but the row's predicate-object maps carry
    // their own graph maps and are still evaluated below.
    if let Some(s_graphs) = graph_union(subject_graphs.as_deref(), Some(&[])) {
        for cls in &map.classes {
            write_stmt(out, &s_nt, &format!("<{RDF_TYPE}>"), &format!("<{cls}>"), &s_graphs);
        }
        if let Some(table) = &map.provenance {
            write_stmt(
                out,
                &s_nt,
                &format!("<{TABULAR_NS}row>"),
                &format!("\"{row_num}\"^^<{XSD_INTEGER}>"),
                &s_graphs,
            );
            write_stmt(
                out,
                &s_nt,
                &format!("<{TABULAR_NS}table>"),
                &format!("\"{}\"", escape_literal(table)),
                &s_graphs,
            );
        }
    }
    for pom in &map.poms {
        let mut preds: Vec<String> = Vec::with_capacity(pom.predicates.len());
        for p in &pom.predicates {
            match gen_term(p, row, row_num, base)? {
                None => {}
                Some(GenTerm::Iri(i)) => preds.push(format!("<{i}>")),
                Some(other) => {
                    return Err(format!("predicate map generated a non-IRI term {other:?} on row {row_num}"))
                }
            }
        }
        if preds.is_empty() {
            continue;
        }
        let pom_graphs = gen_graphs(&pom.graphs, row, row_num, base)?;
        let Some(graphs) = graph_union(subject_graphs.as_deref(), pom_graphs.as_deref()) else {
            continue; // every graph map that applies to these statements generated NULL.
        };
        for o in &pom.objects {
            // One term map yields at most one object; a referencing object map yields one per
            // joined parent subject.
            let objs: Vec<String> = match o {
                ObjectSpec::Term(spec) => match gen_term(spec, row, row_num, base)? {
                    None => continue, // NULL object: skip this triple only.
                    Some(obj) => vec![nt_term(&obj)],
                },
                ObjectSpec::Join(j) => j.lookup(row).iter().map(nt_term).collect(),
            };
            for o_nt in &objs {
                for p_nt in &preds {
                    write_stmt(out, &s_nt, p_nt, o_nt, &graphs);
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Streaming: CSV rows → N-Triples chunks → `Read`
// ---------------------------------------------------------------------------------------------

/// Iterator adapter: each CSV data row becomes one N-Triples (or N-Quads) chunk, possibly
/// empty. Generic over the ROW iterator — not over a `Read` — so it drives either a
/// [`CsvRows`] directly or the boxed stream a [`TableSource`] hands back.
struct MappedRows<I> {
    rows: I,
    map: CompiledMap,
    row_num: u64,
    failed: bool,
}

impl<I: Iterator<Item = Result<Vec<String>, String>>> Iterator for MappedRows<I> {
    type Item = Result<String, String>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        match self.rows.next()? {
            Err(e) => {
                self.failed = true;
                Some(Err(e))
            }
            Ok(row) => {
                self.row_num += 1;
                let mut chunk = String::new();
                match emit_row(&self.map, &row, self.row_num, &mut chunk) {
                    Ok(()) => Some(Ok(chunk)),
                    Err(e) => {
                        self.failed = true;
                        Some(Err(e))
                    }
                }
            }
        }
    }
}

/// A chunk iterator whose items feed a `Read` (what `Graph::load_reader_parallel` and the
/// `--out` copy loop consume). Multiple sources (one per triples map / CSV file) run in
/// sequence. A mapping error surfaces as an `io::Error`, so the ingest fails loudly.
struct NtReader {
    srcs: Vec<Box<dyn Iterator<Item = Result<String, String>> + Send>>,
    idx: usize,
    buf: Vec<u8>,
    pos: usize,
}

impl NtReader {
    fn new(srcs: Vec<Box<dyn Iterator<Item = Result<String, String>> + Send>>) -> Self {
        NtReader { srcs, idx: 0, buf: Vec::new(), pos: 0 }
    }
}

impl Read for NtReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.pos < self.buf.len() {
                let n = (self.buf.len() - self.pos).min(out.len());
                out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
                self.pos += n;
                return Ok(n);
            }
            let Some(src) = self.srcs.get_mut(self.idx) else { return Ok(0) };
            match src.next() {
                None => self.idx += 1,
                Some(Ok(chunk)) => {
                    self.buf = chunk.into_bytes();
                    self.pos = 0;
                }
                Some(Err(e)) => return Err(std::io::Error::other(e)),
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Direct mapping
// ---------------------------------------------------------------------------------------------

/// Direct-mapping knobs (all flag-fed; defaults documented on the module).
struct DirectOpts {
    base: String,
    template: Option<String>,
    /// `None` = default class `<base><table>`; `Some(None)` = `--class none`; `Some(Some(iri))`.
    class: Option<Option<String>>,
    infer: bool,
    /// `--provenance`: also emit the row/table provenance triples.
    provenance: bool,
}

/// Compile the direct mapping of one CSV header.
fn compile_direct(header: &[String], table: &str, opts: &DirectOpts) -> Result<CompiledMap, String> {
    check_header(header)?;
    let t_enc = iri_safe(table);
    let tpl = match &opts.template {
        Some(t) => t.clone(),
        None => format!("{}{t_enc}/row/{{_row}}", opts.base),
    };
    let subject = TermSpec { value: ValueSpec::Template(parse_template(&tpl, header, true)?), kind: TermKind::Iri };
    let classes: Vec<String> = match &opts.class {
        None => vec![format!("{}{t_enc}", opts.base)],
        Some(None) => Vec::new(),
        Some(Some(iri)) => vec![iri.clone()],
    };
    for c in &classes {
        check_iri(c)?;
    }
    let poms = header
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let pred = format!("{}{t_enc}#{}", opts.base, iri_safe(col));
            check_iri(&pred)?;
            Ok(CompiledPom {
                predicates: vec![TermSpec { value: ValueSpec::Constant(GenTerm::Iri(pred)), kind: TermKind::Iri }],
                objects: vec![ObjectSpec::Term(TermSpec {
                    value: ValueSpec::Column(i),
                    kind: if opts.infer { TermKind::LitInfer } else { TermKind::LitPlain },
                })],
                graphs: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(CompiledMap {
        subject,
        classes,
        subject_graphs: Vec::new(),
        poms,
        base: Some(opts.base.clone()),
        width: header.len(),
        provenance: opts.provenance.then(|| table.to_owned()),
    })
}

/// Header sanity: no empty and no duplicate column names (both would map ambiguously).
fn check_header(header: &[String]) -> Result<(), String> {
    for (i, h) in header.iter().enumerate() {
        if h.is_empty() {
            return Err(format!("CSV header column {} is empty", i + 1));
        }
    }
    for i in 0..header.len() {
        if header[i + 1..].contains(&header[i]) {
            return Err(format!("duplicate CSV header column {:?}", header[i]));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// R2RML (materializing subset over CSV logical tables)
// ---------------------------------------------------------------------------------------------

/// The mapping document as a subject→(predicate, object) adjacency over full IRIs — small
/// (a mapping file), so materialising it wholesale via one engine query is fine.
struct RrGraph {
    adj: HashMap<String, Vec<(String, oxrdf::Term)>>,
}

/// The `rr:` local names this subset implements. Any OTHER `rr:` predicate in the mapping is
/// a loud error (fail-closed): `rr:sqlQuery`, `rr:sqlVersion`, `rr:inverseExpression`, … are
/// all rejected, never silently skipped.
const RR_SUPPORTED: &[&str] = &[
    "logicalTable",
    "tableName",
    "subjectMap",
    "subject",
    "class",
    "termType",
    "template",
    "column",
    "constant",
    "predicateObjectMap",
    "predicateMap",
    "predicate",
    "objectMap",
    "object",
    "datatype",
    "language",
    // [SONNET-4.6] (sq-u1z86) referencing object maps (cross-CSV joins) …
    "parentTriplesMap",
    "joinCondition",
    "child",
    "parent",
    // … and named-graph output.
    "graphMap",
    "graph",
];

impl RrGraph {
    /// Values of `pred` on `node` (node key = the term's N-Triples-ish display form).
    fn props<'a>(&'a self, node: &str, pred: &str) -> impl Iterator<Item = &'a oxrdf::Term> {
        let full = format!("{RR}{pred}");
        self.adj.get(node).into_iter().flatten().filter_map(move |(p, o)| (p == &full).then_some(o))
    }

    /// At most one value of `pred` on `node` (two is a mapping error).
    fn prop1(&self, node: &str, pred: &str) -> Result<Option<&oxrdf::Term>, String> {
        let mut it = self.props(node, pred);
        let first = it.next();
        if it.next().is_some() {
            return Err(format!("term map {node} has more than one rr:{pred}"));
        }
        Ok(first)
    }
}

/// A parsed R2RML mapping: the adjacency + each triples map's node key and CSV table name.
struct R2rmlMapping {
    rr: RrGraph,
    /// (triples-map node key, logical table name — surrounding `"` stripped).
    tms: Vec<(String, String)>,
}

impl R2rmlMapping {
    /// The logical table a triples map reads, by node key.
    fn table_of(&self, tm_key: &str) -> Option<&str> {
        self.tms.iter().find(|(k, _)| k == tm_key).map(|(_, t)| t.as_str())
    }
}

/// One logical table's rows: the header, plus a still-streaming record iterator.
type TableRows = (Vec<String>, Box<dyn Iterator<Item = Result<Vec<String>, String>>>);

/// [SONNET-4.6] (sq-u1z86) Resolves an R2RML logical-table name to a FRESHLY-opened CSV row
/// stream. The indirection exists for `rr:parentTriplesMap`: building a join index has to read
/// the parent table independently of (and possibly identically to — a self-join) the child
/// stream the compiler was handed. It also lets the unit tests bind in-memory CSVs.
trait TableSource {
    fn open_table(&self, table: &str) -> Result<TableRows, String>;
}

fn term_key(t: &oxrdf::Term) -> String {
    t.to_string()
}

fn lit_value(t: &oxrdf::Term, what: &str) -> Result<String, String> {
    match t {
        oxrdf::Term::Literal(l) => Ok(l.value().to_owned()),
        other => Err(format!("{what} must be a literal, got {other}")),
    }
}

fn iri_value(t: &oxrdf::Term, what: &str) -> Result<String, String> {
    match t {
        oxrdf::Term::NamedNode(n) => Ok(n.as_str().to_owned()),
        other => Err(format!("{what} must be an IRI, got {other}")),
    }
}

/// Parse an R2RML mapping document (Turtle) into [`R2rmlMapping`], failing loudly on any
/// `rr:` construct outside the supported materializing-CSV subset.
fn parse_r2rml(mapping_ttl: &str) -> Result<R2rmlMapping, String> {
    // Resolve relative IRIs (the conventional `<#TriplesMap1>` style) against the same base
    // the W3C R2RML test suite uses; map-node identity is internal, so the exact base is
    // immaterial to the generated triples.
    let g = sparq_core::Graph::load_str_with_base(mapping_ttl, "turtle", "http://example.com/base/")
        .map_err(|e| format!("parsing R2RML mapping: {e}"))?;
    let res = sparq_engine::query(&g, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
        .map_err(|e| format!("reading R2RML mapping graph: {e}"))?;
    let mut adj: HashMap<String, Vec<(String, oxrdf::Term)>> = HashMap::new();
    for row in &res.rows {
        let (Some(Some(s)), Some(Some(p)), Some(Some(o))) = (row.first(), row.get(1), row.get(2)) else {
            return Err("unexpected unbound value in mapping graph".into());
        };
        let p_iri = iri_value(p, "predicate")?;
        if let Some(local) = p_iri.strip_prefix(RR) {
            if !RR_SUPPORTED.contains(&local) {
                return Err(format!(
                    "unsupported R2RML construct rr:{local} — this materializing subset covers CSV logical tables only \
                     (no rr:sqlQuery / rr:sqlVersion / rr:inverseExpression: SQL virtualization is a non-goal)"
                ));
            }
        }
        adj.entry(term_key(s)).or_default().push((p_iri, o.clone()));
    }
    let rr = RrGraph { adj };
    // A triples map is any resource with an rr:logicalTable (R2RML's structural definition).
    let mut tms: Vec<(String, String)> = Vec::new();
    let mut keys: Vec<&String> = rr.adj.keys().collect();
    keys.sort(); // deterministic emission order
    for node in keys {
        let Some(lt) = rr.prop1(node, "logicalTable")? else { continue };
        let lt_key = term_key(lt);
        let name = rr
            .prop1(&lt_key, "tableName")?
            .ok_or_else(|| format!("logical table of {node} has no rr:tableName (CSV logical tables only)"))?;
        let name = lit_value(name, "rr:tableName")?;
        let name = name.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(&name).to_owned();
        tms.push((node.clone(), name));
    }
    Ok(R2rmlMapping { rr, tms })
}

/// Human name of a term-map position, for error messages.
fn pos_name(pos: u8) -> &'static str {
    match pos {
        0 => "subject",
        1 => "predicate",
        2 => "object",
        _ => "graph",
    }
}

/// Build the [`TermSpec`] of one term-map node. `pos`: 0 = subject, 1 = predicate, 2 = object,
/// 3 = graph (drives the R2RML default term type + which properties are legal).
fn term_spec(
    rr: &RrGraph,
    node: &str,
    pos: u8,
    header: &[String],
) -> Result<TermSpec, String> {
    let template = rr.prop1(node, "template")?.map(|t| lit_value(t, "rr:template")).transpose()?;
    let column = rr.prop1(node, "column")?.map(|t| lit_value(t, "rr:column")).transpose()?;
    let constant = rr.prop1(node, "constant")?.cloned();
    let n_kinds = [template.is_some(), column.is_some(), constant.is_some()].iter().filter(|b| **b).count();
    if n_kinds != 1 {
        return Err(format!("term map {node} must have exactly one of rr:template / rr:column / rr:constant (found {n_kinds})"));
    }
    let datatype = rr.prop1(node, "datatype")?.map(|t| iri_value(t, "rr:datatype")).transpose()?;
    let language = rr.prop1(node, "language")?.map(|t| lit_value(t, "rr:language")).transpose()?;
    if datatype.is_some() && language.is_some() {
        return Err(format!("term map {node} has both rr:datatype and rr:language"));
    }
    let term_type = match rr.prop1(node, "termType")? {
        None => {
            // R2RML §7.4.1 defaults: object position is a Literal iff the map is
            // column-valued or carries rr:datatype/rr:language; everything else is an IRI.
            if pos == 2 && (column.is_some() || datatype.is_some() || language.is_some()) {
                "Literal".to_owned()
            } else {
                "IRI".to_owned()
            }
        }
        Some(t) => {
            let t = iri_value(t, "rr:termType")?;
            t.strip_prefix(RR).ok_or_else(|| format!("invalid rr:termType {t}"))?.to_owned()
        }
    };
    if (datatype.is_some() || language.is_some()) && term_type != "Literal" {
        return Err(format!("term map {node}: rr:datatype/rr:language require rr:termType rr:Literal"));
    }
    let kind = match term_type.as_str() {
        "IRI" => TermKind::Iri,
        "BlankNode" => {
            // Predicate and graph names must be IRIs (RDF 1.1 / R2RML §7.4.2).
            if pos == 1 || pos == 3 {
                return Err(format!("{} map {node} cannot be a blank node", pos_name(pos)));
            }
            TermKind::Blank
        }
        "Literal" => {
            if pos != 2 {
                return Err(format!("term map {node}: rr:Literal is only valid in object position"));
            }
            if let Some(l) = &language {
                TermKind::LitLang(l.clone())
            } else if let Some(dt) = &datatype {
                if dt == XSD_STRING {
                    TermKind::LitPlain
                } else {
                    TermKind::LitTyped(dt.clone())
                }
            } else {
                TermKind::LitPlain
            }
        }
        other => return Err(format!("unsupported rr:termType rr:{other}")),
    };
    let value = if let Some(t) = template {
        ValueSpec::Template(parse_template(&t, header, false)?)
    } else if let Some(c) = column {
        ValueSpec::Column(resolve_col(&c, header)?)
    } else {
        ValueSpec::Constant(constant_term(&constant.expect("checked above"), pos)?)
    };
    Ok(TermSpec { value, kind })
}

/// Convert an `rr:constant` (or `rr:subject`/`rr:predicate`/`rr:object` shortcut) value.
fn constant_term(t: &oxrdf::Term, pos: u8) -> Result<GenTerm, String> {
    match t {
        oxrdf::Term::NamedNode(n) => Ok(GenTerm::Iri(n.as_str().to_owned())),
        oxrdf::Term::Literal(l) => {
            if pos != 2 {
                return Err(format!("constant literal {l} is only valid in object position"));
            }
            let lang = l.language().map(str::to_owned);
            let dt = if lang.is_some() || l.datatype().as_str() == XSD_STRING {
                None
            } else {
                Some(l.datatype().as_str().to_owned())
            };
            Ok(GenTerm::Lit { value: l.value().to_owned(), datatype: dt, lang })
        }
        other => Err(format!("unsupported constant term {other} (blank-node constants are not shared-scope-safe)")),
    }
}

/// [SONNET-4.6] (sq-u1z86) Compile the `rr:graph` constants + `rr:graphMap` nodes attached to a
/// subject map or a predicate-object map. Order is deterministic (constants first, then maps in
/// sorted node order) so emission is reproducible.
fn graph_specs(rr: &RrGraph, node: &str, header: &[String]) -> Result<Vec<TermSpec>, String> {
    let mut specs: Vec<TermSpec> = Vec::new();
    for g in rr.props(node, "graph") {
        specs.push(TermSpec { value: ValueSpec::Constant(constant_term(g, 3)?), kind: TermKind::Iri });
    }
    let mut gm_keys: Vec<String> = rr.props(node, "graphMap").map(term_key).collect();
    gm_keys.sort();
    for gm_key in gm_keys {
        specs.push(term_spec(rr, &gm_key, 3, header)?);
    }
    Ok(specs)
}

/// The subject side of a triples map: its term spec, its `rr:class` IRIs, and its graph maps.
/// Shared by [`compile_tm`] and [`build_join_index`] (which needs a PARENT triples map's
/// subject and nothing else).
fn subject_of(
    m: &R2rmlMapping,
    tm_key: &str,
    header: &[String],
) -> Result<(TermSpec, Vec<String>, Vec<TermSpec>), String> {
    let rr = &m.rr;
    // Subject: rr:subjectMap node or the rr:subject constant shortcut — exactly one.
    let sm = rr.prop1(tm_key, "subjectMap")?;
    let s_const = rr.prop1(tm_key, "subject")?;
    match (sm, s_const) {
        (Some(sm), None) => {
            let sm_key = term_key(sm);
            let spec = term_spec(rr, &sm_key, 0, header)?;
            let mut classes = Vec::new();
            for c in rr.props(&sm_key, "class") {
                classes.push(iri_value(c, "rr:class")?);
            }
            classes.sort();
            let graphs = graph_specs(rr, &sm_key, header)?;
            Ok((spec, classes, graphs))
        }
        (None, Some(c)) => Ok((
            TermSpec { value: ValueSpec::Constant(constant_term(c, 0)?), kind: TermKind::Iri },
            Vec::new(),
            Vec::new(),
        )),
        (None, None) => Err(format!("triples map {tm_key} has no rr:subjectMap / rr:subject")),
        (Some(_), Some(_)) => Err(format!("triples map {tm_key} has both rr:subjectMap and rr:subject")),
    }
}

/// [SONNET-4.6] (sq-u1z86) Compile a REFERENCING object map (`rr:parentTriplesMap` +
/// `rr:joinCondition`) into a [`JoinIndex`]: read the parent logical table once, generate its
/// subject term per row, and index those subjects by the `rr:parent` key columns.
fn build_join_index(
    m: &R2rmlMapping,
    om_key: &str,
    child_header: &[String],
    base: Option<&str>,
    tables: &dyn TableSource,
) -> Result<JoinIndex, String> {
    let rr = &m.rr;
    // A referencing object map is a term map ONLY through its parent; the value-producing and
    // literal-shaping properties are mutually exclusive with it (R2RML §8.2).
    for banned in ["template", "column", "constant", "datatype", "language", "termType"] {
        if rr.prop1(om_key, banned)?.is_some() {
            return Err(format!(
                "referencing object map {om_key} has both rr:parentTriplesMap and rr:{banned}"
            ));
        }
    }
    let parent = rr.prop1(om_key, "parentTriplesMap")?.expect("caller checked");
    let parent_key = term_key(parent);
    let parent_table = m.table_of(&parent_key).ok_or_else(|| {
        format!("rr:parentTriplesMap {parent_key} is not a triples map with an rr:logicalTable")
    })?;
    let (parent_header, parent_rows) = tables.open_table(parent_table)?;
    check_header(&parent_header)?;

    // Join conditions: each carries exactly one rr:child and one rr:parent column name.
    let mut jc_keys: Vec<String> = rr.props(om_key, "joinCondition").map(term_key).collect();
    jc_keys.sort(); // deterministic composite-key column order
    let mut child_cols: Vec<usize> = Vec::with_capacity(jc_keys.len());
    let mut parent_cols: Vec<usize> = Vec::with_capacity(jc_keys.len());
    for jc in &jc_keys {
        let c = rr
            .prop1(jc, "child")?
            .ok_or_else(|| format!("join condition {jc} has no rr:child"))?;
        let p = rr
            .prop1(jc, "parent")?
            .ok_or_else(|| format!("join condition {jc} has no rr:parent"))?;
        child_cols.push(resolve_col(&lit_value(c, "rr:child")?, child_header)?);
        parent_cols.push(resolve_col(&lit_value(p, "rr:parent")?, &parent_header)?);
    }

    // Materialize the PARENT side only: its generated subjects, keyed. The child table streams.
    let (subject, _, _) = subject_of(m, &parent_key, &parent_header)?;
    let parent_map = CompiledMap {
        subject,
        classes: Vec::new(),
        subject_graphs: Vec::new(),
        poms: Vec::new(),
        base: base.map(str::to_owned),
        width: parent_header.len(),
        provenance: None,
    };
    let mut keyed: HashMap<Vec<String>, Vec<GenTerm>> = HashMap::new();
    let mut all: Vec<GenTerm> = Vec::new();
    // Deduplicate (key, subject) in O(1). A linear `contains` over the accumulated subjects
    // would be QUADRATIC in the parent row count — the cross-product case scans the whole
    // subject list once per parent row — so the seen-set is a correctness-preserving guard
    // against a large parent table quietly becoming the bottleneck. The empty key vector is
    // the cross-product bucket.
    let mut seen: HashSet<(Vec<String>, String)> = HashSet::new();
    let mut row_num: u64 = 0;
    for row in parent_rows {
        let row = row.map_err(|e| format!("parent table {parent_table:?}: {e}"))?;
        row_num += 1;
        if row.len() != parent_map.width {
            return Err(format!(
                "parent table {parent_table:?}: ragged CSV row {row_num}: {} field(s), header has {}",
                row.len(),
                parent_map.width
            ));
        }
        let Some(subj) = gen_term(&parent_map.subject, &row, row_num, base)? else { continue };
        if matches!(subj, GenTerm::Lit { .. }) {
            return Err(format!(
                "parent table {parent_table:?}: subject map generated a literal on row {row_num}"
            ));
        }
        if parent_cols.is_empty() {
            // Zero join conditions: R2RML's cross product — every parent subject joins.
            if seen.insert((Vec::new(), nt_term(&subj))) {
                all.push(subj);
            }
            continue;
        }
        let mut key: Vec<String> = Vec::with_capacity(parent_cols.len());
        let mut null_key = false;
        for &i in &parent_cols {
            if row[i].is_empty() {
                null_key = true; // a NULL key never joins
                break;
            }
            key.push(row[i].clone());
        }
        if null_key {
            continue;
        }
        if seen.insert((key.clone(), nt_term(&subj))) {
            keyed.entry(key).or_default().push(subj);
        }
    }
    Ok(JoinIndex { child_cols, keyed, all })
}

/// Compile one triples map against its CSV header. `tables` resolves any `rr:parentTriplesMap`
/// parent logical table; `provenance` is the table name to stamp under `--provenance`.
fn compile_tm(
    m: &R2rmlMapping,
    tm_key: &str,
    header: &[String],
    base: Option<&str>,
    tables: &dyn TableSource,
    provenance: Option<&str>,
) -> Result<CompiledMap, String> {
    check_header(header)?;
    let rr = &m.rr;
    // Graph maps belong on a subject map or a predicate-object map, never directly on the
    // triples map. Putting one there is an easy slip, so stay fail-closed rather than silently
    // dropping the named-graph routing the mapping author asked for.
    for misplaced in ["graphMap", "graph"] {
        if rr.props(tm_key, misplaced).next().is_some() {
            return Err(format!(
                "triples map {tm_key} carries rr:{misplaced} directly — it belongs on the rr:subjectMap or on an rr:predicateObjectMap"
            ));
        }
    }
    let (subject, classes, subject_graphs) = subject_of(m, tm_key, header)?;
    let mut poms: Vec<CompiledPom> = Vec::new();
    let mut pom_keys: Vec<String> = rr.props(tm_key, "predicateObjectMap").map(term_key).collect();
    pom_keys.sort(); // deterministic emission order
    for pom_key in pom_keys {
        let mut predicates: Vec<TermSpec> = Vec::new();
        for p in rr.props(&pom_key, "predicate") {
            predicates.push(TermSpec { value: ValueSpec::Constant(constant_term(p, 1)?), kind: TermKind::Iri });
        }
        for pm in rr.props(&pom_key, "predicateMap") {
            predicates.push(term_spec(rr, &term_key(pm), 1, header)?);
        }
        let mut objects: Vec<ObjectSpec> = Vec::new();
        for o in rr.props(&pom_key, "object") {
            let c = constant_term(o, 2)?;
            let kind = match &c {
                GenTerm::Iri(_) => TermKind::Iri,
                _ => TermKind::LitPlain, // kind is unused for constants; value wins
            };
            objects.push(ObjectSpec::Term(TermSpec { value: ValueSpec::Constant(c), kind }));
        }
        for om in rr.props(&pom_key, "objectMap") {
            let om_key = term_key(om);
            if rr.prop1(&om_key, "parentTriplesMap")?.is_some() {
                objects.push(ObjectSpec::Join(build_join_index(m, &om_key, header, base, tables)?));
            } else {
                // A join condition with nothing to join to would otherwise be silently dropped.
                if rr.props(&om_key, "joinCondition").next().is_some() {
                    return Err(format!(
                        "object map {om_key} has rr:joinCondition but no rr:parentTriplesMap"
                    ));
                }
                objects.push(ObjectSpec::Term(term_spec(rr, &om_key, 2, header)?));
            }
        }
        if predicates.is_empty() || objects.is_empty() {
            return Err(format!("predicate-object map {pom_key} needs at least one predicate and one object"));
        }
        poms.push(CompiledPom { predicates, objects, graphs: graph_specs(rr, &pom_key, header)? });
    }
    Ok(CompiledMap {
        subject,
        classes,
        subject_graphs,
        poms,
        base: base.map(str::to_owned),
        width: header.len(),
        provenance: provenance.map(str::to_owned),
    })
}

// ---------------------------------------------------------------------------------------------
// CLI driver
// ---------------------------------------------------------------------------------------------

/// The file stem used as the logical table name: file name minus directory, minus one
/// compression extension (`.gz`/`.zst`/`.zstd`/`.bz2`), minus one format extension.
fn table_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    let name = ["gz", "zst", "zstd", "bz2"]
        .iter()
        .find_map(|e| name.strip_suffix(&format!(".{e}")))
        .unwrap_or(name);
    match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem.to_owned(),
        _ => name.to_owned(),
    }
}

fn die(msg: impl std::fmt::Display) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn usage() -> ! {
    eprintln!(
        "usage: sparq-cli tabular <csv[.gz|.zst|.bz2]> [<name>=<csv> ...] [flags]\n\
         \n  direct mapping (default):\
         \n    --base <iri>        vocabulary/subject base (default http://example.com/)\
         \n    --template <tpl>    subject IRI template; {{col}} + {{_row}} placeholders\
         \n                        (default <base><table>/row/{{_row}})\
         \n    --class <iri|none>  per-row rdf:type (default <base><table>)\
         \n    --no-infer          disable datatype inference (all cells plain strings)\
         \n  R2RML:\
         \n    --mapping <r2rml.ttl>  execute the mapping (CSV logical tables bound by\
         \n                        rr:tableName = file stem, or explicit <name>=<csv>)\
         \n                        incl. rr:parentTriplesMap/rr:joinCondition joins and\
         \n                        rr:graphMap/rr:graph named graphs (output becomes N-Quads)\
         \n  common:\
         \n    --provenance        also emit row-provenance triples (<sparq:row>, <sparq:table>)\
         \n    --sep <char|tab>    field separator (default ',')\
         \n    --out <file.nt[.gz|.zst]>  stream N-Triples (N-Quads with graph maps) out\
         \n                        instead of loading\
         \n    --query <sparql> [--format <table|tsv|csv|xml|json|ntriples>] [--count]\
         \n                        run one query over the loaded graph"
    );
    std::process::exit(2);
}

struct Flags {
    files: Vec<(Option<String>, String)>, // (explicit table name, path)
    mapping: Option<String>,
    base: Option<String>,
    template: Option<String>,
    class: Option<String>,
    sep: u8,
    infer: bool,
    provenance: bool,
    out: Option<String>,
    query: Option<String>,
}

fn parse_flags(args: &[String]) -> Flags {
    let mut f = Flags {
        files: Vec::new(),
        mapping: None,
        base: None,
        template: None,
        class: None,
        sep: b',',
        infer: true,
        provenance: false,
        out: None,
        query: None,
    };
    let mut i = 2;
    let value = |args: &[String], i: usize| -> String {
        args.get(i + 1).cloned().unwrap_or_else(|| {
            eprintln!("{} needs a value", args[i]);
            std::process::exit(2);
        })
    };
    while i < args.len() {
        match args[i].as_str() {
            "--mapping" => {
                f.mapping = Some(value(args, i));
                i += 2;
            }
            "--base" => {
                f.base = Some(value(args, i));
                i += 2;
            }
            "--template" => {
                f.template = Some(value(args, i));
                i += 2;
            }
            "--class" => {
                f.class = Some(value(args, i));
                i += 2;
            }
            "--sep" => {
                let v = value(args, i);
                f.sep = match v.as_str() {
                    "tab" | "\\t" => b'\t',
                    s if s.len() == 1 && s.is_ascii() => s.as_bytes()[0],
                    _ => {
                        eprintln!("--sep must be a single ASCII character (or 'tab')");
                        std::process::exit(2);
                    }
                };
                i += 2;
            }
            "--no-infer" => {
                f.infer = false;
                i += 1;
            }
            // [SONNET-4.6] (sq-u1z86) Row provenance applies to BOTH modes, so — unlike
            // --template/--class/--no-infer — it is not rejected alongside --mapping.
            "--provenance" => {
                f.provenance = true;
                i += 1;
            }
            "--out" => {
                f.out = Some(value(args, i));
                i += 2;
            }
            "--query" => {
                f.query = Some(value(args, i));
                i += 2;
            }
            // `--format`/`--count` belong to the query-result emission (read again by
            // `out_format_flag`); just step over them here.
            "--format" => i += 2,
            "--count" => i += 1,
            flag if flag.starts_with("--") => {
                eprintln!("unknown flag {flag}");
                usage();
            }
            positional => {
                match positional.split_once('=') {
                    Some((name, path)) if !name.is_empty() && !path.is_empty() => {
                        f.files.push((Some(name.to_owned()), path.to_owned()));
                    }
                    _ => f.files.push((None, positional.to_owned())),
                }
                i += 1;
            }
        }
    }
    if f.files.is_empty() {
        usage();
    }
    if f.mapping.is_some() && (f.template.is_some() || f.class.is_some() || !f.infer) {
        eprintln!("--template/--class/--no-infer are direct-mapping flags; they do not apply with --mapping");
        std::process::exit(2);
    }
    f
}

/// A CSV row stream over the CLI's transparently-decompressing reader.
type FileRows = CsvRows<Box<dyn Read + Send>>;

/// Open one CSV, read its header, and return (header, the still-streaming row reader).
fn open_csv(path: &str, sep: u8) -> Result<(Vec<String>, FileRows), String> {
    let reader = crate::open_reader(path).map_err(|e| format!("opening {path}: {e}"))?;
    let mut rows = CsvRows::new(reader, sep);
    match rows.next() {
        Some(Ok(header)) => Ok((header, rows)),
        Some(Err(e)) => Err(format!("{path}: {e}")),
        None => Err(format!("{path}: empty CSV (no header row)")),
    }
}

type ChunkIter = Box<dyn Iterator<Item = Result<String, String>> + Send>;

/// [SONNET-4.6] (sq-u1z86) The invocation's CSV files, resolved by logical-table name: the
/// `<name>=<path>` positionals plus file stems. Exact match wins; a unique case-insensitive
/// match is the fallback (SQL unquoted-identifier semantics); ambiguity is a loud error.
struct CsvTables {
    /// (logical table name, path), in command-line order.
    bound: Vec<(String, String)>,
    sep: u8,
}

impl CsvTables {
    fn index_of(&self, table: &str) -> Result<usize, String> {
        let exact: Vec<usize> = (0..self.bound.len()).filter(|&i| self.bound[i].0 == table).collect();
        match exact.len() {
            1 => return Ok(exact[0]),
            0 => {}
            _ => return Err(format!("logical table {table:?} matches multiple CSV files")),
        }
        let ci: Vec<usize> =
            (0..self.bound.len()).filter(|&i| self.bound[i].0.eq_ignore_ascii_case(table)).collect();
        match ci.len() {
            1 => Ok(ci[0]),
            0 => Err(format!(
                "no CSV bound for logical table {table:?} (files: {:?}; bind explicitly with {table}=<path>)",
                self.bound.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
            )),
            _ => Err(format!("logical table {table:?} matches multiple CSV files case-insensitively")),
        }
    }
}

impl TableSource for CsvTables {
    fn open_table(&self, table: &str) -> Result<TableRows, String> {
        let path = &self.bound[self.index_of(table)?].1;
        let (header, rows) = open_csv(path, self.sep)?;
        Ok((header, Box::new(rows)))
    }
}

/// The chunk iterators for the whole invocation, plus whether the generated document is
/// N-Quads (any mapping used `rr:graphMap`/`rr:graph`) rather than N-Triples.
struct Sources {
    srcs: Vec<ChunkIter>,
    quads: bool,
}

/// Assemble the per-file chunk iterators for the whole invocation.
fn build_sources(f: &Flags) -> Result<Sources, String> {
    let mut srcs: Vec<ChunkIter> = Vec::new();
    let mut quads = false;
    match &f.mapping {
        None => {
            let opts = DirectOpts {
                base: f.base.clone().unwrap_or_else(|| "http://example.com/".to_owned()),
                template: f.template.clone(),
                class: match f.class.as_deref() {
                    None => None,
                    Some("none") => Some(None),
                    Some(iri) => Some(Some(iri.to_owned())),
                },
                infer: f.infer,
                provenance: f.provenance,
            };
            for (name, path) in &f.files {
                let table = name.clone().unwrap_or_else(|| table_stem(path));
                let (header, rows) = open_csv(path, f.sep)?;
                let map = compile_direct(&header, &table, &opts).map_err(|e| format!("{path}: {e}"))?;
                srcs.push(Box::new(MappedRows { rows, map, row_num: 0, failed: false }));
            }
        }
        Some(mapping_path) => {
            let mut ttl = String::new();
            crate::open_reader(mapping_path)
                .and_then(|mut r| r.read_to_string(&mut ttl))
                .map_err(|e| format!("reading {mapping_path}: {e}"))?;
            let m = parse_r2rml(&ttl)?;
            // Bind each triples map's table name to a CSV path.
            let tables = CsvTables {
                bound: f
                    .files
                    .iter()
                    .map(|(name, path)| (name.clone().unwrap_or_else(|| table_stem(path)), path.clone()))
                    .collect(),
                sep: f.sep,
            };
            let mut used = vec![false; tables.bound.len()];
            for (tm_key, table) in &m.tms {
                let idx = tables.index_of(table)?;
                used[idx] = true;
                let path = &tables.bound[idx].1;
                let (header, rows) = open_csv(path, f.sep)?;
                let prov = f.provenance.then_some(table.as_str());
                let map = compile_tm(&m, tm_key, &header, f.base.as_deref(), &tables, prov)
                    .map_err(|e| format!("{path}: {e}"))?;
                quads |= map.emits_quads();
                srcs.push(Box::new(MappedRows { rows, map, row_num: 0, failed: false }));
            }
            for (i, (table, path)) in tables.bound.iter().enumerate() {
                if !used[i] {
                    eprintln!("warning: {path} (table {table:?}) matched no triples map — it contributes nothing");
                }
            }
        }
    }
    Ok(Sources { srcs, quads })
}

/// Sink for `--out`: plain, `.gz`, or `.zst` N-Triples — finished EXPLICITLY so a failed
/// trailer write is an error, not a silently-truncated archive.
enum OutSink {
    Plain(std::io::BufWriter<std::fs::File>),
    Gz(flate2::write::GzEncoder<std::io::BufWriter<std::fs::File>>),
    Zst(Box<zstd::stream::write::Encoder<'static, std::io::BufWriter<std::fs::File>>>),
}

impl OutSink {
    fn create(path: &str) -> std::io::Result<OutSink> {
        let w = std::io::BufWriter::new(std::fs::File::create(path)?);
        Ok(if path.ends_with(".gz") {
            OutSink::Gz(flate2::write::GzEncoder::new(w, flate2::Compression::default()))
        } else if path.ends_with(".zst") || path.ends_with(".zstd") {
            OutSink::Zst(Box::new(zstd::stream::write::Encoder::new(w, 0)?))
        } else {
            OutSink::Plain(w)
        })
    }
    fn writer(&mut self) -> &mut dyn Write {
        match self {
            OutSink::Plain(w) => w,
            OutSink::Gz(w) => w,
            OutSink::Zst(w) => w,
        }
    }
    fn finish(self) -> std::io::Result<()> {
        match self {
            OutSink::Plain(mut w) => w.flush(),
            OutSink::Gz(w) => w.finish().and_then(|mut w| w.flush()),
            OutSink::Zst(w) => w.finish().and_then(|mut w| w.flush()),
        }
    }
}

/// `sparq-cli tabular …` — see [`usage`] and the module docs for the exact contract.
pub(crate) fn cmd_tabular(args: &[String]) {
    let flags = parse_flags(args);
    let Sources { srcs, quads } = build_sources(&flags).unwrap_or_else(|e| die(e));
    let mut reader = NtReader::new(srcs);
    let t = Instant::now();
    // [SONNET-4.6] (sq-u1z86) With graph maps the generated document is N-Quads. Every
    // default-graph statement is still a plain N-Triples line, so the N-Triples-only path is
    // byte-identical to before when no mapping uses rr:graphMap/rr:graph.
    let noun = if quads { "quads" } else { "triples" };

    if let Some(out_path) = &flags.out {
        // Stream straight out — no graph build, constant memory.
        if quads && (out_path.ends_with(".nt") || out_path.contains(".nt.")) {
            eprintln!("warning: the mapping uses rr:graphMap/rr:graph, so {out_path} receives N-QUADS (consider a .nq name)");
        }
        let mut sink = OutSink::create(out_path).unwrap_or_else(|e| die(format!("creating {out_path}: {e}")));
        let mut buf = vec![0u8; 64 * 1024];
        let mut stmts: u64 = 0;
        loop {
            let n = reader.read(&mut buf).unwrap_or_else(|e| die(e));
            if n == 0 {
                break;
            }
            stmts += buf[..n].iter().filter(|&&b| b == b'\n').count() as u64;
            sink.writer().write_all(&buf[..n]).unwrap_or_else(|e| die(format!("writing {out_path}: {e}")));
        }
        sink.finish().unwrap_or_else(|e| die(format!("finishing {out_path}: {e}")));
        eprintln!("wrote {stmts} {noun} to {out_path} in {:.3}s", t.elapsed().as_secs_f64());
        return;
    }

    // HONEST COST: the dataset path buffers the generated N-Quads, because sparq-core's
    // dataset loader (`load_dataset`) is whole-text — there is no streaming quad reader yet.
    // The N-Triples path keeps the fully-streaming parallel ingest.
    let (g, loaded) = if quads {
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap_or_else(|e| die(e));
        let n = text.lines().filter(|l| !l.trim().is_empty()).count() as u64;
        (sparq_core::Graph::load_dataset(&text, "nquads").unwrap_or_else(|e| die(e)), n)
    } else {
        let g = sparq_core::Graph::load_reader_parallel(reader, "ntriples").unwrap_or_else(|e| die(e));
        let n = g.len() as u64;
        (g, n)
    };
    let secs = t.elapsed().as_secs_f64();
    eprintln!(
        "loaded {loaded} {noun} in {secs:.3}s ({:.2} M/s) from tabular import",
        loaded as f64 / secs / 1e6
    );
    if let Some(q) = &flags.query {
        let count_only = args.iter().any(|a| a == "--count");
        crate::emit_query_results(&g, q, count_only, crate::out_format_flag(args));
    }
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn rows(csv: &str) -> Vec<Vec<String>> {
        CsvRows::new(Cursor::new(csv.as_bytes().to_vec()), b',')
            .collect::<Result<Vec<_>, _>>()
            .expect("csv parses")
    }

    fn rows_err(csv: &str) -> String {
        CsvRows::new(Cursor::new(csv.as_bytes().to_vec()), b',')
            .collect::<Result<Vec<Vec<String>>, _>>()
            .expect_err("csv must fail")
    }

    // ---- CSV reader --------------------------------------------------------------------

    #[test]
    fn csv_basic_lf_and_crlf_and_no_trailing_newline() {
        assert_eq!(rows("a,b\r\n1,2\n3,4"), vec![vec!["a", "b"], vec!["1", "2"], vec!["3", "4"]]);
    }

    #[test]
    fn csv_quoted_separator_newline_and_escaped_quote() {
        let got = rows("name,quote\n\"Doe, Jane\",\"said \"\"hi\"\"\nbye\"\n");
        assert_eq!(got, vec![vec!["name", "quote"], vec!["Doe, Jane", "said \"hi\"\nbye"]]);
    }

    #[test]
    fn csv_bom_stripped_and_blank_lines_skipped() {
        assert_eq!(rows("\u{feff}a,b\n\n1,2\n"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn csv_empty_fields_kept() {
        assert_eq!(rows("a,b,c\n,,\n"), vec![vec!["a", "b", "c"], vec!["", "", ""]]);
    }

    #[test]
    fn csv_custom_separator() {
        let got: Vec<Vec<String>> = CsvRows::new(Cursor::new(b"a\tb\n1\t2\n".to_vec()), b'\t')
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(got, vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn csv_loud_errors() {
        assert!(rows_err("a\"b\n").contains("stray"));
        assert!(rows_err("\"abc\n").contains("unterminated"));
        assert!(rows_err("a\rb\n").contains("bare CR"));
        assert!(rows_err("\"a\"x\n").contains("after the closing"));
        assert!(CsvRows::new(Cursor::new(vec![b'a', 0xFF, b'\n']), b',')
            .collect::<Result<Vec<Vec<String>>, _>>()
            .expect_err("invalid utf8")
            .contains("UTF-8"));
    }

    // ---- term helpers ------------------------------------------------------------------

    #[test]
    fn datatype_inference_exact() {
        let xsd = "http://www.w3.org/2001/XMLSchema#";
        assert_eq!(infer_datatype("42"), Some(format!("{xsd}integer")).as_deref());
        assert_eq!(infer_datatype("-7"), Some(format!("{xsd}integer")).as_deref());
        assert_eq!(infer_datatype("3.25"), Some(format!("{xsd}decimal")).as_deref());
        assert_eq!(infer_datatype("-0.5"), Some(format!("{xsd}decimal")).as_deref());
        assert_eq!(infer_datatype("6.02e23"), Some(format!("{xsd}double")).as_deref());
        assert_eq!(infer_datatype("1E-9"), Some(format!("{xsd}double")).as_deref());
        assert_eq!(infer_datatype("true"), Some(format!("{xsd}boolean")).as_deref());
        for s in ["", "hello", "1.2.3", "1e", "e9", ".", "+", "TRUE", "NaN", "INF", "0x1F", "12 "] {
            assert_eq!(infer_datatype(s), None, "{s:?} must stay a plain string");
        }
    }

    #[test]
    fn iri_safe_percent_encodes() {
        assert_eq!(iri_safe("Hello World/42?"), "Hello%20World%2F42%3F");
        assert_eq!(iri_safe("a-b.c_d~e"), "a-b.c_d~e");
        assert_eq!(iri_safe("café"), "café"); // non-ASCII kept (IRI, not URI)
    }

    #[test]
    fn nt_literal_escaping_exact() {
        assert_eq!(escape_literal("a\"b\\c\nd\te\r"), "a\\\"b\\\\c\\nd\\te\\r");
        assert_eq!(escape_literal("bell\u{7}"), "bell\\u0007");
    }

    #[test]
    fn blank_label_injective() {
        assert_eq!(blank_label("a b"), "a_20b");
        assert_ne!(blank_label("a_20b"), blank_label("a b")); // '_' itself is escaped
        assert_eq!(blank_label(""), "_empty");
    }

    #[test]
    fn check_iri_rejects_relative_and_bad_chars() {
        assert!(check_iri("http://example.com/x").is_ok());
        assert!(check_iri("row/1").is_err());
        assert!(check_iri("http://e.com/a b").is_err());
    }

    // ---- direct mapping ----------------------------------------------------------------

    fn direct_nt(csv: &str, opts: &DirectOpts, table: &str) -> Vec<String> {
        let mut all = CsvRows::new(Cursor::new(csv.as_bytes().to_vec()), b',');
        let header = all.next().unwrap().unwrap();
        let map = compile_direct(&header, table, opts).unwrap();
        let mr = MappedRows { rows: all, map, row_num: 0, failed: false };
        let mut lines: Vec<String> = Vec::new();
        for chunk in mr {
            lines.extend(chunk.unwrap().lines().map(str::to_owned));
        }
        lines.sort();
        lines
    }

    fn dopts() -> DirectOpts {
        DirectOpts {
            base: "http://example.com/".into(),
            template: None,
            class: None,
            infer: true,
            provenance: false,
        }
    }

    #[test]
    fn direct_mapping_default_template_class_and_inference() {
        let lines = direct_nt("name,age\nalice,34\n", &dopts(), "people");
        assert_eq!(
            lines,
            vec![
                "<http://example.com/people/row/1> <http://example.com/people#age> \"34\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                "<http://example.com/people/row/1> <http://example.com/people#name> \"alice\" .",
                "<http://example.com/people/row/1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/people> .",
            ]
        );
    }

    #[test]
    fn direct_mapping_custom_template_class_none_no_infer_null_cell() {
        let opts = DirectOpts {
            base: "http://example.com/".into(),
            template: Some("http://example.com/emp/{ID}".into()),
            class: Some(None),
            infer: false,
            provenance: false,
        };
        let lines = direct_nt("ID,name,age\n7,ann,\n", &opts, "emp");
        assert_eq!(
            lines,
            vec![
                // age is NULL → no triple; no rdf:type; "7" stays a plain string under --no-infer
                "<http://example.com/emp/7> <http://example.com/emp#ID> \"7\" .",
                "<http://example.com/emp/7> <http://example.com/emp#name> \"ann\" .",
            ]
        );
    }

    #[test]
    fn direct_mapping_iri_safe_encoding_in_predicates_and_subjects() {
        let opts = DirectOpts {
            base: "http://example.com/".into(),
            template: Some("http://example.com/x/{full name}".into()),
            class: Some(None),
            infer: true,
            provenance: false,
        };
        let lines = direct_nt("full name\nJane Doe\n", &opts, "my table");
        assert_eq!(
            lines,
            vec!["<http://example.com/x/Jane%20Doe> <http://example.com/my%20table#full%20name> \"Jane Doe\" ."]
        );
    }

    #[test]
    fn direct_mapping_header_errors() {
        assert!(compile_direct(&["a".into(), "a".into()], "t", &dopts()).unwrap_err().contains("duplicate"));
        assert!(compile_direct(&["a".into(), String::new()], "t", &dopts()).unwrap_err().contains("empty"));
    }

    #[test]
    fn ragged_row_is_loud() {
        let mut all = CsvRows::new(Cursor::new(b"a,b\n1\n".to_vec()), b',');
        let header = all.next().unwrap().unwrap();
        let map = compile_direct(&header, "t", &dopts()).unwrap();
        let err = MappedRows { rows: all, map, row_num: 0, failed: false }
            .collect::<Result<Vec<_>, _>>()
            .unwrap_err();
        assert!(err.contains("ragged"), "{err}");
    }

    #[test]
    fn template_parsing_escapes_and_errors() {
        let hdr = vec!["a".to_string()];
        assert!(parse_template("x\\{y\\}z{a}", &hdr, false).is_ok());
        assert!(parse_template("{missing}", &hdr, false).unwrap_err().contains("not found"));
        assert!(parse_template("{a", &hdr, false).unwrap_err().contains("unclosed"));
        assert!(parse_template("a}b", &hdr, false).unwrap_err().contains("unbalanced"));
        assert!(parse_template("{_row}", &hdr, false).unwrap_err().contains("_row"));
        assert!(parse_template("{_row}", &hdr, true).is_ok());
    }

    #[test]
    fn resolve_col_exact_ci_delimited_ambiguous() {
        let hdr: Vec<String> = ["Name", "NAME", "Age"].map(String::from).to_vec();
        assert_eq!(resolve_col("Name", &hdr).unwrap(), 0); // exact wins
        assert_eq!(resolve_col("age", &hdr).unwrap(), 2); // case-insensitive fallback
        assert!(resolve_col("name", &hdr).unwrap_err().contains("ambiguous"));
        assert_eq!(resolve_col("\"NAME\"", &hdr).unwrap(), 1); // delimited = exact only
        assert!(resolve_col("\"name\"", &hdr).unwrap_err().contains("not found"));
    }

    // ---- R2RML -------------------------------------------------------------------------

    /// In-memory logical tables, so the unit tests drive the SAME [`TableSource`] indirection
    /// the CLI's `rr:parentTriplesMap` join-index builder uses.
    struct MemTables(Vec<(String, String)>);

    impl TableSource for MemTables {
        fn open_table(&self, table: &str) -> Result<TableRows, String> {
            let (_, csv) = self
                .0
                .iter()
                .find(|(n, _)| n == table)
                .ok_or_else(|| format!("test: no CSV for table {table}"))?;
            let mut rows = CsvRows::new(Cursor::new(csv.as_bytes().to_vec()), b',');
            let header = rows.next().ok_or("empty csv")??;
            Ok((header, Box::new(rows)))
        }
    }

    /// Run an R2RML mapping over in-memory CSV tables; sorted unique N-Triples (or, with graph
    /// maps, N-Quads) lines out.
    fn r2rml_nt(mapping: &str, tables: &[(&str, &str)]) -> Result<Vec<String>, String> {
        r2rml_nt_prov(mapping, tables, false)
    }

    /// [`r2rml_nt`] with the `--provenance` switch.
    fn r2rml_nt_prov(
        mapping: &str,
        tables: &[(&str, &str)],
        provenance: bool,
    ) -> Result<Vec<String>, String> {
        let m = parse_r2rml(mapping)?;
        let src = MemTables(tables.iter().map(|(n, c)| ((*n).to_owned(), (*c).to_owned())).collect());
        let mut lines: Vec<String> = Vec::new();
        for (tm_key, table) in &m.tms {
            let (header, rows) = src.open_table(table)?;
            let prov = provenance.then_some(table.as_str());
            let map = compile_tm(&m, tm_key, &header, None, &src, prov)?;
            for chunk in (MappedRows { rows, map, row_num: 0, failed: false }) {
                lines.extend(chunk?.lines().map(str::to_owned));
            }
        }
        lines.sort();
        lines.dedup();
        Ok(lines)
    }

    const PREAMBLE: &str = "@prefix rr: <http://www.w3.org/ns/r2rml#> .\n@prefix ex: <http://example.com/> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    #[test]
    fn r2rml_template_subject_class_and_column_object() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM a rr:TriplesMap ;\n\
               rr:logicalTable [ rr:tableName \"student\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/{{ID}}/{{Name}}\" ; rr:class ex:Student ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:name ; rr:objectMap [ rr:column \"Name\" ] ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("student", "ID,Name\n10,Venus Williams\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/10/Venus%20Williams> <http://example.com/name> \"Venus Williams\" .",
                "<http://example.com/10/Venus%20Williams> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/Student> .",
            ]
        );
    }

    #[test]
    fn r2rml_datatype_language_constant_and_null() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:age ; rr:objectMap [ rr:column \"age\" ; rr:datatype xsd:integer ] ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:label ; rr:objectMap [ rr:column \"label\" ; rr:language \"en\" ] ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:src ; rr:object \"csv\" ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "id,age,label\n1,33,alpha\n2,,beta\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/age> \"33\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                "<http://example.com/r/1> <http://example.com/label> \"alpha\"@en .",
                "<http://example.com/r/1> <http://example.com/src> \"csv\" .",
                // row 2: age is NULL → that triple only is skipped
                "<http://example.com/r/2> <http://example.com/label> \"beta\"@en .",
                "<http://example.com/r/2> <http://example.com/src> \"csv\" .",
            ]
        );
    }

    #[test]
    fn r2rml_column_iri_blank_node_and_predicate_map() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:column \"who\" ; rr:termType rr:IRI ] ;\n\
               rr:predicateObjectMap [\n\
                 rr:predicateMap [ rr:template \"http://example.com/p/{{prop}}\" ] ;\n\
                 rr:objectMap [ rr:column \"tag\" ; rr:termType rr:BlankNode ]\n\
               ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "who,prop,tag\nhttp://example.com/alice,knows,x 1\n")]).unwrap();
        assert_eq!(got, vec!["<http://example.com/alice> <http://example.com/p/knows> _:x_201 ."]);
    }

    #[test]
    fn r2rml_null_subject_skips_row_and_default_literal_termtype() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:v ; rr:objectMap [ rr:column \"v\" ] ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "id,v\n,gone\n1,kept\n")]).unwrap();
        assert_eq!(got, vec!["<http://example.com/r/1> <http://example.com/v> \"kept\" ."]);
    }

    #[test]
    fn r2rml_empty_mapping_generates_nothing() {
        let got = r2rml_nt(PREAMBLE, &[]).unwrap();
        assert!(got.is_empty());
    }

    /// SQL virtualization stays a NON-GOAL: `rr:sqlQuery`/`rr:sqlVersion` and the other
    /// out-of-subset `rr:` constructs are still loud, fail-closed errors — never silent skips.
    #[test]
    fn r2rml_unsupported_constructs_fail_closed() {
        for (frag, needle) in [
            ("ex:TM rr:logicalTable [ rr:sqlQuery \"SELECT 1\" ] .", "rr:sqlQuery"),
            (
                "ex:TM rr:logicalTable [ rr:tableName \"t\" ; rr:sqlVersion ex:SQL2008 ] .",
                "rr:sqlVersion",
            ),
            (
                "ex:TM rr:logicalTable [ rr:tableName \"t\" ] ; rr:subjectMap [ rr:template \"http://e/{{x}}\" ; rr:inverseExpression \"{{x}} = 1\" ] .",
                "rr:inverseExpression",
            ),
        ] {
            let err = r2rml_nt(&format!("{PREAMBLE}{frag}\n"), &[("t", "x\n1\n")]).unwrap_err();
            assert!(err.contains(needle), "{err}");
        }
    }

    // ---- R2RML joins (rr:parentTriplesMap / rr:joinCondition), sq-u1z86 ------------------

    /// The canonical W3C shape: a child table joins a parent triples map on one column pair,
    /// and the object is the PARENT's generated subject.
    #[test]
    fn r2rml_join_parent_triples_map_single_condition() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:Dept rr:logicalTable [ rr:tableName \"dept\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/dept/{{code}}\" ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:name ; rr:objectMap [ rr:column \"name\" ] ] .\n\
             ex:Emp rr:logicalTable [ rr:tableName \"emp\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/emp/{{id}}\" ] ;\n\
               rr:predicateObjectMap [\n\
                 rr:predicate ex:worksIn ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:Dept ; rr:joinCondition [ rr:child \"dept\" ; rr:parent \"code\" ] ]\n\
               ] .\n"
        );
        let got = r2rml_nt(
            &mapping,
            &[("dept", "code,name\nENG,Engineering\nOPS,Operations\n"), ("emp", "id,dept\n1,ENG\n2,OPS\n3,ENG\n")],
        )
        .unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/dept/ENG> <http://example.com/name> \"Engineering\" .",
                "<http://example.com/dept/OPS> <http://example.com/name> \"Operations\" .",
                "<http://example.com/emp/1> <http://example.com/worksIn> <http://example.com/dept/ENG> .",
                "<http://example.com/emp/2> <http://example.com/worksIn> <http://example.com/dept/OPS> .",
                "<http://example.com/emp/3> <http://example.com/worksIn> <http://example.com/dept/ENG> .",
            ]
        );
    }

    /// A COMPOSITE key (two join conditions) must match on both columns, an unmatched key
    /// yields no triple, and a NULL on either side never joins.
    #[test]
    fn r2rml_join_composite_key_unmatched_and_null() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:P rr:logicalTable [ rr:tableName \"p\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/p/{{a}}-{{b}}\" ] .\n\
             ex:C rr:logicalTable [ rr:tableName \"c\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/c/{{id}}\" ] ;\n\
               rr:predicateObjectMap [\n\
                 rr:predicate ex:ref ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:P ;\n\
                   rr:joinCondition [ rr:child \"x\" ; rr:parent \"a\" ] ;\n\
                   rr:joinCondition [ rr:child \"y\" ; rr:parent \"b\" ] ]\n\
               ] .\n"
        );
        // c row 1 matches (1,2); row 2 has the right x but the wrong y; row 3 has a NULL x.
        // p row 3 has a NULL key cell, so it is never indexed and nothing can join to it —
        // c row 4 would otherwise have matched it on the ("", "5") key.
        let got = r2rml_nt(
            &mapping,
            &[("p", "a,b\n1,2\n3,4\n,5\n"), ("c", "id,x,y\n10,1,2\n11,1,9\n12,,2\n13,,5\n")],
        )
        .unwrap();
        assert_eq!(got, vec!["<http://example.com/c/10> <http://example.com/ref> <http://example.com/p/1-2> ."]);
    }

    /// ZERO join conditions is the spec's CROSS PRODUCT: every parent subject joins every
    /// child row. A parent row whose subject is NULL contributes nothing.
    #[test]
    fn r2rml_join_no_conditions_is_cross_product() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:P rr:logicalTable [ rr:tableName \"p\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/p/{{a}}\" ] .\n\
             ex:C rr:logicalTable [ rr:tableName \"c\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/c/{{id}}\" ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:any ; rr:objectMap [ rr:parentTriplesMap ex:P ] ] .\n"
        );
        // Parent row 3 has a NULL subject cell -> it generates no subject; row 4 repeats
        // row 1's subject -> the index deduplicates it. (The table needs a second column: a
        // fully-empty CSV LINE is not a record at all, so it would never reach the term map.)
        let got =
            r2rml_nt(&mapping, &[("p", "a,z\n1,x\n2,x\n,x\n1,y\n"), ("c", "id\n7\n8\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/c/7> <http://example.com/any> <http://example.com/p/1> .",
                "<http://example.com/c/7> <http://example.com/any> <http://example.com/p/2> .",
                "<http://example.com/c/8> <http://example.com/any> <http://example.com/p/1> .",
                "<http://example.com/c/8> <http://example.com/any> <http://example.com/p/2> .",
            ]
        );
    }

    /// A SELF-join re-opens the same logical table: only the parent's SUBJECT map is
    /// evaluated, so index construction terminates instead of recursing.
    #[test]
    fn r2rml_join_self_reference_terminates() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:N rr:logicalTable [ rr:tableName \"n\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/n/{{id}}\" ] ;\n\
               rr:predicateObjectMap [\n\
                 rr:predicate ex:parent ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:N ; rr:joinCondition [ rr:child \"pid\" ; rr:parent \"id\" ] ]\n\
               ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("n", "id,pid\n1,\n2,1\n3,2\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/n/2> <http://example.com/parent> <http://example.com/n/1> .",
                "<http://example.com/n/3> <http://example.com/parent> <http://example.com/n/2> .",
            ]
        );
    }

    /// Direct unit test of [`JoinIndex::lookup`]'s own NULL guard. The index BUILDER also
    /// refuses to index a NULL parent key, so through the mapping path the two guards are
    /// redundant — this pins `lookup` independently, against an index that DOES contain an
    /// empty-string key, so the guard cannot silently rot into a no-op.
    #[test]
    fn join_index_lookup_null_child_key_never_joins() {
        let subj = GenTerm::Iri("http://example.com/p/1".into());
        let mut keyed = HashMap::new();
        keyed.insert(vec![String::new()], vec![subj.clone()]);
        keyed.insert(vec!["k".to_string()], vec![subj.clone()]);
        let idx = JoinIndex { child_cols: vec![0], keyed, all: Vec::new() };
        assert_eq!(idx.lookup(&["k".to_string()]), std::slice::from_ref(&subj));
        // The empty cell is NULL: it must NOT match the empty-string key that IS in the index.
        assert!(idx.lookup(&[String::new()]).is_empty());
        // A key that is simply absent misses too.
        assert!(idx.lookup(&["nope".to_string()]).is_empty());
        // Zero join conditions: the cross product ignores the child row entirely.
        let cross = JoinIndex { child_cols: Vec::new(), keyed: HashMap::new(), all: vec![subj.clone()] };
        assert_eq!(cross.lookup(&[String::new()]), &[subj]);
    }

    #[test]
    fn r2rml_join_shape_errors() {
        let head = format!(
            "{PREAMBLE}ex:P rr:logicalTable [ rr:tableName \"p\" ] ;\n\
             rr:subjectMap [ rr:template \"http://example.com/p/{{a}}\" ] .\n\
             ex:C rr:logicalTable [ rr:tableName \"c\" ] ;\n\
             rr:subjectMap [ rr:template \"http://example.com/c/{{id}}\" ] ;\n"
        );
        let tables = &[("p", "a\n1\n"), ("c", "id\n7\n")];
        // rr:parentTriplesMap alongside a value-producing property
        let err = r2rml_nt(
            &format!(
                "{head} rr:predicateObjectMap [ rr:predicate ex:p ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:P ; rr:column \"id\" ] ] .\n"
            ),
            tables,
        )
        .unwrap_err();
        assert!(err.contains("rr:parentTriplesMap and rr:column"), "{err}");
        // parent is not a triples map
        let err = r2rml_nt(
            &format!(
                "{head} rr:predicateObjectMap [ rr:predicate ex:p ; rr:objectMap [ rr:parentTriplesMap ex:Nope ] ] .\n"
            ),
            tables,
        )
        .unwrap_err();
        assert!(err.contains("is not a triples map"), "{err}");
        // a join condition missing its rr:parent
        let err = r2rml_nt(
            &format!(
                "{head} rr:predicateObjectMap [ rr:predicate ex:p ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:P ; rr:joinCondition [ rr:child \"id\" ] ] ] .\n"
            ),
            tables,
        )
        .unwrap_err();
        assert!(err.contains("no rr:parent"), "{err}");
        // a join column that does not exist in the parent header
        let err = r2rml_nt(
            &format!(
                "{head} rr:predicateObjectMap [ rr:predicate ex:p ;\n\
                 rr:objectMap [ rr:parentTriplesMap ex:P ; rr:joinCondition [ rr:child \"id\" ; rr:parent \"nope\" ] ] ] .\n"
            ),
            tables,
        )
        .unwrap_err();
        assert!(err.contains("not found"), "{err}");
    }

    // ---- R2RML named graphs (rr:graphMap / rr:graph), sq-u1z86 ---------------------------

    /// A subject-map graph map routes the row's class triple AND every predicate-object map's
    /// statements into the named graph — the emitted document is N-Quads.
    #[test]
    fn r2rml_subject_graph_map_emits_nquads() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ; rr:class ex:Row ;\n\
                 rr:graphMap [ rr:template \"http://example.com/g/{{id}}\" ] ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:v ; rr:objectMap [ rr:column \"v\" ] ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "id,v\n1,x\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/v> \"x\" <http://example.com/g/1> .",
                "<http://example.com/r/1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/Row> <http://example.com/g/1> .",
            ]
        );
    }

    /// A predicate-object-map graph map applies to that map ALONE, and the statement's graph
    /// set is the UNION with the subject map's — so one triple can land in two graphs.
    #[test]
    fn r2rml_pom_graph_map_unions_with_subject_graphs() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ; rr:graph ex:gs ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:a ; rr:objectMap [ rr:column \"a\" ] ; rr:graph ex:gp ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:b ; rr:objectMap [ rr:column \"b\" ] ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "id,a,b\n1,x,y\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/a> \"x\" <http://example.com/gp> .",
                "<http://example.com/r/1> <http://example.com/a> \"x\" <http://example.com/gs> .",
                "<http://example.com/r/1> <http://example.com/b> \"y\" <http://example.com/gs> .",
            ]
        );
    }

    /// A NULL graph map is PER-POSITION: a subject graph map that generates NULL must not
    /// discard a graph the predicate-object map generated for itself. Only the statements whose
    /// every applicable graph map is NULL (here the `rr:class` triple and the graph-map-less
    /// `ex:b` map) are dropped.
    #[test]
    fn r2rml_null_subject_graph_keeps_pom_generated_graph() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ; rr:class ex:Row ;\n\
                 rr:graphMap [ rr:template \"http://example.com/g/{{g}}\" ] ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:a ; rr:objectMap [ rr:column \"a\" ] ; rr:graph ex:gp ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:b ; rr:objectMap [ rr:column \"b\" ] ] .\n"
        );
        // Row 1's graph cell is NULL; row 2 shows the same mapping's non-NULL behaviour.
        let got = r2rml_nt(&mapping, &[("t", "id,a,b,g\n1,x,y,\n2,x,y,A\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/a> \"x\" <http://example.com/gp> .",
                "<http://example.com/r/2> <http://example.com/a> \"x\" <http://example.com/g/A> .",
                "<http://example.com/r/2> <http://example.com/a> \"x\" <http://example.com/gp> .",
                "<http://example.com/r/2> <http://example.com/b> \"y\" <http://example.com/g/A> .",
                "<http://example.com/r/2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.com/Row> <http://example.com/g/A> .",
            ]
        );
    }

    /// The mirror image: a predicate-object-map graph map that generates NULL must not discard
    /// the graph the SUBJECT map generated — the statement still lands in `ex:gs`.
    #[test]
    fn r2rml_null_pom_graph_keeps_subject_generated_graph() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ; rr:graph ex:gs ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:a ; rr:objectMap [ rr:column \"a\" ] ;\n\
                 rr:graphMap [ rr:template \"http://example.com/g/{{g}}\" ] ] .\n"
        );
        // Row 1's graph cell is NULL; row 2 shows the same mapping's non-NULL behaviour.
        let got = r2rml_nt(&mapping, &[("t", "id,a,g\n1,x,\n2,x,A\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/a> \"x\" <http://example.com/gs> .",
                "<http://example.com/r/2> <http://example.com/a> \"x\" <http://example.com/g/A> .",
                "<http://example.com/r/2> <http://example.com/a> \"x\" <http://example.com/gs> .",
            ]
        );
    }

    /// `rr:defaultGraph` names the DEFAULT graph, so those statements stay plain triples; a
    /// graph map that generates NULL drops the statement rather than defaulting it.
    #[test]
    fn r2rml_default_graph_constant_and_null_graph() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:d ; rr:objectMap [ rr:column \"v\" ] ; rr:graph rr:defaultGraph ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:n ; rr:objectMap [ rr:column \"v\" ] ;\n\
                 rr:graphMap [ rr:template \"http://example.com/g/{{g}}\" ] ] .\n"
        );
        // Row 2's graph cell is NULL -> only the ex:n statement is dropped.
        let got = r2rml_nt(&mapping, &[("t", "id,v,g\n1,x,A\n2,y,\n")]).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/1> <http://example.com/d> \"x\" .",
                "<http://example.com/r/1> <http://example.com/n> \"x\" <http://example.com/g/A> .",
                "<http://example.com/r/2> <http://example.com/d> \"y\" .",
            ]
        );
    }

    /// Misplaced/orphaned constructs must FAIL rather than be silently dropped — a mapping
    /// that quietly loses its named-graph routing or its join is the worst failure mode.
    #[test]
    fn r2rml_misplaced_graph_map_and_orphan_join_condition_fail_closed() {
        let err = r2rml_nt(
            &format!(
                "{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] ; rr:graph ex:g ;\n\
                 rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] .\n"
            ),
            &[("t", "id\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("directly") && err.contains("rr:graph"), "{err}");

        let err = r2rml_nt(
            &format!(
                "{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
                 rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] ;\n\
                 rr:predicateObjectMap [ rr:predicate ex:p ;\n\
                   rr:objectMap [ rr:column \"id\" ; rr:joinCondition [ rr:child \"id\" ; rr:parent \"id\" ] ] ] .\n"
            ),
            &[("t", "id\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("rr:joinCondition but no rr:parentTriplesMap"), "{err}");
    }

    #[test]
    fn r2rml_graph_map_must_be_an_iri() {
        let err = r2rml_nt(
            &format!(
                "{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
                 rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ;\n\
                   rr:graphMap [ rr:column \"id\" ; rr:termType rr:BlankNode ] ] .\n"
            ),
            &[("t", "id\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("graph map") && err.contains("blank node"), "{err}");
    }

    // ---- row provenance (--provenance), sq-u1z86 -----------------------------------------

    #[test]
    fn direct_mapping_row_provenance_triples() {
        let opts = DirectOpts {
            base: "http://example.com/".into(),
            template: None,
            class: Some(None),
            infer: true,
            provenance: true,
        };
        let lines = direct_nt("v\nx\ny\n", &opts, "people");
        assert_eq!(
            lines,
            vec![
                "<http://example.com/people/row/1> <http://example.com/people#v> \"x\" .",
                "<http://example.com/people/row/1> <https://sparq.dev/ns/tabular#row> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                "<http://example.com/people/row/1> <https://sparq.dev/ns/tabular#table> \"people\" .",
                "<http://example.com/people/row/2> <http://example.com/people#v> \"y\" .",
                "<http://example.com/people/row/2> <https://sparq.dev/ns/tabular#row> \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
                "<http://example.com/people/row/2> <https://sparq.dev/ns/tabular#table> \"people\" .",
            ]
        );
    }

    /// R2RML has no row concept, which is exactly why `--provenance` is useful there — and
    /// the provenance triples follow the subject map into its named graph.
    #[test]
    fn r2rml_row_provenance_follows_the_subject_graph() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ; rr:graph ex:g ] ;\n\
               rr:predicateObjectMap [ rr:predicate ex:v ; rr:objectMap [ rr:column \"v\" ] ] .\n"
        );
        let got = r2rml_nt_prov(&mapping, &[("t", "id,v\n9,x\n")], true).unwrap();
        assert_eq!(
            got,
            vec![
                "<http://example.com/r/9> <http://example.com/v> \"x\" <http://example.com/g> .",
                "<http://example.com/r/9> <https://sparq.dev/ns/tabular#row> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> <http://example.com/g> .",
                "<http://example.com/r/9> <https://sparq.dev/ns/tabular#table> \"t\" <http://example.com/g> .",
            ]
        );
    }

    #[test]
    fn r2rml_shape_errors() {
        // both template and column
        let err = r2rml_nt(
            &format!(
                "{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
                 rr:subjectMap [ rr:template \"http://e/{{x}}\" ; rr:column \"x\" ] .\n"
            ),
            &[("t", "x\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("exactly one"), "{err}");
        // datatype + language together
        let err = r2rml_nt(
            &format!(
                "{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
                 rr:subjectMap [ rr:template \"http://e/{{x}}\" ] ;\n\
                 rr:predicateObjectMap [ rr:predicate ex:p ; rr:objectMap [ rr:column \"x\" ; rr:datatype xsd:int ; rr:language \"en\" ] ] .\n"
            ),
            &[("t", "x\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("both rr:datatype and rr:language"), "{err}");
        // missing subject
        let err = r2rml_nt(
            &format!("{PREAMBLE}ex:TM rr:logicalTable [ rr:tableName \"t\" ] .\n"),
            &[("t", "x\n1\n")],
        )
        .unwrap_err();
        assert!(err.contains("no rr:subjectMap"), "{err}");
    }

    #[test]
    fn r2rml_pom_cartesian_two_predicates_two_objects() {
        let mapping = format!(
            "{PREAMBLE}\
             ex:TM rr:logicalTable [ rr:tableName \"t\" ] ;\n\
               rr:subjectMap [ rr:template \"http://example.com/r/{{id}}\" ] ;\n\
               rr:predicateObjectMap [\n\
                 rr:predicate ex:p1 ; rr:predicate ex:p2 ;\n\
                 rr:objectMap [ rr:column \"a\" ] ; rr:objectMap [ rr:column \"b\" ]\n\
               ] .\n"
        );
        let got = r2rml_nt(&mapping, &[("t", "id,a,b\n1,x,y\n")]).unwrap();
        assert_eq!(got.len(), 4, "{got:?}");
        for (p, o) in [("p1", "x"), ("p1", "y"), ("p2", "x"), ("p2", "y")] {
            assert!(got.contains(&format!("<http://example.com/r/1> <http://example.com/{p}> \"{o}\" .")), "{got:?}");
        }
    }

    // ---- plumbing ----------------------------------------------------------------------

    #[test]
    fn table_stem_strips_dir_compression_and_format() {
        assert_eq!(table_stem("people.csv"), "people");
        assert_eq!(table_stem("/a/b/people.csv.gz"), "people");
        assert_eq!(table_stem("x.csv.zst"), "x");
        assert_eq!(table_stem("noext"), "noext");
    }

    #[test]
    fn nt_reader_streams_and_surfaces_errors() {
        let ok: ChunkIter = Box::new(vec![Ok("a\n".to_string()), Ok(String::new()), Ok("b\n".to_string())].into_iter());
        let mut out = String::new();
        NtReader::new(vec![ok]).read_to_string(&mut out).unwrap();
        assert_eq!(out, "a\nb\n");
        let bad: ChunkIter = Box::new(vec![Ok("a\n".to_string()), Err("boom".to_string())].into_iter());
        let err = NtReader::new(vec![bad]).read_to_string(&mut String::new()).unwrap_err();
        assert!(err.to_string().contains("boom"));
    }

    /// End-to-end into the engine: direct-mapped CSV chunks load through
    /// `load_reader_parallel` and are queryable.
    #[test]
    fn direct_mapping_loads_into_graph_and_queries() {
        let csv = "name,age\nalice,34\nbob,28\n";
        let mut all = CsvRows::new(Cursor::new(csv.as_bytes().to_vec()), b',');
        let header = all.next().unwrap().unwrap();
        let map = compile_direct(&header, "people", &dopts()).unwrap();
        let src: ChunkIter = Box::new(MappedRows { rows: all, map, row_num: 0, failed: false });
        let g = sparq_core::Graph::load_reader_parallel(NtReader::new(vec![src]), "ntriples").unwrap();
        assert_eq!(g.len(), 6); // 2 rows × (2 columns + rdf:type)
        let r = sparq_engine::query(
            &g,
            "SELECT ?n WHERE { ?s <http://example.com/people#age> ?a . ?s <http://example.com/people#name> ?n . FILTER(?a > 30) }",
        )
        .unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.rows[0][0], Some(oxrdf::Term::Literal(oxrdf::Literal::new_simple_literal("alice"))));
    }
}
