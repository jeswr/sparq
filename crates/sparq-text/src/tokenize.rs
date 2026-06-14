//! The shared tokenizer: UAX #29 Unicode word segmentation + Unicode
//! lowercasing.
//!
//! Design (deliberately minimal — deterministic and language-neutral):
//!
//! - **Segmentation**: `unicode-segmentation`'s `unicode_words` (UAX #29 word
//!   boundaries, keeping only word-like segments — punctuation and whitespace
//!   never become tokens; numbers like `42` or `3.14` do).
//! - **Folding**: full Unicode lowercase (`str::to_lowercase`) — so `Fox` ==
//!   `fox`, `ΣΟΦΙΑ` == `σοφια`, `STRASSE` == `strasse`. No stemming, no
//!   stopword removal, no diacritic stripping (`café` ≠ `cafe`): exact-token
//!   semantics are predictable across languages and never throw data away.
//! - **CJK caveat**: there is no dictionary-based word splitting. UAX #29 emits
//!   each Han/CJK ideograph as its OWN word segment (`東京都` → `東`,`京`,`都`),
//!   so an unspaced ideograph run becomes one token per character — a query
//!   token matches a document iff that ideograph occurs in it, and a multi-char
//!   CJK term behaves like the AND of its characters (use `phrase` for adjacency,
//!   which then requires the characters consecutive). Kana/Hangul runs that
//!   UAX #29 treats as a single word stay one token. (Empirically pinned in
//!   `tests/analyzer_edges.rs`.) [OPUS-4.8]
//!
//! Query strings go through [`tokenize_query`], which additionally recognises
//! a trailing `*` on a token as a PREFIX (autocomplete) marker: `auto*`
//! matches `automatic`, `autonomous`, …. The `*` is only meaningful
//! immediately after a token; anywhere else it is segmentation punctuation
//! and disappears.

use unicode_segmentation::UnicodeSegmentation;

/// Tokenizes document text (a literal's lexical form): UAX #29 words,
/// lowercased, in order (duplicates preserved — term frequency matters).
pub fn tokenize(text: &str) -> Vec<String> {
    text.unicode_words().map(str::to_lowercase).collect()
}

/// One parsed query token: the lowercased token text and whether it is a
/// prefix (`*`-suffixed) token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryToken {
    pub token: String,
    pub prefix: bool,
}

/// Tokenizes a query string: [`tokenize`]'s exact pipeline, plus a token
/// immediately followed by `*` becomes a prefix token.
pub fn tokenize_query(query: &str) -> Vec<QueryToken> {
    query
        .unicode_word_indices()
        .map(|(start, word)| QueryToken {
            token: word.to_lowercase(),
            prefix: query[start + word.len()..].starts_with('*'),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(text: &str) -> Vec<String> {
        tokenize(text)
    }

    #[test]
    fn words_and_punctuation() {
        assert_eq!(toks("The quick, brown fox!"), ["the", "quick", "brown", "fox"]);
        assert_eq!(toks("state-of-the-art"), ["state", "of", "the", "art"]);
        // Apostrophes stay inside a UAX #29 word.
        assert_eq!(toks("can't stop"), ["can't", "stop"]);
        assert_eq!(toks(""), Vec::<String>::new());
        assert_eq!(toks("  \t\n .,;!? "), Vec::<String>::new());
    }

    #[test]
    fn numbers_are_tokens() {
        assert_eq!(toks("RFC 3986 v2.1"), ["rfc", "3986", "v2.1"]);
    }

    #[test]
    fn unicode_case_folding() {
        assert_eq!(toks("ΣΟΦΊΑ Sophia"), ["σοφία", "sophia"]);
        assert_eq!(toks("МОСКВА"), ["москва"]);
        // ß has no uppercase round-trip issue on the lowercase-only fold.
        assert_eq!(toks("Straße STRASSE"), ["straße", "strasse"]);
    }

    #[test]
    fn no_diacritic_folding() {
        // Documented: café and cafe are DIFFERENT tokens.
        assert_eq!(toks("Café"), ["café"]);
        assert_ne!(toks("café"), toks("cafe"));
    }

    #[test]
    fn duplicates_preserved_in_order() {
        assert_eq!(toks("fox fox FOX"), ["fox", "fox", "fox"]);
    }

    #[test]
    fn query_prefix_markers() {
        let q = tokenize_query("Auto* complete");
        assert_eq!(
            q,
            [
                QueryToken { token: "auto".into(), prefix: true },
                QueryToken { token: "complete".into(), prefix: false },
            ]
        );
        // `*` not adjacent to a token end is plain punctuation.
        assert_eq!(
            tokenize_query("* fox"),
            [QueryToken { token: "fox".into(), prefix: false }]
        );
        // A mid-word `*` splits the word; only the left half is a prefix token.
        assert_eq!(
            tokenize_query("auto*complete"),
            [
                QueryToken { token: "auto".into(), prefix: true },
                QueryToken { token: "complete".into(), prefix: false },
            ]
        );
    }
}
