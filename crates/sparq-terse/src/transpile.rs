//! The verifiable pre-parse transpiler — Phase 1 of the LLM-ergonomic SPARQL surface
//! (`research/llm-ergonomic-sparql-surface.md` §3, §6, §8.2).
//!
//! 🤖 SPARQ agent. This module is the crate's single load-bearing contract: a *textual*
//! expansion that takes a terse / LLM-friendly query and **always emits canonical
//! SPARQL** which the unmodified vendored `spargebra` parser then accepts. It does NOT
//! modify the grammar — `vendor/spargebra` carries the W3C-conformance patches and must
//! stay untouched — so the engine only ever sees standard SPARQL.
//!
//! ## The soundness contract (the design's §6 envelope)
//!
//! 1. **Echo the canonical expansion, always.** [`Expansion::canonical_sparql`] is
//!    standard SPARQL that re-parses under `spargebra`; it is what executes and what the
//!    agent must be shown. There is no path where the engine runs something the caller
//!    cannot read — this is *stronger* than a lenient parser, which hides the rewrite.
//! 2. **The silent-rewrite canary.** When the input has no terse directives it is a pure
//!    *identity* pass-through: the canonical output equals the input byte-for-byte (see
//!    [`Expansion::is_identity`] and the canary tests). Any future rewrite that fires on
//!    an already-canonical query is a bug the canary catches.
//! 3. **No lenient parsing.** Per the design's §3.2 verdict, this transpiler never
//!    rewrites a *malformed* query into a guessed valid one: a query that is neither a
//!    recognised terse directive nor canonical SPARQL fails *loudly* (the `spargebra`
//!    error), preserving the agent's recoverable feedback loop.
//!
//! ## What expands
//!
//! The only terse directive in this skeleton is the **`V("phrase")` concept-resolution
//! call site**, which the [`crate::resolve`] layer (behind the `link` feature) binds to a
//! concrete IRI under its own confidence-gate. Phase 1 *locates* the call sites (outside
//! string literals and comments) and exposes them; resolution itself is Phase 2.

use std::fmt;

/// The reserved terse function name for concept resolution: `V("phrase")`.
pub const V_FUNC: &str = "V";

/// A located `V("phrase")` call site in the terse source, found by the pre-parse scanner.
/// Byte offsets are into the *original* source string and the half-open range
/// `[start, end)` covers the whole `V(...)` call including the closing paren.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCall {
    /// The phrase argument with the surrounding quotes and any escapes resolved.
    pub phrase: String,
    /// Byte offset of the `V` in the original source.
    pub start: usize,
    /// Byte offset one past the closing `)` in the original source.
    pub end: usize,
}

/// What went wrong turning terse source into canonical SPARQL. Every variant is a *loud*
/// failure — the transpiler never silently "fixes" malformed input (the §6.6 rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerseError {
    /// A `V(` call site was opened but never closed, or its argument was not a single
    /// quoted string literal. Carries the byte offset of the offending `V`.
    MalformedVCall { at: usize, reason: String },
    /// The expanded output failed to parse as canonical SPARQL under `spargebra`. This is
    /// the loud-fail path for a malformed terse query: the message is the parser's own.
    NotCanonical(String),
    /// A `V("phrase")` was present but resolution is unavailable in this build (the `link`
    /// feature is off) or the caller passed no resolution context. Loud, never a guess.
    ResolutionUnavailable { phrase: String },
    /// A `V("phrase")` could not be resolved under the confidence envelope: either nothing
    /// matched, or the match was below the confidence floor / ambiguous against its
    /// runner-up. The candidate list is surfaced so the agent can disambiguate — never a
    /// silent low-confidence bind (the §6.3 rule).
    Unresolved { phrase: String, detail: String },
}

