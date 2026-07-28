//! The non-rewriting **did-you-mean diagnostic** (Phase 4, sq-h7zlx — design §3.2/§8.5).
//!
//! Design §3.2 recommends *against* lever 2 (lenient parse / typo-alias mode): silently
//! turning `FLTR` into `FILTER` can yield a *different, valid, wrong* query, and it destroys
//! the agent's loud, recoverable feedback loop. The one safe sliver is this module: **on parse
//! failure only**, name the token that looks like a mistyped SPARQL keyword and offer the
//! nearest legend entries — *without applying them*.
//!
//! So the contract is deliberately narrow:
//!
//! - It runs **only** from the silent-rewrite canary's failure path
//!   ([`crate::TerseError::CanaryFailed`]). A query that parses is never inspected, so no
//!   valid query can be perturbed by it.
//! - It **never rewrites**. [`keyword_hints`] returns *strings to show a human/agent*; nothing
//!   in this crate consumes them to edit a query. The transpiler still fails loudly and the
//!   agent still fixes its own typo — the diagnostic just shortens the search.
//! - It is **conservative**. Only a bare word (outside strings, IRIs, comments, prefixed
//!   names, variables and language tags) that is *not* a known SPARQL keyword and is within a
//!   small edit distance of one produces a hint. An ordinary syntax error (an unbalanced
//!   brace, a missing dot) yields **no** hints rather than noise.
//!
//! The known-keyword set is derived from the terminals of the **vendored** `spargebra`
//! grammar, which is the parser whose rejection triggers the diagnostic — a unit test
//! re-derives it from `vendor/spargebra/src/parser.rs` so the two cannot drift.

/// One did-you-mean diagnostic: a token in the failing query that looks like a mistyped
/// SPARQL keyword, plus the closest real keywords.
///
/// This is **advisory output only** — no code path applies a suggestion (design §3.2: at most
/// a *diagnostic suggestion on parse failure*, never an auto-applied repair).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordHint {
    /// The unrecognised token exactly as it was written in the query.
    pub token: String,
    /// The closest SPARQL keywords in their canonical spelling, nearest first (never empty —
    /// a token with no close keyword produces no hint at all).
    pub suggestions: Vec<String>,
}

/// How many distinct tokens are reported per failing query. A typo'd query usually has one;
/// the cap keeps the error message readable rather than dumping every unknown word.
const MAX_HINTS: usize = 3;

/// How many candidate keywords are offered per token (matching `constrain.rs`'s
/// dictionary did-you-mean, which this lifts to the keyword level).
const MAX_SUGGESTIONS: usize = 3;

/// Returns the did-you-mean hints for `sparql`, in source order (at most three tokens, each
/// with at most three candidates).
///
/// Intended for a query that has **already failed to parse** — that is the only place this
/// crate calls it (see [`crate::TerseError::CanaryFailed`]). It is public so a caller holding
/// a parse error from anywhere (the engine, the server) can produce the same hint without
/// re-implementing it, but it is a *diagnostic*: it reports, it never repairs.
///
/// Returns an empty vector when nothing in the query resembles a mistyped keyword — including
/// for every *valid* query, so calling it on parsing input is pointless rather than harmful.
///
/// ```
/// let hints = sparq_terse::keyword_hints("SELECT ?s WHERE { ?s ?p ?o } FLTR(?s > 1)");
/// assert_eq!(hints[0].token, "FLTR");
/// assert_eq!(hints[0].suggestions[0], "FILTER");
/// // Nothing was rewritten — the caller decides what to do with the hint.
/// ```
pub fn keyword_hints(sparql: &str) -> Vec<KeywordHint> {
    let bytes = sparql.as_bytes();
    let n = bytes.len();
    let mut hints: Vec<KeywordHint> = Vec::new();
    let mut i = 0usize;
    while i < n && hints.len() < MAX_HINTS {
        let c = bytes[i];
        match c {
            // Skip the contexts where a word is data, not syntax: IRIs, comments, string
            // literals (the same lexical discipline as the K:/V() scanners).
            b'<' => {
                let mut j = i + 1;
                while j < n && bytes[j] != b'>' && bytes[j] != b'\n' {
                    j += 1;
                }
                i = if j < n { j + 1 } else { n };
            }
            b'#' => {
                let mut j = i + 1;
                while j < n && bytes[j] != b'\n' {
                    j += 1;
                }
                i = j;
            }
            b'"' | b'\'' => i = crate::transpile::skip_string(bytes, i),
            // A language tag (`"x"@en`) or a Turtle-style `@prefix` directive: the word
            // belongs to the sigil, not to the keyword vocabulary.
            b'@' => {
                let mut j = i + 1;
                while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                    j += 1;
                }
                i = j.max(i + 1);
            }
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let end = word_end(bytes, i);
                // A word glued to an identifier (`?fooBAR`, the local part of `ex:name`) is not
                // a standalone token, and a word followed by `:` is a prefix label or a `K:`
                // keyword — neither is a SPARQL keyword position. Both boundaries also have to
                // account for the name bytes this ASCII scanner cannot itself walk (non-ASCII
                // `PN_CHARS`, a `PN_LOCAL_ESC`), else a candidate starts or ends inside a valid
                // name. The two sides differ only in `?`/`$`, which can *start* a name but never
                // continue one — so a word merely ending at one (`FLTR?x`) is still a bare word.
                let standalone = !preceded_by_name(bytes, i);
                let glued_right = bytes
                    .get(end)
                    .is_some_and(|&b| b == b':' || !b.is_ascii() || b == b'\\');
                if standalone && !glued_right {
                    if let Some(hint) = hint_for(&sparql[i..end]) {
                        if !hints.iter().any(|h| h.token == hint.token) {
                            hints.push(hint);
                        }
                    }
                }
                i = end;
            }
            _ => i += 1,
        }
    }
    hints
}

