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

// [FABLE-5] sq-sqtk2.4 (epic sq-sqtk2, property B-1 of `research/mechanized-proof-program.md`
// §3.2/§5): Kani bounded-proof harnesses for the ORDER LAWS of [`compare_terms`] — the laws
// `ORDER BY`'s `sort_by` validity rests on (an inconsistent comparator makes Rust's sort
// panic or produce garbage). HARNESS-ONLY: the `compare_terms` body above is byte-unchanged.
//
// WHAT IS PROVED (tier: PROVED (bounded), per the design record's vocabulary) — over every
// value of the PER-HARNESS domain, each a stated sub-domain of the bounded model `M` below
// (the harness doc-comment is the authoritative domain statement):
//   • REFLEXIVITY (NaN-free domain):        compare_terms(x, x) == Some(Equal)
//   • ANTISYMMETRY-CONSISTENCY (full domain, NaN included):
//        compare_terms(x, y) == Some(o)  iff  compare_terms(y, x) == Some(o.reverse())
//        (equivalently: None in one direction iff None in the other)
//   • TRANSITIVITY on the defined domain, per LITERAL KIND (see the honest boundary below):
//        exact-integer literals INCLUDING the 2^53 f64-collapse straddle, double literals,
//        string literals, strict/temporal literals, a collapse-free exact/inexact numeric
//        mix, recursive triple terms, the non-literal scalar classes, and a REDUCED
//        int-with-non-literal composition (3 straddle ints × 2-string blanks/IRIs × Err —
//        a stated shrink, see its doc)
//   • WITHIN-CLASS TOTALITY (NaN-free domain): same-class pairs always compare `Some`
//   • EXACT-ORDER AGREEMENT: for exact-tier integer pairs, compare_terms equals the exact
//     i128 value order even where the f64 images collapse (THE guarantee the `exact_cmp`
//     recheck tier exists to provide — delete the recheck in `compare_terms` and this goes
//     red on the 2^53/2^53+1 pair)
//
// TRACTABILITY DISCIPLINE (why the harnesses are shaped this way): a term whose enum
// DISCRIMINANT is symbolic (built as `let x = match kani::any() { .. => Int, .. => Dbl }`)
// forks CBMC symex at EVERY downstream `match self` in the comparator's trait methods —
// multiplicatively per call, so even a UNARY harness over a handful of kinds does not
// terminate practically. The `for_each_*` kind-dispatch combinators below invert that:
// the symbolic kind CHOICE happens once, in a match whose arms each pass `f` a term with
// a CONCRETE discriminant (only the value index stays symbolic), keeping every dispatch
// leaf linear while the selector still quantifies over all kinds. Additionally, every
// harness carries the SMALLEST `#[kani::unwind]` bound its loops/recursion need (the
// bound also caps the syntactic expansion of `compare_terms`' triple-term recursion,
// which inflates the formula when set high). Under-bounding is LOUD: Kani's unwinding
// assertion fails, so a too-small bound cannot silently weaken a proof.
//
// DOMAIN-COVERAGE SELF-CHECKS (the sq-og8u8 pattern): a re-scoped (shrunk) domain can
// silently drop the very inputs that make a harness adversarial, so each shrunk domain's
// interesting inputs are PINNED by a proof: `domain_exhibits_the_2p53_collapse` pins the
// collapse straddle used by the full `INTS` table, the reduced int-with-non-literal
// domain, and the triple-object domain;
// `domain_cf_numeric_is_collapse_free_with_signed_zero_pair` pins the collapse-free
// mixed range (exact images only) and its ±0.0 equal-but-lexically-distinct pair. If a
// future re-scope silences an interesting input, one of these goes red rather than the
// law harness silently proving less.
//
// HONESTY BOUNDARY (per the bead): this machine-checks the `compare_terms` ALGORITHM over
// the model observation surface `M` below. It is explicitly NOT a claim about the engine's
// `Value` impl of `CompareTerm` in `sparq-engine/src/exec.rs` (that instance stays covered
// by unit + W3C conformance tests; ledgered as the next wave in the design record §6). The
// model is, however, deliberately FAITHFUL to that impl's observation semantics — each trait
// method below documents the `exec.rs` behaviour it mirrors — because a convenient model
// would make these proofs vacuous.
//
// MACHINE-CHECKED FINDINGS (not masked — see the `witness_*` harnesses): the order laws do
// NOT extend across mixed literal KINDS. Three concrete counterexamples, each PROVED
// reachable (the witness harnesses assert current behaviour and fail if it changes):
//   1. exact/inexact numeric-tier mix AT the 2^53 collapse boundary is intransitive
//      (`witness_mixed_tier_collapse_intransitivity`);
//   2. numeric-vs-plain-string comparison falls to the LEXICAL form, which disagrees with
//      the numeric order (`witness_numeric_vs_string_lexical_intransitivity`);
//   3. NaN makes the comparator PARTIAL — `compare_terms` returns `None`, which callers map
//      to `Equal` (`witness_nan_comparison_partiality`).
// All three are reachable through the engine's real `Value` impl (verified against
// `exec.rs`: `num_compare` falls back to the collapsed f64 for float/double operands;
// `value_compare_strict` is `None` cross-family so the string fallback fires; `parse_xsd_f64`
// accepts `NaN`). Tracked as bug bead sq-wjl8i (P1) — the per-kind transitivity harnesses
// above scope exactly which sub-domains ARE lawful, and the witnesses pin exactly where the
// law breaks.
//
// RUN:  cargo kani -p sparq-substrate --features compare
// (nightly lane wiring is bead sq-sqtk2.5; under normal build/clippy/test this module is
// stripped — the `kani` cfg is registered in this crate's Cargo.toml `[lints.rust]`.)
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use std::cmp::Ordering;

    /// 2^53 — the f64 integer-exactness boundary: every integer of magnitude ≤ 2^53 has an
    /// exact f64 image; above it, adjacent integers start sharing one image ("collapse").
    const TWO53: i128 = 9_007_199_254_740_992;

    /// Adversarial exact-integer domain (the model's `xsd:integer` tier). Deliberately
    /// STRADDLES the collapse boundary in both signs: `±(2^53 + 1)` are the first
    /// non-representable integers (ties-to-even rounds them onto `±2^53`), while
    /// `2^53 - 1` and `2^53 + 2` are exactly representable neighbours. The small values
    /// `2` and `10` exist for the numeric-vs-lexical witness (`"10" < "11" < "2"`).
    const INTS: [i128; 10] = [
        -(TWO53 + 1), // collapses onto -(2^53)
        -TWO53,
        -2,
        0,
        2,
        10,
        TWO53 - 1, // exactly representable
        TWO53,     // the collapse image
        TWO53 + 1, // NOT representable: rounds onto 2^53 — the collapse
        TWO53 + 2, // exactly representable again
    ];
    // Index names for the values the harnesses pick out explicitly.
    const I_TWO: u8 = 4;
    const I_TEN: u8 = 5;
    const I_2P53_M1: u8 = 6;
    const I_2P53: u8 = 7;
    const I_2P53_P1: u8 = 8;
    const I_2P53_P2: u8 = 9;

    /// The f64 images of `INTS`, computed at COMPILE TIME by the same `as f64`
    /// (round-ties-to-even) conversion the engine's numeric tower applies — so the collapse
    /// is data the solver can see: `INT_F64S[I_2P53] == INT_F64S[I_2P53_P1]`.
    const INT_F64S: [f64; 10] = [
        INTS[0] as f64,
        INTS[1] as f64,
        INTS[2] as f64,
        INTS[3] as f64,
        INTS[4] as f64,
        INTS[5] as f64,
        INTS[6] as f64,
        INTS[7] as f64,
        INTS[8] as f64,
        INTS[9] as f64,
    ];

    /// The decimal lexical forms of `INTS`, index-parallel. COMPILE-TIME constants: the
    /// model must not run an int→string conversion loop under CBMC, and the values are
    /// load-bearing for the lexical-fallback arms (`"10" < "11" < "2"` lexically).
    const INT_STRS: [&str; 10] = [
        "-9007199254740993",
        "-9007199254740992",
        "-2",
        "0",
        "2",
        "10",
        "9007199254740991",
        "9007199254740992",
        "9007199254740993",
        "9007199254740994",
    ];

    /// Adversarial inexact (`xsd:double`) domain: `±0.0` (an equal-but-lexically-distinct
    /// pair), a fraction, and — crucially — `2^53` itself: the shared f64 image of the
    /// collapsed integers, which is exactly where the mixed-tier witness lives.
    const DBLS: [f64; 6] = [
        -9_007_199_254_740_992.0, // -(2^53)
        -0.0,
        0.0,
        0.5,
        9_007_199_254_740_992.0, // 2^53 — the collapse image
        9_007_199_254_740_994.0, // 2^53 + 2
    ];
    const D_2P53: u8 = 4;
    /// Lexical forms of `DBLS` (fixed representative renderings; the laws only need
    /// `value_str` to be a deterministic function of the model value).
    const DBL_STRS: [&str; 6] = [
        "-9.007199254740992E15",
        "-0.0E0",
        "0.0E0",
        "5.0E-1",
        "9.007199254740992E15",
        "9.007199254740994E15",
    ];

    /// Blank labels / IRIs / plain-string literal values: tiny but adversarial — the empty
    /// string, a prefix pair (`"a"` < `"ab"`), and `"11"` (lexically ABOVE `"10"` and BELOW
    /// `"2"`: the digit-string inversion the numeric-vs-string witness rides on).
    const STRS: [&str; 4] = ["", "11", "a", "ab"];
    const S_11: u8 = 1;

    /// Strictly-comparable typed-literal tier (models the engine's dateTime/date-by-timeline
    /// arm): `Strict(i)` compares to `Strict(j)` by index. The lexical forms are chosen to
    /// AGREE with that order — the real engine's mixed-timezone dateTimes can order
    /// lexically ≠ timeline, but that pair never reaches the string fallback (same family ⇒
    /// `strict_cmp` decides), so agreement here is not a fidelity loss for these laws.
    const STRICT_STRS: [&str; 3] = ["2024-01-01", "2025-01-01", "2026-01-01"];

    /// The bounded model term. One variant per observation SHAPE the algorithm
    /// distinguishes; every [`TermClass`] is covered. Triple terms are depth-1 (scalar
    /// components) — a stated bound; the recursion body is the same for deeper nesting.
    /// NOT modelled (stated bounds): booleans (an inexact-shaped numeric at 0.0/1.0 —
    /// `Dbl` covers the shape), decimals (same exact tier as `Int` through `exact_cmp`),
    /// and `±INF` doubles (totally ordered f64s like every other non-NaN double).
    enum M {
        /// Unbound / SPARQL type error.
        Err,
        /// Blank node with label `STRS[i]`.
        Blank(u8),
        /// IRI `STRS[i]`.
        Iri(u8),
        /// Exact-tier numeric literal (`xsd:integer`) with value `INTS[i]`: `as_f64` is the
        /// COLLAPSING image, `exact_cmp` is exact — the engine's int/decimal tier.
        Int(u8),
        /// Inexact-tier numeric literal (`xsd:double`) with value `DBLS[i]`: its value IS
        /// its f64; no exact tier.
        Dbl(u8),
        /// The `xsd:double` NaN (reachable in the engine: `parse_xsd_f64` accepts `NaN`).
        Nan,
        /// Plain string literal `STRS[i]`: no numeric / strict tier — lexical fallback.
        Str(u8),
        /// Same-family strictly-comparable typed literal (dateTime-by-timeline model).
        Strict(u8),
        /// Depth-1 RDF-1.2 quoted triple: `(Iri STRS[s], Iri STRS[p], Int INTS[o])`.
        Trip(u8, u8, u8),
    }

    impl M {
        /// The model's `as_numeric` (mirrors `exec.rs`): `(exact_tier, exact_value, f64_image)`.
        /// `exact_value` is meaningful only when `exact_tier` — for the inexact tier the
        /// value IS the image (the engine's `Num::Double`, whose `to_dec()` is `None`).
        fn num(&self) -> Option<(bool, i128, f64)> {
            match self {
                M::Int(i) => Some((true, INTS[usize::from(*i)], INT_F64S[usize::from(*i)])),
                M::Dbl(i) => Some((false, 0, DBLS[usize::from(*i)])),
                M::Nan => Some((false, 0, f64::NAN)),
                _ => None,
            }
        }
    }

    impl CompareTerm for M {
        fn term_class(&self) -> TermClass {
            match self {
                M::Err => TermClass::ErrorOrUnbound,
                M::Blank(_) => TermClass::Blank,
                M::Iri(_) => TermClass::Iri,
                M::Int(_) | M::Dbl(_) | M::Nan | M::Str(_) | M::Strict(_) => TermClass::Literal,
                M::Trip(..) => TermClass::Triple,
            }
        }
        fn value_str(&self) -> Option<String> {
            // Engine (`value_str`): the term's lexical form — a deterministic function of
            // the term. Model: fixed compile-time tables (no conversion loops under CBMC).
            match self {
                M::Err => None,
                M::Blank(i) | M::Iri(i) | M::Str(i) => Some(STRS[usize::from(*i)].to_string()),
                M::Int(i) => Some(INT_STRS[usize::from(*i)].to_string()),
                M::Dbl(i) => Some(DBL_STRS[usize::from(*i)].to_string()),
                M::Nan => Some("NaN".to_string()),
                M::Strict(i) => Some(STRICT_STRS[usize::from(*i)].to_string()),
                // Never observed by `compare_terms` (the Triple arm recurses via
                // `triple_parts` before any string fallback).
                M::Trip(..) => None,
            }
        }
        fn as_f64(&self) -> Option<f64> {
            // Engine (`as_num`): the LENIENT f64 coercion — for an exact integer this is the
            // collapsing `as f64` image; that collapse is the entire point of the domain.
            self.num().map(|(_, _, f)| f)
        }
        fn exact_cmp(&self, other: &Self) -> Option<Ordering> {
            // Engine (`exec.rs` `exact_cmp` → `num_compare`): EXACT (decimal tower) only
            // when BOTH operands are int/decimal tier; a float/double operand falls back to
            // the — possibly collapsed — f64 `partial_cmp` (its value IS its f64). NOT the
            // convenient "None on mixed pairs": the engine returns the collapsed verdict,
            // and the mixed-tier witness below exists precisely because of it.
            let (ax, av, af) = self.num()?;
            let (bx, bv, bf) = other.num()?;
            if ax && bx {
                return Some(av.cmp(&bv));
            }
            af.partial_cmp(&bf)
        }
        fn strict_cmp(&self, other: &Self) -> Option<Ordering> {
            // Engine (`value_compare_strict`): decides SAME-FAMILY pairs (dateTime/date by
            // timeline, same-tag/same-other-XSD lexically); every cross-family pair is
            // `None` ⇒ the caller falls back to the lexical string form. The model's only
            // strict family is `Strict`. (The engine also decides plain-string pairs here —
            // lexically, identical to the fallback the model routes them through.)
            match (self, other) {
                (M::Strict(a), M::Strict(b)) => Some(a.cmp(b)),
                _ => None,
            }
        }
        fn triple_parts(&self) -> Option<[Self; 3]> {
            match self {
                M::Trip(s, p, o) => Some([M::Iri(*s), M::Iri(*p), M::Int(*o)]),
                _ => None,
            }
        }
    }

    /// A symbolic index below `n`.
    fn any_idx(n: u8) -> u8 {
        let i: u8 = kani::any();
        kani::assume(i < n);
        i
    }

    /// Any depth-1 triple. Objects range over `{2, 2^53, 2^53+1}` so the RECURSIVE numeric
    /// arm exercises the exact-tier recheck on the collapsed pair inside a triple.
    fn any_trip() -> M {
        let o: u8 = kani::any();
        kani::assume(o == I_TWO || o == I_2P53 || o == I_2P53_P1);
        M::Trip(any_idx(2), any_idx(2), o)
    }

    /// Strict-weak-order composition at one symbolic triple `(x, y, z)`. Asserting the
    /// `Less`/`Equal` compositions is COMPLETE: the `Greater` compositions are the same
    /// instantiations under an `(x, z)` swap, and the harness quantifies symbolically over
    /// every permutation of the domain (antisymmetry-consistency is proved separately).
    fn assert_transitive_at(x: &M, y: &M, z: &M) {
        let (Some(xy), Some(yz)) = (compare_terms(x, y), compare_terms(y, z)) else {
            return; // outside the defined domain (law scoped to Some legs)
        };
        let xz = compare_terms(x, z);
        match (xy, yz) {
            (Ordering::Less, Ordering::Less)
            | (Ordering::Less, Ordering::Equal)
            | (Ordering::Equal, Ordering::Less) => {
                assert!(xz == Some(Ordering::Less), "transitivity: x<=y<=z (strict leg) => x<z");
            }
            (Ordering::Equal, Ordering::Equal) => {
                assert!(xz == Some(Ordering::Equal), "transitivity: x~y~z => x~z");
            }
            _ => (),
        }
    }

    /// Domain self-check: the numeric domain genuinely exhibits the 2^53 collapse (distinct
    /// integers, one f64 image) and the double table contains the collapse image itself.
    /// If this fails, the domain is non-adversarial and the other harnesses are vacuous.
    #[kani::proof]
    fn domain_exhibits_the_2p53_collapse() {
        assert!(INTS[usize::from(I_2P53)] != INTS[usize::from(I_2P53_P1)]);
        // Deliberate exact f64 comparisons: the collapse IS bit-level image equality.
        assert!(INT_F64S[usize::from(I_2P53)] == INT_F64S[usize::from(I_2P53_P1)]);
        assert!(INT_F64S[usize::from(I_2P53_M1)] < INT_F64S[usize::from(I_2P53)]);
        assert!(INT_F64S[usize::from(I_2P53)] < INT_F64S[usize::from(I_2P53_P2)]);
        assert!(DBLS[usize::from(D_2P53)] == INT_F64S[usize::from(I_2P53)]);
        assert!(INTS[0] != INTS[1] && INT_F64S[0] == INT_F64S[1]); // negative-sign collapse
    }

    // ------------------------------------------------------------------------------
    // KIND-DISPATCH COMBINATORS. Each `for_each_*` symbolically SELECTS one literal kind
    // and calls `f` with a term whose enum DISCRIMINANT is concrete (only the value index
    // inside the kind stays symbolic). This is the load-bearing tractability device: a
    // term with a SYMBOLIC discriminant forks CBMC symex at every downstream match in the
    // comparator (multiplicatively, per trait-method call), which does not terminate
    // practically; a concrete-discriminant term keeps each dispatch leaf linear. Coverage
    // is unchanged — the symbolic selector still quantifies over every kind in the list.
    // ------------------------------------------------------------------------------

    /// One symbolic choice of NUMERIC literal kind — `Int` / `Dbl` over their full tables,
    /// plus NaN when `with_nan` — passed to `f` with a concrete discriminant.
    fn for_each_numeric(with_nan: bool, f: impl Fn(&M)) {
        match kani::any::<u8>() {
            0 => f(&M::Int(any_idx(INTS.len() as u8))),
            1 if with_nan => f(&M::Nan),
            _ => f(&M::Dbl(any_idx(DBLS.len() as u8))),
        }
    }

    /// One symbolic choice of NON-NUMERIC scalar kind — `Err`, `Blank`/`Iri`/`Str` over the
    /// full `STRS` table, `Strict` over the full `STRICT_STRS` table.
    fn for_each_nonnumeric_scalar(f: impl Fn(&M)) {
        match kani::any::<u8>() {
            0 => f(&M::Err),
            1 => f(&M::Blank(any_idx(STRS.len() as u8))),
            2 => f(&M::Iri(any_idx(STRS.len() as u8))),
            3 => f(&M::Str(any_idx(STRS.len() as u8))),
            _ => f(&M::Strict(any_idx(STRICT_STRS.len() as u8))),
        }
    }

    /// One symbolic choice of non-literal scalar class — `Err` / `Blank` / `Iri`.
    fn for_each_nonlit_scalar(f: impl Fn(&M)) {
        match kani::any::<u8>() {
            0 => f(&M::Err),
            1 => f(&M::Blank(any_idx(STRS.len() as u8))),
            _ => f(&M::Iri(any_idx(STRS.len() as u8))),
        }
    }

    /// One symbolic choice of NaN-free literal kind — `Int` / `Dbl` / `Str` / `Strict`.
    fn for_each_literal_nan_free(f: impl Fn(&M)) {
        match kani::any::<u8>() {
            0 => f(&M::Int(any_idx(INTS.len() as u8))),
            1 => f(&M::Dbl(any_idx(DBLS.len() as u8))),
            2 => f(&M::Str(any_idx(STRS.len() as u8))),
            _ => f(&M::Strict(any_idx(STRICT_STRS.len() as u8))),
        }
    }

    /// One symbolic choice from the REDUCED int-with-non-literal composition domain:
    /// `Int` over the 3-value collapse straddle `{2, 2^53, 2^53+1}`, `Err`, and
    /// `Blank`/`Iri` over the first two `STRS` entries.
    fn for_each_int_or_nonlit_small(f: impl Fn(&M)) {
        match kani::any::<u8>() {
            0 => f(&M::Err),
            1 => f(&M::Blank(any_idx(2))),
            2 => f(&M::Iri(any_idx(2))),
            _ => {
                let o: u8 = kani::any();
                kani::assume(o == I_TWO || o == I_2P53 || o == I_2P53_P1);
                f(&M::Int(o))
            }
        }
    }

    // REFLEXIVITY over the NaN-free domain: `compare_terms(x, x) == Some(Equal)`.
    // (With NaN the comparator is PARTIAL — see `witness_nan_comparison_partiality`.)
    // Reflexivity is UNARY and per-value, so splitting the domain by literal kind proves
    // the SAME law over the SAME union domain: the three harnesses below jointly cover
    // every NaN-free model term.
    fn assert_reflexive_at(x: &M) {
        assert!(compare_terms(x, x) == Some(Ordering::Equal), "reflexivity");
    }

    /// REFLEXIVITY: numeric literals — `Int` over the full `INTS` table (collapse straddle
    /// included) and `Dbl` over the full `DBLS` table (NaN excluded — see the witness).
    #[kani::proof]
    #[kani::unwind(3)]
    fn reflexivity_numeric_literals_nan_free() {
        for_each_numeric(false, |x| assert_reflexive_at(x));
    }

    /// REFLEXIVITY: the non-numeric scalar kinds — `Err`, `Blank`/`Iri`/`Str` over the full
    /// `STRS` table, `Strict` over the full `STRICT_STRS` table.
    #[kani::proof]
    #[kani::unwind(4)]
    fn reflexivity_nonnumeric_scalars() {
        for_each_nonnumeric_scalar(|x| assert_reflexive_at(x));
    }

    /// REFLEXIVITY: depth-1 triple terms (objects straddle the 2^53 collapse, so the
    /// recursive exact-recheck arm is exercised on the x == x diagonal too).
    #[kani::proof]
    #[kani::unwind(4)]
    fn reflexivity_triple_terms() {
        assert_reflexive_at(&any_trip());
    }

    // ANTISYMMETRY-CONSISTENCY over the FULL domain (NaN included): the two directions are
    // exact mirrors — `Some(o)` iff `Some(o.reverse())`, and `None` iff `None`. The
    // assertion checks BOTH directions of one symbolic pair, so covering every unordered
    // kind-GROUP pair covers every ordered pair of the full domain: with groups
    // N = {Int, Dbl, Nan}, S = {Err, Blank, Iri, Str, Strict}, T = {Trip}, the five
    // harnesses below cover N×N, N×S, S×S, T×(N∪S), T×T = (N∪S∪T)².
    fn assert_antisymmetric_at(x: &M, y: &M) {
        match compare_terms(x, y) {
            Some(o) => {
                assert!(compare_terms(y, x) == Some(o.reverse()), "antisymmetry: mirror");
            }
            None => assert!(compare_terms(y, x).is_none(), "antisymmetry: None mirror"),
        }
    }

    /// ANTISYMMETRY: numeric × numeric, NaN INCLUDED (the `None`-mirror leg is live here:
    /// NaN against any numeric is `None` in BOTH directions).
    #[kani::proof]
    #[kani::unwind(3)]
    fn antisymmetry_numeric_pairs_incl_nan() {
        for_each_numeric(true, |x| for_each_numeric(true, |y| assert_antisymmetric_at(x, y)));
    }

    /// ANTISYMMETRY: numeric (NaN included) × non-numeric scalar — the mixed-kind literal
    /// pairs ride the lexical fallback (including the long `INT_STRS`/`DBL_STRS` forms
    /// against `STRICT_STRS`, hence the larger unwind); cross-class pairs ride the rank.
    #[kani::proof]
    #[kani::unwind(12)]
    fn antisymmetry_numeric_vs_nonnumeric_incl_nan() {
        for_each_numeric(true, |x| {
            for_each_nonnumeric_scalar(|y| assert_antisymmetric_at(x, y));
        });
    }

    /// ANTISYMMETRY: non-numeric scalar × non-numeric scalar (string arms + class ranks).
    #[kani::proof]
    #[kani::unwind(4)]
    fn antisymmetry_nonnumeric_scalar_pairs() {
        for_each_nonnumeric_scalar(|x| {
            for_each_nonnumeric_scalar(|y| assert_antisymmetric_at(x, y));
        });
    }

    /// ANTISYMMETRY: triple term × any non-triple (always a cross-class pair — the class
    /// rank decides; NaN included on the scalar side).
    #[kani::proof]
    #[kani::unwind(4)]
    fn antisymmetry_triple_vs_scalar() {
        let t = any_trip();
        if kani::any() {
            for_each_numeric(true, |y| assert_antisymmetric_at(&t, y));
        } else {
            for_each_nonnumeric_scalar(|y| assert_antisymmetric_at(&t, y));
        }
    }

    /// ANTISYMMETRY: triple term × triple term (the component-wise recursion mirrored).
    #[kani::proof]
    #[kani::unwind(4)]
    fn antisymmetry_triple_pairs() {
        assert_antisymmetric_at(&any_trip(), &any_trip());
    }

    // WITHIN-CLASS TOTALITY over the NaN-free domain: every same-class pair compares
    // `Some` — including MIXED literal kinds (which stay totally DEFINED via the string
    // fallback even where they are not transitive; see the witnesses). Split: the Literal
    // class (the only class with multiple kinds) and the singleton-kind classes.

    /// WITHIN-CLASS TOTALITY: the Literal class, NaN-free — every literal-kind pair
    /// (numeric × numeric, numeric × string/strict via the lexical fallback, ...) is
    /// `Some`. The larger unwind covers the long-form numeric-vs-`Strict` lexical compare.
    #[kani::proof]
    #[kani::unwind(12)]
    fn within_class_totality_literals_nan_free() {
        for_each_literal_nan_free(|x| {
            for_each_literal_nan_free(|y| {
                assert!(compare_terms(x, y).is_some(), "within-class totality: literals");
            });
        });
    }

    /// WITHIN-CLASS TOTALITY: the singleton-kind classes — `Err`/`Blank`/`Iri`/`Trip`.
    /// For these classes same-class MEANS same-kind, so the four same-kind pairs below are
    /// exactly the same-class pairs the law quantifies over.
    #[kani::proof]
    #[kani::unwind(4)]
    fn within_class_totality_nonliterals() {
        let tot = |x: &M, y: &M| {
            assert!(compare_terms(x, y).is_some(), "within-class totality: non-literals");
        };
        match kani::any::<u8>() {
            0 => tot(&M::Err, &M::Err),
            1 => tot(
                &M::Blank(any_idx(STRS.len() as u8)),
                &M::Blank(any_idx(STRS.len() as u8)),
            ),
            2 => tot(
                &M::Iri(any_idx(STRS.len() as u8)),
                &M::Iri(any_idx(STRS.len() as u8)),
            ),
            _ => tot(&any_trip(), &any_trip()),
        }
    }

    /// TRANSITIVITY: exact-integer literals over the FULL `INTS` table, INCLUDING the 2^53
    /// collapse straddle. The collapsed pair `{2^53, 2^53+1}` is in-domain, so this proves
    /// the `exact_cmp` recheck keeps the exact tier transitive ACROSS the boundary.
    /// Domain: Int-ONLY (a fixed kind pattern) — the composition with the non-literal
    /// scalar classes lives in `transitivity_nonliteral_scalars` /
    /// `transitivity_int_with_nonliteral_scalars` (see the tractability note above).
    #[kani::proof]
    #[kani::unwind(3)]
    fn transitivity_exact_int_literals_incl_2p53_collapse() {
        let term = || M::Int(any_idx(INTS.len() as u8));
        assert_transitive_at(&term(), &term(), &term());
    }

    /// TRANSITIVITY: the non-literal scalar classes (`Err` / `Blank` / `Iri` over the full
    /// `STRS` table) — cross-class precedence plus the within-class string arms.
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_nonliteral_scalars() {
        for_each_nonlit_scalar(|x| {
            for_each_nonlit_scalar(|y| {
                for_each_nonlit_scalar(|z| assert_transitive_at(x, y, z));
            });
        });
    }

    /// TRANSITIVITY: exact-integer literals COMPOSED with the non-literal scalar classes —
    /// the cross-class legs of the law. REDUCED domain (a stated shrink, for symex
    /// tractability): Int over the 3-value collapse straddle `{2, 2^53, 2^53+1}` (so the
    /// exact-recheck arm is still exercised next to cross-class legs), Blank/Iri over the
    /// first two `STRS` entries, and `Err`. The FULL-domain within-class laws are the two
    /// harnesses above; cross-class order itself is decided purely by the `TermClass` rank,
    /// which does not depend on the per-kind value domains.
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_int_with_nonliteral_scalars() {
        for_each_int_or_nonlit_small(|x| {
            for_each_int_or_nonlit_small(|y| {
                for_each_int_or_nonlit_small(|z| assert_transitive_at(x, y, z));
            });
        });
    }

    /// TRANSITIVITY: double literals (NaN-free — with NaN the comparator is partial, see the
    /// witness). Includes the `±0.0` equal-but-lexically-distinct pair: a mutation that let
    /// numeric ties fall through to the string form would go red here.
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_double_literals() {
        let term = || M::Dbl(any_idx(DBLS.len() as u8));
        assert_transitive_at(&term(), &term(), &term());
    }

    /// One symbolic choice from the collapse-free MIXED numeric domain: `Int` over
    /// `{-2, 0, 2, 10}` (indices 2..=I_TEN) or `Dbl` over `{-0.0, 0.0, 0.5}` (indices
    /// 1..=3) — every value exactly representable in f64.
    fn for_each_cf_numeric(f: impl Fn(&M)) {
        if kani::any() {
            let i: u8 = kani::any();
            kani::assume(i >= 2 && i <= I_TEN);
            f(&M::Int(i));
        } else {
            let i: u8 = kani::any();
            kani::assume(i >= 1 && i <= 3);
            f(&M::Dbl(i));
        }
    }

    /// TRANSITIVITY: a MIXED exact/inexact numeric domain restricted BELOW the collapse
    /// boundary (see `for_each_cf_numeric` for the exact values). Lawful here — which
    /// sharpens the mixed-tier witness: the law breaks specifically AT the collapse, not
    /// on every exact/inexact mix.
    #[kani::proof]
    #[kani::unwind(3)]
    fn transitivity_mixed_numeric_collapse_free_range() {
        for_each_cf_numeric(|x| {
            for_each_cf_numeric(|y| {
                for_each_cf_numeric(|z| assert_transitive_at(x, y, z));
            });
        });
    }

    /// Domain self-check for the collapse-free mixed-numeric domain (companion to
    /// `domain_exhibits_the_2p53_collapse`, which pins the straddle domains): every `Int`
    /// the `for_each_cf_numeric` combinator can emit has an EXACT f64 image (the range is
    /// genuinely collapse-free — an index drift onto the straddle would surface here), and
    /// the `Dbl` table's `±0.0` entries (indices 1/2, inside the combinator's `1..=3`
    /// range) are the equal-but-lexically-distinct pair the double-transitivity harness
    /// relies on to catch a ties-fall-to-string-form mutation. If this fails, a re-scope
    /// has silenced the domain's interesting inputs and the harnesses above it are weaker
    /// than documented. [FABLE-5] sq-sqtk2.4
    #[kani::proof]
    #[kani::unwind(3)]
    fn domain_cf_numeric_is_collapse_free_with_signed_zero_pair() {
        for_each_cf_numeric(|x| match x {
            M::Int(i) => {
                // Exact image: the f64 roundtrips to the same integer (deliberate exact
                // comparison — exactness IS the property).
                assert!(
                    INT_F64S[usize::from(*i)] as i128 == INTS[usize::from(*i)],
                    "cf domain: Int range must be collapse-free"
                );
            }
            M::Dbl(i) => assert!(usize::from(*i) < DBLS.len(), "cf domain: Dbl index in-table"),
            _ => unreachable!("for_each_cf_numeric emits only Int/Dbl"),
        });
        // The signed-zero pair: numerically equal, lexically distinct.
        assert!(DBLS[1] == DBLS[2], "signed zeros compare numerically equal");
        assert!(DBL_STRS[1] != DBL_STRS[2], "signed zeros are lexically distinct");
    }

    /// TRANSITIVITY: plain-string literals (lexical order; includes the empty string and a
    /// prefix pair).
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_string_literals() {
        let term = || M::Str(any_idx(STRS.len() as u8));
        assert_transitive_at(&term(), &term(), &term());
    }

    /// TRANSITIVITY: strict/temporal literals (the dateTime-by-timeline model arm).
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_strict_typed_literals() {
        let term = || M::Strict(any_idx(STRICT_STRS.len() as u8));
        assert_transitive_at(&term(), &term(), &term());
    }

    /// TRANSITIVITY: depth-1 triple terms — the component-wise recursion, with objects
    /// ranging over the collapsed integer pair so the recursive numeric arm hits the
    /// exact-tier recheck INSIDE a triple.
    #[kani::proof]
    #[kani::unwind(4)]
    fn transitivity_triple_terms_recursive() {
        assert_transitive_at(&any_trip(), &any_trip(), &any_trip());
    }

    /// EXACT-ORDER AGREEMENT: for every exact-tier pair, `compare_terms` equals the exact
    /// i128 value order — INCLUDING the collapsed pairs, where the lenient f64 arm alone
    /// would report `Equal`. This is the load-bearing guarantee of the `exact_cmp` recheck:
    /// delete the recheck in `compare_terms` (or weaken the model's `exact_cmp` to `None` on
    /// exact pairs) and this harness goes red on `{2^53, 2^53+1}` — the non-vacuity anchor.
    #[kani::proof]
    #[kani::unwind(3)]
    fn exact_int_order_agreement_incl_2p53_collapse() {
        let i = any_idx(INTS.len() as u8);
        let j = any_idx(INTS.len() as u8);
        let verdict = compare_terms(&M::Int(i), &M::Int(j));
        assert!(
            verdict == Some(INTS[usize::from(i)].cmp(&INTS[usize::from(j)])),
            "exact tier orders by exact value, collapse notwithstanding"
        );
    }

    /// FINDING 1 (machine-checked witness, current behaviour): transitivity FAILS on the
    /// exact/inexact numeric-tier MIX at the 2^53 collapse boundary. A double equal to the
    /// shared image ties (via the engine-faithful collapsed-f64 fallback) with BOTH collapsed
    /// integers, which the exact tier orders strictly:
    ///   `2^53 ~ 9.007199254740992E15 ~ 2^53+1`  but  `2^53 < 2^53+1`.
    /// Engine-reachable: `num_compare` falls back to f64 when either operand is
    /// float/double. Tracked as bead sq-wjl8i. If a fix lands (e.g. exact mixed-tier
    /// comparison — every finite f64 is an exact rational), this harness goes red and
    /// should be REMOVED with the fix.
    #[kani::proof]
    fn witness_mixed_tier_collapse_intransitivity() {
        let x = M::Int(I_2P53);
        let y = M::Dbl(D_2P53);
        let z = M::Int(I_2P53_P1);
        assert!(compare_terms(&x, &y) == Some(Ordering::Equal));
        assert!(compare_terms(&y, &z) == Some(Ordering::Equal));
        assert!(compare_terms(&x, &z) == Some(Ordering::Less)); // intransitive triple
    }

    /// FINDING 2 (machine-checked witness, current behaviour): transitivity FAILS on the
    /// numeric-vs-plain-string mix — cross-family pairs fall back to the LEXICAL form, and
    /// lexical digit-string order disagrees with numeric order:
    ///   `10 < "11" < 2` (lexically)  but  `10 > 2` (numerically).
    /// Engine-reachable: `value_compare_strict` is `None` for a (numeric, plain-string)
    /// pair, so `compare_terms` reaches its string fallback. Tracked as bead sq-wjl8i.
    #[kani::proof]
    #[kani::unwind(4)]
    fn witness_numeric_vs_string_lexical_intransitivity() {
        let x = M::Int(I_TEN); // 10
        let y = M::Str(S_11); // "11"
        let z = M::Int(I_TWO); // 2
        assert!(compare_terms(&x, &y) == Some(Ordering::Less)); // "10" < "11"
        assert!(compare_terms(&y, &z) == Some(Ordering::Less)); // "11" < "2"
        assert!(compare_terms(&x, &z) == Some(Ordering::Greater)); // 10 > 2
    }

    /// FINDING 3 (machine-checked witness, current behaviour): NaN makes the comparator
    /// PARTIAL — numeric comparison against NaN is `None` (callers map it to `Equal`, so at
    /// the call site NaN ties with EVERY numeric, collapsing distinct equivalence classes),
    /// while NaN against a plain string is still DEFINED (lexical fallback). Engine-
    /// reachable: `parse_xsd_f64` accepts the XSD `NaN` spelling. Tracked as bead
    /// sq-wjl8i. Also pins that the module-level doc's "`None` only when a term has no
    /// string form" understates `None` — that doc drift is tracked as bead sq-ma9fb.
    #[kani::proof]
    #[kani::unwind(4)]
    fn witness_nan_comparison_partiality() {
        assert!(compare_terms(&M::Nan, &M::Nan).is_none());
        assert!(compare_terms(&M::Nan, &M::Dbl(2)).is_none()); // vs 0.0
        assert!(compare_terms(&M::Dbl(2), &M::Nan).is_none());
        assert!(compare_terms(&M::Nan, &M::Int(I_2P53)).is_none());
        // ... but the string fallback still fires cross-family: "NaN" vs "a".
        assert!(compare_terms(&M::Nan, &M::Str(2)) == Some(Ordering::Less));
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

    // --- BUG WITNESSES (pinned by the sq-sqtk2.4 Kani harnesses; tracked as P1 bug bead
    //     sq-wjl8i) ---
    //
    // These three tests reproduce the machine-checked counterexamples from the Kani
    // `kani_proofs` module's `witness_*` harnesses. They are `#[ignore]` because they assert
    // the ORDER LAW (transitivity of equality) which CURRENTLY FAILS — i.e., removing
    // `#[ignore]` turns them RED under `cargo test`. Remove `#[ignore]` and run them to
    // verify a fix; they will pass once the comparator is made consistent. [SONNET-4.6]

    /// BUG witness 1 — mixed exact/inexact numeric pair at the 2^53 collapse boundary.
    ///
    /// The transitivity law for equality is: x = y AND y = z => x = z.
    /// Here: `xsd:integer 2^53` equals `xsd:double 9.007199254740992E15` (their f64 images
    /// are identical and `exact_cmp` returns `None` on a mixed pair), and
    /// `xsd:double 9.007199254740992E15` also equals `xsd:integer 2^53+1` (same reason),
    /// but `compare_terms(xsd:integer 2^53, xsd:integer 2^53+1)` returns `Less` (the
    /// exact recheck correctly orders them). The triple `(2^53, double(2^53), 2^53+1)` is an
    /// intransitive witness. Tracked as P1 correctness bead sq-wjl8i.
    /// `ORDER BY` / `GROUP BY` / `MIN` / `MAX` over a column mixing integer and double values
    /// near 2^53 may produce permutation-unstable output violating SPARQL ordering semantics.
    #[test]
    #[ignore = "BUG sq-wjl8i: mixed exact/inexact numeric intransitivity at 2^53 collapse — remove once fixed"]
    fn bug_witness_mixed_exact_inexact_numeric_intransitivity_at_2p53() {
        let two53: f64 = 9_007_199_254_740_992.0;
        // x = xsd:integer 2^53 (exact tier)
        let x = T::ExactNum { f: two53, mant: 9_007_199_254_740_992_i128, scale: 0 };
        // y = xsd:double 2^53 (inexact tier — no exact_cmp)
        let y = T::NumLit(two53);
        // z = xsd:integer 2^53+1 (exact tier; shares f64 image with x)
        let z = T::ExactNum { f: two53, mant: 9_007_199_254_740_993_i128, scale: 0 };
        // Confirm the witnesses: x = y, y = z (the comparator is defined and returns Equal)
        assert_eq!(compare_terms(&x, &y), Some(Ordering::Equal), "x ~ y");
        assert_eq!(compare_terms(&y, &z), Some(Ordering::Equal), "y ~ z");
        // Transitivity of equality would require: x = z.
        // Currently FAILS (returns Less) — the exact recheck orders x < z correctly, but
        // inconsistently with the mixed-tier Equal above.
        assert_eq!(compare_terms(&x, &z), Some(Ordering::Equal), "transitivity: x=y AND y=z => x=z");
    }

    /// BUG witness 2 — numeric-vs-plain-string cross-family fallback to lexical order.
    ///
    /// Cross-family literal pairs (numeric vs plain-string) have no strict_cmp, so
    /// `compare_terms` falls back to the LEXICAL string form. Lexical digit-string order
    /// inverts numeric order on certain pairs: `"10" < "11" < "2"` (lexical) while
    /// `10 > 2` (numeric). The triple `(Int 10, StrLit "11", Int 2)` is an intransitive
    /// witness: `10 < "11"` and `"11" < 2` (lexical) but `10 > 2` (numeric).
    /// Tracked as P1 correctness bead sq-wjl8i.
    #[test]
    #[ignore = "BUG sq-wjl8i: numeric-vs-plain-string lexical-fallback intransitivity — remove once fixed"]
    fn bug_witness_numeric_vs_string_lexical_fallback_intransitivity() {
        // x = integer 10, z = integer 2: compare_terms(x, z) = Greater (10 > 2 numerically)
        let x = T::ExactNum { f: 10.0, mant: 10, scale: 0 };
        let z = T::ExactNum { f: 2.0, mant: 2, scale: 0 };
        // y = plain-string "11": compare_terms(x, y) falls back to lexical "10" < "11" (Less)
        // and compare_terms(y, z) falls back to lexical "11" < "2" (Less)
        let y = T::StrLit("11".into());
        // Confirm the two Less legs (current behaviour)
        assert_eq!(compare_terms(&x, &y), Some(Ordering::Less), "\"10\" < \"11\" lexically");
        assert_eq!(compare_terms(&y, &z), Some(Ordering::Less), "\"11\" < \"2\" lexically");
        // Transitivity of the strict-order Less would require: x < z.
        // Currently FAILS (returns Greater) — numeric order disagrees with lexical.
        assert_eq!(compare_terms(&x, &z), Some(Ordering::Less), "transitivity: x<y AND y<z => x<z");
    }

    /// BUG witness 3 — NaN makes the comparator partial on same-class literal pairs.
    ///
    /// `xsd:double NaN` has an f64 form (`NaN`), so `as_f64` returns `Some(NaN)`. The numeric
    /// `partial_cmp` then returns `None` for every NaN comparison. Callers that map `None` to
    /// `Equal` effectively place NaN equal to every numeric, collapsing distinct equivalence
    /// classes. Within-class totality is violated: `compare_terms(NaN, Int(0))` returns `None`
    /// even though both are `TermClass::Literal`. Tracked as P1 correctness bead sq-wjl8i.
    #[test]
    #[ignore = "BUG sq-wjl8i: NaN makes comparator partial on same-class Literal pairs — remove once fixed"]
    fn bug_witness_nan_partiality_violates_within_class_totality() {
        let nan = T::NumLit(f64::NAN);
        let zero = T::NumLit(0.0);
        // Both are TermClass::Literal — within-class totality requires Some(_).
        // Currently returns None (f64::NAN.partial_cmp(0.0) = None).
        assert!(
            compare_terms(&nan, &zero).is_some(),
            "within-class totality: Literal NaN vs Literal 0 must compare Some"
        );
    }
}
