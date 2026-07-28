//! L3 (part 0) — the **concrete-domain satisfiability oracle** for the datatype-aware
//! ALCH(D) fragment (opt-in `dl_datatypes`).
//!
//! 🤖 SPARQ agent [SONNET-4.6]. Bead sq-pbz04.4.19 (epic sq-pbz04.4, design record
//! `research/owl2-direct-semantics-scoping.md` §5c). This module decides ONE question, and
//! the tableau (`crate::tableau`) asks it about every concrete (data) node it builds:
//!
//! > given a finite conjunction of data-range literals `D₁ ⊓ … ⊓ Dₘ ⊓ ¬E₁ ⊓ … ⊓ ¬Eₙ` over
//! > the ADMITTED datatype set [`Datatype`], is there a data value satisfying all of them?
//!
//! The answer is **exact** (never approximate, never three-valued): the admitted set is a
//! deliberately small, closed sub-lattice of the OWL 2 datatype map over which value-space
//! containment and disjointness are fully determined, and the L1 extractor
//! (`crate::extract`) refuses — fail-closed, exactly as before sq-pbz04.4.19 — every
//! datatype construct this module cannot decide. So the tableau stays TWO-valued: it never
//! has to propagate an "oracle abstained" state.
//!
//! # 1. The admitted set and why each membership is decidable
//!
//! [`Datatype`] enumerates the admitted datatypes. They partition into value-space
//! **families** which the OWL 2 datatype map makes pairwise DISJOINT (OWL 2 Structural
//! Specification §4, "Datatype Maps"; the same partition the in-repo D-entailment value
//! seam `sparq_reason::dtype::d_value_key` already encodes as distinct `DValue` variants —
//! see the differential-parity test in `tests/cdomain.rs`):
//!
//! | Family | Members | Internal structure |
//! |---|---|---|
//! | ⊤ | `rdfs:Literal` | the union of EVERY datatype value space (also the ones NOT admitted) |
//! | real | `owl:real` ⊃ `owl:rational` ⊃ `xsd:decimal` ⊃ ℤ-derived | a 4-level tier chain; the ℤ level is an interval over the integers |
//! | string | `xsd:string` | one member |
//! | boolean | `xsd:boolean` | one member |
//! | dateTime | `xsd:dateTime` | one member |
//! | anyURI | `xsd:anyURI` | one member |
//!
//! The 13 integer-derived types (`xsd:integer` and its bounded / sign-constrained
//! restrictions) are modelled EXACTLY as integer intervals `[lo, hi]` with `Option<i128>`
//! endpoints (`None` = unbounded), because their value spaces differ from `xsd:integer`
//! only by the `minInclusive`/`maxInclusive` facets XSD 1.1 §3.4 fixes for each. Two of
//! them are disjoint iff their intervals are, and one contains the other iff its interval
//! does — so the sub-lattice is not a tree (`xsd:long` and `xsd:nonNegativeInteger` are
//! incomparable but overlap on `[0, 2⁶³−1]`) and interval arithmetic, not a tree walk, is
//! what decides it.
//!
//! # 2. What is deliberately NOT admitted (and stays a fail-closed L1 refusal)
//!
//! `xsd:float` / `xsd:double` (their relationship to `owl:real` and to each other is
//! exactly the point the OWL 2 datatype map treats specially, and getting it wrong would
//! silently manufacture verdicts), `xsd:hexBinary` / `xsd:base64Binary` (XSD 1.1 §3.3.15–16
//! give them the SAME value space, so they are equal rather than disjoint — admitting one
//! without the other would be a trap), `rdf:PlainLiteral` / `rdf:XMLLiteral` and the
//! token-derived string types (`xsd:token`, `xsd:language`, `xsd:Name`, … — pairwise
//! non-disjoint in ways a chain model does not capture: `"en"` is BOTH a valid
//! `xsd:language` and a valid `xsd:Name`), `xsd:date` / `xsd:time` / the gregorian types,
//! all **facets** (`owl:withRestrictions`, `owl:onDatatype`), `owl:datatypeComplementOf`,
//! `owl:oneOf` data ranges, and literals in any position. Each of those remains an
//! `ExtractError::DataConstruct` — the design-record §5 deferral, narrowed but not closed.
//!
//! # 3. Non-emptiness witnesses (the load-bearing facts of §4's decision procedure)
//!
//! The procedure below concludes "non-empty" only from these, each of which is a fact about
//! the OWL 2 datatype map, not about this implementation:
//!
//! - **W1.** `owl:real ∖ owl:rational ≠ ∅` (√2 is real and not rational).
//! - **W2.** `owl:rational ∖ xsd:decimal ≠ ∅` (⅓ is rational and has no finite decimal form).
//! - **W3.** `xsd:decimal ∖ ℤ ≠ ∅` (0.5), and removing FINITELY many integer intervals from
//!   the decimals cannot exhaust them (between any two integers lies a decimal).
//! - **W4.** `rdfs:Literal` strictly contains the union of every ADMITTED family — witness:
//!   a language-tagged `rdf:PlainLiteral` value such as `"a"@en`, which lies in no admitted
//!   family. This is what makes "no positive datatype at all" satisfiable under any set of
//!   admitted negations.
//! - **W5.** Every admitted datatype has a NON-EMPTY value space.
//!
//! # 4. The decision procedure
//!
//! Let `P` be the positive datatypes and `N` the negated ones.
//!
//! 1. If `rdfs:Literal ∈ N` the answer is **UNSAT**: `¬⊤_D` is `⊥_D`.
//! 2. Drop `rdfs:Literal` from `P` (it is `⊤_D` and constrains nothing).
//! 3. If the remaining `P` mentions **two distinct families**, the answer is **UNSAT**
//!    (families are pairwise disjoint).
//! 4. If `P` is now empty the region is the whole datatype universe; by **W4** the union of
//!    the admitted families is a PROPER subset of it, so the answer is **SAT**.
//! 5. Otherwise the region `R` is the intersection inside the single pinned family:
//!    - **string / boolean / dateTime / anyURI** — `R` is that one value space (**W5**,
//!      non-empty); it survives iff that datatype is not negated.
//!    - **real** — `R` is the MINIMUM tier present, with the ℤ-level interval intersected
//!      across every integer-derived member. A negated datatype of a tier at least as high
//!      as `R`'s covers `R` entirely and kills it; a negated datatype of a strictly LOWER
//!      tier cannot (**W1**/**W2**/**W3**). When `R` is at the ℤ level the surviving set is
//!      `R`'s interval minus the negated intervals — decided by exact interval subtraction.
//!
//! Every step is a total function of `(P, N)` with no floating point, no wall clock and no
//! allocation beyond a handful of small vectors, so the oracle is deterministic — the
//! property the tableau's reproducibility contract (`crate::tableau` module docs §6)
//! requires of everything it calls.
//!
//! # 5. Relationship to the shared value seams
//!
//! This module performs NO value arithmetic of its own: the admitted sub-lattice is decided
//! structurally (families, tiers, integer intervals), so there is no private numeric tower
//! here to drift from `sparq_substrate::numeric`. What IS cross-checked is the lattice
//! itself: `tests/cdomain.rs` runs a differential over the XSD literal matrix against
//! `sparq_reason::dtype::d_value_key` — the repo's existing D-entailment value seam — and
//! asserts EXACT agreement between "this oracle says the pair intersects" and "a literal
//! exists whose value lies in both". Facet-bearing ranges are out of scope here precisely
//! because deciding them DOES need the shared exact-decimal tower, as
//! `sparq_reason_el::cdomain` already demonstrates; that is the next unlock, not this one.

