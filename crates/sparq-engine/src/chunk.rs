//! Vectorized (columnar / vector-at-a-time) execution primitives — the first
//! concrete building block of the **M4 plan** (`research/optimization-techniques.md`,
//! bead `sq-hvfe`). [OPUS-4.8]
//!
//! The general evaluator (the crate's internal `exec` module) is *row-materialising*: an intermediate
//! result is a `Vec<Row>` where `Row = SmallVec<[Id; 4]>` (one heap-friendly tuple
//! per solution), and a FILTER walks rows one at a time dispatching per cell. The
//! measured scaling loss (`research/optimization-techniques.md`) is exactly that
//! **per-row dispatch + per-row id→value lookup** — a vector engine instead keeps a
//! batch of solutions *column-major* and applies one tight, branch-predictable,
//! auto-vectorisable loop per column.
//!
//! This module provides that columnar intermediate, [`DataChunk`], plus the two
//! kernels a vector-at-a-time FILTER needs:
//!
//! * a **comparison kernel** ([`DataChunk::select_numeric`]) that scans one column
//!   through the graph's f64 value cache and produces a [`SelVec`] (the indices that
//!   pass) — never materialising a term, mirroring the row evaluator's sargable
//!   numeric pushdown (`ScanCmp::Num` in the internal `exec` module);
//! * a **selection / gather kernel** ([`DataChunk::apply_selection`]) that compacts a
//!   chunk down to the surviving rows, the vector-at-a-time analogue of dropping
//!   filtered rows.
//!
//! It is **opt-in** (the `vectorized` cargo feature) and **OFF by default**: when the
//! feature is off, zero columnar code compiles and the default native + wasm builds
//! are byte-identical. It pulls in **no new dependencies** (only [`sparq_core::dict::Id`]
//! and [`sparq_core::Graph`], both already direct deps). The kernels are
//! `forbid(unsafe_code)`-clean — they are plain index loops the compiler is free to
//! auto-vectorise (`+simd128` on wasm, NEON/AVX on native), not hand-written SIMD.
//!
//! ## Equivalence contract
//!
//! A [`DataChunk`] is a column-major *transpose* of a slice of rows: column `c`,
//! position `r` holds the id that the row representation stores at `row[r][c]`. The
//! round-trip [`DataChunk::from_rows`] → [`DataChunk::to_rows`] is the identity (see
//! the tests), and `select_numeric` selects exactly the rows a row-at-a-time numeric
//! filter over the same column would keep. This is what lets the vector path be wired
//! into the evaluator later (roadmap T1.3 morsel-driven pipelining) without changing
//! query results.

use sparq_core::dict::Id;
use sparq_core::Graph;

/// A numeric comparison operator for the vectorized FILTER kernel. Mirrors the
/// row evaluator's `NumCmp` (`crate::exec`) so a column filtered here keeps exactly
/// the rows the scalar path would. `NaN` (a non-numeric or unparseable cell) never
/// passes any comparison, matching the scalar `f64` operators (`NaN < t`, `NaN == t`,
/// … are all `false`) and the engine's treatment of a non-numeric FILTER operand as a
/// type error → row excluded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecCmp {
    /// `value > threshold`
    Gt(f64),
    /// `value >= threshold`
    Ge(f64),
    /// `value < threshold`
    Lt(f64),
    /// `value <= threshold`
    Le(f64),
    /// `value == threshold`
    Eq(f64),
}

impl VecCmp {
    /// Tests one f64 against the threshold. `NaN` fails every comparison (the IEEE-754
    /// default for `f64` relational/equality operators).
    #[inline]
    #[must_use]
    pub fn test(self, x: f64) -> bool {
        match self {
            VecCmp::Gt(t) => x > t,
            VecCmp::Ge(t) => x >= t,
            VecCmp::Lt(t) => x < t,
            VecCmp::Le(t) => x <= t,
            VecCmp::Eq(t) => x == t,
        }
    }
}

/// A *selection vector*: the positions (row indices into a [`DataChunk`]) that
/// survived a kernel, in ascending order. This is the canonical vector-engine way to
/// represent a filtered batch without copying the surviving rows out (DuckDB / Velox
/// call it a *selection vector*); the alternative — a packed copy — is
/// [`DataChunk::apply_selection`].
pub type SelVec = Vec<usize>;

/// A column-major batch of id-level solutions: `width` columns, each holding the
/// same number of ids (one per row). The vector-at-a-time analogue of a slice of the
/// row evaluator's `Vec<Row>` — see the module docs for the equivalence contract.
///
/// Invariant: every column has the same length, recorded as [`DataChunk::len`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataChunk {
    /// `columns[c][r]` is the id bound to column `c` in row `r`.
    columns: Vec<Vec<Id>>,
    /// The number of rows (the shared length of every column).
    len: usize,
}

