//! [OPUS-4.8] sq-rh4gu (epic sq-pbz04) — a **RIF-Core** front-end over the N3
//! forward-chaining rule engine.
//!
//! RIF-Core is the W3C *Rule Interchange Format* **Core** dialect — the
//! **monotone Horn-rule** common subset of RIF-BLD (Basic Logic Dialect) and
//! RIF-PRD (Production Rule Dialect). This module accepts a RIF-Core rule set as
//! a faithful *in-engine representation* (the `Document` / `Rule` / `Atom` /
//! `Term` types below), **validates** it (range-restriction / DATALOG safety),
//! and **lowers** it to the existing N3 rule model so the proven
//! `reason_n3` forward chainer computes its closure. Reasoning is engaged only
//! behind the opt-in `rif-core` cargo feature — the lean default build carries
//! zero RIF code.
//!
//! ## Scope — RIF-Core, the MONOTONE Horn subset (honest boundary)
//!
//! This is **RIF-Core**, not full RIF-BLD or RIF-PRD. Specifically:
//!
//! * **Conditions**: a conjunction (`And`) of positive atoms — frame (`o[p->v]`),
//!   membership (`o # c`), subclass (`c1 ## c2`) atoms,
//!   plus **externally-defined builtin** calls ([`Builtin`]). `Equal` (`a = b`)
//!   atoms are permitted in rule BODIES only (see Equal-atom semantics below).
//!   No `Or`, no existentials in the body beyond the implicit universal closure,
//!   no negation.
//! * **Rules**: `Forall ?v… ( head :- body )` Horn implications with a single
//!   (conjunctive) head. The head's atoms and variables are **range-restricted**
//!   by the body (validated — see below).
//! * **Facts**: ground atoms (a rule with an empty/`True` body).
//!
//! ## Equal-atom semantics — body-only, ground-identity, fail-closed on value-equality
//!
//! [SONNET-4.6] sq-pbz04.5.4 — RIF-Core is an important special case for `Equal`
//! (`a = b`) atoms: unlike RIF-BLD (which permits equality in heads and applies
//! **congruence closure** / equality-propagating semantics), **RIF-Core forbids
//! `Equal` in rule conclusions** and scopes its body-semantics to
//! **ground-identity** (two terms are equal iff they are identical after variable
//! substitution, which is structural/syntactic equality over RDF terms).
//!
//! Concretely, the validator ([`Document::validate`]) applies three rules:
//!
//! 1. **Equal in a head is rejected** — `validate()` returns
//!    [`RifError::EqualInConclusion`] for any rule whose head contains an `Equal`
//!    atom. This is a Core syntactic restriction relative to RIF-BLD, enforced
//!    fail-closed.
//! 2. **Ground-identity in the body** — a body `Equal { left, right }` atom where
//!    both `left` and `right` are syntactically identical is trivially true and is
//!    *eliminated* at lowering time (no N3 atom is emitted). `Equal { left:
//!    Term::Iri("x"), right: Term::Iri("x") }` is always satisfied; it adds no
//!    constraint. Variable equality works at runtime because both sides are bound to
//!    the same RDF term through the chainer's matching (lowered to an
//!    `owl:sameAs`-pattern triple in the N3 body, which holds when both variables
//!    are bound to the same RDF node in the closure).
//! 3. **Distinct ground constants are rejected fail-closed** — `validate()` returns
//!    [`RifError::DistinctGroundEqual`] for any body `Equal` atom whose `left` and
//!    `right` are non-variable, non-identical terms (e.g.
//!    `"1"^^xsd:integer = "1.0"^^xsd:decimal`). Such atoms require **value-space
//!    equality** (comparing integer `1` with decimal `1.0`), which depends on the
//!    shared value-space comparator not yet merged (sq-v5evr, issue #1646). Until
//!    that seam lands, this front-end refuses rather than answering incorrectly.
//!
//! **EXPLICITLY EXCLUDED** (RIF-Core is monotone, so these would break
//! monotonicity and are *not* in the dialect): negation-as-failure / `Naf`, the
//! RIF-PRD production constructs (`Assert`/`Retract`/`Modify` actions, conflict
//! resolution, the `Negation` strong negation of RIF-BLD), function symbols in
//! the *logic* sense (uninterpreted functions / `Frame`-nesting beyond what Core
//! allows), and aggregation. This front-end **rejects** any attempt to use a
//! negative or nonmonotonic construct rather than silently mis-evaluating it.
//! Because every rule is a positive Horn implication and the chainer only ever
//! *adds* facts, the closure is **monotone**: adding facts to the input can only
//! add conclusions, never retract one. See [`Document::closure`].
//!
//! ## Builtins (RIF-Core's `func:`/`pred:` externals) — with SAFETY enforced
//!
//! RIF-Core imports a standard set of datatype / numeric / string / list
//! builtins from XPath/XQuery 1.0 Functions & Operators (the
//! `rif-builtin-function`/`rif-builtin-predicate` IRIs). [`Builtin`] models the
//! subset this front-end implements; each lowers to the equivalent N3
//! `math:`/`string:`/`list:` builtin the chainer already evaluates. **Builtin
//! safety / range-restriction is enforced**: a builtin may compute a value into a
//! variable only when its *inputs* are range-restricted (bound by a positive body
//! atom), and a head variable must likewise be bound by a positive body atom.
//! [`Document::validate`] **rejects** an unsafe rule (returning [`RifError`])
//! instead of letting the chainer loop or over-derive on unbound builtin inputs.
//!
//! Unimplemented RIF features are reported as documented out-of-scope (the
//! [`UNIMPLEMENTED`] list), never faked.

use crate::n3::Term as N3Term;
use sparq_core::dict::{Dict, Id};
use std::collections::BTreeSet;
use std::fmt;

/// RIF datatype / standard-vocabulary IRIs used when lowering.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
/// N3 builtin namespaces the RIF builtins lower onto (the chainer recognises these).
const MATH: &str = "http://www.w3.org/2000/10/swap/math#";
const STRING: &str = "http://www.w3.org/2000/10/swap/string#";
const LIST: &str = "http://www.w3.org/2000/10/swap/list#";

