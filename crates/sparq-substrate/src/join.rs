//! The shared id-tuple join kernels — sorted **merge join**, radix-partitioned
//! **hash join**, index-nested-loop **bind join**, and **leapfrog trie-join**
//! (WCOJ) — over the [`Row`] / [`Key`] / [`Posting`] vocabulary.
//!
//! These are the SPARQL engine's join probe loops, **moved here** from
//! `sparq-engine::exec`, generalised to operate on plain `&[Row]` slices plus a
//! tiny [`JoinKeys`] descriptor (the column layout) rather than the engine's
//! private `Bindings` struct. The engine keeps its planner, `Bindings`,
//! `LocalVocab` interning and `ScanCmp` filter pushdown private and wraps these
//! kernels with a thin adapter (`research/shared-eval-substrate.md` §2.3, Phase 3
//! of epic sq-qonbz). A future reasoner can drive the *same* kernels with its own
//! adapter, so both consumers share one join body without either depending on the
//! other.
//!
//! # Zero-overhead intent (the load-bearing contract)
//!
//! There is NO `Box<dyn>` / `&dyn` / vtable anywhere between a join's probe loop
//! and its key projection or its cooperative-cancellation poll. The kernels are
//! monomorphic over the concrete `Id = u32` and the `SmallVec` row/key aliases,
//! and the cancellation hook is a generic [`Budget`] type parameter (not a trait
//! object), so the compiler emits one specialised, inlinable body per call site.
//! Every hot item carries `#[inline]` so cross-crate inlining (with the workspace
//! LTO profile) keeps the engine's join hot loops identical to the pre-move
//! codegen. This is verified by the engine join/scan/BGP micro-benches staying
//! within noise of the pre-move baseline and the W3C SPARQL conformance floor
//! staying bit-identical.
//!
//! The kernels make **no entailment claim** — they compute joins over id-tuples;
//! which triples *should* exist is entirely the caller's (engine / reasoner)
//! concern (`research/shared-eval-substrate.md` §6).

use crate::rows::{Id, Key, Posting, Row, NO_ID};
use rustc_hash::FxHashMap;
use std::cmp::Ordering;

/// A cooperative-cancellation poll the join kernels call once per key-group /
/// probe-row / distinct-value so a long join can be bounded without the kernel
/// knowing *how* the caller bounds it.
///
/// This is a **generic type parameter** on each kernel, NOT a trait object: the
/// engine supplies a zero-sized type whose [`exhausted`](Budget::exhausted) reads
/// its thread-local `QueryBudget`, a reasoner can supply its own closure-budget,
/// and a caller that wants no bound uses [`NoBudget`] — each monomorphises to a
/// direct call (or, for `NoBudget`, to a constant `false` the optimiser deletes),
/// so the probe loop pays no vtable.
pub trait Budget {
    /// Whether the join has produced enough output to stop (a sticky row/byte/time
    /// cap, or an external cancellation). `rows` is the current output length, so a
    /// caller can price the working set. Returning `true` cleanly truncates the
    /// result at a key-group boundary.
    fn exhausted(&self, rows: usize) -> bool;
}

/// An unbounded [`Budget`]: never stops the join. Zero-sized; its `exhausted`
/// folds to a constant `false` the optimiser removes, so an unbounded kernel call
/// has byte-identical codegen to a hand-written unbounded loop.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoBudget;

impl Budget for NoBudget {
    #[inline]
    fn exhausted(&self, _rows: usize) -> bool {
        false
    }
}

/// A pure (no thread-local) exhaustion snapshot for parallel workers, where the
/// installing thread's sticky flag is out of reach. Generic, so the engine's
/// flattened limits implement it without a vtable on the rayon fold.
pub trait BudgetSnapshot: Sync {
    /// Whether a worker holding `rows` accumulated rows should stop producing.
    fn hit(&self, rows: usize) -> bool;
}

/// A [`BudgetSnapshot`] that never trips — the unbounded parallel case.
impl BudgetSnapshot for NoBudget {
    #[inline]
    fn hit(&self, _rows: usize) -> bool {
        false
    }
}

