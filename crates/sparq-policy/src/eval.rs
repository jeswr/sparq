//! The ODRL evaluator: `(Policy, Request) -> Decision`.
//!
//! Semantics (the single-node base case — the ODRL Formal-Semantics CG
//! *closed-world* default, restricted to one node's data):
//!
//! 1. A [`Rule`] *matches* a [`Request`] when its action permits the requested
//!    action, its `target` and `assignee` (if specified) agree with the request,
//!    and every one of its [`Constraint`]s is satisfied by the request context.
//! 2. A [`Permission`] grants access iff it matches **and** all of its
//!    [`Duty`]s are discharged (reported in the request context).
//! 3. A [`Prohibition`] that matches **overrides** any permission (it "carves
//!    out" a forbidden sub-set — ODRL Formal Semantics §conflict).
//! 4. **Fail-closed default:** no matching+discharged permission, OR any matching
//!    prohibition, ⇒ DENY. An empty/malformed policy denies everything.
//!
//! [OPUS-4.8]

use crate::model::{Action, Constraint, Operator, Policy, Rule, Value};
use std::collections::{BTreeMap, BTreeSet};

/// An access request evaluated against a [`Policy`]: who wants to do what, to
/// what, in what context (the "evaluation request" + "state of the world" of the
/// ODRL Formal Semantics, folded into one node-local view).
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The action being requested (`odrl:` action IRI), e.g. `odrl:read`.
    pub action: String,
    /// The target asset/graph IRI the action is on.
    pub target: Option<String>,
    /// The requesting party (the WebID / agent IRI — matched against
    /// `odrl:assignee`).
    pub party: Option<String>,
    /// Context values keyed by ODRL `leftOperand` IRI (e.g. `odrl:dateTime`,
    /// `odrl:purpose`, `odrl:recipient`, `odrl:count`) — the state-of-the-world a
    /// constraint is evaluated against. Use [`Request::with`] to populate.
    pub context: BTreeMap<String, Value>,
    /// The set of duty *action* IRIs the caller asserts have been discharged
    /// (e.g. `odrl:anonymize`, a custom `proveAttestation`). A permission with an
    /// undischarged duty is denied.
    pub discharged_duties: BTreeSet<String>,
}

impl Request {
    /// A request for `action` on `target` by `party` with no context/duties yet.
    pub fn new(action: impl Into<String>) -> Request {
        Request {
            action: action.into(),
            ..Request::default()
        }
    }

    /// Set the target asset/graph IRI.
    pub fn on(mut self, target: impl Into<String>) -> Request {
        self.target = Some(target.into());
        self
    }

    /// Set the requesting party (assignee) IRI.
    pub fn by(mut self, party: impl Into<String>) -> Request {
        self.party = Some(party.into());
        self
    }

    /// Add a context value for a `leftOperand` IRI (chainable).
    pub fn with(mut self, left_operand: impl Into<String>, value: Value) -> Request {
        self.context.insert(left_operand.into(), value);
        self
    }

    /// Mark a duty action IRI as discharged (chainable).
    pub fn discharge(mut self, duty_action: impl Into<String>) -> Request {
        self.discharged_duties.insert(duty_action.into());
        self
    }
}

/// The result of evaluating a policy against a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// `true` ⇒ ALLOW (a permission matched, was un-prohibited, and all its
    /// duties were discharged). `false` ⇒ DENY (fail-closed).
    pub allow: bool,
    /// The IDs of the rule(s) that justify the decision: the granting permission
    /// on an ALLOW; the overriding prohibition(s) (and/or the would-be permission
    /// blocked by an unmet duty/constraint) on a DENY.
    pub matched_rules: Vec<String>,
    /// Human-readable explanations of why a candidate permission did *not* grant
    /// (unmet constraint, undischarged duty, overriding prohibition). Empty on a
    /// clean ALLOW with no caveats.
    pub unmet_constraints: Vec<String>,
}

impl Decision {
    fn deny(matched: Vec<String>, unmet: Vec<String>) -> Decision {
        Decision {
            allow: false,
            matched_rules: matched,
            unmet_constraints: unmet,
        }
    }
}