impl fmt::Display for TerseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerseError::MalformedVCall { at, reason } => {
                write!(f, "malformed V(...) call at byte {}: {}", at, reason)
            }
            TerseError::NotCanonical(msg) => {
                write!(f, "expansion did not parse as canonical SPARQL: {}", msg)
            }
            TerseError::ResolutionUnavailable { phrase } => write!(
                f,
                "V({:?}) needs concept resolution, which is unavailable in this build \
                 (enable the `link` feature and pass a resolution context)",
                phrase
            ),
            TerseError::Unresolved { phrase, detail } => {
                write!(f, "V({:?}) could not be resolved: {}", phrase, detail)
            }
        }
    }
}

impl std::error::Error for TerseError {}

/// One resolved `V("phrase")` — the §6.2 echo: the agent can verify the bind, and a close
/// runner-up is a visible ambiguity flag.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    /// The phrase the agent wrote inside `V("…")`.
    pub phrase: String,
    /// The IRI the phrase resolved to (echoed into the canonical SPARQL as a full IRI).
    pub iri: String,
    /// The match score in `[0, 1]` (higher is better) for the chosen IRI.
    pub score: f64,
    /// The next-best candidate IRI, if any — the visible ambiguity signal.
    pub runner_up: Option<String>,
    /// The runner-up's score, if there was a runner-up.
    pub runner_up_score: Option<f64>,
    /// A confidence in `[0, 1]` combining the score with the margin over the runner-up.
    pub confidence: f64,
    /// Which path resolved it — the deterministic lexical match, or the vector fallback.
    pub method: ResolutionMethod,
}

/// Which resolution path bound a `V("phrase")` (the §6.4 `method` marker).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionMethod {
    /// The deterministic, no-model lexical label match (the default, preferred path).
    Lexical,
    /// The vector-nearest fallback, used only when lexical returned nothing.
    Vector,
}

impl fmt::Display for ResolutionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionMethod::Lexical => f.write_str("lexical"),
            ResolutionMethod::Vector => f.write_str("vector"),
        }
    }
}

/// The transpiler's output — the verifiable expansion. The contract is that
/// `canonical_sparql` is standard SPARQL the agent can read and the engine can run, and
/// `resolutions` documents every concept bind so nothing is hidden.
#[derive(Debug, Clone, PartialEq)]
pub struct Expansion {
    /// Canonical, conformant SPARQL — re-parsed under `spargebra` before this is returned.
    pub canonical_sparql: String,
    /// Every `V("phrase")` resolution performed, in source order (the §6.2 echo).
    pub resolutions: Vec<Resolution>,
    /// Non-fatal warnings (e.g. a close runner-up that was still above the margin).
    pub warnings: Vec<String>,
}

impl Expansion {
    /// `true` iff the expansion was a pure identity pass-through — no directive fired and
    /// the canonical output equals the input. The mechanical heart of the silent-rewrite
    /// canary: an already-canonical query in must come back unchanged.
    pub fn is_identity(&self, original: &str) -> bool {
        self.resolutions.is_empty() && self.canonical_sparql == original
    }
}

/// Validates that `sparql` parses under the unmodified vendored `spargebra` — the proof
/// that the expansion is genuinely canonical. Returns the input on success so call sites
/// can chain it. Never mutates the string (this is validation, not rewriting).
pub fn validate_canonical(sparql: &str) -> Result<(), TerseError> {
    spargebra::SparqlParser::new()
        .parse_query(sparql)
        .map(|_| ())
        .map_err(|e| TerseError::NotCanonical(e.to_string()))
}