/// RIF features deliberately NOT implemented by this Core front-end — reported as
/// documented out-of-scope (never faked into a passing test). RIF-Core is the
/// monotone Horn subset, so the nonmonotonic / production / non-Core items here
/// are not even *in* the dialect; the rest are larger-dialect surface tracked for
/// honesty. Cross-referenced by the expressivity suite so the gaps are legible.
// [SONNET-4.6] sq-pbz04.5.2 — deferral ledger (§3.2) added below.
pub const UNIMPLEMENTED: &[&str] = &[
    // NONMONOTONIC / NON-CORE by dialect (would break monotonicity — excluded by design):
    "negation-as-failure (Naf) — RIF-Core is monotone; NAF is not in the dialect",
    "RIF-BLD strong negation (Negation)",
    "RIF-PRD actions (Assert/Retract/Modify) + conflict resolution",
    "aggregation (min/max/count/sum over a group)",
    // LARGER-DIALECT logic surface, tracked-not-implemented:
    "uninterpreted function symbols / nested function terms (RIF-BLD)",
    "disjunction (Or) in rule bodies",
    "the RIF/XML presentation-syntax importer (this front-end takes the in-engine model)",
    // DEFERRED BUILTINS — soundly not mapped, each with its precise reason (§3.2):
    // [SONNET-4.6] sq-pbz04.5.2
    "func:numeric-integer-divide DEFERRED: F&O op:numeric-integer-divide truncates \
     toward zero; EYE/N3 math:integerQuotient is floor division. They diverge on any \
     negative operand (−7 idiv 2 = −3 vs floor(−3.5) = −4). A truncating N3-side \
     builtin or RIF-side compilation is needed — deferred.",
    "func:numeric-mod DEFERRED: F&O op:numeric-mod result follows dividend sign; \
     math:remainder is integer-only with divisor-sign semantics. Mixed-sign operands \
     diverge — deferred.",
    "pred:matches DEFERRED: XPath/XSD regex dialect ≠ Rust regex crate dialect \
     (e.g. XSD character-class subtraction is not supported). A mapping cannot fail \
     closed on dialect-divergent patterns without a real XSD-regex front-end — deferred.",
    // [SONNET-4.6] sq-pbz04.5.4 — Equal-atom audit clarification:
    // Equal in a rule CONCLUSION is REJECTED (RIF-Core syntactic restriction) via
    // RifError::EqualInConclusion. Body Equal with syntactically-identical terms is
    // eliminated (trivially true). Body Equal with distinct ground constants is
    // REJECTED fail-closed via RifError::DistinctGroundEqual (pending sq-v5evr).
    "pred:boolean-equal / pred:literal-not-identical / equality of distinct value-equal \
     constants DEFERRED (fail-closed): need value-space equality beyond the numeric tower \
     (sq-v5evr comparator, issue #1646, a named consumer) — rejects with \
     DistinctGroundEqual pending that seam; body ground-identity (t=t) works.",
    "guard predicates (pred:is-literal-integer etc.) DEFERRED: would require inventing \
     non-EYE N3 builtins, polluting the chainer's EYE-differential story — deferred.",
    "func:substring / func:substring-before / func:substring-after / func:string-join / \
     func:compare DEFERRED: no semantically-matching N3 target exists today \
     (string:scrape is regex capture, not substring) — deferred.",
    "list utility builtins (func:get/sublist/reverse/index-of/insert-before/remove/\
     union/distinct-values/except / pred:is-list) DEFERRED: no 1:1 N3 target; \
     multi-triple lowerings change the producer/consumer shape and its safety analysis \
     — deferred.",
    "date/time/duration builtins DEFERRED: no shared temporal tower exists (the \
     chainer's time: support is partial/EYE-shaped; sparq-substrate has no temporal \
     module) — deferred wholesale; a temporal seam is its own future design record.",
];

/// A RIF-Core term: a constant (IRI or typed literal), a universally-quantified
/// variable, or a `List` (RIF's `List(…)` term, used by the list builtins).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Term {
    /// An IRI constant (RIF `Const` of type `rif:iri`).
    Iri(String),
    /// A typed literal `"lex"^^<dt>` (RIF `Const` of an XSD type). The datatype
    /// IRI is stored explicitly; a plain string uses `xsd:string`.
    Lit {
        /// The lexical form.
        lex: String,
        /// The datatype IRI.
        datatype: String,
    },
    /// A universally-quantified variable (`?v`).
    Var(String),
    /// A RIF `List(…)` term (first-class, like N3 collections).
    List(Vec<Term>),
}

impl Term {
    /// An `xsd:integer` literal constant — convenience for facts/tests.
    pub fn int(n: i64) -> Term {
        Term::Lit { lex: n.to_string(), datatype: format!("{}integer", XSD) }
    }

    /// An `xsd:string` literal constant.
    pub fn string(s: impl Into<String>) -> Term {
        Term::Lit { lex: s.into(), datatype: format!("{}string", XSD) }
    }

    /// Is this term a variable (`?v`)?
    pub fn is_var(&self) -> bool {
        matches!(self, Term::Var(_))
    }

    /// Collect every variable name occurring in this term (recursing into lists).
    fn vars_into(&self, out: &mut BTreeSet<String>) {
        match self {
            Term::Var(v) => {
                out.insert(v.clone());
            }
            Term::List(items) => {
                for t in items {
                    t.vars_into(out);
                }
            }
            _ => {}
        }
    }

    /// Lower to the N3 term model.
    fn to_n3(&self) -> N3Term {
        match self {
            Term::Iri(i) => N3Term::Iri(i.clone()),
            Term::Lit { lex, datatype } => N3Term::Lit(lex.clone(), datatype.clone(), None),
            Term::Var(v) => N3Term::Var(v.clone()),
            Term::List(items) => N3Term::List(items.iter().map(Term::to_n3).collect()),
        }
    }
}

/// A RIF-Core **builtin** call — an externally-defined `func:`/`pred:` term.
///
/// RIF-Core imports these from XPath/XQuery 1.0 F&O. This front-end models the
/// numeric / string / list / datatype subset the expressivity suite exercises;
/// each lowers to the equivalent N3 builtin the chainer evaluates. A builtin is
/// either a **predicate** (`pred:…`, a body filter — `is_filter()`) or a
/// **function** (`func:…`, computes its last argument from the earlier ones).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Builtin {
    // --- numeric predicates (pred:numeric-*) — body filters ---
    /// `pred:numeric-equal` (a = b).
    NumericEqual,
    /// `pred:numeric-less-than` (a < b).
    NumericLessThan,
    /// `pred:numeric-greater-than` (a > b).
    NumericGreaterThan,
    /// `pred:numeric-greater-than-or-equal` (a >= b).
    NumericNotLessThan,
    /// `pred:numeric-less-than-or-equal` (a <= b).
    NumericNotGreaterThan,
    // --- numeric functions (func:numeric-*) — compute the result arg ---
    /// `func:numeric-add` (c = a + b).
    NumericAdd,
    /// `func:numeric-subtract` (c = a - b).
    NumericSubtract,
    /// `func:numeric-multiply` (c = a * b).
    NumericMultiply,
    /// `func:numeric-divide` (c = a / b).
    NumericDivide,
    // --- string predicates / functions ---
    /// `func:contains` (predicate use: a contains b).
    StringContains,
    /// `func:starts-with` (predicate use).
    StringStartsWith,
    /// `func:ends-with` (predicate use).
    StringEndsWith,
    /// `func:concat` (function: c = concat(a, b)).
    StringConcat,
    /// `func:string-length` (function: n = length(a)).
    StringLength,
    // --- list functions / predicates ---
    /// `pred:list-contains` (predicate: list contains member).
    ListContains,
    /// list length (function: n = length(list)).
    ListLength,
    // [SONNET-4.6] sq-pbz04.5.2 — 5 soundly-mapped builtins (§3.1 equivalences):
    /// `pred:numeric-not-equal` (a ≠ b) — N3 target: `math:notEqualTo`.
    /// §3.1 equivalence: both are numeric value-space inequality over the same
    /// promotion tower; the chainer's `MathNe` exists and is exercised by the N3
    /// suites.
    NumericNotEqual,
    /// `func:upper-case` (maps string to upper case) — N3 target: `string:upperCase`.
    /// §3.1 equivalence: XPath `fn:upper-case` uses default Unicode case mapping ≙
    /// Rust `str::to_uppercase` (same default, no locale/tailored mappings on either
    /// side).
    StringUpperCase,
    /// `func:lower-case` (maps string to lower case) — N3 target: `string:lowerCase`.
    /// §3.1 equivalence: symmetric to upper-case.
    StringLowerCase,
    /// `func:encode-for-uri` (RFC 3986 percent-encoding) — N3 target: `string:encodeForUri`.
    /// §3.1 equivalence: the chainer's builtin is documented as exactly XPath
    /// `fn:encode-for-uri` (RFC 3986 unreserved-set, uppercase hex) — definitionally
    /// the same function.
    StringEncodeForUri,
    /// `func:concatenate` (list concatenation) — N3 target: `list:append`.
    /// §3.1 equivalence: both concatenate a sequence of lists into one list,
    /// order-preserving; the chainer's `Append` takes `( list… )` which matches
    /// DTB's variadic signature. Variadic: `arity()` returns the MINIMUM (2 = at
    /// least one input list + one output); validate uses `>=` for this variant.
    ListConcatenate,
}

