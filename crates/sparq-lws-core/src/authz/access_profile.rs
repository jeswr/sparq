// AUTHORED-BY Claude Sonnet 4.6
//! The strict ODRL access profile `https://w3id.org/jeswr/lws/access-profile/odrl-1` —
//! a Rust port of the LWS spec's NORMATIVE, executable access-decision rule set
//! (`lws-spec/semantics/access-decision.n3`, opt-in `access-profile-odrl1`, sq-gg0qq.6).
//!
//! The contract this module implements is [`jeswr/lws-spec`](https://github.com/jeswr/lws-spec),
//! vendored at a pinned commit under `lws-spec/` (see `lws-spec/README.md`). **Where this code
//! and the spec disagree, the SPEC WINS**: the rule set is the definition of the profile's
//! `evaluate-access` decision function, and the 19 `evaluate-access` test-vectors are point-wise
//! samples of it. `tests/lws_spec_vectors.rs` is the gate that keeps the port honest — it runs
//! every vendored vector through [`evaluate_access_json`] and reports a divergence as
//! `suite/vector-id`.
//!
//! ## Why a port and not the reasoner
//! Upstream executes the rule set under an N3 reasoner (EYE / `eyereasoner`), which needs Node.
//! This crate reproduces the same decision function natively, as a synchronous in-process call with
//! no external process and no I/O. The division of authority is deliberate: upstream's oracle checks
//! the rule set against the vectors; this port is checked against the same vectors. Re-deriving the
//! vectors from the rule set stays an opt-in lane (`lws-spec/README.md`).
//!
//! This module is a PURE library function and is deliberately NOT called from [`crate::ldp`], so
//! enabling the feature changes no request's outcome. Wiring the profile into the request path is a
//! separate, soundness-sensitive slice.
//!
//! ## The decision rule (closed-world, decision-time)
//! [`Decision::Permit`] iff SOME recorded grant justifies the request; [`Decision::Deny`] is the
//! closed-world absence of any justification. Revocation is structural — a revoked grant is
//! simply absent from the input — so revoking one of two covering grants cannot deny.
//!
//! A grant justifies a request when it is an `odrl:Offer` carrying exactly this profile IRI, one
//! of its `odrl:permission` rules matches, and — scoped to THAT SAME grant, never globally — no
//! `odrl:prohibition` rule matches (ODRL 2.2 `odrl:prohibit` conflict resolution: deny overrides)
//! and no `odrl:obligation` rule matches (the profile defines no wire representation of duty
//! fulfilment, so a matching obligation is unverifiable and blocks — fail closed).
//!
//! A rule matches when its assignee matches the agent, the requested action satisfies one of the
//! rule's granted actions, one of its targets covers the requested resource, and none of its own
//! constraints is unsatisfied (conjunctive). Every fallible edge fails CLOSED: an unknown action
//! grants nothing, a constraint the profile cannot evaluate is unsatisfied, a malformed constraint
//! is unsatisfied, and a missing request-context value is unsatisfied.
//!
//! ## Known asymmetry, ported deliberately (not a bug in this port)
//! Because "does this rule match" is one derivation shared by all three rule kinds, an omitted
//! request-context value makes a *prohibition*'s constraint unsatisfied too — so the prohibition
//! does not match and fails OPEN, while the same omission on a permission fails closed. Upstream
//! documents this in `docs/design/odrl-prohibition-indeterminate.md` and leaves `odrl-1`
//! semantics unchanged; changing it here would put this crate out of conformance. A caller that
//! needs the stricter reading must supply the full context, or wait for the successor profile.
//!
//! ## Untrusted-input discipline
//! Profile facts — which actions exist, the `odrl:includedIn` action lattice, and the
//! (leftOperand, operator) pairs this version can evaluate — are defined HERE as constants and
//! are never read from an evaluated document, so a hostile grant cannot inject them. The decoder
//! ([`Grant::from_json`] / [`AccessRequest::from_json`]) maps ONLY the fields of the profile's
//! document shape and REJECTS anything else, mirroring upstream's fail-loud encoder: an unknown
//! action, operand, operator, target type, or context key, a non-absolute IRI, or a non-canonical
//! `dateTime` is an error, never a silently dropped field.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde_json::Value;

/// The profile IRI a grant must carry for [`decide`] to consider it (rule D).
pub const PROFILE_ODRL1: &str = "https://w3id.org/jeswr/lws/access-profile/odrl-1";

const FOAF_AGENT: &str = "http://xmlns.com/foaf/0.1/Agent";

const ODRL_READ: &str = "http://www.w3.org/ns/odrl/2/read";
const ODRL_MODIFY: &str = "http://www.w3.org/ns/odrl/2/modify";
const ODRL_DELETE: &str = "http://www.w3.org/ns/odrl/2/delete";
const JLWS_CREATE: &str = "https://w3id.org/jeswr/lws#create";
const JLWS_APPEND: &str = "https://w3id.org/jeswr/lws#append";

