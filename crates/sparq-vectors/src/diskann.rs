//! The **persistent on-disk Vamana ANN index** (`.spqg`) over a [`VectorStore`], keyed by sparq
//! dictionary term ids.
//!
//! [OPUS-5] (issue #3699) **The engine was EXTRACTED into the stand-alone, RDF-agnostic
//! [`sparq_vamana`] crate**: the Vamana build (RobustPrune + the two-pass α schedule), the greedy
//! beam search, the versioned little-endian `.spqg` on-disk format, the PQ candidate cache and the
//! shared mmap read-backing all live there now, and there is exactly ONE copy of each in the tree.
//! The `.spqg` byte layout is UNCHANGED by the move — files written before it still open, and
//! files written after it are byte-identical.
//!
//! What stays HERE is the **consumer glue**, which is genuinely sparq-specific:
//!
//! * **dict-id keying** — the ids in each node record are `sparq_core::dict::Id`s, so
//!   [`nearest`](DiskAnnIndex::nearest) hands back term ids and
//!   [`nearest_term`](DiskAnnIndex::nearest_term) resolves a [`Term`] through the graph dictionary
//!   and maps neighbours back to `Term`s;
//! * **the graph fingerprint** — the extracted index stores an OPAQUE 24-byte
//!   [`sparq_vamana::StalenessToken`] it never interprets; this module supplies
//!   [`Fingerprint::of(graph)`](crate::fingerprint::Fingerprint::of) as that token and performs the
//!   staleness comparison ([`check_graph`](DiskAnnIndex::check_graph),
//!   [`nearest_term_checked`](DiskAnnIndex::nearest_term_checked));
//! * **the filtered-ANN strategy choice** — the pre-filter-vs-traversal crossover driven by
//!   [`FilterConfig`](crate::FilterConfig) and an [`IdMask`](crate::IdMask) carved out by a SPARQL
//!   BGP.
//!
//! See [`sparq_vamana::graph`] for the algorithm, the on-disk format and the honest scope vs. full
//! DiskANN; the caveats there apply verbatim.
//!
//! [OPUS-4.8] (sq-wlzi) **ID-KEYED STALENESS CONTRACT.** Each node record stores its term's
//! build-time dictionary id, and neighbour entries are stored slots — so this index, like the
//! [`VectorStore`] under it, is valid ONLY against the **exact graph generation it was built
//! against**. To serve it, persist that graph (`Graph::save`) and reopen THAT graph (`Graph::open`,
//! which mmaps the **frozen** dict id order — both gated by `sparq-core`'s `mmap` feature) to resolve
//! query terms — **never re-parse the source RDF** (`Graph::load_str` et al.): sparq-core's parallel
//! sharded dict merge assigns thread-count-dependent ids, so a re-parse gives a *different*
//! `id → term` binding and `nearest_term` mis-resolves. [`check_graph`](DiskAnnIndex::check_graph)
//! is a backstop, **not** a sufficient guard — the sq-xhiv fingerprint is thread-count-stable, so it
//! PASSES a re-parse of the same RDF whose ids permuted. See [`crate::fingerprint`] for the full
//! rationale.

use crate::fingerprint::{self, Fingerprint};
use crate::quant::PqConfig;
use crate::store::VectorStore;
use oxrdf::Term;
use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_vamana::{StalenessToken, VamanaIndex};
use std::path::Path;

// The format surface is the extracted crate's; re-exported here so `sparq_vectors::diskann::*`
// (and the crate-root re-exports in `lib.rs`) are unchanged for existing consumers.
pub use sparq_vamana::{sibling_graph_path, VamanaConfig, SPQG_MAGIC, SPQG_VERSION};

/// Translates a sparq graph [`Fingerprint`] into the extracted index's opaque staleness token.
fn token_of(fp: Fingerprint) -> StalenessToken {
    StalenessToken::new(fp.to_bytes())
}

/// A **persistent on-disk Vamana index** opened over a `.spqg` file (memory-mapped on native
/// targets) — the out-of-core counterpart to `VectorIndex`. Built once with
/// [`build`](Self::build) / [`build_with`](Self::build_with), reopened with [`open`](Self::open)
/// at near-zero cost (mmap + header validation, no rebuild) or from fetched/embedded bytes with
/// [`open_from_bytes`](Self::open_from_bytes) (the wasm/filesystem-less path — memmap2 is
/// target-gated out of wasm32 builds).
///
/// [OPUS-5] (issue #3699) A thin dict-id/`Term`-keyed facade over [`sparq_vamana::VamanaIndex`],
/// which owns the graph, the search and the `.spqg` format; see the module docs for the split.
pub struct DiskAnnIndex {
    inner: VamanaIndex,
}

