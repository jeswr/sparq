//! Triple store: the six sorted permutation indexes over dictionary-encoded
//! triples (Hexastore / RDF-3X / QLever design).
//!
//! Storing all six orderings (SPO SOP PSO POS OSP OPS) means every triple
//! pattern is answered by a single contiguous range (binary search on the
//! bound prefix), and the scan output is sorted by the remaining positions —
//! which is exactly what merge joins need. M1 holds each permutation as a
//! sorted `Vec<[Id; 3]>`; later milestones replace these with block-compressed,
//! optionally memory-mapped columns.

use crate::dict::Id;
use rayon::prelude::*;

/// The six permutations. Each names the order of (subject, predicate, object)
/// columns as stored.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Perm {
    Spo,
    Sop,
    Pso,
    Pos,
    Osp,
    Ops,
}

impl Perm {
    pub const ALL: [Perm; 6] = [Perm::Spo, Perm::Sop, Perm::Pso, Perm::Pos, Perm::Osp, Perm::Ops];

    /// The column indices (into a canonical [s,p,o] triple) in this permutation's
    /// sort order. e.g. POS -> [1,2,0].
    #[inline]
    pub fn order(self) -> [usize; 3] {
        match self {
            Perm::Spo => [0, 1, 2],
            Perm::Sop => [0, 2, 1],
            Perm::Pso => [1, 0, 2],
            Perm::Pos => [1, 2, 0],
            Perm::Osp => [2, 0, 1],
            Perm::Ops => [2, 1, 0],
        }
    }
}

/// A triple pattern over ids: `None` is a variable (wildcard), `Some(id)` is
/// bound.
pub type Pattern = [Option<Id>; 3];

pub struct TripleStore {
    // Each permutation as a flat row-major array of [keyed columns], sorted.
    // Stored in the permutation's column order (so binary search on a prefix is
    // a plain lexicographic comparison of the leading columns).
    perms: [Vec<[Id; 3]>; 6],
}

impl TripleStore {
    /// Builds the six permutation indexes from canonical [s,p,o] triples. The
    /// SPO permutation is sorted (in parallel) and deduplicated first; the other
    /// five permutations are independent and are built and sorted concurrently.
    pub fn from_triples(mut triples: Vec<[Id; 3]>) -> Self {
        // Deduplicate via the SPO ordering first.
        triples.par_sort_unstable();
        triples.dedup();

        let build = |order: [usize; 3]| -> Vec<[Id; 3]> {
            let mut v: Vec<[Id; 3]> = triples
                .iter()
                .map(|t| [t[order[0]], t[order[1]], t[order[2]]])
                .collect();
            v.sort_unstable();
            v
        };

        // The five non-SPO permutations are mutually independent — build them on
        // the rayon pool (each its own sequential sort) for 5-way parallelism.
        let mut others: Vec<Vec<[Id; 3]>> = [Perm::Sop, Perm::Pso, Perm::Pos, Perm::Osp, Perm::Ops]
            .par_iter()
            .map(|p| build(p.order()))
            .collect();

        // Canonical order is [Spo, Sop, Pso, Pos, Osp, Ops]; `others` holds the
        // last five, and `triples` is the already-sorted Spo.
        let perms = [
            triples,
            std::mem::take(&mut others[0]),
            std::mem::take(&mut others[1]),
            std::mem::take(&mut others[2]),
            std::mem::take(&mut others[3]),
            std::mem::take(&mut others[4]),
        ];
        TripleStore { perms }
    }

    pub fn len(&self) -> usize {
        self.perms[0].len()
    }

    pub fn is_empty(&self) -> bool {
        self.perms[0].is_empty()
    }

    /// Heap footprint of the six permutation indexes in bytes (for benchmarking).
    pub fn heap_bytes(&self) -> usize {
        self.perms.iter().map(|p| p.capacity() * std::mem::size_of::<[Id; 3]>()).sum()
    }