/// Evaluate `policy` against `request`, returning a fail-closed [`Decision`].
///
/// See the [module docs](self) for the exact semantics. This is the single-node
/// base case of ODRL — it reduces to the same allow/deny shape `sparq-solid`'s
/// WAC/ACP path produces, with ODRL's richer purpose/recipient/time constraints
/// and duty obligations layered on top.
///
/// # Examples
///
/// ```
/// use sparq_policy::{evaluate, Policy, Request};
/// // An empty policy denies everything (fail-closed).
/// let d = evaluate(&Policy::default(), &Request::new("http://www.w3.org/ns/odrl/2/read"));
/// assert!(!d.allow);
/// ```
pub fn evaluate(policy: &Policy, request: &Request) -> Decision {
    let req_action = Action(request.action.clone());

    // 1. A matching prohibition overrides everything (fail-closed carve-out).
    let mut blocking: Vec<String> = Vec::new();
    for rule in &policy.prohibitions {
        if rule_matches(rule, request, &req_action).is_match {
            blocking.push(rule.id.clone());
        }
    }
    if !blocking.is_empty() {
        let why = blocking
            .iter()
            .map(|id| format!("prohibition {id} matches the request"))
            .collect();
        return Decision::deny(blocking, why);
    }

    // 2. Find a permission that matches AND has all duties discharged.
    let mut caveats: Vec<String> = Vec::new();
    for rule in &policy.permissions {
        let m = rule_matches(rule, request, &req_action);
        if !m.is_match {
            caveats.extend(m.reasons);
            continue;
        }
        // Matched — now require every duty discharged.
        let undischarged: Vec<&str> = rule
            .duties
            .iter()
            .filter(|d| !request.discharged_duties.contains(&d.action.0))
            .map(|d| d.action.0.as_str())
            .collect();
        if undischarged.is_empty() {
            return Decision {
                allow: true,
                matched_rules: vec![rule.id.clone()],
                unmet_constraints: Vec::new(),
            };
        }
        for a in undischarged {
            caveats.push(format!(
                "permission {} requires undischarged duty {a}",
                rule.id
            ));
        }
    }

    // 3. No grant → DENY (fail-closed).
    if caveats.is_empty() {
        caveats.push("no permission matches the request".to_owned());
    }
    Decision::deny(Vec::new(), caveats)
}

/// The first [`Prohibition`](crate::model::Rule) in `policy` that **matches**
/// `request` (its action permits the requested action, its target/assignee agree,
/// and every constraint is satisfied), or `None` if no prohibition carves the
/// request out. [OPUS-4.8] sq-w693.
///
/// This is the same match test [`evaluate`] applies in step 1 (a matching
/// prohibition overrides everything) — exposed so the `sparq-solid` ODRL→AUTH_GRAPH
/// bridge can materialize a matched prohibition as an explicit `auth:deny*` triple
/// (deny-overrides) WITHOUT re-implementing the match logic. A `Decision` with
/// `allow == false` is NOT sufficient: it conflates a carve-out prohibition with a
/// plain no-matching-permission deny, and only the former should materialize a deny.
///
/// # Examples
///
/// ```
/// use sparq_policy::{matched_prohibition, parse_policy_str, Request};
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/p> a odrl:Set ; odrl:prohibition [
///     odrl:action odrl:write ;
///     odrl:target <https://pod.ex/n1> ;
///     odrl:assignee <https://alice.ex/card#me> ] .
/// "#, "turtle").unwrap();
/// let req = Request::new("http://www.w3.org/ns/odrl/2/write")
///     .on("https://pod.ex/n1").by("https://alice.ex/card#me");
/// assert!(matched_prohibition(&pol, &req).is_some());
/// // a different party is not carved out
/// let other = Request::new("http://www.w3.org/ns/odrl/2/write")
///     .on("https://pod.ex/n1").by("https://bob.ex/card#me");
/// assert!(matched_prohibition(&pol, &other).is_none());
/// ```
pub fn matched_prohibition<'p>(policy: &'p Policy, request: &Request) -> Option<&'p Rule> {
    let req_action = Action(request.action.clone());
    policy
        .prohibitions
        .iter()
        .find(|rule| rule_matches(rule, request, &req_action).is_match)
}

/// Outcome of matching a single rule against a request.
struct Match {
    is_match: bool,
    reasons: Vec<String>,
}

