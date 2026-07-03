//! Columnar aggregate reducer kernels for the M4 Phase 4 seam (`sq-pntvh.4`). [SONNET-4.6]
//!
//! These operate over gathered per-group id slices (all-inline-integer gate):
//! exact integer arithmetic, gather-free value decoding via the inline encoding.
//! Byte-identity with the scalar `eval_aggregate` path is proven per-reducer — see
//! the per-function doc comments. Used only under the `vectorized` feature.

use sparq_core::dict::{is_inline, Id, INLINE_BASE, NO_ID};

/// The maximum value encodable by an inline integer id.
/// `id - INLINE_BASE ∈ [0, INLINE_MAX_I64]` for every inline id.
pub(crate) const INLINE_MAX_I64: i64 = (1_u64 << 30) as i64 - 1;

/// Exact i128 SUM over an all-inline-integer id slice.
///
/// Returns `None` if any id is non-inline (the caller should decline to the scalar path).
/// Returns `Some(0)` for an empty slice (the identity element, matching the empty-SUM
/// SPARQL rule `SUM({}) = 0`).
///
/// **Byte-identity with the scalar `sum_values` path:**
///
/// The scalar accumulator is `Num::Int(i64)`.
/// `Num::Int(x).binop(Num::Int(y), ArithOp::Add)` uses `i64::checked_add`.
///
/// Overflow bound (load-bearing for byte-identity with the scalar path):
///   max inline value = `INLINE_MAX_I64` = 2^30 - 1 < 2^30
///   max rows bounded by dictionary capacity (< 2^31 distinct terms)
///   max sum < 2^30 · 2^31 = 2^61 < i64::MAX (2^63 − 1)
///
/// Therefore the scalar SUM accumulator (`Num::Int(i64)`) does NOT overflow for
/// any all-inline-integer column, and the exact i128 sum equals the scalar i64 result.
/// A `debug_assert!` at the cast site in `columnar_aggregate` checks this at test time.
pub(crate) fn reduce_sum(ids: &[Id]) -> Option<i128> {
    let mut acc: i128 = 0;
    for &id in ids {
        if !is_inline(id) {
            return None;
        }
        // Each inline integer value is in [0, INLINE_MAX_I64] (load-bearing bound).
        debug_assert!((id - INLINE_BASE) as i64 <= INLINE_MAX_I64);
        acc += i128::from(id - INLINE_BASE);
    }
    Some(acc)
}

/// COUNT of non-`NO_ID` ids in the slice.
///
/// For an all-inline-integer group every id is non-`NO_ID` (since `NO_ID = 0 < INLINE_BASE`),
/// so this equals `ids.len()` in that case. Provided as a generic count primitive for
/// callers that do not need the all-inline guard.
///
/// **Byte-identity:** the scalar `CountSolutions { distinct: false }` returns
/// `Value::Num(Num::Int(members.len() as i64))`. For an all-inline group every member is
/// bound, so `reduce_count(group_ids) == members.len()`.
pub(crate) fn reduce_count(ids: &[Id]) -> usize {
    ids.iter().filter(|&&id| id != NO_ID).count()
}

/// MIN inline id in the slice (= MIN value, since value = id − INLINE_BASE and the base
/// is the same for every inline id).
///
/// Returns `None` if the slice is empty OR if any id is non-inline. The caller should:
/// * for an EMPTY result — treat as unbound (`NO_ID`), matching `minmax_values([], _) == Value::Unbound`;
/// * for a non-empty `None` — decline to the scalar path (non-inline id present).
///
/// **Byte-identity:** for an all-inline-integer group the scalar `minmax_values` path
/// folds `Num::Int(v)` values; the minimum value maps to the minimum id (monotone offset).
/// `value_to_id(graph, local, &Value::Num(Num::Int(v_min)))` equals the id this function
/// returns, because `inline_id_of_int(v_min) = INLINE_BASE + v_min` and
/// `min id = INLINE_BASE + min value`.
pub(crate) fn reduce_min_id(ids: &[Id]) -> Option<Id> {
    if ids.is_empty() {
        return None;
    }
    let mut best: Id = Id::MAX;
    for &id in ids {
        if !is_inline(id) {
            return None;
        }
        if id < best {
            best = id;
        }
    }
    Some(best)
}

