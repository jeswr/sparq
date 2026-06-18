//! [`TextIndex`]: an owned BM25 inverted index over the string literals of a
//! sparq [`Graph`]'s dictionary.
//!
//! ## What is indexed
//!
//! [`TextIndex::build`] scans the graph's dictionary ONCE (sharded across
//! rayon workers under the `parallel` feature; the resulting index is
//! identical) and indexes every **string literal**: plain/`xsd:string`
//! literals and language-tagged literals. Typed literals (numbers, dates,
//! `geo:wktLiteral`, …) are not text and are skipped. The literal's
//! dictionary term id ([`sparq_core::dict::Id`]) is its **document id** — a
//! search returns matching literal ids (or terms), and joining those back to
//! subjects/predicates is the store's ordinary permutation-index work, not
//! the text index's.
//!
//! Named graphs keep their own dictionaries (`Graph::named`), so an index is
//! per-graph: build one over `graph` for the default graph, or over a named
//! graph's `Graph` for that graph (ids are dictionary-local).
//!
//! ## Index design
//!
//! A classic small inverted index, owned in full (no tantivy):
//!
//! - `postings: BTreeMap<token, Vec<(doc id, term frequency)>>` — posting
//!   lists sorted by doc id (AND = sorted-list intersection); the `BTreeMap`'s
//!   ordered keys give prefix (`auto*`) expansion as a range scan.
//! - `docs: doc id -> token count` plus the total token count — the document
//!   lengths BM25 normalises by.
//!
//! **Scoring is BM25** (k1 = 1.2, b = 0.75, the standard Robertson/Sparck
//! Jones idf with the +1 floor) — chosen over raw tf because it is the
//! simplest scheme that both rewards rare terms and stops long literals from
//! dominating, and it costs only the two small tables above. A `*`-prefix
//! token scores as ONE pseudo-term: its expansions' postings are unioned
//! (term frequencies summed, document frequency = the union's size).
//!
//! ## Token positions (opt-in) and phrase queries [OPUS-4.8]
//!
//! By DEFAULT the base postings stay at 8 bytes per (token, doc) pair and no
//! positions are stored — every existing caller is byte-for-byte unchanged.
//! Positions are **opt-in**: [`TextIndex::build_with_positions`] (or
//! [`TextIndex::with_positions`] for the empty/delta-fed case) turns on a
//! SEPARATE parallel structure (`positions`) recording,
//! for each (token, doc), the token's 0-based offsets within that document. The
//! BM25 tables and the single-/multi-term [`search`](TextIndex::search) path are
//! identical either way; only the extra map is allocated, so the cheap default
//! never pays for what phrase search needs.
//!
//! With positions on, [`phrase`](TextIndex::phrase) answers ordered-adjacency
//! queries: `"foo bar"` matches a document only where `foo` is immediately
//! followed by `bar` (consecutive positions, in order). It honours the SAME
//! analyzer (UAX #29 segmentation + Unicode casefolding) as indexing, so phrase
//! tokens match exactly how the literals were tokenized.
//!
//! [`phrase_near`](TextIndex::phrase_near) is the proximity/slop generalisation
//! over the same positional postings: tokens still in order, but spread over a
//! bounded total gap (the `slop`), and RELEVANCE-RANKED — tighter clustering
//! scores higher (`1 / (1 + gap)`, so adjacency scores 1.0). `phrase_near(q, 0)`
//! is exactly `phrase(q)` (gap 0 = adjacency), and the hit set grows
//! monotonically with `slop`. [OPUS-4.8]
//!
//! ## Incremental maintenance
//!
//! [`apply_delta`](TextIndex::apply_delta) mirrors a [`Graph::apply_delta`]
//! batch: inserted triples whose object is a not-yet-indexed string literal
//! are tokenized and added (O(batch), independent of index size). Deletions
//! are deliberately a **no-op**: the dictionary retains terms after triple
//! deletion (only `Graph::compact` rebuilds it), so a rebuilt index would
//! contain the same documents — the incremental index stays EXACTLY equal to
//! a rebuild (the differential property `tests/delta.rs` pins). An orphaned
//! literal id is harmless downstream: joining it back to triples through the
//! permutations yields zero rows. In-process `Graph::compact` folds the fork
//! structure but keeps every dictionary id, so the index stays valid across it;
//! reach for a rebuild only when a *durably*-recompacted base is reopened in a
//! fresh process (the [`needs_rebuild`](TextIndex::needs_rebuild) case below).
//!
//! ## Durability / sharing: rebuild-on-boot + reconcile contract [OPUS-4.8]
//!
//! `TextIndex` is **in-memory, per-process, and not serialised** — there is no
//! on-disk index format and no cross-process sharing. Under a stateless-core
//! invariant (a server that rebuilds its in-memory state from the durable
//! [`Graph`] on every boot — the [`Graph`]'s WAL/blob *is* the source of truth)
//! the text index follows a **rebuild-on-boot + reconcile** contract rather than
//! a durable format:
//!
//! 1. **Boot:** call [`build`](TextIndex::build) (or
//!    [`build_with_positions`](TextIndex::build_with_positions)) over the freshly
//!    opened graph. This is the only sound entry point after a process restart —
//!    the index is a pure function of the dictionary, so a boot-time rebuild is
//!    always exactly what the live index would have been.
//! 2. **Reconcile (warm):** if a [`TextIndex`] survives in memory while the graph
//!    keeps growing (e.g. shared across requests),
//!    [`reconcile`](TextIndex::reconcile) brings it up to date in **O(new
//!    terms)** by scanning only the dictionary tail appended since the last
//!    build/reconcile — the boot-free fast path when you do not have the insert
//!    batches to hand for [`apply_delta`](TextIndex::apply_delta). After it
//!    returns, the index equals a fresh [`build`](TextIndex::build).
//! 3. **Staleness self-check:**
//!    [`is_consistent_with`](TextIndex::is_consistent_with) reports whether the
//!    index has seen every dictionary term the graph now holds (cheap — one
//!    `usize` compare), and [`needs_rebuild`](TextIndex::needs_rebuild) reports
//!    the one case reconcile canNOT repair incrementally: the graph's dictionary
//!    is **shorter** than what the index indexed, so dense ids must have been
//!    reassigned to *different* terms — the id→document mapping is stale and a
//!    full [`build`](TextIndex::build) is mandatory.
//!
//! The generation marker these methods pin to is
//! [`indexed_dict_len`](TextIndex::indexed_dict_len): the dictionary length
//! (count of interned terms) the index has indexed up to. Within one live
//! [`Graph`], dictionary ids are dense (`1..=len`) and the length only ever
//! GROWS — [`Graph::apply_delta`](sparq_core::Graph::apply_delta) appends and
//! [`Graph::compact`](sparq_core::Graph::compact) folds the fork structure
//! WITHOUT renumbering (every id `1..=len` keeps its term). So in-process a
//! longer dict always means "appends only since" (reconcilable) and
//! `is_consistent_with` is all you need. The shorter-dict case
//! [`needs_rebuild`](TextIndex::needs_rebuild) guards is the CROSS-process one: a
//! stateless core reopening a *durably compacted* base (whose persisted store
//! dropped orphaned terms and renumbered) loads a shorter dictionary than a
//! previously-built in-memory index expected — there the ids no longer line up,
//! so reconcile is unsound and a fresh boot-time [`build`](TextIndex::build) is
//! the contract. This marker is part of the index's state, so the incremental
//! and rebuilt indexes the differential test compares are equal on it too.