/// `true` if `b` can be part of a SPARQL **name** — a variable, a prefixed name, a blank node
/// label — as opposed to a delimiter that a bare keyword may sit against.
///
/// This is deliberately wider than [`crate::transpile::is_ident_char`], which is ASCII-only.
/// Two classes of name byte fall outside that set, and both are *invisible* to this module's
/// ASCII word scanner, so without them a candidate word can begin (or end) in the middle of a
/// perfectly legal name:
///
/// - **any non-ASCII byte**, because `VARNAME`/`PN_CHARS_BASE` admit Unicode — in the valid
///   variable `?aéselct` the `é` would otherwise read as a delimiter and expose `selct`; and
/// - **`\`**, which opens a `PN_LOCAL_ESC` in the local part of a prefixed name — in the valid
///   `ex:a\~selct` the escape would otherwise expose `selct` the same way.
///
/// Both cases report a *typo* in syntax the vendored parser accepts, which contradicts this
/// module's contract that a valid query yields no hints — pinned against the real parser by
/// `unicode_names_and_pn_local_escapes_are_never_flagged` below.
fn is_name_byte(b: u8) -> bool {
    !b.is_ascii() || b == b'\\' || crate::transpile::is_ident_char(b)
}

/// `true` if the word starting at `i` is glued to the end of a name, so it is not a bare word.
///
/// One byte of lookback is not enough: a `PN_LOCAL_ESC` is *two* bytes (`\` plus one escaped
/// ASCII character such as `~`), and it is the escaped character — not the backslash — that
/// sits immediately before the word. In the valid `ex:a\~selct` a one-byte look-back sees `~`,
/// reads it as a delimiter, and reports `selct`. Outside strings, IRIs and comments (all
/// skipped before this point) a `\` in SPARQL *only* opens that escape, so looking back past
/// one byte for it cannot mask a real bare word.
fn preceded_by_name(bytes: &[u8], i: usize) -> bool {
    if i == 0 {
        return false;
    }
    is_name_byte(bytes[i - 1]) || (i >= 2 && bytes[i - 2] == b'\\')
}

/// The byte index just past the word starting at `start` (`[A-Za-z0-9_]+`).
fn word_end(bytes: &[u8], start: usize) -> usize {
    let mut j = start + 1;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    j
}

