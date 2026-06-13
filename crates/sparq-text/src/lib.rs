//! sparq-text: **opt-in full-text search over literals** for the sparq RDF engine.
//!
//! Three layers, bottom-up:
//!
//! 1. [`tokenize`] — the shared tokenizer: UAX #29 Unicode word segmentation
//!    (`unicode-segmentation`) + Unicode lowercasing. No stemming, no stopword
//!    list, no diacritic folding — deterministic and language-neutral; query
//!    tokens may end in `*` for prefix (autocomplete) matching.
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
//!    positions-enabled index), and `?lit text:score ?s` (the BM25 score).
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

pub mod index;
#[cfg(feature = "engine")]
pub mod rewrite;
pub mod tokenize;

pub use index::{Hit, TextIndex};
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
    /// Requires a positions-enabled index ([`TextIndex::build_with_positions`]);
    /// the cheap default index stores no positions and is rejected at rewrite
    /// time. No `text:score` companion (a phrase match is boolean adjacency).
    /// [OPUS-4.8]
    pub const PHRASE: &str = "http://sparq.dev/text#phrase";
    /// `?lit text:score ?s` — binds the BM25 score of `?lit`'s match (must
    /// accompany exactly one `text:matches`/`text:matchesAny` on the same
    /// subject variable in the same basic graph pattern).
    pub const SCORE: &str = "http://sparq.dev/text#score";
}