use crate::tokenize::{tokenize_query_with, tokenize_with, Analyzer, QueryToken};
use oxrdf::Term;
use rustc_hash::FxHashMap;
use sparq_core::dict::{Id, TermParts};
use sparq_core::Graph;
use std::collections::BTreeMap;

/// BM25 term-frequency saturation.
const K1: f32 = 1.2;
/// BM25 document-length normalisation strength.
const B: f32 = 0.75;

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// Opt-in positional postings: token -> doc id -> the token's 0-based offsets
/// in that doc, ascending (see [`TextIndex::positions`]). [OPUS-4.8]
type Positions = BTreeMap<Box<str>, FxHashMap<Id, Vec<u32>>>;

/// One search hit: the matching literal's dictionary id and its BM25 score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// The matching literal's dictionary term id (its document id).
    pub id: Id,
    /// BM25 relevance (higher is better; comparable within one query only).
    pub score: f32,
}

/// A BM25 inverted index over the string literals of a sparq [`Graph`]'s
/// dictionary (see the module docs for design and maintenance).
///
/// `PartialEq` compares the full index state (postings, document lengths, and
/// — when enabled — positions) — the differential incremental-vs-rebuilt test
/// is `assert_eq!` over it. Two indexes differing only in whether positions are
/// recorded are NOT equal.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextIndex {
    /// token -> posting list, sorted by doc id; tf is the token's count in the doc.
    postings: BTreeMap<Box<str>, Vec<(Id, u32)>>,
    /// doc id -> document length in tokens (every indexed literal, even empty).
    docs: FxHashMap<Id, u32>,
    /// Sum of all document lengths (for the BM25 average).
    total_tokens: u64,
    /// Opt-in positional postings (`None` = the cheap 8-B default; see the
    /// module docs). When `Some`, maps token -> doc id -> the token's 0-based
    /// offsets in that doc, ascending. Only [`build_with_positions`] /
    /// [`with_positions`] turn this on; it powers [`phrase`](Self::phrase).
    positions: Option<Positions>,
    /// Generation marker for the rebuild-on-boot + reconcile contract: the
    /// graph dictionary length (interned-term count) this index has indexed up
    /// to — every string literal with a dict id `<= indexed_dict_len` is
    /// accounted for. `0` for an empty/default index. Advanced by every
    /// [`build`](Self::build), [`reconcile`](Self::reconcile), and
    /// [`apply_delta`](Self::apply_delta) to the graph's current `dict.len()`.
    /// Part of the index state, so a rebuilt and an incrementally-maintained
    /// index over the same graph compare equal on it. [OPUS-4.8]
    indexed_dict_len: usize,
    /// The tokenization strategy this index was built with — used for BOTH
    /// indexing and querying so the two always agree. The default
    /// [`Analyzer::Unicode`] reproduces the historical behaviour exactly;
    /// [`Analyzer::CjkNgram`] is the opt-in CJK character-bigram analyzer. Part
    /// of the index state, so two indexes built with different analyzers are not
    /// equal. [OPUS-4.8] sq-m3ln
    analyzer: Analyzer,
}

