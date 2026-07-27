//! IRI Expansion (JSON-LD 1.1 API §5.2) and RFC 3986 reference resolution.
//!
//! [OPUS-4.8] (sq-oy1f.24) IRI Expansion resolves a term / compact IRI / relative
//! reference against an [`ActiveContext`], honouring keyword
//! aliases, `@vocab`, `@base`, and on-demand term creation from a local context. The
//! compaction-side companions (IRI Compaction, Term Selection, inverse-context
//! construction) are deferred to a follow-on bead — see the module docs.

use std::collections::BTreeMap;

use super::process::{create_term_definition, Env};
use super::{has_keyword_form, is_keyword, ActiveContext};
use crate::error::JsonLdError;
use crate::json::Json;

/// True iff `s` is a blank node identifier (`_:` prefix).
pub(crate) fn is_blank_node(s: &str) -> bool {
    s.starts_with("_:")
}

/// True iff `s` is an absolute IRI: a valid scheme followed by `:`, whose remaining
/// characters are all drawn from the RFC 3987 IRI code-point set and whose `%` escapes are
/// all well-formed.
pub(crate) fn is_absolute_iri(s: &str) -> bool {
    match s.find(':') {
        Some(colon) => is_valid_scheme(&s[..colon]) && has_only_iri_chars(s),
        None => false,
    }
}

/// True iff every code point of `s` is one an IRI may contain (RFC 3987 §2.2) **and** every
/// `%` opens a well-formed `pct-encoded` triplet (`"%" HEXDIG HEXDIG`).
///
/// The admitted set is:
/// * ASCII — `unreserved` / `gen-delims` / `sub-delims` / `%`, i.e. every printable ASCII
///   character except the RFC 3986 §2.2 "excluded" delimiters (space, `<`, `>`, `"`, `{`,
///   `}`, `|`, `\`, `^`, `` ` ``) and the C0/DEL controls.
/// * Non-ASCII — the `ucschar` and `iprivate` ranges (see [`is_ucschar_or_iprivate`]),
///   which excludes the C1 controls and the Unicode noncharacters.
///
/// [OPUS-5] sq-gzsky — this is the "Processors MUST validate datatype IRIs" obligation
/// behind the `invalid typed value` negative (W3C expand/0123 pins
/// `"http://example.com/baz z"`, a scheme-valid string carrying a SPACE). Scheme validity
/// alone accepted it.
///
/// [SONNET-4.6] (PR #4610 review) The first cut was a character *denylist* with no escape
/// validation, so `http://example.com/%ZZ` and a trailing bare `%` still passed. This is
/// now a complete code-point + `pct-encoded` check.
///
/// **Scope, stated honestly:** this validates the IRI *code-point* grammar, not the full
/// RFC 3987 *structural* grammar — component shape (authority/host/port form, where
/// `gen-delims` such as `[`/`]` are legal, the query-only restriction on `iprivate`) is not
/// enforced. That is a deliberate stopping point: the reference processors validate at this
/// level too, and over-rejecting a well-formed IRI would silently cost conformance passes
/// through the many [`is_absolute_iri`] call sites in context processing.
fn has_only_iri_chars(s: &str) -> bool {
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            // pct-encoded = "%" HEXDIG HEXDIG — a bare (`%`), truncated (`%A`) or
            // non-hex (`%ZZ`) escape is not an IRI.
            let (Some(h1), Some(h2)) = (chars.next(), chars.next()) else {
                return false;
            };
            if !h1.is_ascii_hexdigit() || !h2.is_ascii_hexdigit() {
                return false;
            }
        } else if c.is_ascii() {
            if c.is_ascii_control()
                || matches!(c, ' ' | '<' | '>' | '"' | '{' | '}' | '|' | '\\' | '^' | '`')
            {
                return false;
            }
        } else if !is_ucschar_or_iprivate(c) {
            return false;
        }
    }
    true
}

