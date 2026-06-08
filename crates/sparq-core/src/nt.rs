//! A byte-level N-Triples parser that interns directly into the [`Dict`], skipping the
//! intermediate `oxrdf::Term` allocation that a general parser makes for every term
//! *occurrence*. The common no-escape case interns the raw UTF-8 slice with zero
//! allocation; terms containing `\` take a correct escape-decoding slow path. Used by
//! the parallel bulk loader for the `ntriples` format (it is one statement per line, so
//! a byte buffer can be split at newlines and parsed in parallel).
//!
//! Blank-node labels are kept verbatim (file-scoped, as the loader folds one document).

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
        let (o, j) = term(bytes, i, dict)?;
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
            let lang = str_of(&b[lstart..j])?;
            Ok((dict.intern_lit(&value, RDF_LANG_STRING, Some(lang)), j))
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