/// The lexical value of a string literal (plain `xsd:string` or
/// language-tagged), `None` for every other term kind.
fn text_value<'a>(parts: &TermParts<'a>) -> Option<&'a str> {
    match parts {
        TermParts::Lit { value, datatype, lang } => {
            (lang.is_some() || *datatype == XSD_STRING).then_some(*value)
        }
        _ => None,
    }
}

impl TextIndex {
    /// Builds the index from the graph's dictionary: one scan over all real
    /// term ids, indexing every string literal (parallel-sharded under the
    /// `parallel` feature; the result is identical to the serial build). A
    /// graph with no string literals yields an empty (but usable) index.
    ///
    /// Positions are NOT recorded — this is the cheap 8-B-per-posting default;
    /// use [`build_with_positions`](Self::build_with_positions) for phrase
    /// search.
    pub fn build(graph: &Graph) -> TextIndex {
        Self::build_with(graph, false, Analyzer::Unicode)
    }

    /// Like [`build`](Self::build) but with an explicit [`Analyzer`] — pass
    /// [`Analyzer::CjkNgram`] for the opt-in CJK character-bigram analyzer (a
    /// multi-char CJK term then matches as the AND of *its* bigrams, not of its
    /// individual ideographs). The index records its analyzer, so
    /// [`search`](Self::search) and friends tokenize the query the same way.
    /// No positions; combine with positions via
    /// [`build_with_positions_analyzer`](Self::build_with_positions_analyzer).
    /// [OPUS-4.8] sq-m3ln
    pub fn build_with_analyzer(graph: &Graph, analyzer: Analyzer) -> TextIndex {
        Self::build_with(graph, false, analyzer)
    }

    /// The position-enabled counterpart of
    /// [`build_with_analyzer`](Self::build_with_analyzer): records token offsets
    /// (so [`phrase`](Self::phrase) / [`phrase_near`](Self::phrase_near) work)
    /// AND uses `analyzer` for tokenization. With [`Analyzer::CjkNgram`] a phrase
    /// is over the bigram stream, so `phrase("東京都")` requires the bigrams
    /// `東京` then `京都` consecutive — exactly the original ideograph adjacency.
    /// [OPUS-4.8] sq-m3ln
    pub fn build_with_positions_analyzer(graph: &Graph, analyzer: Analyzer) -> TextIndex {
        Self::build_with(graph, true, analyzer)
    }

    /// An empty index that records token positions and uses `analyzer`, for the
    /// incremental/delta-fed case (the [`with_positions`](Self::with_positions)
    /// counterpart with an explicit analyzer). [OPUS-4.8] sq-m3ln
    pub fn with_positions_analyzer(analyzer: Analyzer) -> TextIndex {
        TextIndex { positions: Some(BTreeMap::new()), analyzer, ..Default::default() }
    }

    /// An empty index that uses `analyzer` (the `TextIndex::default()`
    /// counterpart with an explicit analyzer), for the delta-fed case without
    /// positions. [OPUS-4.8] sq-m3ln
    pub fn with_analyzer(analyzer: Analyzer) -> TextIndex {
        TextIndex { analyzer, ..Default::default() }
    }

    /// The [`Analyzer`] this index was built/seeded with. [OPUS-4.8] sq-m3ln
    pub fn analyzer(&self) -> Analyzer {
        self.analyzer
    }

    /// Like [`build`](Self::build) but additionally records token positions, so
    /// [`phrase`](Self::phrase) can answer ordered-adjacency queries. Costs an
    /// extra positional map (`u32` offset per indexed token occurrence) on top
    /// of the BM25 tables; reach for it only when phrase search is needed.
    /// [OPUS-4.8]
    pub fn build_with_positions(graph: &Graph) -> TextIndex {
        Self::build_with(graph, true, Analyzer::Unicode)
    }

