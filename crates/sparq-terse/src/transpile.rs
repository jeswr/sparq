//! The verifiable transpiler skeleton (design §3/§6, Phase 1).
//!
//! [`terse_to_sparql`] is the crate's single public contract. It takes a *terse* query
//! string and returns an [`Expansion`] whose [`Expansion::canonical_sparql`] is **always**
//! standard, conformant SPARQL — the exact text the engine will run and the agent can
//! inspect. There is no path where the engine executes something the agent cannot read
//! (design §6.1, "echo the canonical expanded query, always").
//!
//! Two invariants this skeleton enforces mechanically — the *silent-rewrite canary*
//! (design §6.7):
//!
//! 1. **Conformance.** The emitted `canonical_sparql` is re-parsed under the unmodified
//!    vendored `spargebra` parser. If it does not parse, [`terse_to_sparql`] returns
//!    [`TerseError::CanaryFailed`] rather than handing the agent invalid SPARQL — the
//!    transpiler can only ever emit a query the engine could actually run.
//! 2. **Pass-through identity.** A query with no terse constructs is emitted *byte-for-byte
//!    unchanged*. The transpiler never silently rewrites canonical SPARQL.
//!
//! The one terse construct this crate defines is `V("phrase")` concept resolution
//! ([`crate::resolve`], design §3.3). Detection of `V(...)` is feature-independent so the
//! *default* (no-`vectors`) build fails **loudly** with [`TerseError::FeatureRequired`]
//! instead of passing an un-expanded `V(...)` through to the parser (which would be a
//! confusing downstream parse error). The actual resolution lives behind the `vectors`
//! feature.

use crate::error::TerseError;

/// The result of transpiling a terse query: the canonical SPARQL to execute, plus the
/// verification surface (resolutions + warnings) the agent inspects.
///
/// `canonical_sparql` is the *whole* contract: it is standard SPARQL, it is what runs, and
/// it is what the agent must be shown. `resolutions` and `warnings` let the agent verify
/// *how* any terse construct was expanded (design §6.2).
#[derive(Debug, Clone, PartialEq)]
pub struct Expansion {
    /// The canonical, conformant SPARQL the engine will run. Re-parsed under `spargebra`
    /// before return (the canary), so it is guaranteed to parse.
    pub canonical_sparql: String,
    /// One entry per terse concept-resolution (`V("phrase")`) that was expanded, echoing
    /// the resolved IRI, its score, the runner-up, the confidence and which method fired
    /// (design §6.2). Empty for a pure pass-through. The type is always present (so callers
    /// compile in both feature states); it is only ever populated behind the `vectors`
    /// feature.
    pub resolutions: Vec<crate::resolve::Resolution>,
    /// Non-fatal advisories surfaced to the agent (e.g. a close runner-up flagging
    /// ambiguity). Never hides a rewrite — a rewrite that *changes intent* is a hard error,
    /// not a warning.
    pub warnings: Vec<String>,
}

