//! [OPUS-4.8] (sq-pi44) **Incremental add / remove / update of vectors** against an
//! already-finalized [`VectorStore`](crate::store::VectorStore) — WITHOUT a full re-embed
//! and file rebuild.
//!
//! # Why
//!
//! A `.spqv` store is build-once-immutable: [`VectorStore::put`](crate::store::VectorStore::put)
//! errors after [`finalize`](crate::store::VectorStore::finalize), and there is no remove or
//! update. So today *any* graph change — one new entity, one deleted entity — forces a full
//! re-embed and a fresh file. The structural indexes ([`sparq-sim`](https://docs.rs/sparq-sim)
//! style) track graph mutation for free because the index *is* the data; the vector store does
//! not, because it is keyed by dictionary id and stored densely.
//!
//! # What this adds (the **delta sidecar**)
//!
//! A small in-RAM layer over the immutable base mmap:
//!   * an **append map** `id → vector` of vectors added or updated since the base was built, and
//!   * a **tombstone set** of ids removed since the base was built.
//!
//! Every read path on the store ([`get`](crate::store::VectorStore::get),
//! [`iter`](crate::store::VectorStore::iter), [`len`](crate::store::VectorStore::len)) consults
//! the delta transparently: a tombstoned id reads as absent, an appended/updated id reads its
//! delta vector (the delta SHADOWS the base for an id present in both), and a plain base id reads
//! the mmap. Search (`nearest_exact`, the ANN indexes that consume `iter`) therefore unions
//! base+delta and honours tombstones with no change to the search code itself.
//!
//! [`VectorStore::compact`](crate::store::VectorStore::compact) folds the delta back into a fresh
//! base file: the result is **equivalent to a from-scratch rebuild over the same final vector
//! set** (same `(id, vector)` pairs ⇒ a store whose `get`/`iter`/`len` agree exactly with one
//! built by `create` + `put` over that set).
//!
//! # Generation tie (the staleness guard, sq-32i5)
//!
//! A delta is only meaningful against the graph generation it was built against — the ids it
//! appends/tombstones are that generation's dictionary ids. A [`VectorDelta`] therefore carries
//! the [`Fingerprint`] of the graph it targets, and [`VectorStore::apply_delta`] REJECTS a delta
//! whose generation does not match the base store's bound fingerprint (gate: a delta tied to
//! generation N applied to a base of generation M ≠ N is an error, not a silent mis-key). This
//! reuses the sq-32i5 header fingerprint rather than inventing a second staleness mechanism.
//!
//! # Scope (honest boundary)
//!
//! The delta is **in-RAM only**: it lives on the open [`VectorStore`] handle and is lost when the
//! handle is dropped (the base `.spqv` on disk is unchanged until
//! [`compact`](crate::store::VectorStore::compact) writes a new one). A *persisted* delta sidecar
//! (its own on-disk format, crash-durable across process restarts) is tracked as a follow-up — see
//! the crate docs / the sq-pi44 follow-up bead. `compact` is the durability path today: it
//! materializes the delta into a normal, validated `.spqv`.
//!
//! Opt-in: the whole layer is gated behind the **`delta`** cargo feature, off by default, and adds
//! NO dependency (it reuses the in-tree `rustc-hash` map/set), so the default build carries zero
//! delta code.

use crate::fingerprint::Fingerprint;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::Id;

/// [OPUS-4.8] (sq-pi44) The in-RAM delta sidecar: vectors appended/updated and ids tombstoned
/// since the base store was built, plus the graph [`Fingerprint`] the delta is keyed against.
///
/// An id is in AT MOST one of `appended` / `tombstones` at any time — adding/updating an id clears
/// its tombstone, removing an id drops it from `appended` and tombstones it. The base store is never
/// mutated; the delta merely shadows it on read until
/// [`compact`](crate::store::VectorStore::compact) folds it in.
#[derive(Clone, Debug, Default)]
pub struct VectorDelta {
    /// The graph generation this delta targets, or `None` for a delta over a store with no base
    /// fingerprint. [`VectorStore::apply_delta`] checks it against the receiving store.
    pub(crate) generation: Option<Fingerprint>,
    /// `id → vector` for vectors added or updated since the base; SHADOWS the base for a shared id.
    pub(crate) appended: FxHashMap<Id, Vec<f32>>,
    /// ids removed since the base (a base id reads as absent; an appended id is dropped, not kept).
    pub(crate) tombstones: FxHashSet<Id>,
}

impl VectorDelta {
    /// A fresh empty delta bound to `generation` (the fingerprint of the graph it targets).
    pub(crate) fn new(generation: Option<Fingerprint>) -> VectorDelta {
        VectorDelta {
            generation,
            appended: FxHashMap::default(),
            tombstones: FxHashSet::default(),
        }
    }

    /// The graph generation this delta is keyed against (the base store's fingerprint), or `None`
    /// if the base carried none. See [`VectorStore::apply_delta`](crate::store::VectorStore::apply_delta).
    pub fn generation(&self) -> Option<Fingerprint> {
        self.generation
    }

    /// Number of appended/updated vectors held in the delta.
    pub fn appended_len(&self) -> usize {
        self.appended.len()
    }

    /// Number of tombstoned ids held in the delta.
    pub fn tombstone_len(&self) -> usize {
        self.tombstones.len()
    }

    /// Whether the delta holds no appends and no tombstones (a no-op delta).
    pub fn is_empty(&self) -> bool {
        self.appended.is_empty() && self.tombstones.is_empty()
    }
}