    /// An empty index that records token positions as documents are added
    /// (via [`apply_delta`](Self::apply_delta)) — the position-enabled
    /// counterpart of `TextIndex::default()`, for the incremental/delta-fed
    /// case. [OPUS-4.8]
    pub fn with_positions() -> TextIndex {
        TextIndex { positions: Some(BTreeMap::new()), ..Default::default() }
    }

    // Note: `Analyzer::default()` is `Unicode`, so `TextIndex::default()` and
    // `with_positions()` keep the historical analyzer. [OPUS-4.8] sq-m3ln

    /// Whether this index records token positions (i.e. supports
    /// [`phrase`](Self::phrase)). [OPUS-4.8]
    pub fn has_positions(&self) -> bool {
        self.positions.is_some()
    }

    fn build_with(graph: &Graph, positions: bool, analyzer: Analyzer) -> TextIndex {
        let mut idx = Self::build_docs(graph, positions, analyzer);
        // Pin the generation marker to the dictionary we just scanned in full.
        idx.indexed_dict_len = graph.dict.len();
        idx
    }

    fn build_docs(graph: &Graph, positions: bool, analyzer: Analyzer) -> TextIndex {
        #[cfg(feature = "parallel")]
        {
            // Shard the id range; per-shard sub-indexes concatenate cleanly
            // because shard id ranges are disjoint and increasing (posting
            // lists — and per-token position maps — stay keyed by doc id).
            const MIN_PARALLEL_TERMS: usize = 4096;
            let n = graph.dict.len();
            if n >= MIN_PARALLEL_TERMS {
                use rayon::prelude::*;
                let shards = rayon::current_num_threads().max(1) * 4;
                let chunk = n.div_ceil(shards);
                let parts: Vec<TextIndex> = (0..shards)
                    .into_par_iter()
                    .map(|s| {
                        let mut idx = Self::new(positions, analyzer);
                        for i in (s * chunk)..((s + 1) * chunk).min(n) {
                            let id = (i + 1) as Id;
                            if let Some(text) = text_value(&graph.dict.term_parts(id)) {
                                idx.add_doc(id, text);
                            }
                        }
                        idx
                    })
                    .collect();
                let mut out = Self::new(positions, analyzer);
                for part in parts {
                    out.append_shard(part);
                }
                return out;
            }
        }
        let mut idx = Self::new(positions, analyzer);
        for (id, parts) in graph.dict.iter() {
            if let Some(text) = text_value(&parts) {
                idx.add_doc(id, text);
            }
        }
        idx
    }

    /// An empty index with positions on/off and an explicit analyzer.
    fn new(positions: bool, analyzer: Analyzer) -> TextIndex {
        TextIndex {
            positions: positions.then(BTreeMap::new),
            analyzer,
            ..Default::default()
        }
    }

    /// Number of indexed literals (documents).
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Number of distinct tokens.
    pub fn token_count(&self) -> usize {
        self.postings.len()
    }

    /// A rough estimate of the index's heap footprint in bytes (for
    /// benchmarking): posting entries, token keys, the doc-length table, and —
    /// when enabled — the positional postings (one `u32` per token occurrence
    /// plus per-(token, doc) map overhead).
    pub fn heap_bytes(&self) -> usize {
        let postings: usize = self
            .postings
            .iter()
            .map(|(k, v)| k.len() + std::mem::size_of::<Box<str>>() + 24 + v.capacity() * 8)
            .sum();
        let positions: usize = self
            .positions
            .iter()
            .flat_map(|m| m.iter())
            .map(|(k, docs)| {
                k.len() + std::mem::size_of::<Box<str>>() + 24
                    + docs.values().map(|p| 16 + p.capacity() * 4).sum::<usize>()
            })
            .sum();
        postings + positions + self.docs.len() * 16
    }

    /// Tokenizes `text` and adds it as document `id`. Caller guarantees `id`
    /// is not already indexed.
    fn add_doc(&mut self, id: Id, text: &str) {
        let tokens = tokenize_with(text, self.analyzer);
        self.docs.insert(id, tokens.len() as u32);
        self.total_tokens += tokens.len() as u64;
        // Record this token's position the moment we enumerate it (positions
        // arrive ascending — `tokenize` yields in document order), before the
        // tf aggregation collapses duplicates. No-op when positions are off.
        if let Some(pos) = &mut self.positions {
            for (offset, token) in tokens.iter().enumerate() {
                pos.entry(token.as_str().into())
                    .or_default()
                    .entry(id)
                    .or_default()
                    .push(offset as u32);
            }
        }
        let mut tf: FxHashMap<String, u32> = FxHashMap::default();
        for t in tokens {
            *tf.entry(t).or_insert(0) += 1;
        }
        for (token, n) in tf {
            let rows = self.postings.entry(token.into_boxed_str()).or_default();
            match rows.last() {
                // Build and delta both feed increasing ids: append is the fast path.
                None => rows.push((id, n)),
                Some(&(last, _)) if last < id => rows.push((id, n)),
                _ => {
                    let i = rows.partition_point(|r| r.0 < id);
                    if rows.get(i).is_none_or(|r| r.0 != id) {
                        rows.insert(i, (id, n));
                    }
                }
            }
        }
    }