impl DataChunk {
    /// An empty chunk with `width` columns and no rows.
    #[must_use]
    pub fn empty(width: usize) -> Self {
        DataChunk {
            columns: vec![Vec::new(); width],
            len: 0,
        }
    }

    /// Builds a chunk directly from columns. Returns `None` if the columns are not all
    /// `len` long (the [`DataChunk`] invariant). A zero-column input is the unit chunk
    /// with `len` rows (each an empty tuple) — the representation of a result that has
    /// solutions but binds no variables.
    #[must_use]
    pub fn from_columns(columns: Vec<Vec<Id>>, len: usize) -> Option<Self> {
        if columns.iter().any(|c| c.len() != len) {
            return None;
        }
        Some(DataChunk { columns, len })
    }

    /// Transposes a slice of row-major tuples into a column-major chunk. All rows must
    /// have the same `width`; returns `None` otherwise. The inverse of [`Self::to_rows`].
    #[must_use]
    pub fn from_rows(rows: &[Vec<Id>], width: usize) -> Option<Self> {
        if rows.iter().any(|r| r.len() != width) {
            return None;
        }
        let mut columns: Vec<Vec<Id>> =
            (0..width).map(|_| Vec::with_capacity(rows.len())).collect();
        for row in rows {
            for (c, &id) in row.iter().enumerate() {
                columns[c].push(id);
            }
        }
        Some(DataChunk {
            columns,
            len: rows.len(),
        })
    }

    /// Materialises the chunk back into row-major tuples. The inverse of
    /// [`Self::from_rows`]: `to_rows()` of `from_rows(rows, w)` equals `rows` for any
    /// well-formed `rows`.
    #[must_use]
    pub fn to_rows(&self) -> Vec<Vec<Id>> {
        (0..self.len)
            .map(|r| self.columns.iter().map(|col| col[r]).collect())
            .collect()
    }

    /// The number of rows in the chunk.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the chunk has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of columns.
    #[must_use]
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// Read-only access to column `c` (`None` if out of range).
    #[must_use]
    pub fn column(&self, c: usize) -> Option<&[Id]> {
        self.columns.get(c).map(Vec::as_slice)
    }

    /// **Vectorized comparison kernel.** Scans column `c` through the graph's f64 value
    /// cache and returns the [`SelVec`] of row positions whose value passes `cmp`. A
    /// non-numeric / unparseable cell (the cache yields `None`) never passes — the row
    /// evaluator's `ScanCmp::Num` semantics (a FILTER type error excludes the row).
    /// Returns an empty [`SelVec`] if `c` is out of range.
    ///
    /// This is one tight loop over a contiguous column (the compiler is free to
    /// auto-vectorise the value test), versus the row path's per-row dispatch — the
    /// per-row-overhead win the M4 plan targets.
    #[must_use]
    pub fn select_numeric(&self, graph: &Graph, c: usize, cmp: VecCmp) -> SelVec {
        let Some(col) = self.columns.get(c) else {
            return SelVec::new();
        };
        let mut sel = SelVec::new();
        for (r, &id) in col.iter().enumerate() {
            if graph.numeric_value(id).is_some_and(|x| cmp.test(x)) {
                sel.push(r);
            }
        }
        sel
    }

    /// **Selection / gather kernel.** Returns a new chunk containing only the rows named
    /// by `sel` (ascending, in-range positions — as produced by [`Self::select_numeric`]).
    /// Out-of-range positions are skipped. The width is preserved; this is the
    /// vector-at-a-time analogue of dropping filtered rows.
    #[must_use]
    pub fn apply_selection(&self, sel: &[usize]) -> DataChunk {
        let columns: Vec<Vec<Id>> = self
            .columns
            .iter()
            .map(|col| {
                sel.iter()
                    .filter_map(|&r| col.get(r).copied())
                    .collect::<Vec<Id>>()
            })
            .collect();
        let len = columns.first().map_or_else(
            // Zero-column chunk: the row count is the count of in-range selections.
            || sel.iter().filter(|&&r| r < self.len).count(),
            Vec::len,
        );
        DataChunk { columns, len }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;

    /// A graph whose objects are the integer literals `0..n` (subject `ex:s{i}`,
    /// predicate `ex:age`), and the parallel list of object ids in value order, so a
    /// chunk built from the object column can be filtered numerically and checked
    /// against a hand-computed expectation.
    fn ages_graph(n: usize) -> (Graph, Vec<Id>) {
        let mut nt = String::new();
        for i in 0..n {
            nt.push_str(&format!(
                "<http://ex/s{i}> <http://ex/age> \"{i}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n"
            ));
        }
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        let obj_ids: Vec<Id> = (0..n)
            .map(|i| {
                let lit = oxrdf::Literal::new_typed_literal(
                    i.to_string(),
                    oxrdf::NamedNode::new("http://www.w3.org/2001/XMLSchema#integer").unwrap(),
                );
                g.id_of(&oxrdf::Term::Literal(lit))
                    .expect("integer literal interned")
            })
            .collect();
        (g, obj_ids)
    }

    #[test]
    fn from_columns_rejects_ragged() {
        assert!(DataChunk::from_columns(vec![vec![1, 2], vec![3]], 2).is_none());
        let c = DataChunk::from_columns(vec![vec![1, 2], vec![3, 4]], 2).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.width(), 2);
    }

