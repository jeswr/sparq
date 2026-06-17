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
//! permutations yields zero rows. After `Graph::compact` (ids reassigned),
//! rebuild the index.

use crate::tokenize::{tokenize, tokenize_query, QueryToken};
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
        Self::build_with(graph, false)
    }

    /// Like [`build`](Self::build) but additionally records token positions, so
    /// [`phrase`](Self::phrase) can answer ordered-adjacency queries. Costs an
    /// extra positional map (`u32` offset per indexed token occurrence) on top
    /// of the BM25 tables; reach for it only when phrase search is needed.
    /// [OPUS-4.8]
    pub fn build_with_positions(graph: &Graph) -> TextIndex {
        Self::build_with(graph, true)
    }

    /// An empty index that records token positions as documents are added
    /// (via [`apply_delta`](Self::apply_delta)) — the position-enabled
    /// counterpart of `TextIndex::default()`, for the incremental/delta-fed
    /// case. [OPUS-4.8]
    pub fn with_positions() -> TextIndex {
        TextIndex { positions: Some(BTreeMap::new()), ..Default::default() }
    }

    /// Whether this index records token positions (i.e. supports
    /// [`phrase`](Self::phrase)). [OPUS-4.8]
    pub fn has_positions(&self) -> bool {
        self.positions.is_some()
    }

    fn build_with(graph: &Graph, positions: bool) -> TextIndex {
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
                        let mut idx = Self::new(positions);
                        for i in (s * chunk)..((s + 1) * chunk).min(n) {
                            let id = (i + 1) as Id;
                            if let Some(text) = text_value(&graph.dict.term_parts(id)) {
                                idx.add_doc(id, text);
                            }
                        }
                        idx
                    })
                    .collect();
                let mut out = Self::new(positions);
                for part in parts {
                    out.append_shard(part);
                }
                return out;
            }
        }
        let mut idx = Self::new(positions);
        for (id, parts) in graph.dict.iter() {
            if let Some(text) = text_value(&parts) {
                idx.add_doc(id, text);
            }
        }
        idx
    }

    /// An empty index with positions on/off.
    fn new(positions: bool) -> TextIndex {
        TextIndex {
            positions: positions.then(BTreeMap::new),
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
        let tokens = tokenize(text);
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
    /// segmentation + Unicode casefolding, via [`tokenize`]), so the phrase
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
        let tokens = tokenize(query);
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
    /// segmentation + Unicode casefolding, via [`tokenize`]); a trailing `*` is
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
        let tokens = tokenize(query);
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
        let mut qtokens = tokenize_query(query);
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