const ODRL_PURPOSE: &str = "http://www.w3.org/ns/odrl/2/purpose";
const ODRL_DATETIME: &str = "http://www.w3.org/ns/odrl/2/dateTime";
const ODRL_EQ: &str = "http://www.w3.org/ns/odrl/2/eq";
const ODRL_LT: &str = "http://www.w3.org/ns/odrl/2/lt";
const JLWS_CLIENT: &str = "https://w3id.org/jeswr/lws#client";
const JLWS_MEDIA_TYPE: &str = "https://w3id.org/jeswr/lws#mediaType";
const JLWS_RESOURCE_TYPE: &str = "https://w3id.org/jeswr/lws#resourceType";

const JLWS_DATA_RESOURCE: &str = "https://w3id.org/jeswr/lws#DataResource";
const JLWS_CONTAINER: &str = "https://w3id.org/jeswr/lws#Container";
const JLWS_STORAGE_RESOURCE: &str = "https://w3id.org/jeswr/lws#StorageResource";

const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

/// Rule P — the profile's own action vocabulary. An action outside this set is UNKNOWN and can
/// never satisfy anything, not even a grant of itself (rule A1, fail closed).
const KNOWN_ACTIONS: [&str; 5] = [
    ODRL_READ,
    ODRL_MODIFY,
    ODRL_DELETE,
    JLWS_CREATE,
    JLWS_APPEND,
];

/// Rule P/A2 — the `odrl:includedIn` lattice, `(narrow, wide)`. One-directional BY CONSTRUCTION:
/// nothing derives the converse edge, so a `create`-only grant is never widened to `modify`.
const ACTION_INCLUDED_IN: [(&str, &str); 2] =
    [(JLWS_CREATE, ODRL_MODIFY), (JLWS_APPEND, ODRL_MODIFY)];

/// Rule P/K4 — the `(leftOperand, operator)` pairs THIS profile version can evaluate. A constraint
/// outside this table is unsatisfied (fail closed), never assumed to hold.
const SUPPORTED_CONSTRAINTS: [(&str, &str); 5] = [
    (ODRL_PURPOSE, ODRL_EQ),
    (JLWS_CLIENT, ODRL_EQ),
    (JLWS_MEDIA_TYPE, ODRL_EQ),
    (JLWS_RESOURCE_TYPE, ODRL_EQ),
    (ODRL_DATETIME, ODRL_LT),
];

/// The decision a request receives under the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// At least one recorded grant justifies the request.
    Permit,
    /// No recorded grant justifies the request at decision time (default deny).
    Deny,
}

impl Decision {
    /// The wire spelling the test-vectors use in `expected.decision` (`"permit"` / `"deny"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Decision::Permit => "permit",
            Decision::Deny => "deny",
        }
    }
}

/// A document this profile refused to decode. The message names the offending field or value; it
/// is for the OPERATOR (and the conformance report), never for a requester.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileError(String);

impl ProfileError {
    fn new(message: impl Into<String>) -> ProfileError {
        ProfileError(message.into())
    }
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ProfileError {}

/// An RDF term as the profile's encoding produces it. The IRI/literal distinction is load-bearing:
/// constraint equality (rule K1) is TERM equality, so the IRI `<https://p.example/x>` and the
/// literal `"https://p.example/x"` are different values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// An absolute IRI.
    Iri(String),
    /// A plain literal.
    Literal(String),
}

impl Term {
    /// The lexical form, for the string-ordered `dateTime` comparison (rule K3).
    fn lexical(&self) -> &str {
        match self {
            Term::Iri(s) | Term::Literal(s) => s,
        }
    }

    /// Is this a canonical RFC 3339 UTC instant literal? A non-literal term is NOT (fail closed —
    /// rules K3b/K3c only compare literals, and the decoder never produces a non-literal
    /// `dateTime` bound, so this only guards a programmatically-built [`Grant`]).
    fn is_canonical_instant(&self) -> bool {
        matches!(self, Term::Literal(s) if is_canonical_instant(s))
    }
}

/// The two document types of the profile's shape. Only an [`Offer`](DocumentKind::Offer) can
/// justify a request (rule D); a `Request` document records an ASK, not a grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    /// `odrl:Offer` — a recorded grant.
    Offer,
    /// `odrl:Request` — a recorded access request.
    Request,
}

/// One `odrl:constraint` of a rule. Every field is optional so a MALFORMED constraint round-trips
/// into the decision as "unsatisfied" (rules K5–K7) instead of being dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    left_operand: Option<String>,
    operator: Option<String>,
    right_operand: Option<Term>,
}

/// One `odrl:target` of a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    kind: Option<String>,
    uid: Option<String>,
    recursive: bool,
}

/// One `odrl:permission` / `odrl:prohibition` / `odrl:obligation` rule. All three kinds share this
/// shape, which is exactly why "does this rule match the request" is a single derivation (rule M)
/// reused by the permit, prohibition, and obligation strata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    assignee: Option<String>,
    actions: Vec<String>,
    targets: Vec<Target>,
    constraints: Vec<Constraint>,
}

/// A recorded access-grant document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    kind: DocumentKind,
    uid: Option<String>,
    profile: Option<String>,
    permissions: Vec<Rule>,
    prohibitions: Vec<Rule>,
    obligations: Vec<Rule>,
}