/// True iff `c` is in RFC 3987 §2.2 `ucschar` or `iprivate` — the non-ASCII code points an
/// IRI may carry.
///
/// ```text
/// ucschar  = %xA0-D7FF / %xF900-FDCF / %xFDF0-FFEF / %x10000-1FFFD / %x20000-2FFFD
///          / %x30000-3FFFD / %x40000-4FFFD / %x50000-5FFFD / %x60000-6FFFD
///          / %x70000-7FFFD / %x80000-8FFFD / %x90000-9FFFD / %xA0000-AFFFD
///          / %xB0000-BFFFD / %xC0000-CFFFD / %xD0000-DFFFD / %xE1000-EFFFD
/// iprivate = %xE000-F8FF / %xF0000-FFFFD / %x100000-10FFFD
/// ```
///
/// `iprivate` is admitted anywhere rather than in the query component only — component
/// scoping belongs to the structural grammar this check deliberately does not implement
/// (see [`has_only_iri_chars`]). Surrogates need no handling: a Rust `char` cannot be one.
fn is_ucschar_or_iprivate(c: char) -> bool {
    let cp = c as u32;
    match cp {
        // ucschar, BMP portion; the gaps are the C1 controls (<0xA0), the surrogate and
        // private-use blocks, and the noncharacters 0xFDD0-0xFDEF / 0xFFF0-0xFFFF.
        0xA0..=0xD7FF | 0xF900..=0xFDCF | 0xFDF0..=0xFFEF => true,
        // iprivate, BMP portion (the private-use area).
        0xE000..=0xF8FF => true,
        // ucschar, supplementary planes 1-13: each admits 0x0000-0xFFFD of its plane.
        0x10000..=0xDFFFD => cp & 0xFFFF <= 0xFFFD,
        // ucschar, plane 14: starts at 0xE1000, so the tag / variation-selector block
        // 0xE0000-0xE0FFF is excluded.
        0xE1000..=0xEFFFD => true,
        // iprivate, supplementary planes 15-16.
        0xF0000..=0x10FFFD => cp & 0xFFFF <= 0xFFFD,
        _ => false,
    }
}

/// True iff `s` is a syntactically valid URI scheme: `ALPHA *( ALPHA / DIGIT / "+" / "-" /
/// "." )` (RFC 3986 §3.1).
fn is_valid_scheme(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b[0].is_ascii_alphabetic()
        && b.iter()
            .all(|&c| c.is_ascii_alphanumeric() || c == b'+' || c == b'-' || c == b'.')
}

impl ActiveContext {
    /// IRI Expansion (JSON-LD 1.1 API §5.2) — read-only.
    ///
    /// Expands `value` (a term, compact IRI, or relative reference) against this active
    /// context. With `vocab` true, `value` is resolved as a *vocabulary* reference (against
    /// term definitions and `@vocab`); with `document_relative` true, an otherwise
    /// unresolved reference is resolved against the base IRI. Returns `None` when `value`
    /// expands to null (a keyword-shaped non-keyword, or a term bound to null).
    ///
    /// This is the entry point once the active context is fully built. During Context
    /// Processing, expansion may need to define terms on demand; that variant lives in
    /// the crate-internal `expand_iri_in_context`.
    pub fn expand_iri(&self, value: &str, document_relative: bool, vocab: bool) -> Option<String> {
        self.expand_iri_readonly(value, document_relative, vocab)
    }