/// Scans `src` and returns every `V("phrase")` call site that is **not** inside a SPARQL
/// string literal or comment, in source order. This is the parser-free pre-parse step:
/// it understands just enough SPARQL lexical structure (string literals with `'`/`"` and
/// their triple-quoted forms, `#` line comments, and IRIs in `<…>`) to avoid mistaking a
/// `V(` inside a quoted value for a directive.
///
/// A bare identifier `V` immediately followed by `(` is a call site; the argument must be
/// a single quoted string literal (`"…"` or `'…'`). Anything else is a [`TerseError::
/// MalformedVCall`] — a loud failure, never a guess.
pub fn scan_v_calls(src: &str) -> Result<Vec<VCall>, TerseError> {
    let bytes = src.as_bytes();
    let mut calls = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'#' => {
                // Line comment: skip to end of line.
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'<' => {
                // An IRI ref `<…>` — skip to the matching `>` so a `V(` inside an IRI is
                // never seen. (spargebra forbids `>`/whitespace inside an IRI, so this is
                // safe and conservative.)
                i += 1;
                while i < bytes.len() && bytes[i] != b'>' && bytes[i] != b'\n' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1; // consume the '>'
                }
            }
            b'"' | b'\'' => {
                i = skip_string_literal(bytes, i);
            }
            b'V' => {
                // A `V` is a call site only if it is a standalone token (the preceding
                // byte is not an identifier char) immediately followed by `(`.
                let prev_is_ident = i > 0 && is_sparql_ident_byte(bytes[i - 1]);
                let mut j = i + 1;
                // Allow no whitespace between V and ( — keep the directive tight and
                // unambiguous (a deliberate non-lenient choice).
                if !prev_is_ident && j < bytes.len() && bytes[j] == b'(' {
                    j += 1;
                    let call = parse_v_argument(bytes, i, j)?;
                    i = call.end;
                    calls.push(call);
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    Ok(calls)
}

/// Returns true if `b` can appear inside a SPARQL identifier/local-name run (so we can
/// tell a standalone `V` token from the `V` inside e.g. `myVar` or `ex:V`).
fn is_sparql_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b':' || b == b'?' || b == b'$'
}

/// Skips a SPARQL string literal starting at `start` (a `"` or `'`), handling the
/// triple-quoted long forms and backslash escapes. Returns the index one past the close.
fn skip_string_literal(bytes: &[u8], start: usize) -> usize {
    let q = bytes[start];
    let triple = start + 2 < bytes.len() && bytes[start + 1] == q && bytes[start + 2] == q;
    if triple {
        let mut i = start + 3;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if i + 2 < bytes.len() && bytes[i] == q && bytes[i + 1] == q && bytes[i + 2] == q {
                return i + 3;
            }
            i += 1;
        }
        bytes.len()
    } else {
        let mut i = start + 1;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 2,
                c if c == q => return i + 1,
                _ => i += 1,
            }
        }
        bytes.len()
    }
}

/// Parses the argument of a `V(` call whose `(` is at `open` (one past it). The argument
/// must be a single quoted string literal optionally surrounded by whitespace, then `)`.
fn parse_v_argument(bytes: &[u8], v_at: usize, open: usize) -> Result<VCall, TerseError> {
    let mut i = open;
    // Leading whitespace.
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || (bytes[i] != b'"' && bytes[i] != b'\'') {
        return Err(TerseError::MalformedVCall {
            at: v_at,
            reason: "expected a quoted string argument, e.g. V(\"a phrase\")".to_string(),
        });
    }
    let q = bytes[i];
    let str_start = i;
    let str_end = skip_string_literal(bytes, str_start);
    if str_end > bytes.len() || (str_end == bytes.len() && bytes[bytes.len() - 1] != q) {
        return Err(TerseError::MalformedVCall {
            at: v_at,
            reason: "unterminated string argument".to_string(),
        });
    }
    let phrase = decode_string_literal(&bytes[str_start..str_end]);
    i = str_end;
    // Trailing whitespace then the closing paren.
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b')' {
        return Err(TerseError::MalformedVCall {
            at: v_at,
            reason: "expected ')' after the V(\"…\") argument".to_string(),
        });
    }
    Ok(VCall {
        phrase,
        start: v_at,
        end: i + 1,
    })
}

