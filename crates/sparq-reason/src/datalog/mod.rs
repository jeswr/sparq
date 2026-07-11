//! [FABLE-5] sq-6tykl.3 (epic sq-6tykl, RDFox-parity BIG ROCK) — **stratified Datalog
//! rules** with negation-as-failure and aggregation, behind the opt-in `datalog` feature.
//!
//! This is Phase 1 of the stratified-Datalog program (design record
//! `research/stratified-datalog-rules.md`): a small **native rule dialect** modelled on
//! RDFox's Datalog surface (the maintainer's open question 4 — see the record §2 for the
//! decision), a **stratification checker**, and a **non-incremental per-stratum
//! evaluator** supporting `NOT` (store-scoped negation as failure) and
//! `AGGREGATE … BIND COUNT(?v) AS ?c` atoms plus a minimal numeric `FILTER`. Semi-naive
//! per-stratum evaluation, the remaining aggregate functions (`SUM`/`MIN`/`MAX`/`AVG`)
//! and incremental maintenance under insert/delete are **later phases**, beaded from the
//! design record — this module is honest about being the fixture-scale foundation.
//!
//! # The dialect (Phase-1 fragment)
//!
//! ```text
//! @prefix ex: <http://example.org/> .
//!
//! [?y, ex:reach, "y"] :- [?x, ex:reach, "y"], [?x, ex:edge, ?y] .          # recursion
//! [?x, ex:orphan, "y"] :- [?x, a, ex:Node], NOT [?p, ex:child, ?x] .      # NAF
//! [?x, ex:deg, ?c]     :- AGGREGATE([?x, ex:edge, ?y] ON ?x
//!                                    BIND COUNT(?y) AS ?c) .               # aggregation
//! [?x, a, ex:Hub]      :- [?x, ex:deg, ?c], FILTER(?c >= 3) .             # threshold
//! ```
//!
//! * **Atoms** are triple patterns `[s, p, o]` with a **constant predicate** (an IRI or
//!   `a` = `rdf:type`); subjects/objects are IRIs, literals (bare integers are
//!   `xsd:integer`; `"…"` with optional `^^dt`), or `?variables`.
//! * **`NOT atom`** is negation as failure against the completed lower strata. Variables
//!   not bound by a positive atom are existential wildcards (a repeated wildcard inside
//!   one `NOT` atom must match equal terms).
//! * **`AGGREGATE(atoms ON ?g… BIND COUNT(?v) AS ?c)`** groups the DISTINCT matches of
//!   its positive body (set semantics — Datalog relations are sets, so `COUNT(?v)`
//!   counts distinct body matches per group; there are no rows for empty groups) by the
//!   `ON` variables and binds the count (an `xsd:integer`) to `?c`. Body variables other
//!   than the `ON` list are aggregate-local. `ON` may be omitted for a global count.
//! * **`FILTER(x op y)`** compares two bound-or-constant EXACT numeric values
//!   (`xsd:integer`/`xsd:decimal` + the derived integer types) with
//!   `= != < <= > >=` via the shared [`sparq_substrate::numeric::Dec`] tower —
//!   non-numeric or float/double operands fail the row (fail-closed; floats are a
//!   later-phase decision, see the record §5).
//!
//! Everything else is a **loud [`parse_program`] error**, never a silent divergence:
//! variable predicates, `SUM`/`MIN`/`MAX`/`AVG` (named as not-yet-implemented), nested
//! `NOT`/`AGGREGATE` inside an aggregate body, unbound head/filter variables, and
//! aggregate-local variable capture.
//!
//! # Stratified semantics
//!
//! [`stratify`] builds the predicate dependency graph (an edge per body atom of each
//! rule's head predicate; `NOT` atoms and every predicate inside an `AGGREGATE` body are
//! **non-monotonic** edges) and either assigns each rule a stratum or rejects the
//! program with an error naming a predicate on a negation/aggregation cycle. [`eval`]
//! then runs a fixpoint per stratum in order, so every negated or aggregated predicate
//! is **complete** before it is read — the textbook perfect-model semantics, and the
//! checked replacement for the caller-discipline stratification the N3 compiled path
//! documents (`crate::n3::compiled`).
//!
//! # Join machinery
//!
//! Positive-atom and aggregate-table joins drive the SHARED [`sparq_substrate::join`]
//! kernels ([`sparq_substrate::join::build_table`] +
//! [`sparq_substrate::join::hash_probe_serial`] — the same monomorphic bodies the SPARQL
//! engine, the RDFS materialiser and the N3 compiled path drive; deliberately **not**
//! another join implementation, only this module's thin layout adapter over them).
//! `FILTER` numerics delegate to the shared substrate `numeric` tower — the same
//! `Dec` the engine's FILTER path uses, so the two can never diverge on exact-decimal
//! comparison.
//!
//! The correctness oracle is the in-module differential suite: an independent naive
//! substitution-based evaluator (`oracle`, test-only — no substrate kernels, no
//! indexes) must agree with the kernel-driven evaluator on every fixture and on
//! seed-randomised graphs.