/// The request under decision. `context` is keyed by the constraint left-operand IRI — in this
/// profile the left-operand IRIs ARE the context keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRequest {
    agent: String,
    action: String,
    target: String,
    context: BTreeMap<String, Term>,
}

// ---------------------------------------------------------------------------
// Profile facts (rules P, A, B, C, K) — constants above, predicates here.
// ---------------------------------------------------------------------------

/// Rule A — does the requested action satisfy a rule's granted action? An unknown requested action
/// satisfies nothing (A1); inclusion widens one way only (A2).
fn satisfies_granted(requested: &str, granted: &str) -> bool {
    (requested == granted && KNOWN_ACTIONS.contains(&requested))
        || ACTION_INCLUDED_IN
            .iter()
            .any(|&(narrow, wide)| narrow == requested && wide == granted)
}

/// Rule K4 — can this profile version evaluate this `(leftOperand, operator)` pair?
fn supports_operator(left_operand: &str, operator: &str) -> bool {
    SUPPORTED_CONSTRAINTS
        .iter()
        .any(|&(l, o)| l == left_operand && o == operator)
}

/// A canonical RFC 3339 UTC instant — `YYYY-MM-DDTHH:MM:SSZ`, fixed width, no fractional seconds,
/// no offsets. This is the ONLY form under which lexicographic order is chronological, so rule K3's
/// string comparison is only sound over it; anything else fails closed (K3b/K3c) rather than
/// sorting arbitrarily and widening an `lt` bound to "never expires".
///
/// Every component is range-checked explicitly, month-specific day counts and Gregorian leap years
/// included, so a nonexistent instant (`2027-02-30`, `04-31`, `T24:00`, `:60`) is rejected instead
/// of being normalised into a comparable one.
fn is_canonical_instant(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 20 {
        return false;
    }
    let digits = |range: std::ops::Range<usize>| b[range].iter().all(|c| c.is_ascii_digit());
    if !(digits(0..4) && digits(5..7) && digits(8..10) && digits(11..13) && digits(14..16)
        && digits(17..19))
    {
        return false;
    }
    if !(b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':' && b[16] == b':'
        && b[19] == b'Z')
    {
        return false;
    }
    let num = |range: std::ops::Range<usize>| -> u32 {
        s[range].parse().expect("range-checked ASCII digits")
    };
    let (year, month, day) = (num(0..4), num(5..7), num(8..10));
    let (hour, minute, second) = (num(11..13), num(14..16), num(17..19));
    if !(1..=12).contains(&month) || day < 1 || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    day <= days_in_month(year, month)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    if month == 2 && leap {
        29
    } else {
        DAYS[(month - 1) as usize]
    }
}

impl Constraint {
    /// Rule K — is this constraint UNSATISFIED for the request? Every failure mode is positive and
    /// the caller requires the absence of all of them, so a constraint no rule below falsifies is
    /// satisfied. The clauses are an OR, so their order here is immaterial (it is in the rule set
    /// too — the strata fire independently).
    fn unsatisfied_for(&self, request: &AccessRequest) -> bool {
        // K5–K7: a malformed constraint can never be satisfied.
        let (Some(left), Some(operator), Some(right)) = (
            self.left_operand.as_deref(),
            self.operator.as_deref(),
            self.right_operand.as_ref(),
        ) else {
            return true;
        };
        // K4: a (leftOperand, operator) pair outside the profile's table.
        if !supports_operator(left, operator) {
            return true;
        }
        // K2: the request context carries no value for the left operand — nothing to check.
        let Some(observed) = request.context.get(left) else {
            return true;
        };
        if left == ODRL_DATETIME {
            // K3b/K3c: a non-canonical bound or request instant is incomparable.
            if !right.is_canonical_instant() || !observed.is_canonical_instant() {
                return true;
            }
            // K3: the request instant is not strictly before the bound (an expired grant).
            if operator == ODRL_LT && observed.lexical() >= right.lexical() {
                return true;
            }
        }
        // K1: `eq` against a different term.
        operator == ODRL_EQ && observed != right
    }
}

impl Target {
    /// Rule C — does this target cover the requested resource? Container and storage URIs are
    /// path-aligned (they end in `/`) and the prefix rules REQUIRE that trailing slash, which is
    /// what makes prefix coverage segment-safe: `…/notes/` covers `…/notes/deep/file.txt` but
    /// never `…/notes-evil.txt`, and a non-slash-terminated uid grants no prefix coverage at all.
    fn covers(&self, resource: &str) -> bool {
        let Some(uid) = self.uid.as_deref() else {
            return false;
        };
        match self.kind.as_deref() {
            // C1: a data resource covers exactly its own URI.
            Some(JLWS_DATA_RESOURCE) => uid == resource,
            // C2 + C3: a container covers itself, and its descendants only when recursive.
            Some(JLWS_CONTAINER) => {
                uid == resource
                    || (self.recursive && uid.ends_with('/') && resource.starts_with(uid))
            }
            // C4: a storage covers every resource in it.
            Some(JLWS_STORAGE_RESOURCE) => uid.ends_with('/') && resource.starts_with(uid),
            // An absent or extension target type covers nothing (no rule fires — fail closed).
            _ => false,
        }
    }
}