/// The XSD namespace.
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
/// The OWL namespace.
const OWL: &str = "http://www.w3.org/2002/07/owl#";
/// `rdfs:Literal`, the top data range.
const RDFS_LITERAL: &str = "http://www.w3.org/2000/01/rdf-schema#Literal";

/// A datatype of the ADMITTED sub-lattice of the OWL 2 datatype map (module docs §1).
///
/// This is a CLOSED enum on purpose: the L1 extractor resolves a datatype IRI to one of
/// these variants once, and everything downstream (the structural model, the NNF pass, the
/// tableau, this oracle) works on the resolved variant — so no layer below L1 needs a
/// `Dict`, and no unadmitted datatype can reach the oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Datatype {
    /// `rdfs:Literal` — ⊤_D, the union of every datatype value space.
    RdfsLiteral,
    /// `owl:real`.
    OwlReal,
    /// `owl:rational`.
    OwlRational,
    /// `xsd:decimal`.
    XsdDecimal,
    /// `xsd:integer`.
    XsdInteger,
    /// `xsd:long`.
    XsdLong,
    /// `xsd:int`.
    XsdInt,
    /// `xsd:short`.
    XsdShort,
    /// `xsd:byte`.
    XsdByte,
    /// `xsd:nonNegativeInteger`.
    XsdNonNegativeInteger,
    /// `xsd:positiveInteger`.
    XsdPositiveInteger,
    /// `xsd:nonPositiveInteger`.
    XsdNonPositiveInteger,
    /// `xsd:negativeInteger`.
    XsdNegativeInteger,
    /// `xsd:unsignedLong`.
    XsdUnsignedLong,
    /// `xsd:unsignedInt`.
    XsdUnsignedInt,
    /// `xsd:unsignedShort`.
    XsdUnsignedShort,
    /// `xsd:unsignedByte`.
    XsdUnsignedByte,
    /// `xsd:string`.
    XsdString,
    /// `xsd:boolean`.
    XsdBoolean,
    /// `xsd:dateTime`.
    XsdDateTime,
    /// `xsd:anyURI`.
    XsdAnyUri,
}