impl Builtin {
    /// `true` for the **predicate** builtins (used as a body filter, all args are
    /// inputs); `false` for the **function** builtins (the LAST argument is the
    /// computed output, the earlier ones are inputs).
    pub fn is_filter(self) -> bool {
        matches!(
            self,
            Builtin::NumericEqual
                | Builtin::NumericLessThan
                | Builtin::NumericGreaterThan
                | Builtin::NumericNotLessThan
                | Builtin::NumericNotGreaterThan
                | Builtin::NumericNotEqual // [SONNET-4.6] sq-pbz04.5.2
                | Builtin::StringContains
                | Builtin::StringStartsWith
                | Builtin::StringEndsWith
                | Builtin::ListContains
        )
    }

    /// Exact argument arity (including the result arg for function builtins).
    /// For variadic builtins (`is_variadic()` is true) this is the MINIMUM arity;
    /// validation uses `>=` instead of `==`.
    fn arity(self) -> usize {
        match self {
            // binary predicates
            Builtin::NumericEqual
            | Builtin::NumericLessThan
            | Builtin::NumericGreaterThan
            | Builtin::NumericNotLessThan
            | Builtin::NumericNotGreaterThan
            | Builtin::NumericNotEqual // [SONNET-4.6] sq-pbz04.5.2
            | Builtin::StringContains
            | Builtin::StringStartsWith
            | Builtin::StringEndsWith
            | Builtin::ListContains => 2,
            // binary-input functions producing one result
            Builtin::NumericAdd
            | Builtin::NumericSubtract
            | Builtin::NumericMultiply
            | Builtin::NumericDivide
            | Builtin::StringConcat => 3,
            // unary-input functions producing one result
            Builtin::StringLength
            | Builtin::ListLength
            | Builtin::StringUpperCase  // [SONNET-4.6] sq-pbz04.5.2
            | Builtin::StringLowerCase
            | Builtin::StringEncodeForUri => 2,
            // variadic list concat: MINIMUM arity (1 input + 1 output = 2)
            // [SONNET-4.6] sq-pbz04.5.2
            Builtin::ListConcatenate => 2,
        }
    }

    /// `true` only for the variadic-arity builtins (`func:concatenate`), where
    /// `arity()` returns the **minimum** accepted argument count and `validate`
    /// uses a `>=` check. All other builtins have a fixed arity (`==` check).
    // [SONNET-4.6] sq-pbz04.5.2
    pub fn is_variadic(self) -> bool {
        matches!(self, Builtin::ListConcatenate)
    }
}

/// A RIF-Core **condition atom** — what appears as a fact, a body conjunct, or a
/// head conclusion. (RIF-Core's positive atom forms, plus the builtin call.)
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Atom {
    /// A **frame**: `obj[pred -> val]` — RIF's frame atom, the workhorse positive
    /// atom (lowers to the triple `obj pred val`).
    Frame {
        /// The frame object.
        obj: Term,
        /// The slot predicate.
        pred: Term,
        /// The slot value.
        val: Term,
    },
    /// **Membership**: `obj # class` (lowers to `obj rdf:type class`).
    Member {
        /// The member.
        obj: Term,
        /// The class.
        class: Term,
    },
    /// **Subclass**: `sub ## sup` (lowers to `sub rdfs:subClassOf sup`).
    Subclass {
        /// The subclass.
        sub: Term,
        /// The superclass.
        sup: Term,
    },
    /// **Equality**: `a = b` (lowers to `a owl:sameAs b`).
    Equal {
        /// The left term.
        left: Term,
        /// The right term.
        right: Term,
    },
    /// A **builtin** call `op(args…)`. In a body: a filter (predicate) or a
    /// value-computing function (the last arg is the output). NOT allowed in a
    /// head.
    Builtin {
        /// The builtin operator.
        op: Builtin,
        /// Its arguments.
        args: Vec<Term>,
    },
}

impl Atom {
    fn vars(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        match self {
            Atom::Frame { obj, pred, val } => {
                obj.vars_into(&mut out);
                pred.vars_into(&mut out);
                val.vars_into(&mut out);
            }
            Atom::Member { obj, class } => {
                obj.vars_into(&mut out);
                class.vars_into(&mut out);
            }
            Atom::Subclass { sub, sup } => {
                sub.vars_into(&mut out);
                sup.vars_into(&mut out);
            }
            Atom::Equal { left, right } => {
                left.vars_into(&mut out);
                right.vars_into(&mut out);
            }
            Atom::Builtin { args, .. } => {
                for a in args {
                    a.vars_into(&mut out);
                }
            }
        }
        out
    }

    /// Is this a **positive (non-builtin) atom** — one that binds its variables by
    /// matching facts? Frame/Member/Subclass/Equal are positive; a builtin is not
    /// (it filters or computes, it does not generate bindings for arbitrary
    /// inputs). Used by range-restriction validation.
    fn is_positive(&self) -> bool {
        !matches!(self, Atom::Builtin { .. })
    }

    /// Lower a positive atom to one N3 triple. (Builtins lower separately, since a
    /// function builtin emits the producer/consumer ordering the chainer needs.)
    fn positive_to_n3(&self) -> Option<[N3Term; 3]> {
        match self {
            Atom::Frame { obj, pred, val } => Some([obj.to_n3(), pred.to_n3(), val.to_n3()]),
            Atom::Member { obj, class } => {
                Some([obj.to_n3(), N3Term::Iri(RDF_TYPE.to_string()), class.to_n3()])
            }
            Atom::Subclass { sub, sup } => {
                Some([sub.to_n3(), N3Term::Iri(RDFS_SUBCLASS_OF.to_string()), sup.to_n3()])
            }
            Atom::Equal { left, right } => {
                Some([left.to_n3(), N3Term::Iri(OWL_SAME_AS.to_string()), right.to_n3()])
            }
            Atom::Builtin { .. } => None,
        }
    }
}