    /// Concatenates a shard whose doc ids are all GREATER than this index's
    /// (the parallel build's merge step).
    #[cfg(feature = "parallel")]
    fn append_shard(&mut self, shard: TextIndex) {
        for (token, rows) in shard.postings {
            self.postings.entry(token).or_default().extend(rows);
        }
        // Merge per-token position maps the same way: shard doc ids are
        // disjoint and greater, so the per-token doc maps simply union (no key
        // collisions). Both sides agree on whether positions are on.
        if let (Some(into), Some(from)) = (&mut self.positions, shard.positions) {
            for (token, docs) in from {
                into.entry(token).or_default().extend(docs);
            }
        }
        self.docs.extend(shard.docs);
        self.total_tokens += shard.total_tokens;
    }

    // ---- Incremental maintenance ---------------------------------------------------

    /// Mirrors a [`Graph::apply_delta`] batch into the index: call with the
    /// SAME graph (after its `apply_delta`) and the same insert/delete
    /// batches. Inserted triples whose object is a not-yet-indexed string
    /// literal are indexed; everything else is ignored, so it is safe (just a
    /// no-op) to forward every update batch. Deletions are intentionally not
    /// mirrored — the dictionary retains deleted terms, so this keeps the
    /// index EXACTLY equal to a fresh [`build`](Self::build) (see the module
    /// docs; orphaned literal ids join to zero rows downstream).
    pub fn apply_delta(&mut self, graph: &Graph, inserts: &[[Term; 3]], deletes: &[[Term; 3]]) {
        let _ = deletes; // Documented no-op: rebuild-equality + join semantics.
        for [_, _, o] in inserts {
            let Term::Literal(lit) = o else { continue };
            if lit.language().is_none() && lit.datatype().as_str() != XSD_STRING {
                continue;
            }
            // The literal was interned by the graph's own apply_delta.
            let Some(id) = graph.id_of(o) else { continue };
            if !self.docs.contains_key(&id) {
                self.add_doc(id, lit.value());
            }
        }
        // Advance the generation marker to the post-delta dictionary, exactly as
        // a fresh `build` would — the batch may have interned more terms than it
        // indexed (subject/predicate IRIs, non-text objects), so pin to the
        // graph's current length, not just the literals seen here. [OPUS-4.8]
        self.indexed_dict_len = graph.dict.len();
    }

    // ---- Rebuild-on-boot + reconcile contract --------------------------------------
    // [OPUS-4.8] sq-oddt. See the module docs for the full contract.

    /// The graph dictionary length (count of interned terms) this index has
    /// indexed up to — the generation marker the reconcile contract pins to.
    /// `0` for an empty/default index; advanced by every [`build`](Self::build),
    /// [`reconcile`](Self::reconcile), and [`apply_delta`](Self::apply_delta) to
    /// the graph's `dict.len()` at that point. Dictionary ids are dense
    /// (`1..=len`), so this is exactly "every string literal with id ≤ this is
    /// accounted for". [OPUS-4.8]
    pub fn indexed_dict_len(&self) -> usize {
        self.indexed_dict_len
    }

    /// Whether this index reflects every term the `graph`'s dictionary now holds
    /// — i.e. it has indexed up to the graph's current length. A cheap `usize`
    /// compare; `false` means [`reconcile`](Self::reconcile) (dict grew) or a
    /// full [`build`](Self::build) (dict shrank — see
    /// [`needs_rebuild`](Self::needs_rebuild)) is required to make it current.
    ///
    /// Pass the SAME graph the index was built/reconciled against. Cross-graph
    /// or post-[`Graph::compact`](sparq_core::Graph::compact) comparisons are
    /// only meaningful through [`needs_rebuild`](Self::needs_rebuild). [OPUS-4.8]
    pub fn is_consistent_with(&self, graph: &Graph) -> bool {
        self.indexed_dict_len == graph.dict.len()
    }

    /// Whether the index MUST be discarded and rebuilt from scratch (rather than
    /// reconciled): the graph's dictionary is SHORTER than what the index
    /// indexed. Within one live [`Graph`] the dictionary length only grows (even
    /// across [`Graph::compact`](sparq_core::Graph::compact), which folds the
    /// fork structure WITHOUT renumbering ids), so this returns `false` there. It
    /// is the CROSS-process signal: reopening a durably-compacted base whose
    /// persisted store dropped orphaned terms and renumbered yields a shorter
    /// dictionary, where the index's dense id→document mapping no longer lines
    /// up. Reconcile is append-only and cannot repair that — call
    /// [`build`](Self::build) (or
    /// [`build_with_positions`](Self::build_with_positions)) afresh. [OPUS-4.8]
    pub fn needs_rebuild(&self, graph: &Graph) -> bool {
        graph.dict.len() < self.indexed_dict_len
    }

