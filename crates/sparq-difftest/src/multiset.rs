//! The solution-**multiset** comparator and the `ORDER BY` sort-key-equivalence-class comparator,
//! over the neutral `{var → term}` model. Engine-independent. [OPUS-4.8] sq-qcnn.4
//!
//! A SPARQL `SELECT` result is a **bag** of solution mappings (`DISTINCT` is not implicit, so
//! duplicates are significant): the un-ordered comparison is exact **multiset equality**, keyed by the
//! value-canonical form of each term ([`crate::term::canonical_key`]). This subsumes the old
//! cardinality-only cross-check (equal multisets ⇒ equal cardinality, *plus* the values).
//!
//! `ORDER BY` results are compared **up to permutation within each maximal run of sort-key-equal
//! rows**, because SPARQL `ORDER BY` is a *partial* order — rows equal on all sort keys may appear in
//! any relative order across engines. Comparing the full sequence element-for-element would be wrong
//! in general (it is only safe when the sort key is a total order over the projected variables).
//! "Sort-key-equal" here is SPARQL *ordering* equality ([`crate::term::order_equiv_key`], which
//! folds numeric promotion), not the bag key — keying a run by the bag key would split the tie class
//! `1`^^`xsd:integer` / `1.0`^^`xsd:decimal` and reject a conforming permutation of it.
//!
//! Run partitioning is not a *complete* `ORDER BY` oracle, because the tie relation it partitions by
//! is not an equivalence relation once `f64` promotion is in play. Refining a tie relation can only
//! make the comparison reject, never wrongly accept — so [`order_by_compare`] keeps every acceptance
//! as [`OrderVerdict::Equal`], and downgrades a **rejection** on such a column to the third verdict
//! [`OrderVerdict::TieUnmodelled`], **failing closed** instead of emitting the false failure the
//! split run would otherwise produce.
//!
//! These are pure comparators over already-normalised results. `crates/sparq-bench`'s Oxigraph
//! fuzzer wires them to a live oracle (`sq-qcnn.5`): its `check_bindings` decides the `SELECT` bag
//! with [`multiset_equal`], and its `check_ordered` decides an `ORDER BY` sequence with
//! [`order_by_compare`] over every projected variable — falling back to a counted skip where a
//! `LIMIT` truncates a sequence whose sort key is not total over the projection, and to a second
//! counted skip on a `TieUnmodelled` column.

use crate::json::Solution;
use crate::term::{canonical_key, order_equiv_key, order_key_splits_tie, Term};

/// A solution's value-canonical multiset key: the sorted `(var, term-key)` pairs held **structurally**
/// (a `Vec` of pairs, not a delimiter-joined string). Delimiter-joining would be collision-prone — a
/// literal lexical form can contain *any* byte, including a chosen separator, so a differently-shaped
/// solution could forge an identical joined string and read as "equal". A vector of pairs has no
/// separator to forge: two solutions share a key iff they bind the same variables to the same
/// value-canonical terms.
type SolutionKey = Vec<(String, String)>;

/// A sort-variable-projection key: one `Option` per sort variable (`None` marks an unbound sort var, so
/// bound-vs-unbound stays a distinct sort key), held structurally for the same anti-collision reason as
/// [`SolutionKey`]. Drives the `ORDER BY` run partitioning.
///
/// Keyed by [`order_equiv_key`], **not** by [`canonical_key`]: a run is a set of rows `ORDER BY`
/// leaves tied, and SPARQL ties numeric operands after promotion, so `1`^^`xsd:integer` and
/// `1.0`^^`xsd:decimal` belong to one run even though they are different RDF terms (and so key
/// differently in a [`SolutionKey`], which is a *bag* key and must keep them apart). Ties this key
/// still cannot express are handled by failing closed, not by keying — see [`tie_unmodelled`].
type SortKey = Vec<Option<String>>;