/// A RIF-Core **rule**: `Forall ?v… ( head :- body )`. An empty `body` makes the
/// head a (universally-closed) **fact** when ground.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    /// The conclusion atoms (a conjunctive head). Must be positive (no builtins).
    pub head: Vec<Atom>,
    /// The premise atoms (a conjunction). May contain builtins.
    pub body: Vec<Atom>,
}

impl Rule {
    /// A ground **fact** (empty body, single head atom).
    pub fn fact(head: Atom) -> Rule {
        Rule { head: vec![head], body: Vec::new() }
    }

    /// An implication `head :- body`.
    pub fn implies(head: Vec<Atom>, body: Vec<Atom>) -> Rule {
        Rule { head, body }
    }
}

/// A RIF-Core document: a set of rules (facts are rules with an empty body).
#[derive(Clone, Default, Debug)]
pub struct Document {
    /// The rules (and facts) of the document.
    pub rules: Vec<Rule>,
}

/// Why a RIF-Core document was rejected. RIF-Core requires DATALOG **safety**
/// (range-restriction); violating it is rejected up front rather than letting the
/// chainer loop or over-derive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RifError {
    /// A head variable is not bound by any positive body atom (unsafe rule — the
    /// head is not range-restricted).
    UnboundHeadVar {
        /// The offending variable name.
        var: String,
    },
    /// A builtin INPUT variable is not bound by a positive body atom (or by an
    /// earlier function builtin's output) — its input is not range-restricted, so
    /// it could not be evaluated.
    UnboundBuiltinInput {
        /// The offending variable name.
        var: String,
    },
    /// A builtin was used with the wrong arity.
    BadBuiltinArity {
        /// The builtin.
        op: Builtin,
        /// The supplied argument count.
        got: usize,
        /// The expected argument count.
        want: usize,
    },
    /// A builtin appeared in a rule HEAD (builtins are body-only).
    BuiltinInHead {
        /// The offending builtin.
        op: Builtin,
    },
    /// An `Equal` atom appeared in a rule HEAD. RIF-Core (unlike RIF-BLD) does
    /// **not** permit equality in conclusions — it is a Core syntactic restriction.
    /// `validate()` rejects any rule whose head contains an `Equal` atom with this
    /// error. [SONNET-4.6] sq-pbz04.5.4
    EqualInConclusion,
    /// A body `Equal` atom has two **distinct non-variable** terms (e.g.
    /// `"1"^^xsd:integer = "1.0"^^xsd:decimal`). This requires **value-space
    /// equality** across XSD types, which depends on the shared value-space
    /// comparator (sq-v5evr, issue #1646, not yet merged). Until that seam lands
    /// this front-end rejects rather than answering incorrectly.
    /// [SONNET-4.6] sq-pbz04.5.4
    DistinctGroundEqual {
        /// The left-hand term (rendered as a string for display).
        left: String,
        /// The right-hand term (rendered as a string for display).
        right: String,
    },
    /// A nonmonotonic / non-Core construct was supplied (defensive — these are not
    /// representable in the [`Atom`] model, so this is reserved for importers and
    /// internal lowering invariants).
    Nonmonotonic {
        /// What was rejected.
        what: String,
    },
}

impl fmt::Display for RifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RifError::UnboundHeadVar { var } => write!(
                f,
                "unsafe RIF-Core rule: head variable ?{} is not range-restricted \
                 (no positive body atom binds it)",
                var
            ),
            RifError::UnboundBuiltinInput { var } => write!(
                f,
                "unsafe RIF-Core rule: builtin input ?{} is not range-restricted \
                 (no positive body atom or earlier function builtin binds it)",
                var
            ),
            RifError::BadBuiltinArity { op, got, want } => write!(
                f,
                "RIF-Core builtin {:?} expects {} argument(s), got {}",
                op, want, got
            ),
            RifError::BuiltinInHead { op } => write!(
                f,
                "RIF-Core builtin {:?} may not appear in a rule head (builtins are body-only)",
                op
            ),
            // [SONNET-4.6] sq-pbz04.5.4
            RifError::EqualInConclusion => write!(
                f,
                "RIF-Core does not permit Equal (=) in a rule conclusion; use RIF-BLD \
                 for equality-in-head semantics (Core syntactic restriction)"
            ),
            // [SONNET-4.6] sq-pbz04.5.4
            RifError::DistinctGroundEqual { left, right } => write!(
                f,
                "RIF-Core body Equal with distinct ground terms ({} = {}) requires \
                 value-space equality (e.g. xsd:integer 1 vs xsd:decimal 1.0); \
                 deferred pending sq-v5evr value-space comparator (#1646)",
                left, right
            ),
            RifError::Nonmonotonic { what } => write!(
                f,
                "RIF-Core is monotone: {} is not in the dialect (negation/production \
                 constructs are excluded)",
                what
            ),
        }
    }
}

impl std::error::Error for RifError {}

impl Document {
    /// An empty document.
    pub fn new() -> Document {
        Document::default()
    }