/// The column layout for combining a left (build) row with a right (probe) row:
/// which columns form the equi-join key on each side, which already-bound columns
/// must additionally agree (a repeated non-key shared variable), and which right
/// columns are appended to the output.
///
/// This is the **descriptor** the design calls `JoinKeys`: it captures the
/// row→key projection and the combine layout as plain index data (built by the
/// caller from its own variable layout), so the kernel stays generic over *what*
/// the columns mean while being monomorphic over the concrete `Row`/`Key` types.
/// No part of it is a trait object.
#[derive(Clone, Debug, Default)]
pub struct JoinKeys {
    /// `(left_col, right_col)` index pairs whose ids form the equi-join key — the
    /// columns hashed / merged on. For a hash join these are split into the build
    /// and probe key projections; for a merge join this is the single sorted
    /// variable. Build and probe must agree on the order of these pairs so the key
    /// tuples line up.
    pub key_cols: Vec<(usize, usize)>,
    /// The right-side columns appended (in order) after the left row's columns to
    /// form the combined output row.
    pub right_only: Vec<usize>,
}

impl JoinKeys {
    /// The left-side key column indices, in `key_cols` order.
    #[inline]
    pub fn left_key(&self, row: &[Id]) -> Key {
        self.key_cols.iter().map(|&(lc, _)| row[lc]).collect()
    }

    /// The right-side key column indices, in `key_cols` order — projected to the
    /// SAME key tuple shape as [`left_key`](JoinKeys::left_key) so build and probe
    /// keys are equal exactly when the join columns are equal.
    #[inline]
    pub fn right_key(&self, row: &[Id]) -> Key {
        self.key_cols.iter().map(|&(_, rc)| row[rc]).collect()
    }
}

/// The partition/lookup hash for a join key — build and probe must agree on it.
#[inline]
pub fn key_hash(key: &Key) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = rustc_hash::FxHasher::default();
    key.hash(&mut h);
    h.finish()
}

/// Number of radix partitions for the parallel hash-join build. 64 spreads well at
/// high thread counts while keeping the per-partition tag-scan cheap.
pub const JOIN_PARTS: usize = 64;

/// Sorted **merge join** of two relations already sorted on a single shared key
/// column (`lk` on the left, `rk` on the right), with optional `extra_shared`
/// `(left_col, right_col)` pairs that must additionally agree (a second shared
/// variable that is not the sorted key). Appends one combined row — the full left
/// row plus the `right_only` columns — per matching pair into `out`.
///
/// The inputs are plain row slices; the caller owns the output buffer and the
/// `out_vars` layout. `budget` is polled once per key group so a capped query
/// truncates cleanly at a group boundary (identical to the engine's pre-move
/// per-group check).
// The column-layout arguments are the price of operating on plain row slices instead of the
// engine's private `Bindings` struct — the whole point of the move (zero-overhead, shareable).
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn merge_join<B: Budget>(
    left: &[Row],
    lk: usize,
    right: &[Row],
    rk: usize,
    extra_shared: &[(usize, usize)],
    right_only: &[usize],
    budget: &B,
    out: &mut Vec<Row>,
) {
    let (l, r) = (left, right);
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        // Coarse budget check once per key group.
        if budget.exhausted(out.len()) {
            break;
        }
        let (lv, rv) = (l[i][lk], r[j][rk]);
        match lv.cmp(&rv) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                let mut i2 = i;
                while i2 < l.len() && l[i2][lk] == lv {
                    i2 += 1;
                }
                let mut j2 = j;
                while j2 < r.len() && r[j2][rk] == rv {
                    j2 += 1;
                }
                for lrow in l.iter().take(i2).skip(i) {
                    for rrow in r.iter().take(j2).skip(j) {
                        if extra_shared.iter().all(|&(lc, rc)| lrow[lc] == rrow[rc]) {
                            let mut row = lrow.clone();
                            for &rc in right_only {
                                row.push(rrow[rc]);
                            }
                            out.push(row);
                        }
                    }
                }
                i = i2;
                j = j2;
            }
        }
    }
}

/// Builds the serial hash table for a hash join: maps each build-side key to the
/// ascending list of build-row indices sharing it.
#[inline]
pub fn build_table(build: &[Row], keys: &JoinKeys) -> FxHashMap<Key, Posting> {
    let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
    for (ri, row) in build.iter().enumerate() {
        t.entry(keys.left_key(row)).or_default().push(ri);
    }
    t
}

