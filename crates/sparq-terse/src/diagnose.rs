//! The non-rewriting **did-you-mean diagnostic** (design §3.2 "the one safe sliver",
//! §8.5 Phase 4, sq-h7zlx).
//!
//! Lever 2 (lenient parsing — accept `FLTR` and quietly run `FILTER`) is *recommended
//! against* by the design record: it silently rewrites the agent's intent and can yield a
//! different, valid, **wrong** query. The only sliver worth shipping is the inverse — on a
//! **parse failure only**, hand back a *suggestion* and never apply it:
//!
//! ```text
//! unknown token `FLTR` — did you mean FILTER? (not applied)
//! ```
//!
//! That is the `crates/sparq-nlq/src/constrain.rs` loud-fail-with-a-hint pattern lifted from
//! the *vocabulary* level (unknown IRI → nearest known IRI) to the **keyword** level, and it
//! preserves exactly what makes a typo cheap for an agent: the parse still fails, loudly, and
//! the agent fixes it on the next turn from a hint rather than from a bare syntax error.
//!
//! Two invariants make this diagnostics rather than lenient parsing:
//!
//! 1. **Nothing is ever rewritten.** [`keyword_suggestions`] is a read-only scan; the
//!    transpiler's emission is unchanged and the error is still returned.
//! 2. **It only runs on failure.** The scan is invoked from the silent-rewrite canary's
//!    error path ([`crate::TerseError::CanaryFailed`]), never on a query that parses — so a
//!    successful transpile can never be influenced by it.
//!
//! Like the rest of the keyword layer this lives in the **lean default build**: a frozen
//! static keyword table plus a local edit distance, with no `sparq-core`, no model and no
//! network.

/// The closest SPARQL keywords to one unrecognised token in a query that failed to parse —
/// a **diagnostic only**, carried by [`crate::TerseError::CanaryFailed`] and *never* applied
/// (design §3.2: a suggestion preserves the agent's loud, recoverable feedback loop; a
/// silent rewrite destroys it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordSuggestion {
    /// The unrecognised token exactly as written in the query (e.g. `FLTR`).
    pub token: String,
    /// The closest SPARQL keywords, nearest first (never empty — a token with no near
    /// keyword is simply not reported).
    pub suggestions: Vec<String>,
}

/// How many keywords are suggested per unrecognised token.
const MAX_SUGGESTIONS: usize = 3;

/// How many unrecognised tokens are reported per failing query. A parse failure usually has
/// one typo; capping keeps the error message bounded and the hint pointed at the first thing
/// to fix (the compiler-literature "scroll back to the FIRST error" discipline, design §3.2).
const MAX_TOKENS: usize = 3;

/// The shortest token worth a suggestion. Below this every short keyword is within the edit
/// budget of every other, so a "hint" would be noise rather than a signal.
const MIN_TOKEN_LEN: usize = 3;

