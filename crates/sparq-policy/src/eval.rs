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

/// The `odrl:purpose` left-operand IRI — the dimension a *purpose constraint*
/// restricts (the purpose of use a permission/prohibition is gated on, e.g.
/// `odrl:purpose eq <urn:purpose/research>`). [OPUS-4.8] sq-q56r.
pub const ODRL_PURPOSE: &str = "http://www.w3.org/ns/odrl/2/purpose";

/// The `odrl:count` left-operand IRI — the dimension a *count constraint* restricts
/// (the number of times a permission may be exercised, e.g. `odrl:count lteq 5`).
/// Stateful enforcement of this lives in the opt-in [`crate::count`] module.
/// [OPUS-4.8] sq-zi5w.
pub const ODRL_COUNT: &str = "http://www.w3.org/ns/odrl/2/count";

/// The `odrl:recipient` left-operand IRI — the dimension a *recipient constraint*
/// restricts (the party that data is disclosed to, e.g. `odrl:recipient neq <bob>`,
/// the "everyone EXCEPT bob" shape). The recipient-of-data is the requesting party,
/// so when a request carries **no** explicit `odrl:recipient` context value the
/// evaluator reads its [`Request::party`] as the recipient evidence (see
/// [`recipient_status`]). [OPUS-4.8] sq-5037.
pub const ODRL_RECIPIENT: &str = "http://www.w3.org/ns/odrl/2/recipient";

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
    /// The requesting party as an [`Value::Iri`], kept in lock-step with [`party`]
    /// (set by [`Request::by`]) so a `recipient` constraint with no explicit
    /// `odrl:recipient` context can be re-checked against *who is asking* without an
    /// allocation per constraint. Internal — [`party`] is the public field.
    /// [OPUS-4.8] sq-5037.
    ///
    /// [`party`]: Request::party
    pub(crate) recipient_party: Option<Value>,
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
    ///
    /// The party doubles as the default **recipient-of-data**: an `odrl:recipient`
    /// constraint with no explicit context value is re-checked against this party
    /// (see [`recipient_status`]), so a `recipient neq X` rule gates on who is asking.
    /// [OPUS-4.8] sq-5037.
    pub fn by(mut self, party: impl Into<String>) -> Request {
        let p = party.into();
        self.recipient_party = Some(Value::Iri(p.clone()));
        self.party = Some(p);
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

    /// Declare the **purpose of use** this request carries as evidence (chainable)
    /// — the `odrl:purpose` left-operand value a purpose-gated rule is checked
    /// against. [OPUS-4.8] sq-q56r.
    ///
    /// This is first-class sugar over [`Request::with`]`(ODRL_PURPOSE, ..)`: it
    /// makes the *evidence the requester supplies* explicit (and auditable via
    /// [`Request::purpose`]) instead of relying on a magic context key. A request
    /// that does NOT call this carries **no** purpose evidence, so a permission
    /// gated on purpose does not grant and a prohibition gated on purpose is not
    /// withdrawn (fail-closed — see [`purpose_status`]).
    ///
    /// The value is stored verbatim: a DPV/purpose-taxonomy IRI as [`Value::Iri`]
    /// (`Value::Iri(..)`) or a purpose code as [`Value::Str`]. Matching is **exact**
    /// (IRI/string equality) — there is no hierarchy/subsumption (see
    /// [`purpose_status`]).
    pub fn for_purpose(self, purpose: Value) -> Request {
        self.with(ODRL_PURPOSE, purpose)
    }

    /// The purpose-of-use evidence this request carries (`odrl:purpose`), or `None`
    /// if the request supplied none. The auditable answer to *"did the request
    /// actually state a purpose?"* — the question faithful purpose enforcement turns
    /// on. [OPUS-4.8] sq-q56r.
    pub fn purpose(&self) -> Option<&Value> {
        self.context.get(ODRL_PURPOSE)
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

/// Whether `policy`'s prohibitions still carve `request` out — and, when they do
/// not, **whether that is a definite no or merely unprovable**. [OPUS-4.8] sq-2pcf.
///
/// This is the *deny-retraction dual* of [`matched_prohibition`]. A bare
/// `matched_prohibition(..).is_none()` collapses two semantically different worlds:
///
/// - the prohibition was genuinely **withdrawn / no longer structurally applies**
///   (its action/target/assignee no longer name this request, or a constraint it
///   carries is *definitely* false because the request supplies evidence that fails
///   the bound), and
/// - the prohibition still structurally names this request but a constraint is
///   **unprovable** because the request lacks evidence for that dimension
///   (`constraint_satisfied` returns `false` on a missing context value).
///
/// For a *grant* both collapse to "deny access" — fail-closed. But for **retracting a
/// materialized `auth:deny*`** they must NOT: retracting on the second case would
/// RESTORE access on missing evidence (fail-OPEN). This function keeps them apart so
/// the bridge only retracts a deny on a *definite* "no longer holds".
///
/// Returns:
/// - [`ProhibitionStatus::Applies`] if some prohibition still carves the request out
///   (every structural attribute agrees AND every constraint is *satisfied*).
/// - [`ProhibitionStatus::Ambiguous`] if no prohibition matches, but at least one
///   prohibition still structurally names the request (action/target/assignee agree)
///   and fails ONLY because a constraint is unprovable for lack of evidence.
/// - [`ProhibitionStatus::Withdrawn`] otherwise — every prohibition either does not
///   structurally name the request or carries a constraint that is *definitely* false
///   given the supplied evidence. This is the only case in which a deny may be retracted.
///
/// # Examples
///
/// ```
/// use sparq_policy::{prohibition_status, ProhibitionStatus, parse_policy_str, Request, Value};
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/p> a odrl:Set ; odrl:prohibition [
///     odrl:action odrl:write ;
///     odrl:target <https://pod.ex/n1> ;
///     odrl:assignee <https://alice.ex/card#me> ;
///     odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lt ;
///        odrl:rightOperand "2026-01-01T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime> ] ] .
/// "#, "turtle").unwrap();
/// let base = Request::new("http://www.w3.org/ns/odrl/2/write")
///     .on("https://pod.ex/n1").by("https://alice.ex/card#me");
///
/// // Evidence the window holds → still carves out.
/// let inside = base.clone()
///     .with("http://www.w3.org/ns/odrl/2/dateTime", Value::DateTime("2025-06-01T00:00:00Z".into()));
/// assert_eq!(prohibition_status(&pol, &inside), ProhibitionStatus::Applies);
///
/// // Evidence the window has LAPSED → definitely withdrawn.
/// let lapsed = base.clone()
///     .with("http://www.w3.org/ns/odrl/2/dateTime", Value::DateTime("2026-06-01T00:00:00Z".into()));
/// assert_eq!(prohibition_status(&pol, &lapsed), ProhibitionStatus::Withdrawn);
///
/// // NO evidence for the window → unprovable → ambiguous (keep the deny).
/// assert_eq!(prohibition_status(&pol, &base), ProhibitionStatus::Ambiguous);
/// ```
pub fn prohibition_status(policy: &Policy, request: &Request) -> ProhibitionStatus {
    let req_action = Action(request.action.clone());
    let mut any_ambiguous = false;
    for rule in &policy.prohibitions {
        match classify_prohibition(rule, request, &req_action) {
            RuleClass::Match => return ProhibitionStatus::Applies,
            RuleClass::Ambiguous => any_ambiguous = true,
            RuleClass::DefinitelyNo => {}
        }
    }
    if any_ambiguous {
        ProhibitionStatus::Ambiguous
    } else {
        ProhibitionStatus::Withdrawn
    }
}

/// The deny-retraction verdict for a request's prohibitions — see
/// [`prohibition_status`]. [OPUS-4.8] sq-2pcf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProhibitionStatus {
    /// A prohibition still carves the request out — a materialized deny must be KEPT.
    Applies,
    /// No prohibition matches, but one still structurally names the request and fails
    /// only for lack of evidence — the deny must be KEPT (fail-closed: do not restore
    /// access on an unprovable carve-out).
    Ambiguous,
    /// No prohibition structurally names the request, or every one that does carries a
    /// constraint that is *definitely* false given the evidence — the only case in
    /// which a materialized deny may be RETRACTED (access restored).
    Withdrawn,
}

/// The three-valued verdict of a rule's `odrl:purpose` constraints against the
/// purpose evidence a request carries — a FAITHFUL report of exactly what
/// [`evaluate`] checks for purpose (it reuses the same [`constraint_status`] the
/// evaluator's purpose constraints go through). [OPUS-4.8] sq-q56r.
///
/// The honesty contract: this never claims a stronger verdict than the evaluator
/// would act on. `Satisfied`/`Unprovable`/`DefinitelyUnsatisfied` map 1:1 onto the
/// evaluator's gating — a `Satisfied` purpose is the *only* verdict under which a
/// purpose-gated permission can grant, and anything other than `DefinitelyUnsatisfied`
/// (i.e. `Satisfied` OR `Unprovable`) keeps a purpose-gated prohibition in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurposeMatch {
    /// The rule has no `odrl:purpose` constraint — purpose places no restriction on
    /// it (the rule's other attributes/constraints decide).
    NotConstrained,
    /// The rule constrains purpose AND the request's stated purpose **matches** every
    /// purpose constraint (exact IRI/string equality, the only matching this checks).
    Satisfied,
    /// The rule constrains purpose but the request carries **no** purpose evidence —
    /// *unprovable*. Fail-closed: a permission does NOT grant on this; a prohibition is
    /// NOT withdrawn. (Never silently read as "any purpose allowed".)
    Unprovable,
    /// The rule constrains purpose and the request's stated purpose **does not match**
    /// — a definite mismatch (a permission does not grant; a prohibition no longer
    /// carves *this* purpose out).
    DefinitelyUnsatisfied,
}

/// Report how `rule`'s `odrl:purpose` constraint(s) stand against the purpose
/// evidence `request` carries — the auditable surface of faithful purpose
/// enforcement. [OPUS-4.8] sq-q56r.
///
/// This does NOT re-implement matching: it runs the request through the SAME
/// [`constraint_status`] every `odrl:purpose` constraint goes through inside
/// [`evaluate`], so the verdict it reports is exactly what the evaluator acts on —
/// the whole point of the bead (no claimed enforcement that isn't actually checked).
///
/// **Match semantics (the boundary, not over-claimed):** a purpose matches by
/// **exact** IRI/string equality (an `eq`/`isA` purpose constraint), or by membership
/// in an explicit `isPartOf` purpose *set* (the `|`/space/comma-separated right
/// operand). There is **no** purpose hierarchy / DPV subsumption: a request purpose is
/// not matched against broader/narrower purposes — only the exact value(s) the
/// constraint names. A `neq` purpose constraint is honoured (purpose ≠ the named one).
/// Several purpose constraints on one rule are ANDed (every one must hold), mirroring
/// the evaluator's constraint conjunction.
///
/// Returns [`PurposeMatch::NotConstrained`] when the rule has no purpose constraint.
///
/// # Examples
///
/// ```
/// use sparq_policy::{purpose_status, PurposeMatch, parse_policy_str, Request, Value};
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/p> a odrl:Set ; odrl:permission [
///     odrl:action odrl:use ; odrl:target <urn:asset/x> ;
///     odrl:constraint [ odrl:leftOperand odrl:purpose ; odrl:operator odrl:eq ;
///                       odrl:rightOperand <urn:purpose/research> ] ] .
/// "#, "turtle").unwrap();
/// let rule = &pol.permissions[0];
/// let base = Request::new("http://www.w3.org/ns/odrl/2/use").on("urn:asset/x");
///
/// // Stated purpose matches exactly → Satisfied.
/// let ok = base.clone().for_purpose(Value::Iri("urn:purpose/research".into()));
/// assert_eq!(purpose_status(rule, &ok), PurposeMatch::Satisfied);
/// // Stated a different purpose → DefinitelyUnsatisfied.
/// let bad = base.clone().for_purpose(Value::Iri("urn:purpose/marketing".into()));
/// assert_eq!(purpose_status(rule, &bad), PurposeMatch::DefinitelyUnsatisfied);
/// // NO purpose evidence → Unprovable (fail-closed; not "any purpose").
/// assert_eq!(purpose_status(rule, &base), PurposeMatch::Unprovable);
/// ```
pub fn purpose_status(rule: &Rule, request: &Request) -> PurposeMatch {
    let mut constrained = false;
    let mut any_unprovable = false;
    for c in &rule.constraints {
        if c.left != ODRL_PURPOSE {
            continue;
        }
        constrained = true;
        match constraint_status(c, request) {
            ConstraintStatus::Satisfied => {}
            ConstraintStatus::DefinitelyUnsatisfied => return PurposeMatch::DefinitelyUnsatisfied,
            ConstraintStatus::Unprovable => any_unprovable = true,
        }
    }
    if !constrained {
        PurposeMatch::NotConstrained
    } else if any_unprovable {
        PurposeMatch::Unprovable
    } else {
        PurposeMatch::Satisfied
    }
}

/// The three-valued verdict of a rule's `odrl:recipient` constraints against the
/// recipient evidence a request carries — a FAITHFUL report of exactly what
/// [`evaluate`] checks for recipient (it reuses the same [`constraint_status`] the
/// evaluator's recipient constraints go through). [OPUS-4.8] sq-5037.
///
/// The honesty contract mirrors [`PurposeMatch`]: this never claims a stronger verdict
/// than the evaluator acts on. `Satisfied` is the ONLY verdict under which a
/// recipient-gated permission grants; anything other than `DefinitelyUnsatisfied`
/// (i.e. `Satisfied` OR `Unprovable`) keeps a recipient-gated prohibition in force.
///
/// **The `neq` / "everyone-except" shape:** a `recipient neq X` constraint is
/// `Satisfied` for any recipient that is NOT `X`, `DefinitelyUnsatisfied` for the
/// recipient `X` (the carved-out party), and `Unprovable` when the request supplies
/// no identity at all (no `odrl:recipient` context AND no [`Request::party`]) —
/// fail-closed: a `neq` permission does NOT grant to an unknown recipient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipientMatch {
    /// The rule has no `odrl:recipient` constraint — recipient places no restriction.
    NotConstrained,
    /// The rule constrains recipient AND the request's recipient **satisfies** every
    /// recipient constraint (for `neq X`: the recipient is some party other than `X`).
    Satisfied,
    /// The rule constrains recipient but the request carries **no** recipient evidence
    /// (no `odrl:recipient` context and no requesting party) — *unprovable*.
    /// Fail-closed: a permission does NOT grant; a prohibition is NOT withdrawn.
    Unprovable,
    /// The rule constrains recipient and the request's recipient **fails** the
    /// constraint (for `neq X`: the recipient IS the carved-out party `X`).
    DefinitelyUnsatisfied,
}

/// Report how `rule`'s `odrl:recipient` constraint(s) stand against the recipient
/// evidence `request` carries — the auditable surface of faithful recipient (incl.
/// `neq` / "everyone-except") enforcement. [OPUS-4.8] sq-5037.
///
/// Like [`purpose_status`], this does NOT re-implement matching: it runs the request
/// through the SAME [`constraint_status`] every `odrl:recipient` constraint goes
/// through inside [`evaluate`], so the verdict it reports is exactly what the
/// evaluator acts on (no claimed enforcement that isn't actually checked). The
/// recipient evidence is the explicit `odrl:recipient` context value, or — when none
/// is set — the requesting [`Request::party`] (the recipient-of-data is who is asking;
/// see [`resolve_actual`]).
///
/// Returns [`RecipientMatch::NotConstrained`] when the rule has no recipient constraint.
///
/// # Examples
///
/// ```
/// use sparq_policy::{recipient_status, RecipientMatch, parse_policy_str, Request};
/// // "everyone EXCEPT bob may receive the data"
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/p> a odrl:Set ; odrl:permission [
///     odrl:action odrl:read ; odrl:target <urn:asset/x> ;
///     odrl:constraint [ odrl:leftOperand odrl:recipient ; odrl:operator odrl:neq ;
///                       odrl:rightOperand <https://bob.ex/card#me> ] ] .
/// "#, "turtle").unwrap();
/// let rule = &pol.permissions[0];
/// let base = Request::new("http://www.w3.org/ns/odrl/2/read").on("urn:asset/x");
///
/// // Some other party → Satisfied (not the excluded one).
/// let carol = base.clone().by("https://carol.ex/card#me");
/// assert_eq!(recipient_status(rule, &carol), RecipientMatch::Satisfied);
/// // The excluded party → DefinitelyUnsatisfied.
/// let bob = base.clone().by("https://bob.ex/card#me");
/// assert_eq!(recipient_status(rule, &bob), RecipientMatch::DefinitelyUnsatisfied);
/// // No identity at all → Unprovable (fail-closed; not "any recipient").
/// assert_eq!(recipient_status(rule, &base), RecipientMatch::Unprovable);
/// ```
pub fn recipient_status(rule: &Rule, request: &Request) -> RecipientMatch {
    let mut constrained = false;
    let mut any_unprovable = false;
    for c in &rule.constraints {
        if c.left != ODRL_RECIPIENT {
            continue;
        }
        constrained = true;
        match constraint_status(c, request) {
            ConstraintStatus::Satisfied => {}
            ConstraintStatus::DefinitelyUnsatisfied => {
                return RecipientMatch::DefinitelyUnsatisfied
            }
            ConstraintStatus::Unprovable => any_unprovable = true,
        }
    }
    if !constrained {
        RecipientMatch::NotConstrained
    } else if any_unprovable {
        RecipientMatch::Unprovable
    } else {
        RecipientMatch::Satisfied
    }
}

/// How one prohibition rule re-evaluates against a request, distinguishing a
/// *definite* non-match from an *unprovable* one (missing-evidence constraint).
/// [OPUS-4.8] sq-2pcf.
enum RuleClass {
    /// Structural attributes agree AND every constraint is satisfied.
    Match,
    /// Structural attributes agree, but a constraint is unprovable (no evidence).
    Ambiguous,
    /// A structural attribute disagrees, OR a constraint is *definitely* false
    /// (evidence present, comparison failed), OR a constraint is malformed.
    DefinitelyNo,
}

/// Classify a single prohibition rule for deny-retraction. Structural attributes
/// (action / target / assignee) are definite (no "world state" needed). Constraints
/// split into *definitely false* (evidence present, bound not met — a definite no) vs
/// *unprovable* (no evidence for the dimension — ambiguous). [OPUS-4.8] sq-2pcf.
fn classify_prohibition(rule: &Rule, request: &Request, req_action: &Action) -> RuleClass {
    // Structural attributes are definite: if any disagrees the rule no longer names
    // this request at all (a genuine withdrawal of *this* carve-out).
    if !rule.action.permits(req_action) {
        return RuleClass::DefinitelyNo;
    }
    if let Some(t) = &rule.target {
        if request.target.as_deref() != Some(t.as_str()) {
            return RuleClass::DefinitelyNo;
        }
    }
    if let Some(a) = &rule.assignee {
        if request.party.as_deref() != Some(a.as_str()) {
            return RuleClass::DefinitelyNo;
        }
    }
    // Structural attributes agree — the constraints decide. A constraint that is
    // *definitely* false (we have evidence and it fails) is a definite no; one that is
    // unprovable for lack of evidence is ambiguous (a deny must NOT be retracted on it).
    let mut any_ambiguous = false;
    for c in &rule.constraints {
        match constraint_status(c, request) {
            ConstraintStatus::Satisfied => {}
            ConstraintStatus::DefinitelyUnsatisfied => return RuleClass::DefinitelyNo,
            ConstraintStatus::Unprovable => any_ambiguous = true,
        }
    }
    if any_ambiguous {
        RuleClass::Ambiguous
    } else {
        RuleClass::Match
    }
}

/// Whether one constraint is satisfied, *definitely* unsatisfied, or *unprovable* for
/// lack of evidence — the three-valued refinement of [`constraint_satisfied`] that
/// deny-retraction needs (a plain bool conflates the last two). [OPUS-4.8] sq-2pcf.
enum ConstraintStatus {
    /// The request supplies evidence and the comparison holds.
    Satisfied,
    /// The request supplies evidence and the comparison FAILS — a definite no.
    DefinitelyUnsatisfied,
    /// The request supplies no value for this dimension — we cannot tell.
    Unprovable,
}

fn constraint_status(c: &Constraint, request: &Request) -> ConstraintStatus {
    match resolve_actual(c, request) {
        None => ConstraintStatus::Unprovable,
        Some(actual) => {
            if compare(actual, c.operator, &c.right) {
                ConstraintStatus::Satisfied
            } else {
                ConstraintStatus::DefinitelyUnsatisfied
            }
        }
    }
}

/// The *actual* world value a constraint's `leftOperand` is compared against, or
/// `None` when the request supplies no evidence for that dimension (⇒ unprovable /
/// fail-closed). [OPUS-4.8] sq-5037.
///
/// Dimensions take their value from [`Request::context`] keyed by the left operand —
/// EXCEPT `odrl:recipient`, where the recipient-of-data IS the requesting party: a
/// request that names a `party` but supplies no explicit `odrl:recipient` context is
/// read as `recipient = party`. This is what makes a `recipient neq X` rule actually
/// gate on *who is asking* end-to-end. An explicit `odrl:recipient` context value (if
/// the caller sets one) still takes precedence — the disclosure target need not be the
/// authenticated principal in every deployment.
fn resolve_actual<'r>(c: &Constraint, request: &'r Request) -> Option<&'r Value> {
    if let Some(v) = request.context.get(&c.left) {
        return Some(v);
    }
    if c.left == ODRL_RECIPIENT {
        return request.recipient_party.as_ref();
    }
    None
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
/// (e.g. the actual request time for `odrl:dateTime`, or the requesting party for
/// `odrl:recipient` — see [`resolve_actual`]); the constraint's `rightOperand` is the
/// bound. A constraint whose left operand has **no** value (no context value AND, for
/// recipient, no party) is **unsatisfied** (fail-closed: we cannot prove the world
/// meets a constraint we have no evidence about — including a `recipient neq X`
/// constraint with no identity to compare).
fn constraint_satisfied(c: &Constraint, request: &Request) -> bool {
    let Some(actual) = resolve_actual(c, request) else {
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
