//! The typed ODRL Information Model (the subset sparq-policy evaluates).
//!
//! This is a faithful-but-scoped Rust rendering of the [ODRL 2.2 Information
//! Model](https://www.w3.org/TR/odrl-model/): a [`Policy`] is a container of
//! deontic [`Rule`]s (Permission / Prohibition), each carrying an [`Action`], a
//! `target` asset, optional `assignee`/`assigner` parties, zero or more
//! [`Constraint`]s, and (for permissions) zero or more [`Duty`] obligations.
//!
//! Scope (the single-node base case — see the crate README): the evaluator
//! supports `Permission` and `Prohibition` rules with the common
//! `leftOperand`/`operator`/`rightOperand` constraints; `Offer`/`Agreement`
//! policy subtypes are parsed identically to `Set` (the subtype affects
//! contracting, not single-node evaluation).
//! [OPUS-4.8]

use std::fmt;

/// The ODRL namespace IRI prefix (`odrl:`).
pub const ODRL_NS: &str = "http://www.w3.org/ns/odrl/2/";

/// An ODRL policy: a container of deontic rules.
///
/// A policy is evaluated as a whole against a single request: a `Permission`
/// whose action/target/constraints all match the request grants access, and any
/// matching `Prohibition` overrides it (fail-closed — see
/// [`crate::evaluate`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Policy {
    /// The policy IRI (the `odrl:Policy`/`Set`/`Offer`/`Agreement` subject), if any.
    pub iri: Option<String>,
    /// `odrl:permission` rules — each, if matched and unprohibited, grants access.
    pub permissions: Vec<Rule>,
    /// `odrl:prohibition` rules — each, if matched, forbids access (overrides).
    pub prohibitions: Vec<Rule>,
}

/// A deontic rule (a Permission or a Prohibition). They share the same shape;
/// the [`Policy`] field they live under fixes their deontic force.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// The rule node IRI/blank-id (for justification in the decision).
    pub id: String,
    /// The regulated [`Action`] (`odrl:action`).
    pub action: Action,
    /// The target asset IRI (`odrl:target`) — the data/graph the rule is about.
    /// `None` means the rule applies to any target (an unrefined rule).
    pub target: Option<String>,
    /// The party the rule is assigned to (`odrl:assignee`) — the requesting
    /// party it grants/forbids for. `None` = applies to any party.
    pub assignee: Option<String>,
    /// The party issuing the rule (`odrl:assigner`). Informational for single-node
    /// evaluation; carried for justification + audit.
    pub assigner: Option<String>,
    /// Constraints (`odrl:constraint`) that must all hold for the rule to be
    /// *satisfied* (logical AND, per the ODRL default `odrl:and`).
    pub constraints: Vec<Constraint>,
    /// Duties (`odrl:duty`) — obligations that must be discharged for a
    /// *permission* to be exercisable. Ignored on prohibitions (a prohibition
    /// has no pre-condition duty in this base case).
    pub duties: Vec<Duty>,
}

/// An ODRL action: the regulated operation (`odrl:use`, `odrl:read`,
/// `odrl:distribute`, `odrl:aggregate`, `odrl:anonymize`, …).
///
/// Stored as the full IRI. The well-known `odrl:use` action is the ODRL
/// "umbrella" action: a permission for `odrl:use` matches a request for any
/// action (per the ODRL action hierarchy, `use` is a super-action). All other
/// actions match by exact IRI equality in this base case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Action(pub String);

impl Action {
    /// The `odrl:use` umbrella action IRI.
    pub fn use_() -> Action {
        Action(format!("{ODRL_NS}use"))
    }