impl DiskAnnIndex {
    /// Builds the Vamana graph over `store` with default parameters and writes it to `path`,
    /// then opens it. Equivalent to `build_with(store, path, VamanaConfig::default())`. The index
    /// is written WITHOUT a graph fingerprint (unverifiable — [`check_graph`](Self::check_graph)
    /// errors); use [`build_for`](Self::build_for) to bind it to its graph.
    pub fn build<P: AsRef<Path>>(store: &VectorStore, path: P) -> Result<DiskAnnIndex, String> {
        Self::build_with(store, path, VamanaConfig::default())
    }

    /// Builds the Vamana graph over `store` with `cfg`, writes the `.spqg` file at `path`, and
    /// opens it memory-mapped. The build is in RAM (one-off); the open is cheap forever after.
    /// Written without a fingerprint — see [`build`](Self::build) / [`build_with_for`](Self::build_with_for).
    pub fn build_with<P: AsRef<Path>>(
        store: &VectorStore,
        path: P,
        cfg: VamanaConfig,
    ) -> Result<DiskAnnIndex, String> {
        VamanaIndex::build(store, path, cfg, None).map(|inner| DiskAnnIndex { inner })
    }

    /// [OPUS-4.8] (sq-32i5) Like [`build`](Self::build) but binds the index to `graph` (embeds its
    /// fingerprint), so [`check_graph`](Self::check_graph) / [`nearest_term_checked`](Self::nearest_term_checked)
    /// can reject a query against a different graph generation. Pass the graph whose term ids `store`
    /// is keyed by.
    pub fn build_for<P: AsRef<Path>>(
        store: &VectorStore,
        path: P,
        graph: &Graph,
    ) -> Result<DiskAnnIndex, String> {
        Self::build_with_for(store, path, VamanaConfig::default(), graph)
    }

    /// [OPUS-4.8] (sq-32i5) Like [`build_with`](Self::build_with) but binds the index to `graph`'s
    /// fingerprint (see [`build_for`](Self::build_for)).
    pub fn build_with_for<P: AsRef<Path>>(
        store: &VectorStore,
        path: P,
        cfg: VamanaConfig,
        graph: &Graph,
    ) -> Result<DiskAnnIndex, String> {
        VamanaIndex::build(store, path, cfg, Some(token_of(Fingerprint::of(graph))))
            .map(|inner| DiskAnnIndex { inner })
    }

    /// [OPUS-4.8] (sq-qamd) Builds the Vamana graph **with a PQ candidate cache**: in addition to
    /// the full-precision node records, it fits a [`ProductQuantizer`](crate::ProductQuantizer)
    /// over `store` (with `pq_cfg`), encodes every vector into the in-RAM code cache, and persists
    /// both alongside the graph (the trailing PQ section, encoding tag `1`). The opened index then
    /// searches DiskANN-style: rank candidates on the RAM codes (no disk), re-rank the final beam
    /// off the mmap.
    ///
    /// Errors if `pq_cfg` is invalid for `store`'s dimension or the store is empty (PQ needs at
    /// least one training vector — use the plain [`build_with`](Self::build_with) for an empty
    /// store). Like [`build_with`](Self::build_with), the index is written WITHOUT a graph binding.
    pub fn build_with_pq<P: AsRef<Path>>(
        store: &VectorStore,
        path: P,
        cfg: VamanaConfig,
        pq_cfg: PqConfig,
    ) -> Result<DiskAnnIndex, String> {
        VamanaIndex::build_with_pq(store, path, cfg, pq_cfg, None)
            .map(|inner| DiskAnnIndex { inner })
    }