/// MAX inline id in the slice (= MAX value). Returns `None` if the slice is empty or
/// any id is non-inline (see `reduce_min_id` for the caller's responsibility).
///
/// **Byte-identity:** symmetric to `reduce_min_id`.
pub(crate) fn reduce_max_id(ids: &[Id]) -> Option<Id> {
    if ids.is_empty() {
        return None;
    }
    let mut best: Id = 0;
    let mut found = false;
    for &id in ids {
        if !is_inline(id) {
            return None;
        }
        if !found || id > best {
            best = id;
            found = true;
        }
    }
    if found { Some(best) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_inline(v: u32) -> Id {
        INLINE_BASE + v
    }

    /// T1: Exactness (mutation-proof). Verifies the exact i128 result for a known set of
    /// inline ids. If someone changes the accumulator to `f64`, this test will fail to
    /// compile when the return type changes, OR fail at runtime when large-value sums
    /// exceed f64 precision (sum > 2^53, achievable with many max-value rows in production
    /// — see the overflow bound in `reduce_sum`'s doc comment). Pinning the exact i128
    /// value is the soundest mutation kill short of runtime f64-divergence at this scale.
    #[test]
    fn t1_reduce_sum_exact_i128_result() {
        // Three distinct inline values; sum is exact and pinned.
        // 1_000_000_000 + 73_741_823 + 500_000 = 1_074_241_823
        let ids = vec![mk_inline(1_000_000_000), mk_inline(73_741_823), mk_inline(500_000)];
        let sum = reduce_sum(&ids).unwrap();
        assert_eq!(sum, 1_074_241_823_i128, "exact i128 sum must equal 1_074_241_823");

        // Two max-inline values: 2 * (2^30 - 1) = 2^31 - 2 = 2_147_483_646
        let v_max = INLINE_MAX_I64 as u32;
        let ids2 = vec![mk_inline(v_max), mk_inline(v_max)];
        let sum2 = reduce_sum(&ids2).unwrap();
        let expected2: i128 = 2 * (INLINE_MAX_I64 as i128);
        assert_eq!(sum2, expected2, "two max-inline values must sum to 2*(2^30-1)");

        // Empty sum = 0 (identity).
        assert_eq!(reduce_sum(&[]), Some(0_i128), "empty sum must be 0");
    }

    /// T2: Min/max tie-freeness: multiple rows with the same inline id — both reducers
    /// return that same id (no ambiguity, no sentinel confusion).
    #[test]
    fn t2_min_max_tie_free() {
        let id = mk_inline(42);
        let ids = vec![id, id, id];
        assert_eq!(reduce_min_id(&ids), Some(id));
        assert_eq!(reduce_max_id(&ids), Some(id));
    }

    /// T2b: Min/max order correctness across a spread of values.
    #[test]
    fn t2b_min_max_spread() {
        let ids = vec![mk_inline(100), mk_inline(1), mk_inline(50), mk_inline(200), mk_inline(3)];
        assert_eq!(reduce_min_id(&ids), Some(mk_inline(1)));
        assert_eq!(reduce_max_id(&ids), Some(mk_inline(200)));
        // Mutation check: a MIN returning MAX would fail.
        assert_ne!(reduce_min_id(&ids), reduce_max_id(&ids), "MIN != MAX over spread");
    }

    /// T3: Empty / non-inline decline.
    #[test]
    fn t3_empty_and_non_inline_decline() {
        // Empty: sum = Some(0), min/max = None.
        assert_eq!(reduce_sum(&[]), Some(0_i128));
        assert_eq!(reduce_min_id(&[]), None);
        assert_eq!(reduce_max_id(&[]), None);

        // Non-inline dict id (id = 1 < INLINE_BASE).
        let non_inline: Id = 1;
        assert!(!is_inline(non_inline), "id=1 must be non-inline (dict id)");

        assert_eq!(reduce_sum(&[mk_inline(5), non_inline]), None);
        assert_eq!(reduce_min_id(&[mk_inline(5), non_inline]), None);
        assert_eq!(reduce_max_id(&[mk_inline(5), non_inline]), None);

        // NO_ID (= 0) is also non-inline.
        let no_id: Id = NO_ID;
        assert!(!is_inline(no_id));
        assert_eq!(reduce_sum(&[mk_inline(7), no_id]), None);
    }

    /// T4: reduce_count counts non-NO_ID ids.
    #[test]
    fn t4_reduce_count() {
        let ids = vec![mk_inline(1), mk_inline(2), mk_inline(3)];
        assert_eq!(reduce_count(&ids), 3);

        // With NO_ID: only 2 of 3 are non-NO_ID.
        let ids_with_noid = vec![mk_inline(1), NO_ID, mk_inline(3)];
        assert_eq!(reduce_count(&ids_with_noid), 2);

        // Empty.
        assert_eq!(reduce_count(&[]), 0);

        // All NO_ID.
        assert_eq!(reduce_count(&[NO_ID, NO_ID]), 0);
    }
}