/// Every admitted datatype, in declaration order. Used by the exhaustiveness tests and the
/// differential-parity lane; the oracle itself never iterates it.
pub const ALL_DATATYPES: &[Datatype] = &[
    Datatype::RdfsLiteral,
    Datatype::OwlReal,
    Datatype::OwlRational,
    Datatype::XsdDecimal,
    Datatype::XsdInteger,
    Datatype::XsdLong,
    Datatype::XsdInt,
    Datatype::XsdShort,
    Datatype::XsdByte,
    Datatype::XsdNonNegativeInteger,
    Datatype::XsdPositiveInteger,
    Datatype::XsdNonPositiveInteger,
    Datatype::XsdNegativeInteger,
    Datatype::XsdUnsignedLong,
    Datatype::XsdUnsignedInt,
    Datatype::XsdUnsignedShort,
    Datatype::XsdUnsignedByte,
    Datatype::XsdString,
    Datatype::XsdBoolean,
    Datatype::XsdDateTime,
    Datatype::XsdAnyUri,
];

/// The value-space family a datatype belongs to. Distinct concrete families are PAIRWISE
/// DISJOINT in the OWL 2 datatype map (module docs §1); `Family::Top` is `rdfs:Literal`,
/// which is not a family but their union (and more — witness **W4**).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Top,
    Real,
    Str,
    Bool,
    DateTime,
    AnyUri,
}

/// The tier of a member of the `owl:real` family, ordered by value-space CONTAINMENT:
/// `Int ⊂ Decimal ⊂ Rational ⊂ Real` (module docs §3, witnesses W1–W3). Derived `Ord` gives
/// exactly that order from the declaration order — the decision procedure relies on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Int,
    Decimal,
    Rational,
    Real,
}

/// A (possibly half-open) integer interval; `None` is unbounded on that side.
type Interval = (Option<i128>, Option<i128>);

/// The value-space descriptor of one admitted datatype.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    /// `rdfs:Literal`.
    Top,
    /// A member of the `owl:real` family at `Tier`; the interval is meaningful only at
    /// `Tier::Int` (the other tiers are not restricted to the integers at all).
    Real(Tier, Interval),
    Str,
    Bool,
    DateTime,
    AnyUri,
}

/// The largest magnitude any bound in the `Datatype::space` table may take. Keeps the
/// `±1` endpoint arithmetic in `subtract_interval` far from `i128` overflow; enforced by the
/// `bounds_are_safe` debug assertion and pinned by `datatype_table_bounds_are_small`.
const MAX_BOUND_MAGNITUDE: i128 = 1 << 100;