/// The hint for one bare word, or `None` when the word is a known keyword, is too short to
/// judge, or has no keyword within the distance budget.
///
/// The budget is deliberately tight — 1 edit for a 2–3 character word, 2 for anything longer
/// (so the design's own example, `FLTR` → `FILTER`, is reachable at 2). Beyond that the
/// "suggestion" would be a guess, which is exactly what §3.2 forbids.
fn hint_for(word: &str) -> Option<KeywordHint> {
    // A single letter is never judged: `a` is the rdf:type keyword, and one character is
    // within one edit of far too much to be a signal.
    if word.len() < 2 {
        return None;
    }
    let upper = word.to_ascii_uppercase();
    if is_keyword(&upper) {
        return None;
    }
    let budget = if word.len() <= 3 { 1 } else { 2 };
    let mut scored: Vec<(usize, &'static str)> = KEYWORDS
        .iter()
        .filter(|kw| kw.len() >= 2)
        .filter_map(|kw| {
            // Uppercase both sides: SPARQL keywords are case-insensitive, so `fltr` and
            // `FLTR` must produce the same hint. (Allocating per candidate is fine — this
            // only ever runs on an already-failed parse.)
            let d = edit_distance(&upper, &kw.to_ascii_uppercase());
            (d > 0 && d <= budget).then_some((d, *kw))
        })
        .collect();
    // Nearest first; ties broken by canonical spelling for a stable, deterministic order.
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    scored.truncate(MAX_SUGGESTIONS);
    if scored.is_empty() {
        return None;
    }
    Some(KeywordHint {
        token: word.to_string(),
        suggestions: scored.into_iter().map(|(_, kw)| kw.to_string()).collect(),
    })
}

/// `true` if `upper` (an ASCII-uppercased word) is a SPARQL keyword the vendored parser knows.
fn is_keyword(upper: &str) -> bool {
    // `a` — the Turtle/SPARQL shorthand for rdf:type — is a grammar literal rather than one of
    // the `i("…")` terminals below, so it is spelled out here.
    upper == "A" || KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(upper))
}

/// The keyword vocabulary the diagnostic measures against: every keyword terminal of the
/// **vendored** `spargebra` grammar (`vendor/spargebra/src/parser.rs`), in canonical spelling.
/// Kept in sync by `keyword_set_matches_the_vendored_grammar` below, so a grammar update
/// cannot silently leave the diagnostic suggesting a keyword the parser no longer accepts (or
/// flagging one it just gained).
const KEYWORDS: &[&str] = &[
    "ABS", "ADD", "ADJUST", "ALL", "AS", "ASC", "ASK", "AVG", "BASE", "BIND", "BNODE", "BOUND",
    "BY", "CEIL", "CLEAR", "COALESCE", "CONCAT", "CONSTRUCT", "CONTAINS", "COPY", "COUNT",
    "CREATE", "DATA", "DATATYPE", "DAY", "DEFAULT", "DELETE", "DESC", "DESCRIBE", "DISTINCT",
    "DROP", "ENCODE_FOR_URI", "EXISTS", "FILTER", "FLOOR", "FROM", "GRAPH", "GROUP",
    "GROUP_CONCAT", "HAVING", "HOURS", "IF", "IN", "INSERT", "INTO", "IRI", "LANG", "LANGDIR",
    "LANGMATCHES", "LATERAL", "LCASE", "LIMIT", "LOAD", "MAX", "MD5", "MIN", "MINUS",
    "MINUTES", "MONTH", "MOVE", "MULTIPLICITY", "NAMED", "NOT", "NOW", "OBJECT", "OFFSET",
    "OPTIONAL", "ORDER", "PREDICATE", "PREFIX", "RAND", "REDUCED", "REGEX", "REPLACE", "ROUND",
    "SAMPLE", "SECONDS", "SELECT", "SEPARATOR", "SERVICE", "SHA1", "SHA256", "SHA384",
    "SHA512", "SILENT", "STR", "STRAFTER", "STRBEFORE", "STRDT", "STRENDS", "STRLANG",
    "STRLANGDIR", "STRLEN", "STRSTARTS", "STRUUID", "SUBJECT", "SUBSTR", "SUM", "TIMEZONE",
    "TO", "TRIPLE", "TZ", "UCASE", "UNDEF", "UNION", "URI", "USING", "UUID", "VALUES",
    "VERSION", "WHERE", "WITH", "YEAR", "false", "hasLang", "hasLangDir", "isBLANK", "isIRI",
    "isLITERAL", "isNUMERIC", "isTriple", "isURI", "sameTerm", "true",
];