    /// True iff this action permits a request for `requested`: exact-IRI match,
    /// or this action is the `odrl:use` umbrella (which subsumes every action).
    pub fn permits(&self, requested: &Action) -> bool {
        self == requested || self.0 == format!("{ODRL_NS}use")
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An ODRL constraint: a `(leftOperand, operator, rightOperand)` triple
/// (`odrl:Constraint`) evaluated against the request + state-of-the-world.
///
/// Example: `(odrl:dateTime, odrl:lteq, "2026-12-31T00:00:00Z")` — the request
/// must occur at or before that instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    /// The dimension being constrained (`odrl:leftOperand`), e.g. `odrl:dateTime`,
    /// `odrl:purpose`, `odrl:recipient`, `odrl:spatial`, `odrl:count`. Stored as
    /// the full IRI.
    pub left: String,
    /// The comparison [`Operator`] (`odrl:operator`).
    pub operator: Operator,
    /// The value to compare against (`odrl:rightOperand`), as a typed
    /// [`Value`].
    pub right: Value,
}

/// The constraint comparison operators sparq-policy supports (the common ODRL
/// operator set — [ODRL operators](https://www.w3.org/TR/odrl-vocab/#constraintLeftOperandCommon)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `odrl:eq` — equality.
    Eq,
    /// `odrl:neq` — inequality.
    Neq,
    /// `odrl:lt` — strictly less than (numeric or dateTime).
    Lt,
    /// `odrl:lteq` — less than or equal.
    Lteq,
    /// `odrl:gt` — strictly greater than.
    Gt,
    /// `odrl:gteq` — greater than or equal.
    Gteq,
    /// `odrl:isPartOf` — the request value is a member of / part of the
    /// right-operand set, OR — for the taxonomic dimensions `odrl:purpose` and
    /// `odrl:spatial` — **transitively part-of** a named value under the subsumption
    /// closure the request supplies (the DPV-purpose taxonomy / spatial region *tree*
    /// case — see `Request::with_purpose_subsumption`). [OPUS-4.8] sq-z3ve / sq-wukl.
    IsPartOf,
    /// `odrl:isA` — the request value IS the right operand (a type/identity
    /// membership check, treated as equality here).
    IsA,
}

impl Operator {
    /// Parse an `odrl:` operator IRI into an [`Operator`]. Unknown operators
    /// return `None` (and an unparseable constraint fails closed).
    pub fn from_iri(iri: &str) -> Option<Operator> {
        let local = iri.strip_prefix(ODRL_NS)?;
        Some(match local {
            "eq" => Operator::Eq,
            "neq" => Operator::Neq,
            "lt" => Operator::Lt,
            "lteq" => Operator::Lteq,
            "gt" => Operator::Gt,
            "gteq" => Operator::Gteq,
            "isPartOf" => Operator::IsPartOf,
            "isA" => Operator::IsA,
            _ => return None,
        })
    }
}

/// A constraint right-operand or request value, kept as a typed scalar so the
/// evaluator can compare numerically / temporally where appropriate and fall
/// back to string equality otherwise.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// An IRI (an `odrl:rightOperandReference` or a named resource/party/purpose).
    Iri(String),
    /// A plain or typed string literal (purpose codes, recipient ids, regions).
    Str(String),
    /// An `xsd:decimal`/`integer`/`double` numeric literal (for `odrl:count`, …).
    Num(f64),
    /// An `xsd:dateTime`/`xsd:date` literal, normalized to a comparable key.
    DateTime(String),
}

impl Value {
    /// The value as a string for equality / membership comparison.
    pub fn as_str(&self) -> &str {
        match self {
            Value::Iri(s) | Value::Str(s) | Value::DateTime(s) => s,
            Value::Num(_) => "",
        }
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Iri(s) => write!(f, "<{s}>"),
            Value::Str(s) | Value::DateTime(s) => write!(f, "{s:?}"),
            Value::Num(n) => write!(f, "{n}"),
        }
    }
}

/// An ODRL duty (`odrl:duty`): an obligation that must be discharged for a
/// permission to be exercisable (the "usage-control kernel" pure access control
/// lacks). In the single-node base case a duty is modelled by its [`Action`] and
/// (optionally) its own constraints; the evaluator checks whether the request
/// context reports the duty as discharged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duty {
    /// The duty node IRI/blank-id (for justification).
    pub id: String,
    /// The action that must be performed (`odrl:action`), e.g.
    /// `odrl:anonymize`, `odrl:compensate`, a custom `proveAttestation`.
    pub action: Action,
    /// Constraints refining the duty (e.g. compensate `count = 5`). Carried for
    /// justification; the base case checks discharge by action.
    pub constraints: Vec<Constraint>,
}