/// The frozen SPARQL 1.1/1.2 keyword + built-in-function table the diagnostic matches
/// against. Matching is case-insensitive (SPARQL keywords are), and this table is used in
/// exactly two ways: to decide a bare word is *already* a keyword (so it is not reported),
/// and to rank the nearest keywords to one that is not. It is deliberately a plain static
/// list — no alias dictionary, no per-user entries (design §3.2: aliases are the anti-pattern
/// this diagnostic exists to avoid).
const KEYWORDS: &[&str] = &[
    // --- query forms + solution modifiers ---
    "BASE", "PREFIX", "SELECT", "CONSTRUCT", "DESCRIBE", "ASK", "DISTINCT", "REDUCED", "AS",
    "FROM", "NAMED", "WHERE", "GROUP", "BY", "HAVING", "ORDER", "ASC", "DESC", "LIMIT",
    "OFFSET", "VALUES", "UNDEF", "OPTIONAL", "GRAPH", "UNION", "MINUS", "FILTER", "BIND",
    "SERVICE", "SILENT", "EXISTS", "NOT", "IN", "A", "TRUE", "FALSE",
    // --- update ---
    "LOAD", "INTO", "CLEAR", "DROP", "CREATE", "ADD", "MOVE", "COPY", "INSERT", "DELETE",
    "DATA", "WITH", "USING", "DEFAULT", "ALL", "TO",
    // --- aggregates ---
    "COUNT", "SUM", "MIN", "MAX", "AVG", "SAMPLE", "GROUP_CONCAT", "SEPARATOR",
    // --- built-in functions ---
    "STR", "LANG", "LANGMATCHES", "DATATYPE", "BOUND", "IRI", "URI", "BNODE", "RAND", "ABS",
    "CEIL", "FLOOR", "ROUND", "CONCAT", "STRLEN", "UCASE", "LCASE", "ENCODE_FOR_URI",
    "CONTAINS", "STRSTARTS", "STRENDS", "STRBEFORE", "STRAFTER", "YEAR", "MONTH", "DAY",
    "HOURS", "MINUTES", "SECONDS", "TIMEZONE", "TZ", "NOW", "UUID", "STRUUID", "MD5", "SHA1",
    "SHA256", "SHA384", "SHA512", "COALESCE", "IF", "STRLANG", "STRDT", "SAMETERM", "ISIRI",
    "ISURI", "ISBLANK", "ISLITERAL", "ISNUMERIC", "REGEX", "SUBSTR", "REPLACE",
    // --- SPARQL 1.2 term constructors/accessors ---
    "TRIPLE", "ISTRIPLE", "SUBJECT", "PREDICATE", "OBJECT",
];

/// Scans `sparql` — a query that has already **failed to parse** — for bare word tokens that
/// are not SPARQL keywords but are one or two edits away from one, returning the
/// did-you-mean hints (design §3.2, Phase 4). At most three tokens are reported, each with
/// at most three candidates, nearest first.
///
/// This **never** rewrites anything: it is a read-only scan whose result is attached to
/// [`crate::TerseError::CanaryFailed`] so the agent gets a hint alongside the (still fatal)
/// parse error. Call it only on a parse failure — on a query that parses, every bare word is
/// a keyword by construction and the scan has nothing useful to say.
///
/// Tokens inside string literals, IRIs and comments are ignored, as are variables (`?x`,
/// `$x`), prefixed names (`ex:thing`), blank-node labels (`_:b`), language tags (`@en`) and
/// numbers — the same lexical-context discipline the `K:` and `V()` scanners use. A word with
/// no near keyword is not reported at all (an unknown *term* is not a keyword typo).
pub fn keyword_suggestions(sparql: &str) -> Vec<KeywordSuggestion> {
    let mut out: Vec<KeywordSuggestion> = Vec::new();
    scan_bare_words(sparql, &mut |word: &str| {
        if out.len() >= MAX_TOKENS || word.chars().count() < MIN_TOKEN_LEN || is_keyword(word) {
            return;
        }
        // Report a given misspelling once, however many times it appears.
        if out.iter().any(|s| s.token == word) {
            return;
        }
        let suggestions = nearest_keywords(word);
        if !suggestions.is_empty() {
            out.push(KeywordSuggestion {
                token: word.to_string(),
                suggestions,
            });
        }
    });
    out
}

/// `true` if `word` is a SPARQL keyword (case-insensitive — SPARQL keywords are).
fn is_keyword(word: &str) -> bool {
    KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(word))
}