    /// Opens a `.spqg` file memory-mapped, **without rebuilding** — the whole point of this index.
    /// Validates the header and that the file size matches `count` records so no later search can
    /// read out of bounds; the records themselves page in on access.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DiskAnnIndex, String> {
        VamanaIndex::open(path).map(|inner| DiskAnnIndex { inner })
    }

    /// Opens a `.spqg` document held entirely in memory — for environments without a filesystem
    /// (the bytes were fetched, embedded, or decompressed by the caller), the `.spqg` counterpart
    /// of [`VectorStore::open_from_bytes`]. Validation is identical to [`open`](Self::open).
    /// [FABLE-5] (sq-98c)
    pub fn open_from_bytes(bytes: Vec<u8>) -> Result<DiskAnnIndex, String> {
        VamanaIndex::open_from_bytes(bytes).map(|inner| DiskAnnIndex { inner })
    }

    /// [OPUS-4.8] (sq-qamd) Whether this index carries an in-RAM PQ candidate cache.
    pub fn has_pq_cache(&self) -> bool {
        self.inner.has_pq_cache()
    }

    /// The index's vector dimension.
    pub fn dim(&self) -> usize {
        self.inner.dim()
    }
    /// Number of indexed nodes (= store vectors at build time).
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// Whether the index holds no nodes.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// [OPUS-4.8] (sq-1wc1) **Predicate-constrained (filtered) approximate top-`k`**: like
    /// [`nearest`](Self::nearest) but the result is restricted to ids the `mask` permits — the
    /// RDF-native filtered-ANN path. The mask is the candidate id-set a SPARQL BGP selects (e.g.
    /// `?node a :Car`); only neighbours in it are returned, while the traversal still hops through
    /// non-matching nodes for connectivity.
    ///
    /// Strategy is chosen by the mask's **selectivity** ([`FilterConfig`](crate::FilterConfig)):
    /// a *very selective* mask is served by an exact **pre-filter** scan over just the masked ids
    /// (cheaper and exact than touching the whole graph), a *broad* mask by **filtered traversal**
    /// of the Vamana graph. Both honour the mask; see the [`filter`](crate::filter) module docs and
    /// `tests/filtered.rs` for the measured recall. Uses `FilterConfig::default`; for a custom
    /// threshold/beam use [`nearest_filtered_with`](Self::nearest_filtered_with).
    ///
    /// An empty mask returns no results (the BGP matched nothing); an all-zero query returns no
    /// results (same degenerate contract as [`nearest`](Self::nearest)).
    #[cfg(feature = "filtered-ann")]
    pub fn nearest_filtered(
        &self,
        query: &[f32],
        mask: &crate::IdMask,
        store: &VectorStore,
        k: usize,
    ) -> Vec<(Id, f32)> {
        self.nearest_filtered_with(query, mask, store, k, crate::FilterConfig::default())
    }

    /// [OPUS-4.8] (sq-1wc1) [`nearest_filtered`](Self::nearest_filtered) with an explicit
    /// [`FilterConfig`](crate::FilterConfig) (pre-filter ↔ traversal crossover and the traversal
    /// beam factor). The `store` is needed for the pre-filter strategy (it scans the masked vectors
    /// directly); for the traversal strategy it is unused, but the method takes it unconditionally so
    /// the strategy choice stays an internal detail.
    #[cfg(feature = "filtered-ann")]
    pub fn nearest_filtered_with(
        &self,
        query: &[f32],
        mask: &crate::IdMask,
        store: &VectorStore,
        k: usize,
        cfg: crate::FilterConfig,
    ) -> Vec<(Id, f32)> {
        assert_eq!(
            query.len(),
            self.dim(),
            "query dim {} != index dim {}",
            query.len(),
            self.dim()
        );
        if mask.is_empty() {
            return Vec::new();
        }
        // Selective mask → exact pre-filter scan over just the masked ids (exact AND cheaper than
        // walking the whole graph to accept a handful of nodes; also avoids a graph-connectivity
        // miss to an isolated accepted node). Broad mask → filtered traversal.
        if cfg.prefer_prefilter(mask.len(), self.len()) {
            return crate::filter::nearest_exact_filtered(store, query, mask, k);
        }
        let beam = (self.inner.search_beam().max(k)) * cfg.traversal_beam_factor.max(1);
        self.inner.nearest_filtered_by(query, k, beam, |id| mask.contains(id))
    }

    /// Approximate top-`k` ids by cosine similarity to `query`, best first. An all-zero
    /// `query` returns no results (same contract as [`nearest_exact`](crate::ann::nearest_exact)
    /// and `VectorIndex::nearest`).
    pub fn nearest(&self, query: &[f32], k: usize) -> Vec<(Id, f32)> {
        self.inner.nearest(query, k)
    }

    /// Approximate top-`k` neighbours of `term`: resolves it through the graph's dictionary,
    /// looks its vector up in `store`, excludes the term itself and maps neighbour ids back to
    /// [`Term`]s. Empty if the term is absent or unembedded. Mirrors
    /// `VectorIndex::nearest_term`.
    ///
    /// [OPUS-4.8] (sq-32i5) This does NOT verify the index/store match `graph` — pass a graph
    /// whose ids have shifted since build and the results are silently WRONG. Use
    /// [`nearest_term_checked`](Self::nearest_term_checked) (or call [`check_graph`](Self::check_graph)
    /// once after open) to make a mismatch a hard error.
    pub fn nearest_term(
        &self,
        term: &Term,
        graph: &Graph,
        store: &VectorStore,
        k: usize,
    ) -> Vec<(Term, f32)> {
        let Some(id) = graph.id_of(term) else { return Vec::new() };
        let Some(query) = store.get(id) else { return Vec::new() };
        self.nearest(query, k + 1)
            .into_iter()
            .filter(|&(n, _)| n != id)
            .take(k)
            .map(|(n, s)| (graph.dict.term(n), s))
            .collect()
    }

    /// [OPUS-4.8] (sq-32i5) The graph fingerprint this index was built against, or `None` for a
    /// legacy version-1 file / an index built without a graph. See [`check_graph`](Self::check_graph).
    ///
    /// [OPUS-5] Decoded from the extracted index's opaque staleness token — the same 24 header
    /// bytes, reinterpreted here as the sparq graph fingerprint that wrote them.
    pub fn fingerprint(&self) -> Option<Fingerprint> {
        self.inner
            .staleness_token()
            .map(|t| Fingerprint::from_bytes(t.as_bytes()))
    }

    /// [OPUS-4.8] (sq-32i5) **Checked open guard**: verifies this index was built against `graph`
    /// (and, since the index is queried alongside the store, that `store` matches it too) by
    /// recomputing `graph`'s fingerprint and comparing it to BOTH stored fingerprints. Returns a
    /// descriptive `Err` on any mismatch — the index and store are keyed by `graph`'s dictionary
    /// ids, so a mismatch means a query would silently resolve to the WRONG vectors. A legacy
    /// version-1 index/store (no stored fingerprint) also errors, as "unverifiable".
    ///
    /// Call once after [`open`](Self::open) (it is O(dict_len), not per-query). The store is checked
    /// as well because [`nearest_term`](Self::nearest_term) resolves the query vector through it.
    pub fn check_graph(&self, store: &VectorStore, graph: &Graph) -> fingerprint::CheckResult {
        let origin = "<.spqg index>";
        fingerprint::check_against(self.fingerprint(), graph, fingerprint::Artifact::Index, origin)?;
        store.check_graph(graph)
    }

    /// [OPUS-4.8] (sq-32i5) [`nearest_term`](Self::nearest_term) with the staleness check: returns
    /// `Err` if this index or `store` was built against a different graph generation than `graph`
    /// (which would otherwise return silently-wrong neighbours), else `Ok` with the neighbours.
    pub fn nearest_term_checked(
        &self,
        term: &Term,
        graph: &Graph,
        store: &VectorStore,
        k: usize,
    ) -> Result<Vec<(Term, f32)>, String> {
        self.check_graph(store, graph)?;
        Ok(self.nearest_term(term, graph, store, k))
    }
}

