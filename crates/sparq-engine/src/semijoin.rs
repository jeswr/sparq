//! [OPUS-4.8] (sq-gr8mb / survey §A3) Exact-bitmap semi-join reducer on dense `u32` ids.
//!
//! A binary BGP join materialises its left side, then scans the next pattern and
//! merge/hash-joins on the connecting variable. A scan row survives that join **iff**
//! its join-key id appears in the left side's join column — every other scanned row is
//! pure waste. This module builds a membership set ([`KeyFilter`]) over the left side's
//! distinct join-key ids and lets [`crate::exec::scan_to_bindings`] DROP a row whose
//! join key is absent **before** projection, so the wasted rows never enter the join.
//!
//! Why a bitmap fits *this* engine. sparq's dictionary assigns DENSE `u32` ids in
//! `[1, dict::INLINE_BASE)` (see `sparq_core::dict`), so a flat bitmap over a key range
//! is a perfect-hash, ZERO-false-positive, branch-free membership test — strictly
//! cheaper and more accurate than the Bloom filters the wider join literature settled
//! for (CIDR'26 "Not Yannakakis"). When the keys are dense the bitmap is chosen; when
//! they are SPARSE+HUGE (a few keys scattered across a wide id span — e.g. inline
//! integers high in the id space, or local-vocab ids) a flat bitmap would waste memory,
//! so the reducer falls back to a hash set. Both are EXACT (no false positives): the
//! semi-join only removes rows the join would have dropped anyway, so the produced
//! result is unchanged — only fewer rows are scanned.
//!
//! **Correctness contract (load-bearing).** This is an OPT-IN performance feature
//! (`semijoin-bitmap`, OFF by default). With the feature OFF none of this code compiles
//! and the executor's existing path is byte-identical. With it ON the RESULT is
//! identical to OFF (the prefilter is membership-exact and only drops rows that would
//! fail the join), proven by the feature-on-vs-off differential
//! (`tests/semijoin_differential.rs`).

use sparq_core::dict::Id;

/// How many distinct keys, per id span covered, below which a flat bitmap is judged too
/// sparse and the hash-set fallback is used instead. The bitmap costs `span / 8` bytes;
/// the hash set costs roughly a machine word per key. Choosing the bitmap only when the
/// keys are dense enough keeps the membership structure small. (Density heuristic only —
/// it never affects RESULTS, just which exact structure backs the membership test.)
const MIN_KEY_DENSITY_INV: u64 = 64;

/// The largest id span for which a flat bitmap is ever allocated, regardless of density.
/// Bounds the worst-case bitmap to `MAX_BITMAP_SPAN / 8` bytes so a pathological key
/// range can never blow up memory; above it the hash set is used. (Also never affects
/// RESULTS.)
const MAX_BITMAP_SPAN: u64 = 1 << 28; // 256 Mi ids => 32 MiB bitmap ceiling.

/// An EXACT membership test over a set of join-key ids — the semi-join reducer's
/// "reachable id" structure. Backed by a flat bitmap when the keys are dense (the common
/// case for dictionary ids) and a hash set when they are sparse+huge. Either way
/// [`KeyFilter::contains`] has NO false positives, so filtering a scan by it removes
/// only rows that would fail the downstream join.
pub(crate) enum KeyFilter {
    /// Dense path: one bit per id in `[base, base + bits.len()*64)`. `contains(id)` is a
    /// word index + bit test. Ids outside the range are absent by construction (the
    /// range spans exactly the observed min..=max key).
    Bitmap { base: Id, bits: Vec<u64> },
    /// Sparse path: an exact hash set of the keys.
    Set(rustc_hash::FxHashSet<Id>),
}

impl KeyFilter {
    /// Builds a membership filter over the DISTINCT values in `keys` (the left side's
    /// join column). Returns `None` when there is nothing to gain — an empty key set
    /// (the join is already empty; the executor handles that separately) — so the caller
    /// can skip installing a prefilter. Chooses the bitmap when the keys are dense within
    /// their min..=max span and the span is bounded, else an exact hash set.
    pub(crate) fn build(keys: impl Iterator<Item = Id>) -> Option<KeyFilter> {
        // First pass: distinct keys + min/max, via an exact set. (The set is reused as
        // the sparse fallback, so the sparse path pays for it only once.)
        let mut set: rustc_hash::FxHashSet<Id> = rustc_hash::FxHashSet::default();
        let mut min = Id::MAX;
        let mut max = Id::MIN;
        for k in keys {
            if set.insert(k) {
                if k < min {
                    min = k;
                }
                if k > max {
                    max = k;
                }
            }
        }
        if set.is_empty() {
            return None;
        }
        // The id range the bitmap would have to span (inclusive). `max >= min` holds
        // because the set is non-empty.
        let span = (max as u64) - (min as u64) + 1;
        // Dense ENOUGH and bounded => flat bitmap; else exact hash set. `set.len()` is the
        // distinct-key count; `span / set.len()` is the average gap between keys.
        let dense = span <= MAX_BITMAP_SPAN && span <= (set.len() as u64) * MIN_KEY_DENSITY_INV;
        if dense {
            let words = (span / 64 + 1) as usize;
            let mut bits = vec![0u64; words];
            for &k in &set {
                let off = (k - min) as usize;
                bits[off / 64] |= 1u64 << (off % 64);
            }
            Some(KeyFilter::Bitmap { base: min, bits })
        } else {
            Some(KeyFilter::Set(set))
        }
    }