/// Decodes the inner text of a SPARQL string literal token (with its quotes), resolving
/// the common backslash escapes (`\\`, `\"`, `\'`, `\n`, `\t`, `\r`). Conservative: an
/// unknown escape keeps the following char verbatim.
fn decode_string_literal(token: &[u8]) -> String {
    let s = std::str::from_utf8(token).unwrap_or("");
    let q = s.as_bytes()[0] as char;
    // Strip the surrounding quotes (triple or single).
    let inner = if s.len() >= 6 && s.starts_with(&q.to_string().repeat(3)) {
        &s[3..s.len() - 3]
    } else {
        &s[1..s.len().saturating_sub(1)]
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Rebuilds `src` with each `V(...)` call site replaced by its `replacement` IRI text,
/// preserving everything else verbatim. `replacements` must be in source order and
/// correspond 1:1 with [`scan_v_calls`]'s output. Used by the resolver to splice in the
/// resolved IRIs. With no calls this returns `src` unchanged (the identity path).
pub fn splice_v_calls(src: &str, calls: &[VCall], replacements: &[String]) -> String {
    debug_assert_eq!(calls.len(), replacements.len());
    if calls.is_empty() {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len());
    let mut cursor = 0usize;
    for (call, repl) in calls.iter().zip(replacements) {
        out.push_str(&src[cursor..call.start]);
        out.push_str(repl);
        cursor = call.end;
    }
    out.push_str(&src[cursor..]);
    out
}

/// Phase-1 entry point: turn terse `src` into a verifiable [`Expansion`] **without** any
/// concept resolution. If `src` contains no `V("…")` directives it is validated as
/// canonical SPARQL and returned as a pure identity pass-through (the canary). If it
/// contains `V("…")` this errors with [`TerseError::ResolutionUnavailable`] — resolution
/// is the Phase-2 [`crate::resolve`] surface, which needs a graph + the `link` feature.
///
/// This is the dependency-light skeleton: it pulls only `spargebra`, proving the
/// transpiler contract (always-canonical output + the canary) in the default build.
pub fn terse_to_sparql(src: &str) -> Result<Expansion, TerseError> {
    let calls = scan_v_calls(src)?;
    if let Some(first) = calls.first() {
        return Err(TerseError::ResolutionUnavailable {
            phrase: first.phrase.clone(),
        });
    }
    // No directives: the canonical SPARQL is the input itself — validate and echo it.
    validate_canonical(src)?;
    Ok(Expansion {
        canonical_sparql: src.to_string(),
        resolutions: Vec::new(),
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- The silent-rewrite canary: an already-canonical query is a no-op. ----

    #[test]
    fn canary_identity_passthrough_simple() {
        let q = "SELECT * WHERE { ?s ?p ?o }";
        let exp = terse_to_sparql(q).expect("canonical SPARQL passes through");
        assert!(
            exp.is_identity(q),
            "expected byte-identical identity expansion"
        );
        assert_eq!(exp.canonical_sparql, q);
        assert!(exp.resolutions.is_empty());
    }

    #[test]
    fn canary_identity_passthrough_with_prefixes_and_filter() {
        let q = "PREFIX ex: <http://example.org/>\n\
                 SELECT ?x WHERE { ?x ex:p ?y . FILTER(?y > 3) }";
        let exp = terse_to_sparql(q).expect("canonical");
        assert!(exp.is_identity(q));
    }

    #[test]
    fn canary_does_not_touch_v_inside_a_string_literal() {
        // "V(...)" appears INSIDE a string literal — it must NOT be seen as a directive,
        // and the whole query is a plain identity pass-through.
        let q = r#"SELECT ?s WHERE { ?s ?p "V(\"not a call\")" }"#;
        let calls = scan_v_calls(q).expect("scan ok");
        assert!(
            calls.is_empty(),
            "V( inside a string literal is not a call: {calls:?}"
        );
        let exp = terse_to_sparql(q).expect("canonical");
        assert!(exp.is_identity(q));
    }

    #[test]
    fn canary_does_not_touch_v_inside_a_comment() {
        let q = "SELECT * WHERE { ?s ?p ?o } # try V(\"later\")";
        let calls = scan_v_calls(q).expect("scan ok");
        assert!(calls.is_empty());
        assert!(terse_to_sparql(q).expect("canonical").is_identity(q));
    }

    #[test]
    fn canary_does_not_touch_v_as_part_of_an_identifier() {
        // `?V` is a variable, `ex:V` a prefixed name, `myV` a local name — none is a call.
        let q = "SELECT ?V WHERE { ?V ex:V ?myV }";
        let calls = scan_v_calls(q).expect("scan ok");
        assert!(calls.is_empty(), "identifier V is not a call: {calls:?}");
    }

    // ---- Scanning V("phrase") call sites. ----

    #[test]
    fn scans_a_single_v_call() {
        let q = r#"SELECT ?f WHERE { ?f pkg:about V("cardinality estimation") }"#;
        let calls = scan_v_calls(q).expect("scan ok");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].phrase, "cardinality estimation");
        // The range covers the whole V(...) call.
        assert_eq!(
            &q[calls[0].start..calls[0].end],
            r#"V("cardinality estimation")"#
        );
    }

    #[test]
    fn scans_multiple_v_calls_in_order() {
        let q = r#"SELECT * WHERE { V("a") pkg:rel V("b") }"#;
        let calls = scan_v_calls(q).expect("scan ok");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].phrase, "a");
        assert_eq!(calls[1].phrase, "b");
        assert!(calls[0].start < calls[1].start);
    }

    #[test]
    fn scans_single_quoted_and_escaped_argument() {
        let q = r#"SELECT * WHERE { ?s ?p V('a \'quoted\' phrase') }"#;
        let calls = scan_v_calls(q).expect("scan ok");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].phrase, "a 'quoted' phrase");
    }

    #[test]
    fn malformed_v_call_without_string_arg_is_loud() {
        let q = "SELECT * WHERE { ?s ?p V(?x) }";
        let err = scan_v_calls(q).unwrap_err();
        assert!(
            matches!(err, TerseError::MalformedVCall { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn malformed_v_call_unterminated_is_loud() {
        let q = r#"SELECT * WHERE { ?s ?p V("oops }"#;
        let err = scan_v_calls(q).unwrap_err();
        assert!(
            matches!(err, TerseError::MalformedVCall { .. }),
            "got {err:?}"
        );
    }

    // ---- terse_to_sparql in the skeleton (no resolution). ----

    #[test]
    fn v_call_without_resolution_is_loud_not_silent() {
        let q = r#"SELECT ?f WHERE { ?f pkg:about V("a topic") }"#;
        let err = terse_to_sparql(q).unwrap_err();
        match err {
            TerseError::ResolutionUnavailable { phrase } => assert_eq!(phrase, "a topic"),
            other => panic!("expected ResolutionUnavailable, got {other:?}"),
        }
    }

    #[test]
    fn malformed_canonical_sparql_fails_loudly() {
        // No terse directive, but not valid SPARQL either — must fail LOUD (no lenient fix).
        let q = "SELECT * WHERE { this is not a graph pattern }";
        let err = terse_to_sparql(q).unwrap_err();
        assert!(matches!(err, TerseError::NotCanonical(_)), "got {err:?}");
    }

    // ---- Splicing. ----

    #[test]
    fn splice_replaces_each_call_in_place() {
        let q = r#"SELECT * WHERE { V("a") p V("b") }"#;
        let calls = scan_v_calls(q).expect("scan ok");
        let repls = vec!["<http://x/a>".to_string(), "<http://x/b>".to_string()];
        let out = splice_v_calls(q, &calls, &repls);
        assert_eq!(out, "SELECT * WHERE { <http://x/a> p <http://x/b> }");
    }

    #[test]
    fn splice_with_no_calls_is_identity() {
        let q = "SELECT * WHERE { ?s ?p ?o }";
        assert_eq!(splice_v_calls(q, &[], &[]), q);
    }

    #[test]
    fn validate_canonical_accepts_real_sparql_rejects_garbage() {
        assert!(validate_canonical("ASK { ?s ?p ?o }").is_ok());
        assert!(validate_canonical("definitely not sparql").is_err());
    }
}