impl Rule {
    /// Rule M — does this rule match the request? Shared by all three rule kinds.
    fn matches(&self, request: &AccessRequest) -> bool {
        // B1/B2: the named assignee, or foaf:Agent for public access. An assignee-less rule
        // matches nothing.
        let assignee_matches = self
            .assignee
            .as_deref()
            .is_some_and(|a| a == FOAF_AGENT || a == request.agent);
        assignee_matches
            && self
                .actions
                .iter()
                .any(|granted| satisfies_granted(&request.action, granted))
            && self.targets.iter().any(|t| t.covers(&request.target))
            && !self.constraints.iter().any(|c| c.unsatisfied_for(request))
    }
}

/// Rule D — the decision. [`Decision::Permit`] iff some grant in `grants` justifies `request`;
/// [`Decision::Deny`] is the closed-world absence of any justification, so a revoked grant denies
/// precisely by being absent from `grants`.
///
/// Prohibition and obligation composition is PER GRANT: a prohibition recorded in one grant never
/// reaches into another grant's justification.
pub fn decide(grants: &[Grant], request: &AccessRequest) -> Decision {
    for grant in grants {
        if grant.kind != DocumentKind::Offer || grant.profile.as_deref() != Some(PROFILE_ODRL1) {
            continue;
        }
        // N: a matching prohibition of THIS grant overrides its own permissions (deny overrides).
        // O: a matching obligation of THIS grant is unverifiable, so it blocks too (fail closed).
        let blocked = grant.prohibitions.iter().any(|r| r.matches(request))
            || grant.obligations.iter().any(|r| r.matches(request));
        if blocked {
            continue;
        }
        if grant.permissions.iter().any(|r| r.matches(request)) {
            return Decision::Permit;
        }
    }
    Decision::Deny
}

// ---------------------------------------------------------------------------
// The decoder — the profile's document shape -> terms, fail-loud.
//
// Mirrors the upstream oracle's encoder field for field (lws-spec
// test-suite/tools/oracle-access.mjs). Anything outside the shape is an error, never a silently
// dropped field: a dropped `odrl:prohibition` would turn a deny into a permit.
// ---------------------------------------------------------------------------

/// An absolute `http(s)` IRI with no character that would need escaping in N3.
fn is_absolute_iri(s: &str) -> bool {
    let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty()
        && !s.chars().any(|c| {
            c.is_whitespace() || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '\\' | '^' | '`')
        })
}

fn as_iri(value: &Value, field: &str) -> Result<String, ProfileError> {
    match value.as_str() {
        Some(s) if is_absolute_iri(s) => Ok(s.to_owned()),
        _ => Err(ProfileError::new(format!(
            "{}: not an encodable IRI: {}",
            field, value
        ))),
    }
}

fn as_literal(value: &Value, field: &str) -> Result<Term, ProfileError> {
    match value.as_str() {
        Some(s) => Ok(Term::Literal(s.to_owned())),
        None => Err(ProfileError::new(format!(
            "{}: not an encodable literal: {}",
            field, value
        ))),
    }
}

/// A compact term mapped through a profile table, an absolute IRI passed through, or an error.
fn mapped(
    value: &Value,
    table: &[(&str, &str)],
    label: &str,
) -> Result<String, ProfileError> {
    if let Some(s) = value.as_str() {
        if let Some(&(_, iri)) = table.iter().find(|&&(term, _)| term == s) {
            return Ok(iri.to_owned());
        }
        if is_absolute_iri(s) {
            return Ok(s.to_owned());
        }
    }
    Err(ProfileError::new(format!("unknown {}: {}", label, value)))
}

const ACTION_TERMS: [(&str, &str); 5] = [
    ("create", JLWS_CREATE),
    ("append", JLWS_APPEND),
    ("read", ODRL_READ),
    ("modify", ODRL_MODIFY),
    ("delete", ODRL_DELETE),
];

const TARGET_TYPE_TERMS: [(&str, &str); 3] = [
    ("DataResource", JLWS_DATA_RESOURCE),
    ("Container", JLWS_CONTAINER),
    ("StorageResource", JLWS_STORAGE_RESOURCE),
];

const LEFT_OPERAND_TERMS: [(&str, &str); 5] = [
    ("purpose", ODRL_PURPOSE),
    ("dateTime", ODRL_DATETIME),
    ("client", JLWS_CLIENT),
    ("mediaType", JLWS_MEDIA_TYPE),
    ("resourceType", JLWS_RESOURCE_TYPE),
];

const OPERATOR_TERMS: [(&str, &str); 6] = [
    ("eq", "http://www.w3.org/ns/odrl/2/eq"),
    ("lt", "http://www.w3.org/ns/odrl/2/lt"),
    ("gt", "http://www.w3.org/ns/odrl/2/gt"),
    ("lteq", "http://www.w3.org/ns/odrl/2/lteq"),
    ("gteq", "http://www.w3.org/ns/odrl/2/gteq"),
    ("neq", "http://www.w3.org/ns/odrl/2/neq"),
];