/// The up-to-[`MAX_SUGGESTIONS`] keywords closest to `word` by edit distance, nearest first.
/// Equal distances are broken first by a shared initial letter (a typo rarely changes the
/// first character, so `FLTR` should offer `FILTER` before the equally-distant `STR`) and
/// then alphabetically, for a deterministic message. Only candidates within the edit budget
/// for the token's length are returned — a larger distance shares almost nothing with the
/// token and would be noise, so such a token yields no hint at all.
fn nearest_keywords(word: &str) -> Vec<String> {
    let budget = edit_budget(word.chars().count());
    let needle = word.to_ascii_uppercase();
    let initial = needle.chars().next();
    // `false` sorts before `true`, so the flag is "the initial DIFFERS" (i.e. rank last).
    let mut scored: Vec<(usize, bool, &'static str)> = KEYWORDS
        .iter()
        .filter_map(|kw| {
            let d = edit_distance(&needle, kw);
            (d <= budget).then_some((d, kw.chars().next() != initial, *kw))
        })
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(b.2)));
    scored.truncate(MAX_SUGGESTIONS);
    scored.into_iter().map(|(_, _, kw)| kw.to_string()).collect()
}

/// How many edits a candidate keyword may be from a token of `len` characters. Two edits
/// covers the real typo shapes (a dropped vowel pair, as in `FLTR` for `FILTER`; a pair of
/// swapped letters), but only once the token is long enough that two edits still leave most
/// of it intact.
fn edit_budget(len: usize) -> usize {
    if len <= 3 {
        1
    } else {
        2
    }
}

/// Levenshtein edit distance between the characters of `a` and `b`.
///
/// Deliberately a small local copy of `sparq_core::strdist::edit_distance`: this diagnostic
/// ships in the crate's **lean default build**, whose whole point is that it depends on
/// `spargebra` alone — pulling `sparq-core` (a triplestore, with `rayon`/`memchr`) in
/// unconditionally to reach a 20-line helper would break that invariant for the sake of
/// sharing. Tokens here are short SPARQL keywords, so the quadratic cost is trivial.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Walks `src` and invokes `f` on every **bare word** token — an identifier-shaped run that
/// is not inside a string/IRI/comment and is not a variable, prefixed name, blank-node label,
/// language tag or number. In valid SPARQL every such token is a keyword, which is what makes
/// an unrecognised one a credible keyword typo.
fn scan_bare_words(src: &str, f: &mut dyn FnMut(&str)) {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    // The enclosing brackets, innermost last — the lexical context that decides what a `<`
    // means (see [`opens_iriref`]). Only `(`/`{` are tracked; brackets inside a string, IRI
    // or comment are never seen because those spans are skipped whole.
    let mut brackets: Vec<u8> = Vec::new();
    // The last byte of the preceding token. Whitespace and comments separate tokens without
    // being one, so they leave it alone.
    let mut prev = 0u8;
    while i < n {
        let c = bytes[i];
        let mut token = true;
        let next = match c {
            // `<` opens an IRI ref only in term position with a well-formed IRIREF body;
            // in expression position after an operand it is the less-than (`<`, `<=`)
            // comparison operator, and what follows is ordinary keyword position.
            b'<' if opens_iriref(&brackets, prev) => iriref_end(bytes, i).unwrap_or(i + 1),
            b'#' => {
                token = false;
                let mut j = i + 1;
                while j < n && bytes[j] != b'\n' {
                    j += 1;
                }
                j
            }
            b'"' | b'\'' => crate::transpile::skip_string(bytes, i),
            // A variable (?x / $x) or a language tag (@en, @prefix): the name after the
            // sigil is not a keyword position.
            b'?' | b'$' | b'@' => {
                let mut j = i + 1;
                while j < n && (is_word_byte(bytes[j]) || bytes[j] == b'-') {
                    j += 1;
                }
                j
            }
            b'(' | b'{' => {
                brackets.push(c);
                i + 1
            }
            b')' | b'}' => {
                brackets.pop();
                i + 1
            }
            _ if c.is_ascii_digit() => {
                let mut j = i;
                while j < n && (is_word_byte(bytes[j]) || bytes[j] == b'.') {
                    j += 1;
                }
                j
            }
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                let mut j = i;
                while j < n && is_word_byte(bytes[j]) {
                    j += 1;
                }
                if j < n && bytes[j] == b':' {
                    // A prefixed name (`ex:thing`) or blank-node label (`_:b`): skip the
                    // colon and the local part; neither half is a keyword position.
                    j += 1;
                    while j < n && (is_word_byte(bytes[j]) || bytes[j] == b'-') {
                        j += 1;
                    }
                } else if let Ok(word) = std::str::from_utf8(&bytes[start..j]) {
                    f(word);
                }
                j
            }
            _ => i + 1,
        };
        if token && !c.is_ascii_whitespace() {
            prev = bytes[next - 1];
        }
        i = next;
    }
}