    /// The warm fast-path of the rebuild-on-boot + reconcile contract: bring this
    /// index up to date with `graph` by indexing ONLY the dictionary terms
    /// appended since the last build/reconcile/delta — `O(new terms)`, not a full
    /// rescan. Use it when a [`TextIndex`] outlives a single update and you do
    /// not have the insert batches to hand for
    /// [`apply_delta`](Self::apply_delta) (e.g. an index shared across requests
    /// against a growing graph). After it returns the index equals a fresh
    /// [`build`](Self::build) over `graph` (the same property the differential
    /// test pins for [`apply_delta`](Self::apply_delta)).
    ///
    /// Returns the number of newly indexed documents (string literals); `0` when
    /// already current. Reconcile preserves this index's positions mode: a
    /// positions-enabled index keeps recording offsets for the new terms.
    ///
    /// Dictionary ids being dense and monotonic, "appended since" is exactly the
    /// id range `(indexed_dict_len, dict.len()]`.
    ///
    /// # Panics
    ///
    /// Panics if [`needs_rebuild`](Self::needs_rebuild) holds (the dictionary
    /// shrank, invalidating the id→document mapping): reconcile is an append-only
    /// operation and cannot repair this — call [`build`](Self::build) instead.
    /// This cannot arise from in-process
    /// [`Graph::compact`](sparq_core::Graph::compact) (which keeps ids); guard
    /// with [`needs_rebuild`](Self::needs_rebuild) when the index might be
    /// applied to a reopened/durably-recompacted graph. [OPUS-4.8]
    pub fn reconcile(&mut self, graph: &Graph) -> usize {
        let now = graph.dict.len();
        assert!(
            now >= self.indexed_dict_len,
            "TextIndex::reconcile cannot repair a shrunken dictionary (the persisted base was recompacted + renumbered); rebuild with TextIndex::build"
        );
        let mut added = 0;
        // Scan only the appended tail: ids are 1-based and dense, so the new
        // terms are exactly (indexed_dict_len, now].
        for i in self.indexed_dict_len..now {
            let id = (i + 1) as Id;
            if let Some(text) = text_value(&graph.dict.term_parts(id)) {
                // The build/apply_delta path never indexes a term twice; the
                // tail is fresh, but guard for robustness against an unexpected
                // overlap (keeps reconcile idempotent).
                if !self.docs.contains_key(&id) {
                    self.add_doc(id, text);
                    added += 1;
                }
            }
        }
        self.indexed_dict_len = now;
        added
    }

    // ---- Queries --------------------------------------------------------------------

    /// The posting list for one query token: `(document frequency, rows)`.
    /// A prefix token unions every expansion in the `BTreeMap` range
    /// (tf summed, df = union size) — one pseudo-term for scoring.
    fn resolve(&self, t: &QueryToken) -> Vec<(Id, u32)> {
        if !t.prefix {
            return self.postings.get(t.token.as_str()).cloned().unwrap_or_default();
        }
        let mut merged: FxHashMap<Id, u32> = FxHashMap::default();
        for (token, rows) in self.postings.range(t.token.clone().into_boxed_str()..) {
            if !token.starts_with(t.token.as_str()) {
                break;
            }
            for &(id, tf) in rows {
                *merged.entry(id).or_insert(0) += tf;
            }
        }
        let mut rows: Vec<(Id, u32)> = merged.into_iter().collect();
        rows.sort_unstable_by_key(|r| r.0);
        rows
    }

    /// Literals containing EVERY query token (`*`-suffixed tokens match as
    /// prefixes), BM25-ranked best-first (ties by id). An empty query — or
    /// any token with no matches — returns no hits.
    pub fn search(&self, query: &str) -> Vec<Hit> {
        self.run(query, true)
    }

    /// Literals containing AT LEAST ONE query token, BM25-ranked best-first
    /// (scores sum over the tokens present).
    pub fn search_any(&self, query: &str) -> Vec<Hit> {
        self.run(query, false)
    }