/// The five request-context keys, paired with the left-operand IRI each one carries. The value
/// encoding differs per key: `dateTime` must be a canonical instant, `mediaType` is always a
/// literal, and the rest are an IRI when absolute and a literal otherwise.
const CONTEXT_KEYS: [(&str, &str); 5] = [
    ("dateTime", ODRL_DATETIME),
    ("purpose", ODRL_PURPOSE),
    ("client", JLWS_CLIENT),
    ("mediaType", JLWS_MEDIA_TYPE),
    ("resourceType", JLWS_RESOURCE_TYPE),
];

/// `null`/absent -> empty; a JSON array -> its items; anything else -> a one-item list. Matches the
/// profile's shape, where a rule-bearing predicate may be a single object or an array.
fn to_array(value: Option<&Value>) -> Vec<&Value> {
    match value {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items.iter().collect(),
        Some(other) => vec![other],
    }
}

fn instant(value: &Value, field: &str) -> Result<Term, ProfileError> {
    match value.as_str() {
        Some(s) if is_canonical_instant(s) => Ok(Term::Literal(s.to_owned())),
        _ => Err(ProfileError::new(format!(
            "{}: not a canonical RFC 3339 UTC instant: {}",
            field, value
        ))),
    }
}

/// A `dateTime` bound: `xsd:dateTime`-typed (or a bare string) AND canonical. A foreign datatype is
/// an error, never silently lowered into a comparable string.
fn date_time_operand(value: &Value) -> Result<Term, ProfileError> {
    if let Some(inner) = value.get("@value") {
        let datatype = value.get("@type").and_then(Value::as_str);
        if datatype != Some("xsd:dateTime") && datatype != Some(XSD_DATETIME) {
            return Err(ProfileError::new(format!(
                "dateTime bound carries a non-xsd:dateTime datatype: {}",
                value.get("@type").unwrap_or(&Value::Null)
            )));
        }
        return instant(inner, "rightOperand");
    }
    instant(value, "rightOperand")
}

fn right_operand(value: &Value) -> Result<Term, ProfileError> {
    if let Some(inner) = value.get("@value") {
        return as_literal(inner, "rightOperand");
    }
    match value.as_str() {
        Some(s) if is_absolute_iri(s) => Ok(Term::Iri(s.to_owned())),
        _ => as_literal(value, "rightOperand"),
    }
}

fn decode_constraint(value: &Value) -> Result<Constraint, ProfileError> {
    let left_operand = match value.get("leftOperand") {
        Some(l) => Some(mapped(l, &LEFT_OPERAND_TERMS, "constraint leftOperand")?),
        None => None,
    };
    let operator = match value.get("operator") {
        Some(o) => Some(mapped(o, &OPERATOR_TERMS, "constraint operator")?),
        None => None,
    };
    let right = match value.get("rightOperand") {
        Some(r) if left_operand.as_deref() == Some(ODRL_DATETIME) => Some(date_time_operand(r)?),
        Some(r) => Some(right_operand(r)?),
        None => None,
    };
    Ok(Constraint {
        left_operand,
        operator,
        right_operand: right,
    })
}

fn decode_target(value: &Value) -> Result<Target, ProfileError> {
    let kind = match value.get("@type") {
        Some(t) => Some(mapped(t, &TARGET_TYPE_TERMS, "target @type")?),
        None => None,
    };
    let uid = match value.get("uid") {
        Some(u) => Some(as_iri(u, "target uid")?),
        None => None,
    };
    Ok(Target {
        kind,
        uid,
        // Strictly boolean `true`; a truthy string or 1 is not the profile's spelling.
        recursive: value.get("recursive") == Some(&Value::Bool(true)),
    })
}

fn decode_rules(grant: &Value, key: &str) -> Result<Vec<Rule>, ProfileError> {
    to_array(grant.get(key))
        .into_iter()
        .map(|rule| {
            let assignee = match rule.get("assignee") {
                Some(a) => Some(as_iri(a, "assignee")?),
                None => None,
            };
            let actions = to_array(rule.get("action"))
                .into_iter()
                .map(|a| mapped(a, &ACTION_TERMS, "action"))
                .collect::<Result<Vec<_>, _>>()?;
            let targets = to_array(rule.get("target"))
                .into_iter()
                .map(decode_target)
                .collect::<Result<Vec<_>, _>>()?;
            let constraints = to_array(rule.get("constraint"))
                .into_iter()
                .map(decode_constraint)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Rule {
                assignee,
                actions,
                targets,
                constraints,
            })
        })
        .collect()
}

impl Grant {
    /// Decode one recorded grant document from the profile's JSON-LD shape.
    ///
    /// Strict by design: a missing or unknown `@type` is an error and is NEVER fabricated into an
    /// `odrl:Offer` (an Offer is what [`decide`] keys on), and every IRI-valued field must be an
    /// absolute `http(s)` IRI.
    pub fn from_json(value: &Value) -> Result<Grant, ProfileError> {
        let kind = match value.get("@type").and_then(Value::as_str) {
            Some("Offer") => DocumentKind::Offer,
            Some("Request") => DocumentKind::Request,
            other => {
                return Err(ProfileError::new(format!(
                    "grant record with missing/unknown @type: {:?}",
                    other
                )))
            }
        };
        let uid = match value.get("uid") {
            Some(u) => Some(as_iri(u, "grant uid")?),
            None => None,
        };
        let profile = match value.get("profile") {
            Some(p) => Some(as_iri(p, "profile")?),
            None => None,
        };
        Ok(Grant {
            kind,
            uid,
            profile,
            permissions: decode_rules(value, "permission")?,
            prohibitions: decode_rules(value, "prohibition")?,
            obligations: decode_rules(value, "obligation")?,
        })
    }

