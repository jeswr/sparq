//! [OPUS-5] The non-rewriting **did-you-mean diagnostic** (design §3.2, Phase 4, sq-h7zlx) —
//! the only sliver of lever 2 the design record endorses.
//!
//! Lever 2 as originally sketched (lenient parsing: accept `FLTR` and quietly read it as
//! `FILTER`) is **recommend-against** in the design (§3.2/§7): silently repairing a keyword
//! can turn a typo into a *different, valid, wrong* query, and it destroys the agent's loud,
//! recoverable feedback loop. So this module implements the *other* half of that idea and
//! nothing more:
//!
//! - It runs **on parse failure only** — [`crate::transpile::canary`] calls it after
//!   `spargebra` has already rejected the emission, never on a query that parses.
//! - It **suggests, never applies**. The suggestion rides along on
//!   [`crate::TerseError::CanaryFailed`] as data; the canonical SPARQL that failed is
//!   returned untouched, and the call still fails. There is no path by which a suggestion
//!   changes the query.
//!
//! This is the same loud-fail-with-a-hint shape as `crates/sparq-nlq/src/constrain.rs`'s
//! IRI-level did-you-mean (design §3.2: "exactly `constrain.rs`'s existing did-you-mean,
//! lifted to the keyword level") and as the `K:<name>` suggestions on
//! [`crate::TerseError::UnknownKeyword`].
//!
//! **Honest value framing.** Keyword typos are a *rare* first-shot failure mode (grounding is
//! the real one — design §1.1), and the Phase-5 A/B (`bench/terse/RESULTS.md`) measured
//! levers 1 and 3 only, so there is **no measurement** showing this diagnostic helps. It is
//! deliberately tiny, costs nothing on the success path, and cannot change a query — that is
//! the whole justification for shipping it.

/// One did-you-mean hint about a word in SPARQL that failed to parse: the token as written
/// and the nearest SPARQL keyword to it. Carried by [`crate::TerseError::CanaryFailed`].
///
/// **It is a diagnostic, never a repair** (design §3.2). Nothing in this crate ever
/// substitutes `suggestion` for `token`; the query that failed to parse is returned verbatim
/// and the call still errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordSuggestion {
    /// The word as it appears in the query (original case), e.g. `FLTR`.
    pub token: String,
    /// The nearest SPARQL keyword, in the canonical upper-case spelling, e.g. `FILTER`.
    pub suggestion: String,
}

/// At most this many hints per failure — a parse error with a page of suggestions is noise,
/// and the *parse error itself* is the primary signal.
const MAX_SUGGESTIONS: usize = 3;

/// Stop scanning after this many bare words. The scan only runs on the failure path, but that
/// path is reachable from hostile input (the `parse_terse` fuzz target), so the work is
/// bounded rather than proportional to an attacker-chosen input size.
const MAX_WORDS_SCANNED: usize = 4_096;

/// Words shorter than this are never suggested against: at 1–2 characters almost every
/// token is within the edit budget of some keyword, so the hints would be noise.
const MIN_WORD_LEN: usize = 3;

/// Every keyword the vendored `spargebra` grammar accepts, upper-cased and **sorted** (the
/// lookup binary-searches it; `table_is_sorted_and_unique` pins both properties). Source of
/// truth: the `i("…")` case-insensitive literals in `vendor/spargebra/src/parser.rs`, plus
/// the bare `a` predicate keyword — `table_covers_every_vendored_parser_keyword` fails if
/// that grammar grows a keyword this table lacks. Completeness matters in one direction only:
/// a keyword MISSING here would make a legitimate token look like a typo (a false hint), so
/// the table is a superset, never a curated subset.
const KEYWORDS: &[&str] = &[
    "A",
    "ABS",
    "ADD",
    "ADJUST",
    "ALL",
    "AS",
    "ASC",
    "ASK",
    "AVG",
    "BASE",
    "BIND",
    "BNODE",
    "BOUND",
    "BY",
    "CEIL",
    "CLEAR",
    "COALESCE",
    "CONCAT",
    "CONSTRUCT",
    "CONTAINS",
    "COPY",
    "COUNT",
    "CREATE",
    "DATA",
    "DATATYPE",
    "DAY",
    "DEFAULT",
    "DELETE",
    "DESC",
    "DESCRIBE",
    "DISTINCT",
    "DROP",
    "ENCODE_FOR_URI",
    "EXISTS",
    "FALSE",
    "FILTER",
    "FLOOR",
    "FROM",
    "GRAPH",
    "GROUP",
    "GROUP_CONCAT",
    "HASLANG",
    "HASLANGDIR",
    "HAVING",
    "HOURS",
    "IF",
    "IN",
    "INSERT",
    "INTO",
    "IRI",
    "ISBLANK",
    "ISIRI",
    "ISLITERAL",
    "ISNUMERIC",
    "ISTRIPLE",
    "ISURI",
    "LANG",
    "LANGDIR",
    "LANGMATCHES",
    "LATERAL",
    "LCASE",
    "LIMIT",
    "LOAD",
    "MAX",
    "MD5",
    "MIN",
    "MINUS",
    "MINUTES",
    "MONTH",
    "MOVE",
    "MULTIPLICITY",
    "NAMED",
    "NOT",
    "NOW",
    "OBJECT",
    "OFFSET",
    "OPTIONAL",
    "ORDER",
    "PREDICATE",
    "PREFIX",
    "RAND",
    "REDUCED",
    "REGEX",
    "REPLACE",
    "ROUND",
    "SAMETERM",
    "SAMPLE",
    "SECONDS",
    "SELECT",
    "SEPARATOR",
    "SERVICE",
    "SHA1",
    "SHA256",
    "SHA384",
    "SHA512",
    "SILENT",
    "STR",
    "STRAFTER",
    "STRBEFORE",
    "STRDT",
    "STRENDS",
    "STRLANG",
    "STRLANGDIR",
    "STRLEN",
    "STRSTARTS",
    "STRUUID",
    "SUBJECT",
    "SUBSTR",
    "SUM",
    "TIMEZONE",
    "TO",
    "TRIPLE",
    "TRUE",
    "TZ",
    "UCASE",
    "UNDEF",
    "UNION",
    "URI",
    "USING",
    "UUID",
    "VALUES",
    "VERSION",
    "WHERE",
    "WITH",
    "YEAR",
];

