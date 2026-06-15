//! [OPUS-4.8] (sq-32i5) A cheap, strongly-distinguishing **graph fingerprint** embedded in
//! the `.spqv` / `.spqg` headers so a stale store can never silently return WRONG results.
//!
//! # The foot-gun this closes
//!
//! A [`VectorStore`](crate::store::VectorStore) (`.spqv`) and a
//! [`DiskAnnIndex`](crate::diskann::DiskAnnIndex) (`.spqg`) are build-once-immutable and keyed by
//! **dictionary term id**. They carry no link to the graph they were built against. But a graph
//! mutation/rebuild can SHIFT dict ids (interning order changes), and a query entry point such as
//! [`nearest_term`](crate::diskann::DiskAnnIndex::nearest_term) resolves the query term through the
//! *caller-provided* graph's dictionary (`graph.id_of(term)`) and then looks that id up in the
//! store. If the store was built against a different generation of the graph, the id resolves to a
//! DIFFERENT term's vector — and the search returns plausible-looking but WRONG neighbours, with no
//! error. This module makes that mismatch a hard, descriptive error instead.
//!
//! # What is fingerprinted (the exact inputs — keep this comment in sync with [`Fingerprint::of`])
//!
//! Three fields, all cheap to compute at build time and on open:
//!   1. **`dict_len`** — `graph.dict.len()`, the number of real dictionary terms. O(1).
//!   2. **`triple_count`** — `graph.len()`, the number of triples in the default graph. O(1).
//!   3. **`content_hash`** — a 64-bit hash that folds, **in ascending id order**, every dictionary
//!      term's content (its [`TermParts`](sparq_core::dict::TermParts) — IRI prefix+suffix, literal
//!      value+datatype+lang, blank label, or a triple term's three child ids). This directly
//!      captures the **id → term binding**: if any id maps to a different term (the exact failure
//!      mode of a dict-id shift), the sequence of folded writes changes and the hash changes. The
//!      two scalars are folded in first so a fingerprint that agrees on the hash but differs on
//!      length/count is still rejected. `FxHasher` is used because it is **deterministic with no
//!      seed** (the same property the core dictionary relies on for its own content hashing), so a
//!      fingerprint computed at build time on one machine matches one recomputed at open time on
//!      another. [OPUS-4.8] (sq-32i5) Every hashed input is **fixed-width** — lengths/counts as
//!      `u64`, ids as `u32`, tags as `u8`, never `usize` — so the folded byte sequence (and thus
//!      the hash) is identical on 32-bit wasm32 and 64-bit native; the cross-machine guarantee
//!      holds by construction. The cost is O(dict_len) over already-resident term parts — a small
//!      fraction of the embed/build work that produced the store in the first place.
//!
//! `content_hash` is a **collision-resistance-irrelevant integrity check**, not a security MAC: it
//! exists to catch accidental staleness, not to defend against an adversary crafting a colliding
//! graph. dict_len + triple_count are folded alongside it precisely so the common shift cases
//! (terms added/removed, triples added/removed) are caught structurally even before the hash.

use sparq_core::dict::TermParts;
use sparq_core::Graph;
use std::hash::Hasher;

/// Number of bytes a [`Fingerprint`] occupies in a file header: three little-endian `u64`s.
pub const FINGERPRINT_LEN: usize = 24;

/// A graph fingerprint: dictionary length, triple count, and a content hash over the
/// id-ordered dictionary terms. See the module docs for the exact inputs and rationale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    /// `graph.dict.len()` at build time.
    pub dict_len: u64,
    /// `graph.len()` (default-graph triple count) at build time.
    pub triple_count: u64,
    /// 64-bit content hash over the id-ordered dictionary term parts (see module docs).
    pub content_hash: u64,
}

impl Fingerprint {
    /// Computes the fingerprint of `graph`. Cheap: two O(1) reads plus one O(dict_len) pass over
    /// the already-resident term parts. The exact inputs are documented at the module level — keep
    /// the two in sync.
    pub fn of(graph: &Graph) -> Fingerprint {
        let dict_len = graph.dict.len() as u64;
        let triple_count = graph.len() as u64;
        // FxHasher: deterministic, no seed — so a fingerprint computed at build time matches one
        // recomputed at open time (possibly on another machine). The dict module relies on the
        // same property for its own content hashing.
        let mut h = rustc_hash::FxHasher::default();
        // Fold the scalars first so a hash collision alone cannot mask a length/count change.
        h.write_u64(dict_len);
        h.write_u64(triple_count);
        for (id, parts) in graph.dict.iter() {
            // Bind the id explicitly: a permutation of the same term set (a pure dict-id shift)
            // must change the hash even if every individual term is still present somewhere.
            h.write_u32(id);
            fold_parts(&mut h, &parts);
        }
        Fingerprint {
            dict_len,
            triple_count,
            content_hash: h.finish(),
        }
    }