impl Expansion {
    /// A pure pass-through expansion: the input was already canonical SPARQL, emitted
    /// unchanged, with no resolutions and no warnings.
    fn passthrough(sparql: String) -> Expansion {
        Expansion {
            canonical_sparql: sparql,
            resolutions: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Transpiles a terse query into canonical SPARQL (design §3, Phase 1 contract).
///
/// In the *default* build (no `vectors` feature) this is a verifiable **identity
/// pass-through**: canonical SPARQL is returned unchanged after the conformance canary,
/// and any `V("phrase")` construct is rejected with [`TerseError::FeatureRequired`] — the
/// surface never silently drops or mis-expands a terse construct it cannot resolve.
///
/// With the `vectors` feature plus a resolution [`crate::resolve::ResolveCtx`], use
/// [`terse_to_sparql_with`] instead; this entry point keeps the no-context, pass-through
/// contract that is always available.
///
/// # Errors
/// - [`TerseError::FeatureRequired`] if the input uses `V(...)` but no resolution context
///   was supplied (this entry point) or the `vectors` feature is off.
/// - [`TerseError::CanaryFailed`] if the (otherwise canonical) input does not parse under
///   `spargebra` — surfaced loudly rather than handing back invalid SPARQL.
pub fn terse_to_sparql(src: &str) -> Result<Expansion, TerseError> {
    // The only terse construct is V(...): without a resolution context there is nothing to
    // expand, so this is a strict pass-through. Detect V(...) and fail loudly (design §6.6:
    // loud-fail beats silent-wrong) rather than letting an un-expanded `V(...)` reach the
    // parser as a baffling syntax error.
    if let Some(phrase) = find_first_v_phrase(src) {
        return Err(TerseError::FeatureRequired {
            phrase,
            why: "V(\"...\") concept resolution requires the `vectors` feature and a \
                  resolution context (terse_to_sparql_with)"
                .to_string(),
        });
    }
    let exp = Expansion::passthrough(src.to_string());
    canary(&exp.canonical_sparql)?;
    Ok(exp)
}

/// Transpiles a terse query, resolving every `V("phrase")` construct against a
/// [`crate::resolve::ResolveCtx`] (design §3.3 / §6, Phase 2). Each `V("phrase")` is
/// replaced by the canonical `<iri>` it resolves to (lexical-first, confidence-gated,
/// staleness-guarded), and the resulting canonical SPARQL is run through the silent-rewrite
/// canary before return. Every resolution is echoed in [`Expansion::resolutions`] so the
/// agent can audit each bind (design §6.2). A close runner-up is surfaced as a warning.
///
/// `embed` is an optional embedder for the vector FALLBACK: it is called *only* for a phrase
/// that fails lexical linking, to produce the query vector the vector search needs. Pass a
/// closure that returns `None` (or use [`terse_to_sparql`] for canonical-only input) to stay
/// on the no-model, lexical-only path; the design (§6/§9 Q5) keeps the embedding model the
/// caller's explicit, opt-in dependency rather than forcing one into this crate.
///
/// # Errors
/// - [`TerseError::Unresolved`] if any `V("phrase")` cannot be confidently resolved (below
///   the score floor, ambiguous, or no candidate) — the bind is refused loudly, never
///   guessed (design §6.3).
/// - [`TerseError::StaleStore`] if the vector fallback's store is stale vs the graph
///   (design §6.5).
/// - [`TerseError::CanaryFailed`] if the expanded SPARQL does not parse — surfaced rather
///   than executed.
#[cfg(feature = "vectors")]
pub fn terse_to_sparql_with(
    src: &str,
    ctx: &crate::resolve::ResolveCtx<'_>,
    mut embed: impl FnMut(&str) -> Option<Vec<f32>>,
) -> Result<Expansion, TerseError> {
    let spans = v_spans(src);
    if spans.is_empty() {
        // No terse construct: strict pass-through (still canary-checked).
        let exp = Expansion::passthrough(src.to_string());
        canary(&exp.canonical_sparql)?;
        return Ok(exp);
    }
    let mut out = String::with_capacity(src.len());
    let mut cursor = 0usize;
    let mut resolutions = Vec::with_capacity(spans.len());
    let mut warnings = Vec::new();
    for span in &spans {
        // Copy the source up to this V(...) verbatim.
        out.push_str(&src[cursor..span.start]);
        // Resolve: lexical-first; the embedder is consulted only for the vector fallback,
        // and only when lexical returns nothing (the ctx decides — we pre-embed lazily).
        let query_vec = embed(&span.phrase);
        let res = ctx.resolve(&span.phrase, query_vec.as_deref())?;
        // Splice the canonical <iri> in place of the V(...) construct.
        out.push('<');
        out.push_str(&res.iri);
        out.push('>');
        // Surface a close runner-up as a visible ambiguity warning (design §6.2).
        if let (Some(runner), Some(rs)) = (&res.runner_up, res.runner_up_score) {
            warnings.push(format!(
                "V(\"{}\") -> <{}> (score {:.3}, confidence {:.3}) with a runner-up <{}> \
                 (score {:.3}); verify the bind",
                span.phrase, res.iri, res.score, res.confidence, runner, rs
            ));
        }
        resolutions.push(res);
        cursor = span.end;
    }
    out.push_str(&src[cursor..]);
    canary(&out)?;
    Ok(Expansion {
        canonical_sparql: out,
        resolutions,
        warnings,
    })
}

/// Runs the silent-rewrite canary (design §6.7): re-parse the canonical output under the
/// unmodified `spargebra` parser. The transpiler must only ever emit SPARQL the engine
/// could run. We accept either a query or an update (the surface transpiles both).
pub(crate) fn canary(canonical_sparql: &str) -> Result<(), TerseError> {
    use spargebra::SparqlParser;
    // The surface transpiles both queries and updates, so the emission is conformant if it
    // parses as EITHER. Capture the query parse error (the common case) for the diagnostic.
    let query_err = match SparqlParser::new().parse_query(canonical_sparql) {
        Ok(_) => return Ok(()),
        Err(e) => e.to_string(),
    };
    if SparqlParser::new().parse_update(canonical_sparql).is_ok() {
        return Ok(());
    }
    Err(TerseError::CanaryFailed {
        sparql: canonical_sparql.to_string(),
        parse_error: query_err,
    })
}

/// Scans `src` for the first `V("phrase")` (or `V('phrase')`) construct OUTSIDE of any
/// SPARQL string/IRI literal, returning the inner phrase. Returns `None` if there is no
/// such construct. This is the feature-independent *detector*: it is deliberately a tiny,
/// conservative lexical scan (the design forbids touching the vendored grammar), and it
/// only recognises a single, fixed shape — `V` (case-sensitive) immediately followed by
/// `(` then a quoted string then `)`. Anything else passes through untouched.
///
/// We skip over SPARQL string literals (`'...'`, `"..."`, `'''...'''`, `"""..."""`) and
/// IRIs (`<...>`) so a `V(` appearing *inside* a literal/IRI is never mistaken for the
/// construct — the construct is a top-level token, not text inside a value.
pub(crate) fn find_first_v_phrase(src: &str) -> Option<String> {
    scan_v_constructs(src, |phrase, _start, _end| Some(phrase.to_string())).into_iter().next()
}

/// Where a `V("phrase")` construct sits in the source: the inner phrase and the byte range
/// `[start, end)` of the whole `V(...)` span (so a caller can splice a replacement in).
/// Only used by the `vectors`-gated [`terse_to_sparql_with`] expansion path.
#[cfg(feature = "vectors")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VSpan {
    pub phrase: String,
    pub start: usize,
    pub end: usize,
}

/// Returns every `V("phrase")` construct in `src`, in source order, as [`VSpan`]s.
#[cfg(feature = "vectors")]
pub(crate) fn v_spans(src: &str) -> Vec<VSpan> {
    scan_v_constructs(src, |phrase, start, end| {
        Some(VSpan {
            phrase: phrase.to_string(),
            start,
            end,
        })
    })
}

/// The shared lexer for `V("phrase")` constructs. Walks `src` char-by-char, tracking
/// whether we are inside a string literal or IRI (where a `V(` must be ignored), and for
/// each top-level `V(` immediately followed by a single quoted string and a `)`, invokes
/// `f(phrase, start, end)`; non-`None` results are collected in source order.
fn scan_v_constructs<T>(
    src: &str,
    mut f: impl FnMut(&str, usize, usize) -> Option<T>,
) -> Vec<T> {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < n {
        let c = bytes[i];
        match c {
            b'<' => {
                // Skip an IRI ref <...> (no nested '<', '>' ends it; newline cancels it,
                // matching SPARQL's IRIREF lexing where these are illegal inside).
                let mut j = i + 1;
                while j < n && bytes[j] != b'>' && bytes[j] != b'\n' {
                    j += 1;
                }
                i = if j < n { j + 1 } else { n };
            }
            b'#' => {
                // Line comment to end-of-line.
                let mut j = i + 1;
                while j < n && bytes[j] != b'\n' {
                    j += 1;
                }
                i = j;
            }
            b'"' | b'\'' => {
                i = skip_string(bytes, i);
            }
            b'V' => {
                // Must be a standalone `V` token: the preceding char (if any) must not be
                // part of an identifier (so `?fooV(` or `abcV(` are NOT the construct).
                let prev_ok = i == 0 || !is_ident_char(bytes[i - 1]);
                if prev_ok {
                    if let Some((phrase, end)) = parse_v_call(bytes, i) {
                        if let Some(t) = f(&phrase, i, end) {
                            out.push(t);
                        }
                        i = end;
                        continue;
                    }
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    out
}

/// `true` if `b` can appear inside a SPARQL identifier / prefixed-name local part (so a `V`
/// glued to one is not a standalone token).
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b':' || b == b'?' || b == b'$'
}

/// At `bytes[start] == b'V'`, tries to parse `V("phrase")` / `V('phrase')` allowing
/// whitespace around the parens and the string. On success returns `(phrase, end)` where
/// `end` is the byte index just past the closing `)`. The phrase is the *unescaped* string
/// content (we honour `\"`, `\'`, `\\`, `\n`, `\t`, `\r` — the subset SPARQL strings use).
fn parse_v_call(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let n = bytes.len();
    let mut i = start + 1; // past 'V'
    i = skip_ws(bytes, i);
    if i >= n || bytes[i] != b'(' {
        return None;
    }
    i += 1;
    i = skip_ws(bytes, i);
    if i >= n || (bytes[i] != b'"' && bytes[i] != b'\'') {
        return None;
    }
    let quote = bytes[i];
    i += 1;
    let mut phrase = String::new();
    while i < n {
        let b = bytes[i];
        if b == b'\\' && i + 1 < n {
            let esc = bytes[i + 1];
            match esc {
                b'"' => phrase.push('"'),
                b'\'' => phrase.push('\''),
                b'\\' => phrase.push('\\'),
                b'n' => phrase.push('\n'),
                b't' => phrase.push('\t'),
                b'r' => phrase.push('\r'),
                other => {
                    phrase.push('\\');
                    phrase.push(other as char);
                }
            }
            i += 2;
            continue;
        }
        if b == quote {
            i += 1;
            break;
        }
        if b == b'\n' {
            // Unterminated single-line string: not a valid V() call.
            return None;
        }
        // Push the raw byte; for multi-byte UTF-8 we copy the full sequence below.
        let ch_len = utf8_len(b);
        if i + ch_len > n {
            return None;
        }
        let s = std::str::from_utf8(&bytes[i..i + ch_len]).ok()?;
        phrase.push_str(s);
        i += ch_len;
    }
    i = skip_ws(bytes, i);
    if i >= n || bytes[i] != b')' {
        return None;
    }
    Some((phrase, i + 1))
}

/// Skips ASCII whitespace, returning the index of the first non-whitespace byte.
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// At `bytes[start]` being a quote, skips a SPARQL string literal (single- or
/// triple-quoted, honouring backslash escapes), returning the index just past it.
fn skip_string(bytes: &[u8], start: usize) -> usize {
    let n = bytes.len();
    let quote = bytes[start];
    // Triple-quoted?
    let triple = start + 2 < n && bytes[start + 1] == quote && bytes[start + 2] == quote;
    if triple {
        let mut i = start + 3;
        while i + 2 < n {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == quote && bytes[i + 1] == quote && bytes[i + 2] == quote {
                return i + 3;
            }
            i += 1;
        }
        return n;
    }
    let mut i = start + 1;
    while i < n {
        if bytes[i] == b'\\' && i + 1 < n {
            i += 2;
            continue;
        }
        if bytes[i] == quote || bytes[i] == b'\n' {
            return i + 1;
        }
        i += 1;
    }
    n
}

/// Byte-length of the UTF-8 sequence whose lead byte is `b`.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_is_byte_identical() {
        let q = "SELECT ?s WHERE { ?s <http://ex/p> ?o }";
        let e = terse_to_sparql(q).expect("canonical SPARQL passes through");
        assert_eq!(e.canonical_sparql, q, "pass-through must be byte-identical");
        assert!(e.resolutions.is_empty());
        assert!(e.warnings.is_empty());
    }

    #[test]
    fn canary_rejects_invalid_sparql() {
        let bad = "SELECT ?s WHERE { ?s ?p"; // unbalanced — does not parse
        let err = terse_to_sparql(bad).expect_err("invalid SPARQL must fail the canary");
        assert!(matches!(err, TerseError::CanaryFailed { .. }));
    }

    #[test]
    fn canary_accepts_update() {
        let u = "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }";
        let e = terse_to_sparql(u).expect("a valid UPDATE passes the canary");
        assert_eq!(e.canonical_sparql, u);
    }

    #[test]
    fn v_construct_detected_and_rejected_without_context() {
        let q = "SELECT ?f WHERE { ?f <http://ex/about> V(\"cardinality estimation\") }";
        let err = terse_to_sparql(q).expect_err("V() needs the vectors feature + a context");
        match err {
            TerseError::FeatureRequired { phrase, .. } => {
                assert_eq!(phrase, "cardinality estimation");
            }
            other => panic!("expected FeatureRequired, got {other:?}"),
        }
    }

    #[test]
    fn v_inside_a_string_literal_is_not_a_construct() {
        // A V( appearing inside a SPARQL string is data, not the construct.
        let q = "SELECT ?s WHERE { ?s <http://ex/p> \"V(\\\"x\\\")\" }";
        let e = terse_to_sparql(q).expect("V inside a literal is passed through");
        assert_eq!(e.canonical_sparql, q);
    }

    #[test]
    fn v_inside_an_iri_is_not_a_construct() {
        let q = "SELECT ?s WHERE { ?s <http://ex/V(x)> ?o }";
        let e = terse_to_sparql(q).expect("V inside an IRI is passed through");
        assert_eq!(e.canonical_sparql, q);
    }

    #[test]
    fn glued_v_is_not_a_construct() {
        // `?fooV("x")` — the V is glued to an identifier, not a standalone token. There is
        // no V() construct here; the text is left for the parser (which will reject it, but
        // via the canary, not a false V-detection).
        assert!(find_first_v_phrase("?fooV(\"x\")").is_none());
    }

    #[cfg(feature = "vectors")]
    #[test]
    fn v_spans_finds_all_in_order() {
        let q = "{ ?a <p> V(\"one\") . ?b <p> V('two') }";
        let spans = v_spans(q);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].phrase, "one");
        assert_eq!(spans[1].phrase, "two");
        // The spans must point at the actual V(...) text.
        assert_eq!(&q[spans[0].start..spans[0].end], "V(\"one\")");
        assert_eq!(&q[spans[1].start..spans[1].end], "V('two')");
    }

    #[test]
    fn v_escapes_are_unescaped() {
        let phrase = find_first_v_phrase("V(\"a\\\"b\")").unwrap();
        assert_eq!(phrase, "a\"b");
    }

    #[test]
    fn v_with_whitespace_around_parens() {
        let phrase = find_first_v_phrase("V ( \"spaced\" )").unwrap();
        assert_eq!(phrase, "spaced");
    }

    #[test]
    fn v_in_a_comment_is_ignored() {
        let q = "SELECT ?s WHERE { # V(\"commented\")\n ?s <http://ex/p> ?o }";
        let e = terse_to_sparql(q).expect("V in a comment is not a construct");
        assert_eq!(e.canonical_sparql, q);
    }
}