/// Scans SPARQL that has **already failed to parse** and returns up to [`MAX_SUGGESTIONS`]
/// did-you-mean hints, in source order, deduplicated by token.
///
/// A candidate is a *bare word* — a token in keyword position. Everything that is lexically
/// something else is skipped, so it can never be mistaken for a mistyped keyword: string
/// literals, `<IRI>`s, `#` comments, `?var`/`$var`, `@lang` tags, numbers, and prefixed
/// names / blank-node labels (`ex:Cat`, `_:b1` — the local part of a prefixed name is data,
/// not a keyword). What remains in valid SPARQL is essentially keywords only, which is why a
/// bare word that is *not* a keyword is worth a hint.
pub(crate) fn keyword_suggestions(src: &str) -> Vec<KeywordSuggestion> {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut out: Vec<KeywordSuggestion> = Vec::new();
    let mut words = 0usize;
    let mut i = 0usize;
    while i < n && out.len() < MAX_SUGGESTIONS && words < MAX_WORDS_SCANNED {
        let c = bytes[i];
        match c {
            b'<' => {
                // An IRI ref: <...> (a newline cancels it, matching SPARQL's IRIREF lexing).
                let mut j = i + 1;
                while j < n && bytes[j] != b'>' && bytes[j] != b'\n' {
                    j += 1;
                }
                i = if j < n { j + 1 } else { n };
            }
            b'#' => {
                // A line comment to end-of-line.
                let mut j = i + 1;
                while j < n && bytes[j] != b'\n' {
                    j += 1;
                }
                i = j;
            }
            b'"' | b'\'' => i = crate::transpile::skip_string(bytes, i),
            b'?' | b'$' | b'@' => {
                // A variable (?x / $x) or a language tag (@en-GB): the name is not a keyword.
                let mut j = i + 1;
                while j < n
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-')
                {
                    j += 1;
                }
                i = j.max(i + 1);
            }
            b'0'..=b'9' => {
                // A numeric literal (including a decimal/exponent tail): never a keyword.
                let mut j = i + 1;
                while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'.') {
                    j += 1;
                }
                i = j;
            }
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let mut j = i + 1;
                while j < n && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                if j < n && bytes[j] == b':' {
                    // A prefixed name (`ex:Cat`) or blank-node label (`_:b1`): skip the whole
                    // token, local part included — neither half sits in keyword position.
                    j += 1;
                    while j < n
                        && (bytes[j].is_ascii_alphanumeric()
                            || bytes[j] == b'_'
                            || bytes[j] == b'-'
                            || bytes[j] == b'.')
                    {
                        j += 1;
                    }
                    i = j;
                    continue;
                }
                // ASCII-only by construction, so `i..j` is a char boundary and the word is
                // safe to upper-case bytewise.
                let word = &src[i..j];
                words += 1;
                if let Some(kw) = nearest_keyword(word) {
                    if !out.iter().any(|s| s.token.eq_ignore_ascii_case(word)) {
                        out.push(KeywordSuggestion {
                            token: word.to_string(),
                            suggestion: kw.to_string(),
                        });
                    }
                }
                i = j;
            }
            _ => i += 1,
        }
    }
    out
}