/// Builds the radix-partitioned hash tables for a parallel hash join: `JOIN_PARTS`
/// private maps, each over the build rows whose key-hash falls in that partition.
/// Within a partition rows are scanned in ascending index, so each posting list
/// stays in ascending build-row order — exactly the serial build — and the probe
/// output is byte-identical. Requires the `parallel` feature (the inner rayon
/// import is the caller's; this returns the per-partition maps).
#[inline]
pub fn build_partitioned(build: &[Row], keys: &JoinKeys, parts: &[u8]) -> Vec<FxHashMap<Key, Posting>> {
    (0..JOIN_PARTS)
        .map(|p| {
            let mut t: FxHashMap<Key, Posting> = FxHashMap::default();
            for (ri, row) in build.iter().enumerate() {
                if parts[ri] as usize == p {
                    t.entry(keys.left_key(row)).or_default().push(ri);
                }
            }
            t
        })
        .collect()
}

/// Emits, for one probe row, every combined output row (its build-side matches
/// from `tables`, each extended with the probe-only columns). Shared by the serial
/// and parallel probe paths so they are byte-identical. `tables` is either one
/// serial table or `JOIN_PARTS` radix partitions; `probe_only` are the probe
/// columns appended after the build row.
#[inline]
pub fn probe_emit(
    prow: &Row,
    keys: &JoinKeys,
    build: &[Row],
    tables: &[FxHashMap<Key, Posting>],
    probe_only: &[usize],
    out: &mut Vec<Row>,
) {
    let key: Key = keys.right_key(prow);
    let table = if tables.len() == 1 {
        &tables[0]
    } else {
        &tables[(key_hash(&key) % JOIN_PARTS as u64) as usize]
    };
    if let Some(matches) = table.get(&key) {
        for &bi in matches {
            let mut combined = build[bi].clone();
            for &pi in probe_only {
                combined.push(prow[pi]);
            }
            out.push(combined);
        }
    }
}

/// Serial **hash join** probe: for every probe row, emit its build-side matches.
/// `budget` is polled once per probe row (the engine's pre-move per-row check).
/// The build table(s) and the layout are the caller's; this is the probe hot loop.
#[inline]
pub fn hash_probe_serial<B: Budget>(
    probe: &[Row],
    keys: &JoinKeys,
    build: &[Row],
    tables: &[FxHashMap<Key, Posting>],
    probe_only: &[usize],
    budget: &B,
    out: &mut Vec<Row>,
) {
    for prow in probe {
        // Coarse budget check once per probe row.
        if budget.exhausted(out.len()) {
            break;
        }
        probe_emit(prow, keys, build, tables, probe_only, out);
    }
}

/// Index-nested-loop **bind join** combine step: given the groups of result rows
/// keyed by the join value and, for one distinct value, the projected new-variable
/// tuples of the matching scanned triples, append one combined row per
/// (result-row, match) pair. The scan + filter pushdown stay with the caller (the
/// engine), which owns the store and the `ScanCmp`; this is the pure id-tuple
/// combine that produced the per-value output rows.
#[inline]
pub fn bind_combine(result_rows: &[Row], ris: &[usize], new_vals: &[Id], out: &mut Vec<Row>) {
    for &ri in ris {
        let mut combined = result_rows[ri].clone();
        combined.extend(new_vals.iter().copied());
        out.push(combined);
    }
}

// ---- Leapfrog Triejoin (WCOJ) ------------------------------------------------
//
// LFTJ (Veldhuizen 2014) evaluates a BGP one *variable* at a time in a fixed
// global order. At each variable it intersects, via the "leapfrog" galloping
// search, the sorted value streams of every pattern mentioning that variable,
// then recurses. Each pattern is a [`Trie`] of its variable columns. The total
// work is bounded by the AGM fractional-edge-cover bound. The trie *contents* are
// built by the caller (the engine projects them from a permutation index); this
// module owns the navigation (the `TrieIter` cursor, the `Leapfrog` intersection,
// and `lftj_recurse`), which is the WCOJ hot loop.

/// A pattern's relation projected onto its variables (in global order) as sorted,
/// deduplicated tuples — the trie LFTJ navigates level by level. The caller builds
/// and fills `tuples`.
#[derive(Clone, Debug, Default)]
pub struct Trie {
    /// The sorted, deduplicated variable tuples (each in global variable order).
    pub tuples: Vec<Vec<Id>>,
}