    /// Phrase query: literals where the query's tokens appear ADJACENT
    /// (consecutive positions) and IN ORDER. `phrase("foo bar")` matches a
    /// document only where `foo` is immediately followed by `bar`; the same
    /// tokens at non-adjacent positions do not match, and order is significant
    /// (`"foo bar"` ≠ `"bar foo"`). Returns the matching literal ids ascending
    /// (a phrase match is boolean adjacency, not a BM25 ranking). [OPUS-4.8]
    ///
    /// The query is analyzed by the SAME pipeline as the indexed text (UAX #29
    /// segmentation + Unicode casefolding, via [`tokenize`](crate::tokenize::tokenize)), so the phrase
    /// tokens match exactly how the literals were tokenized. A trailing `*` is
    /// NOT a prefix marker here — it is segmentation punctuation, as in any
    /// document text. An empty phrase (no word tokens) matches nothing.
    ///
    /// # Panics
    ///
    /// Panics if the index was built without positions; build it with
    /// [`build_with_positions`](Self::build_with_positions) (or seed it with
    /// [`with_positions`](Self::with_positions)). Guard with
    /// [`has_positions`](Self::has_positions) if unsure.
    pub fn phrase(&self, query: &str) -> Vec<Id> {
        let positions = self
            .positions
            .as_ref()
            .expect("phrase() requires a positional index: build with TextIndex::build_with_positions");
        let tokens = tokenize_with(query, self.analyzer);
        if tokens.is_empty() {
            return Vec::new();
        }

        // Per-token doc -> sorted positions. A missing token means an
        // unsatisfiable phrase; the rarest token (fewest docs) drives the scan.
        let mut lists: Vec<&FxHashMap<Id, Vec<u32>>> = Vec::with_capacity(tokens.len());
        for t in &tokens {
            match positions.get(t.as_str()) {
                Some(docs) => lists.push(docs),
                None => return Vec::new(),
            }
        }
        let (driver_i, driver) = lists
            .iter()
            .enumerate()
            .min_by_key(|(_, docs)| docs.len())
            .map(|(i, docs)| (i, *docs))
            .unwrap();

        let mut hits: Vec<Id> = driver
            .keys()
            .copied()
            .filter(|&id| Self::phrase_in_doc(&lists, driver_i, id))
            .collect();
        hits.sort_unstable();
        hits
    }

    /// Does the document `id` contain the token sequence at consecutive
    /// positions, in order? `lists[k]` is token k's doc->positions map;
    /// `driver_i` is the token whose docs we are iterating (already known to
    /// contain `id`). A single-token phrase is satisfied by presence alone.
    fn phrase_in_doc(lists: &[&FxHashMap<Id, Vec<u32>>], driver_i: usize, id: Id) -> bool {
        // Every other token must also occur in this doc.
        for (k, docs) in lists.iter().enumerate() {
            if k != driver_i && !docs.contains_key(&id) {
                return false;
            }
        }
        // For each start position of token 0, require token k at start + k.
        let first = &lists[0][&id];
        first.iter().any(|&start| {
            (1..lists.len() as u32).all(|k| {
                let want = start + k;
                lists[k as usize][&id].binary_search(&want).is_ok()
            })
        })
    }

    /// Proximity ("slop") phrase query: literals where the query's tokens occur
    /// IN ORDER within a bounded total gap, RANKED by how tightly they cluster.
    /// [OPUS-4.8]
    ///
    /// `slop` is the maximum *extra* span the tokens may spread over their
    /// tightest possible (adjacent) packing — the same notion of slop Lucene's
    /// `PhraseQuery` uses. For a candidate alignment placing token `k` at
    /// position `p_k` (with `p_0 < p_1 < … < p_{n-1}`, i.e. strictly in order),
    /// its gap is `(p_{n-1} − p_0) − (n − 1)`: zero when the tokens are exactly
    /// adjacent, growing by one for every position of separation introduced.
    /// A document matches when SOME in-order alignment has gap ≤ `slop`; its
    /// score is `1 / (1 + g)` where `g` is the *smallest* gap any alignment in
    /// that document achieves (so 1.0 for an adjacent occurrence, decreasing
    /// monotonically as the best occurrence loosens).
    ///
    /// Relationship to [`phrase`](Self::phrase): `phrase_near(q, 0)` returns
    /// exactly [`phrase`](Self::phrase)`(q)`'s ids — gap 0 is adjacency — each
    /// at score 1.0; and the id SET grows monotonically with `slop` (a larger
    /// slop only ever admits more documents). Order remains significant:
    /// `"foo bar"` and `"bar foo"` are different queries (the reverse never
    /// matches, no matter how large the slop). Results are returned best-first
    /// (highest score), ties broken by ascending id — the same ordering
    /// [`search`](Self::search) uses.
    ///
    /// The query is analyzed by the SAME pipeline as the indexed text (UAX #29
    /// segmentation + Unicode casefolding, via [`tokenize`](crate::tokenize::tokenize)); a trailing `*` is
    /// NOT a prefix marker (it is segmentation punctuation). An empty phrase (no
    /// word tokens) returns no hits. A single-token phrase is presence (gap 0,
    /// score 1.0) for any `slop`.
    ///
    /// # Panics
    ///
    /// Panics if the index was built without positions; build it with
    /// [`build_with_positions`](Self::build_with_positions) (or seed it with
    /// [`with_positions`](Self::with_positions)). Guard with
    /// [`has_positions`](Self::has_positions) if unsure.
    pub fn phrase_near(&self, query: &str, slop: u32) -> Vec<Hit> {
        let positions = self.positions.as_ref().expect(
            "phrase_near() requires a positional index: build with TextIndex::build_with_positions",
        );
        let tokens = tokenize_with(query, self.analyzer);
        if tokens.is_empty() {
            return Vec::new();
        }

        // Per-token doc -> sorted positions. A missing token means an
        // unsatisfiable phrase; the rarest token (fewest docs) drives the scan.
        let mut lists: Vec<&FxHashMap<Id, Vec<u32>>> = Vec::with_capacity(tokens.len());
        for t in &tokens {
            match positions.get(t.as_str()) {
                Some(docs) => lists.push(docs),
                None => return Vec::new(),
            }
        }
        let (driver_i, driver) = lists
            .iter()
            .enumerate()
            .min_by_key(|(_, docs)| docs.len())
            .map(|(i, docs)| (i, *docs))
            .unwrap();

        let mut hits: Vec<Hit> = driver
            .keys()
            .copied()
            .filter_map(|id| {
                Self::min_phrase_gap(&lists, driver_i, id)
                    .filter(|&g| g <= slop)
                    .map(|g| Hit { id, score: 1.0 / (1.0 + g as f32) })
            })
            .collect();
        // Best-first (tighter proximity = higher score), ties by ascending id —
        // the ordering `search` uses.
        hits.sort_unstable_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
        hits
    }