/// The nearest SPARQL keyword to `word`, or `None` when `word` is already a keyword (SPARQL
/// keywords are case-insensitive) or nothing is close enough to be worth saying.
///
/// "Close enough" is deliberately tight, because a wrong hint is worse than none:
/// - edit distance ≤ 2 for a word of 4+ characters (this is what catches the design's
///   motivating `FLTR` → `FILTER`, a two-vowel elision), ≤ 1 for a 3-character word;
/// - the first character must agree (case-insensitively). Real keyword typos are elisions,
///   transpositions and doubled letters *inside* a word the model already knows; requiring
///   the initial letter is what stops noise like `and` → `RAND`.
///
/// Ties break on the alphabetically-first keyword so the hint is deterministic.
///
/// It is still only a heuristic: a bare word that was never a keyword typo can sit one edit
/// from a keyword (`and` → `ADD`), so a hint can be unhelpful. That is tolerable *because it
/// is never applied* — the parse error is the primary signal and the query is unchanged. It
/// would NOT be tolerable in the lenient-parse design the record rejects.
fn nearest_keyword(word: &str) -> Option<&'static str> {
    if word.len() < MIN_WORD_LEN {
        return None;
    }
    let upper = word.to_ascii_uppercase();
    if KEYWORDS.binary_search(&upper.as_str()).is_ok() {
        return None;
    }
    let budget = if upper.len() >= 4 { 2 } else { 1 };
    let first = upper.as_bytes()[0];
    let mut best: Option<(usize, &'static str)> = None;
    for &kw in KEYWORDS {
        if kw.as_bytes()[0] != first || kw.len().abs_diff(upper.len()) > budget {
            continue;
        }
        let d = edit_distance(upper.as_bytes(), kw.as_bytes());
        if d == 0 || d > budget {
            continue;
        }
        if best.is_none_or(|(bd, bk)| d < bd || (d == bd && kw < bk)) {
            best = Some((d, kw));
        }
    }
    best.map(|(_, kw)| kw)
}