    /// Add a rule (or fact) to the document.
    pub fn push(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// **Validate** the document for RIF-Core **safety / range-restriction** and
    /// **syntactic restrictions** — the load-bearing invariant. Returns `Ok(())` if
    /// every rule is valid, else the first [`RifError`].
    ///
    /// A rule is safe when, processing the body **left to right**:
    /// * every builtin's **input** arguments are range-restricted — each input
    ///   variable is bound by an earlier positive body atom or by an earlier
    ///   function builtin's output; and
    /// * every variable in the **head** is bound by some positive body atom or a
    ///   function builtin output.
    ///
    /// In addition:
    /// * Builtins must have correct arity and may not appear in a head.
    /// * `Equal` atoms may **not** appear in a head (`RifError::EqualInConclusion`
    ///   — RIF-Core syntactic restriction; see module-level doc comment).
    /// * A body `Equal` with two **distinct non-variable** terms is rejected
    ///   fail-closed (`RifError::DistinctGroundEqual`) pending the value-space
    ///   comparator (sq-v5evr/#1646).
    pub fn validate(&self) -> Result<(), RifError> {
        for rule in &self.rules {
            // Head atoms must be positive and must NOT be Equal.
            // [SONNET-4.6] sq-pbz04.5.4: RIF-Core forbids Equal in conclusions.
            for h in &rule.head {
                match h {
                    Atom::Builtin { op, .. } => {
                        return Err(RifError::BuiltinInHead { op: *op });
                    }
                    Atom::Equal { .. } => {
                        return Err(RifError::EqualInConclusion);
                    }
                    _ => {}
                }
            }
            // Walk the body left to right, growing the set of bound variables.
            // A positive atom binds all its variables (matched against facts). A
            // FUNCTION builtin requires its input args bound and then binds its
            // output arg; a PREDICATE builtin requires ALL its args bound.
            let mut bound: BTreeSet<String> = BTreeSet::new();
            for atom in &rule.body {
                match atom {
                    Atom::Builtin { op, args } => {
                        // [SONNET-4.6] sq-pbz04.5.2 — variadic builtins use >= check.
                        let arity_ok = if op.is_variadic() {
                            args.len() >= op.arity()
                        } else {
                            args.len() == op.arity()
                        };
                        if !arity_ok {
                            return Err(RifError::BadBuiltinArity {
                                op: *op,
                                got: args.len(),
                                want: op.arity(),
                            });
                        }
                        if op.is_filter() {
                            // Predicate: every arg is an input — all must be bound.
                            require_bound(args, &bound)?;
                        } else {
                            // Function: all but the last arg are inputs; they must
                            // be bound. The last arg is the OUTPUT — it may be an
                            // unbound variable (which this builtin binds) or a
                            // constant/bound term (then it is a check).
                            let (inputs, output) = args.split_at(args.len() - 1);
                            require_bound(inputs, &bound)?;
                            output[0].vars_into(&mut bound);
                        }
                    }
                    // [SONNET-4.6] sq-pbz04.5.4 — body Equal audit:
                    // * Distinct non-variable ground terms → fail-closed pending sq-v5evr.
                    // * All other Equal atoms (var on at least one side, or identical
                    //   ground terms) fall through to the positive arm to bind their vars.
                    Atom::Equal { left, right } if !left.is_var() && !right.is_var() => {
                        // Both sides are non-variable (IRI or Lit or List).
                        // If they are syntactically identical, the atom is trivially true
                        // (no variable contribution, no conflict — let it through to the
                        // positive arm to harmlessly bind zero variables).
                        // If they differ, reject fail-closed: value-space equality across
                        // distinct typed literals requires sq-v5evr and is not yet landed.
                        if left != right {
                            return Err(RifError::DistinctGroundEqual {
                                left: format!("{:?}", left),
                                right: format!("{:?}", right),
                            });
                        }
                        // Syntactically identical: treated as trivially true in the body.
                        // No variable contribution; no error. (At lowering time this atom
                        // is eliminated — see `lower_body_atom`.)
                    }
                    positive => {
                        debug_assert!(positive.is_positive());
                        bound.append(&mut positive.vars());
                    }
                }
            }
            // Every head variable must be range-restricted by the body.
            let mut head_vars = BTreeSet::new();
            for h in &rule.head {
                head_vars.extend(h.vars());
            }
            for v in &head_vars {
                if !bound.contains(v) {
                    return Err(RifError::UnboundHeadVar { var: v.clone() });
                }
            }
        }
        Ok(())
    }

    /// Lower the (validated) document to an N3 source string. Facts become
    /// top-level triples; implications become `{ body } => { head }` N3 rules. The
    /// builtins lower to their `math:`/`string:`/`list:` N3 equivalents.
    ///
    /// Returns [`RifError`] if validation fails (so a caller cannot run an unsafe
    /// document by going straight to lowering).
    pub fn to_n3_source(&self) -> Result<String, RifError> {
        self.validate()?;
        let mut out = String::new();
        for rule in &self.rules {
            if rule.body.is_empty() {
                // Fact(s): emit each head atom as a ground triple.
                for h in &rule.head {
                    let t = h.positive_to_n3().expect("validated head atoms are positive");
                    out.push_str(&n3_triple(&t));
                    out.push_str(" .\n");
                }
            } else {
                out.push_str("{ ");
                for atom in &rule.body {
                    // [SONNET-4.6] sq-pbz04.5.4 — lower_body_atom returns None for
                    // trivially-true body Equal atoms (ground-identity elimination).
                    if let Some(n3_atom) = lower_body_atom(atom) {
                        out.push_str(&n3_atom);
                        out.push_str(" . ");
                    }
                }
                out.push_str("} => { ");
                for h in &rule.head {
                    let t = h.positive_to_n3().expect("validated head atoms are positive");
                    out.push_str(&n3_triple(&t));
                    out.push_str(" . ");
                }
                out.push_str("} .\n");
            }
        }
        Ok(out)
    }

    /// Compute the **monotone** forward-chaining closure of the document and
    /// return the entailed ground triples interned into `dict`. This validates,
    /// lowers to N3, and drives the existing `reason_n3` forward chainer.
    ///
    /// MONOTONE: every rule is a positive Horn implication and the chainer only
    /// ever *adds* facts, so adding input facts can only add conclusions — never
    /// retract one.
    pub fn closure(&self, dict: &mut Dict) -> Result<Vec<[Id; 3]>, RifError> {
        let src = self.to_n3_source()?;
        // The N3 engine is the proven monotone Horn chainer; reuse it wholesale.
        // A lowering bug surfaces as a parse error from a valid document, which we
        // surface as a Nonmonotonic-flavoured internal error (should never happen
        // for a validated document — covered by the unit tests).
        crate::reason_n3(dict, &src).map_err(|e| RifError::Nonmonotonic {
            what: format!("internal lowering error: {}", e),
        })
    }
}

/// Require every variable occurring in `terms` to be present in `bound`.
fn require_bound(terms: &[Term], bound: &BTreeSet<String>) -> Result<(), RifError> {
    for t in terms {
        let mut vs = BTreeSet::new();
        t.vars_into(&mut vs);
        for v in vs {
            if !bound.contains(&v) {
                return Err(RifError::UnboundBuiltinInput { var: v });
            }
        }
    }
    Ok(())
}

/// Lower a body atom (positive OR builtin) to its N3 surface form.
///
/// Returns `None` for atoms that are trivially true and require no N3 emission
/// (specifically: a body `Equal` atom whose two sides are syntactically identical).
/// [SONNET-4.6] sq-pbz04.5.4 — ground-identity elimination.
fn lower_body_atom(atom: &Atom) -> Option<String> {
    match atom {
        Atom::Builtin { op, args } => Some(lower_builtin(*op, args)),
        // [SONNET-4.6] sq-pbz04.5.4 — ground-identity body Equal: if both sides are
        // syntactically identical (same IRI, same literal, same variable name), the
        // atom is trivially true (no N3 emission needed). For all other Equal atoms
        // (variable on at least one side, non-identical) we fall through to the
        // standard positive_to_n3 path, which lowers to an `owl:sameAs` triple match
        // in the N3 body (the chainer matches this when both terms ground to the
        // same RDF node in the closure).
        Atom::Equal { left, right } if left == right => None,
        other => {
            let t = other.positive_to_n3().expect("non-builtin body atom is positive");
            Some(n3_triple(&t))
        }
    }
}

/// Lower a builtin call to its N3 builtin triple. Predicate builtins become
/// `arg0 math:lessThan arg1` etc.; function builtins become
/// `( in0 in1 ) math:sum out`.
fn lower_builtin(op: Builtin, args: &[Term]) -> String {
    let n3 = |t: &Term| n3_term(&t.to_n3());
    match op {
        // numeric predicates
        Builtin::NumericEqual => bin(&n3(&args[0]), MATH, "equalTo", &n3(&args[1])),
        Builtin::NumericLessThan => bin(&n3(&args[0]), MATH, "lessThan", &n3(&args[1])),
        Builtin::NumericGreaterThan => bin(&n3(&args[0]), MATH, "greaterThan", &n3(&args[1])),
        Builtin::NumericNotLessThan => bin(&n3(&args[0]), MATH, "notLessThan", &n3(&args[1])),
        Builtin::NumericNotGreaterThan => bin(&n3(&args[0]), MATH, "notGreaterThan", &n3(&args[1])),
        // string predicates
        Builtin::StringContains => bin(&n3(&args[0]), STRING, "contains", &n3(&args[1])),
        Builtin::StringStartsWith => bin(&n3(&args[0]), STRING, "startsWith", &n3(&args[1])),
        Builtin::StringEndsWith => bin(&n3(&args[0]), STRING, "endsWith", &n3(&args[1])),
        // list predicate: `?list list:member ?x` — subject is the list (arg0),
        // object the candidate member (arg1).
        Builtin::ListContains => bin(&n3(&args[0]), LIST, "member", &n3(&args[1])),
        // numeric functions: ( a b ) math:sum out
        Builtin::NumericAdd => func2(&n3(&args[0]), &n3(&args[1]), MATH, "sum", &n3(&args[2])),
        Builtin::NumericSubtract => {
            func2(&n3(&args[0]), &n3(&args[1]), MATH, "difference", &n3(&args[2]))
        }
        Builtin::NumericMultiply => {
            func2(&n3(&args[0]), &n3(&args[1]), MATH, "product", &n3(&args[2]))
        }
        Builtin::NumericDivide => {
            func2(&n3(&args[0]), &n3(&args[1]), MATH, "quotient", &n3(&args[2]))
        }
        // string functions
        Builtin::StringConcat => {
            func2(&n3(&args[0]), &n3(&args[1]), STRING, "concatenation", &n3(&args[2]))
        }
        Builtin::StringLength => bin(&n3(&args[0]), STRING, "length", &n3(&args[1])),
        // list function: list list:length out
        Builtin::ListLength => bin(&n3(&args[0]), LIST, "length", &n3(&args[1])),
        // [SONNET-4.6] sq-pbz04.5.2 — 5 new soundly-mapped builtins:
        // pred:numeric-not-equal: a math:notEqualTo b
        Builtin::NumericNotEqual => bin(&n3(&args[0]), MATH, "notEqualTo", &n3(&args[1])),
        // func:upper-case: s string:upperCase out
        Builtin::StringUpperCase => bin(&n3(&args[0]), STRING, "upperCase", &n3(&args[1])),
        // func:lower-case: s string:lowerCase out
        Builtin::StringLowerCase => bin(&n3(&args[0]), STRING, "lowerCase", &n3(&args[1])),
        // func:encode-for-uri: s string:encodeForUri out
        Builtin::StringEncodeForUri => bin(&n3(&args[0]), STRING, "encodeForUri", &n3(&args[1])),
        // func:concatenate (variadic lists): ( L1 L2 ... ) list:append out
        Builtin::ListConcatenate => {
            let n_inputs = args.len() - 1;
            let input_terms: Vec<String> = args[..n_inputs].iter().map(n3).collect();
            format!("( {} ) <{}append> {}", input_terms.join(" "), LIST, n3(&args[n_inputs]))
        }
    }
}

/// `s <ns#local> o`.
fn bin(s: &str, ns: &str, local: &str, o: &str) -> String {
    format!("{} <{}{}> {}", s, ns, local, o)
}

/// `( a b ) <ns#local> out` — a binary functional builtin.
fn func2(a: &str, b: &str, ns: &str, local: &str, out: &str) -> String {
    format!("( {} {} ) <{}{}> {}", a, b, ns, local, out)
}

/// Render one N3 triple `s p o`.
fn n3_triple(t: &[N3Term; 3]) -> String {
    format!("{} {} {}", n3_term(&t[0]), n3_term(&t[1]), n3_term(&t[2]))
}

/// Render an N3 term to its surface syntax (IRIs in `<…>`, vars as `?v`, literals
/// `"lex"^^<dt>`, lists `( … )`).
fn n3_term(t: &N3Term) -> String {
    match t {
        N3Term::Iri(i) => format!("<{}>", i),
        N3Term::Var(v) => format!("?{}", v),
        N3Term::Lit(lex, dt, lang) => match lang {
            Some(l) => format!("\"{}\"@{}", escape_lit(lex), l),
            None => format!("\"{}\"^^<{}>", escape_lit(lex), dt),
        },
        N3Term::Blank(b) => format!("_:{}", b),
        N3Term::List(items) => {
            let inner: Vec<String> = items.iter().map(n3_term).collect();
            format!("( {} )", inner.join(" "))
        }
        // A nested formula cannot appear in a lowered RIF-Core document.
        N3Term::Formula(_) => "{ }".to_string(),
    }
}

/// Escape a literal lexical form for N3 double-quoted strings.
fn escape_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{NamedNode, Term as OT};
    use sparq_core::dict::Dict;