    /// The grant's `uid`, when it carries one.
    pub fn uid(&self) -> Option<&str> {
        self.uid.as_deref()
    }
}

impl AccessRequest {
    /// Decode the request under decision from the profile's JSON shape. `agent`, `action`, and
    /// `target` are required; every `context` key must be one of the profile's five left operands.
    pub fn from_json(value: &Value) -> Result<AccessRequest, ProfileError> {
        let agent = as_iri(
            value
                .get("agent")
                .ok_or_else(|| ProfileError::new("request: missing agent"))?,
            "agent",
        )?;
        let action = mapped(
            value
                .get("action")
                .ok_or_else(|| ProfileError::new("request: missing action"))?,
            &ACTION_TERMS,
            "action",
        )?;
        let target = as_iri(
            value
                .get("target")
                .ok_or_else(|| ProfileError::new("request: missing target"))?,
            "target",
        )?;
        let mut context = BTreeMap::new();
        if let Some(entries) = value.get("context") {
            let object = entries.as_object().ok_or_else(|| {
                ProfileError::new(format!("request context is not an object: {}", entries))
            })?;
            for (key, raw) in object {
                let Some(&(_, operand)) = CONTEXT_KEYS.iter().find(|&&(k, _)| k == key) else {
                    return Err(ProfileError::new(format!(
                        "unknown request context key: {:?}",
                        key
                    )));
                };
                let term = match operand {
                    ODRL_DATETIME => instant(raw, "context dateTime")?,
                    JLWS_MEDIA_TYPE => as_literal(raw, "context mediaType")?,
                    _ => match raw.as_str() {
                        Some(s) if is_absolute_iri(s) => Term::Iri(s.to_owned()),
                        _ => as_literal(raw, "context value")?,
                    },
                };
                context.insert(operand.to_owned(), term);
            }
        }
        Ok(AccessRequest {
            agent,
            action,
            target,
            context,
        })
    }
}