    /// EXACT membership: `true` iff `id` is one of the keys the filter was built from. No
    /// false positives in either backing representation.
    #[inline]
    pub(crate) fn contains(&self, id: Id) -> bool {
        match self {
            KeyFilter::Bitmap { base, bits } => {
                // Below the range or above it => not a member (the bitmap spans exactly
                // the observed min..=max). The unsigned subtraction underflow-guard is
                // the `id < *base` check.
                if id < *base {
                    return false;
                }
                let off = (id - *base) as usize;
                let word = off / 64;
                match bits.get(word) {
                    Some(w) => (w >> (off % 64)) & 1 == 1,
                    None => false,
                }
            }
            KeyFilter::Set(s) => s.contains(&id),
        }
    }
}

/// The kind of backing store a [`KeyFilter`] chose, for tests / introspection only.
#[cfg(test)]
impl KeyFilter {
    pub(crate) fn is_bitmap(&self) -> bool {
        matches!(self, KeyFilter::Bitmap { .. })
    }
}

/// Whether `id` is a DICTIONARY id (the dense `[1, INLINE_BASE)` range the bitmap path is
/// designed for) rather than an inline integer / local-vocab id high in the `u32` space.
/// Not used to gate correctness (the filter is exact for any id) — kept as a documented
/// predicate next to the density rationale so the "dense ids" claim is checkable.
#[inline]
#[cfg(test)]
pub(crate) fn is_dense_dict_id(id: Id) -> bool {
    id != sparq_core::dict::NO_ID && id < sparq_core::dict::INLINE_BASE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_keys_build_none() {
        assert!(KeyFilter::build(std::iter::empty()).is_none());
    }

    #[test]
    fn dense_keys_pick_bitmap_and_membership_is_exact() {
        // 0..1000 dense dictionary ids (id 0 is NO_ID, so start at 1).
        let keys: Vec<Id> = (1..1000).collect();
        let f = KeyFilter::build(keys.iter().copied()).unwrap();
        assert!(f.is_bitmap(), "a fully-dense range must use the bitmap");
        for k in 1..1000 {
            assert!(f.contains(k), "present key {} must be a member", k);
        }
        assert!(!f.contains(0), "id below the range is absent");
        assert!(!f.contains(1000), "id above the range is absent");
        assert!(!f.contains(Id::MAX), "far-away id is absent");
    }

    #[test]
    fn sparse_huge_keys_fall_back_to_set_and_stay_exact() {
        // A handful of keys scattered across the whole u32 span: a bitmap would need
        // ~512 MiB, so the sparse fallback must kick in.
        let keys = [1u32, 1 << 20, 1 << 28, (1u32 << 31) + 5, u32::MAX - 1];
        let f = KeyFilter::build(keys.iter().copied()).unwrap();
        assert!(!f.is_bitmap(), "a sparse+huge span must use the set fallback");
        for &k in &keys {
            assert!(f.contains(k), "present key {} must be a member", k);
        }
        assert!(!f.contains(2), "absent key is not a member");
        assert!(!f.contains((1 << 28) + 1), "absent key is not a member");
    }

    #[test]
    fn duplicate_keys_are_deduplicated() {
        let keys = [5u32, 5, 5, 7, 7, 9];
        let f = KeyFilter::build(keys.iter().copied()).unwrap();
        for k in [5, 7, 9] {
            assert!(f.contains(k));
        }
        assert!(!f.contains(6));
        assert!(!f.contains(8));
    }

    #[test]
    fn single_key_membership() {
        let f = KeyFilter::build(std::iter::once(42u32)).unwrap();
        assert!(f.contains(42));
        assert!(!f.contains(41));
        assert!(!f.contains(43));
    }

    #[test]
    fn dense_dict_id_predicate_matches_dict_partition() {
        use sparq_core::dict;
        assert!(is_dense_dict_id(1));
        assert!(is_dense_dict_id(dict::INLINE_BASE - 1));
        assert!(!is_dense_dict_id(dict::NO_ID));
        assert!(!is_dense_dict_id(dict::INLINE_BASE)); // first inline integer id
    }
}