    /// Chooses the permutation whose sort order places all bound pattern
    /// positions as a contiguous prefix, so the matches form one range. Returns
    /// the permutation and the number of leading bound columns.
    fn choose(pattern: &Pattern) -> (Perm, usize) {
        // Prefer an order where every bound position precedes every unbound one.
        let bound = |i: usize| pattern[i].is_some();
        for perm in Perm::ALL {
            let order = perm.order();
            // count leading bound columns
            let mut lead = 0;
            while lead < 3 && bound(order[lead]) {
                lead += 1;
            }
            // valid if all bound positions are within the leading prefix
            let total_bound = (0..3).filter(|&i| bound(i)).count();
            if lead == total_bound {
                return (perm, lead);
            }
        }
        (Perm::Spo, 0)
    }

    fn index(&self, perm: Perm) -> &Vec<[Id; 3]> {
        &self.perms[perm as usize]
    }

    /// Like [`choose`], but among the permutations whose sort order places every
    /// bound position as a prefix, prefers one whose first *unbound* column is
    /// `sort_col` (a position 0..3 into a canonical triple). This makes the scan
    /// output sorted by that column, enabling a merge join on it.
    fn choose_sorted(pattern: &Pattern, sort_col: usize) -> (Perm, usize) {
        let bound = |i: usize| pattern[i].is_some();
        let total_bound = (0..3).filter(|&i| bound(i)).count();
        // Prefer: bound positions form the leading prefix AND column `sort_col`
        // is the first column after the prefix.
        for perm in Perm::ALL {
            let order = perm.order();
            let mut lead = 0;
            while lead < 3 && bound(order[lead]) {
                lead += 1;
            }
            if lead == total_bound && lead < 3 && order[lead] == sort_col {
                return (perm, lead);
            }
        }
        Self::choose(pattern)
    }

    /// Returns the contiguous slice of rows (in `perm` order) matching the bound
    /// prefix of the pattern, together with the chosen permutation.
    pub fn scan(&self, pattern: &Pattern) -> Scan<'_> {
        let (perm, lead) = Self::choose(pattern);
        self.scan_with(pattern, perm, lead)
    }

    /// Scans choosing a permutation whose output is sorted by canonical column
    /// `sort_col` (when possible), for merge joins.
    pub fn scan_sorted(&self, pattern: &Pattern, sort_col: usize) -> Scan<'_> {
        let (perm, lead) = Self::choose_sorted(pattern, sort_col);
        self.scan_with(pattern, perm, lead)
    }

    fn scan_with(&self, pattern: &Pattern, perm: Perm, lead: usize) -> Scan<'_> {
        let order = perm.order();
        let rows = self.index(perm);

        // Build the inclusive lower/upper prefix bounds from the bound columns.
        let mut lo = [Id::MIN; 3];
        let mut hi = [Id::MAX; 3];
        for k in 0..lead {
            let v = pattern[order[k]].unwrap();
            lo[k] = v;
            hi[k] = v;
        }
        // Lower columns beyond the prefix stay MIN/MAX.
        let start = lower_bound(rows, &lo);
        let end = upper_bound(rows, &hi);
        Scan {
            rows: &rows[start..end],
            perm,
        }
    }

    /// Estimated number of matches for a pattern (the range length) — the M1
    /// cardinality estimate used by the greedy planner.
    pub fn estimate(&self, pattern: &Pattern) -> usize {
        let s = self.scan(pattern);
        s.rows.len()
    }
}

/// A range of rows in a permutation's column order.
pub struct Scan<'a> {
    pub rows: &'a [[Id; 3]],
    pub perm: Perm,
}

impl<'a> Scan<'a> {
    /// Maps a stored row back to a canonical [s,p,o] triple.
    #[inline]
    pub fn to_spo(&self, row: &[Id; 3]) -> [Id; 3] {
        let order = self.perm.order();
        let mut out = [0; 3];
        out[order[0]] = row[0];
        out[order[1]] = row[1];
        out[order[2]] = row[2];
        out
    }
}

/// First index where `rows[i] >= key` comparing only the leading columns that
/// are constrained (MIN acts as -inf in unconstrained columns of `key`).
fn lower_bound(rows: &[[Id; 3]], key: &[Id; 3]) -> usize {
    rows.partition_point(|row| row < key)
}

/// First index where `rows[i] > key` (MAX acts as +inf).
fn upper_bound(rows: &[[Id; 3]], key: &[Id; 3]) -> usize {
    rows.partition_point(|row| row <= key)
}