    /// Serializes to `FINGERPRINT_LEN` little-endian bytes (the on-disk header layout).
    pub fn to_bytes(self) -> [u8; FINGERPRINT_LEN] {
        let mut out = [0u8; FINGERPRINT_LEN];
        out[0..8].copy_from_slice(&self.dict_len.to_le_bytes());
        out[8..16].copy_from_slice(&self.triple_count.to_le_bytes());
        out[16..24].copy_from_slice(&self.content_hash.to_le_bytes());
        out
    }

    /// Parses a fingerprint from `FINGERPRINT_LEN` little-endian bytes. `bytes` must be at least
    /// [`FINGERPRINT_LEN`] long (the caller validated the header length first).
    pub fn from_bytes(bytes: &[u8]) -> Fingerprint {
        Fingerprint {
            dict_len: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            triple_count: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            content_hash: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        }
    }

    /// [OPUS-4.8] (sq-32i5) Parses a fingerprint from a current-format header block, returning
    /// `None` for an **all-zero block**. A v2 artifact finalized *without* binding a graph (no
    /// `with_fingerprint`) writes a zeroed block; decoding that to `Some(Fingerprint{0,0,0})` would
    /// make [`check_against`] report a stale-graph *mismatch* (a "DIFFERENT graph" error) instead of
    /// the accurate "carries no fingerprint / unverifiable" message the `store.rs`/`diskann.rs` docs
    /// promise. Treating all-zero as `None` aligns the behaviour with those docs. (An all-zero block
    /// is unreachable from a real [`Fingerprint::of`]: `content_hash` is the FxHash of at least the
    /// two folded scalars, so a genuine empty graph still has a non-zero hash; even in the
    /// astronomically unlikely event a real fingerprint hashed to all-zero, the only consequence is
    /// that it is reported as "unverifiable" rather than checked — fail-safe, never a false match.)
    pub fn from_bytes_opt(bytes: &[u8]) -> Option<Fingerprint> {
        if bytes[0..FINGERPRINT_LEN].iter().all(|&b| b == 0) {
            None
        } else {
            Some(Fingerprint::from_bytes(bytes))
        }
    }

    /// A human-readable one-line summary for error messages.
    fn describe(&self) -> String {
        format!(
            "dict_len={} triples={} hash={:#018x}",
            self.dict_len, self.triple_count, self.content_hash
        )
    }
}

/// Folds one term's content into the hasher. The byte/field sequence (and the leading tag) mirror
/// the discipline of `sparq_core::dict`'s own content hashing: a length-prefixed, tagged write per
/// variant, so distinct terms cannot alias by concatenation. We do not depend on the core's hash
/// VALUE (it is private); we only need a self-consistent fold that distinguishes terms here.
fn fold_parts(h: &mut rustc_hash::FxHasher, parts: &TermParts<'_>) {
    // [OPUS-4.8] (sq-32i5) All length prefixes are hashed as fixed-width `u64`, NOT `usize`:
    // `usize` is 32-bit on wasm32 and 64-bit on native, so `write_usize` would fold a different
    // byte sequence on each and the on-disk fingerprint would mismatch between a store built on
    // 64-bit native and the same graph opened on 32-bit wasm — silently breaking the documented
    // "build/open fingerprints match across machines" guarantee. `as u64` is lossless on both
    // (lengths fit in 32 bits) and produces an architecture-independent fingerprint by construction.
    match parts {
        TermParts::Iri { prefix, suffix } => {
            h.write_u8(0);
            h.write_u64(prefix.len() as u64);
            h.write(prefix.as_bytes());
            h.write(suffix.as_bytes());
        }
        TermParts::Lit {
            value,
            datatype,
            lang,
        } => {
            h.write_u8(1);
            h.write_u64(value.len() as u64);
            h.write(value.as_bytes());
            h.write_u64(datatype.len() as u64);
            h.write(datatype.as_bytes());
            match lang {
                Some(l) => {
                    h.write_u8(1);
                    h.write(l.as_bytes());
                }
                None => h.write_u8(0),
            }
        }
        TermParts::Blank(label) => {
            h.write_u8(2);
            h.write(label.as_bytes());
        }
        TermParts::Triple(ids) => {
            h.write_u8(3);
            for id in ids {
                h.write_u32(*id);
            }
        }
    }
}