fn solution_key(sol: &Solution) -> SolutionKey {
    // `Solution` is a BTreeMap, so iteration is already in sorted-variable order.
    sol.iter()
        .map(|(var, term)| (var.clone(), canonical_key(term)))
        .collect()
}

fn sort_key(sol: &Solution, sort_vars: &[&str]) -> SortKey {
    sort_vars
        .iter()
        .map(|v| sol.get(*v).map(order_equiv_key))
        .collect()
}

/// Exact **multiset (bag) equality** of two solution sequences, order-insensitive: equal iff they
/// contain the same solutions with the same multiplicities (each solution keyed value-canonically).
pub fn multiset_equal(a: &[Solution], b: &[Solution]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut ka: Vec<SolutionKey> = a.iter().map(solution_key).collect();
    let mut kb: Vec<SolutionKey> = b.iter().map(solution_key).collect();
    ka.sort_unstable();
    kb.sort_unstable();
    ka == kb
}

/// The verdict of [`order_by_compare`]. Three outcomes, not two: "the comparator cannot decide this
/// input" is *not* a divergence, and reporting it as one would be a false failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderVerdict {
    /// The sequences agree up to permutation within each sort-key tie run.
    Equal,
    /// They genuinely disagree: a different bag, or a reordering across tie runs.
    Differs,
    /// **Undecidable, fail closed.** The run partition rejected, but a sort column holds a pair that
    /// `ORDER BY` leaves TIED and [`order_equiv_key`] keys apart — so the rejection may be nothing
    /// but a split of a run the spec allows either engine to permute. The tie relation there is not
    /// transitive (`2^70+1` and `2^70+2` each tie with the double `2^70` yet are strictly ordered
    /// against each other), so no key — finer or coarser — models it, and this input simply has no
    /// honest verdict. The caller must count it as a **skip**, never fold it into either
    /// other verdict: as `Equal` it would hide a real ordering bug, as `Differs` it is the false
    /// failure this variant exists to prevent.
    TieUnmodelled,
}

/// Compare two `ORDER BY` result sequences up to permutation within each sort-key-equivalence class.
///
/// Both sequences are partitioned into maximal runs of rows sharing a `sort_key` (over `sort_vars`);
/// the runs must line up (same count, same order, matching sort keys) and each corresponding run-pair
/// must be **multiset-equal**. With `sort_vars` empty this degrades to [`multiset_equal`].
///
/// The three verdicts are decided in this order, which is what keeps the skip **narrow**:
///
/// 1. A *bag* difference is [`OrderVerdict::Differs`] — decidable whatever latitude `ORDER BY`
///    leaves, so it is never traded away for a skip.
/// 2. If the run partition **accepts**, the answer is [`OrderVerdict::Equal`] even on a column with
///    an unmodellable tie. The partition is a *refinement* of the true tie relation, so accepting
///    under it implies accepting under the true (coarser) one: merging adjacent accepted runs
///    preserves both the ordering and the multiset match. Nothing is hidden.
/// 3. Only a **rejection** is ambiguous — it may be the real divergence, or it may be the artifact
///    of a split tie run — and only then is the column tested for an unmodellable tie and the
///    verdict downgraded to [`OrderVerdict::TieUnmodelled`].
///
/// Note: with a *total* sort key over all projected variables, each run is a single row and this
/// becomes exact sequence equality (the strict option the harness can force with a tiebreaker).
pub fn order_by_compare(a: &[Solution], b: &[Solution], sort_vars: &[&str]) -> OrderVerdict {
    if !multiset_equal(a, b) {
        return OrderVerdict::Differs;
    }
    if runs_agree(a, b, sort_vars) {
        return OrderVerdict::Equal;
    }
    if tie_unmodelled(a, sort_vars) || tie_unmodelled(b, sort_vars) {
        return OrderVerdict::TieUnmodelled;
    }
    OrderVerdict::Differs
}