/// One open level of a [`TrieIter`]: `hi` bounds the current key's subtree (rows
/// sharing all already-fixed columns), `cur` is the cursor within it.
struct Frame {
    hi: usize,
    cur: usize,
}

/// A cursor over a [`Trie`], using Veldhuizen's open-on-entry semantics: it starts
/// *above* the root, and `open()` descends one column, resetting the cursor to the
/// start of that subtree. This reset is what makes non-contiguous variable
/// participation correct — re-entering a level re-opens (rewinds) the iterator.
pub struct TrieIter<'a> {
    trie: &'a Trie,
    frames: Vec<Frame>,
}

impl<'a> TrieIter<'a> {
    /// A fresh cursor positioned above the root of `trie`.
    #[inline]
    pub fn new(trie: &'a Trie) -> Self {
        TrieIter { trie, frames: Vec::new() }
    }
    /// The column currently being iterated (valid once at least one `open`).
    #[inline]
    fn col(&self) -> usize {
        self.frames.len() - 1
    }
    /// Whether the current level's cursor has run past its subtree.
    #[inline]
    fn at_end(&self) -> bool {
        let f = self.frames.last().unwrap();
        f.cur >= f.hi
    }
    /// The id at the cursor in the current column.
    #[inline]
    fn key(&self) -> Id {
        let col = self.col();
        let f = self.frames.last().unwrap();
        self.trie.tuples[f.cur][col]
    }
    /// End (exclusive) of the run of rows in `[start, hi)` whose `col` equals the
    /// value at `start`. The slice is sorted, so this is a binary search — O(log n)
    /// rather than a linear scan of the (possibly large) run.
    #[inline]
    fn run_end(&self, col: usize, start: usize, hi: usize, val: Id) -> usize {
        start + self.trie.tuples[start..hi].partition_point(|row| row[col] <= val)
    }
    /// Advances to the next distinct value in the current column.
    #[inline]
    fn next(&mut self) {
        let col = self.col();
        let (cur, hi) = {
            let f = self.frames.last().unwrap();
            (f.cur, f.hi)
        };
        let val = self.trie.tuples[cur][col];
        self.frames.last_mut().unwrap().cur = self.run_end(col, cur, hi, val);
    }
    /// Galloping seek: first value `>= x` in the current column.
    #[inline]
    fn seek(&mut self, x: Id) {
        let col = self.col();
        let f = self.frames.last_mut().unwrap();
        let (mut a, mut b) = (f.cur, f.hi);
        while a < b {
            let m = a + (b - a) / 2;
            if self.trie.tuples[m][col] < x {
                a = m + 1;
            } else {
                b = m;
            }
        }
        f.cur = a;
    }
    /// Descends one column: into the subtree of the parent's current key (or, at
    /// the root, the whole relation), with the cursor reset to its start.
    #[inline]
    fn open(&mut self) {
        match self.frames.last() {
            None => {
                self.frames.push(Frame { hi: self.trie.tuples.len(), cur: 0 });
            }
            Some(&Frame { cur: plo, hi: phi }) => {
                let pcol = self.frames.len() - 1;
                let val = self.trie.tuples[plo][pcol];
                let end = self.run_end(pcol, plo, phi, val);
                self.frames.push(Frame { hi: end, cur: plo });
            }
        }
    }
    /// Ascends one column.
    #[inline]
    fn up(&mut self) {
        self.frames.pop();
    }
}

/// Leapfrog intersection of the participating iterators at one level.
struct Leapfrog {
    order: Vec<usize>, // participant indices, kept in cyclic key order
    p: usize,
    ended: bool,
    key: Id,
}