/// Whether the `<` that follows the token ending in `prev`, inside `brackets`, opens an
/// IRIREF rather than being the less-than operator.
///
/// The body test in [`iriref_end`] alone cannot decide this: SPARQL punctuation delimits
/// tokens without whitespace, so a compact expression like `?a<1&&FLTR(?o)&&?b>2` contains
/// no character the IRIREF production excludes — treating it as an IRI would swallow the
/// very bare words the scan exists to report. The *position* does decide it, from two
/// grammar facts:
///
/// 1. Less-than is only ever reachable through an `Expression`, and every production that
///    admits one (`Constraint`, `BrackettedExpression`, an argument list, `BIND`, `HAVING`)
///    is parenthesised — so a `<` whose innermost enclosing bracket is `{` (a graph pattern,
///    e.g. `FILTER(EXISTS { ?s <http://ex/p> ?o })`) or nothing at all (`BASE`/`FROM`) is a
///    term, never an operator.
/// 2. Inside those parentheses a binary operator must follow an operand, whereas an IRIREF
///    argument follows an opener or a separator (`(`, `,`, `=`, `^^`, …). So the operator
///    reading needs *both* an expression context and a preceding operand.
///
/// The two tests are conjunctive — a `<` opens an IRI ref only if the position allows it
/// *and* the body is well formed — so failing either reads the `<` as an operator. That is
/// the conservative direction for a diagnostic: at worst the scan walks into an IRI body and
/// offers a spurious hint, rather than silently hiding a real typo.
fn opens_iriref(brackets: &[u8], prev: u8) -> bool {
    !(brackets.last() == Some(&b'(') && ends_operand(prev))
}

/// `true` if a token ending in byte `b` can end an expression operand — a variable or
/// number (alphanumeric or `_`), a bracketed sub-expression or call (`)`), or a string
/// literal (`"`, `'`). Notably `>` is excluded: in `FILTER(?a > <http://ex/p>)` the `>` is
/// itself an operator, so the IRI after it is still term position.
fn ends_operand(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b')' || b == b'"' || b == b'\''
}