impl Datatype {
    /// The datatype's IRI.
    #[must_use]
    pub fn iri(self) -> String {
        match self {
            Datatype::RdfsLiteral => RDFS_LITERAL.to_string(),
            Datatype::OwlReal => format!("{}real", OWL),
            Datatype::OwlRational => format!("{}rational", OWL),
            Datatype::XsdDecimal => format!("{}decimal", XSD),
            Datatype::XsdInteger => format!("{}integer", XSD),
            Datatype::XsdLong => format!("{}long", XSD),
            Datatype::XsdInt => format!("{}int", XSD),
            Datatype::XsdShort => format!("{}short", XSD),
            Datatype::XsdByte => format!("{}byte", XSD),
            Datatype::XsdNonNegativeInteger => format!("{}nonNegativeInteger", XSD),
            Datatype::XsdPositiveInteger => format!("{}positiveInteger", XSD),
            Datatype::XsdNonPositiveInteger => format!("{}nonPositiveInteger", XSD),
            Datatype::XsdNegativeInteger => format!("{}negativeInteger", XSD),
            Datatype::XsdUnsignedLong => format!("{}unsignedLong", XSD),
            Datatype::XsdUnsignedInt => format!("{}unsignedInt", XSD),
            Datatype::XsdUnsignedShort => format!("{}unsignedShort", XSD),
            Datatype::XsdUnsignedByte => format!("{}unsignedByte", XSD),
            Datatype::XsdString => format!("{}string", XSD),
            Datatype::XsdBoolean => format!("{}boolean", XSD),
            Datatype::XsdDateTime => format!("{}dateTime", XSD),
            Datatype::XsdAnyUri => format!("{}anyURI", XSD),
        }
    }

    /// Resolve a datatype IRI to its admitted variant, or `None` when the IRI is NOT in the
    /// admitted sub-lattice (module docs §2) — including when it is a perfectly good OWL 2
    /// datatype this oracle deliberately declines to decide. `None` is the extractor's
    /// signal to keep refusing fail-closed.
    #[must_use]
    pub fn from_iri(iri: &str) -> Option<Datatype> {
        if iri == RDFS_LITERAL {
            return Some(Datatype::RdfsLiteral);
        }
        if let Some(local) = iri.strip_prefix(OWL) {
            return match local {
                "real" => Some(Datatype::OwlReal),
                "rational" => Some(Datatype::OwlRational),
                _ => None,
            };
        }
        let local = iri.strip_prefix(XSD)?;
        match local {
            "decimal" => Some(Datatype::XsdDecimal),
            "integer" => Some(Datatype::XsdInteger),
            "long" => Some(Datatype::XsdLong),
            "int" => Some(Datatype::XsdInt),
            "short" => Some(Datatype::XsdShort),
            "byte" => Some(Datatype::XsdByte),
            "nonNegativeInteger" => Some(Datatype::XsdNonNegativeInteger),
            "positiveInteger" => Some(Datatype::XsdPositiveInteger),
            "nonPositiveInteger" => Some(Datatype::XsdNonPositiveInteger),
            "negativeInteger" => Some(Datatype::XsdNegativeInteger),
            "unsignedLong" => Some(Datatype::XsdUnsignedLong),
            "unsignedInt" => Some(Datatype::XsdUnsignedInt),
            "unsignedShort" => Some(Datatype::XsdUnsignedShort),
            "unsignedByte" => Some(Datatype::XsdUnsignedByte),
            "string" => Some(Datatype::XsdString),
            "boolean" => Some(Datatype::XsdBoolean),
            "dateTime" => Some(Datatype::XsdDateTime),
            "anyURI" => Some(Datatype::XsdAnyUri),
            _ => None,
        }
    }