use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Dict, Id};

mod eval;
#[cfg(test)]
mod oracle;
mod parser;
mod stratify;

pub use stratify::{stratify, Stratification};

/// One term position of a parsed atom: a dictionary constant or a rule-scoped
/// (or aggregate-local) variable slot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DTerm {
    /// A ground term, already interned into the caller's [`Dict`].
    Const(Id),
    /// A variable slot (a column of the binding row in its scope).
    Var(u32),
}

/// One triple-pattern atom. The predicate is always a constant in the Phase-1
/// fragment (`pred` duplicates `t[1]` for direct access).
#[derive(Clone, Debug)]
pub(crate) struct Atom {
    pub(crate) t: [DTerm; 3],
    pub(crate) pred: Id,
}

/// The aggregate functions of the surface syntax. Phase 1 implements `COUNT`;
/// the others parse to a loud error naming the follow-up phase.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AggFunc {
    Count,
}

/// One `AGGREGATE(body ON ?g… BIND COUNT(?v) AS ?c)` atom. The body has its own
/// variable scope (`n_slots` local slots); `on` maps each grouping variable's
/// aggregate-local slot to its outer rule slot.
#[derive(Clone, Debug)]
pub(crate) struct AggAtom {
    pub(crate) body: Vec<Atom>,
    /// `(aggregate-local slot, outer rule slot)` per `ON` variable, in `ON` order.
    pub(crate) on: Vec<(u32, u32)>,
    pub(crate) func: AggFunc,
    /// Aggregate-local slot of the counted variable (must occur in `body`).
    pub(crate) counted: u32,
    /// Outer rule slot the aggregate value binds to (fresh in the outer scope).
    pub(crate) out: u32,
    /// Aggregate-local slot count.
    pub(crate) n_slots: usize,
}

/// A `FILTER` comparison operator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// One `FILTER(a op b)` condition over bound variables / numeric constants.
#[derive(Clone, Debug)]
pub(crate) struct Filter {
    pub(crate) a: DTerm,
    pub(crate) op: CmpOp,
    pub(crate) b: DTerm,
}

/// One parsed rule. Body elements are grouped by kind; positive atoms keep their
/// source order (the join order), and `NOT`/`FILTER` run after all joins — every
/// correlated variable is bound by then (checked at parse time).
#[derive(Clone, Debug)]
pub(crate) struct Rule {
    pub(crate) head: Vec<Atom>,
    pub(crate) positive: Vec<Atom>,
    pub(crate) aggregates: Vec<AggAtom>,
    pub(crate) negated: Vec<Atom>,
    pub(crate) filters: Vec<Filter>,
    /// Rule-scope slot count (head + positive + NOT + `ON` + `AS` variables).
    pub(crate) n_slots: usize,
}

/// A parsed, validated stratified-Datalog program whose constants are interned
/// into the [`Dict`] passed to [`parse_program`]. Evaluate with [`eval`] (or check
/// stratifiability alone with [`stratify`]).
#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) rules: Vec<Rule>,
}

impl Program {
    /// Number of parsed rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_reason::datalog::parse_program;
    /// let mut dict = sparq_core::dict::Dict::new();
    /// let p = parse_program(
    ///     &mut dict,
    ///     "@prefix ex: <http://ex/> . [?x, ex:q, ?y] :- [?x, ex:p, ?y] .",
    /// )?;
    /// assert_eq!(p.n_rules(), 1);
    /// # Ok::<(), String>(())
    /// ```
    pub fn n_rules(&self) -> usize {
        self.rules.len()
    }
}

/// Parse a stratified-Datalog rule document (the Phase-1 fragment — see the module
/// docs) and intern its ground terms into `dict`.
///
/// # Errors
///
/// Returns `Err` on a syntax error or on any construct outside the documented
/// fragment (variable predicates, unimplemented aggregate functions, unbound head
/// or `FILTER` variables, aggregate-local variable capture, …) — always a loud
/// error naming the construct, never a silent divergence.
///
/// # Examples
///
/// ```
/// use sparq_reason::datalog::parse_program;
/// let mut dict = sparq_core::dict::Dict::new();
/// let p = parse_program(
///     &mut dict,
///     "@prefix ex: <http://ex/> .
///      [?x, ex:big, \"y\"] :- AGGREGATE([?x, ex:member, ?m] ON ?x
///                                       BIND COUNT(?m) AS ?n),
///                             FILTER(?n >= 2) .",
/// )?;
/// assert_eq!(p.n_rules(), 1);
/// # Ok::<(), String>(())
/// ```
pub fn parse_program(dict: &mut Dict, src: &str) -> Result<Program, String> {
    parser::parse(dict, src)
}

