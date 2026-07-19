//! The shared id-tuple vocabulary: the `SmallVec`-based [`Row`] / [`Key`] / [`Posting`]
//! aliases over the `sparq-core` dictionary [`Id`], plus the inline-integer id helpers.
//!
//! These are the representation BOTH the SPARQL engine's joins and the reasoners'
//! rule-firing must agree on for a join to be correct (a join is only sound if both sides
//! agree on term identity — see `research/shared-eval-substrate.md` §6, the single-id-space
//! soundness keystone). Today the engine defines `Row`/`Key`/`Posting` privately in
//! `sparq-engine::exec`; this module hoists the *same* aliases into the leaf substrate so a
//! later phase can move the join kernels here and have both consumers share one definition.
//!
//! # Zero-overhead intent
//!
//! The aliases are concrete monomorphic types (`SmallVec<[Id; N]>`), not generic over a
//! `dyn` trait. A kernel written over `&[Row]` therefore compiles to one specialised,
//! inlinable body — no vtable, no `Box<dyn>` between a join's probe and its key comparison.
//! This scaffold introduces no dynamic dispatch; the kernels themselves arrive in a later
//! bead of epic sq-qonbz.
//!
//! # Scaffold status
//!
//! Types and re-exports only — NO join/scan logic lives here yet. The aliases mirror, byte
//! for byte, the engine's current `exec.rs` definitions so the eventual code-move is a pure
//! relabelling validated by the existing deterministic ratchets.

use smallvec::SmallVec;

/// The `sparq-core` dictionary id: a `u32` term identifier (id `0` is the `NO_ID`
/// sentinel). Re-exported so the substrate, the engine and the reasoners all name the
/// same id type without each importing `sparq-core` directly. See [`sparq_core::dict`].
pub use sparq_core::dict::Id;

/// The reserved sentinel id meaning "no such term".
pub use sparq_core::dict::NO_ID;

/// Whether an id encodes an inline integer value (vs a dictionary id or a query-computed
/// local-vocab id). Re-exported from `sparq-core` so a value-space reasoner can recognise
/// inline integers without depending on the engine.
pub use sparq_core::dict::is_inline;

/// The inline id of an integer VALUE in the inline range — the id the canonical
/// `xsd:integer` literal of that value interns / looks up to, with no term construction
/// or lexical-form parse. Re-exported from `sparq-core`. Returns `None` for values outside
/// the inline range (they live in the dictionary).
pub use sparq_core::dict::inline_id_of_int;

/// One solution / fact row: the ids bound to each column. Inlined up to 4 columns (the
/// common case) so a join produces no heap allocation per row — the dominant cost on large
/// join results.
///
/// Mirrors the engine's private `exec::Row` byte for byte (`SmallVec<[Id; 4]>`) so the
/// later join-kernel move is a pure code-move.
pub type Row = SmallVec<[Id; 4]>;

/// A join / group key (the ids of the shared or grouping columns). Inlined up to 2
/// columns — most joins are on one or two variables — so building a hash table or probing
/// it allocates nothing per key.
///
/// Mirrors the engine's private `exec::Key` (`SmallVec<[Id; 2]>`).
pub type Key = SmallVec<[Id; 2]>;

/// A hash-table posting list (row indices sharing a key). Inlined up to 2 — many join keys
/// (and almost all OPTIONAL keys) match only one or two rows — so the build allocates
/// nothing per bucket in the common case.
///
/// Mirrors the engine's private `exec::Posting` (`SmallVec<[usize; 2]>`).
pub type Posting = SmallVec<[usize; 2]>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_aliases_match_sparq_core() {
        // The substrate `Id` IS `sparq_core::dict::Id` (a re-export, not a copy), so the
        // engine, the reasoners and the substrate all agree on the id type. A compile-time
        // type-identity check: assigning across the two names must type-check.
        let core_id: sparq_core::dict::Id = 7;
        let sub_id: Id = core_id;
        assert_eq!(sub_id, 7);
        assert_eq!(NO_ID, 0);
    }

    #[test]
    fn row_is_inline_until_four_columns() {
        // The whole point of the inline width: a 4-column row spills no allocation. We
        // exercise the alias directly (the real type, not a mock) to lock the inline width.
        let mut r: Row = Row::new();
        r.extend_from_slice(&[1, 2, 3, 4]);
        assert!(
            !r.spilled(),
            "a 4-column Row must stay inline (no heap allocation)"
        );
        r.push(5);
        assert!(
            r.spilled(),
            "a 5-column Row spills, as the [Id; 4] inline width dictates"
        );
        assert_eq!(r.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn key_is_inline_until_two_columns() {
        let mut k: Key = Key::new();
        k.extend_from_slice(&[10, 20]);
        assert!(!k.spilled(), "a 2-column Key must stay inline");
        k.push(30);
        assert!(
            k.spilled(),
            "a 3-column Key spills, as the [Id; 2] inline width dictates"
        );
    }

    #[test]
    fn posting_holds_row_indices_inline_until_two() {
        let mut p: Posting = Posting::new();
        p.push(0usize);
        p.push(1usize);
        assert!(!p.spilled(), "a 2-entry Posting must stay inline");
        p.push(2usize);
        assert!(
            p.spilled(),
            "a 3-entry Posting spills, as the [usize; 2] inline width dictates"
        );
    }

    #[test]
    fn inline_id_helpers_round_trip() {
        // The re-exported inline-int helpers are the SAME `sparq-core` functions, exercised
        // through the substrate name. A small in-range value inlines and is recognised.
        let id = inline_id_of_int(42).expect("42 is in the inline integer range");
        assert!(
            is_inline(id),
            "an inline-int id must be recognised by is_inline"
        );
        // A non-inline id (a plausible dictionary id) is not an inline integer.
        assert!(!is_inline(1));
        // A negative value has no inline id (the inline range is non-negative integers).
        assert!(inline_id_of_int(-1).is_none());
    }
}