/// The result of a stored-fingerprint vs. live-graph check, produced by a CHECKED open path.
/// `Ok(())` means the store is safe to query against the graph; `Err` carries a descriptive
/// message naming the mismatch so a stale store can never be queried silently.
pub type CheckResult = Result<(), String>;

/// [OPUS-4.8] (sq-32i5) Identifies which kind of artifact a [`check_against`] call is guarding, so
/// the error text is accurate at both call sites (a `.spqv` [`VectorStore`](crate::store::VectorStore)
/// and a `.spqg` [`DiskAnnIndex`](crate::diskann::DiskAnnIndex)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Artifact {
    /// A `.spqv` vector store.
    Store,
    /// A `.spqg` ANN index.
    Index,
}

impl Artifact {
    /// The noun used in error messages ("store" / "index").
    fn noun(self) -> &'static str {
        match self {
            Artifact::Store => "store",
            Artifact::Index => "index",
        }
    }
}

/// Compares a `stored` fingerprint (read from a header) against the live `graph`, returning a
/// descriptive `Err` on any mismatch. `stored` is `None` when the artifact carries no fingerprint —
/// either a pre-fingerprinting (version-1) file, or a current-format file finalized without a graph
/// binding (`with_fingerprint`); in both cases it cannot be certified, so the check fails with a
/// clear "unverified" message rather than silently passing. `artifact` selects the noun used in the
/// message (store vs. index) so it is accurate at both call sites; `origin` names the file.
pub fn check_against(
    stored: Option<Fingerprint>,
    graph: &Graph,
    artifact: Artifact,
    origin: &str,
) -> CheckResult {
    let kind = artifact.noun();
    let Some(stored) = stored else {
        return Err(format!(
            "{origin}: this {kind} carries no graph fingerprint (either a legacy file that predates \
             graph-fingerprinting, or one finalized without binding it to a graph) and so cannot be \
             verified against the graph; rebuild it with a graph binding to enable the staleness check"
        ));
    };
    let live = Fingerprint::of(graph);
    if stored == live {
        Ok(())
    } else {
        Err(format!(
            "{origin}: graph fingerprint mismatch — this {kind} was built against a DIFFERENT graph \
             (stored {}, this graph {}); querying it would silently return wrong results. Rebuild \
             the {kind} against the current graph.",
            stored.describe(),
            live.describe()
        ))
    }
}

#[cfg(test)]
mod tests {
    // [OPUS-4.8] (sq-32i5) fingerprint unit tests: discrimination, round-trip, and the
    // legacy (None) path. The store/diskann modules cover the end-to-end checked-open behaviour.
    use super::*;
    use sparq_core::Graph;