    fn iri(s: &str) -> Term {
        Term::Iri(s.to_string())
    }
    fn var(s: &str) -> Term {
        Term::Var(s.to_string())
    }

    /// Id of an IRI in `dict` (0 if absent) — mirrors the n3 module's test helper.
    fn id(dict: &Dict, i: &str) -> Id {
        dict.lookup(&OT::NamedNode(NamedNode::new_unchecked(i.to_string())))
    }

    /// Does the closure contain the ground IRI triple (s, p, o)?
    fn has(dict: &Dict, closure: &[[Id; 3]], s: &str, p: &str, o: &str) -> bool {
        let (a, b, c) = (id(dict, s), id(dict, p), id(dict, o));
        a != 0 && b != 0 && c != 0 && closure.contains(&[a, b, c])
    }

    /// The lexical value of an `xsd:integer` object in an (s, p, ?) frame.
    fn int_object(dict: &Dict, closure: &[[Id; 3]], s: &str, p: &str) -> Option<String> {
        use oxrdf::Term as OxT;
        let (a, b) = (id(dict, s), id(dict, p));
        closure.iter().find(|t| t[0] == a && t[1] == b).map(|t| match dict.term(t[2]) {
            OxT::Literal(l) => l.value().to_string(),
            other => other.to_string(),
        })
    }

    #[test]
    fn term_constructors_and_vars() {
        assert_eq!(
            Term::int(7),
            Term::Lit { lex: "7".into(), datatype: format!("{}integer", XSD) }
        );
        assert_eq!(
            Term::string("hi"),
            Term::Lit { lex: "hi".into(), datatype: format!("{}string", XSD) }
        );
        assert!(var("x").is_var());
        assert!(!iri("u").is_var());
        let mut vs = BTreeSet::new();
        Term::List(vec![var("a"), iri("u"), var("b")]).vars_into(&mut vs);
        assert_eq!(vs, ["a".to_string(), "b".to_string()].into_iter().collect());
    }

    #[test]
    fn builtin_arity_and_filter_flags() {
        assert!(Builtin::NumericLessThan.is_filter());
        assert!(!Builtin::NumericAdd.is_filter());
        assert_eq!(Builtin::NumericAdd.arity(), 3);
        assert_eq!(Builtin::NumericLessThan.arity(), 2);
        assert_eq!(Builtin::StringLength.arity(), 2);
    }

