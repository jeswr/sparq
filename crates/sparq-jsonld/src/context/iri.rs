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

/// [OPUS-5] sq-gzsky — True iff `s` is a WELL-FORMED absolute IRI, i.e. it parses against
/// the RFC 3987 `IRI` production
/// `scheme ":" ihier-part [ "?" iquery ] [ "#" ifragment ]`.
///
/// [`is_absolute_iri`] only inspects the scheme, which is the right (cheap) predicate
/// where the value has already been produced by IRI expansion. This stricter form is for
/// the places the spec says a processor MUST *validate* an authored IRI — notably a value
/// object's `@type` datatype (§5.1.2 step 15.5, W3C expand/#t0123: `"http://example.com/baz
/// z"` must raise `invalid typed value`).
///
/// Each component is checked against its own character class (`ipchar` / `iquery` /
/// `ifragment` / `iuserinfo` / `ireg-name`, with `iunreserved` covering the `ucschar`
/// ranges), and every `%` must introduce exactly two `HEXDIG` — so `http://example.com/%ZZ`
/// is rejected, unlike a character blacklist. Deliberate residual laxity, both beyond what
/// §5.1.2 needs: a bracketed `IP-literal` host is checked for *shape* (`HEXDIG` / `:` / `.`,
/// or an `IPvFuture` `v…`) rather than parsed as a numeric address, and a `port` is not
/// required to fit any integer width.
///
/// [SONNET-4.6] (PR #4041 review round 1) replaces the original scheme-plus-blacklist
/// form, which accepted invalid percent-encoding and ignored component boundaries.
pub(crate) fn is_well_formed_absolute_iri(s: &str) -> bool {
    // scheme ":" — the only part of the grammar `is_absolute_iri` covers.
    let Some(colon) = s.find(':') else {
        return false;
    };
    if !is_valid_scheme(&s[..colon]) {
        return false;
    }
    // Split the trailing components off, outermost first (RFC 3986 Appendix B).
    let rest = &s[colon + 1..];
    let (rest, fragment) = match rest.find('#') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    let (rest, query) = match rest.find('?') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    // ihier-part: "//" iauthority ipath-abempty / ipath-absolute / ipath-rootless / empty.
    let (authority, path) = match rest.strip_prefix("//") {
        Some(after) => {
            let end = after.find('/').unwrap_or(after.len());
            (Some(&after[..end]), &after[end..])
        }
        None => (None, rest),
    };
    if let Some(a) = authority {
        if !is_valid_iauthority(a) {
            return false;
        }
        // ipath-abempty: empty, or rooted at "/".
        if !path.is_empty() && !path.starts_with('/') {
            return false;
        }
    }
    is_valid_component(path, |c| is_ipchar(c) || c == '/')
        && query.is_none_or(|q| {
            is_valid_component(q, |c| is_ipchar(c) || c == '/' || c == '?' || is_iprivate(c))
        })
        && fragment.is_none_or(|f| is_valid_component(f, |c| is_ipchar(c) || c == '/' || c == '?'))
}

/// True iff every character of `s` is `allowed` *and* every `%` introduces two `HEXDIG`
/// (RFC 3987 `pct-encoded`). The percent-encoding rule is what a character blacklist
/// cannot express.
fn is_valid_component(s: &str, allowed: impl Fn(char) -> bool) -> bool {
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let (Some(hi), Some(lo)) = (chars.next(), chars.next()) else {
                return false;
            };
            if !hi.is_ascii_hexdigit() || !lo.is_ascii_hexdigit() {
                return false;
            }
        } else if !allowed(c) {
            return false;
        }
    }
    true
}