    /// The read-only core of IRI Expansion — no on-demand term creation (steps that would
    /// invoke Create Term Definition are skipped; the active context is assumed complete).
    pub(crate) fn expand_iri_readonly(
        &self,
        value: &str,
        document_relative: bool,
        vocab: bool,
    ) -> Option<String> {
        // §5.2 step 1: a keyword (or null) expands to itself.
        if is_keyword(value) {
            return Some(value.to_string());
        }
        // step 2: a keyword-shaped token that is not a recognised keyword expands to null.
        if has_keyword_form(value) {
            return None;
        }
        // step 4: a term whose IRI mapping is a keyword expands to that keyword.
        if let Some(def) = self.term_definitions.get(value) {
            if let Some(iri) = &def.iri {
                if is_keyword(iri) {
                    return Some(iri.clone());
                }
            }
        }
        // step 5: a vocabulary reference that is a defined term expands to its IRI mapping
        // (which may be null — e.g. an explicit `@id: null`, in which case the term drops).
        if vocab {
            if let Some(def) = self.term_definitions.get(value) {
                return def.iri.clone();
            }
        }
        // step 6: a compact IRI or absolute IRI.
        if let Some(colon) = value.find(':') {
            if colon > 0 {
                let prefix = &value[..colon];
                let suffix = &value[colon + 1..];
                // A blank node identifier or an IRI with an authority (`scheme://…`) is
                // returned unchanged.
                if prefix == "_" || suffix.starts_with("//") {
                    return Some(value.to_string());
                }
                // [SONNET-4.6] sq-oy1f.45 — a colon in a fragment (e.g. `#Test:2`) does
                // NOT mark a compact-IRI prefix; the prefix must be a valid RFC 3986 URI
                // scheme (ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )).  A non-scheme
                // prefix (starting with `#`, containing non-scheme chars, etc.) means the
                // value is a relative reference — fall through to base resolution (step 8).
                // W3C expand/0109: `#Test:2` must resolve against the base, not be returned
                // as-is as if it were an absolute IRI.
                if !is_valid_scheme(prefix) {
                    // Not a compact IRI (prefix is not a valid scheme); fall through to
                    // vocab / base resolution.
                } else {
                    // If the prefix is a term flagged `@prefix`, concatenate.
                    if let Some(pdef) = self.term_definitions.get(prefix) {
                        if pdef.prefix {
                            if let Some(piri) = &pdef.iri {
                                return Some(format!("{}{}", piri, suffix));
                            }
                        }
                    }
                    // Otherwise the value already contains a valid scheme: treat as absolute IRI.
                    return Some(value.to_string());
                }
            }
        }
        // step 7: a vocabulary reference with a `@vocab` mapping.
        if vocab {
            if let Some(vocab_iri) = &self.vocabulary_mapping {
                return Some(format!("{}{}", vocab_iri, value));
            }
        }
        // step 8: a document-relative reference resolved against the base IRI.
        if document_relative {
            if let Some(base) = &self.base_iri {
                return Some(resolve_iri(base, value));
            }
        }
        // step 9: return unchanged.
        Some(value.to_string())
    }
}

/// IRI Expansion with on-demand term creation (JSON-LD 1.1 API §5.2 steps 3 and 6.3):
/// while a local context is being processed, expanding a value may require the referenced
/// term — or the prefix of a compact IRI — to be defined first. Used by Create Term
/// Definition; delegates the actual lookup to [`ActiveContext::expand_iri_readonly`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn expand_iri_in_context(
    active: &mut ActiveContext,
    value: &str,
    document_relative: bool,
    vocab: bool,
    local: &Json,
    defined: &mut BTreeMap<String, bool>,
    env: &mut Env,
) -> Result<Option<String>, JsonLdError> {
    // Keyword / keyword-shaped tokens short-circuit before any term is created (§5.2 1–2).
    if is_keyword(value) {
        return Ok(Some(value.to_string()));
    }
    if has_keyword_form(value) {
        return Ok(None);
    }
    // step 3: if the local context defines `value` and it is not yet built, build it.
    ensure_defined(active, value, local, defined, env)?;
    // step 6.3: for a compact IRI, ensure its prefix term is built too.
    if let Some(colon) = value.find(':') {
        if colon > 0 {
            let prefix = &value[..colon];
            let suffix = &value[colon + 1..];
            if prefix != "_" && !suffix.starts_with("//") {
                ensure_defined(active, prefix, local, defined, env)?;
            }
        }
    }
    Ok(active.expand_iri_readonly(value, document_relative, vocab))
}

/// If `term` is defined by the local context and has not yet been built into `active`,
/// invoke Create Term Definition for it.
fn ensure_defined(
    active: &mut ActiveContext,
    term: &str,
    local: &Json,
    defined: &mut BTreeMap<String, bool>,
    env: &mut Env,
) -> Result<(), JsonLdError> {
    if local.get(term).is_some() && defined.get(term) != Some(&true) {
        create_term_definition(active, local, term, defined, false, false, env)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------
// RFC 3986 §5 reference resolution (used for `@base`, remote-context URLs, and
// document-relative IRI expansion). Kept dependency-free and self-contained.
// ---------------------------------------------------------------------------------------

/// The five components of a URI reference (RFC 3986 §3).
struct UriRef<'a> {
    scheme: Option<&'a str>,
    authority: Option<&'a str>,
    path: &'a str,
    query: Option<&'a str>,
    fragment: Option<&'a str>,
}

/// Splits a URI reference into its components (RFC 3986 §3.2 / Appendix B).
fn split_uri(input: &str) -> UriRef<'_> {
    let (rest, fragment) = match input.find('#') {
        Some(i) => (&input[..i], Some(&input[i + 1..])),
        None => (input, None),
    };
    let (rest, query) = match rest.find('?') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    let (scheme, rest) = match rest.find(':') {
        Some(i) if is_valid_scheme(&rest[..i]) => (Some(&rest[..i]), &rest[i + 1..]),
        _ => (None, rest),
    };
    let (authority, path) = if let Some(after) = rest.strip_prefix("//") {
        let end = after.find('/').unwrap_or(after.len());
        (Some(&after[..end]), &after[end..])
    } else {
        (None, rest)
    };
    UriRef {
        scheme,
        authority,
        path,
        query,
        fragment,
    }
}