    /// The smallest in-order phrase gap achievable in document `id`, or `None`
    /// if the tokens never occur in order there. The gap of an alignment placing
    /// token `k` at `p_k` (strictly increasing) is `(p_{n-1} − p_0) − (n − 1)`.
    /// `lists[k]` is token k's doc->positions map; `driver_i` is the token whose
    /// docs we are iterating (already known to contain `id`). [OPUS-4.8]
    fn min_phrase_gap(lists: &[&FxHashMap<Id, Vec<u32>>], driver_i: usize, id: Id) -> Option<u32> {
        // Every token must occur in this doc for any alignment to exist.
        for (k, docs) in lists.iter().enumerate() {
            if k != driver_i && !docs.contains_key(&id) {
                return None;
            }
        }
        let n = lists.len() as u32;
        // Single token: presence is a perfect (gap-0) match.
        if n == 1 {
            return Some(0);
        }
        // For each placement of token 0, greedily place each later token at the
        // EARLIEST position strictly after the previous token's — that minimises
        // the span (`last − start`), hence the gap, for this start. The best gap
        // over all starts is the document's minimum. Positions are ascending, so
        // `partition_point` is a binary search for "strictly greater than prev".
        let first = &lists[0][&id];
        first.iter().filter_map(|&start| {
            let mut prev = start;
            for docs in &lists[1..] {
                let ps = &docs[&id];
                let i = ps.partition_point(|&p| p <= prev);
                prev = *ps.get(i)?; // no in-order position for this token after prev
            }
            Some((prev - start) - (n - 1))
        }).min()
    }

    fn run(&self, query: &str, all: bool) -> Vec<Hit> {
        let mut qtokens = tokenize_query_with(query, self.analyzer);
        // A duplicated token must not double-score ("fox fox" == "fox").
        qtokens.dedup();
        let mut seen: Vec<&QueryToken> = Vec::new();
        qtokens.iter().for_each(|t| {
            if !seen.contains(&t) {
                seen.push(t);
            }
        });
        if seen.is_empty() || self.docs.is_empty() {
            return Vec::new();
        }

        let mut lists: Vec<Vec<(Id, u32)>> = Vec::with_capacity(seen.len());
        for t in &seen {
            let rows = self.resolve(t);
            if rows.is_empty() {
                if all {
                    return Vec::new(); // AND with an absent term matches nothing.
                }
                continue;
            }
            lists.push(rows);
        }
        if lists.is_empty() {
            return Vec::new();
        }

        let n = self.docs.len() as f32;
        let avgdl = (self.total_tokens as f32 / n).max(f32::MIN_POSITIVE);
        // doc id -> (accumulated score, number of query terms present).
        let mut acc: FxHashMap<Id, (f32, u32)> = FxHashMap::default();
        for rows in &lists {
            let df = rows.len() as f32;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            for &(id, tf) in rows {
                let dl = self.docs[&id] as f32;
                let tf = tf as f32;
                let s = idf * tf * (K1 + 1.0) / (tf + K1 * (1.0 - B + B * dl / avgdl));
                let e = acc.entry(id).or_insert((0.0, 0));
                e.0 += s;
                e.1 += 1;
            }
        }
        let need = lists.len() as u32;
        let mut hits: Vec<Hit> = acc
            .into_iter()
            .filter(|(_, (_, cnt))| !all || *cnt == need)
            .map(|(id, (score, _))| Hit { id, score })
            .collect();
        hits.sort_unstable_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
        hits
    }
}