    /// The datatype's value-space descriptor. The integer bounds are the XSD 1.1 §3.4
    /// `minInclusive`/`maxInclusive` facets of each derived integer type.
    fn space(self) -> Space {
        match self {
            Datatype::RdfsLiteral => Space::Top,
            Datatype::OwlReal => Space::Real(Tier::Real, (None, None)),
            Datatype::OwlRational => Space::Real(Tier::Rational, (None, None)),
            Datatype::XsdDecimal => Space::Real(Tier::Decimal, (None, None)),
            Datatype::XsdInteger => Space::Real(Tier::Int, (None, None)),
            Datatype::XsdLong => Space::Real(
                Tier::Int,
                (Some(i64::MIN as i128), Some(i64::MAX as i128)),
            ),
            Datatype::XsdInt => Space::Real(
                Tier::Int,
                (Some(i32::MIN as i128), Some(i32::MAX as i128)),
            ),
            Datatype::XsdShort => Space::Real(Tier::Int, (Some(-32768), Some(32767))),
            Datatype::XsdByte => Space::Real(Tier::Int, (Some(-128), Some(127))),
            Datatype::XsdNonNegativeInteger => Space::Real(Tier::Int, (Some(0), None)),
            Datatype::XsdPositiveInteger => Space::Real(Tier::Int, (Some(1), None)),
            Datatype::XsdNonPositiveInteger => Space::Real(Tier::Int, (None, Some(0))),
            Datatype::XsdNegativeInteger => Space::Real(Tier::Int, (None, Some(-1))),
            Datatype::XsdUnsignedLong => {
                Space::Real(Tier::Int, (Some(0), Some(18_446_744_073_709_551_615)))
            }
            Datatype::XsdUnsignedInt => Space::Real(Tier::Int, (Some(0), Some(4_294_967_295))),
            Datatype::XsdUnsignedShort => Space::Real(Tier::Int, (Some(0), Some(65535))),
            Datatype::XsdUnsignedByte => Space::Real(Tier::Int, (Some(0), Some(255))),
            Datatype::XsdString => Space::Str,
            Datatype::XsdBoolean => Space::Bool,
            Datatype::XsdDateTime => Space::DateTime,
            Datatype::XsdAnyUri => Space::AnyUri,
        }
    }

    /// The family whose value space contains this datatype's.
    fn family(self) -> Family {
        match self.space() {
            Space::Top => Family::Top,
            Space::Real(_, _) => Family::Real,
            Space::Str => Family::Str,
            Space::Bool => Family::Bool,
            Space::DateTime => Family::DateTime,
            Space::AnyUri => Family::AnyUri,
        }
    }
}

/// Decide whether the conjunction `⨅positive ⊓ ⨅{¬e | e ∈ negative}` has a non-empty value
/// space (module docs §4).
///
/// `true` means a data value satisfying every listed constraint provably EXISTS; `false`
/// means provably none does. Both directions are exact over the admitted set — there is no
/// abstention, which is what lets the tableau treat a concrete-domain clash exactly like a
/// `{A, ¬A}` clash. Duplicates in either slice are harmless. The empty conjunction is
/// satisfiable (`⊤_D` is non-empty).
#[must_use]
pub fn satisfiable(positive: &[Datatype], negative: &[Datatype]) -> bool {
    // Step 1 — ¬rdfs:Literal is ⊥_D: no data value lies outside the datatype universe.
    if negative.contains(&Datatype::RdfsLiteral) {
        return false;
    }
    // Steps 2–3 — rdfs:Literal is ⊤_D (drop it); two distinct concrete families cannot be
    // inhabited at once.
    let mut family: Option<Family> = None;
    for d in positive {
        let f = d.family();
        if f == Family::Top {
            continue;
        }
        match family {
            None => family = Some(f),
            Some(g) if g == f => {}
            Some(_) => return false,
        }
    }
    // Step 4 — no positive constraint at all: the region is the whole datatype universe,
    // which strictly contains the union of the admitted families (witness W4).
    let Some(family) = family else {
        return true;
    };
    // Step 5 — intersect, then subtract, inside the single pinned family.
    match family {
        Family::Str => !negative.contains(&Datatype::XsdString),
        Family::Bool => !negative.contains(&Datatype::XsdBoolean),
        Family::AnyUri => !negative.contains(&Datatype::XsdAnyUri),
        Family::DateTime => !negative.contains(&Datatype::XsdDateTime),
        Family::Real => real_satisfiable(positive, negative),
        // `Family::Top` is filtered out above; it can never be the pinned family.
        Family::Top => true,
    }
}

