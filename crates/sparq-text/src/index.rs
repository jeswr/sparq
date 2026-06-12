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
//! Positions are NOT stored, so phrase queries are future work (documented in
//! the crate README) — postings stay at 8 bytes per (token, doc) pair.
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
/// `PartialEq` compares the full index state (postings, document lengths) —
/// the differential incremental-vs-rebuilt test is `assert_eq!` over it.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextIndex {
    /// token -> posting list, sorted by doc id; tf is the token's count in the doc.
    postings: BTreeMap<Box<str>, Vec<(Id, u32)>>,
    /// doc id -> document length in tokens (every indexed literal, even empty).
    docs: FxHashMap<Id, u32>,
    /// Sum of all document lengths (for the BM25 average).
    total_tokens: u64,
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
    pub fn build(graph: &Graph) -> TextIndex {
        #[cfg(feature = "parallel")]
        {
            // Shard the id range; per-shard sub-indexes concatenate cleanly
            // because shard id ranges are disjoint and increasing (posting
            // lists stay sorted by doc id).
            const MIN_PARALLEL_TERMS: usize = 4096;
            let n = graph.dict.len();
            if n >= MIN_PARALLEL_TERMS {
                use rayon::prelude::*;
                let shards = rayon::current_num_threads().max(1) * 4;
                let chunk = n.div_ceil(shards);
                let parts: Vec<TextIndex> = (0..shards)
                    .into_par_iter()
                    .map(|s| {
                        let mut idx = TextIndex::default();
                        for i in (s * chunk)..((s + 1) * chunk).min(n) {
                            let id = (i + 1) as Id;
                            if let Some(text) = text_value(&graph.dict.term_parts(id)) {
                                idx.add_doc(id, text);
                            }
                        }
                        idx
                    })
                    .collect();
                let mut out = TextIndex::default();
                for part in parts {
                    out.append_shard(part);
                }
                return out;
            }
        }
        let mut idx = TextIndex::default();
        for (id, parts) in graph.dict.iter() {
            if let Some(text) = text_value(&parts) {
                idx.add_doc(id, text);
            }
        }
        idx
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
    /// benchmarking): posting entries, token keys, and the doc-length table.
    pub fn heap_bytes(&self) -> usize {
        let postings: usize = self
            .postings
            .iter()
            .map(|(k, v)| k.len() + std::mem::size_of::<Box<str>>() + 24 + v.capacity() * 8)
            .sum();
        postings + self.docs.len() * 16
    }

    /// Tokenizes `text` and adds it as document `id`. Caller guarantees `id`
    /// is not already indexed.
    fn add_doc(&mut self, id: Id, text: &str) {
        let tokens = tokenize(text);
        self.docs.insert(id, tokens.len() as u32);
        self.total_tokens += tokens.len() as u64;
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