/// RFC 3987 `iauthority = [ iuserinfo "@" ] ihost [ ":" port ]`.
fn is_valid_iauthority(s: &str) -> bool {
    // `@` is legal inside neither iuserinfo nor ihost, so the LAST one delimits.
    let host_port = match s.rfind('@') {
        Some(i) => {
            if !is_valid_component(&s[..i], |c| is_iunreserved(c) || is_sub_delim(c) || c == ':') {
                return false;
            }
            &s[i + 1..]
        }
        None => s,
    };
    // IP-literal hosts are bracketed and may be followed by ":" port.
    if let Some(after_bracket) = host_port.strip_prefix('[') {
        let Some(end) = after_bracket.find(']') else {
            return false;
        };
        let literal = &after_bracket[..end];
        let literal_ok = match literal.strip_prefix('v') {
            // IPvFuture = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
            Some(future) => match future.split_once('.') {
                Some((ver, tail)) => {
                    !ver.is_empty()
                        && ver.chars().all(|c| c.is_ascii_hexdigit())
                        && !tail.is_empty()
                        && tail
                            .chars()
                            .all(|c| is_iunreserved(c) || is_sub_delim(c) || c == ':')
                }
                None => false,
            },
            // IPv6address (shape only — see the caller's doc comment).
            None => {
                !literal.is_empty()
                    && literal
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
            }
        };
        return literal_ok && is_valid_port_suffix(&after_bracket[end + 1..]);
    }
    // ireg-name / IPv4address (a character-class subset of ireg-name), plus ":" port.
    // ireg-name may be empty (`file:///path`), and a colon can only start the port.
    let (name, port_suffix) = match host_port.find(':') {
        Some(i) => (&host_port[..i], &host_port[i..]),
        None => (host_port, ""),
    };
    is_valid_component(name, |c| is_iunreserved(c) || is_sub_delim(c))
        && is_valid_port_suffix(port_suffix)
}

/// True iff `s` is either empty or `":" *DIGIT` (RFC 3986 `port`).
fn is_valid_port_suffix(s: &str) -> bool {
    match s.strip_prefix(':') {
        Some(port) => port.chars().all(|c| c.is_ascii_digit()),
        None => s.is_empty(),
    }
}

/// RFC 3987 `ipchar = iunreserved / pct-encoded / sub-delims / ":" / "@"` (the
/// `pct-encoded` alternative is handled by [`is_valid_component`]).
fn is_ipchar(c: char) -> bool {
    is_iunreserved(c) || is_sub_delim(c) || c == ':' || c == '@'
}

/// RFC 3987 `iunreserved = ALPHA / DIGIT / "-" / "." / "_" / "~" / ucschar`.
fn is_iunreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~') || is_ucschar(c)
}

/// RFC 3986 `sub-delims`.
fn is_sub_delim(c: char) -> bool {
    matches!(
        c,
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
    )
}

/// RFC 3987 `ucschar` — the non-ASCII code points an IRI may carry unescaped.
fn is_ucschar(c: char) -> bool {
    matches!(u32::from(c),
        0xA0..=0xD7FF
        | 0xF900..=0xFDCF
        | 0xFDF0..=0xFFEF
        | 0x1_0000..=0x1_FFFD
        | 0x2_0000..=0x2_FFFD
        | 0x3_0000..=0x3_FFFD
        | 0x4_0000..=0x4_FFFD
        | 0x5_0000..=0x5_FFFD
        | 0x6_0000..=0x6_FFFD
        | 0x7_0000..=0x7_FFFD
        | 0x8_0000..=0x8_FFFD
        | 0x9_0000..=0x9_FFFD
        | 0xA_0000..=0xA_FFFD
        | 0xB_0000..=0xB_FFFD
        | 0xC_0000..=0xC_FFFD
        | 0xD_0000..=0xD_FFFD
        | 0xE_1000..=0xE_FFFD)
}

