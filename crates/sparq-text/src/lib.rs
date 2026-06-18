//! sparq-text: **opt-in full-text search over literals** for the sparq RDF engine.
//!
//! Three layers, bottom-up:
//!
//! 1. [`tokenize`] — the shared tokenizer: UAX #29 Unicode word segmentation
//!    (`unicode-segmentation`) + Unicode lowercasing. No stemming, no stopword
//!    list, no diacritic folding — deterministic and language-neutral; query
//!    tokens may end in `*` for prefix (autocomplete) matching. An opt-in
//!    [`Analyzer::CjkNgram`] adds character-bigram indexing for unspaced CJK
//!    text (`東京都` → `東京`,`京都`) so a multi-char CJK term is no longer the
//!    low-precision AND of its individual ideographs; the default
//!    [`Analyzer::Unicode`] is byte-for-byte unchanged. [OPUS-4.8] sq-m3ln
//! 2. [`index`] — [`TextIndex`]: an owned BM25 inverted index over the string
//!    literals (`xsd:string` + language-tagged) of a sparq
//!    [`Graph`](sparq_core::Graph)'s dictionary. The dictionary term id of a
//!    literal IS its document id, so [`TextIndex::search`] /
//!    [`search_any`](TextIndex::search_any) return literal ids that join back
//!    to triples through the store's ordinary permutation indexes. Deltas are
//!    mirrored incrementally via [`TextIndex::apply_delta`] (the
//!    `GeoIndex::apply_delta` shape).
//! 3. [`rewrite`] (default-on `engine` feature) — the `text:` magic
//!    predicates ([`vocab`]): `?lit text:matches "query"` (AND of tokens,
//!    `*`-suffix prefix tokens), `?lit text:matchesAny "query"` (OR),
//!    `?lit text:phrase "foo bar"` (adjacent, in-order tokens — needs a
//!    positions-enabled index), `?lit text:near "foo bar"` (proximity/slop:
//!    in-order within a bounded gap, relevance-ranked — `text:slop N` sets the
//!    gap budget), and `?lit text:score ?s` (the relevance score). [OPUS-4.8]
//!    [`query_text`] rewrites them into
//!    inline `VALUES` over the index's hits at the spargebra-algebra level and
//!    executes through sparq-engine's prepared-query seam
//!    (`PreparedQuery: From<spargebra::Query>`) — the engine itself is
//!    untouched.
//!
//! No existing sparq crate depends on this one (in particular the wasm build
//! carries zero text-search code); full-text support is engaged only by
//! depending on `sparq-text` — mirroring how `sparq-geo` and `sparq-vectors`
//! stay out of the default build.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod index;
#[cfg(feature = "engine")]
pub mod rewrite;
pub mod tokenize;

pub use index::{Hit, TextIndex};
pub use tokenize::Analyzer;
#[cfg(feature = "engine")]
pub use rewrite::{prepare_text, query_text, query_text_with_budget, rewrite_query};

/// The `text:` vocabulary — magic predicates recognised by [`rewrite`]
/// (`http://sparq.dev/text#`, the sparq extension namespace).
pub mod vocab {
    /// `text:` — the sparq full-text-search namespace.
    pub const TEXT_NS: &str = "http://sparq.dev/text#";
    /// `?lit text:matches "query"` — `?lit` ranges over indexed literals
    /// containing EVERY query token (tokens ending in `*` match as prefixes).
    pub const MATCHES: &str = "http://sparq.dev/text#matches";
    /// `?lit text:matchesAny "query"` — literals containing AT LEAST ONE token.
    pub const MATCHES_ANY: &str = "http://sparq.dev/text#matchesAny";
    /// `?lit text:phrase "foo bar"` — literals where the query's tokens appear
    /// ADJACENT and IN ORDER (a positional phrase match, not a BM25 ranking).
    /// Requires a positions-enabled index (`TextIndex::build_with_positions`);
    /// the cheap default index stores no positions and is rejected at rewrite
    /// time. No `text:score` companion (a phrase match is boolean adjacency).
    /// [OPUS-4.8]
    pub const PHRASE: &str = "http://sparq.dev/text#phrase";
    /// `?lit text:near "foo bar"` — the proximity/slop generalisation of
    /// `text:phrase`: literals where the tokens occur IN ORDER within a bounded
    /// total gap, RELEVANCE-RANKED (tighter clustering scores higher). The gap
    /// budget defaults to [`DEFAULT_SLOP`](crate::rewrite::DEFAULT_SLOP); set it
    /// with a `?lit text:slop N` companion (a non-negative integer literal) on
    /// the same subject variable in the same basic graph pattern. Unlike
    /// `text:phrase` (boolean adjacency) it IS scored, so it also takes an
    /// optional `text:score ?s` companion. Requires a positions-enabled index
    /// (`TextIndex::build_with_positions`); `text:near "foo bar"` at slop 0 is
    /// exactly `text:phrase "foo bar"`. [OPUS-4.8]
    pub const NEAR: &str = "http://sparq.dev/text#near";
    /// `?lit text:slop N` — sets the proximity gap budget for the `text:near`
    /// on the same subject variable in the same basic graph pattern (a
    /// non-negative `xsd:integer`). Only meaningful alongside `text:near`.
    /// [OPUS-4.8]
    pub const SLOP: &str = "http://sparq.dev/text#slop";
    /// `?lit text:score ?s` — binds the relevance score of `?lit`'s match: the
    /// BM25 score for a `text:matches`/`text:matchesAny`, or the proximity score
    /// for a `text:near` (`1/(1+gap)`). Must accompany exactly one such scored
    /// match on the same subject variable in the same basic graph pattern. NOT
    /// valid for `text:phrase` (boolean adjacency, unscored). [OPUS-4.8]
    pub const SCORE: &str = "http://sparq.dev/text#score";
}