    #[test]
    fn atom_vars_and_positive() {
        let f = Atom::Frame { obj: var("o"), pred: iri("p"), val: var("v") };
        assert!(f.is_positive());
        assert_eq!(f.vars(), ["o".to_string(), "v".to_string()].into_iter().collect());
        let b = Atom::Builtin { op: Builtin::NumericLessThan, args: vec![var("a"), var("b")] };
        assert!(!b.is_positive());
    }

    #[test]
    fn rule_constructors() {
        let fact = Rule::fact(Atom::Member { obj: iri("a"), class: iri("C") });
        assert!(fact.body.is_empty());
        let r = Rule::implies(
            vec![Atom::Member { obj: var("x"), class: iri("C") }],
            vec![Atom::Member { obj: var("x"), class: iri("D") }],
        );
        assert_eq!(r.body.len(), 1);
    }

    #[test]
    fn document_new_and_push() {
        let mut d = Document::new();
        assert!(d.rules.is_empty());
        d.push(Rule::fact(Atom::Member { obj: iri("a"), class: iri("C") }));
        assert_eq!(d.rules.len(), 1);
    }

    /// The load-bearing MONOTONE Horn closure over frame + membership.
    #[test]
    fn closure_propagates_membership() {
        // Forall ?x ( ?x # C :- ?x # D ) ; a # D.
        let mut doc = Document::new();
        doc.push(Rule::fact(Atom::Member { obj: iri("http://ex/a"), class: iri("http://ex/D") }));
        doc.push(Rule::implies(
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/C") }],
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/D") }],
        ));
        let mut dict = Dict::new();
        let closure = doc.closure(&mut dict).expect("safe document");
        assert!(
            has(&dict, &closure, "http://ex/a", RDF_TYPE, "http://ex/C"),
            "a is a C, derived from the rule + a # D"
        );
    }

