//! A byte-level N-Triples parser that interns directly into the [`Dict`], skipping the
//! intermediate `oxrdf::Term` allocation that a general parser makes for every term
//! *occurrence*. The common no-escape case interns the raw UTF-8 slice with zero
//! allocation; terms containing `\` take a correct escape-decoding slow path. Used by
//! the parallel bulk loader for the `ntriples` format (it is one statement per line, so
//! a byte buffer can be split at newlines and parsed in parallel).
//!
//! Blank-node labels are kept verbatim (file-scoped, as the loader folds one document).
//!
//! [OPUS-4.8] (sq-hxgb) RDF 1.2 triple terms `<<( s p o )>>` are accepted in the OBJECT
//! position (object-only, may nest), interned structurally to agree with the Turtle loader
//! and the engine's CONSTRUCT N-Triples serializer (which emits this form). The fast plain-
//! triple path is unchanged: a triple term is only looked for when the object byte is `<`
//! AND the next two bytes are `<(`, so a plain `<…>` IRI pays just two extra byte peeks.

use crate::dict::{Dict, Id};
use std::borrow::Cow;

/// Parses a slice of complete N-Triples lines, interning each term and returning the
/// id-triples. Errors carry the byte offset.
pub fn parse_chunk(bytes: &[u8], dict: &mut Dict) -> Result<Vec<[Id; 3]>, String> {
    let mut out = Vec::new();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        i = skip_ws(bytes, i);
        if i >= n {
            break;
        }
        if bytes[i] == b'#' {
            i = next_line(bytes, i);
            continue;
        }
        let (s, j) = term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        let (p, j) = term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        // [OPUS-4.8] (sq-hxgb) Only the OBJECT position may be an RDF 1.2 triple term
        // `<<( s p o )>>` (matching the Turtle loader / RDF 1.2 grammar — triple terms
        // are object-only and may nest through their own object).
        let (o, j) = object_term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        if i >= n || bytes[i] != b'.' {
            return Err(format!("N-Triples: expected '.' at byte {i}"));
        }
        i += 1;
        out.push([s, p, o]);
    }
    Ok(out)
}

#[inline]
fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    i
}

#[inline]
fn next_line(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }
    i
}

#[inline]
fn str_of(raw: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(raw).map_err(|_| "N-Triples: invalid UTF-8".to_string())
}

fn term(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    match b.get(i) {
        Some(b'<') => iri(b, i, dict),
        Some(b'_') => blank(b, i, dict),
        Some(b'"') => literal(b, i, dict),
        _ => Err(format!("N-Triples: unexpected term start at byte {i}")),
    }
}

/// [OPUS-4.8] (sq-hxgb) Parses a term in the OBJECT position, where (and only where) RDF 1.2
/// N-Triples permits a triple term `<<( s p o )>>`. The `<<(` opener disambiguates a triple
/// term from a plain IRIREF (`<…>`); anything else falls through to the shared [`term`] path.
fn object_term(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    if b.get(i) == Some(&b'<') && b.get(i + 1) == Some(&b'<') && b.get(i + 2) == Some(&b'(') {
        triple_term(b, i, dict)
    } else {
        term(b, i, dict)
    }
}

/// [OPUS-4.8] (sq-hxgb) Parses an RDF 1.2 triple term `<<( s p o )>>` and interns it
/// STRUCTURALLY (subject/predicate/object interned first, then the triple by their ids — the
/// same `Dict::intern_triple_ids` path `intern(&Term::Triple(_))` and the Turtle loader take,
/// so the two loaders content-address triple terms identically). The subject is an IRI or
/// blank node, the predicate an IRI, and the object any term — including a NESTED triple term
/// (recursion through `object_term`). Returns the interned id and the index past the `>>`.
fn triple_term(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    // Skip the `<<(` opener.
    let mut j = skip_ws(b, i + 3);
    let (s, k) = term(b, j, dict)?;
    j = skip_ws(b, k);
    let (p, k) = term(b, j, dict)?;
    j = skip_ws(b, k);
    let (o, k) = object_term(b, j, dict)?;
    j = skip_ws(b, k);
    // Closer `)>>`.
    if b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b'>') && b.get(j + 2) == Some(&b'>') {
        Ok((dict.intern_triple_ids([s, p, o]), j + 3))
    } else {
        Err(format!("N-Triples: expected ')>>' closing triple term at byte {j}"))
    }
}

/// Scans `<...>` returning the (escaped?) content range and the index past `>`.
fn scan_delim(b: &[u8], open: usize, close: u8) -> Result<(usize, usize, bool, usize), String> {
    let start = open + 1;
    let mut j = start;
    let mut esc = false;
    while j < b.len() && b[j] != close {
        if b[j] == b'\\' {
            esc = true;
            j += 1;
        }
        j += 1;
    }
    if j >= b.len() {
        return Err(format!("N-Triples: unterminated delimiter from byte {open}"));
    }
    Ok((start, j, esc, j + 1))
}

fn decode<'a>(raw: &'a [u8], esc: bool) -> Result<Cow<'a, str>, String> {
    let s = str_of(raw)?;
    if esc {
        Ok(Cow::Owned(unescape(s)?))
    } else {
        Ok(Cow::Borrowed(s))
    }
}

fn iri(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    let (start, end, esc, next) = scan_delim(b, i, b'>')?;
    let s = decode(&b[start..end], esc)?;
    Ok((dict.intern_iri(&s), next))
}