/// Removes `.` and `..` complete path segments (RFC 3986 §5.2.4).
fn remove_dot_segments(path: &str) -> String {
    let mut input = path.to_string();
    let mut output = String::new();
    while !input.is_empty() {
        if let Some(r) = input.strip_prefix("../") {
            input = r.to_string();
        } else if let Some(r) = input.strip_prefix("./") {
            input = r.to_string();
        } else if let Some(r) = input.strip_prefix("/./") {
            input = format!("/{}", r);
        } else if input == "/." {
            input = "/".to_string();
        } else if let Some(r) = input.strip_prefix("/../") {
            input = format!("/{}", r);
            remove_last_segment(&mut output);
        } else if input == "/.." {
            input = "/".to_string();
            remove_last_segment(&mut output);
        } else if input == "." || input == ".." {
            input.clear();
        } else {
            let start = usize::from(input.starts_with('/'));
            let next = match input[start..].find('/') {
                Some(i) => start + i,
                None => input.len(),
            };
            output.push_str(&input[..next]);
            input = input[next..].to_string();
        }
    }
    output
}

/// Removes the last `/`-delimited segment (and its preceding `/`) from `output`.
fn remove_last_segment(output: &mut String) {
    match output.rfind('/') {
        Some(i) => output.truncate(i),
        None => output.clear(),
    }
}

/// Merges a relative reference path with a base's path (RFC 3986 §5.2.3).
fn merge(base: &UriRef<'_>, ref_path: &str) -> String {
    if base.authority.is_some() && base.path.is_empty() {
        format!("/{}", ref_path)
    } else {
        match base.path.rfind('/') {
            Some(i) => format!("{}{}", &base.path[..=i], ref_path),
            None => ref_path.to_string(),
        }
    }
}

/// Recomposes URI components into a string (RFC 3986 §5.3).
fn recompose(
    scheme: Option<&str>,
    authority: Option<&str>,
    path: &str,
    query: Option<&str>,
    fragment: Option<&str>,
) -> String {
    let mut s = String::new();
    if let Some(sc) = scheme {
        s.push_str(sc);
        s.push(':');
    }
    if let Some(a) = authority {
        s.push_str("//");
        s.push_str(a);
    }
    s.push_str(path);
    if let Some(q) = query {
        s.push('?');
        s.push_str(q);
    }
    if let Some(f) = fragment {
        s.push('#');
        s.push_str(f);
    }
    s
}

/// Resolves a URI `reference` against a `base` URI (RFC 3986 §5.2.2). Used for `@base`,
/// remote-context URL resolution, and document-relative IRI expansion.
pub(crate) fn resolve_iri(base: &str, reference: &str) -> String {
    let r = split_uri(reference);
    let b = split_uri(base);

    let (t_scheme, t_authority, t_path, t_query);
    if r.scheme.is_some() {
        t_scheme = r.scheme;
        t_authority = r.authority;
        t_path = remove_dot_segments(r.path);
        t_query = r.query;
    } else {
        if r.authority.is_some() {
            t_authority = r.authority;
            t_path = remove_dot_segments(r.path);
            t_query = r.query;
        } else if r.path.is_empty() {
            t_path = b.path.to_string();
            t_query = if r.query.is_some() { r.query } else { b.query };
            t_authority = b.authority;
        } else {
            if r.path.starts_with('/') {
                t_path = remove_dot_segments(r.path);
            } else {
                t_path = remove_dot_segments(&merge(&b, r.path));
            }
            t_query = r.query;
            t_authority = b.authority;
        }
        t_scheme = b.scheme;
    }
    recompose(t_scheme, t_authority, &t_path, t_query, r.fragment)
}