/// Check stratifiability and evaluate `program` over `facts`: run the per-stratum
/// forward fixpoint to completion and return the full ground closure — input facts
/// plus every derivation, de-duplicated. Treat the result as a SET (order is
/// unspecified). Non-incremental Phase-1 evaluation (naive rounds per stratum);
/// `dict` is needed mutably because aggregation MINTS `xsd:integer` count literals
/// into the caller's id space.
///
/// # Errors
///
/// Returns `Err` (via [`stratify`]) when the program has a cycle through negation
/// or aggregation — such programs have no stratified model and are rejected, never
/// silently mis-evaluated.
///
/// # Examples
///
/// ```
/// use sparq_reason::datalog::{eval, parse_program};
/// let mut dict = sparq_core::dict::Dict::new();
/// let p = parse_program(
///     &mut dict,
///     "@prefix ex: <http://ex/> . [?x, ex:q, ?y] :- [?x, ex:p, ?y] .",
/// )?;
/// let (s, pr, o) = (
///     dict.intern_iri("http://ex/s"),
///     dict.intern_iri("http://ex/p"),
///     dict.intern_iri("http://ex/o"),
/// );
/// let closure = eval(&mut dict, &[[s, pr, o]], &p)?;
/// assert_eq!(closure.len(), 2); // the fact + the derivation
/// # Ok::<(), String>(())
/// ```
pub fn eval(dict: &mut Dict, facts: &[[Id; 3]], program: &Program) -> Result<Vec<[Id; 3]>, String> {
    let strat = stratify(dict, program)?;
    Ok(eval::eval_stratified(dict, facts, program, &strat))
}

/// The XSD datatypes whose literals the `FILTER` comparison accepts as EXACT
/// numerics (Phase 1: the `Dec`-representable integer/decimal family; float/double
/// are a later-phase decision — see the design record §5).
pub(crate) const NUMERIC_XSD: &[&str] = &[
    "integer",
    "decimal",
    "long",
    "int",
    "short",
    "byte",
    "nonNegativeInteger",
    "nonPositiveInteger",
    "negativeInteger",
    "positiveInteger",
    "unsignedLong",
    "unsignedInt",
    "unsignedShort",
    "unsignedByte",
];

/// Resolve a dictionary id to its exact numeric value, if it is a literal of a
/// recognised exact-numeric XSD datatype with a valid lexical form.
pub(crate) fn numeric_value(dict: &Dict, id: Id) -> Option<sparq_substrate::numeric::Dec> {
    let oxrdf::Term::Literal(l) = dict.term(id) else {
        return None;
    };
    let local = l
        .datatype()
        .as_str()
        .strip_prefix("http://www.w3.org/2001/XMLSchema#")?;
    if !NUMERIC_XSD.contains(&local) {
        return None;
    }
    sparq_substrate::numeric::Dec::parse_lexical(l.value())
}

/// The set of variable slots a rule's positive machinery binds: positive-atom
/// variables, aggregate `ON` variables (outer slots) and aggregate outputs. Head,
/// `FILTER` and correlated `NOT` variables must come from this set.
pub(crate) fn bound_slots(rule: &Rule) -> FxHashSet<u32> {
    let mut bound: FxHashSet<u32> = FxHashSet::default();
    for a in &rule.positive {
        for t in &a.t {
            if let DTerm::Var(v) = t {
                bound.insert(*v);
            }
        }
    }
    for agg in &rule.aggregates {
        for &(_, outer) in &agg.on {
            bound.insert(outer);
        }
        bound.insert(agg.out);
    }
    bound
}

/// Predicate-indexed fact store shared by the evaluator: the de-duplicating set,
/// the append-only list (the closure in insertion order) and a per-predicate index.
#[derive(Default)]
pub(crate) struct FactStore {
    pub(crate) all: FxHashSet<[Id; 3]>,
    pub(crate) list: Vec<[Id; 3]>,
    pub(crate) by_pred: FxHashMap<Id, Vec<[Id; 3]>>,
}

impl FactStore {
    /// Insert a fact; returns `true` when new.
    pub(crate) fn insert(&mut self, f: [Id; 3]) -> bool {
        if self.all.insert(f) {
            self.list.push(f);
            self.by_pred.entry(f[1]).or_default().push(f);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests;