/// `true` if `b` can appear inside an identifier-shaped SPARQL token.
fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The *body* half of the IRIREF test (the positional half is [`opens_iriref`], which gates
/// this call): given a `<` at `start` already in term position, the index just past its
/// closing `>`, or `None` if the span between them is not a well-formed IRIREF body — in
/// which case the `<` is the less-than comparison operator (`?o < 1`, `?o <= 1`).
///
/// Distinguishing the two matters because this scan runs on queries that have *already failed
/// to parse*: treating every `<` as an IRI would skip from a comparison operator to the next
/// `>` — or, with none, to end of input — silently swallowing the very bare words the scan
/// exists to report. The test is the SPARQL grammar's own IRIREF production,
/// `'<' ([^<>"{}|^\`\\] - [#x00-#x20])* '>'`: a body character that the production excludes
/// (notably the space in `?o < 1`) means this was never an IRI ref.
fn iriref_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut j = start + 1;
    while j < bytes.len() {
        match bytes[j] {
            b'>' => return Some(j + 1),
            b'<' | b'"' | b'{' | b'}' | b'|' | b'^' | b'`' | b'\\' => return None,
            b if b <= 0x20 => return None,
            _ => j += 1,
        }
    }
    // Unterminated: no closing `>` before end of input, so not an IRI ref.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn misspelled_keyword_is_suggested_but_never_applied() {
        // The design's own example: FLTR must surface FILTER as a hint.
        let hints = keyword_suggestions("SELECT ?s WHERE { ?s ?p ?o FLTR(?o > 1) }");
        assert_eq!(hints.len(), 1, "got {hints:?}");
        assert_eq!(hints[0].token, "FLTR");
        assert_eq!(
            hints[0].suggestions.first().map(String::as_str),
            Some("FILTER"),
            "got {hints:?}"
        );
        // Equally-distant candidates that do not share the token's initial rank last.
        assert!(
            !hints[0].suggestions.iter().take(2).any(|s| s == "STR"),
            "a differing initial must not out-rank a shared one, got {hints:?}"
        );
    }

    /// `word` with the character at `idx` deleted — the "dropped character" typo shape.
    /// Typos are *derived* rather than written out so no misspelled literal sits in the
    /// source for the repository's spell-check gate to flag.
    fn dropped(word: &str, idx: usize) -> String {
        word.chars().enumerate().filter(|(i, _)| *i != idx).map(|(_, c)| c).collect()
    }

    /// `word` with the characters at `idx` and `idx + 1` swapped — the transposition shape.
    fn transposed(word: &str, idx: usize) -> String {
        let mut chars: Vec<char> = word.chars().collect();
        chars.swap(idx, idx + 1);
        chars.into_iter().collect()
    }

    #[test]
    fn common_typo_shapes_are_covered() {
        // A dropped character and a transposition are the two shapes worth catching.
        let cases = [
            (dropped("SELECT", 3), "SELECT"),
            (dropped("OPTIONAL", 4), "OPTIONAL"),
            (transposed("WHERE", 1), "WHERE"),
        ];
        for (typo, want) in cases {
            let hints = keyword_suggestions(&format!("{} {{ }}", typo));
            assert!(
                hints.iter().any(|h| h.suggestions.iter().any(|s| s == want)),
                "{} should suggest {}, got {:?}",
                typo,
                want,
                hints
            );
        }
    }

    #[test]
    fn keywords_and_non_keyword_positions_are_never_reported() {
        // Real keywords (in any case), variables, prefixed names, blank nodes, IRIs,
        // literals, comments, language tags and numbers are all silent — even when the
        // non-code text contains a token that would otherwise earn a hint.
        let typo = dropped("SELECT", 3);
        let q = format!(
            "prefix ex: <http://ex/> select ?s where {{ _:b ex:p \"{}\"@en . \
             ?s <http://ex/FLTR> 42 }} # {}",
            typo, typo
        );
        assert!(keyword_suggestions(&q).is_empty(), "got {:?}", keyword_suggestions(&q));
    }

    #[test]
    fn comparison_operators_do_not_swallow_the_rest_of_the_query() {
        // `<` is less-than as well as an IRI opener. Skipping to the next `>` — or, with
        // none, to end of input — would hide every bare word after the operator.
        for q in [
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?o < 1) FLTR(?o) }",
            // Both operators present: the span between them must stay keyword position.
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?a < 1 && ?b > 2) FLTR(?o) }",
            // The typo sits *between* the two operators, where an IRI skip would eat it.
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?a < 1) FLTR(?o) FILTER(?b > 2) }",
            // `<=` likewise, and a newline-free single line so no other rule rescues it.
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?o <= 1) FLTR(?o) }",
        ] {
            let hints = keyword_suggestions(q);
            let hint = hints
                .iter()
                .find(|h| h.token == "FLTR")
                .unwrap_or_else(|| panic!("FLTR must survive a comparison in {q:?}, got {hints:?}"));
            assert!(hint.suggestions.contains(&"FILTER".to_string()), "got {hint:?}");
        }
    }

    #[test]
    fn compact_comparisons_do_not_swallow_the_rest_of_the_query() {
        // SPARQL punctuation delimits tokens without whitespace, so a comparison can be
        // written with no space at all — and then every byte between `<` and the later `>`
        // is IRIREF-legal. Only the *position* of the `<` tells the two apart.
        for q in [
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?a<1&&FLTR(?o)&&?b>2) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?a<1) FLTR(?o) FILTER(?b>2) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?a<=1&&FLTR(?o)) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(COUNT(?a)<1&&FLTR(?o)&&?b>2) }",
        ] {
            let hints = keyword_suggestions(q);
            let hint = hints
                .iter()
                .find(|h| h.token == "FLTR")
                .unwrap_or_else(|| panic!("FLTR must survive a comparison in {q:?}, got {hints:?}"));
            assert!(hint.suggestions.contains(&"FILTER".to_string()), "got {hint:?}");
        }
    }

    #[test]
    fn a_real_iri_is_still_skipped_next_to_a_comparison() {
        // The other half of the contract: distinguishing the operator must not stop genuine
        // IRIs — whose body the SPARQL grammar forbids spaces in — from being ignored.
        for q in [
            "SELECT ?s WHERE { ?s <http://ex/FLTR> ?o FILTER(?o < 1) }",
            // Inside a FILTER's parentheses, where the positional test is at its riskiest:
            // after an opener, after a separator, after an operator, and as a datatype.
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?o = <http://ex/FLTR>) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?o IN (<http://ex/FLTR>, <http://ex/SELCT>)) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(<http://ex/FLTR>(?o)) }",
            "SELECT ?s WHERE { ?s ?p ?o FILTER(?o > \"1\"^^<http://ex/FLTR>) }",
            // A graph pattern nested inside those parentheses is term position again.
            "SELECT ?s WHERE { ?s ?p ?o FILTER(EXISTS { ?s <http://ex/FLTR> ?o }) }",
        ] {
            assert!(keyword_suggestions(q).is_empty(), "{q:?} got {:?}", keyword_suggestions(q));
        }
        // An unterminated `<` is not an IRI either, so the words after it stay visible.
        let hints = keyword_suggestions("SELECT ?s WHERE { ?s ?p ?o } FLTR <http://ex/unclosed");
        assert!(hints.iter().any(|h| h.token == "FLTR"), "got {hints:?}");
    }

    #[test]
    fn an_unrelated_word_yields_no_hint() {
        // An unknown *term*, not a keyword typo — outside the edit budget, so no noise.
        assert!(keyword_suggestions("zzzzzzzz { }").is_empty());
        // And a token too short to disambiguate is never reported.
        assert!(keyword_suggestions("xy { }").is_empty());
    }

    #[test]
    fn suggestions_are_bounded_and_deterministic() {
        // Four misspellings, so the per-query cap is exercised.
        let q = format!(
            "{} ?s {} {{ ?s ?p ?o FLTR(?o) }} {} {{ }}",
            dropped("SELECT", 3),
            transposed("WHERE", 1),
            dropped("OPTIONAL", 4)
        );
        let hints = keyword_suggestions(&q);
        assert!(hints.len() <= MAX_TOKENS, "capped at {}, got {:?}", MAX_TOKENS, hints);
        for h in &hints {
            assert!(!h.suggestions.is_empty(), "an empty hint must not be reported");
            assert!(h.suggestions.len() <= MAX_SUGGESTIONS, "got {:?}", h);
        }
        // Same input, same output — the message must be stable across runs.
        assert_eq!(hints, keyword_suggestions(&q));
    }

    #[test]
    fn a_repeated_typo_is_reported_once() {
        let hints = keyword_suggestions("SELCT ?s { ?s ?p ?o } SELCT");
        assert_eq!(hints.len(), 1, "got {hints:?}");
    }

    #[test]
    fn edit_distance_matches_the_shared_definition() {
        assert_eq!(edit_distance("FILTER", "FILTER"), 0);
        assert_eq!(edit_distance("FLTR", "FILTER"), 2);
        assert_eq!(edit_distance("", "ASK"), 3);
        assert_eq!(edit_distance("ASK", ""), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn edit_budget_scales_with_token_length() {
        assert_eq!(edit_budget(3), 1);
        assert_eq!(edit_budget(4), 2);
    }
}