#[cfg(test)]
mod fingerprint_tests {
    // [OPUS-4.8] (sq-32i5) Checked-open tests for the `.spqg` on-disk index: build against graph A
    // then (1) query against A → OK + correct neighbours, (2) query against a DIFFERENT graph B →
    // descriptive Err, (3) fingerprint survives reopen, (4) a legacy version-1 `.spqg` opens (and
    // searches) but reports as unverifiable.
    use super::*;
    use crate::Fingerprint;
    use oxrdf::NamedNode;
    use sparq_vamana::{SPQG_HEADER_LEN, SPQG_HEADER_LEN_V1};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp(tag: &str, ext: &str) -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sparq_fpg_{tag}_{}_{n}.{ext}", std::process::id()))
    }

    fn graph(ttl: &str) -> Graph {
        Graph::load_str(ttl, "turtle").expect("load test turtle")
    }

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    const A: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:carol .
    "#;
    const B: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:dave ex:likes ex:eve .
        ex:eve ex:likes ex:frank .
    "#;

    fn build_store(g: &Graph, path: &std::path::Path) -> VectorStore {
        let alice = g.id_of(&iri("http://example.org/alice")).unwrap();
        let bob = g.id_of(&iri("http://example.org/bob")).unwrap();
        let carol = g.id_of(&iri("http://example.org/carol")).unwrap();
        let mut s = VectorStore::create(path, 4).unwrap().with_fingerprint(g);
        s.put(alice, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        s.put(bob, &[0.9, 0.1, 0.0, 0.0]).unwrap();
        s.put(carol, &[0.0, 0.0, 0.0, 1.0]).unwrap();
        s.finalize().unwrap();
        s
    }

    #[test]
    fn build_for_then_query_against_build_graph_ok_and_correct() {
        let ga = graph(A);
        let store_path = tmp("ok", "spqv");
        let store = build_store(&ga, &store_path);
        let idx_path = tmp("ok", "spqg");
        let idx = DiskAnnIndex::build_for(&store, &idx_path, &ga).unwrap();
        // (1) Checked against the build graph → OK, and bob is alice's nearest.
        assert!(idx.check_graph(&store, &ga).is_ok());
        let got = idx
            .nearest_term_checked(&iri("http://example.org/alice"), &ga, &store, 1)
            .expect("checked query against the build graph must succeed");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, iri("http://example.org/bob"));
        std::fs::remove_file(&store_path).ok();
        std::fs::remove_file(&idx_path).ok();
    }

    #[test]
    fn query_against_different_graph_errs() {
        let ga = graph(A);
        let store_path = tmp("mm", "spqv");
        let store = build_store(&ga, &store_path);
        let idx_path = tmp("mm", "spqg");
        let idx = DiskAnnIndex::build_for(&store, &idx_path, &ga).unwrap();
        let gb = graph(B);
        // (2) Against a DIFFERENT graph → descriptive Err, not silently wrong neighbours.
        assert!(idx.check_graph(&store, &gb).is_err());
        let qerr = idx
            .nearest_term_checked(&iri("http://example.org/dave"), &gb, &store, 1)
            .expect_err("a checked query against a mismatched graph must error");
        assert!(qerr.contains("mismatch") || qerr.contains("wrong results"), "err: {qerr}");
        std::fs::remove_file(&store_path).ok();
        std::fs::remove_file(&idx_path).ok();
    }

    #[test]
    fn fingerprint_survives_reopen() {
        let ga = graph(A);
        let store_path = tmp("rt", "spqv");
        let store = build_store(&ga, &store_path);
        let idx_path = tmp("rt", "spqg");
        DiskAnnIndex::build_for(&store, &idx_path, &ga).unwrap();
        // (3) Reopen the .spqg and confirm the stored fingerprint equals the live one.
        let reopened = DiskAnnIndex::open(&idx_path).unwrap();
        assert_eq!(reopened.fingerprint(), Some(Fingerprint::of(&ga)));
        std::fs::remove_file(&store_path).ok();
        std::fs::remove_file(&idx_path).ok();
    }

    #[test]
    fn v2_index_built_without_graph_binding_is_unverifiable_not_mismatch() {
        // [OPUS-4.8] (sq-32i5) A v2 `.spqg` built WITHOUT a graph binding writes an all-zero
        // fingerprint block; on reopen it must decode to `None` (reported as "unverifiable"),
        // NOT a zero fingerprint that would surface as a spurious "DIFFERENT graph" mismatch.
        let ga = graph(A);
        let store_path = tmp("nob", "spqv");
        let store = build_store(&ga, &store_path);
        let idx_path = tmp("nob", "spqg");
        DiskAnnIndex::build(&store, &idx_path).unwrap(); // no graph binding
        let idx = DiskAnnIndex::open(&idx_path).unwrap();
        assert_eq!(
            idx.fingerprint(),
            None,
            "all-zero block must decode to None"
        );
        let err = idx
            .check_graph(&store, &ga)
            .expect_err("an unbound index must not be certified");
        assert!(
            err.contains("carries no graph fingerprint"),
            "err must say unverifiable, not mismatch: {err}"
        );
        std::fs::remove_file(&store_path).ok();
        std::fs::remove_file(&idx_path).ok();
    }

    #[test]
    fn legacy_v1_spqg_opens_but_is_unverifiable() {
        // (4) A version-1 `.spqg`: build the default (no-fingerprint) v2 file, then rewrite its
        // header to a genuine v1 (version=1, fingerprint block dropped) so the legacy open path is
        // exercised. It still opens and searches; check_graph reports it unverifiable.
        let ga = graph(A);
        let store_path = tmp("legacy", "spqv");
        let store = build_store(&ga, &store_path);
        let idx_path = tmp("legacy", "spqg");
        DiskAnnIndex::build(&store, &idx_path).unwrap();
        let mut bytes = std::fs::read(&idx_path).unwrap();
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes.drain(SPQG_HEADER_LEN_V1..SPQG_HEADER_LEN); // remove the 24-byte fingerprint block
        std::fs::write(&idx_path, &bytes).unwrap();

        let idx = DiskAnnIndex::open(&idx_path).expect("a legacy v1 .spqg must still open");
        assert!(idx.fingerprint().is_none());
        // It still searches correctly against the build graph (data layout is offset-32).
        let got = idx.nearest_term(&iri("http://example.org/alice"), &ga, &store, 1);
        assert_eq!(got[0].0, iri("http://example.org/bob"));
        // ...but cannot be verified.
        assert!(idx.check_graph(&store, &ga).is_err());
        std::fs::remove_file(&store_path).ok();
        std::fs::remove_file(&idx_path).ok();
    }
}