/// Generates a relative-reference from `base` to `iri` — the inverse of
/// [`resolve_iri`] (RFC 3986 §5.3 recomposition in reverse).
///
/// Used by IRI Compaction (JSON-LD 1.1 API §7.1 step 6) to produce a
/// base-relative compact form instead of the literal `strip_prefix` fallback.
/// The output satisfies the round-trip property:
/// `resolve_iri(base, relativize_iri(base, iri).unwrap()) == iri`.
///
/// Returns `None` when `iri` cannot be expressed relative to `base`:
/// - different URI scheme
/// - different authority (host/port)
///
/// [SONNET-4.6] (sq-90mu3) §7.1 step 6 defect fix: the prior
/// `iri.strip_prefix(base)` only handled the literal-prefix case
/// (`http://ex/a/b` → `c` when base is `http://ex/a/b` → ∅); this helper
/// covers same-directory, child-path, parent-traversal, and
/// query/fragment preservation.
pub(crate) fn relativize_iri(base: &str, iri: &str) -> Option<String> {
    let b = split_uri(base);
    let r = split_uri(iri);

    // Scheme and authority must agree — otherwise the IRI cannot be expressed
    // as a relative reference and the caller should fall through to the
    // absolute form.
    if b.scheme != r.scheme || b.authority != r.authority {
        return None;
    }

    // [FABLE-5] (sq-oy1f.27) Same-document references (RFC 3986 §4.4): when the
    // paths agree, prefer a fragment-only reference (queries also agreeing) or a
    // query reference over re-stating the last path segment — the forms the W3C
    // compact suite expects for `#fragment` / `?query` targets.
    if b.path == r.path && !r.path.is_empty() {
        if b.query == r.query {
            if let Some(f) = r.fragment {
                return Some(format!("#{}", f));
            }
        }
        if let Some(q) = r.query {
            let mut s = format!("?{}", q);
            if let Some(f) = r.fragment {
                s.push('#');
                s.push_str(f);
            }
            return Some(s);
        }
        // Target has no query: fall through to the path-based form (a bare
        // fragment reference would wrongly inherit the base's query).
    }

    // Base "directory": path up to and including the last '/'.
    // e.g. "/a/b" → "/a/"   "/a/" → "/a/"   "/" → "/"
    let base_dir_end = b.path.rfind('/').map(|i| i + 1).unwrap_or(0);
    let base_dir = &b.path[..base_dir_end];
    let base_segs: Vec<&str> = base_dir.split('/').filter(|s| !s.is_empty()).collect();

    // IRI directory and filename:
    // "/a/c"  → dir="/a/"  file="c"
    // "/a/c/" → dir="/a/c/" file="" (directory IRI)
    let iri_dir_end = r.path.rfind('/').map(|i| i + 1).unwrap_or(0);
    let iri_dir = &r.path[..iri_dir_end];
    let iri_file = &r.path[iri_dir_end..];
    let iri_dir_segs: Vec<&str> = iri_dir.split('/').filter(|s| !s.is_empty()).collect();

    // Common directory-segment prefix length.
    let common = base_segs
        .iter()
        .zip(iri_dir_segs.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Number of `../` hops needed to reach the common ancestor.
    let up = base_segs.len() - common;
    // IRI-unique directory segments after the common prefix.
    let tail_dir = &iri_dir_segs[common..];

    // Build the relative path.
    let mut rel = String::new();
    for _ in 0..up {
        rel.push_str("../");
    }
    for seg in tail_dir {
        rel.push_str(seg);
        rel.push('/');
    }
    // If iri ends with '/' and the relative path so far is empty, we need
    // "./" so it resolves to the directory rather than the current document.
    if iri_file.is_empty() && rel.is_empty() {
        rel.push_str("./");
    } else {
        rel.push_str(iri_file);
    }

    // Preserve query and fragment from the target IRI.
    let mut result = rel;
    if let Some(q) = r.query {
        result.push('?');
        result.push_str(q);
    }
    if let Some(f) = r.fragment {
        result.push('#');
        result.push_str(f);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_and_absolute_iri_predicates() {
        assert!(is_valid_scheme("http"));
        assert!(is_valid_scheme("urn"));
        assert!(!is_valid_scheme("1http")); // must start with ALPHA
        assert!(!is_valid_scheme(""));
        assert!(is_absolute_iri("http://example.org/a"));
        assert!(is_absolute_iri("mailto:a@example.org"));
        assert!(!is_absolute_iri("example.org/a"));
        assert!(!is_absolute_iri("_:b0"));
        assert!(is_blank_node("_:b0"));
        assert!(!is_blank_node("http://x"));
    }

    /// [SONNET-4.6] (PR #4610 review) `pct-encoded = "%" HEXDIG HEXDIG` — a malformed
    /// escape makes the string a non-IRI, so it must not pass as a datatype IRI. The
    /// original character-denylist check accepted all of these.
    #[test]
    fn malformed_percent_escapes_are_not_absolute_iris() {
        assert!(!is_absolute_iri("http://example.com/%ZZ")); // non-hex digits
        assert!(!is_absolute_iri("http://example.com/%")); // bare, at end
        assert!(!is_absolute_iri("http://example.com/%A")); // truncated, at end
        assert!(!is_absolute_iri("http://example.com/%2")); // truncated, at end
        assert!(!is_absolute_iri("http://example.com/%A/b")); // truncated, mid-string
        assert!(!is_absolute_iri("http://example.com/%%20")); // `%` opening `%2` + `0`
        assert!(!is_absolute_iri("http://example.com/%2G")); // second digit non-hex
        assert!(!is_absolute_iri("http://example.com/%G2")); // first digit non-hex
    }

    /// The complement of the above: well-formed escapes must keep passing — over-rejection
    /// here would silently cost conformance passes at every `is_absolute_iri` call site.
    #[test]
    fn well_formed_percent_escapes_are_absolute_iris() {
        assert!(is_absolute_iri("http://example.com/a%20b"));
        assert!(is_absolute_iri("http://example.com/%C3%A9"));
        assert!(is_absolute_iri("http://example.com/%ff%FF%aB")); // case-insensitive HEXDIG
        assert!(is_absolute_iri("http://example.com/p?q=%2F#%2F"));
    }

    /// RFC 3987 `ucschar` / `iprivate`: a native Unicode IRI is well-formed and must not be
    /// rejected, while the code points outside those ranges (C1 controls, noncharacters,
    /// the plane-14 tag block) are not IRI characters.
    #[test]
    fn unicode_iri_code_points_follow_ucschar_and_iprivate() {
        // Positive: ucschar in the BMP, in the CJK compatibility range, and in plane 1;
        // plus an iprivate private-use code point.
        assert!(is_absolute_iri("http://example.com/na\u{ef}ve"));
        assert!(is_absolute_iri("http://\u{4f8b}\u{3048}.jp/\u{30d1}\u{30b9}"));
        assert!(is_absolute_iri("http://example.com/\u{f900}"));
        assert!(is_absolute_iri("http://example.com/\u{10000}"));
        assert!(is_absolute_iri("http://example.com/\u{e000}"));

        // Negative: C1 control, BMP noncharacters, and the excluded plane-14 tag block.
        assert!(!is_absolute_iri("http://example.com/\u{80}"));
        assert!(!is_absolute_iri("http://example.com/\u{9f}"));
        assert!(!is_absolute_iri("http://example.com/\u{fdd0}"));
        assert!(!is_absolute_iri("http://example.com/\u{fffe}"));
        assert!(!is_absolute_iri("http://example.com/\u{1fffe}"));
        assert!(!is_absolute_iri("http://example.com/\u{e0001}"));
    }

    /// The excluded-delimiter and control classes the original check already covered stay
    /// rejected (a regression tripwire for the rewrite into an allowlist).
    #[test]
    fn excluded_ascii_delimiters_are_not_absolute_iris() {
        assert!(!is_absolute_iri("http://example.com/baz z"));
        for bad in ['<', '>', '"', '{', '}', '|', '\\', '^', '`'] {
            let s = format!("http://example.com/{}", bad);
            assert!(!is_absolute_iri(&s), "should reject {:?}", bad);
        }
        assert!(!is_absolute_iri("http://example.com/\u{7f}")); // DEL
        assert!(!is_absolute_iri("http://example.com/a\tb"));
    }

    // RFC 3986 §5.4 normal-example reference-resolution vectors.
    #[test]
    fn rfc3986_normal_examples() {
        let base = "http://a/b/c/d;p?q";
        assert_eq!(resolve_iri(base, "g"), "http://a/b/c/g");
        assert_eq!(resolve_iri(base, "./g"), "http://a/b/c/g");
        assert_eq!(resolve_iri(base, "g/"), "http://a/b/c/g/");
        assert_eq!(resolve_iri(base, "/g"), "http://a/g");
        assert_eq!(resolve_iri(base, "?y"), "http://a/b/c/d;p?y");
        assert_eq!(resolve_iri(base, "g?y"), "http://a/b/c/g?y");
        assert_eq!(resolve_iri(base, "#s"), "http://a/b/c/d;p?q#s");
        assert_eq!(resolve_iri(base, "g#s"), "http://a/b/c/g#s");
        assert_eq!(resolve_iri(base, ""), "http://a/b/c/d;p?q");
        assert_eq!(resolve_iri(base, "."), "http://a/b/c/");
        assert_eq!(resolve_iri(base, ".."), "http://a/b/");
        assert_eq!(resolve_iri(base, "../.."), "http://a/");
        assert_eq!(resolve_iri(base, "../../g"), "http://a/g");
    }

    // RFC 3986 §5.4.2 abnormal dot-segment examples.
    #[test]
    fn rfc3986_abnormal_dot_segments() {
        let base = "http://a/b/c/d;p?q";
        assert_eq!(resolve_iri(base, "../../../g"), "http://a/g");
        assert_eq!(resolve_iri(base, "/./g"), "http://a/g");
        assert_eq!(resolve_iri(base, "g."), "http://a/b/c/g.");
        assert_eq!(resolve_iri(base, ".g"), "http://a/b/c/.g");
        assert_eq!(resolve_iri(base, "g/./h"), "http://a/b/c/g/h");
        assert_eq!(resolve_iri(base, "g/../h"), "http://a/b/c/h");
    }

    #[test]
    fn absolute_reference_keeps_its_own_scheme() {
        assert_eq!(
            resolve_iri("http://a/b", "https://c/d?x#y"),
            "https://c/d?x#y"
        );
    }

    // [SONNET-4.6] Issue #3714 — mutation tripwire for the dependency-free RFC
    // 3986 implementation. oxiri remains test-only so the crate's zero-mandatory-
    // dependency design is preserved.
    #[test]
    fn resolve_iri_matches_oxiri_on_shared_corpus() {
        let cases = [
            // RFC 3986 §5.4 normal examples.
            ("http://a/b/c/d;p?q", "g:h"),
            ("http://a/b/c/d;p?q", "g"),
            ("http://a/b/c/d;p?q", "./g"),
            ("http://a/b/c/d;p?q", "g/"),
            ("http://a/b/c/d;p?q", "/g"),
            ("http://a/b/c/d;p?q", "//g"),
            ("http://a/b/c/d;p?q", "?y"),
            ("http://a/b/c/d;p?q", "g?y"),
            ("http://a/b/c/d;p?q", "#s"),
            ("http://a/b/c/d;p?q", "g#s"),
            ("http://a/b/c/d;p?q", "g?y#s"),
            ("http://a/b/c/d;p?q", ";x"),
            ("http://a/b/c/d;p?q", "g;x"),
            ("http://a/b/c/d;p?q", "g;x?y#s"),
            ("http://a/b/c/d;p?q", ""),
            ("http://a/b/c/d;p?q", "."),
            ("http://a/b/c/d;p?q", "./"),
            ("http://a/b/c/d;p?q", ".."),
            ("http://a/b/c/d;p?q", "../"),
            ("http://a/b/c/d;p?q", "../g"),
            ("http://a/b/c/d;p?q", "../.."),
            ("http://a/b/c/d;p?q", "../../"),
            ("http://a/b/c/d;p?q", "../../g"),
            // RFC 3986 §5.4.2 abnormal examples.
            ("http://a/b/c/d;p?q", "../../../g"),
            ("http://a/b/c/d;p?q", "../../../../g"),
            ("http://a/b/c/d;p?q", "/./g"),
            ("http://a/b/c/d;p?q", "/../g"),
            ("http://a/b/c/d;p?q", "g."),
            ("http://a/b/c/d;p?q", ".g"),
            ("http://a/b/c/d;p?q", "g.."),
            ("http://a/b/c/d;p?q", "..g"),
            ("http://a/b/c/d;p?q", "./../g"),
            ("http://a/b/c/d;p?q", "./g/."),
            ("http://a/b/c/d;p?q", "g/./h"),
            ("http://a/b/c/d;p?q", "g/../h"),
            ("http://a/b/c/d;p?q", "g;x=1/./y"),
            ("http://a/b/c/d;p?q", "g;x=1/../y"),
            ("http://a/b/c/d;p?q", "g?y/./x"),
            ("http://a/b/c/d;p?q", "g?y/../x"),
            ("http://a/b/c/d;p?q", "g#s/./x"),
            ("http://a/b/c/d;p?q", "g#s/../x"),
            ("http://a/b/c/d;p?q", "http:g"),
            // Existing resolver cases and JSON-LD @base/document-relative shapes.
            ("http://a/b", "https://c/d?x#y"),
            ("https://example.com/doc", "#node"),
            ("https://example.com/a/context.jsonld", "../vocab/term"),
            ("https://example.com/a/", "child"),
            ("https://example.com", "relative"),
            ("https://example.com/a?old=1", "?new=2"),
            ("https://example.com/a?old=1", "#fragment"),
            ("urn:example:base", "next"),
        ];

        for (base, reference) in cases {
            let oxiri_base = oxiri::Iri::parse(base.to_string()).unwrap();
            let Ok(expected) = oxiri_base.resolve(reference) else {
                continue;
            };
            assert_eq!(
                resolve_iri(base, reference),
                expected.into_inner(),
                "base={base:?}, reference={reference:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // relativize_iri — round-trip correctness matrix
    // [SONNET-4.6] (sq-90mu3) Fix for §7.1 step 6 base-relative compaction.
    // Each case asserts: resolve_iri(base, relativize_iri(base, iri)) == iri
    // -----------------------------------------------------------------------

    fn round_trip(base: &str, iri: &str) -> String {
        let rel = relativize_iri(base, iri)
            .unwrap_or_else(|| panic!("expected Some for base={base:?} iri={iri:?}"));
        let resolved = resolve_iri(base, &rel);
        assert_eq!(
            resolved, iri,
            "round-trip failed: base={base:?} iri={iri:?} rel={rel:?}"
        );
        rel
    }

    /// Same directory: shared parent, different filename.
    #[test]
    fn relativize_same_directory() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/c");
        assert_eq!(rel, "c");
    }

    /// Child path: iri is deeper than base.
    #[test]
    fn relativize_child_path() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/b/c/d");
        assert_eq!(rel, "b/c/d");
    }

    /// Parent traversal: iri is in a sibling directory.
    #[test]
    fn relativize_parent_traversal() {
        let rel = round_trip("http://ex/a/b/c", "http://ex/a/d");
        assert_eq!(rel, "../d");
    }

    /// Query preservation: relative reference keeps the query string.
    #[test]
    fn relativize_query_preservation() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/c?q=1");
        assert_eq!(rel, "c?q=1");
    }

    /// Fragment preservation: relative reference keeps the fragment.
    #[test]
    fn relativize_fragment_preservation() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/c#sec");
        assert_eq!(rel, "c#sec");
    }

    /// Query and fragment together.
    #[test]
    fn relativize_query_and_fragment() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/c?q=1#sec");
        assert_eq!(rel, "c?q=1#sec");
    }

    /// Same-file reference (iri == base): resolves back to the same resource.
    #[test]
    fn relativize_same_file() {
        let rel = round_trip("http://ex/a/b", "http://ex/a/b");
        // The relative ref must resolve back; "b" is the canonical form here.
        assert_eq!(rel, "b");
    }

    /// Scheme mismatch: returns None (cannot relativize across schemes).
    #[test]
    fn relativize_scheme_mismatch_returns_none() {
        assert!(relativize_iri("http://ex/a", "https://ex/a").is_none());
    }

    /// Authority mismatch: returns None (cannot relativize across hosts).
    #[test]
    fn relativize_authority_mismatch_returns_none() {
        assert!(relativize_iri("http://ex/a", "http://other.ex/a").is_none());
    }

    /// Root-relative base: iri one level deep from the server root.
    #[test]
    fn relativize_root_base() {
        let rel = round_trip("http://ex/", "http://ex/foo");
        assert_eq!(rel, "foo");
    }

    /// Deep parent traversal: two levels up.
    #[test]
    fn relativize_two_levels_up() {
        let rel = round_trip("http://ex/a/b/c/d", "http://ex/e");
        assert_eq!(rel, "../../../e");
    }
}
