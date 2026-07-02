//! The shared SPARQL **term total order** — the `ORDER BY` / `MIN`/`MAX`-fallback
//! lenient comparison and the recursive RDF-1.2 triple-term ordering.
//!
//! This is the SPARQL engine's `compare_values` total order, **moved here** from
//! `sparq-engine::exec`, generalised over a tiny [`CompareTerm`] trait rather than
//! the engine's private `Value` enum + `Term` / `Num` / `Timeline` subsystem. The
//! engine implements [`CompareTerm`] for its `Value` (the impl is a set of zero-cost
//! wrappers over its existing `value_str` / numeric / strict-typed-compare helpers)
//! and calls [`compare_terms`]; a reasoner that materialises ordered solutions (RIF
//! `order`, an EL/QL `ORDER BY` over an entailed answer set) can implement the SAME
//! trait for its own term representation and reuse this body. Neither consumer pays a
//! vtable (`research/shared-eval-substrate.md` §2.1, §2.3 — Phase 4 of epic sq-qonbz).
//!
//! # What lives here vs. stays engine-private
//!
//! The substrate holds the **ordering ALGORITHM**: the SPARQL class precedence
//! (error/unbound < blank < IRI < literal < triple-term), the numeric-aware /
//! strict-typed / string-fallback arm selection within the literal class, and the
//! recursive component-wise triple-term order. It deliberately does **not** hoist the
//! engine's `Value` enum, its `LitKind` literal-family classifier, or its
//! `value_compare_strict` typed/temporal comparison: those are reused by the engine's
//! relational `<` / `>` / `=` operators too (not only the ORDER BY total order), and
//! are tightly coupled to `oxrdf::Term`, so they stay engine-resident and are surfaced
//! to this algorithm through the trait's [`strict_cmp`](CompareTerm::strict_cmp) /
//! [`as_f64`](CompareTerm::as_f64) hooks. This is the same "generic seam" the `join`
//! kernels use for `JoinKeys` and `numeric` uses for `Num`: the shared *algorithm* is
//! generic over a small abstraction the consumer implements, monomorphised at the call
//! site.
//!
//! # Zero-overhead intent (the load-bearing contract)
//!
//! [`compare_terms`] is generic over `T: CompareTerm` — a **generic type parameter**,
//! NOT a trait object. There is NO `Box<dyn>` / `&dyn` / vtable between the algorithm
//! and the term observations it makes; the compiler emits one specialised, inlinable
//! body per call site. Every item carries `#[inline]`, so with the workspace LTO
//! profile the engine's `ORDER BY` / sort / range-filter hot loops keep codegen
//! identical to the pre-move `compare_values`. This is verified by the W3C SPARQL
//! conformance floor staying bit-identical and the structural `no-dyn-dispatch` gate
//! (`scripts/check-no-dyn-dispatch.py`) listing this file in its hot-path set.

use std::cmp::Ordering;

/// The SPARQL total-order **class** of a term: the top-level precedence buckets the
/// `ORDER BY` order ranks by, *before* any within-class value comparison.
///
/// SPARQL 1.1 orders `unbound < blank node < IRI < literal`; the SPARQL 1.2 total-order
/// extension places **triple terms after literals**. The discriminants are the exact
/// ranks the engine's `compare_values` used (`error`/`unbound` = 0, blank = 1, IRI = 2,
/// literal = 3, triple = 4), so cross-class ordering is byte-identical to pre-move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermClass {
    /// An unbound variable or a SPARQL type error — sorts first.
    ErrorOrUnbound = 0,
    /// A blank node.
    Blank = 1,
    /// An IRI (named node).
    Iri = 2,
    /// Any literal (including a computed numeric / boolean).
    Literal = 3,
    /// An RDF-1.2 quoted triple term — sorts after literals.
    Triple = 4,
}