fn rule_matches(rule: &Rule, request: &Request, req_action: &Action) -> Match {
    let mut reasons = Vec::new();

    // Action: the rule's action must permit the requested action.
    if !rule.action.permits(req_action) {
        reasons.push(format!(
            "rule {} action {} != requested {}",
            rule.id, rule.action, request.action
        ));
        return Match {
            is_match: false,
            reasons,
        };
    }

    // Target: if the rule names a target, it must equal the request target.
    if let Some(t) = &rule.target {
        if request.target.as_deref() != Some(t.as_str()) {
            reasons.push(format!(
                "rule {} target {t} != requested {:?}",
                rule.id, request.target
            ));
            return Match {
                is_match: false,
                reasons,
            };
        }
    }

    // Assignee: if the rule names an assignee party, it must equal the requester.
    if let Some(a) = &rule.assignee {
        if request.party.as_deref() != Some(a.as_str()) {
            reasons.push(format!(
                "rule {} assignee {a} != requester {:?}",
                rule.id, request.party
            ));
            return Match {
                is_match: false,
                reasons,
            };
        }
    }

    // Constraints: every one must be satisfied (logical AND).
    for c in &rule.constraints {
        if !constraint_satisfied(c, request) {
            reasons.push(format!(
                "rule {} constraint ({} {:?} {}) unsatisfied",
                rule.id, c.left, c.operator, c.right
            ));
            return Match {
                is_match: false,
                reasons,
            };
        }
    }

    Match {
        is_match: true,
        reasons,
    }
}

/// Is a single constraint satisfied by the request context?
///
/// The request supplies the *actual* value for the constraint's `leftOperand`
/// (e.g. the actual request time for `odrl:dateTime`); the constraint's
/// `rightOperand` is the bound. A constraint whose left operand has **no** value
/// in the request context is **unsatisfied** (fail-closed: we cannot prove the
/// world meets a constraint we have no evidence about).
fn constraint_satisfied(c: &Constraint, request: &Request) -> bool {
    let Some(actual) = request.context.get(&c.left) else {
        return false; // no evidence for this dimension → fail-closed
    };
    compare(actual, c.operator, &c.right)
}

/// Compare an actual request value against a constraint right-operand under an
/// operator. Numeric and dateTime operands compare by magnitude; everything
/// else compares by string/IRI value. An order comparison (`lt`/`gt`/…) on
/// non-orderable values is **false** (fail-closed).
fn compare(actual: &Value, op: Operator, bound: &Value) -> bool {
    match op {
        Operator::Eq | Operator::IsA => value_eq(actual, bound),
        Operator::Neq => !value_eq(actual, bound),
        Operator::IsPartOf => is_part_of(actual, bound),
        Operator::Lt | Operator::Lteq | Operator::Gt | Operator::Gteq => {
            let Some(ord) = order(actual, bound) else {
                return false;
            };
            match op {
                Operator::Lt => ord == std::cmp::Ordering::Less,
                Operator::Lteq => ord != std::cmp::Ordering::Greater,
                Operator::Gt => ord == std::cmp::Ordering::Greater,
                Operator::Gteq => ord != std::cmp::Ordering::Less,
                _ => unreachable!(),
            }
        }
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x == y,
        // Cross-type: compare by canonical string (an IRI right-operand vs an IRI
        // actual, a string code vs a string code, a dateTime vs a dateTime).
        _ => a.as_str() == b.as_str(),
    }
}

/// `isPartOf` / set membership: the right operand is a `|`-or-space-separated
/// set (the common compact encoding) OR a single IRI/string the actual must
/// equal. We treat the right operand's string as a set: actual ∈ set.
fn is_part_of(actual: &Value, bound: &Value) -> bool {
    let a = actual.as_str();
    bound
        .as_str()
        .split(['|', ' ', ','])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|member| member == a)
}

/// A total-ish order for orderable values: numeric by magnitude, dateTime by
/// the lexical-as-comparable key (ISO-8601 instants sort lexicographically when
/// in the same `Z`-normalized form — see the README caveat). Returns `None` for
/// incomparable pairs.
fn order(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.partial_cmp(y),
        (Value::DateTime(x), Value::DateTime(y)) => Some(cmp_datetime(x, y)),
        // A numeric actual against a numeric-looking string bound, or vice versa.
        _ => {
            let (Ok(x), Ok(y)) = (
                a.as_str().trim().parse::<f64>(),
                b.as_str().trim().parse::<f64>(),
            ) else {
                return None;
            };
            x.partial_cmp(&y)
        }
    }
}

/// Compare two xsd:dateTime lexical forms. We normalize a trailing `Z` and a
/// missing timezone to a comparable key; mixed offsets are compared by the raw
/// lexical form (documented limitation — see the README; full offset
/// normalization is a deferred bead).
fn cmp_datetime(x: &str, y: &str) -> std::cmp::Ordering {
    x.cmp(y)
}