/// Whether the two sequences' maximal sort-key runs line up (same count, same order, matching sort
/// keys) with multiset-equal contents.
fn runs_agree(a: &[Solution], b: &[Solution], sort_vars: &[&str]) -> bool {
    let runs_a = runs(a, sort_vars);
    let runs_b = runs(b, sort_vars);
    if runs_a.len() != runs_b.len() {
        return false;
    }
    runs_a.iter().zip(&runs_b).all(|(&(sa, ea), &(sb, eb))| {
        // Corresponding runs must share the sort key AND be equal as multisets (which also enforces
        // equal run length).
        sort_key(&a[sa], sort_vars) == sort_key(&b[sb], sort_vars)
            && multiset_equal(&a[sa..ea], &b[sb..eb])
    })
}

/// Whether any sort column of `seq` holds a pair that `ORDER BY` ties but [`order_equiv_key`] keys
/// apart ([`crate::term::order_key_splits_tie`]) — the fail-closed trigger.
///
/// Checked over the WHOLE column rather than within a run: the pair is exactly the one the run
/// partition would put in two different runs, so looking only inside a run could never see it. The
/// scan is quadratic in the number of DISTINCT keys per column, which is what bounds it — a column
/// of one repeated value costs one comparison — and it runs only on the rejection path.
fn tie_unmodelled(seq: &[Solution], sort_vars: &[&str]) -> bool {
    for var in sort_vars {
        let mut distinct: Vec<(String, &Term)> = Vec::new();
        for term in seq.iter().filter_map(|sol| sol.get(*var)) {
            let key = order_equiv_key(term);
            if distinct.iter().any(|(seen, _)| *seen == key) {
                continue;
            }
            if distinct
                .iter()
                .any(|(_, seen)| order_key_splits_tie(seen, term))
            {
                return true;
            }
            distinct.push((key, term));
        }
    }
    false
}