/// The `owl:real`-family arm of [`satisfiable`] (module docs §4, step 5 "real").
fn real_satisfiable(positive: &[Datatype], negative: &[Datatype]) -> bool {
    // The region's tier is the MINIMUM tier present; its integer interval is the
    // intersection of the intervals of every integer-derived member (the higher tiers are
    // supersets of ℤ, so they impose no interval).
    let mut tier = Tier::Real;
    let mut region: Interval = (None, None);
    for d in positive {
        if let Space::Real(t, iv) = d.space() {
            if t < tier {
                tier = t;
            }
            if t == Tier::Int {
                region = intersect_interval(region, iv);
            }
        }
    }
    let negated: Vec<(Tier, Interval)> = negative
        .iter()
        .filter_map(|d| match d.space() {
            Space::Real(t, iv) => Some((t, iv)),
            _ => None,
        })
        .collect();
    if tier == Tier::Int {
        // Any negated datatype at a tier ABOVE ℤ contains all of ℤ and so empties the
        // region outright; otherwise subtract the negated integer intervals exactly.
        if negated.iter().any(|&(t, _)| t > Tier::Int) {
            return false;
        }
        let mut pieces: Vec<Interval> = if interval_nonempty(region) {
            vec![region]
        } else {
            Vec::new()
        };
        for &(_, cut) in &negated {
            pieces = pieces
                .into_iter()
                .flat_map(|piece| subtract_interval(piece, cut))
                .collect();
            if pieces.is_empty() {
                return false;
            }
        }
        return !pieces.is_empty();
    }
    // The region is a whole dense tier (decimal / rational / real). A negated datatype of
    // the SAME or a HIGHER tier covers it; strictly lower ones cannot exhaust it (W1–W3).
    !negated.iter().any(|&(t, _)| t >= tier)
}

/// `true` iff the interval contains at least one integer.
fn interval_nonempty(iv: Interval) -> bool {
    match iv {
        (Some(lo), Some(hi)) => lo <= hi,
        _ => true,
    }
}

/// The intersection of two integer intervals (`None` endpoints are ∓∞).
fn intersect_interval(a: Interval, b: Interval) -> Interval {
    (max_lower(a.0, b.0), min_upper(a.1, b.1))
}

/// The tighter of two lower bounds (`None` = −∞).
fn max_lower(a: Option<i128>, b: Option<i128>) -> Option<i128> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

/// The tighter of two upper bounds (`None` = +∞).
fn min_upper(a: Option<i128>, b: Option<i128>) -> Option<i128> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

/// `true` iff both endpoints stay inside the `MAX_BOUND_MAGNITUDE` overflow-safety margin.
fn bounds_are_safe(iv: Interval) -> bool {
    [iv.0, iv.1]
        .into_iter()
        .flatten()
        .all(|b| b.abs() < MAX_BOUND_MAGNITUDE)
}