/// Decode and decide one `evaluate-access` input — the `{ "grants": [...], "request": {...} }`
/// object the LWS test-vectors carry — in a single call.
///
/// A decode error is returned, NOT folded into [`Decision::Deny`]: a document this profile cannot
/// read is an operator-visible fault, and silently denying would hide it.
pub fn evaluate_access_json(input: &Value) -> Result<Decision, ProfileError> {
    let grants = to_array(input.get("grants"))
        .into_iter()
        .map(Grant::from_json)
        .collect::<Result<Vec<_>, _>>()?;
    let request = AccessRequest::from_json(
        input
            .get("request")
            .ok_or_else(|| ProfileError::new("input: missing request"))?,
    )?;
    Ok(decide(&grants, &request))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ALICE: &str = "https://id.example/alice";
    const BOB: &str = "https://id.example/bob";
    const DOC: &str = "https://storage.example/alice/notes/a.txt";
    const NOW: &str = "2026-07-01T12:00:00Z";

    /// A read grant to Bob on DOC, with the caller's extra rules spliced in.
    fn grant(extra: Value) -> Value {
        let mut base = json!({
            "@context": ["http://www.w3.org/ns/odrl.jsonld", "https://w3id.org/jeswr/lws/v1"],
            "@type": "Offer",
            "uid": "https://storage.example/alice/.grants/g1",
            "assigner": ALICE,
            "profile": PROFILE_ODRL1,
            "permission": [{
                "action": "read",
                "assignee": BOB,
                "target": {"@type": "DataResource", "uid": DOC},
            }],
        });
        let (Value::Object(base_map), Value::Object(extra_map)) = (&mut base, extra) else {
            unreachable!("both literals are objects")
        };
        base_map.extend(extra_map);
        base
    }

    fn read_request() -> Value {
        json!({"action": "read", "agent": BOB, "target": DOC, "context": {"dateTime": NOW}})
    }

    fn decide_json(grants: Value, request: Value) -> Decision {
        evaluate_access_json(&json!({"grants": grants, "request": request}))
            .expect("decodable input")
    }

    #[test]
    fn evaluate_access_json_permits_a_matching_grant() {
        assert_eq!(
            decide_json(json!([grant(json!({}))]), read_request()),
            Decision::Permit
        );
    }

    #[test]
    fn evaluate_access_json_denies_with_no_recorded_grant() {
        assert_eq!(decide_json(json!([]), read_request()), Decision::Deny);
    }

    #[test]
    fn evaluate_access_json_rejects_an_unknown_context_key() {
        let input = json!({
            "grants": [grant(json!({}))],
            "request": {"action": "read", "agent": BOB, "target": DOC,
                        "context": {"geolocation": "https://geo.example/eu"}},
        });
        let err = evaluate_access_json(&input).expect_err("unknown context key");
        assert!(err.to_string().contains("unknown request context key"), "{}", err);
    }

    #[test]
    fn decide_requires_the_profile_iri() {
        // A grant that is otherwise a perfect match but carries a different profile justifies
        // nothing: rule D keys on this exact profile.
        let foreign = grant(json!({"profile": "https://w3id.org/jeswr/lws/access-profile/other"}));
        assert_eq!(decide_json(json!([foreign]), read_request()), Decision::Deny);
    }

    #[test]
    fn decide_ignores_a_request_document() {
        let asked = grant(json!({"@type": "Request"}));
        assert_eq!(decide_json(json!([asked]), read_request()), Decision::Deny);
    }

    #[test]
    fn decide_composes_prohibition_and_obligation_per_grant() {
        let deny_rule = json!([{"action": "read", "assignee": BOB,
                                "target": {"@type": "DataResource", "uid": DOC}}]);
        // A matching prohibition of the SAME grant overrides its own permission...
        let prohibited = grant(json!({"prohibition": deny_rule.clone()}));
        assert_eq!(
            decide_json(json!([prohibited.clone()]), read_request()),
            Decision::Deny
        );
        // ...an obligation of the same grant blocks it too (no discharge mechanism)...
        let obliged = grant(json!({"obligation": deny_rule}));
        assert_eq!(decide_json(json!([obliged]), read_request()), Decision::Deny);
        // ...but neither reaches into a DIFFERENT grant that also covers the request.
        let clean = grant(json!({"uid": "https://storage.example/alice/.grants/g2"}));
        assert_eq!(
            decide_json(json!([prohibited, clean]), read_request()),
            Decision::Permit
        );
    }

    #[test]
    fn satisfies_granted_widens_one_way_only() {
        assert!(satisfies_granted(JLWS_CREATE, ODRL_MODIFY));
        assert!(satisfies_granted(JLWS_APPEND, ODRL_MODIFY));
        assert!(!satisfies_granted(ODRL_MODIFY, JLWS_CREATE));
        assert!(satisfies_granted(ODRL_READ, ODRL_READ));
        // An unknown action cannot even satisfy a grant of itself (rule A1, fail closed).
        let extension = "https://extension.example/administer";
        assert!(!satisfies_granted(extension, extension));
    }

    #[test]
    fn supports_operator_is_the_closed_profile_table() {
        assert!(supports_operator(ODRL_DATETIME, ODRL_LT));
        assert!(supports_operator(ODRL_PURPOSE, ODRL_EQ));
        // `dateTime eq` and `purpose lt` are NOT in the table — fail closed.
        assert!(!supports_operator(ODRL_DATETIME, ODRL_EQ));
        assert!(!supports_operator(ODRL_PURPOSE, ODRL_LT));
    }

    #[test]
    fn target_coverage_is_slash_guarded() {
        let container = |uid: &str, recursive: bool| Target {
            kind: Some(JLWS_CONTAINER.to_owned()),
            uid: Some(uid.to_owned()),
            recursive,
        };
        let notes = "https://storage.example/alice/notes/";
        assert!(container(notes, true).covers("https://storage.example/alice/notes/deep/f.txt"));
        assert!(container(notes, true).covers(notes));
        // Non-recursive covers only the container itself.
        assert!(!container(notes, false).covers("https://storage.example/alice/notes/f.txt"));
        // A uid without the trailing slash grants NO prefix coverage — the sibling-prefix trap.
        let unslashed = "https://storage.example/alice/notes";
        assert!(!container(unslashed, true).covers("https://storage.example/alice/notes-evil.txt"));
        // A data resource covers exactly itself.
        let data = Target {
            kind: Some(JLWS_DATA_RESOURCE.to_owned()),
            uid: Some(notes.to_owned()),
            recursive: true,
        };
        assert!(!data.covers("https://storage.example/alice/notes/f.txt"));
    }

    #[test]
    fn is_canonical_instant_rejects_nonexistent_instants() {
        assert!(is_canonical_instant("2026-07-01T12:00:00Z"));
        assert!(is_canonical_instant("2028-02-29T00:00:00Z"));
        assert!(is_canonical_instant("2000-02-29T00:00:00Z"));
        for bad in [
            "2027-02-29T00:00:00Z", // not a leap year
            "1900-02-29T00:00:00Z", // century, not divisible by 400
            "2026-02-30T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-07-01T24:00:00Z",
            "2026-07-01T12:60:00Z",
            "2026-00-01T12:00:00Z",
            "2026-07-01T12:00:00+00:00",
            "2026-07-01T12:00:00.000Z",
            "zzzz",
        ] {
            assert!(!is_canonical_instant(bad), "accepted {}", bad);
        }
    }

    #[test]
    fn a_malformed_datetime_bound_fails_closed_in_the_decision() {
        // The decoder rejects a non-canonical bound up front, so reach past it and build the
        // constraint directly: the DECISION itself must also refuse to compare an incomparable
        // bound, or "zzzz" would sort after every real instant and mean "never expires".
        let request = AccessRequest::from_json(&read_request()).expect("decodable request");
        let bound = |literal: &str| Constraint {
            left_operand: Some(ODRL_DATETIME.to_owned()),
            operator: Some(ODRL_LT.to_owned()),
            right_operand: Some(Term::Literal(literal.to_owned())),
        };
        assert!(bound("zzzz").unsatisfied_for(&request));
        assert!(bound("2027-02-30T00:00:00Z").unsatisfied_for(&request));
        // A real, later bound is satisfied — the guard is not just rejecting everything.
        assert!(!bound("2026-12-31T00:00:00Z").unsatisfied_for(&request));
        // ...and an IRI-valued bound is incomparable, so it fails closed too.
        let iri_bound = Constraint {
            right_operand: Some(Term::Iri("https://example.org/soon".to_owned())),
            ..bound("2026-12-31T00:00:00Z")
        };
        assert!(iri_bound.unsatisfied_for(&request));
    }

    #[test]
    fn constraint_equality_is_term_equality() {
        let request = AccessRequest::from_json(&json!({
            "action": "read", "agent": BOB, "target": DOC,
            "context": {"dateTime": NOW, "purpose": "https://purpose.example/collaboration"},
        }))
        .expect("decodable request");
        let purpose = |right: Term| Constraint {
            left_operand: Some(ODRL_PURPOSE.to_owned()),
            operator: Some(ODRL_EQ.to_owned()),
            right_operand: Some(right),
        };
        let iri = Term::Iri("https://purpose.example/collaboration".to_owned());
        assert!(!purpose(iri).unsatisfied_for(&request));
        // The same characters as a LITERAL is a different term, so the constraint is unsatisfied.
        let literal = Term::Literal("https://purpose.example/collaboration".to_owned());
        assert!(purpose(literal).unsatisfied_for(&request));
    }

    #[test]
    fn a_malformed_constraint_is_unsatisfied() {
        let request = AccessRequest::from_json(&read_request()).expect("decodable request");
        let empty = Constraint {
            left_operand: None,
            operator: None,
            right_operand: None,
        };
        assert!(empty.unsatisfied_for(&request));
        // Present operand + operator, missing bound (K7).
        let no_bound = Constraint {
            left_operand: Some(ODRL_DATETIME.to_owned()),
            operator: Some(ODRL_LT.to_owned()),
            right_operand: None,
        };
        assert!(no_bound.unsatisfied_for(&request));
        // A supported pair whose left operand the request context omits (K2).
        let no_context = Constraint {
            left_operand: Some(ODRL_PURPOSE.to_owned()),
            operator: Some(ODRL_EQ.to_owned()),
            right_operand: Some(Term::Iri("https://purpose.example/x".to_owned())),
        };
        assert!(no_context.unsatisfied_for(&request));
    }

    #[test]
    fn is_absolute_iri_rejects_relative_and_unescapable_forms() {
        assert!(is_absolute_iri("https://id.example/bob"));
        assert!(is_absolute_iri("http://xmlns.com/foaf/0.1/Agent"));
        for bad in [
            "",
            "https://",
            "id.example/bob",
            "urn:uuid:1234",
            "https://id.example/a b",
            "https://id.example/<b>",
            "https://id.example/\"b\"",
        ] {
            assert!(!is_absolute_iri(bad), "accepted {}", bad);
        }
    }

    #[test]
    fn grant_from_json_refuses_a_missing_type() {
        let mut without = grant(json!({}));
        without.as_object_mut().expect("object").remove("@type");
        let err = Grant::from_json(&without).expect_err("missing @type");
        assert!(err.to_string().contains("missing/unknown @type"), "{}", err);
    }

    #[test]
    fn grant_from_json_refuses_an_unencodable_field() {
        let err = Grant::from_json(&grant(json!({"profile": "not-an-iri"})))
            .expect_err("relative profile IRI");
        assert!(err.to_string().contains("not an encodable IRI"), "{}", err);
        let bad_action = grant(json!({"permission": [{
            "action": "administer", "assignee": BOB,
            "target": {"@type": "DataResource", "uid": DOC},
        }]}));
        let err = Grant::from_json(&bad_action).expect_err("unknown compact action");
        assert!(err.to_string().contains("unknown action"), "{}", err);
    }

    #[test]
    fn grant_uid_is_exposed_for_reporting() {
        let decoded = Grant::from_json(&grant(json!({}))).expect("decodable grant");
        assert_eq!(decoded.uid(), Some("https://storage.example/alice/.grants/g1"));
    }

    #[test]
    fn decision_as_str_matches_the_vector_spelling() {
        assert_eq!(Decision::Permit.as_str(), "permit");
        assert_eq!(Decision::Deny.as_str(), "deny");
    }

    #[test]
    fn to_array_normalises_the_profile_shape() {
        assert!(to_array(None).is_empty());
        assert!(to_array(Some(&Value::Null)).is_empty());
        assert_eq!(to_array(Some(&json!([1, 2]))).len(), 2);
        assert_eq!(to_array(Some(&json!({"a": 1}))).len(), 1);
    }
}