/// RFC 3987 `iprivate` — private-use code points, legal only in `iquery`.
fn is_iprivate(c: char) -> bool {
    matches!(u32::from(c), 0xE000..=0xF8FF | 0xF_0000..=0xF_FFFD | 0x10_0000..=0x10_FFFD)
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

    // -----------------------------------------------------------------------
    // is_well_formed_absolute_iri — the RFC 3987 `IRI` production.
    // [SONNET-4.6] (PR #4041 review round 1) The predicate used to be
    // `is_absolute_iri` plus a character blacklist, so invalid percent-encoding
    // and component-misplaced characters slipped through.
    // -----------------------------------------------------------------------

    /// IRIs that must be accepted, one per grammar feature the validator has to
    /// admit (percent-encoding, userinfo, port, IP-literal, empty host, ucschar,
    /// iprivate-in-query, sub-delims, rootless path).
    const WELL_FORMED: &[&str] = &[
        "http://example.com/bazz",
        "http://example.com/%20quoted",
        "http://example.com/a%2Fb",
        "http://example.com/a?q=1&r=2#frag",
        "http://user:pw@example.com:8080/p",
        "http://example.com:8080/p",
        "http://[::1]:8080/p",
        "http://[v7.host]/p",
        "file:///etc/hosts",
        "http://example.com",
        "http://example.com/\u{3c0}",
        "http://example.com/a?q=\u{e000}",
        "http://example.com/a!$&'()*+,;=:@b",
        "urn:example:dt",
        "mailto:a@example.org",
    ];

    /// IRIs that must be rejected, paired with the rule each one violates.
    const MALFORMED: &[(&str, &str)] = &[
        ("http://example.com/%ZZ", "pct-encoded needs two HEXDIG"),
        ("http://example.com/%A", "truncated pct-encoded"),
        ("http://example.com/%", "bare percent"),
        ("http://example.com/baz z", "SPACE is not an ipchar"),
        ("http://example.com/a<b", "`<` is not an ipchar"),
        ("http://example.com/a|b", "`|` is not an ipchar"),
        ("http://example.com/a\u{7f}b", "DEL is not an ipchar"),
        ("http://example.com/a\u{0}b", "NUL is not an ipchar"),
        ("http://ex ample.com/p", "SPACE is not an ireg-name char"),
        ("http://exa/mple.com/%GG", "pct-encoded in a later segment"),
        ("http://example.com:port/p", "port must be *DIGIT"),
        ("http://example.com/a?q=%2", "truncated pct-encoded in iquery"),
        ("http://example.com/a#f%2", "truncated pct-encoded in ifragment"),
        ("http://[zzz]/p", "IP-literal shape"),
        ("http://[::1/p", "unterminated IP-literal"),
        ("http://example.com/a\u{e000}b", "iprivate is iquery-only"),
        ("example.com/no-scheme", "no scheme"),
        ("_:dt", "a blank node identifier is not an IRI"),
        ("1http://example.com", "scheme must start with ALPHA"),
    ];

    #[test]
    fn well_formed_iris_are_accepted() {
        for iri in WELL_FORMED {
            assert!(
                is_well_formed_absolute_iri(iri),
                "expected well-formed: {:?}",
                iri
            );
        }
    }

    #[test]
    fn malformed_iris_are_rejected() {
        for (iri, why) in MALFORMED {
            assert!(
                !is_well_formed_absolute_iri(iri),
                "expected malformed ({}): {:?}",
                why,
                iri
            );
        }
    }

    /// The stricter predicate is a strict refinement of [`is_absolute_iri`]: every
    /// well-formed IRI has a valid scheme, but not every valid scheme makes an IRI.
    #[test]
    fn well_formed_refines_is_absolute_iri() {
        for iri in WELL_FORMED {
            assert!(is_absolute_iri(iri), "{:?}", iri);
        }
        assert!(is_absolute_iri("http://example.com/%ZZ"));
        assert!(!is_well_formed_absolute_iri("http://example.com/%ZZ"));
    }

    /// Mutation tripwire for the hand-written grammar, mirroring
    /// `resolve_iri_matches_oxiri_on_shared_corpus`: oxiri stays a dev-dependency so
    /// the crate keeps zero mandatory dependencies, but its RFC 3987 parser pins the
    /// accept/reject verdict on both corpora.
    #[test]
    fn well_formed_matches_oxiri_on_shared_corpus() {
        for iri in WELL_FORMED {
            assert!(
                oxiri::Iri::parse(*iri).is_ok(),
                "corpus says well-formed but oxiri rejects: {:?}",
                iri
            );
        }
        for (iri, why) in MALFORMED {
            if !is_absolute_iri(iri) {
                continue; // scheme-level rejections are not oxiri's disagreement surface
            }
            assert!(
                oxiri::Iri::parse(*iri).is_err(),
                "corpus says malformed ({}) but oxiri accepts: {:?}",
                why,
                iri
            );
        }
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