    #[test]
    fn from_rows_rejects_ragged() {
        assert!(DataChunk::from_rows(&[vec![1, 2], vec![3]], 2).is_none());
    }

    #[test]
    fn rows_roundtrip_is_identity() {
        let rows = vec![vec![10, 20, 30], vec![11, 21, 31], vec![12, 22, 32]];
        let chunk = DataChunk::from_rows(&rows, 3).unwrap();
        assert_eq!(chunk.len(), 3);
        assert_eq!(chunk.width(), 3);
        assert_eq!(chunk.column(1).unwrap(), &[20, 21, 22]);
        assert_eq!(chunk.to_rows(), rows);
    }

    #[test]
    fn empty_and_zero_width() {
        let e = DataChunk::empty(2);
        assert!(e.is_empty());
        assert_eq!(e.width(), 2);
        assert_eq!(e.to_rows(), Vec::<Vec<Id>>::new());

        // Zero-column unit chunk: rows that bind no variables.
        let unit = DataChunk::from_columns(vec![], 3).unwrap();
        assert_eq!(unit.len(), 3);
        assert_eq!(unit.width(), 0);
        assert_eq!(unit.to_rows(), vec![vec![], vec![], vec![]]);
    }

    #[test]
    fn veccmp_nan_never_passes() {
        for cmp in [
            VecCmp::Gt(0.0),
            VecCmp::Ge(0.0),
            VecCmp::Lt(0.0),
            VecCmp::Le(0.0),
            VecCmp::Eq(0.0),
        ] {
            assert!(!cmp.test(f64::NAN), "{cmp:?} should reject NaN");
        }
    }

    #[test]
    fn select_numeric_matches_scalar_semantics() {
        let n = 64;
        let (g, obj_ids) = ages_graph(n);
        // One-column chunk over the object ids (the integer values 0..n).
        let chunk = DataChunk::from_columns(vec![obj_ids.clone()], n).unwrap();

        for cmp in [
            VecCmp::Gt(30.0),
            VecCmp::Ge(30.0),
            VecCmp::Lt(10.0),
            VecCmp::Le(10.0),
            VecCmp::Eq(42.0),
        ] {
            let sel = chunk.select_numeric(&g, 0, cmp);
            // Scalar reference: the exact rows a row-at-a-time numeric FILTER keeps.
            let expected: Vec<usize> = (0..n)
                .filter(|&r| g.numeric_value(obj_ids[r]).is_some_and(|x| cmp.test(x)))
                .collect();
            assert_eq!(sel, expected, "selection mismatch for {cmp:?}");

            // Gather, then verify every surviving value passes and the count is right.
            let filtered = chunk.apply_selection(&sel);
            assert_eq!(filtered.len(), expected.len());
            for &id in filtered.column(0).unwrap() {
                assert!(g.numeric_value(id).is_some_and(|x| cmp.test(x)));
            }
        }
    }

    #[test]
    fn select_numeric_out_of_range_column_is_empty() {
        let (g, _ids) = ages_graph(4);
        let chunk = DataChunk::empty(1);
        assert!(chunk.select_numeric(&g, 5, VecCmp::Gt(0.0)).is_empty());
    }

    #[test]
    fn apply_selection_preserves_columns_and_skips_out_of_range() {
        let rows = vec![vec![1, 100], vec![2, 200], vec![3, 300]];
        let chunk = DataChunk::from_rows(&rows, 2).unwrap();
        // Keep rows 0 and 2; an out-of-range index (9) is ignored.
        let out = chunk.apply_selection(&[0, 2, 9]);
        assert_eq!(out.to_rows(), vec![vec![1, 100], vec![3, 300]]);
    }

    #[test]
    fn apply_selection_on_zero_width_counts_in_range() {
        let unit = DataChunk::from_columns(vec![], 3).unwrap();
        let out = unit.apply_selection(&[0, 2, 9]); // 9 is out of range
        assert_eq!(out.width(), 0);
        assert_eq!(out.len(), 2);
    }
}