/// The minimal observation surface [`compare_terms`] needs from a term to compute the
/// SPARQL total order, **without** the substrate knowing about the engine's `Value`
/// enum, `oxrdf::Term`, or its temporal subsystem.
///
/// The engine implements this for its `Value` as zero-cost wrappers over its existing
/// helpers (`value_str`, the `as_num` f64 coercion, `value_compare_strict`, and the
/// triple decomposition). A reasoner implements it for its own ordered-term type. The
/// trait is a **monomorphisation seam**, never used as `dyn CompareTerm` on a hot path.
pub trait CompareTerm: Sized {
    /// The term's top-level SPARQL ordering class (the cross-class precedence bucket).
    fn term_class(&self) -> TermClass;

    /// The term's lexical string form for the deterministic within-class fallback
    /// (the literal string arm, and the blank-node / IRI comparison): the literal
    /// lexical value, the IRI string, or the blank-node label. `None` for an
    /// unbound / error term (which never reaches a within-class string compare).
    ///
    /// This is the engine's `value_str`: the same form ORDER BY used to break ties
    /// across mixed literal datatypes, so the total order stays deterministic.
    fn value_str(&self) -> Option<String>;

    /// The term's numeric value as `f64` for the **numeric-aware** literal arm — the
    /// lenient ORDER BY coercion that lets `"1"^^xsd:integer` and `"1.5"^^xsd:double`
    /// order by value across the numeric tower. `None` for a non-numeric literal.
    ///
    /// This mirrors the engine's `as_num`: numerics (and booleans, as 0.0/1.0) order
    /// by value, everything else falls through to [`strict_cmp`](Self::strict_cmp)
    /// then the string form.
    fn as_f64(&self) -> Option<f64>;

    /// An EXACT recheck of two **numeric** literals for the f64-collapse case. The lenient
    /// numeric arm coerces both operands to `f64` (see [`as_f64`](Self::as_f64)); f64
    /// rounding is MONOTONIC, so it can only COLLAPSE two distinct numeric values to Equal,
    /// never flip a `Less`/`Greater`. So [`compare_terms`] calls this **only** when the f64
    /// arm reports Equal, to recover the true order for distinct integers beyond 2^53 and
    /// high-precision decimals that share one f64.
    ///
    /// Returns `Some(ordering)` only when BOTH terms are numeric and value-exactly
    /// comparable via the exact numeric tower (`xsd:integer` / `xsd:decimal`); `None` when
    /// either operand is non-numeric, is an inexact tier (`xsd:float`/`xsd:double`, whose
    /// value IS its f64), or the exact comparison cannot decide — the collapsed f64 verdict
    /// then stands. A purely symbolic consumer with no exact numeric tier returns `None`.
    ///
    /// This mirrors the engine's relational `=`/`<` recheck and its `MIN`/`MAX` value
    /// comparison — the same value-exact numeric order, surfaced to the shared total order
    /// so `ORDER BY` / `MIN` / `MAX` agree with them rather than collapsing an f64 tie to
    /// Equal. [OPUS-4.8] sq-rikm7
    fn exact_cmp(&self, other: &Self) -> Option<Ordering>;

    /// The **strict** value comparison for same-family typed literals the numeric arm
    /// cannot decide — principally `xsd:dateTime` / `xsd:date` by timeline, and the
    /// lenient same-tag / same-other-XSD lexical orders. `Some(ordering)` only when the
    /// operands are value-comparable; `None` when they are not (the algorithm then
    /// falls back to the string form, exactly as `compare_values` did).
    ///
    /// This is the engine's `value_compare_strict`; it stays engine-resident because it
    /// is also driven by the relational `<` / `>` operators and is coupled to `oxrdf`.
    fn strict_cmp(&self, other: &Self) -> Option<Ordering>;