/// Levenshtein edit distance over the characters of `a` and `b`.
///
/// This is a deliberate crate-local copy of `sparq_core::strdist::edit_distance`: the whole
/// point of the lean default build is that it depends on nothing but `spargebra`, and the
/// diagnostic must work there. The `vectors`-feature test below asserts the two agree, so the
/// copy cannot drift from the shared helper (which is why that helper was promoted in the
/// first place — gh-3694).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_design_example_suggests_filter() {
        // design §3.2's own example: "unknown keyword FLTR — did you mean FILTER?".
        let hints = keyword_hints("SELECT ?s WHERE { ?s ?p ?o } FLTR(?s > 1)");
        assert_eq!(hints.len(), 1, "got {hints:?}");
        assert_eq!(hints[0].token, "FLTR");
        assert_eq!(hints[0].suggestions[0], "FILTER");
    }

    #[test]
    fn keyword_case_is_irrelevant() {
        // SPARQL keywords are case-insensitive, so a lowercase typo gets the same hint and a
        // lowercase *correct* keyword gets none.
        let hints = keyword_hints("select ?s where { ?s ?p ?o } fltr(?s > 1)");
        assert_eq!(hints.len(), 1, "got {hints:?}");
        assert_eq!(hints[0].token, "fltr", "the token is echoed as written");
        assert!(hints[0].suggestions.contains(&"FILTER".to_string()), "got {hints:?}");
    }

    #[test]
    fn an_ordinary_syntax_error_produces_no_hints() {
        // The load-bearing anti-noise property: a query that fails for a structural reason
        // (unbalanced brace) has no mistyped keyword, so the diagnostic stays silent rather
        // than inventing a suggestion.
        assert!(keyword_hints("SELECT ?s WHERE { ?s ?p").is_empty());
        assert!(keyword_hints("SELECT ?s WHERE { ?s ?p ?o . . }").is_empty());
    }

    #[test]
    fn valid_queries_are_never_flagged() {
        // Every token of a well-formed query — keywords in mixed case, `a`, prefixed names,
        // variables, literals with language tags and datatypes, blank nodes — is recognised.
        for q in [
            "PREFIX ex: <http://ex/> SELECT DISTINCT ?s WHERE { ?s a ex:Thing } LIMIT 10",
            "SELECT (COUNT(?s) AS ?n) WHERE { ?s ?p \"hi\"@en-GB } GROUP BY ?p HAVING (?n > 1)",
            "ASK { _:b1 <http://ex/p> \"7\"^^<http://www.w3.org/2001/XMLSchema#integer> }",
            "select ?s where { ?s ?p ?o . filter(isIRI(?o) && bound(?o)) } order by desc(?s)",
            "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }",
        ] {
            assert!(keyword_hints(q).is_empty(), "false positive in: {q}");
        }
    }

    #[test]
    fn unicode_names_and_pn_local_escapes_are_never_flagged() {
        // `valid_queries_are_never_flagged` only exercises ASCII names, so it cannot pin the
        // contract for the name characters the ASCII word scanner cannot walk: a non-ASCII
        // `PN_CHARS` byte and a `PN_LOCAL_ESC`. Both used to read as delimiters, which started
        // a candidate word *inside* a valid name and reported its keyword-ish ASCII tail as a
        // typo. These queries are checked against the real vendored parser first, so the test
        // asserts the actual contract ("valid query ⇒ no hints") rather than a guess at one.
        for q in [
            // `é` (U+00E9) is a legal VARNAME character; `selct` is one edit from SELECT.
            "SELECT ?aéselct WHERE { ?aéselct ?p ?o }",
            // ...and in the local part of a prefixed name, and in a blank node label.
            "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:péselct ?o }",
            "SELECT ?s WHERE { _:béwhre <http://ex/p> ?s }",
            // `\~` is a PN_LOCAL_ESC, not a name terminator.
            r"PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:a\~selct ?o }",
        ] {
            assert!(
                spargebra::SparqlParser::new().parse_query(q).is_ok(),
                "test query is not actually valid SPARQL: {q}"
            );
            assert!(keyword_hints(q).is_empty(), "false positive in: {q}");
        }

        // Positive control: the wider name boundary must not silence a genuine bare typo that
        // merely shares a query with a Unicode name.
        let hints = keyword_hints("SELET ?aéselct WHERE { ?aéselct ?p ?o }");
        assert_eq!(hints.len(), 1, "got {hints:?}");
        assert_eq!(hints[0].token, "SELET");
        assert_eq!(hints[0].suggestions[0], "SELECT");
    }

    #[test]
    fn words_inside_literals_iris_and_comments_are_data() {
        // `FLTR` here is text, not a token position — flagging it would be noise.
        let hints = keyword_hints("SELECT ?s WHERE { ?s <http://ex/FLTR> \"FLTR\" } # FLTR\n");
        assert!(hints.is_empty(), "got {hints:?}");
    }

    #[test]
    fn prefixed_names_and_variables_are_not_keywords() {
        // `ordr:` is a prefix label and `?wher` a variable — neither is a keyword position,
        // even though both are one edit from a real keyword.
        let hints =
            keyword_hints("PREFIX ordr: <http://ex/> SELECT ?wher WHERE { ?wher ordr:selct ?o }");
        assert!(hints.is_empty(), "got {hints:?}");
    }

    #[test]
    fn distant_words_get_no_suggestion() {
        // A word nowhere near a keyword yields nothing — a hint is only offered when it is
        // actually close, never as a guess.
        assert!(hint_for("zzzzzzzz").is_none());
        // ...and a two-character word only matches at distance 1 (tight budget).
        assert!(hint_for("qx").is_none());
    }

    #[test]
    fn hints_are_deduped_ordered_and_capped() {
        // Four distinct typos, one repeated: reported in source order, deduped, capped.
        let q = "SELET ?s WHRE { ?s ?p ?o } SELET ?x ORDR BY ?s LIMTI 5";
        let hints = keyword_hints(q);
        assert_eq!(hints.len(), MAX_HINTS, "capped at {MAX_HINTS}: {hints:?}");
        let tokens: Vec<&str> = hints.iter().map(|h| h.token.as_str()).collect();
        assert_eq!(tokens, vec!["SELET", "WHRE", "ORDR"], "source order, deduped");
        assert!(hints.iter().all(|h| h.suggestions.len() <= MAX_SUGGESTIONS));
        assert_eq!(hints[0].suggestions[0], "SELECT");
        assert_eq!(hints[1].suggestions[0], "WHERE");
    }

    #[test]
    fn keyword_set_matches_the_vendored_grammar() {
        // Anti-drift: every keyword terminal of the parser whose rejection triggers this
        // diagnostic must be in KEYWORDS. Without this, a grammar update (e.g. a new SPARQL
        // 1.2 keyword) would leave the diagnostic flagging a keyword the parser accepts.
        let grammar = include_str!("../../../vendor/spargebra/src/parser.rs");
        let terminals = grammar_terminals(grammar);
        assert!(terminals.len() > 100, "extraction looks broken: {} terminals", terminals.len());
        for terminal in &terminals {
            assert!(
                KEYWORDS.iter().any(|kw| kw == terminal),
                "vendored grammar has keyword `{terminal}` that KEYWORDS is missing"
            );
        }
        for kw in KEYWORDS {
            assert!(
                terminals.iter().any(|t| t == kw),
                "KEYWORDS has `{kw}` which the vendored grammar no longer accepts"
            );
        }
    }

    /// Extracts the `i("KEYWORD")` case-insensitive terminals from the vendored PEG grammar.
    fn grammar_terminals(grammar: &str) -> Vec<String> {
        let bytes = grammar.as_bytes();
        let mut out: Vec<String> = Vec::new();
        for i in grammar.match_indices("i(\"").map(|(i, _)| i) {
            // Require a token boundary before the `i` so a call like `Uri("…")` cannot match.
            if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
                continue;
            }
            let rest = &grammar[i + 3..];
            let Some(end) = rest.find('"') else { continue };
            let word = &rest[..end];
            if !word.is_empty()
                && word.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                && !out.iter().any(|w| w == word)
            {
                out.push(word.to_string());
            }
        }
        out
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("FLTR", "FILTER"), 2);
    }

    #[cfg(feature = "vectors")]
    #[test]
    fn local_edit_distance_agrees_with_the_shared_helper() {
        // The crate-local copy exists only because the lean default build cannot depend on
        // sparq-core. Where sparq-core *is* in the graph, assert the two are the same
        // function, so the copy cannot drift from `sparq_core::strdist::edit_distance`.
        for a in ["", "a", "FLTR", "SELECT", "kitten", "STRLANGDIR"] {
            for b in ["", "A", "FILTER", "SELET", "sitting", "STRLANG"] {
                assert_eq!(
                    edit_distance(a, b),
                    sparq_core::strdist::edit_distance(a, b),
                    "drift on ({a}, {b})"
                );
            }
        }
    }
}