/// `iv ∖ cut` as at most two disjoint non-empty integer intervals.
///
/// The `±1` endpoint arithmetic cannot overflow: every bound reaching here comes from the
/// `Datatype::space` table or from an intersection of its entries, and that table's
/// magnitudes are bounded by `MAX_BOUND_MAGNITUDE` (pinned by
/// `datatype_table_bounds_are_small`).
fn subtract_interval(iv: Interval, cut: Interval) -> Vec<Interval> {
    debug_assert!(
        bounds_are_safe(iv) && bounds_are_safe(cut),
        "interval endpoints must stay inside the overflow-safety margin"
    );
    let mut out = Vec::new();
    if let Some(cut_lo) = cut.0 {
        // Everything in `iv` strictly below the cut survives.
        let piece = (iv.0, min_upper(iv.1, Some(cut_lo - 1)));
        if interval_nonempty(piece) {
            out.push(piece);
        }
    }
    if let Some(cut_hi) = cut.1 {
        // Everything in `iv` strictly above the cut survives.
        let piece = (max_lower(iv.0, Some(cut_hi + 1)), iv.1);
        if interval_nonempty(piece) {
            out.push(piece);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `±1` endpoint arithmetic in `subtract_interval` is overflow-free only because
    /// every table bound is small. Pin that precondition directly.
    #[test]
    fn datatype_table_bounds_are_small() {
        for &d in ALL_DATATYPES {
            if let Space::Real(_, (lo, hi)) = d.space() {
                for bound in [lo, hi].into_iter().flatten() {
                    assert!(
                        bound.abs() < MAX_BOUND_MAGNITUDE,
                        "{} bound {} exceeds the overflow-safety margin",
                        d.iri(),
                        bound
                    );
                }
            }
        }
    }

    /// `from_iri` and `iri` round-trip over the whole admitted set, and a datatype the
    /// oracle deliberately declines resolves to `None` (so L1 keeps refusing it).
    #[test]
    fn iri_round_trip_and_declined_datatypes() {
        for &d in ALL_DATATYPES {
            assert_eq!(Datatype::from_iri(&d.iri()), Some(d), "round-trip {:?}", d);
        }
        for declined in [
            "http://www.w3.org/2001/XMLSchema#double",
            "http://www.w3.org/2001/XMLSchema#float",
            "http://www.w3.org/2001/XMLSchema#token",
            "http://www.w3.org/2001/XMLSchema#language",
            "http://www.w3.org/2001/XMLSchema#hexBinary",
            "http://www.w3.org/2001/XMLSchema#date",
            "http://www.w3.org/2001/XMLSchema#dateTimeStamp",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#PlainLiteral",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral",
            "http://www.w3.org/2002/07/owl#Thing",
            "http://ex/NotADatatype",
        ] {
            assert_eq!(
                Datatype::from_iri(declined),
                None,
                "{} must stay unadmitted (fail-closed at L1)",
                declined
            );
        }
    }

    /// The headline unlock (WebOnt-I5.3-015): two ranges from DISJOINT families cannot be
    /// jointly inhabited, while comparable ones can.
    #[test]
    fn cross_family_pairs_are_unsatisfiable() {
        assert!(!satisfiable(
            &[Datatype::XsdInteger, Datatype::XsdString],
            &[]
        ));
        assert!(!satisfiable(
            &[Datatype::XsdBoolean, Datatype::XsdDateTime],
            &[]
        ));
        assert!(!satisfiable(&[Datatype::XsdAnyUri, Datatype::XsdString], &[]));
        // rdfs:Literal is ⊤_D — it never contributes a family clash.
        assert!(satisfiable(
            &[Datatype::RdfsLiteral, Datatype::XsdString],
            &[]
        ));
        // Comparable members of one family intersect.
        assert!(satisfiable(
            &[Datatype::XsdInteger, Datatype::XsdDecimal, Datatype::OwlReal],
            &[]
        ));
    }

    /// The integer sub-lattice is an INTERVAL lattice, not a tree: incomparable types can
    /// still overlap, and disjointness is decided by the intervals.
    #[test]
    fn integer_intervals_decide_overlap_and_disjointness() {
        // long ∩ nonNegativeInteger = [0, 2⁶³−1] — incomparable but non-empty.
        assert!(satisfiable(
            &[Datatype::XsdLong, Datatype::XsdNonNegativeInteger],
            &[]
        ));
        // byte ∩ nonNegativeInteger = [0, 127].
        assert!(satisfiable(
            &[Datatype::XsdByte, Datatype::XsdNonNegativeInteger],
            &[]
        ));
        // positiveInteger ∩ nonPositiveInteger = ∅.
        assert!(!satisfiable(
            &[Datatype::XsdPositiveInteger, Datatype::XsdNonPositiveInteger],
            &[]
        ));
        // negativeInteger ∩ unsignedByte = ∅.
        assert!(!satisfiable(
            &[Datatype::XsdNegativeInteger, Datatype::XsdUnsignedByte],
            &[]
        ));
        // A three-way intersection that empties only when all three are taken together.
        assert!(satisfiable(
            &[Datatype::XsdByte, Datatype::XsdNonNegativeInteger],
            &[]
        ));
        assert!(!satisfiable(
            &[
                Datatype::XsdByte,
                Datatype::XsdNonNegativeInteger,
                Datatype::XsdUnsignedShort,
                Datatype::XsdNegativeInteger
            ],
            &[]
        ));
    }

    /// Negation: `¬rdfs:Literal` is ⊥_D, a datatype minus itself is empty, and a datatype
    /// minus a strictly SMALLER one survives.
    #[test]
    fn negation_uses_the_containment_order() {
        assert!(!satisfiable(&[], &[Datatype::RdfsLiteral]));
        assert!(!satisfiable(&[Datatype::XsdString], &[Datatype::XsdString]));
        assert!(!satisfiable(&[Datatype::XsdByte], &[Datatype::XsdShort]));
        // integer ∖ byte is non-empty (128 survives).
        assert!(satisfiable(&[Datatype::XsdInteger], &[Datatype::XsdByte]));
        // decimal ∖ integer is non-empty (0.5 survives — witness W3).
        assert!(satisfiable(&[Datatype::XsdDecimal], &[Datatype::XsdInteger]));
        // integer ∖ decimal is EMPTY (decimal ⊇ ℤ).
        assert!(!satisfiable(&[Datatype::XsdInteger], &[Datatype::XsdDecimal]));
        // real ∖ rational is non-empty (√2 — witness W1); rational ∖ real is empty.
        assert!(satisfiable(&[Datatype::OwlReal], &[Datatype::OwlRational]));
        assert!(!satisfiable(&[Datatype::OwlRational], &[Datatype::OwlReal]));
        // rational ∖ decimal is non-empty (⅓ — witness W2).
        assert!(satisfiable(&[Datatype::OwlRational], &[Datatype::XsdDecimal]));
        // Negating a datatype from ANOTHER family constrains nothing.
        assert!(satisfiable(&[Datatype::XsdInteger], &[Datatype::XsdString]));
        // No positive datatype at all survives any set of admitted negations (witness W4).
        assert!(satisfiable(
            &[],
            &[
                Datatype::XsdString,
                Datatype::XsdBoolean,
                Datatype::XsdAnyUri,
                Datatype::XsdDateTime,
                Datatype::OwlReal
            ]
        ));
        // …but `rdfs:Literal` as the sole POSITIVE constraint behaves the same way.
        assert!(satisfiable(&[Datatype::RdfsLiteral], &[Datatype::XsdString]));
    }

    /// The singleton families each survive exactly until their own datatype is negated.
    #[test]
    fn singleton_families() {
        for d in [
            Datatype::XsdDateTime,
            Datatype::XsdString,
            Datatype::XsdBoolean,
            Datatype::XsdAnyUri,
        ] {
            assert!(satisfiable(&[d], &[]), "{:?} is inhabited", d);
            assert!(!satisfiable(&[d], &[d]), "{:?} minus itself is empty", d);
            assert!(
                satisfiable(&[d], &[Datatype::XsdInteger]),
                "{:?} is unconstrained by a negation from another family",
                d
            );
        }
    }

    /// The empty conjunction is satisfiable, and the oracle is insensitive to duplicates
    /// and to argument order (the tableau feeds it a set in interning order).
    #[test]
    fn empty_duplicate_and_order_insensitive() {
        assert!(satisfiable(&[], &[]));
        assert!(satisfiable(
            &[Datatype::XsdByte, Datatype::XsdByte, Datatype::XsdInteger],
            &[]
        ));
        assert!(!satisfiable(
            &[Datatype::XsdString, Datatype::XsdInteger],
            &[]
        ));
        assert!(!satisfiable(
            &[Datatype::XsdInteger, Datatype::XsdString],
            &[]
        ));
    }

    /// Direct coverage of the interval helpers, including the no-overlap case that a naive
    /// "left piece / right piece" split gets wrong.
    #[test]
    fn interval_subtraction_handles_disjoint_and_unbounded_cuts() {
        // Disjoint cut leaves the interval intact (as ONE piece, not two).
        assert_eq!(
            subtract_interval((Some(10), Some(20)), (Some(0), Some(5))),
            vec![(Some(10), Some(20))]
        );
        // Interior cut splits in two.
        assert_eq!(
            subtract_interval((Some(0), Some(10)), (Some(4), Some(6))),
            vec![(Some(0), Some(3)), (Some(7), Some(10))]
        );
        // Cut unbounded above removes the tail only.
        assert_eq!(
            subtract_interval((Some(0), Some(10)), (Some(5), None)),
            vec![(Some(0), Some(4))]
        );
        // Cutting ℤ by ℤ leaves nothing.
        assert!(subtract_interval((None, None), (None, None)).is_empty());
        // Covering cut leaves nothing.
        assert!(subtract_interval((Some(-128), Some(127)), (Some(-32768), Some(32767))).is_empty());
        assert!(interval_nonempty((Some(3), Some(3))));
        assert!(!interval_nonempty((Some(4), Some(3))));
        assert!(interval_nonempty((None, Some(3))));
        assert_eq!(
            intersect_interval((Some(0), None), (None, Some(7))),
            (Some(0), Some(7))
        );
    }
}