    /// If the term is an RDF-1.2 quoted triple, its `(subject, predicate, object)`
    /// components **as the same term type**, so [`compare_terms`] can recurse through
    /// nesting and order triple terms component-wise under this very order. `None` for
    /// a non-triple term.
    ///
    /// The predicate is surfaced as a term whose [`term_class`](Self::term_class) is
    /// [`TermClass::Iri`], so the generic recursion compares it by IRI string — byte
    /// identical to the engine's pre-move `predicate.as_str().cmp(...)`.
    fn triple_parts(&self) -> Option<[Self; 3]>;
}

/// The SPARQL **lenient total order** for `ORDER BY` (and the `MIN`/`MAX` fallback),
/// generic over any [`CompareTerm`].
///
/// SPARQL orders unbound/error < blank nodes < IRIs < literals < triple terms, then
/// within each class: blanks / IRIs by their string form; literals by numeric value
/// when both are numeric, else by the strict same-family value order (dateTime/date by
/// timeline, same-tag/same-other-XSD lexically) when decidable, else by lexical string
/// form (which keeps the order deterministic across mixed literal types); triple terms
/// component-wise (subject, then predicate, then object) recursively under this order.
///
/// Returns `None` only when a within-class string fallback is needed but a term has no
/// string form (an unbound/error reaching a literal compare) — matching the engine's
/// `compare_values`, whose callers map `None` to `Ordering::Equal`.
///
/// This is the body **moved verbatim** from `sparq-engine::exec::compare_values`, with
/// the concrete `Value` matches replaced by the trait observations. It is generic (not
/// `dyn`), so it monomorphises and inlines into each caller with no vtable.
#[inline]
pub fn compare_terms<T: CompareTerm>(x: &T, y: &T) -> Option<Ordering> {
    let (ca, cb) = (x.term_class(), y.term_class());
    if ca != cb {
        return Some(ca.cmp(&cb));
    }
    match ca {
        TermClass::ErrorOrUnbound => Some(Ordering::Equal),
        TermClass::Blank | TermClass::Iri => Some(x.value_str()?.cmp(&y.value_str()?)),
        // Triple terms order component-wise (subject, predicate, then object under
        // this same total order, recursing through nesting).
        TermClass::Triple => {
            let (Some(a), Some(b)) = (x.triple_parts(), y.triple_parts()) else {
                return None;
            };
            let [as_, ap, ao] = a;
            let [bs, bp, bo] = b;
            let s = compare_terms(&as_, &bs)?;
            if s != Ordering::Equal {
                return Some(s);
            }
            let p = compare_terms(&ap, &bp)?;
            if p != Ordering::Equal {
                return Some(p);
            }
            compare_terms(&ao, &bo)
        }
        TermClass::Literal => {
            if let (Some(a), Some(b)) = (x.as_f64(), y.as_f64()) {
                let ord = a.partial_cmp(&b);
                // f64-collapse EXACT recheck. The lenient numeric arm coerces both operands
                // to f64, whose rounding is MONOTONIC: it can only COLLAPSE two distinct
                // numeric values to Equal, never flip a Less/Greater. So — and ONLY — when
                // the f64 arm reports Equal, recheck the pair exactly; a decisive exact
                // ordering (distinct integers beyond 2^53, or high-precision decimals that
                // share one f64) overrides the collapsed verdict. This makes ORDER BY / MIN /
                // MAX agree with the engine's relational =/< (`cmp_expr`) and MIN/MAX value
                // comparison (`num_compare`), which already recheck. Perf-neutral: the exact
                // work happens only on an f64 tie. [OPUS-4.8] sq-rikm7
                if ord == Some(Ordering::Equal) {
                    if let Some(exact) = x.exact_cmp(y) {
                        return Some(exact);
                    }
                }
                return ord;
            }
            // dateTime/date (and same-tag / same-other-XSD) order strictly when comparable.
            if let Some(o) = x.strict_cmp(y) {
                return Some(o);
            }
            Some(x.value_str()?.cmp(&y.value_str()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    /// A minimal test-only term, exercising every arm of [`compare_terms`] without
    /// pulling the engine's `Value` (the engine's own conformance + ORDER BY suites
    /// exercise the real `Value` impl; this proves the algorithm in isolation, one
    /// direct test per public item per the coverage-ratchet rule).
    #[derive(Clone, Debug)]
    enum T {
        ErrorOrUnbound,
        Blank(String),
        Iri(String),
        /// A numeric literal (orders by f64 value; no exact tier — models `xsd:double`).
        NumLit(f64),
        /// A numeric literal carrying BOTH its lenient `f` (the possibly-collapsing f64)
        /// AND its EXACT value as a scaled integer `mant * 10^-scale`, so the f64-collapse
        /// recheck arm is exercised: two of these can share an `f` yet differ exactly —
        /// modelling an `xsd:integer` beyond 2^53 (`scale` 0) and a high-precision
        /// `xsd:decimal` (`scale > 0`). [OPUS-4.8] sq-rikm7
        ExactNum { f: f64, mant: i128, scale: u32 },
        /// A plain-string literal (orders by string form; `strict_cmp` decides equal
        /// strings, else `None` → string fallback).
        StrLit(String),
        /// A literal that is comparable only via `strict_cmp` returning the given
        /// ordering versus another `Strict` (models the dateTime/timeline arm).
        Strict(i64),
        /// A triple term `(s, p, o)`.
        Triple(Box<T>, Box<T>, Box<T>),
    }

    impl CompareTerm for T {
        fn term_class(&self) -> TermClass {
            match self {
                T::ErrorOrUnbound => TermClass::ErrorOrUnbound,
                T::Blank(_) => TermClass::Blank,
                T::Iri(_) => TermClass::Iri,
                T::NumLit(_) | T::ExactNum { .. } | T::StrLit(_) | T::Strict(_) => TermClass::Literal,
                T::Triple(..) => TermClass::Triple,
            }
        }
        fn value_str(&self) -> Option<String> {
            match self {
                T::ErrorOrUnbound => None,
                T::Blank(s) | T::Iri(s) | T::StrLit(s) => Some(s.clone()),
                T::NumLit(n) => Some(n.to_string()),
                T::ExactNum { f, .. } => Some(f.to_string()),
                T::Strict(n) => Some(n.to_string()),
                T::Triple(..) => None,
            }
        }
        fn as_f64(&self) -> Option<f64> {
            match self {
                T::NumLit(n) => Some(*n),
                T::ExactNum { f, .. } => Some(*f),
                _ => None,
            }
        }
        fn exact_cmp(&self, other: &Self) -> Option<Ordering> {
            match (self, other) {
                // Align both scaled integers to the common (max) scale, then compare the
                // mantissas exactly (mirrors the substrate `Dec::cmp` the engine's numeric
                // tower uses). Only `ExactNum` pairs have an exact tier; everything else
                // (a plain f64 `NumLit`, a non-numeric literal) returns `None` and keeps
                // the collapsed f64 verdict. [OPUS-4.8] sq-rikm7
                (T::ExactNum { mant: am, scale: asc, .. }, T::ExactNum { mant: bm, scale: bsc, .. }) => {
                    let scale = (*asc).max(*bsc);
                    let a = am.checked_mul(10i128.checked_pow(scale - asc)?)?;
                    let b = bm.checked_mul(10i128.checked_pow(scale - bsc)?)?;
                    Some(a.cmp(&b))
                }
                _ => None,
            }
        }
        fn strict_cmp(&self, other: &Self) -> Option<Ordering> {
            match (self, other) {
                (T::Strict(a), T::Strict(b)) => Some(a.cmp(b)),
                _ => None,
            }
        }
        fn triple_parts(&self) -> Option<[Self; 3]> {
            match self {
                T::Triple(s, p, o) => Some([(**s).clone(), (**p).clone(), (**o).clone()]),
                _ => None,
            }
        }
    }

    #[test]
    fn term_class_discriminants_match_sparql_precedence() {
        // The exact ranks compare_values used: error/unbound < blank < IRI < literal < triple.
        assert!(TermClass::ErrorOrUnbound < TermClass::Blank);
        assert!(TermClass::Blank < TermClass::Iri);
        assert!(TermClass::Iri < TermClass::Literal);
        assert!(TermClass::Literal < TermClass::Triple);
        assert_eq!(TermClass::ErrorOrUnbound as u8, 0);
        assert_eq!(TermClass::Triple as u8, 4);
    }

    #[test]
    fn cross_class_precedence() {
        // unbound < blank < IRI < literal < triple, regardless of within-class value.
        let unbound = T::ErrorOrUnbound;
        let blank = T::Blank("z".into());
        let iri = T::Iri("a".into());
        let lit = T::NumLit(-1.0);
        let triple = T::Triple(Box::new(T::Iri("a".into())), Box::new(T::Iri("a".into())), Box::new(T::Iri("a".into())));
        assert_eq!(compare_terms(&unbound, &blank), Some(Ordering::Less));
        assert_eq!(compare_terms(&blank, &iri), Some(Ordering::Less));
        assert_eq!(compare_terms(&iri, &lit), Some(Ordering::Less));
        assert_eq!(compare_terms(&lit, &triple), Some(Ordering::Less));
        // and the reverse direction.
        assert_eq!(compare_terms(&triple, &unbound), Some(Ordering::Greater));
    }

    #[test]
    fn within_class_error_is_equal() {
        assert_eq!(compare_terms(&T::ErrorOrUnbound, &T::ErrorOrUnbound), Some(Ordering::Equal));
    }

    #[test]
    fn blanks_and_iris_order_by_string() {
        assert_eq!(compare_terms(&T::Blank("a".into()), &T::Blank("b".into())), Some(Ordering::Less));
        assert_eq!(compare_terms(&T::Iri("http://b".into()), &T::Iri("http://a".into())), Some(Ordering::Greater));
        assert_eq!(compare_terms(&T::Iri("x".into()), &T::Iri("x".into())), Some(Ordering::Equal));
    }

    #[test]
    fn numeric_literals_order_by_value_not_lexical() {
        // 9 < 10 by value even though "10" < "9" lexically — the numeric arm wins.
        assert_eq!(compare_terms(&T::NumLit(9.0), &T::NumLit(10.0)), Some(Ordering::Less));
        assert_eq!(compare_terms(&T::NumLit(2.5), &T::NumLit(2.5)), Some(Ordering::Equal));
    }

    #[test]
    fn f64_collapse_recheck_orders_big_integers_beyond_2_53() {
        // 2^53 and 2^53 + 1 share ONE f64 (2^53 + 1 is not representable) — the lenient
        // numeric arm alone would report them Equal. The exact recheck (`exact_cmp`) keeps
        // them ordered by value, so ORDER BY / MIN / MAX agree with relational =/<. This
        // assertion FAILS if the recheck in `compare_terms` is reverted (it would be Equal).
        let two53 = 9_007_199_254_740_992.0_f64;
        let a = T::ExactNum { f: two53, mant: 9_007_199_254_740_992, scale: 0 };
        let b = T::ExactNum { f: two53, mant: 9_007_199_254_740_993, scale: 0 };
        // The f64 keys are byte-equal: the collapse the recheck must repair.
        assert_eq!(a.as_f64(), b.as_f64());
        assert_eq!(compare_terms(&a, &b), Some(Ordering::Less));
        assert_eq!(compare_terms(&b, &a), Some(Ordering::Greater));
        // Genuinely equal exact values stay Equal (no spurious inequality introduced).
        let a2 = T::ExactNum { f: two53, mant: 9_007_199_254_740_992, scale: 0 };
        assert_eq!(compare_terms(&a, &a2), Some(Ordering::Equal));
    }

    #[test]
    fn f64_collapse_recheck_orders_high_precision_decimals() {
        // 0.123456789012345678 and 0.123456789012345679 differ only in the 18th fraction
        // digit and round to the SAME f64 (its shortest form is 0.12345678901234568); the
        // `f` field is that shared collapsed f64. The exact recheck orders them by value.
        let f = 0.12345678901234568_f64;
        let a = T::ExactNum { f, mant: 123_456_789_012_345_678, scale: 18 };
        let b = T::ExactNum { f, mant: 123_456_789_012_345_679, scale: 18 };
        assert_eq!(a.as_f64(), b.as_f64()); // f64 COLLAPSES them
        assert_eq!(compare_terms(&a, &b), Some(Ordering::Less));
        assert_eq!(compare_terms(&b, &a), Some(Ordering::Greater));
        // Different scales still align exactly: 0.10 (mant 10, scale 2) == 0.1 (mant 1, scale 1).
        let ten_hundredths = T::ExactNum { f: 0.1, mant: 10, scale: 2 };
        let one_tenth = T::ExactNum { f: 0.1, mant: 1, scale: 1 };
        assert_eq!(compare_terms(&ten_hundredths, &one_tenth), Some(Ordering::Equal));
        // And `exact_cmp` is `None` when either side has no exact tier (a plain f64 numeric),
        // so such a pair keeps the (correct) f64 verdict.
        assert_eq!(a.exact_cmp(&T::NumLit(f)), None);
        assert_eq!(T::NumLit(f).exact_cmp(&a), None);
    }

    #[test]
    fn strict_arm_decides_when_numeric_arm_cannot() {
        // Two Strict literals: as_f64 is None, strict_cmp decides by the timeline-like order.
        assert_eq!(compare_terms(&T::Strict(100), &T::Strict(200)), Some(Ordering::Less));
        assert_eq!(compare_terms(&T::Strict(5), &T::Strict(5)), Some(Ordering::Equal));
    }

    #[test]
    fn string_fallback_when_no_numeric_and_no_strict() {
        // Two plain string literals: as_f64 None, strict_cmp None → lexical string order.
        assert_eq!(compare_terms(&T::StrLit("apple".into()), &T::StrLit("banana".into())), Some(Ordering::Less));
        // A numeric vs a string literal (same class): numeric arm fails (one side None),
        // strict_cmp None, so both fall to value_str — "1" vs "z".
        assert_eq!(compare_terms(&T::NumLit(1.0), &T::StrLit("z".into())), Some(Ordering::Less));
    }

    #[test]
    fn triple_terms_order_componentwise_recursively() {
        let mk = |s: &str, p: &str, o: f64| {
            T::Triple(Box::new(T::Iri(s.into())), Box::new(T::Iri(p.into())), Box::new(T::NumLit(o)))
        };
        // Equal subject+predicate, object 1 < 2 by numeric value.
        assert_eq!(compare_terms(&mk("s", "p", 1.0), &mk("s", "p", 2.0)), Some(Ordering::Less));
        // Predicate decides before object: p1 < p2 even though o is greater.
        assert_eq!(compare_terms(&mk("s", "p1", 99.0), &mk("s", "p2", 1.0)), Some(Ordering::Less));
        // Subject decides first.
        assert_eq!(compare_terms(&mk("s1", "p", 1.0), &mk("s2", "p", 1.0)), Some(Ordering::Less));
        // Nested triple in subject position recurses.
        let nested = |inner_o: f64| {
            T::Triple(
                Box::new(T::Triple(Box::new(T::Iri("a".into())), Box::new(T::Iri("b".into())), Box::new(T::NumLit(inner_o)))),
                Box::new(T::Iri("p".into())),
                Box::new(T::Iri("o".into())),
            )
        };
        assert_eq!(compare_terms(&nested(1.0), &nested(2.0)), Some(Ordering::Less));
        assert_eq!(compare_terms(&nested(2.0), &nested(2.0)), Some(Ordering::Equal));
    }
}
