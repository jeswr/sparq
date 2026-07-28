//! [OPUS-5] (issue #3699) The **vector source** seam: the only thing this crate needs to know
//! about a caller's vector collection.
//!
//! The index and the quantizers were extracted from `sparq-vectors`, where they read directly
//! from that crate's memory-mapped `VectorStore` and keyed everything by an RDF dictionary term
//! id. Neither coupling is intrinsic: the build only ever walks `(id, vector)` pairs in a fixed
//! slot order and records the id in each node record. [`VectorSource`] is that requirement and
//! nothing more, so any collection — an mmap'd store, a `Vec<Vec<f32>>`, a database cursor —
//! can back a `.spqg` index.
//!
//! # Contract
//!
//! * [`iter`](VectorSource::iter) must yield exactly [`len`](VectorSource::len) pairs, each vector
//!   exactly [`dim`](VectorSource::dim) long, and must yield the **same order** every time it is
//!   called within one build — slot `i` of the on-disk graph is the `i`-th yielded pair, and the
//!   PQ code array is written in the same order.
//! * Ids need not be dense, sorted, or unique to be *stored* correctly, but a caller that expects
//!   to map a result id back to one item should keep them unique.
//! * No vector may be all-zero: it has no direction, so it cannot be L2-normalized and the build
//!   panics. (`sparq-vectors`' `VectorStore::put` rejects zero vectors for exactly this reason.)

/// The caller's opaque identifier for one vector — written verbatim into each node record and
/// returned from every search. `sparq-vectors` uses an RDF dictionary term id here; a
/// stand-alone consumer can use a row number, an offset, or any other `u32` handle.
pub type VectorId = u32;

/// A read-only, fixed-order collection of equal-length `f32` vectors the index can be built over.
/// See the module docs for the full contract.
pub trait VectorSource {
    /// Length of every vector in the source. Must be > 0.
    fn dim(&self) -> usize;

    /// Number of vectors — the node count of the graph built over it.
    fn len(&self) -> usize;

    /// Whether the source holds no vectors (an empty source builds an empty, searchable index).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every `(id, vector)` pair in **slot order** (see the module docs: the order is the graph's
    /// slot numbering, so it must be stable within a build).
    fn iter(&self) -> impl Iterator<Item = (VectorId, &[f32])>;
}

/// A plain in-RAM [`VectorSource`] over a row-major `len × dim` buffer — the zero-ceremony way to
/// build an index from vectors you already hold (and what this crate's own tests use).
///
/// ```
/// use sparq_vamana::{SliceVectors, VectorSource};
/// let src = SliceVectors::new(2, vec![10, 20], vec![1.0, 0.0, 0.0, 1.0]).unwrap();
/// assert_eq!(src.len(), 2);
/// assert_eq!(src.iter().next().unwrap(), (10, &[1.0, 0.0][..]));
/// ```
#[derive(Clone, Debug)]
pub struct SliceVectors {
    dim: usize,
    ids: Vec<VectorId>,
    data: Vec<f32>,
}

impl SliceVectors {
    /// Wraps `ids` + a row-major `ids.len() × dim` `data` buffer. Errors if `dim` is zero or the
    /// buffer length does not match `ids.len() × dim` (so [`iter`](VectorSource::iter) can never
    /// slice out of bounds).
    pub fn new(dim: usize, ids: Vec<VectorId>, data: Vec<f32>) -> Result<SliceVectors, String> {
        if dim == 0 {
            return Err("SliceVectors dim must be > 0".into());
        }
        let want = ids
            .len()
            .checked_mul(dim)
            .ok_or("SliceVectors buffer length overflows")?;
        if data.len() != want {
            return Err(format!(
                "SliceVectors data len {} != ids {} × dim {dim}",
                data.len(),
                ids.len()
            ));
        }
        Ok(SliceVectors { dim, ids, data })
    }
}

impl VectorSource for SliceVectors {
    fn dim(&self) -> usize {
        self.dim
    }
    fn len(&self) -> usize {
        self.ids.len()
    }
    fn iter(&self) -> impl Iterator<Item = (VectorId, &[f32])> {
        self.ids
            .iter()
            .enumerate()
            .map(move |(slot, &id)| (id, &self.data[slot * self.dim..(slot + 1) * self.dim]))
    }
}