fn blank(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    if b.get(i + 1) != Some(&b':') {
        return Err(format!("N-Triples: bad blank node at byte {i}"));
    }
    let start = i + 2;
    let mut j = start;
    while j < b.len() && !matches!(b[j], b' ' | b'\t' | b'\r' | b'\n' | b'.') {
        j += 1;
    }
    Ok((dict.intern_blank(str_of(&b[start..j])?), j))
}

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
const RDF_DIR_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";

fn literal(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    let (vstart, vend, vesc, after) = scan_delim(b, i, b'"')?;
    let value = decode(&b[vstart..vend], vesc)?;
    match b.get(after) {
        // ^^<datatype>
        Some(b'^') if b.get(after + 1) == Some(&b'^') => {
            let open = after + 2;
            if b.get(open) != Some(&b'<') {
                return Err(format!("N-Triples: bad datatype at byte {open}"));
            }
            let (dstart, dend, desc, next) = scan_delim(b, open, b'>')?;
            let dt = decode(&b[dstart..dend], desc)?;
            Ok((dict.intern_lit(&value, &dt, None), next))
        }
        // @lang
        Some(b'@') => {
            let lstart = after + 1;
            let mut j = lstart;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-') {
                j += 1;
            }
            // RDF 1.2 `@lang--dir` keeps the combined slot (the dict's stored encoding)
            // but is typed rdf:dirLangString.
            let lang = str_of(&b[lstart..j])?;
            let dt = if lang.contains("--") { RDF_DIR_LANG_STRING } else { RDF_LANG_STRING };
            Ok((dict.intern_lit(&value, dt, Some(lang)), j))
        }
        // simple literal -> xsd:string
        _ => Ok((dict.intern_lit(&value, XSD_STRING, None), after)),
    }
}

/// Decodes N-Triples ECHAR (`\t \b \n \r \f \" \' \\`) and UCHAR (`\uXXXX`, `\UXXXXXXXX`).
fn unescape(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{8}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('f') => out.push('\u{C}'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('u') => out.push(hex(&mut it, 4)?),
            Some('U') => out.push(hex(&mut it, 8)?),
            _ => return Err("N-Triples: bad escape".into()),
        }
    }
    Ok(out)
}

fn hex(it: &mut std::str::Chars<'_>, n: usize) -> Result<char, String> {
    let mut cp = 0u32;
    for _ in 0..n {
        let d = it.next().and_then(|c| c.to_digit(16)).ok_or("N-Triples: bad \\u escape")?;
        cp = cp * 16 + d;
    }
    char::from_u32(cp).ok_or_else(|| "N-Triples: invalid code point".into())
}

// [OPUS-4.8] (sq-hxgb) Triple-term parsing in the OBJECT position: structural interning,
// nesting, blank-node components, fast-path-unchanged, and the rejection cases.
#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new_unchecked(s))
    }

    #[test]
    fn triple_term_object_parses_and_interns_structurally() {
        // The exact byte form the CONSTRUCT serializer emits.
        let nt = b"<http://ex/r1> <http://ex/reifies> <<( <http://ex/alice> <http://ex/age> \"30\"^^<http://www.w3.org/2001/XMLSchema#integer> )>> .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        let [s, p, o] = t[0];
        assert_eq!(d.term(s), iri("http://ex/r1"));
        assert_eq!(d.term(p), iri("http://ex/reifies"));
        // Object is a structural Term::Triple — not a literal or opaque IRI.
        let want = Term::Triple(Box::new(Triple::new(
            NamedNode::new_unchecked("http://ex/alice"),
            NamedNode::new_unchecked("http://ex/age"),
            Term::Literal(Literal::new_typed_literal("30", NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer"))),
        )));
        assert_eq!(d.term(o), want);
        // Content-addressed: re-interning the identical term shares the id.
        assert_eq!(d.lookup(&want), o);
    }

    #[test]
    fn nested_and_blank_node_triple_terms_parse() {
        let nt = b"<http://ex/r> <http://ex/p> <<( _:b0 <http://ex/q> <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> )>> .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        let inner = Term::Triple(Box::new(Triple::new(
            NamedNode::new_unchecked("http://ex/a"),
            NamedNode::new_unchecked("http://ex/b"),
            iri("http://ex/c"),
        )));
        let outer = Term::Triple(Box::new(Triple::new(
            BlankNode::new_unchecked("b0"),
            NamedNode::new_unchecked("http://ex/q"),
            inner,
        )));
        assert_eq!(d.term(t[0][2]), outer);
    }

    #[test]
    fn plain_triples_unaffected() {
        // A plain IRI object (`<…>`) must NOT be mistaken for a triple term, and a literal
        // object beginning with `<`-free bytes parses as before.
        let nt = b"<http://ex/s> <http://ex/p> <http://ex/o> .\n<http://ex/s> <http://ex/p> \"v\" .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(d.term(t[0][2]), iri("http://ex/o"));
        assert_eq!(d.term(t[1][2]), Term::Literal(Literal::new_simple_literal("v")));
    }

    #[test]
    fn unterminated_triple_term_errors() {
        let nt = b"<http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/b> <http://ex/c> .\n";
        let mut d = Dict::new();
        assert!(parse_chunk(nt, &mut d).is_err(), "missing )>> must error, not silently accept");
    }

    #[test]
    fn triple_term_only_in_object_position() {
        // A `<<(` in SUBJECT position is not a valid term start for `term` (subjects are
        // never triple terms in RDF 1.2) — the parser must reject it rather than misparse.
        let nt = b"<<( <http://ex/a> <http://ex/b> <http://ex/c> )>> <http://ex/p> <http://ex/o> .\n";
        let mut d = Dict::new();
        assert!(parse_chunk(nt, &mut d).is_err(), "triple term in subject position must be rejected");
    }
}
