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

/// True iff `s` is an absolute IRI: a valid scheme followed by `:`.
pub(crate) fn is_absolute_iri(s: &str) -> bool {
    match s.find(':') {
        Some(colon) => is_valid_scheme(&s[..colon]),
        None => false,
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