impl Leapfrog {
    fn init(iters: &mut [TrieIter], parts: &[usize]) -> Self {
        let mut lf = Leapfrog { order: parts.to_vec(), p: 0, ended: false, key: 0 };
        if parts.iter().any(|&i| iters[i].at_end()) {
            lf.ended = true;
            return lf;
        }
        lf.order.sort_by_key(|&i| iters[i].key());
        lf.search(iters);
        lf
    }
    fn search(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        loop {
            let max = iters[self.order[(self.p + k - 1) % k]].key();
            let min = iters[self.order[self.p]].key();
            if min == max {
                self.key = min;
                return;
            }
            iters[self.order[self.p]].seek(max);
            if iters[self.order[self.p]].at_end() {
                self.ended = true;
                return;
            }
            self.p = (self.p + 1) % k;
        }
    }
    fn next(&mut self, iters: &mut [TrieIter]) {
        let k = self.order.len();
        iters[self.order[self.p]].next();
        if iters[self.order[self.p]].at_end() {
            self.ended = true;
            return;
        }
        self.p = (self.p + 1) % k;
        self.search(iters);
    }
}

/// The Leapfrog-Triejoin recursion: at each global `level`, intersect the
/// participating tries' value streams and recurse, appending one output [`Row`]
/// (a copy of `current`) per full match. `budget` is polled once per leapfrog key
/// (sticky, so it also unwinds the enclosing recursion levels) — the engine's
/// pre-move per-key check, threaded as a generic type so the recursion carries no
/// vtable.
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn lftj_recurse<B: Budget>(
    iters: &mut [TrieIter],
    parts_at_level: &[Vec<usize>],
    level: usize,
    n_levels: usize,
    current: &mut [Id],
    budget: &B,
    out: &mut Vec<Row>,
) {
    if level == n_levels {
        out.push(Row::from_slice(current));
        return;
    }
    let parts = &parts_at_level[level];
    // Open-on-entry: descend each relevant iterator into this level (rewinding it).
    for &i in parts {
        iters[i].open();
    }
    let mut lf = Leapfrog::init(iters, parts);
    while !lf.ended {
        // Coarse budget check once per leapfrog key (sticky, so it also unwinds
        // the enclosing recursion levels).
        if budget.exhausted(out.len()) {
            break;
        }
        current[level] = lf.key;
        lftj_recurse(iters, parts_at_level, level + 1, n_levels, current, budget, out);
        lf.next(iters);
    }
    for &i in parts {
        iters[i].up();
    }
}

/// SPARQL solution compatibility on the shared columns: an unbound (`NO_ID`) value
/// never conflicts; two bound values must be equal. Used by the OPTIONAL / UNION /
/// VALUES-UNDEF fallback nested loop the engine drives.
#[inline]
pub fn compatible(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)]) -> bool {
    shared.iter().all(|&(lc, rc)| {
        let (a, b) = (lrow[lc], rrow[rc]);
        a == NO_ID || b == NO_ID || a == b
    })
}

/// Combines two compatible rows: left's row extended with the right-only columns,
/// filling any shared column that was unbound on the left from the right side.
#[inline]
pub fn merge_rows(lrow: &[Id], rrow: &[Id], shared: &[(usize, usize)], right_only: &[usize]) -> Row {
    let mut row = Row::from_slice(lrow);
    for &(lc, rc) in shared {
        if row[lc] == NO_ID {
            row[lc] = rrow[rc];
        }
    }
    for &rc in right_only {
        row.push(rrow[rc]);
    }
    row
}