/// Partition a sequence into maximal `[start, end)` runs of consecutive rows with the same sort key.
fn runs(seq: &[Solution], sort_vars: &[&str]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < seq.len() {
        let key = sort_key(&seq[i], sort_vars);
        let mut j = i + 1;
        while j < seq.len() && sort_key(&seq[j], sort_vars) == key {
            j += 1;
        }
        out.push((i, j));
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::XSD_STRING;

    const XSD_INT: &str = "http://www.w3.org/2001/XMLSchema#integer";
    const XSD_DBL: &str = "http://www.w3.org/2001/XMLSchema#double";

    fn int(n: &str) -> Term {
        Term::Literal {
            lexical: n.to_string(),
            datatype: XSD_INT.to_string(),
            lang: None,
        }
    }
    fn s(v: &str) -> Term {
        Term::Literal {
            lexical: v.to_string(),
            datatype: XSD_STRING.to_string(),
            lang: None,
        }
    }
    fn sol(pairs: &[(&str, Term)]) -> Solution {
        pairs.iter().map(|(k, t)| (k.to_string(), t.clone())).collect()
    }

    #[test]
    fn multiset_equal_is_order_insensitive_and_value_level() {
        let a = vec![sol(&[("x", int("1"))]), sol(&[("x", int("2"))])];
        let b = vec![sol(&[("x", int("2"))]), sol(&[("x", int("1"))])];
        assert!(multiset_equal(&a, &b), "order must not matter");
        // value-level: same cardinality, different value -> NOT equal (the whole point).
        let c = vec![sol(&[("x", int("1"))]), sol(&[("x", int("3"))])];
        assert!(!multiset_equal(&a, &c));
        // lexical variance of the same value collapses (value regime).
        let d = vec![sol(&[("x", int("01"))]), sol(&[("x", int("2"))])];
        assert!(multiset_equal(&a, &d));
        // multiplicities matter (bag, not set).
        let dup = vec![sol(&[("x", int("1"))]), sol(&[("x", int("1"))])];
        assert!(!multiset_equal(&a, &dup));
        // bound-vs-unbound is a distinct solution.
        let bound = vec![sol(&[("x", int("1")), ("y", s("z"))])];
        let unbound = vec![sol(&[("x", int("1"))])];
        assert!(!multiset_equal(&bound, &unbound));
        // different length.
        assert!(!multiset_equal(&a, &a[..1]));
    }

    #[test]
    fn multiset_equal_is_not_fooled_by_delimiter_bytes_in_a_lexical() {
        // Regression for the old delimiter-joined string key: a literal lexical may contain ANY byte,
        // including the historical record/group/unit separators (\u{1e}/\u{1d}/\u{1f}). These two
        // solutions are genuinely different SHAPES (one variable vs two), yet the single lexical of `a`
        // is crafted so that the old `join`-based key of `a` stringified *identically* to that of `b`.
        // With a structural key they must compare UNEQUAL.
        let key_n = canonical_key(&s("N")); // the value-canonical key `b` would emit for its ?y = "N"
        let forged = format!("M\u{1e}b\u{1d}{key_n}");
        let a = vec![sol(&[("a", s(&forged))])];
        let b = vec![sol(&[("a", s("M")), ("b", s("N"))])];
        // Equal solution COUNT, so this exercises the key comparison rather than the length short-circuit.
        assert_eq!(a.len(), b.len());
        assert!(
            !multiset_equal(&a, &b),
            "a crafted literal lexical must not forge a differently-shaped solution's key"
        );
    }

    #[test]
    fn order_by_equal_permutes_within_ties() {
        // ORDER BY ?k; rows tied on ?k may permute their ?v across engines.
        let a = vec![
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", int("1")), ("v", s("b"))]),
            sol(&[("k", int("2")), ("v", s("c"))]),
        ];
        let b = vec![
            sol(&[("k", int("1")), ("v", s("b"))]),
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", int("2")), ("v", s("c"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &b, &["k"]),
            OrderVerdict::Equal,
            "tie run may permute"
        );
        // but the run CONTENTS must still match as a multiset.
        let c = vec![
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", int("1")), ("v", s("x"))]),
            sol(&[("k", int("2")), ("v", s("c"))]),
        ];
        assert_eq!(order_by_compare(&a, &c, &["k"]), OrderVerdict::Differs);
        // cross-run reordering is a real order violation -> NOT equal.
        let d = vec![
            sol(&[("k", int("2")), ("v", s("c"))]),
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", int("1")), ("v", s("b"))]),
        ];
        assert_eq!(order_by_compare(&a, &d, &["k"]), OrderVerdict::Differs);
        // differing lengths.
        assert_eq!(order_by_compare(&a, &a[..2], &["k"]), OrderVerdict::Differs);
        // empty sort key degrades to multiset equality.
        assert_eq!(order_by_compare(&a, &b, &[]), OrderVerdict::Equal);
    }

    #[test]
    fn order_by_ties_follow_numeric_promotion_not_the_bag_key() {
        const XSD_DEC: &str = "http://www.w3.org/2001/XMLSchema#decimal";
        let dec = |n: &str| Term::Literal {
            lexical: n.to_string(),
            datatype: XSD_DEC.to_string(),
            lang: None,
        };
        // ORDER BY ?k over `1`^^xsd:integer and `1.0`^^xsd:decimal: SPARQL promotes, so neither is
        // `<` the other — one tie run, and the two rows may permute across engines.
        let a = vec![
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", dec("1.0")), ("v", s("b"))]),
            sol(&[("k", dec("1.5")), ("v", s("c"))]),
        ];
        let permuted = vec![
            sol(&[("k", dec("1.0")), ("v", s("b"))]),
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", dec("1.5")), ("v", s("c"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &permuted, &["k"]),
            OrderVerdict::Equal,
            "integer 1 and decimal 1.0 are ONE sort-key class; permuting them is conforming"
        );
        // The bag key does keep those two rows distinct — this is what makes the run partitioning a
        // separate keying regime rather than a reuse of `solution_key`.
        assert_ne!(canonical_key(&int("1")), canonical_key(&dec("1.0")));
        // A nearby UNEQUAL numeric is still its own run, so a cross-run swap remains a violation
        // even though every key now folds across datatypes.
        let across_runs = vec![
            sol(&[("k", dec("1.5")), ("v", s("c"))]),
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", dec("1.0")), ("v", s("b"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &across_runs, &["k"]),
            OrderVerdict::Differs
        );
        // And the tie run's CONTENTS still have to match as a multiset.
        let wrong_in_tie = vec![
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", dec("1.0")), ("v", s("x"))]),
            sol(&[("k", dec("1.5")), ("v", s("c"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &wrong_in_tie, &["k"]),
            OrderVerdict::Differs
        );
    }

    /// A tie created by LOSSY `f64` promotion is one the sort key cannot express, so the comparator
    /// must FAIL CLOSED on that column instead of rejecting a legal permutation of the tie.
    ///
    /// `2^70` is exact as an `f64`; the integer `2^70+1` promotes onto it, so `ORDER BY` leaves the
    /// two tied and either engine may emit them in either order. Keying them apart (which is forced
    /// — `2^70+2` also ties with the double yet is strictly greater than `2^70+1`, so the relation
    /// is not transitive and no key models it) would split that tie run and report a divergence
    /// where the spec permits both sequences.
    #[test]
    fn a_lossy_promotion_tie_fails_closed_instead_of_reporting_a_false_divergence() {
        let dbl = |n: &str| Term::Literal {
            lexical: n.to_string(),
            datatype: XSD_DBL.to_string(),
            lang: None,
        };
        let i70p1 = int("1180591620717411303425"); // 2^70 + 1
        let d70 = dbl("1180591620717411303424"); // 2^70, exact as an f64
        let a = vec![
            sol(&[("k", i70p1.clone()), ("v", s("a"))]),
            sol(&[("k", d70.clone()), ("v", s("b"))]),
        ];
        let permuted = vec![
            sol(&[("k", d70.clone()), ("v", s("b"))]),
            sol(&[("k", i70p1.clone()), ("v", s("a"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &permuted, &["k"]),
            OrderVerdict::TieUnmodelled,
            "a permutation of a promotion tie must NOT be reported as a divergence"
        );
        // Fail-closed is not blanket acceptance: the bag is still decidable, so a WRONG row in the
        // same undecidable column is still caught.
        let wrong_row = vec![
            sol(&[("k", d70.clone()), ("v", s("b"))]),
            sol(&[("k", i70p1.clone()), ("v", s("zzz"))]),
        ];
        assert_eq!(
            order_by_compare(&a, &wrong_row, &["k"]),
            OrderVerdict::Differs
        );
        // And the skip is NARROW: a nearby column with no promotion hazard keeps its full strength,
        // so swapping two strictly-ordered rows is still rejected.
        let strict = vec![
            sol(&[("k", int("1")), ("v", s("a"))]),
            sol(&[("k", dbl("2.0")), ("v", s("b"))]),
        ];
        let swapped = vec![
            sol(&[("k", dbl("2.0")), ("v", s("b"))]),
            sol(&[("k", int("1")), ("v", s("a"))]),
        ];
        assert_eq!(
            order_by_compare(&strict, &swapped, &["k"]),
            OrderVerdict::Differs,
            "1 < 2.0 under promotion too — this is a real ordering bug, not latitude"
        );
        // The skip fires only where the split actually changes the verdict: two sequences the run
        // partition ACCEPTS are Equal even on the hazardous column, because refining a tie relation
        // can only reject, never wrongly accept.
        assert_eq!(order_by_compare(&a, &a, &["k"]), OrderVerdict::Equal);
        // ... and a sort variable that carries no hazard keeps its verdict too.
        let by_v = vec![
            sol(&[("k", i70p1), ("v", s("a"))]),
            sol(&[("k", d70), ("v", s("b"))]),
        ];
        assert_eq!(order_by_compare(&by_v, &by_v, &["v"]), OrderVerdict::Equal);
    }
}