    fn graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").expect("load test turtle")
    }

    const A: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:carol .
    "#;
    // B has a different dictionary (different IRIs) AND a different triple count.
    const B: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:dave ex:likes ex:eve .
    "#;

    #[test]
    fn distinguishes_different_graphs() {
        let fa = Fingerprint::of(&graph(A));
        let fb = Fingerprint::of(&graph(B));
        assert_ne!(fa, fb, "distinct graphs must have distinct fingerprints");
    }

    #[test]
    fn stable_for_the_same_graph() {
        // Recomputed twice (different Graph instances of the same content) must agree, so a
        // build-time fingerprint matches an open-time one.
        assert_eq!(Fingerprint::of(&graph(A)), Fingerprint::of(&graph(A)));
    }

    #[test]
    fn detects_dict_id_shift_with_same_term_set() {
        // Same TERMS, different interning order → a pure dict-id shift. The id is folded into
        // the hash, so the fingerprint must still differ (the exact foot-gun sq-32i5 closes).
        let g1 = graph(
            r#"@prefix ex: <http://example.org/> .
               ex:aaa ex:p ex:bbb ."#,
        );
        let g2 = graph(
            r#"@prefix ex: <http://example.org/> .
               ex:bbb ex:p ex:aaa ."#,
        );
        // Same number of terms and triples...
        assert_eq!(g1.dict.len(), g2.dict.len());
        assert_eq!(g1.len(), g2.len());
        let f1 = Fingerprint::of(&g1);
        let f2 = Fingerprint::of(&g2);
        // ...but the id→term binding differs, so content_hash differs.
        assert_ne!(
            f1.content_hash, f2.content_hash,
            "a dict-id shift must change the hash"
        );
    }

    #[test]
    fn bytes_round_trip() {
        let f = Fingerprint::of(&graph(A));
        let back = Fingerprint::from_bytes(&f.to_bytes());
        assert_eq!(f, back);
    }

    #[test]
    fn check_against_matching_is_ok() {
        let f = Fingerprint::of(&graph(A));
        assert!(check_against(Some(f), &graph(A), Artifact::Store, "t").is_ok());
    }

    #[test]
    fn check_against_mismatch_is_descriptive_err() {
        let f = Fingerprint::of(&graph(A));
        let err = check_against(Some(f), &graph(B), Artifact::Store, "t")
            .expect_err("must reject a different graph");
        assert!(err.contains("fingerprint mismatch"), "err was: {err}");
        assert!(
            err.contains("wrong results"),
            "err must warn about wrong results: {err}"
        );
    }

    #[test]
    fn check_against_legacy_none_is_unverifiable_err() {
        let err = check_against(None, &graph(A), Artifact::Store, "t")
            .expect_err("legacy/unbound file must not pass");
        assert!(
            err.contains("carries no graph fingerprint"),
            "err was: {err}"
        );
    }

    #[test]
    fn check_against_error_text_names_the_artifact_kind() {
        // [OPUS-4.8] (sq-32i5) The same routine guards BOTH a `.spqv` store and a `.spqg` index;
        // the error noun must follow the artifact kind, not hard-code "store" at an index call site.
        let f = Fingerprint::of(&graph(A));

        // Mismatch path.
        let store_err = check_against(Some(f), &graph(B), Artifact::Store, "t")
            .expect_err("mismatch must error");
        assert!(
            store_err.contains("this store was built"),
            "err: {store_err}"
        );
        let index_err = check_against(Some(f), &graph(B), Artifact::Index, "t")
            .expect_err("mismatch must error");
        assert!(
            index_err.contains("this index was built"),
            "err: {index_err}"
        );
        // The index error must not call the artifact a "store" (the original hard-coded bug).
        // ("stored …" from describe() is fine; we forbid the *noun* phrasings.)
        assert!(
            !index_err.contains(" store") && !index_err.contains("store "),
            "index error must not call the artifact a 'store': {index_err}"
        );

        // Unverifiable (None) path.
        let index_none =
            check_against(None, &graph(A), Artifact::Index, "t").expect_err("None must error");
        assert!(
            index_none.contains("this index carries no"),
            "err: {index_none}"
        );
        assert!(
            !index_none.contains(" store") && !index_none.contains("store "),
            "index error must not call the artifact a 'store': {index_none}"
        );
    }

    #[test]
    fn golden_fingerprint_is_architecture_independent() {
        // [OPUS-4.8] (sq-32i5) Regression guard against any width/endianness drift in the
        // fingerprint computation. The fingerprint of graph A is fixed by construction:
        //   * lengths/counts are hashed as fixed-width `u64` (never `usize`), so wasm32 (32-bit
        //     usize) and native (64-bit usize) fold the SAME bytes;
        //   * `FxHasher` is deterministic and seedless;
        //   * ids/tags are hashed at fixed widths (`u32`/`u8`).
        // The golden value below was produced by running this test once and pasting the output. If
        // the byte sequence ever becomes architecture-dependent again (e.g. a stray `write_usize`),
        // this assertion fails loudly on at least one target. dict_len / triple_count are stable
        // properties of graph A; content_hash is the FxHash of the documented fold.
        let f = Fingerprint::of(&graph(A));
        // Graph A: 4 IRIs (ex:alice, ex:knows, ex:bob, ex:carol), 2 triples.
        assert_eq!(f.dict_len, 4, "dict_len of graph A");
        assert_eq!(f.triple_count, 2, "triple_count of graph A");
        assert_eq!(
            f.content_hash, GOLDEN_A_CONTENT_HASH,
            "content_hash drifted — a width/endianness/fold change broke the cross-machine guarantee"
        );
    }

    /// [OPUS-4.8] (sq-32i5) Golden content hash of graph `A`, architecture-independent by
    /// construction (all hashed inputs are fixed-width; `FxHasher` is seedless & deterministic).
    /// Produced by running `golden_fingerprint_is_architecture_independent` once.
    const GOLDEN_A_CONTENT_HASH: u64 = 8_668_627_031_413_789_344;
}