/// Levenshtein distance over two ASCII byte strings (two-row DP).
///
/// Deliberately a local copy of `sparq_core::strdist::edit_distance` (the metric
/// `sparq-nlq/src/constrain.rs` ranks its did-you-mean with): the lean default build of this
/// crate depends on `spargebra` alone, and a ~15-line ASCII distance function is not worth
/// pulling `sparq-core` into it. Both inputs here are ASCII by construction, so bytes and
/// characters coincide.
fn edit_distance(a: &[u8], b: &[u8]) -> usize {
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
    fn table_is_sorted_and_unique() {
        // The lookup binary-searches KEYWORDS, so sortedness is load-bearing.
        for pair in KEYWORDS.windows(2) {
            assert!(pair[0] < pair[1], "KEYWORDS must be sorted+unique: {pair:?}");
        }
    }

    #[test]
    fn table_covers_every_vendored_parser_keyword() {
        // Drift guard: a keyword the grammar accepts but this table lacks would make a
        // LEGITIMATE token look like a typo. The vendored parser spells every keyword as a
        // case-insensitive literal `i("…")`, so the grammar is machine-checkable against the
        // table. Skipped (not failed) if the vendored source is not readable from here — the
        // check is a convenience, never a reason for a red test on a packaged build.
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/spargebra/src/parser.rs");
        let Ok(grammar) = std::fs::read_to_string(path) else {
            return;
        };
        let mut missing: Vec<String> = Vec::new();
        let mut rest = grammar.as_str();
        while let Some(at) = rest.find("i(\"") {
            rest = &rest[at + 3..];
            let Some(end) = rest.find('"') else { break };
            let token = &rest[..end];
            rest = &rest[end + 1..];
            if token.is_empty()
                || !token.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                continue;
            }
            let upper = token.to_ascii_uppercase();
            if KEYWORDS.binary_search(&upper.as_str()).is_err() && !missing.contains(&upper) {
                missing.push(upper);
            }
        }
        assert!(
            missing.is_empty(),
            "vendored spargebra accepts keywords missing from KEYWORDS: {missing:?}"
        );
    }

    #[test]
    fn suggests_the_designs_motivating_typo() {
        // design §3.2: "unknown keyword `FLTR` — did you mean `FILTER`?"
        let hints = keyword_suggestions("SELECT ?s WHERE { ?s ?p ?o FLTR(?o > 1) }");
        assert_eq!(hints.len(), 1, "got {hints:?}");
        assert_eq!(hints[0].token, "FLTR");
        assert_eq!(hints[0].suggestion, "FILTER");
    }

    #[test]
    fn suggests_common_elisions_and_transpositions() {
        for (typo, expected) in [
            ("SLECT", "SELECT"),
            ("WEHRE", "WHERE"),
            ("OPTINAL", "OPTIONAL"),
            ("distnct", "DISTINCT"),
        ] {
            let hints = keyword_suggestions(&format!("{typo} ?s"));
            assert_eq!(hints.first().map(|h| h.suggestion.as_str()), Some(expected), "{typo}");
        }
    }

    #[test]
    fn well_spelled_sparql_gets_no_hints() {
        // Every keyword form the scanner meets in ordinary SPARQL must be recognised — a hint
        // on a correctly-spelled query is the false positive that would make this noise.
        let queries = [
            "PREFIX ex: <http://ex/> SELECT DISTINCT ?s WHERE { ?s a ex:Cat ; ex:name ?n \
             FILTER(STRLEN(?n) > 3) } ORDER BY DESC(?n) LIMIT 10",
            "SELECT (COUNT(*) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g HAVING (?c > 1)",
            "INSERT DATA { <http://ex/s> <http://ex/p> \"cat\"@en-GB }",
            "ASK { ?s ?p ?o . FILTER NOT EXISTS { ?s <http://ex/q> 1.5e3 } }",
            "SELECT ?s WHERE { ?s ?p ?o OPTIONAL { ?o <http://ex/r> _:b1 } } OFFSET 5",
        ];
        for q in queries {
            assert!(keyword_suggestions(q).is_empty(), "false hint on: {q}");
        }
    }

    #[test]
    fn non_keyword_lexical_contexts_are_never_candidates() {
        // A typo-shaped word inside a literal / IRI / comment / prefixed local name is DATA:
        // suggesting against it would be pure noise.
        let src = "SELECT ?s WHERE { ?s <http://ex/FLTR> \"FLTR\" . # FLTR\n ?s ex:FLTR ?o }";
        assert!(keyword_suggestions(src).is_empty(), "got {:?}", keyword_suggestions(src));
    }

    #[test]
    fn distant_words_and_short_words_get_no_hint() {
        // A word that is not plausibly a keyword typo yields nothing — a loud parse error with
        // no hint is the correct outcome, never a stretched guess.
        assert!(keyword_suggestions("SELECT ?s WHERE { ?s zzzzzz ?o }").is_empty());
        // `ILTER` is one edit from FILTER, but a DROPPED INITIAL letter is deliberately out of
        // scope: the shared-first-character rule buys precision at the cost of this recall.
        assert!(keyword_suggestions("SELECT ?s WHERE { ?s ?p ?o ILTER(?o) }").is_empty());
        // Below the length floor.
        assert!(keyword_suggestions("SELECT ?s WHERE { ?s ?p qq }").is_empty());
    }

    #[test]
    fn hints_are_capped_and_deduplicated() {
        let src = "SLECT FLTR WEHRE OPTINAL DISTNCT";
        let hints = keyword_suggestions(src);
        assert_eq!(hints.len(), MAX_SUGGESTIONS, "capped at {MAX_SUGGESTIONS}: {hints:?}");
        // Source order, and the same token twice is reported once.
        assert_eq!(hints[0].token, "SLECT");
        let repeated = keyword_suggestions("FLTR ?s FLTR");
        assert_eq!(repeated.len(), 1, "got {repeated:?}");
    }

    #[test]
    fn hints_are_well_formed_on_arbitrary_input() {
        // The scan runs on the failure path, which hostile input reaches (the `parse_terse`
        // fuzz target). Whatever the bytes, the diagnostic must terminate and stay
        // well-formed: bounded, deduplicated, and every hint a REAL keyword suggested for a
        // token that is NOT one (a hint pointing at a keyword the parser already accepts
        // would be actively misleading).
        use proptest::prelude::*;
        use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};

        TestRunner::new_with_rng(
            Config { cases: 256, failure_persistence: None, ..Config::default() },
            TestRng::from_seed(RngAlgorithm::ChaCha, &[0x4d; 32]),
        )
        .run(&any::<String>(), |src| {
            let hints = keyword_suggestions(&src);
            prop_assert!(hints.len() <= MAX_SUGGESTIONS);
            for (n, hint) in hints.iter().enumerate() {
                prop_assert!(KEYWORDS.binary_search(&hint.suggestion.as_str()).is_ok());
                let token = hint.token.to_ascii_uppercase();
                prop_assert!(KEYWORDS.binary_search(&token.as_str()).is_err());
                prop_assert!(!hints[..n].iter().any(|e| e.token.eq_ignore_ascii_case(&hint.token)));
            }
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn edit_distance_matches_the_reference_metric() {
        assert_eq!(edit_distance(b"FLTR", b"FILTER"), 2);
        assert_eq!(edit_distance(b"", b"ASK"), 3);
        assert_eq!(edit_distance(b"ASK", b"ASK"), 0);
    }
}