    /// Monotonicity: adding a fact only ADDS conclusions, never retracts.
    #[test]
    fn closure_is_monotone() {
        let rule = Rule::implies(
            vec![Atom::Frame { obj: var("x"), pred: iri("http://ex/q"), val: var("y") }],
            vec![Atom::Frame { obj: var("x"), pred: iri("http://ex/p"), val: var("y") }],
        );
        let base_fact = Rule::fact(Atom::Frame {
            obj: iri("http://ex/a"),
            pred: iri("http://ex/p"),
            val: iri("http://ex/b"),
        });

        let mut doc1 = Document::new();
        doc1.push(rule.clone());
        doc1.push(base_fact.clone());
        let mut d1 = Dict::new();
        let c1 = doc1.closure(&mut d1).unwrap();
        // a q b is derived.
        assert!(has(&d1, &c1, "http://ex/a", "http://ex/q", "http://ex/b"));

        // Add a SECOND fact; the closure must derive its consequence TOO (monotone:
        // superset, plus a new conclusion).
        let mut doc2 = doc1.clone();
        doc2.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/c"),
            pred: iri("http://ex/p"),
            val: iri("http://ex/d"),
        }));
        let mut d2 = Dict::new();
        let c2 = doc2.closure(&mut d2).unwrap();
        // Everything still derivable, plus the new c q d.
        assert!(has(&d2, &c2, "http://ex/a", "http://ex/q", "http://ex/b"));
        assert!(has(&d2, &c2, "http://ex/c", "http://ex/q", "http://ex/d"));
        assert!(
            c2.len() > c1.len(),
            "adding a fact added a conclusion; never retracted one"
        );
    }

    /// A numeric FUNCTION builtin computes its output and a head can use it.
    #[test]
    fn builtin_function_computes_output() {
        let mut doc = Document::new();
        doc.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/a"),
            pred: iri("http://ex/x"),
            val: Term::int(2),
        }));
        doc.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/a"),
            pred: iri("http://ex/y"),
            val: Term::int(3),
        }));
        doc.push(Rule::implies(
            vec![Atom::Frame {
                obj: iri("http://ex/a"),
                pred: iri("http://ex/sum"),
                val: var("z"),
            }],
            vec![
                Atom::Frame { obj: iri("http://ex/a"), pred: iri("http://ex/x"), val: var("x") },
                Atom::Frame { obj: iri("http://ex/a"), pred: iri("http://ex/y"), val: var("y") },
                Atom::Builtin {
                    op: Builtin::NumericAdd,
                    args: vec![var("x"), var("y"), var("z")],
                },
            ],
        ));
        let mut dict = Dict::new();
        let closure = doc.closure(&mut dict).unwrap();
        assert_eq!(
            int_object(&dict, &closure, "http://ex/a", "http://ex/sum").as_deref(),
            Some("5"),
            "2 + 3 = 5"
        );
    }

    /// A numeric PREDICATE builtin filters the body.
    #[test]
    fn builtin_predicate_filters() {
        let mut doc = Document::new();
        doc.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/a"),
            pred: iri("http://ex/age"),
            val: Term::int(20),
        }));
        doc.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/b"),
            pred: iri("http://ex/age"),
            val: Term::int(10),
        }));
        // adult(?p) :- ?p age ?n , ?n > 18
        doc.push(Rule::implies(
            vec![Atom::Member { obj: var("p"), class: iri("http://ex/Adult") }],
            vec![
                Atom::Frame { obj: var("p"), pred: iri("http://ex/age"), val: var("n") },
                Atom::Builtin {
                    op: Builtin::NumericGreaterThan,
                    args: vec![var("n"), Term::int(18)],
                },
            ],
        ));
        let mut dict = Dict::new();
        let closure = doc.closure(&mut dict).unwrap();
        assert!(
            has(&dict, &closure, "http://ex/a", RDF_TYPE, "http://ex/Adult"),
            "a (age 20) is an Adult"
        );
        assert!(
            !has(&dict, &closure, "http://ex/b", RDF_TYPE, "http://ex/Adult"),
            "b (age 10) is NOT an Adult"
        );
    }

    #[test]
    fn validate_rejects_unbound_head_var() {
        let mut doc = Document::new();
        doc.push(Rule::implies(
            vec![Atom::Frame { obj: var("x"), pred: iri("http://ex/p"), val: var("y") }],
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/C") }],
        ));
        assert_eq!(doc.validate(), Err(RifError::UnboundHeadVar { var: "y".to_string() }));
        // And closure refuses to run it.
        let mut dict = Dict::new();
        assert!(doc.closure(&mut dict).is_err());
    }

    #[test]
    fn validate_rejects_unbound_builtin_input() {
        let mut d = Document::new();
        d.push(Rule::implies(
            vec![Atom::Member { obj: var("p"), class: iri("http://ex/C") }],
            vec![
                Atom::Member { obj: var("p"), class: iri("http://ex/D") },
                Atom::Builtin {
                    op: Builtin::NumericGreaterThan,
                    args: vec![var("n"), Term::int(0)],
                },
            ],
        ));
        assert_eq!(d.validate(), Err(RifError::UnboundBuiltinInput { var: "n".to_string() }));
    }

    #[test]
    fn validate_rejects_builtin_in_head() {
        let mut d = Document::new();
        d.push(Rule::implies(
            vec![Atom::Builtin {
                op: Builtin::NumericLessThan,
                args: vec![var("a"), var("b")],
            }],
            vec![Atom::Frame { obj: var("a"), pred: iri("http://ex/p"), val: var("b") }],
        ));
        assert_eq!(d.validate(), Err(RifError::BuiltinInHead { op: Builtin::NumericLessThan }));
    }

    #[test]
    fn validate_rejects_bad_arity() {
        let mut d = Document::new();
        d.push(Rule::implies(
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/C") }],
            vec![
                Atom::Member { obj: var("x"), class: iri("http://ex/D") },
                Atom::Builtin { op: Builtin::NumericAdd, args: vec![var("x")] },
            ],
        ));
        assert!(matches!(
            d.validate(),
            Err(RifError::BadBuiltinArity { op: Builtin::NumericAdd, got: 1, want: 3 })
        ));
    }

    // ---------------------------------------------------------- Equal-atom tests
    // [SONNET-4.6] sq-pbz04.5.4 — the three Equal-atom semantic requirements:
    // (1) Equal in conclusion is rejected (non-vacuous);
    // (2) ground-identity body Equal works (trivially true, rule fires);
    // (3) value-equality (distinct ground constants) is fail-closed-abstained.

    /// (1) Equal in a CONCLUSION — both as a fact head and as a rule head —
    /// MUST be rejected with `EqualInConclusion`. This is a non-vacuous test:
    /// it asserts the validator catches the case and that `closure` refuses it.
    #[test]
    fn validate_rejects_equal_in_conclusion() {
        // Case A: a "fact" with an Equal head (Rule::fact wraps a single head atom;
        // an empty body makes it a fact, so the head IS the conclusion).
        let mut d = Document::new();
        d.push(Rule::fact(Atom::Equal { left: iri("http://ex/a"), right: iri("http://ex/b") }));
        assert_eq!(
            d.validate(),
            Err(RifError::EqualInConclusion),
            "Equal as a fact head (conclusion) must be rejected"
        );
        let mut dict = Dict::new();
        assert!(d.closure(&mut dict).is_err(), "closure must refuse Equal-in-conclusion");

        // Case B: an implication whose head contains an Equal.
        let mut d2 = Document::new();
        d2.push(Rule::implies(
            vec![Atom::Equal { left: var("x"), right: iri("http://ex/b") }],
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/C") }],
        ));
        assert_eq!(
            d2.validate(),
            Err(RifError::EqualInConclusion),
            "Equal in a rule head must be rejected"
        );
    }

    /// (2) Ground-identity body Equal: `t = t` (syntactically identical terms) is
    /// trivially true and is ELIMINATED at lowering — the rule fires as if the
    /// atom were absent. This exercises the real path (`closure`), not a mock.
    #[test]
    fn body_equal_ground_identity_works() {
        // Rule: ?x # Adult :- ?x age ?n , ?n = ?n
        // The `?n = ?n` atom is ground-identical after substitution → eliminated.
        // The rule should fire just as if the Equal atom were not there.
        let mut doc = Document::new();
        doc.push(Rule::fact(Atom::Frame {
            obj: iri("http://ex/p"),
            pred: iri("http://ex/age"),
            val: Term::int(30),
        }));
        doc.push(Rule::implies(
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/Adult") }],
            vec![
                Atom::Frame { obj: var("x"), pred: iri("http://ex/age"), val: var("n") },
                // ?n = ?n is trivially true; eliminated at lowering.
                Atom::Equal { left: var("n"), right: var("n") },
            ],
        ));
        doc.validate().expect("ground-identity body Equal is valid");
        let mut dict = Dict::new();
        let closure = doc.closure(&mut dict).expect("closure must succeed");
        assert!(
            has(&dict, &closure, "http://ex/p", RDF_TYPE, "http://ex/Adult"),
            "rule fires with trivially-true ground-identity body Equal eliminated"
        );
    }

    /// (3) Distinct ground constants in a body Equal (`"1"^^xsd:integer =
    /// "1.0"^^xsd:decimal`) require value-space equality (sq-v5evr, #1646, not yet
    /// merged). The validator MUST reject fail-closed rather than answer incorrectly.
    #[test]
    fn body_equal_distinct_ground_constants_fail_closed() {
        let xsd_int = format!("{}integer", XSD);
        let xsd_dec = format!("{}decimal", XSD);
        let mut d = Document::new();
        d.push(Rule::implies(
            vec![Atom::Member { obj: var("x"), class: iri("http://ex/C") }],
            vec![
                Atom::Member { obj: var("x"), class: iri("http://ex/D") },
                Atom::Equal {
                    left: Term::Lit { lex: "1".into(), datatype: xsd_int },
                    right: Term::Lit { lex: "1.0".into(), datatype: xsd_dec },
                },
            ],
        ));
        assert!(
            matches!(d.validate(), Err(RifError::DistinctGroundEqual { .. })),
            "distinct ground constants in body Equal must be fail-closed (pending sq-v5evr)"
        );
        let mut dict = Dict::new();
        assert!(d.closure(&mut dict).is_err(), "closure must refuse DistinctGroundEqual");
    }

    #[test]
    fn rif_error_displays() {
        let e = RifError::UnboundHeadVar { var: "y".to_string() };
        assert!(e.to_string().contains("range-restricted"));
        let e = RifError::Nonmonotonic { what: "Naf".to_string() };
        assert!(e.to_string().contains("monotone"));
        let e = RifError::BadBuiltinArity { op: Builtin::NumericAdd, got: 1, want: 3 };
        assert!(e.to_string().contains("argument"));
        let e = RifError::BuiltinInHead { op: Builtin::NumericLessThan };
        assert!(e.to_string().contains("body-only"));
        let e = RifError::UnboundBuiltinInput { var: "n".to_string() };
        assert!(e.to_string().contains("range-restricted"));
        // [SONNET-4.6] sq-pbz04.5.4 — new error variants
        let e = RifError::EqualInConclusion;
        assert!(
            e.to_string().contains("conclusion"),
            "EqualInConclusion display mentions conclusion"
        );
        let e = RifError::DistinctGroundEqual {
            left: "1^^integer".into(),
            right: "1.0^^decimal".into(),
        };
        assert!(
            e.to_string().contains("sq-v5evr"),
            "DistinctGroundEqual display references the deferral bead"
        );
    }

    #[test]
    fn to_n3_source_emits_facts_and_rules() {
        let mut d = Document::new();
        d.push(Rule::fact(Atom::Member { obj: iri("http://ex/a"), class: iri("http://ex/C") }));
        d.push(Rule::implies(
            vec![Atom::Subclass { sub: var("c"), sup: iri("http://ex/Top") }],
            vec![Atom::Subclass { sub: var("c"), sup: iri("http://ex/Mid") }],
        ));
        let src = d.to_n3_source().unwrap();
        assert!(src.contains("=>"), "the rule lowers to an N3 implication");
        assert!(src.contains("rdf-syntax-ns#type"), "membership lowers to rdf:type");
        assert!(src.contains("subClassOf"), "subclass lowers to rdfs:subClassOf");
    }

    #[test]
    fn unimplemented_lists_naf_exclusion() {
        assert!(
            UNIMPLEMENTED.iter().any(|s| s.contains("Naf") && s.contains("monotone")),
            "the documented out-of-scope list names the NAF exclusion"
        );
    }
}