/// Whether any row leaves any of the given columns unbound (`NO_ID`).
#[inline]
pub fn any_unbound(rows: &[Row], cols: &[usize]) -> bool {
    rows.iter().any(|r| cols.iter().any(|&c| r[c] == NO_ID))
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    fn row(xs: &[Id]) -> Row {
        Row::from_slice(xs)
    }

    #[test]
    fn merge_join_single_key_matches_by_value() {
        // left (?a ?b) sorted on ?a (col 0); right (?a ?c) sorted on ?a (col 0).
        // key_cols = [(0,0)]; right_only = [1] (right's ?c).
        let left = vec![row(&[1, 10]), row(&[2, 20]), row(&[2, 21])];
        let right = vec![row(&[2, 200]), row(&[3, 300])];
        let mut out = Vec::new();
        merge_join(&left, 0, &right, 0, &[], &[1], &NoBudget, &mut out);
        // ?a=2 matches both left rows (b=20,21) with right c=200.
        assert_eq!(out.len(), 2);
        assert!(out.contains(&row(&[2, 20, 200])));
        assert!(out.contains(&row(&[2, 21, 200])));
    }

    #[test]
    fn merge_join_respects_extra_shared() {
        // A second shared variable (left col 1 vs right col 1) must also agree.
        let left = vec![row(&[1, 5]), row(&[1, 9])];
        let right = vec![row(&[1, 5, 50]), row(&[1, 7, 70])];
        let mut out = Vec::new();
        // key on col 0; extra shared (1,1); right_only = [2].
        merge_join(&left, 0, &right, 0, &[(1, 1)], &[2], &NoBudget, &mut out);
        assert_eq!(out, vec![row(&[1, 5, 50])]);
    }

    #[test]
    fn hash_join_serial_combines_build_and_probe() {
        // build (?a ?b), probe (?a ?c); key (0,0); probe_only = [1].
        let build = vec![row(&[1, 10]), row(&[2, 20]), row(&[1, 11])];
        let probe = vec![row(&[1, 100]), row(&[3, 300])];
        let keys = JoinKeys { key_cols: vec![(0, 0)], right_only: vec![] };
        let tables = vec![build_table(&build, &keys)];
        let mut out = Vec::new();
        hash_probe_serial(&probe, &keys, &build, &tables, &[1], &NoBudget, &mut out);
        // ?a=1 matches build rows [1,10] and [1,11] (ascending index order), with c=100.
        assert_eq!(out, vec![row(&[1, 10, 100]), row(&[1, 11, 100])]);
    }

    #[test]
    fn budget_truncates_at_group_boundary() {
        struct Cap(usize);
        impl Budget for Cap {
            fn exhausted(&self, rows: usize) -> bool {
                rows >= self.0
            }
        }
        let left = vec![row(&[1, 1]), row(&[2, 2]), row(&[3, 3])];
        let right = vec![row(&[1, 1]), row(&[2, 2]), row(&[3, 3])];
        let mut out = Vec::new();
        // Cap at 1 row: the second key group's pre-check trips and stops.
        merge_join(&left, 0, &right, 0, &[], &[], &Cap(1), &mut out);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn lftj_two_relations_triangle_edge() {
        // Two relations over levels [0,1]: R(a,b) and S(a,b). Both share both
        // levels, so LFTJ yields the intersection of the tuple sets.
        let r = Trie { tuples: vec![vec![1, 2], vec![1, 3], vec![2, 4]] };
        let s = Trie { tuples: vec![vec![1, 2], vec![2, 4], vec![5, 6]] };
        let mut iters = vec![TrieIter::new(&r), TrieIter::new(&s)];
        // both relations participate at both levels.
        let parts_at_level = vec![vec![0, 1], vec![0, 1]];
        let mut current = vec![NO_ID, NO_ID];
        let mut out = Vec::new();
        lftj_recurse(&mut iters, &parts_at_level, 0, 2, &mut current, &NoBudget, &mut out);
        // Intersection: {(1,2),(2,4)}.
        assert_eq!(out.len(), 2);
        assert!(out.contains(&row(&[1, 2])));
        assert!(out.contains(&row(&[2, 4])));
    }

    #[test]
    fn compatible_treats_unbound_as_wildcard() {
        // shared (0,0): left bound to 1, right unbound -> compatible.
        let l = [1u32, 9];
        let r = [NO_ID, 9];
        assert!(compatible(&l, &r, &[(0, 0)]));
        // both bound but unequal -> incompatible.
        let r2 = [2u32, 9];
        assert!(!compatible(&l, &r2, &[(0, 0)]));
    }

    #[test]
    fn merge_rows_fills_unbound_left_from_right() {
        // left col 0 unbound; shared (0,0) fills it from right; right_only = [1].
        let l = [NO_ID, 7];
        let r = [42u32, 99];
        let out = merge_rows(&l, &r, &[(0, 0)], &[1]);
        let expected: Row = smallvec![42u32, 7u32, 99u32];
        assert_eq!(out, expected);
    }

    #[test]
    fn bind_combine_appends_new_vals_per_result_row() {
        let result = vec![row(&[1, 10]), row(&[2, 20]), row(&[3, 30])];
        let mut out = Vec::new();
        // distinct value matched result rows 0 and 2; the scanned triple's new vars are [99].
        bind_combine(&result, &[0, 2], &[99], &mut out);
        assert_eq!(out, vec![row(&[1, 10, 99]), row(&[3, 30, 99])]);
    }
}
