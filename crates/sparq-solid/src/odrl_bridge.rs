//! [OPUS-4.8] sq-h3uk — bridge the `sparq-policy` ODRL evaluator into the
//! `<urn:sparq:auth>` AUTH_GRAPH so the existing graph-level WAC/ACP enforcement
//! applies a matched ODRL [`Permission`](sparq_policy::Rule).
//!
//! # What this is (and is NOT)
//!
//! This is the **single-node** bridge of epic sq-3183 — a research-track surface,
//! NOT a production cutover and NOT the federated/ZK-disclosure path (that one is
//! gated on ZK soundness remediation). It composes two opt-in crates:
//!
//! - `sparq-policy` answers a usage-control question — *may this party USE this
//!   asset, for purpose P, until time T, with obligation O discharged?* — and
//!   returns a fail-closed [`Decision`](sparq_policy::Decision).
//! - `sparq-solid` enforces graph-level access: a principal × [`Mode`] → graph set
//!   is read out of the `<urn:sparq:auth>` view (see [`crate::AuthIndex`]).
//!
//! The bridge **materializes** a definite ODRL Permit as the very same
//! `principal auth:<mode> graphName` triples the existing enforcement already
//! understands, appending them into the AUTH_GRAPH. Net effect: an ODRL
//! `Permission` (action + constraints satisfied + duties discharged) becomes a
//! concrete WAC/ACP grant honoured by the current enforcement — **no new
//! enforcement engine**, the existing one is reused unchanged.
//!
//! # Fail-closed
//!
//! A grant is materialized **only** when [`sparq_policy::evaluate`] returns a
//! definite Permit (`decision.allow == true`) AND the requested ODRL action maps to
//! a concrete WAC/ACP [`Mode`] AND the request names a concrete WebID party + target
//! graph IRI. A Deny, an ambiguous evaluation, an unmapped action, or a missing
//! party/target materializes **nothing** — access is never widened on ambiguity.
//!
//! # Action → Mode mapping
//!
//! The ODRL request's `action` IRI is mapped to a WAC/ACP [`Mode`] by
//! [`action_to_mode`]. The mapping is deliberately conservative (a permit only ever
//! grants the *narrowest* mode the action denotes):
//!
//! | ODRL action (`odrl:`)                                   | WAC/ACP [`Mode`] |
//! |---------------------------------------------------------|------------------|
//! | `read`, `display`, `present`, `print`, `play`           | [`Mode::Read`]   |
//! | `append`                                                | [`Mode::Append`] |
//! | `modify`, `delete`, `write`                             | [`Mode::Write`]  |
//! | anything else (incl. the `odrl:use` umbrella)           | **unmapped → no grant** |
//!
//! The `odrl:use` umbrella is intentionally **not** mapped to a write/control mode:
//! `use` subsumes its sub-actions in the ODRL hierarchy (`read`/`write`/`modify`/… —
//! everything except the disjoint `transfer` ownership subtree, [OPUS-4.8] sq-euhr3),
//! so materializing it as a single WAC mode would have to pick the widest, violating
//! fail-closed. A caller that wants `use → Read` should request `odrl:read` explicitly
//! (a `use` permission in the policy still *grants* a `read` request — `odrl:use`
//! permits its sub-actions in the evaluator — so the bridge maps the **request**
//! action, which is concrete).
//!
//! # Prohibitions → `auth:deny<Mode>` (deny-overrides) — [OPUS-4.8] sq-w693
//!
//! A matched ODRL **Prohibition** is the dual of a Permit: it carves an action out
//! of access. [`materialize_prohibition`] materializes it as the explicit
//! `principal auth:deny<Mode> target` triple the enforcement already understands —
//! `auth:denyRead`/`denyWrite`/`denyAppend`/`denyControl`, via the SAME
//! [`action_to_mode`] mapping (the request's concrete action → the narrowest denied
//! mode). The existing session layer ([`crate::AuthIndex::accessible`]) computes
//! `∪ allow ∖ ∪ deny`, so a materialized deny **takes precedence over any allow
//! grant** for the same principal+target+mode — *deny-overrides*, the same conflict
//! resolution the ODRL evaluator applies (a matching prohibition overrides any
//! permission). No new enforcement engine: the deny path already existed
//! ([`Mode::from_pred`](crate::Mode) parses `deny*`); the bridge only emits the triples.
//!
//! A prohibition materializes a deny **only** when [`sparq_policy::matched_prohibition`]
//! reports it carves THIS request out (action permits + target/assignee agree +
//! constraints satisfied) AND the request action [`action_to_mode`]-maps AND the
//! request names a concrete party (WebID) + target graph IRI. An un-matched / unmapped
//! / partyless / targetless prohibition materializes **nothing** — fail-closed; a deny
//! is never widened on ambiguity (and certainly never silently dropped to widen access).
//!
//! [`materialize_policy`] composes the two: it materializes every applicable Permit
//! grant AND every matched-Prohibition deny for the request, so a policy carrying both
//! a permission and a prohibition on the same principal+target+mode emits both triples,
//! and the deny **wins** at enforcement time.
//!
//! # Deny RETRACTION on prohibition withdrawal — [OPUS-4.8] sq-2pcf
//!
//! The sq-dpk4 [`BridgeLedger`] replays each tracked materialization on
//! [`refresh`](BridgeLedger::refresh), dropping the ones that no longer hold. For an
//! *allow* grant the rule is simple: re-evaluate, and a non-Permit (withdrawn / lapsed
//! / now-Denied / **ambiguous**) emits nothing → the grant is dropped → access is GONE.
//! Dropping on ambiguity is the right (fail-closed) call for an allow.
//!
//! A materialized **deny** is the mirror image, and so its retraction rule must be the
//! mirror image too. A deny should be RETRACTED — restoring access — only when the
//! underlying ODRL Prohibition is genuinely **withdrawn or lapses**. If we reused the
//! allow rule (drop the deny whenever the prohibition no longer matches), an *ambiguous*
//! re-evaluation — a prohibition that still structurally names the request but carries a
//! constraint the refresh request supplies no evidence for — would silently retract the
//! deny and **restore access on missing evidence: fail-OPEN**. That is exactly the trap
//! this bead closes.
//!
//! So deny retraction consults [`sparq_policy::prohibition_status`], a three-valued
//! refinement of [`matched_prohibition`]:
//!
//! | [`ProhibitionStatus`]                       | meaning                                                     | deny action        |
//! |---------------------------------------------|-------------------------------------------------------------|--------------------|
//! | [`Applies`](ProhibitionStatus::Applies)     | a prohibition still carves the request out                  | **keep** (re-emit) |
//! | [`Ambiguous`](ProhibitionStatus::Ambiguous) | still structurally names it, but a constraint is unprovable | **keep** (re-emit) |
//! | [`Withdrawn`](ProhibitionStatus::Withdrawn) | no prohibition names it, or every one is *definitely* false | **retract** (drop) |
//!
//! "Definitely false" means the refresh request DID supply evidence for the dimension
//! and the comparison failed (e.g. a `dateTime < 2026-01-01` window with an actual time
//! of `2026-06-01` — the window has provably lapsed). Only then is the carve-out known
//! to be gone. A retracted deny composes with deny-overrides as expected: it may
//! re-expose an allow grant for the same principal+target+mode — correct, *because the
//! prohibition is genuinely gone*. Static (non-bridged) `auth:deny*` rules are never in
//! the ledger and so are never re-evaluated or retracted by this path.

use crate::authindex::{Mode, AUTHENTICATED, PUBLIC};
use crate::{AUTH_BRIDGED_GRAPH, AUTH_GRAPH, AUTH_NS, SOLIDX_NS};
use oxrdf::{Literal, NamedNode, Term};
use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_policy::{
    conflict_admissibility, evaluate, matched_prohibition, prohibition_status, Constraint,
    ConstraintNode, LogicalOperator, Operator, Policy, ProhibitionStatus, Request, Rule, Value,
};
use std::collections::{BTreeMap, BTreeSet};

/// The ODRL namespace prefix (`odrl:`), re-exported for caller convenience.
pub const ODRL_NS: &str = sparq_policy::ODRL_NS;

/// Map an ODRL **request action** IRI to the WAC/ACP [`Mode`] it materializes as,
/// or `None` if the action has no faithful single-mode mapping (fail-closed: an
/// unmapped action never materializes a grant).
///
/// See the [module docs](self#action--mode-mapping) for the full table and the
/// rationale for leaving the `odrl:use` umbrella unmapped.
///
/// # Examples
///
/// ```
/// # use sparq_solid::odrl_bridge::action_to_mode;
/// # use sparq_solid::Mode;
/// assert_eq!(action_to_mode("http://www.w3.org/ns/odrl/2/read"), Some(Mode::Read));
/// assert_eq!(action_to_mode("http://www.w3.org/ns/odrl/2/modify"), Some(Mode::Write));
/// assert_eq!(action_to_mode("http://www.w3.org/ns/odrl/2/append"), Some(Mode::Append));
/// // the umbrella action and unknown actions are unmapped (no grant)
/// assert_eq!(action_to_mode("http://www.w3.org/ns/odrl/2/use"), None);
/// ```
pub fn action_to_mode(action_iri: &str) -> Option<Mode> {
    let local = action_iri.strip_prefix(ODRL_NS)?;
    Some(match local {
        // Read-family: observe/render the resource's content.
        "read" | "display" | "present" | "print" | "play" => Mode::Read,
        // Append: add without removing (the strictly weaker write).
        "append" => Mode::Append,
        // Write-family: mutate/replace/remove the resource's content.
        "modify" | "delete" | "write" => Mode::Write,
        // `use` (umbrella) and everything else: no faithful single-mode mapping.
        _ => return None,
    })
}

/// The `auth:` view predicate a [`Mode`] grant is materialized under — the SAME
/// predicate the WAC/ACP rules emit and [`crate::AuthIndex`] reads
/// (`auth:read|write|append|control`).
fn mode_predicate(mode: Mode) -> &'static str {
    match mode {
        Mode::Read => "read",
        Mode::Write => "write",
        Mode::Append => "append",
        Mode::Control => "control",
    }
}

/// The `auth:deny<Mode>` view predicate a [`Mode`] DENY is materialized under — the
/// SAME predicate [`crate::AuthIndex`] parses into its deny map
/// (`auth:denyRead|denyWrite|denyAppend|denyControl`), so a materialized deny is
/// subtracted from the allow set by the existing `∪ allow ∖ ∪ deny` enforcement.
/// [OPUS-4.8] sq-w693.
fn deny_predicate(mode: Mode) -> &'static str {
    match mode {
        Mode::Read => "denyRead",
        Mode::Write => "denyWrite",
        Mode::Append => "denyAppend",
        Mode::Control => "denyControl",
    }
}

/// What the bridge decided and (on a Permit / matched Prohibition) materialized.
///
/// [`materialize_permission`] sets the `granted` / `grant_triple` fields;
/// [`materialize_prohibition`] sets the `prohibited` / `deny_triple` fields
/// ([OPUS-4.8] sq-w693); [`materialize_policy`] may set both at once (a policy with a
/// permission AND a prohibition for the request — the deny wins at enforcement time).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeOutcome {
    /// Whether a definite Permit produced an allow grant.
    /// `false` ⇒ no allow triple was written to the AUTH_GRAPH (fail-closed).
    pub granted: bool,
    /// Whether a matched Prohibition produced a DENY. `false` ⇒ no `auth:deny*`
    /// triple was written (fail-closed). [OPUS-4.8] sq-w693.
    pub prohibited: bool,
    /// On a grant: the WAC/ACP mode the matched ODRL action mapped to. On a deny:
    /// the mode the prohibited action mapped to (`deny_triple`'s mode).
    pub mode: Option<Mode>,
    /// On a grant: the materialized `principal auth:<mode> graph` triple, as
    /// `(principal_iri, mode_predicate_iri, graph_iri)`, for audit/justification.
    pub grant_triple: Option<(String, String, String)>,
    /// On a deny: the materialized `principal auth:deny<Mode> graph` triple, as
    /// `(principal_iri, deny_predicate_iri, graph_iri)`. [OPUS-4.8] sq-w693.
    pub deny_triple: Option<(String, String, String)>,
    /// Whether the whole policy was **REFUSED** as unimplementable — the bridge cannot
    /// faithfully honour its declared `odrl:conflict` strategy, so it materialised
    /// NOTHING (no grant, no deny). Distinct from a plain deny (`granted == false`): a
    /// refusal is a loud, fail-closed rejection of the policy itself, not a rule that
    /// simply did not match. The reason is in [`reasons`](BridgeOutcome::reasons). See
    /// [`sparq_policy::conflict_admissibility`]. [OPUS-4.8] sq-ihqbl.
    pub refused: bool,
    /// Human-readable reason a grant/deny was NOT materialized (the ODRL decision's
    /// caveats, an unmapped action, a missing party/target, or a policy-level refusal).
    /// Empty on success.
    pub reasons: Vec<String>,
    /// On a STATEFUL `odrl:count` grant ([OPUS-4.8] sq-58mh, via
    /// [`crate::PodStore::materialize_odrl_permission_counted`]): the new consumed count
    /// (`1..=limit`) after this exercise atomically consumed one unit of budget. `None`
    /// for an uncounted permission, any non-counted bridge path, or a deny.
    #[cfg_attr(not(feature = "count-enforcement"), allow(rustdoc::broken_intra_doc_links))]
    pub consumed: Option<u64>,
    /// The exact auth-view triples this call materialized (allow grant, deny, and/or
    /// the conditional-grant head triples), for the bridge ledger to track so a later
    /// refresh can retract precisely these. Empty when nothing was materialized.
    /// [OPUS-4.8] sq-dpk4.
    pub(crate) emitted: Vec<[Term; 3]>,
}

impl BridgeOutcome {
    fn denied(reasons: Vec<String>) -> BridgeOutcome {
        BridgeOutcome { reasons, ..BridgeOutcome::default() }
    }

    /// A loud, fail-closed **refusal**: the policy's `odrl:conflict` strategy is one the
    /// bridge cannot faithfully honour, so nothing is materialised. [OPUS-4.8] sq-ihqbl.
    fn refused(reason: String) -> BridgeOutcome {
        BridgeOutcome { refused: true, reasons: vec![reason], ..BridgeOutcome::default() }
    }
}

/// Refuse (fail-closed) to materialise `policy` when its declared `odrl:conflict`
/// conflict-resolution strategy is one the bridge cannot faithfully honour — the guard
/// every materialise entry point runs FIRST. Returns `Some(refusal outcome)` to
/// short-circuit (materialising nothing), or `None` when the strategy is admissible
/// (deny-overrides — the one strategy the bridge implements). [OPUS-4.8] sq-ihqbl.
///
/// Silently coercing an unimplementable strategy (e.g. `odrl:perm`) into the bridge's
/// deny-overrides would mis-apply the policy author's intent — an authorization-
/// correctness hazard — so the bridge refuses loudly instead. See
/// [`sparq_policy::conflict_admissibility`] for the exact admissibility rules.
fn refuse_unimplementable_conflict(policy: &Policy) -> Option<BridgeOutcome> {
    conflict_admissibility(policy)
        .err()
        .map(|reason| BridgeOutcome::refused(format!("REFUSED (odrl:conflict): {}", reason)))
}

/// Evaluate `policy` against `request` and, **iff** the result is a definite Permit
/// for a mappable action with a concrete party + target, materialize the equivalent
/// `principal auth:<mode> graph` grant into the dataset's `<urn:sparq:auth>` view —
/// the triple the existing WAC/ACP enforcement ([`crate::AuthIndex`],
/// [`crate::PodStore::accessible`]) already honours.
///
/// The grant is **appended** to the current auth view: any WAC/ACP grants already
/// materialized there are preserved (this widens nothing they denied — it only adds
/// an allow triple, which the `∪ allow ∖ ∪ deny` semantics still subtract any
/// matching deny from). If no `<urn:sparq:auth>` graph exists yet, one is created
/// holding just the bridged grant.
///
/// # Fail-closed
///
/// Returns a [`BridgeOutcome`] with `granted == false` and writes **nothing** when:
/// the ODRL [`Decision`](sparq_policy::Decision) is a Deny (or any caveat blocked the
/// permission); the requested action does not [`action_to_mode`]-map; or the request
/// omits the party (assignee/WebID) or the target graph IRI. The grant principal is
/// the request's `party` (a concrete WebID) — an anonymous/partyless request never
/// materializes a grant, since a partyless grant would widen access to everyone.
///
/// # Note (PodStore observation)
///
/// Like [`crate::materialize_wac`], a direct call on a [`crate::PodStore`]'s `graph`
/// field does NOT rebuild its session index/cache. Use
/// [`crate::PodStore::materialize_odrl_permission`], which reindexes afterward.
///
/// # Examples
///
/// ```
/// use oxrdf::Term;
/// use sparq_solid::odrl_bridge::materialize_permission;
/// use sparq_policy::{parse_policy_str, Request};
///
/// // alice MAY read the target graph (a bare matching permission).
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/1> a odrl:Set ; odrl:permission [
///     odrl:action odrl:read ;
///     odrl:target <https://pod.ex/notes/n1> ;
///     odrl:assignee <https://alice.ex/card#me> ] .
/// "#, "turtle")?;
///
/// let mut graph = sparq_core::Graph::load_dataset(
///     "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#t> \"x\" <https://pod.ex/notes/n1> .",
///     "nquads")?;
/// let req = Request::new("http://www.w3.org/ns/odrl/2/read")
///     .on("https://pod.ex/notes/n1")
///     .by("https://alice.ex/card#me");
///
/// let out = materialize_permission(&mut graph, &pol, &req);
/// assert!(out.granted);
/// // a concrete WAC/ACP grant now lives in the auth view
/// assert!(graph.named.iter().any(|(name, _)| matches!(
///     name, Term::NamedNode(n) if n.as_str() == sparq_solid::AUTH_GRAPH)));
/// # Ok::<(), String>(())
/// ```
pub fn materialize_permission(
    graph: &mut Graph,
    policy: &Policy,
    request: &Request,
) -> BridgeOutcome {
    // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy BEFORE evaluating —
    //    a policy the bridge cannot faithfully honour materialises nothing. [OPUS-4.8] sq-ihqbl.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return refusal;
    }
    // 1. ODRL evaluation — the single source of the allow/deny decision.
    let decision = evaluate(policy, request);
    if !decision.allow {
        // Deny / ambiguous → materialize NOTHING (fail-closed).
        return BridgeOutcome::denied(decision.unmet_constraints);
    }

    // 2. Action → Mode. An unmapped action (incl. the `use` umbrella) → no grant.
    let Some(mode) = action_to_mode(&request.action) else {
        return BridgeOutcome::denied(vec![format!(
            "ODRL action <{}> has no WAC/ACP mode mapping; no grant materialized",
            request.action
        )]);
    };

    // 3. Concrete party (WebID principal) + target graph required — a partyless or
    //    targetless grant would widen access (fail-closed).
    let Some(party) = request.party.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL Permit has no concrete party (assignee/WebID); no grant materialized".to_owned(),
        ]);
    };
    let Some(target) = request.target.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL Permit has no concrete target graph IRI; no grant materialized".to_owned(),
        ]);
    };

    // 4. Materialize `party auth:<mode> target` into the auth view.
    let pred = format!("{AUTH_NS}{}", mode_predicate(mode));
    let triple = append_grant(graph, party, &pred, target);

    BridgeOutcome {
        granted: true,
        mode: Some(mode),
        grant_triple: Some((party.to_owned(), pred, target.to_owned())),
        emitted: vec![triple],
        ..BridgeOutcome::default()
    }
}

/// Evaluate `policy`'s **prohibitions** against `request` and, **iff** a prohibition
/// matches (carves the request out — action permits + target/assignee agree +
/// constraints satisfied) for a mappable action with a concrete party + target,
/// materialize the equivalent `principal auth:deny<Mode> graph` DENY triple into the
/// dataset's `<urn:sparq:auth>` view. [OPUS-4.8] sq-w693.
///
/// The deny is **appended** to the current auth view (any grants/denies already there
/// are preserved). Because the session layer computes `∪ allow ∖ ∪ deny`
/// ([`crate::AuthIndex::accessible`]), this materialized deny **takes precedence over
/// any allow grant** for the same principal+target+mode — *deny-overrides*. The deny
/// path is honoured by the EXISTING enforcement unchanged ([`crate::Mode`]'s
/// `from_pred` already parses `auth:deny*`); this function only emits the triple.
///
/// Whether a prohibition carves THIS request out is decided by
/// [`sparq_policy::matched_prohibition`] — the same match test the ODRL evaluator
/// applies in its conflict step — NOT by `evaluate(...).allow == false`, which would
/// conflate a carve-out with a plain no-matching-permission deny.
///
/// # Fail-closed
///
/// Returns a [`BridgeOutcome`] with `prohibited == false` and writes **nothing** when:
/// no prohibition matches the request; the requested action does not [`action_to_mode`]
/// -map; or the request omits the party (assignee/WebID) or the target graph IRI.
/// A deny is never widened on ambiguity, and an unmappable carve-out is reported (not
/// silently dropped — dropping a deny would widen access).
///
/// # Examples
///
/// ```
/// use oxrdf::Term;
/// use sparq_solid::odrl_bridge::materialize_prohibition;
/// use sparq_solid::Mode;
/// use sparq_policy::{parse_policy_str, Request};
///
/// // alice is PROHIBITED from writing the target graph.
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/1> a odrl:Set ; odrl:prohibition [
///     odrl:action odrl:modify ;
///     odrl:target <https://pod.ex/notes/n1> ;
///     odrl:assignee <https://alice.ex/card#me> ] .
/// "#, "turtle")?;
///
/// let mut graph = sparq_core::Graph::load_dataset(
///     "<https://pod.ex/notes/n1#it> <https://ex.dev/ns#t> \"x\" <https://pod.ex/notes/n1> .",
///     "nquads")?;
/// let req = Request::new("http://www.w3.org/ns/odrl/2/modify")
///     .on("https://pod.ex/notes/n1")
///     .by("https://alice.ex/card#me");
///
/// let out = materialize_prohibition(&mut graph, &pol, &req);
/// assert!(out.prohibited);
/// assert_eq!(out.mode, Some(Mode::Write));
/// // an explicit auth:denyWrite triple now lives in the auth view
/// assert!(graph.named.iter().any(|(name, _)| matches!(
///     name, Term::NamedNode(n) if n.as_str() == sparq_solid::AUTH_GRAPH)));
/// # Ok::<(), String>(())
/// ```
pub fn materialize_prohibition(
    graph: &mut Graph,
    policy: &Policy,
    request: &Request,
) -> BridgeOutcome {
    // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy first — under an
    //    unimplementable strategy we trust NONE of the policy's rules, not even a deny
    //    (e.g. `odrl:perm` means the PERMISSION should win). [OPUS-4.8] sq-ihqbl.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return refusal;
    }
    // 1. Does a prohibition CARVE THIS REQUEST OUT? (Same match test the evaluator's
    //    conflict step uses — not `!decision.allow`, which over-fires on a plain
    //    no-permission deny.)
    if matched_prohibition(policy, request).is_none() {
        return BridgeOutcome::denied(vec![
            "no ODRL prohibition matches the request; no deny materialized".to_owned(),
        ]);
    }

    // 2. Action → Mode. An unmapped action (incl. the `use` umbrella) → no deny.
    //    Fail-closed: we do NOT widen the deny to a default mode, but we also must
    //    not silently drop it — the carve-out is reported in `reasons`.
    let Some(mode) = action_to_mode(&request.action) else {
        return BridgeOutcome::denied(vec![format!(
            "ODRL prohibition matched but action <{}> has no WAC/ACP mode mapping; \
             no deny materialized",
            request.action
        )]);
    };

    // 3. Concrete party (WebID) + target graph required (fail-closed): a partyless
    //    deny would be meaningless / a targetless deny ambiguous.
    let Some(party) = request.party.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL prohibition has no concrete party (assignee/WebID); no deny materialized"
                .to_owned(),
        ]);
    };
    let Some(target) = request.target.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL prohibition has no concrete target graph IRI; no deny materialized".to_owned(),
        ]);
    };

    // 4. Materialize `party auth:deny<Mode> target` into the auth view.
    let pred = format!("{AUTH_NS}{}", deny_predicate(mode));
    let triple = append_grant(graph, party, &pred, target);

    BridgeOutcome {
        prohibited: true,
        mode: Some(mode),
        deny_triple: Some((party.to_owned(), pred, target.to_owned())),
        emitted: vec![triple],
        ..BridgeOutcome::default()
    }
}

/// Materialize **both** sides of `policy` for `request`: the Permit allow grant (via
/// [`materialize_permission`]) AND the matched-Prohibition deny (via
/// [`materialize_prohibition`]), composing them into one [`BridgeOutcome`].
/// [OPUS-4.8] sq-w693.
///
/// A policy carrying both a permission and a prohibition for the same
/// principal+target+mode materializes **both** triples. The deny **wins** at
/// enforcement time — the session layer subtracts `∪ deny` from `∪ allow`
/// ([`crate::AuthIndex::accessible`]), so *deny-overrides* falls out of the existing
/// enforcement without any conflict logic here. Each side is independently
/// fail-closed: a side that does not produce a definite, mappable, concrete result
/// materializes nothing for that side.
///
/// The returned outcome carries `granted`/`grant_triple` from the Permit side and
/// `prohibited`/`deny_triple` from the Prohibition side; `mode` reflects the deny
/// mode when a deny was materialized (the operative decision under deny-overrides),
/// else the grant mode; `reasons` aggregates the caveats of whichever side(s) did not
/// materialize.
pub fn materialize_policy(graph: &mut Graph, policy: &Policy, request: &Request) -> BridgeOutcome {
    // Fail-closed FIRST on an unimplementable odrl:conflict strategy — materialise
    // nothing (neither side), rather than silently apply deny-overrides. [OPUS-4.8] sq-ihqbl.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return refusal;
    }
    let allow = materialize_permission(graph, policy, request);
    let deny = materialize_prohibition(graph, policy, request);

    let mut reasons = allow.reasons;
    reasons.extend(deny.reasons);
    let mut emitted = allow.emitted;
    emitted.extend(deny.emitted);
    BridgeOutcome {
        granted: allow.granted,
        prohibited: deny.prohibited,
        // A refusal on either side (defence-in-depth; the top gate already short-circuits).
        refused: allow.refused || deny.refused,
        // Under deny-overrides the deny is the operative decision when present.
        mode: deny.mode.or(allow.mode),
        grant_triple: allow.grant_triple,
        deny_triple: deny.deny_triple,
        reasons,
        // The policy path is not the stateful-count path; no unit is consumed here.
        consumed: None,
        emitted,
    }
}

/// Append a single `subject predicate object` triple to the `<urn:sparq:auth>`
/// named graph, preserving the triples already there (the WAC/ACP grants), and
/// **mirror it into the bridged-provenance graph** ([`AUTH_BRIDGED_GRAPH`]) so the
/// triple is structurally marked as bridged (vs static) — see [`mirror_bridged`].
/// Returns the emitted triple so the caller can record it in its bridge ledger
/// ([OPUS-4.8] sq-dpk4). Idempotent: an identical grant is not duplicated.
fn append_grant(graph: &mut Graph, subject: &str, predicate: &str, object: &str) -> [Term; 3] {
    let s = Term::NamedNode(NamedNode::new_unchecked(subject));
    let p = Term::NamedNode(NamedNode::new_unchecked(predicate));
    let o = Term::NamedNode(NamedNode::new_unchecked(object));
    let triple = [s, p, o];
    append_bridged_triples(graph, std::slice::from_ref(&triple));
    triple
}

/// Append `new_triples` to BOTH the `<urn:sparq:auth>` enforcement view and the
/// `<urn:sparq:auth-bridged>` provenance graph, preserving existing triples in each
/// and skipping duplicates (idempotent). Mirroring into the provenance graph is what
/// lets a later refresh/static-re-materialization tell bridged triples apart from
/// static WAC/ACP grants without ever inspecting predicate shape. [OPUS-4.8] sq-dpk4.
fn append_bridged_triples(graph: &mut Graph, new_triples: &[[Term; 3]]) {
    extend_named_graph(graph, AUTH_GRAPH, new_triples);
    extend_named_graph(graph, AUTH_BRIDGED_GRAPH, new_triples);
}

/// Re-intern `name`'s existing triples plus `additions` (deduplicated) into a fresh
/// sub-graph dictionary and swap it in (matches `install_auth_view`'s rebuild shape).
fn extend_named_graph(graph: &mut Graph, name: &str, additions: &[[Term; 3]]) {
    let g_name = Term::NamedNode(NamedNode::new_unchecked(name));
    let mut terms: Vec<[Term; 3]> = match graph.named.iter().find(|(n, _)| *n == g_name) {
        Some((_, sub)) => crate::loader::graph_triples(sub),
        None => Vec::new(),
    };
    for t in additions {
        if !terms.contains(t) {
            terms.push(t.clone());
        }
    }
    install_triples(graph, name, terms);
}

/// Replace `name`'s sub-graph with exactly `terms` (re-interned into a fresh dict).
/// When `terms` is empty the named graph is removed entirely (fail-closed: no empty
/// shell left behind that a reader could otherwise treat as an existing-but-empty view).
fn install_triples(graph: &mut Graph, name: &str, terms: Vec<[Term; 3]>) {
    let g_name = Term::NamedNode(NamedNode::new_unchecked(name));
    if terms.is_empty() {
        graph.named.retain(|(n, _)| *n != g_name);
        return;
    }
    let mut dict = Dict::new();
    let ids: Vec<[sparq_core::dict::Id; 3]> = terms
        .iter()
        .map(|t| [dict.intern(&t[0]), dict.intern(&t[1]), dict.intern(&t[2])])
        .collect();
    let sub = Graph::from_parts(dict, ids);
    if let Some(slot) = graph.named.iter_mut().find(|(n, _)| *n == g_name) {
        slot.1 = sub;
    } else {
        graph.named.push((g_name, sub));
    }
}

/// The triples currently in a named graph (empty if absent).
fn named_graph_triples(graph: &Graph, name: &str) -> Vec<[Term; 3]> {
    let g_name = Term::NamedNode(NamedNode::new_unchecked(name));
    match graph.named.iter().find(|(n, _)| *n == g_name) {
        Some((_, sub)) => crate::loader::graph_triples(sub),
        None => Vec::new(),
    }
}

// ============================================================================
// [OPUS-4.8] sq-hiz4 — persist a FAITHFULLY-mappable ODRL constraint as an ACP
// re-checked condition (`auth:ConditionalGrant`) instead of freezing it into a
// one-shot materialization-time allow.
// ============================================================================

/// The `acl:` mode IRI a [`Mode`] is written as on a conditional grant's
/// `auth:mode` (the form [`crate::AuthIndex`] reads via `from_mode_iri`).
fn mode_iri(mode: Mode) -> &'static str {
    match mode {
        Mode::Read => "http://www.w3.org/ns/auth/acl#Read",
        Mode::Write => "http://www.w3.org/ns/auth/acl#Write",
        Mode::Append => "http://www.w3.org/ns/auth/acl#Append",
        Mode::Control => "http://www.w3.org/ns/auth/acl#Control",
    }
}

/// The ODRL `recipient` constraint left-operand IRI.
const ODRL_RECIPIENT: &str = "http://www.w3.org/ns/odrl/2/recipient";
/// The ODRL `assignee` constraint left-operand IRI (the party as a *constraint*,
/// distinct from the `odrl:assignee` rule attribute the evaluator already matches).
const ODRL_ASSIGNEE: &str = "http://www.w3.org/ns/odrl/2/assignee";
/// The ODRL `dateTime` constraint left-operand IRI — the request-time dimension.
/// [OPUS-4.8] sq-0q7n.
const ODRL_DATETIME: &str = "http://www.w3.org/ns/odrl/2/dateTime";

/// A faithfully-mappable `odrl:dateTime` validity window persisted onto an
/// `auth:ConditionalGrant` as live-clock bounds. [OPUS-4.8] sq-0q7n.
///
/// Both bounds are `xsd:dateTime` lexical strings (the constraint's right-operand,
/// verbatim) and **inclusive**, matching [`crate::AuthIndex`]'s `window_admits`:
/// `not_before` = the grant is inactive until this instant; `not_after` = inactive
/// after it. The session layer re-checks `Session::now` against the window per request,
/// so a lapsed window denies *this request* without waiting for a ledger refresh.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TimeWindow {
    /// Lower bound (`odrl:dateTime gteq T` → "from T", inclusive). `None` = unbounded.
    not_before: Option<String>,
    /// Upper bound (`odrl:dateTime lteq T` → "until T", inclusive). `None` = unbounded.
    not_after: Option<String>,
}

impl TimeWindow {
    /// Whether this window constrains anything (else it is the always-open window).
    fn is_some(&self) -> bool {
        self.not_before.is_some() || self.not_after.is_some()
    }
}

/// What [`map_constraints_to_agents`] concluded about a permission's constraints.
enum AgentMapping {
    /// Every constraint maps faithfully to an agent-dimension matcher; persist a
    /// `ConditionalGrant` whose `auth:agent` re-checks each of these principals,
    /// with the `except` principals carved out via an ACP `noneOf` exception matcher
    /// (the "everyone-except-X" / `recipient neq X` shape). [OPUS-4.8] sq-5037.
    Faithful {
        /// Positive recipient principals (`eq`/`isA`/`isPartOf`). Empty ⇒ no positive
        /// restriction → grant to `auth:Public` (everyone), narrowed only by `except`.
        agents: Vec<String>,
        /// Principals carved OUT (`recipient neq X`) — each becomes an ACP `noneOf`
        /// exception matcher on the grant: a session matching the grant head is denied
        /// the grant if it is one of these. Empty ⇒ no exception.
        except: Vec<String>,
        /// The live-clock validity window from a faithfully-mappable `odrl:dateTime`
        /// constraint (`gteq` → `not_before`, `lteq` → `not_after`). [OPUS-4.8] sq-0q7n.
        /// The always-open window (no `odrl:dateTime` constraint) is the default.
        window: TimeWindow,
    },
    /// At least one constraint has NO faithful ACP-condition analogue (purpose /
    /// `odrl:count` / a strict `odrl:dateTime` bound / an unrecognised operand): the rule
    /// MUST stay one-shot (fail-closed — never approximate an unmappable constraint with
    /// a looser persisted condition).
    Unmappable,
}

/// Inspect a matched permission's constraints and decide whether the WHOLE rule can
/// be persisted as re-checked ACP agent conditions.
///
/// **Faithful (→ agent matcher):** an `odrl:recipient`/`odrl:assignee` constraint
/// under `eq`/`isA` (the recipient IS this principal) or `isPartOf` (the recipient is
/// a member of a static principal set). The recipient-of-data is exactly the session
/// agent the ACP `auth:agent` head re-checks, so the persisted condition has the SAME
/// semantics — it just re-evaluates per session instead of being frozen.
///
/// **Faithful (→ noneOf exception):** an `odrl:recipient`/`odrl:assignee` constraint
/// under `neq` (the recipient is everyone EXCEPT the named party — the
/// "everyone-except-X" shape). This maps to an ACP `noneOf`: the grant head is the
/// positive recipient set (or `auth:Public` if there is no positive constraint) with
/// an `auth:exceptMatcher` carving out the named party, re-checked per session by the
/// same machinery WAC/ACP `noneOf` already uses. [OPUS-4.8] sq-5037.
///
/// **Faithful (→ live-clock window):** an `odrl:dateTime` constraint under `lteq`
/// ("until T", inclusive → `auth:notAfter`) or `gteq` ("from T", inclusive →
/// `auth:notBefore`) with an `xsd:dateTime` right-operand. [OPUS-4.8] sq-0q7n — the
/// window is persisted onto the grant and re-checked against the session clock
/// (`Session::now`) per request, so a lapsed window denies *immediately* on the next
/// request instead of only on the next ledger refresh. Strict bounds (`lt`/`gt`) are
/// deliberately left **Unmappable** to avoid an inclusive/exclusive off-by-one against
/// the inclusive `auth:notBefore`/`auth:notAfter` semantics (the one-shot path still
/// enforces them, frozen).
///
/// **Unmappable (→ stay one-shot):** `odrl:purpose` (ACP sessions carry no purpose —
/// a client app is not a purpose-of-use, so mapping it to a client matcher would
/// over-grant), a STRICT `odrl:dateTime` bound (`lt`/`gt` — see above), `odrl:count`
/// (ACP is stateless — no usage counter), and any unrecognised left-operand. Any one
/// such constraint forces the whole rule one-shot.
fn map_constraints_to_agents(rule: &Rule) -> AgentMapping {
    let mut agents: Vec<String> = Vec::new();
    let mut except: Vec<String> = Vec::new();
    let mut window = TimeWindow::default();
    for c in &rule.constraints {
        // [OPUS-4.8] sq-0q7n: a faithfully-mappable dateTime window → live-clock bounds.
        if c.left == ODRL_DATETIME {
            let Value::DateTime(t) = &c.right else {
                // a non-dateTime right-operand on a dateTime constraint is malformed.
                return AgentMapping::Unmappable;
            };
            match c.operator {
                // request time ≤ T → "until T" (inclusive upper bound).
                Operator::Lteq => {
                    // Two upper bounds → keep the EARLIER (tightest) — fail-closed.
                    set_tighter(&mut window.not_after, t, /*keep_earlier=*/ true);
                }
                // request time ≥ T → "from T" (inclusive lower bound).
                Operator::Gteq => {
                    // Two lower bounds → keep the LATER (tightest) — fail-closed.
                    set_tighter(&mut window.not_before, t, /*keep_earlier=*/ false);
                }
                // strict bounds have no inclusive window analogue → one-shot.
                _ => return AgentMapping::Unmappable,
            }
            continue;
        }
        if c.left != ODRL_RECIPIENT && c.left != ODRL_ASSIGNEE {
            // purpose / count / anything else → no faithful condition.
            return AgentMapping::Unmappable;
        }
        match c.operator {
            // recipient IS this principal (identity) → one agent matcher.
            Operator::Eq | Operator::IsA => match &c.right {
                Value::Iri(s) | Value::Str(s) => agents.push(s.clone()),
                // a numeric/dateTime recipient is malformed → fail-closed.
                _ => return AgentMapping::Unmappable,
            },
            // recipient ∈ {a|b|c} (static set) → one agent matcher per member.
            Operator::IsPartOf => {
                let members: Vec<String> = c
                    .right
                    .as_str()
                    .split(['|', ' ', ','])
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect();
                if members.is_empty() {
                    return AgentMapping::Unmappable;
                }
                agents.extend(members);
            }
            // recipient ≠ X (everyone EXCEPT X) → an ACP noneOf exception matcher
            // carving out X from the grant. A numeric/dateTime right-operand is
            // malformed → fail-closed. [OPUS-4.8] sq-5037.
            Operator::Neq => match &c.right {
                Value::Iri(s) | Value::Str(s) => except.push(s.clone()),
                _ => return AgentMapping::Unmappable,
            },
            // order operators (lt/gt/…) on a recipient are not meaningful → one-shot.
            _ => return AgentMapping::Unmappable,
        }
    }
    AgentMapping::Faithful { agents, except, window }
}

/// Tighten an inclusive window bound with a new `xsd:dateTime` candidate `t`, compared
/// by the **real UTC instant** each denotes (`sparq_policy::cmp_datetime`, the SAME
/// offset-aware normalizer the evaluator uses — never raw lexical `str::cmp`, which
/// would pick the wrong bound for mixed-offset times). When two constraints set the
/// same side, keep the TIGHTER bound (the earlier upper bound / the later lower bound)
/// so the persisted window is the intersection — fail-closed, never wider than any one
/// constraint. An **unparseable** incoming `t` is treated as not-tighter (the existing
/// bound is kept) so a malformed candidate can never *widen* the window. [OPUS-4.8]
/// sq-0q7n.
fn set_tighter(slot: &mut Option<String>, t: &str, keep_earlier: bool) {
    use std::cmp::Ordering;
    match slot {
        None => *slot = Some(t.to_owned()),
        Some(cur) => {
            // Replace only when `t` is strictly tighter than `cur` by instant order;
            // an incomparable (unparseable) pair never replaces (fail-closed).
            let replace = match sparq_policy::cmp_datetime(t, cur.as_str()) {
                Some(Ordering::Less) => keep_earlier,
                Some(Ordering::Greater) => !keep_earlier,
                _ => false,
            };
            if replace {
                *cur = t.to_owned();
            }
        }
    }
}

/// Whether a recipient principal IRI is safe to write as an `auth:agent` head: it
/// must NOT smuggle the reserved pair encoding (`urn:sparq:` / `&client=`), or a
/// crafted recipient could impersonate a minted pair principal (defense in depth,
/// mirrors [`crate::loader::session_value_allowed`]). The session-layer check
/// already rejects such SESSIONS; this also keeps them out of the GRANT head.
fn recipient_principal_allowed(p: &str) -> bool {
    crate::loader::session_value_allowed(p)
}

/// Evaluate `policy` against `request` and, for the matched permission, EITHER
/// persist its recipient/assignee constraint as a re-checked ACP
/// `auth:ConditionalGrant` (so the granted agent is verified per session through the
/// real enforcement path) OR — when the permission carries a constraint with no
/// faithful ACP-condition analogue — fall back to the one-shot
/// [`materialize_permission`] (the constraint is checked once, at materialization).
///
/// # When a conditional grant is emitted (faithful)
///
/// All of the matched permission's constraints are `odrl:recipient`/`odrl:assignee`
/// constraints under `eq`/`isA`/`isPartOf` (see the crate-internal
/// `map_constraints_to_agents`). The emitted grant is:
///
/// ```text
/// <grant> a auth:ConditionalGrant ; auth:effect auth:Allow ;
///         auth:agent <recipient> ; auth:client auth:AnyClient ; auth:issuer auth:AnyIssuer ;
///         auth:mode acl:<Mode> ; auth:graph <target> .
/// ```
///
/// re-checked by [`crate::AuthIndex::accessible`]: a session whose agent is the
/// recipient is granted; any other agent (or anonymous) is denied — **without**
/// re-running the ODRL evaluator. A recipient *set* emits one grant per member (the
/// auth view unions the allows). When the rule has NO recipient constraint, the grant
/// head is `auth:Public` (any session) — only valid because the action/target/duties
/// were already satisfied at materialization.
///
/// # Fail-closed
///
/// - A Deny (prohibition override, unmet *unmappable* constraint, undischarged duty)
///   materializes **nothing** — exactly as the one-shot path.
/// - An unmapped action, or a missing target, materializes nothing.
/// - **Mixed constraints fail safe:** if ANY constraint is unmappable (`purpose`,
///   `dateTime`, `count`, a `neq`/order recipient), the WHOLE rule falls back to the
///   one-shot path so the unmappable bound is still enforced (frozen) — a persisted
///   condition is emitted ONLY when every constraint maps faithfully.
/// - A recipient IRI inside the reserved pair encoding is dropped from the grant head
///   (it could otherwise impersonate a minted pair principal).
///
/// Returns a [`BridgeOutcome`]; on a conditional grant `grant_triple` reports the
/// `(agent, auth:effect, graph)` of the FIRST emitted grant (audit anchor) and
/// `mode` the mapped mode. The free-function form does not reindex a [`crate::PodStore`]
/// — use [`crate::PodStore::materialize_odrl_permission_conditional`].
pub fn materialize_permission_conditional(
    graph: &mut Graph,
    policy: &Policy,
    request: &Request,
) -> BridgeOutcome {
    // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy first. [OPUS-4.8] sq-ihqbl.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return refusal;
    }
    // 1. Action → Mode (shared with the one-shot path). Unmapped → no grant.
    let Some(mode) = action_to_mode(&request.action) else {
        return BridgeOutcome::denied(vec![format!(
            "ODRL action <{}> has no WAC/ACP mode mapping; no grant materialized",
            request.action
        )]);
    };

    // 2. Find the permission whose action/target match AND whose duties are
    //    discharged AND whose constraints map faithfully to agent conditions. The
    //    recipient constraint is NOT required to hold against the request party here
    //    — the persisted condition re-checks it per session. Prohibitions still
    //    override (deny-overrides), so consult the evaluator's prohibition verdict.
    if let Some(p) = matched_prohibition(policy, request) {
        return BridgeOutcome::denied(vec![format!(
            "prohibition {} matches the request (deny-overrides); no grant materialized",
            p.id
        )]);
    }
    let Some(target) = request.target.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL Permit has no concrete target graph IRI; no grant materialized".to_owned(),
        ]);
    };

    let mut fallback_reasons: Vec<String> = Vec::new();
    for rule in &policy.permissions {
        // Action + target must agree (assignee/recipient are handled as conditions).
        if !rule_action_target_match(rule, request, mode, target) {
            continue;
        }
        // Duties must be discharged at materialization (no ACP analogue → one-shot
        // semantics; an undischarged duty blocks this rule).
        if rule.duties.iter().any(|d| !request.discharged_duties.contains(&d.action.0)) {
            fallback_reasons.push(format!("permission {} has an undischarged duty", rule.id));
            continue;
        }
        match map_constraints_to_agents(rule) {
            AgentMapping::Faithful { agents: recipients, except, window } => {
                let agents = condition_agents(rule, &recipients);
                if agents.is_empty() {
                    // every recipient was reserved-encoded → fail-closed, nothing.
                    fallback_reasons.push(format!(
                        "permission {} recipients are all reserved-encoded; no grant",
                        rule.id
                    ));
                    continue;
                }
                // `recipient neq X` → an ACP noneOf exception carving out X. A reserved-
                // encoded exclusion would be silently un-enforceable as a matcher, which
                // would WIDEN the grant (X regains access) — fail-closed: drop the whole
                // rule to one-shot rather than emit an exception that cannot bite.
                let excepts = condition_excepts(&except);
                if excepts.len() != except.len() {
                    fallback_reasons.push(format!(
                        "permission {} has a reserved-encoded neq recipient; one-shot path",
                        rule.id
                    ));
                    continue;
                }
                let (first, emitted) = append_conditional_grants(
                    graph, &agents, &excepts, &window, mode, target, GrantEffect::Allow,
                );
                return BridgeOutcome {
                    granted: true,
                    mode: Some(mode),
                    grant_triple: Some(first),
                    emitted,
                    ..BridgeOutcome::default()
                };
            }
            AgentMapping::Unmappable => {
                // This permission carries a constraint with no faithful condition
                // analogue → the one-shot path must check it (frozen) instead.
                fallback_reasons.push(format!(
                    "permission {} has a constraint with no faithful ACP condition; one-shot path",
                    rule.id
                ));
            }
        }
    }

    // 3. No faithfully-conditional permission applied → fall back to the EXISTING
    //    one-shot behaviour (which evaluates the unmappable constraints against the
    //    supplied request context and emits a frozen allow iff they hold).
    let out = materialize_permission(graph, policy, request);
    if !out.granted && out.reasons.is_empty() {
        return BridgeOutcome::denied(fallback_reasons);
    }
    out
}

/// Evaluate `policy`'s **prohibitions** against `request` and, for a prohibition whose
/// recipient/assignee constraints map faithfully to agent conditions, persist a
/// re-checked ACP **conditional deny** (`auth:ConditionalGrant` with `auth:effect
/// auth:Deny`) — the dual of [`materialize_permission_conditional`]. [OPUS-4.8] sq-4r70.
///
/// This is the constraint-CONDITIONAL deny: instead of freezing a deny at
/// materialization time (one-shot [`materialize_prohibition`]), the carve-out is
/// persisted as a condition the session layer re-checks per session. A prohibition
/// `recipient eq bob` materializes a deny that applies **only to bob's sessions**; a
/// `recipient neq bob` materializes a deny that applies to **everyone except bob** (an
/// ACP `noneOf` exception carving bob back IN to access). The deny composes with
/// deny-overrides via the SAME `∪ allow ∖ ∪ deny` enforcement
/// ([`crate::AuthIndex::accessible`]) — a conditional deny that applies to a session
/// removes the target from that session's accessible set, beating any allow.
///
/// # When a conditional deny is emitted (faithful)
///
/// All of the matched prohibition's constraints are `odrl:recipient`/`odrl:assignee`
/// constraints under `eq`/`isA`/`isPartOf`/`neq` (see the crate-internal
/// `map_constraints_to_agents`), the action [`action_to_mode`]-maps, and the request
/// names a target. The recipient
/// constraint is NOT required to hold against the request party — the persisted
/// condition re-checks it per session, exactly as the allow path.
///
/// # Fail-closed
///
/// - **Mixed / unmappable constraints fall back to one-shot:** if the prohibition
///   carries a constraint with no faithful ACP-condition analogue (`purpose`,
///   `dateTime`, `count`), the WHOLE rule falls back to [`materialize_prohibition`], so
///   the unmappable bound is still enforced (the one-shot deny is materialized iff the
///   prohibition currently matches — frozen). A persisted deny condition is emitted ONLY
///   when every constraint maps faithfully.
/// - An unmapped action or a missing target materializes nothing.
/// - A reserved-encoded recipient/exclusion cannot become an enforceable matcher; the
///   rule falls back to one-shot rather than emit a deny condition that cannot bite
///   (which would FAIL OPEN — a deny silently dropped widens access).
///
/// Returns a [`BridgeOutcome`]; on a conditional deny `prohibited == true`,
/// `deny_triple` reports the `(agent, auth:effect, graph)` anchor of the first emitted
/// deny head and `mode` the mapped mode. The free-function form does not reindex a
/// [`crate::PodStore`] — go through a `materialize_*` method for that.
pub fn materialize_prohibition_conditional(
    graph: &mut Graph,
    policy: &Policy,
    request: &Request,
) -> BridgeOutcome {
    // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy first. [OPUS-4.8] sq-ihqbl.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return refusal;
    }
    // 1. Action → Mode (shared with the one-shot path). Unmapped → no deny.
    let Some(mode) = action_to_mode(&request.action) else {
        return BridgeOutcome::denied(vec![format!(
            "ODRL action <{}> has no WAC/ACP mode mapping; no deny materialized",
            request.action
        )]);
    };
    let Some(target) = request.target.as_deref() else {
        return BridgeOutcome::denied(vec![
            "ODRL prohibition has no concrete target graph IRI; no deny materialized".to_owned(),
        ]);
    };

    // 2. Find a prohibition whose action/target match AND whose constraints map
    //    faithfully to agent conditions. The recipient/assignee constraint is NOT
    //    required to hold against the request party — the persisted condition re-checks
    //    it per session (the dual of the conditional allow path).
    let mut fallback_reasons: Vec<String> = Vec::new();
    for rule in &policy.prohibitions {
        if !rule_action_target_match(rule, request, mode, target) {
            continue;
        }
        match map_constraints_to_agents(rule) {
            AgentMapping::Faithful { agents: recipients, except, window } => {
                // [OPUS-4.8] sq-0q7n: a time-windowed DENY is fail-OPEN — outside the
                // window the deny would lapse and the carved-out party would regain
                // access. A live-clock window is safe only on an ALLOW (a lapsed allow
                // removes access — fail-closed). So a dateTime-windowed prohibition must
                // stay one-shot (the frozen deny is materialized iff it matches now).
                if window.is_some() {
                    fallback_reasons.push(format!(
                        "prohibition {} carries a dateTime window (fail-open as a live deny); one-shot path",
                        rule.id
                    ));
                    continue;
                }
                let agents = condition_agents(rule, &recipients);
                if agents.is_empty() {
                    fallback_reasons.push(format!(
                        "prohibition {} recipients are all reserved-encoded; no deny",
                        rule.id
                    ));
                    continue;
                }
                let excepts = condition_excepts(&except);
                if excepts.len() != except.len() {
                    // A reserved-encoded exclusion would silently re-admit the carved-out
                    // party to the DENY (i.e. they'd escape it) — fail-closed to one-shot.
                    fallback_reasons.push(format!(
                        "prohibition {} has a reserved-encoded neq recipient; one-shot path",
                        rule.id
                    ));
                    continue;
                }
                let (first, emitted) = append_conditional_grants(
                    graph, &agents, &excepts, &TimeWindow::default(), mode, target, GrantEffect::Deny,
                );
                return BridgeOutcome {
                    prohibited: true,
                    mode: Some(mode),
                    deny_triple: Some(first),
                    emitted,
                    ..BridgeOutcome::default()
                };
            }
            AgentMapping::Unmappable => {
                // A constraint with no faithful condition analogue (purpose / dateTime /
                // count) → the one-shot deny path must check it (frozen) instead.
                fallback_reasons.push(format!(
                    "prohibition {} has a constraint with no faithful ACP condition; one-shot path",
                    rule.id
                ));
            }
        }
    }

    // 3. No faithfully-conditional prohibition applied → fall back to the EXISTING
    //    one-shot deny (which checks the unmappable constraints against the supplied
    //    request context and emits a frozen `auth:deny*` iff the prohibition matches).
    let out = materialize_prohibition(graph, policy, request);
    if !out.prohibited && out.reasons.is_empty() {
        return BridgeOutcome::denied(fallback_reasons);
    }
    out
}

/// The principal-space `auth:agent` heads for a faithful recipient set, dropping any
/// reserved-encoded recipient. An empty recipient list means the rule had NO agent
/// restriction → a single `auth:Public` head (any session matches).
fn condition_agents(_rule: &Rule, recipients: &[String]) -> Vec<String> {
    if recipients.is_empty() {
        return vec![PUBLIC.to_owned()];
    }
    recipients
        .iter()
        .filter(|r| recipient_principal_allowed(r))
        .map(|r| normalise_recipient_principal(r))
        .collect()
}

/// The principal-space carve-out heads for a `recipient neq X` exception set, dropping
/// any reserved-encoded principal (the caller treats a shortfall as fail-closed — a
/// dropped exclusion would re-admit the carved-out party). [OPUS-4.8] sq-5037.
fn condition_excepts(except: &[String]) -> Vec<String> {
    except
        .iter()
        .filter(|r| recipient_principal_allowed(r))
        .map(|r| normalise_recipient_principal(r))
        .collect()
}

/// Map an ODRL recipient value to a principal-space `auth:agent` IRI. The two ODRL
/// "any recipient" sentinels are folded onto the auth principals the session layer
/// already understands; a concrete WebID passes through unchanged.
fn normalise_recipient_principal(r: &str) -> String {
    match r {
        "http://www.w3.org/ns/odrl/2/All" | "http://www.w3.org/ns/odrl/2/Group" => PUBLIC.to_owned(),
        "http://www.w3.org/ns/odrl/2/AllConnections" => AUTHENTICATED.to_owned(),
        _ => r.to_owned(),
    }
}

/// Does the rule's action permit `mode`'s action and its target agree with the
/// request? (Assignee/recipient are deferred to the persisted condition.)
fn rule_action_target_match(rule: &Rule, request: &Request, _mode: Mode, target: &str) -> bool {
    let req_action = sparq_policy::Action(request.action.clone());
    if !rule.action.permits(&req_action) {
        return false;
    }
    match &rule.target {
        Some(t) => t == target,
        None => true,
    }
}


/// The deontic force of a materialized `auth:ConditionalGrant` — selects the
/// `auth:effect` object emitted by [`append_conditional_grants`]. [OPUS-4.8] sq-4r70.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrantEffect {
    /// A conditional **allow** (`auth:effect auth:Allow`) — the recipient is granted.
    Allow,
    /// A conditional **deny** (`auth:effect auth:Deny`) — the dual: the matched
    /// session is denied, and the deny overrides any allow for the same
    /// principal+target+mode (the session layer subtracts `∪ deny` from `∪ allow`).
    Deny,
}

impl GrantEffect {
    /// The `auth:`-local effect object (`Allow` / `Deny`) and the grant-IRI key.
    fn iri_local(self) -> &'static str {
        match self {
            GrantEffect::Allow => "Allow",
            GrantEffect::Deny => "Deny",
        }
    }
}

/// Append `auth:ConditionalGrant` triples (allow OR deny — see `effect`) for each agent
/// head onto BOTH the `<urn:sparq:auth>` view and the bridged-provenance graph,
/// preserving existing triples. Returns the `(agent, auth:effect, graph)` audit anchor
/// of the first grant AND the full set of emitted head triples (for the bridge ledger).
/// [OPUS-4.8] sq-dpk4 / sq-4r70.
///
/// `excepts` are the principals carved OUT (the `recipient neq X` / "everyone-except"
/// shape — [OPUS-4.8] sq-5037). Each becomes an ACP `noneOf` exception: the grant gets
/// an `auth:exceptMatcher <m>` and the matcher `<m>` is materialized with the accept-set
/// facts the session layer reads (`solidx:acceptsAgentP <X>` + `solidx:acceptsClientP
/// auth:AnyClient`). [`crate::AuthIndex::cond_applies`] then suppresses the grant for any
/// session the matcher accepts — i.e. for `X` under any client — so `X` is denied while
/// every other session keeps the grant. This is EXACTLY the shape the WAC/ACP `noneOf`
/// rules (`rules/acp-c.n3`) emit, re-checked by the same code path.
///
/// `effect` selects the deontic force ([OPUS-4.8] sq-4r70): [`GrantEffect::Allow`]
/// emits `auth:effect auth:Allow` (the conditional grant); [`GrantEffect::Deny`] emits
/// `auth:effect auth:Deny` (the conditional deny — the dual). A conditional deny is
/// honoured by the SAME [`crate::AuthIndex::accessible`] path: a matching deny condition
/// adds the graph to the `denied` set, which is subtracted from `allowed`
/// (deny-overrides). The deny's grant IRI carries an `&effect=deny` key so it never
/// collides with an allow grant for the same `(agent, mode, graph, excepts)`.
fn append_conditional_grants(
    graph: &mut Graph,
    agents: &[String],
    excepts: &[String],
    window: &TimeWindow,
    mode: Mode,
    target: &str,
    effect: GrantEffect,
) -> ((String, String, String), Vec<[Term; 3]>) {
    let mut emitted: Vec<[Term; 3]> = Vec::new();

    let type_p = NamedNode::new_unchecked(RDF_TYPE);
    let cond_class = NamedNode::new_unchecked(format!("{AUTH_NS}ConditionalGrant"));
    let effect_p = NamedNode::new_unchecked(format!("{AUTH_NS}effect"));
    let effect_o = NamedNode::new_unchecked(format!("{AUTH_NS}{}", effect.iri_local()));
    let agent_p = NamedNode::new_unchecked(format!("{AUTH_NS}agent"));
    let client_p = NamedNode::new_unchecked(format!("{AUTH_NS}client"));
    let any_client = NamedNode::new_unchecked(crate::authindex::ANY_CLIENT);
    // [OPUS-4.8] sq-3jtd.6: a bridged grant carries no issuer constraint, so it spans the
    // issuer-dimension top — exactly as it spans the client top. The head MUST state this
    // explicitly: the session layer treats a conditional grant whose `auth:issuer` head is
    // absent as fail-closed (it never applies), matching the `auth:client` convention.
    let issuer_p = NamedNode::new_unchecked(format!("{AUTH_NS}issuer"));
    let any_issuer = NamedNode::new_unchecked(crate::authindex::ANY_ISSUER);
    let mode_p = NamedNode::new_unchecked(format!("{AUTH_NS}mode"));
    let mode_o = NamedNode::new_unchecked(mode_iri(mode));
    let graph_p = NamedNode::new_unchecked(format!("{AUTH_NS}graph"));
    let graph_o = NamedNode::new_unchecked(target);
    let except_p = NamedNode::new_unchecked(format!("{AUTH_NS}exceptMatcher"));
    let accepts_agent_p = NamedNode::new_unchecked(format!("{SOLIDX_NS}acceptsAgentP"));
    let accepts_client_p = NamedNode::new_unchecked(format!("{SOLIDX_NS}acceptsClientP"));
    let accepts_issuer_p = NamedNode::new_unchecked(format!("{SOLIDX_NS}acceptsIssuerP"));
    // [OPUS-4.8] sq-0q7n: the live-clock window predicates + the xsd:dateTime datatype the
    // bounds are typed with (so they round-trip as the same literal the evaluator parses).
    let not_before_p = NamedNode::new_unchecked(format!("{AUTH_NS}notBefore"));
    let not_after_p = NamedNode::new_unchecked(format!("{AUTH_NS}notAfter"));
    let xsd_datetime = NamedNode::new_unchecked(XSD_DATETIME);

    // One deterministic exception matcher per carved-out principal, shared across the
    // grant heads (its accept-set is the same regardless of which positive head it
    // attaches to). Each accepts (agent = X, client = any, issuer = any) → suppresses the
    // grant for X. [OPUS-4.8] sq-3jtd.6: the issuer accept (auth:AnyIssuer) makes the
    // carve-out span the issuer dimension too — without it the three-dimensional
    // matcher_accepts would never fire (it requires all three dimensions to accept).
    let except_matchers: Vec<(String, [Term; 3])> = excepts
        .iter()
        .map(|x| {
            let m = format!("urn:sparq:odrl-except?agent={}", sparq_reason::n3::encode_for_uri(x));
            (
                m,
                [
                    Term::NamedNode(NamedNode::new_unchecked(x)),
                    Term::NamedNode(any_client.clone()),
                    Term::NamedNode(any_issuer.clone()),
                ],
            )
        })
        .collect();

    let mut first: Option<(String, String, String)> = None;
    for agent in agents {
        // A deterministic grant IRI keyed on (agent, mode, graph, excepts) so re-
        // materializing the same condition is idempotent (no duplicate heads) and an
        // exception-bearing grant never collides with an unconditional one.
        let except_key = except_matchers
            .iter()
            .map(|(m, _)| sparq_reason::n3::encode_for_uri(m))
            .collect::<Vec<_>>()
            .join(",");
        // [OPUS-4.8] sq-0q7n: the window is part of the grant identity — two grants for
        // the same (agent, mode, graph, excepts, effect) but DIFFERENT windows must not
        // collide on the same IRI (one would silently overwrite the other's bounds).
        let grant_iri = format!(
            "urn:sparq:odrl-cond?agent={}&mode={}&graph={}&except={}&effect={}&notBefore={}&notAfter={}",
            sparq_reason::n3::encode_for_uri(agent),
            sparq_reason::n3::encode_for_uri(mode_iri(mode)),
            sparq_reason::n3::encode_for_uri(target),
            except_key,
            effect.iri_local(),
            sparq_reason::n3::encode_for_uri(window.not_before.as_deref().unwrap_or("")),
            sparq_reason::n3::encode_for_uri(window.not_after.as_deref().unwrap_or("")),
        );
        let g = Term::NamedNode(NamedNode::new_unchecked(&grant_iri));
        let agent_o = Term::NamedNode(NamedNode::new_unchecked(agent));
        let mut head = vec![
            [g.clone(), Term::NamedNode(type_p.clone()), Term::NamedNode(cond_class.clone())],
            [g.clone(), Term::NamedNode(effect_p.clone()), Term::NamedNode(effect_o.clone())],
            [g.clone(), Term::NamedNode(agent_p.clone()), agent_o],
            [g.clone(), Term::NamedNode(client_p.clone()), Term::NamedNode(any_client.clone())],
            [g.clone(), Term::NamedNode(issuer_p.clone()), Term::NamedNode(any_issuer.clone())],
            [g.clone(), Term::NamedNode(mode_p.clone()), Term::NamedNode(mode_o.clone())],
            [g.clone(), Term::NamedNode(graph_p.clone()), Term::NamedNode(graph_o.clone())],
        ];
        // [OPUS-4.8] sq-0q7n: the live-clock window bounds (xsd:dateTime literals),
        // re-checked against `Session::now` by `crate::AuthIndex::cond_applies`.
        if let Some(nb) = &window.not_before {
            head.push([
                g.clone(),
                Term::NamedNode(not_before_p.clone()),
                Term::Literal(Literal::new_typed_literal(nb.as_str(), xsd_datetime.clone())),
            ]);
        }
        if let Some(na) = &window.not_after {
            head.push([
                g.clone(),
                Term::NamedNode(not_after_p.clone()),
                Term::Literal(Literal::new_typed_literal(na.as_str(), xsd_datetime.clone())),
            ]);
        }
        // Wire each exception matcher onto the grant + materialize its accept-set facts.
        for (m, [agent_accept, client_accept, issuer_accept]) in &except_matchers {
            let m_node = Term::NamedNode(NamedNode::new_unchecked(m));
            head.push([g.clone(), Term::NamedNode(except_p.clone()), m_node.clone()]);
            head.push([m_node.clone(), Term::NamedNode(accepts_agent_p.clone()), agent_accept.clone()]);
            head.push([m_node.clone(), Term::NamedNode(accepts_client_p.clone()), client_accept.clone()]);
            head.push([m_node, Term::NamedNode(accepts_issuer_p.clone()), issuer_accept.clone()]);
        }
        for t in head {
            if !emitted.contains(&t) {
                emitted.push(t);
            }
        }
        if first.is_none() {
            first = Some((
                agent.clone(),
                format!("{AUTH_NS}effect"),
                target.to_owned(),
            ));
        }
    }

    append_bridged_triples(graph, &emitted);
    (first.expect("at least one agent head emitted"), emitted)
}

/// `rdf:type` IRI, for the conditional-grant class triple.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// `xsd:dateTime` datatype IRI, for the live-clock window bound literals. [OPUS-4.8]
/// sq-0q7n.
const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

// ============================================================================
// [OPUS-4.8] sq-dpk4 — refresh / revocation of bridged ODRL grants.
//
// THE GAP this closes: the materialize_* functions above only ever APPEND. When the
// underlying ODRL policy changes — a permission withdrawn, a time window lapsed, a
// re-evaluation that now Denies — the previously-materialized grant stays in the auth
// view, so access that should be GONE persists. And a wholesale static WAC/ACP
// re-materialization (`install_auth_view`) rebuilds `<urn:sparq:auth>` and would drop
// every bridged grant. Both are reconciled by tracking each bridged materialization in
// a ledger and REPLAYING the ledger over a captured static baseline on demand.
//
// FAIL-CLOSED: refresh rebuilds the auth view as `static_baseline ∪ replay(ledger)`.
// An entry whose ODRL re-evaluation no longer yields a grant emits NOTHING on replay,
// so it is dropped — a withdrawn / lapsed / now-Denied grant LOSES access. The static
// baseline is captured independently (the exact `install_auth_view` output), so a
// static grant is never inspected, re-evaluated, or dropped by this path.
// ============================================================================

/// Which bridge entry point produced a tracked grant — replayed verbatim on refresh so
/// the SAME fail-closed evaluation re-runs (a withdrawn/lapsed/now-Denied policy emits
/// nothing → the entry is retracted). [OPUS-4.8] sq-dpk4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeKind {
    /// [`materialize_permission`] — a definite-Permit allow grant.
    Permission,
    /// [`materialize_prohibition`] — a matched-Prohibition deny.
    Prohibition,
    /// [`materialize_policy`] — both sides of a policy at once.
    Policy,
    /// [`materialize_permission_conditional`] — a re-checked conditional grant (or its
    /// one-shot fallback).
    PermissionConditional,
    /// [`materialize_prohibition_conditional`] — a re-checked conditional deny (or its
    /// one-shot fallback). [OPUS-4.8] sq-4r70.
    ProhibitionConditional,
    /// A STATEFUL `odrl:count` allow grant (via
    /// [`crate::PodStore::materialize_odrl_permission_counted`]). The count was consumed
    /// atomically at exercise; on refresh the grant is re-checked (read-only — never
    /// consumes again) against the usage state and RETRACTED once the budget is
    /// exhausted, so the bridged grant self-retracts on exhaustion. Only ever recorded
    /// under the `count-enforcement` feature. [OPUS-4.8] sq-58mh.
    #[cfg(feature = "count-enforcement")]
    PermissionCounted,
}

/// One tracked bridged materialization: the originating ODRL `(policy, request, kind)`
/// plus the auth triples it last emitted. The provenance that distinguishes a bridged
/// grant from a static WAC/ACP one (those are never in the ledger), and the unit of
/// retraction on refresh. [OPUS-4.8] sq-dpk4.
#[derive(Debug, Clone)]
pub struct BridgeEntry {
    /// The ODRL policy this grant was bridged from.
    pub policy: Policy,
    /// The request `(action, target, party, context, duties)` it was evaluated against.
    pub request: Request,
    /// Which bridge entry point produced it (replayed verbatim).
    pub kind: BridgeKind,
}

/// The ordered set of bridged materializations a [`crate::PodStore`] is tracking, plus
/// the static-baseline auth view captured at the last static (WAC/ACP) materialization.
///
/// # The model (sq-dpk4)
///
/// - Each successful bridge call records a [`BridgeEntry`] (the `(policy, request, kind)`).
/// - [`BridgeLedger::capture_static_baseline`] snapshots the EXACT `<urn:sparq:auth>`
///   triples produced by a static WAC/ACP materialization — the grants the refresh must
///   never touch. Capturing the install output directly (not by subtracting provenance)
///   means a static grant byte-identical to a bridged one still survives a refresh.
/// - [`BridgeLedger::refresh`] rebuilds the auth view as `static_baseline ∪
///   replay(valid entries)`: it resets `<urn:sparq:auth>` to the baseline, clears the
///   provenance graph, replays every entry, and DROPS the entries that re-evaluate to
///   nothing (withdrawn permission / lapsed window / now-Deny / now-matching prohibition).
///
/// Fail-closed throughout: a retracted entry loses access immediately, and on any
/// ambiguity the re-evaluation denies (the underlying evaluator is fail-closed), so the
/// entry is dropped rather than left stale.
#[derive(Debug, Clone, Default)]
pub struct BridgeLedger {
    entries: Vec<BridgeEntry>,
    /// The static (WAC/ACP) `<urn:sparq:auth>` triples to rebuild from on refresh.
    /// `None` until the first static materialization is captured (a store that only
    /// ever bridged grants has an empty static baseline — refresh starts from nothing).
    static_baseline: Option<Vec<[Term; 3]>>,
    /// The injected usage-counter store backing every [`BridgeKind::PermissionCounted`]
    /// entry's stateful `odrl:count` budget. Set on the first counted bridge call and
    /// re-used by [`refresh`](BridgeLedger::refresh) to re-check (read-only) whether a
    /// counted grant's budget is still available — so an exhausted grant is retracted.
    /// `None` ⇒ no counted grant was ever bridged. [OPUS-4.8] sq-58mh.
    #[cfg(feature = "count-enforcement")]
    count_store: Option<count::CounterHandle>,
}

impl BridgeLedger {
    /// A fresh, empty ledger.
    pub fn new() -> BridgeLedger {
        BridgeLedger::default()
    }

    /// The tracked bridged entries (for inspection/audit).
    pub fn entries(&self) -> &[BridgeEntry] {
        &self.entries
    }

    /// Record a successful bridge of `kind` for `(policy, request)`. Call this only when
    /// the corresponding `materialize_*` returned a materialized outcome (`granted` /
    /// `prohibited`). A no-op materialization is NOT tracked (nothing to retract).
    ///
    /// Idempotent on `(kind, target, party)`: re-recording the same grant slot REPLACES
    /// the tracked `(policy, request)` rather than appending a duplicate, so a caller can
    /// re-bridge with an updated policy/request and the ledger tracks exactly one entry
    /// per logical grant.
    pub fn record(&mut self, policy: &Policy, request: &Request, kind: BridgeKind) {
        let slot = (kind, request.target.clone(), request.party.clone());
        if let Some(e) = self.entries.iter_mut().find(|e| {
            (e.kind, e.request.target.clone(), e.request.party.clone()) == slot
        }) {
            e.policy = policy.clone();
            e.request = request.clone();
            return;
        }
        self.entries.push(BridgeEntry { policy: policy.clone(), request: request.clone(), kind });
    }

    /// Replace the tracked `(policy, request)` for the grant slot matching
    /// `(kind, request.target, request.party)` with the supplied (updated) ones, so the
    /// next [`BridgeLedger::refresh`] re-evaluates against the NEW policy / request
    /// context (a withdrawn permission, a lapsed window, a now-Deny). Returns `true` if a
    /// tracked entry matched. A no-match returns `false` and changes nothing — there is
    /// no bridged grant to refresh for that slot. [OPUS-4.8] sq-dpk4.
    pub fn update(&mut self, policy: &Policy, request: &Request, kind: BridgeKind) -> bool {
        let slot = (kind, request.target.clone(), request.party.clone());
        match self.entries.iter_mut().find(|e| {
            (e.kind, e.request.target.clone(), e.request.party.clone()) == slot
        }) {
            Some(e) => {
                e.policy = policy.clone();
                e.request = request.clone();
                true
            }
            None => false,
        }
    }

    /// Snapshot the current `<urn:sparq:auth>` triples as the static baseline — the
    /// grants a refresh rebuilds from and never re-evaluates. Call this right after a
    /// static WAC/ACP materialization (which produced the view) and BEFORE replaying any
    /// bridged grant on top.
    pub fn capture_static_baseline(&mut self, graph: &Graph) {
        self.static_baseline = Some(named_graph_triples(graph, AUTH_GRAPH));
    }

    /// Re-evaluate every tracked bridged grant against its (possibly changed) ODRL
    /// policy and rebuild the auth view as `static_baseline ∪ replay(still-valid
    /// entries)`, retracting the grants that no longer hold. [OPUS-4.8] sq-dpk4.
    ///
    /// Returns the number of entries RETRACTED (re-evaluated to nothing). The caller
    /// ([`crate::PodStore::refresh_odrl_grants`]) reindexes afterward so the change
    /// takes effect on the next `accessible`/`query_as`.
    ///
    /// # Fail-closed
    ///
    /// - The view is first reset to the static baseline (or empty if none captured) and
    ///   the provenance graph cleared, so NO stale bridged triple can survive unless an
    ///   entry re-emits it.
    /// - Each entry is replayed through its original `materialize_*` function, which
    ///   re-runs the fail-closed ODRL evaluation: a withdrawn permission, a lapsed time
    ///   window, a now-matching prohibition, or any ambiguous re-eval emits nothing, and
    ///   the entry is dropped.
    /// - A static grant is never in `entries`, never re-evaluated, and always present in
    ///   the baseline — refresh cannot widen or drop it.
    pub fn refresh(&mut self, graph: &mut Graph) -> usize {
        // 1. Reset the enforcement view to the captured static baseline, and clear ALL
        //    bridged provenance — nothing bridged survives unless an entry re-emits it.
        let baseline = self.static_baseline.clone().unwrap_or_default();
        install_triples(graph, AUTH_GRAPH, baseline);
        install_triples(graph, AUTH_BRIDGED_GRAPH, Vec::new());

        // 2. Replay each entry; keep only those that still materialize something. A
        //    counted entry ([OPUS-4.8] sq-58mh) is re-checked against the usage store
        //    (read-only) — exhausted ⇒ emits nothing ⇒ retracted (self-retract on
        //    exhaustion).
        #[cfg(feature = "count-enforcement")]
        let count_store = self.count_store.clone();
        let entries = std::mem::take(&mut self.entries);
        let before = entries.len();
        for entry in entries {
            #[cfg(feature = "count-enforcement")]
            let out = replay(graph, &entry, count_store.as_ref());
            #[cfg(not(feature = "count-enforcement"))]
            let out = replay(graph, &entry);
            if !out.emitted.is_empty() {
                self.entries.push(entry);
            }
        }
        before - self.entries.len()
    }

    /// Remember the usage-counter `store` backing this ledger's counted grants, so a
    /// later [`refresh`](BridgeLedger::refresh) can re-check (read-only) whether each
    /// [`BridgeKind::PermissionCounted`] entry's budget is still available. Idempotent;
    /// the latest store wins (a caller injecting a different store re-points refresh at
    /// it). [OPUS-4.8] sq-58mh.
    #[cfg(feature = "count-enforcement")]
    pub(crate) fn set_count_store(&mut self, store: &count::CounterHandle) {
        self.count_store = Some(store.clone());
    }
}

/// Re-run a tracked entry's original bridge function against the current graph,
/// re-evaluating its ODRL policy and re-emitting iff it still holds. [OPUS-4.8] sq-dpk4.
///
/// # Deny retraction is asymmetric to grant retraction — [OPUS-4.8] sq-2pcf
///
/// A grant replays straight through its `materialize_*`: any non-Permit (withdrawn /
/// lapsed / now-Denied / ambiguous) emits nothing → the grant is dropped → access is
/// GONE. That is fail-closed for an *allow*.
///
/// A **deny** must NOT use the same rule. [`materialize_prohibition`] re-emits only
/// when [`matched_prohibition`] currently matches; on *any* non-match it emits nothing,
/// so the deny would be retracted — restoring access. But `matched_prohibition` returns
/// no-match both when the prohibition is genuinely withdrawn AND when it is merely
/// *unprovable* (a constraint with no evidence in the refresh request). Retracting in
/// the unprovable case is fail-OPEN. So for the deny side we consult
/// [`prohibition_status`] and re-emit the deny on [`ProhibitionStatus::Ambiguous`] just
/// as on [`ProhibitionStatus::Applies`]; the deny is retracted ONLY on a definite
/// [`ProhibitionStatus::Withdrawn`].
#[cfg(not(feature = "count-enforcement"))]
fn replay(graph: &mut Graph, entry: &BridgeEntry) -> BridgeOutcome {
    match entry.kind {
        BridgeKind::Permission => materialize_permission(graph, &entry.policy, &entry.request),
        BridgeKind::Prohibition => refresh_prohibition(graph, &entry.policy, &entry.request),
        BridgeKind::Policy => refresh_policy(graph, &entry.policy, &entry.request),
        BridgeKind::PermissionConditional => {
            materialize_permission_conditional(graph, &entry.policy, &entry.request)
        }
        BridgeKind::ProhibitionConditional => {
            refresh_prohibition_conditional(graph, &entry.policy, &entry.request)
        }
    }
}

/// Re-run a tracked entry's original bridge function against the current graph. The
/// `count_store` variant ([OPUS-4.8] sq-58mh): a [`BridgeKind::PermissionCounted`] entry
/// is re-checked (read-only) against the usage store via
/// [`count::refresh_permission_counted`] — exhausted ⇒ emits nothing ⇒ retracted; every
/// other kind ignores the store.
#[cfg(feature = "count-enforcement")]
fn replay(
    graph: &mut Graph,
    entry: &BridgeEntry,
    count_store: Option<&count::CounterHandle>,
) -> BridgeOutcome {
    match entry.kind {
        BridgeKind::Permission => materialize_permission(graph, &entry.policy, &entry.request),
        BridgeKind::Prohibition => refresh_prohibition(graph, &entry.policy, &entry.request),
        BridgeKind::Policy => refresh_policy(graph, &entry.policy, &entry.request),
        BridgeKind::PermissionConditional => {
            materialize_permission_conditional(graph, &entry.policy, &entry.request)
        }
        BridgeKind::ProhibitionConditional => {
            refresh_prohibition_conditional(graph, &entry.policy, &entry.request)
        }
        BridgeKind::PermissionCounted => {
            count::refresh_permission_counted(graph, &entry.policy, &entry.request, count_store)
        }
    }
}

/// Re-evaluate a tracked **conditional deny** ([`materialize_prohibition_conditional`])
/// on refresh with the fail-closed deny-retraction rule (sq-2pcf). [OPUS-4.8] sq-4r70.
///
/// A faithfully-conditional deny re-checks its recipient/assignee carve-out per session
/// at enforcement time, so on refresh the question is only whether the prohibition still
/// *structurally* names the request (action/target) — which is exactly what
/// [`materialize_prohibition_conditional`] re-checks before re-emitting. A prohibition
/// withdrawn entirely (or whose action/target no longer match) re-emits nothing → the
/// deny condition is retracted → access restored (correct: the prohibition is gone).
///
/// When the tracked deny FELL BACK to one-shot (an unmappable `dateTime`/`purpose`/
/// `count` constraint), [`materialize_prohibition_conditional`]'s fallback runs the
/// one-shot [`materialize_prohibition`] — which would retract on an *unprovable*
/// constraint, FAIL-OPEN. So for the one-shot fallback we route through the deny-
/// retraction-aware [`refresh_prohibition`] (re-emit on Ambiguous, retract only on a
/// definite Withdrawn), exactly as the plain [`BridgeKind::Prohibition`] refresh does.
/// We detect the fallback case by whether ANY prohibition maps faithfully for the
/// request's action/target.
fn refresh_prohibition_conditional(
    graph: &mut Graph,
    policy: &Policy,
    request: &Request,
) -> BridgeOutcome {
    if prohibition_maps_faithfully(policy, request) {
        // Faithful conditional deny: the recipient carve-out is re-checked per session,
        // so re-emitting whenever the prohibition structurally names the request is
        // correct (and fail-closed: a withdrawn prohibition emits nothing → retracted).
        materialize_prohibition_conditional(graph, policy, request)
    } else {
        // One-shot fallback (an unmappable constraint): apply the deny-retraction rule
        // so an unprovable bound KEEPS the deny rather than restoring access (fail-open).
        refresh_prohibition(graph, policy, request)
    }
}

/// Whether SOME prohibition in `policy` whose action/target structurally name `request`
/// maps faithfully to agent conditions (so the conditional-deny path would emit a
/// re-checked condition rather than fall back to one-shot). [OPUS-4.8] sq-4r70.
fn prohibition_maps_faithfully(policy: &Policy, request: &Request) -> bool {
    let Some(mode) = action_to_mode(&request.action) else { return false };
    let Some(target) = request.target.as_deref() else { return false };
    policy.prohibitions.iter().any(|rule| {
        rule_action_target_match(rule, request, mode, target)
            && matches!(map_constraints_to_agents(rule), AgentMapping::Faithful { .. })
    })
}

/// Re-evaluate a tracked **Prohibition** deny on refresh with the fail-closed
/// deny-retraction rule (sq-2pcf): re-emit the `auth:deny*` triple unless the
/// prohibition is *definitely* withdrawn. [OPUS-4.8] sq-2pcf.
///
/// - [`ProhibitionStatus::Applies`] — a prohibition still carves the request out:
///   re-emit (identical to [`materialize_prohibition`]).
/// - [`ProhibitionStatus::Ambiguous`] — a prohibition still structurally names the
///   request but a constraint is unprovable for lack of evidence: **re-emit anyway**
///   (KEEP the deny — never restore access on an unprovable carve-out).
/// - [`ProhibitionStatus::Withdrawn`] — no prohibition names the request, or every one
///   that does is *definitely* false given the evidence: emit nothing → the deny is
///   retracted and access is restored (subject to deny-overrides composition).
fn refresh_prohibition(graph: &mut Graph, policy: &Policy, request: &Request) -> BridgeOutcome {
    match prohibition_status(policy, request) {
        // Genuinely gone → behave exactly like the materialize path (which now also
        // finds no match), emitting nothing so the deny is dropped.
        ProhibitionStatus::Withdrawn => materialize_prohibition(graph, policy, request),
        // Still applies → re-emit through the normal match path.
        ProhibitionStatus::Applies => materialize_prohibition(graph, policy, request),
        // Unprovable → KEEP the deny by re-emitting it directly (fail-closed).
        ProhibitionStatus::Ambiguous => reemit_deny(graph, request),
    }
}

/// Re-evaluate a tracked **Policy** (both sides) on refresh: the allow side keeps the
/// fail-closed grant semantics ([`materialize_permission`]); the deny side uses the
/// fail-closed deny-retraction rule ([`refresh_prohibition`]). Composed exactly as
/// [`materialize_policy`] so deny-overrides still holds. [OPUS-4.8] sq-2pcf.
fn refresh_policy(graph: &mut Graph, policy: &Policy, request: &Request) -> BridgeOutcome {
    let allow = materialize_permission(graph, policy, request);
    let deny = refresh_prohibition(graph, policy, request);

    let mut reasons = allow.reasons;
    reasons.extend(deny.reasons);
    let mut emitted = allow.emitted;
    emitted.extend(deny.emitted);
    BridgeOutcome {
        granted: allow.granted,
        prohibited: deny.prohibited,
        refused: allow.refused || deny.refused,
        mode: deny.mode.or(allow.mode),
        grant_triple: allow.grant_triple,
        deny_triple: deny.deny_triple,
        reasons,
        // The policy-refresh path is not the stateful-count path; no unit consumed.
        consumed: None,
        emitted,
    }
}

/// Re-emit the `principal auth:deny<Mode> target` deny for `request` WITHOUT consulting
/// the prohibition match — used by [`refresh_prohibition`] on an *ambiguous* re-eval to
/// KEEP a deny we cannot prove is gone (fail-closed). The deny triple is fully
/// determined by the request (party + action→mode + target), the same one
/// [`materialize_prohibition`] would emit when the prohibition matched. [OPUS-4.8] sq-2pcf.
///
/// If the request lacks a mappable action or a concrete party/target the deny cannot be
/// reconstructed; that emits nothing (and so retracts) — but such an entry could never
/// have been materialized in the first place (those are the exact fail-closed gates in
/// [`materialize_prohibition`]), so this branch is unreachable for a tracked deny.
fn reemit_deny(graph: &mut Graph, request: &Request) -> BridgeOutcome {
    let Some(mode) = action_to_mode(&request.action) else {
        return BridgeOutcome::denied(vec![
            "ambiguous prohibition re-eval but action no longer maps; deny not re-emitted"
                .to_owned(),
        ]);
    };
    let (Some(party), Some(target)) = (request.party.as_deref(), request.target.as_deref()) else {
        return BridgeOutcome::denied(vec![
            "ambiguous prohibition re-eval but request lost its party/target; deny not re-emitted"
                .to_owned(),
        ]);
    };
    let pred = format!("{AUTH_NS}{}", deny_predicate(mode));
    let triple = append_grant(graph, party, &pred, target);
    BridgeOutcome {
        prohibited: true,
        mode: Some(mode),
        deny_triple: Some((party.to_owned(), pred, target.to_owned())),
        emitted: vec![triple],
        ..BridgeOutcome::default()
    }
}

// ============================================================================
// [FABLE-5] sq-zgbso.2 — the ODRL STATELESS core evaluated as N3 rule strata
// (`rules/odrl-core-{a,b,c,d}.n3`) through the SAME runtime `reason_n3` engine
// WAC/ACP use, differential-locked against the Rust path (epic sq-zgbso, #1582;
// design record `research/odrl-n3-compiled-rules.md`).
//
// OPT-IN, NON-DEFAULT: nothing in the crate calls this path — the Rust evaluator
// ([`materialize_policy`] over [`sparq_policy::evaluate`]) remains THE default. This
// entry point exists to prove decision parity at corpus scale
// (`tests/odrl_n3_differential.rs`); flipping any default is a later maintainer
// decision (and the build-time-compiled flip is sq-zgbso.5, gated on sq-zgbso.3/.4).
//
// # The stateless N3 subset (fail-closed contract)
//
// [`materialize_policy_n3`] evaluates exactly the subset both paths PROVABLY agree
// on, and returns a loud `Err` — materializing NOTHING — for anything outside it:
//
// - **Rules:** Permission/Prohibition with action (incl. the `odrl:use` umbrella
//   against the transfer subtree), exact-IRI target/assignee (or unconstrained),
//   duties (discharge-by-action), and deny-overrides.
// - **Constraints:** `odrl:dateTime` under `lt/lteq/gt/gteq/eq/neq` with the
//   guarded lexical space below; every other dimension under `eq/neq/isA/isPartOf`
//   with IRI/string operands, compared by the same `Value::as_str()` lexical the
//   evaluator uses. Compound `odrl:and`/`or`/`xone` with ATOMIC operands.
// - **Outside the subset (⇒ `Err`):** nested compound constraints, numeric
//   operands/evidence (`odrl:count` is STATEFUL and stays Rust — mutation is not
//   inference), order operators on non-dateTime dimensions, `isA`/`isPartOf` on
//   `odrl:dateTime`, and non-admissible dateTime lexicals.
// - **Impossible by construction:** [`N3Request`] carries no party/asset-collection
//   membership and no purpose/spatial subsumption closure, so exact-IRI matching IS
//   the Rust base case (with such evidence absent the two paths are byte-identical);
//   a caller needing taxonomy evidence must use the Rust path.
//
// # dateTime normalization (spike sq-zgbso.1 finding (a), RESOLVED)
//
// The spike compared `odrl:dateTime` LEXICALLY (`string:notGreaterThan`), which is
// instant-correct only for canonical UTC forms. The production strata instead
// normalize BOTH operands to epoch seconds with the engine's offset-aware
// `time:inSeconds` builtin and compare numerically — mixed `±hh:mm` offsets now
// order by the INSTANT they denote, exactly like `sparq_policy`'s `parse_instant`.
// Residually, the two normalizers differ on sub-second precision (Rust keeps
// nanoseconds; `time:inSeconds` truncates), leap seconds (Rust clamps `:60`), and
// exotic years — so the accepted lexical space is additionally FAIL-CLOSED to
// `YYYY-MM-DD` / `YYYY-MM-DDThh:mm:ss` with an optional `Z`/`±hh:mm` offset, no
// fractional seconds, no leap second, no negative or ≥5-digit year (see
// `n3_datetime_admissible`). Anything else is a loud `Err`, never a silent
// divergence window.
//
// # Stratification (spike finding (c) + design record §7)
//
// The engine's `log:notIncludes` is negation-as-failure WITHOUT retraction, so every
// negated predicate must be COMPLETE before its stratum runs (the WAC/ACP §3.5
// lesson). The spike's single stratum could not carry constraint-bearing
// prohibitions; the production rules run FOUR strata:
//
//   A  structural legs + atomic constraint statuses (sat / unprovable / notSat) +
//      duty blocking — negation over INPUT facts only;
//   B  `or`/`xone` "no operand satisfied" — negation over A-complete `oc:sat`;
//   C  rule matching — negation over B-complete `oc:blocked`;
//   D  decision triples — deny from matched prohibitions; allow negating
//      C-complete prohibition matches (deny-overrides) + A-complete duty blocks.
// ============================================================================

/// Stratum A of the ODRL-as-N3 rule set: structural match legs + atomic constraint
/// statuses (negation over input facts only). [FABLE-5] sq-zgbso.2.
const ODRL_CORE_A: &str = include_str!("../rules/odrl-core-a.n3");
/// Stratum B: `or`/`xone` completion over stratum A's satisfaction facts.
const ODRL_CORE_B: &str = include_str!("../rules/odrl-core-b.n3");
/// Stratum C: rule matching over stratum B's completed blocking facts.
const ODRL_CORE_C: &str = include_str!("../rules/odrl-core-c.n3");
/// Stratum D: auth-view decision triples with deny-overrides.
const ODRL_CORE_D: &str = include_str!("../rules/odrl-core-d.n3");

/// The `oc:` helper vocabulary namespace the ODRL-as-N3 fact schema + rule strata use.
const OC_NS: &str = "https://sparq.dev/ns/odrl-core#";
/// Synthetic node IRI prefix for the serialized policy/request facts (rule nodes,
/// constraint nodes, evidence nodes — stable, collision-free with real IRIs because
/// the reserved `urn:sparq:` prefix is rejected in user-supplied principals).
const OC_NODE: &str = "urn:sparq:odrl-n3#";
const XSD_DATETIME_IRI: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

/// An evaluation request for the **N3 path** ([`materialize_policy_n3`]) — the
/// stateless-subset counterpart of [`sparq_policy::Request`]. [FABLE-5] sq-zgbso.2.
///
/// By construction this type carries only evidence the N3 strata model faithfully:
/// action / target / party, the evaluation time ([`at`](N3Request::at)), per-dimension
/// IRI/string context evidence, and discharged duty actions. It deliberately has **no**
/// party/asset-collection membership and **no** purpose/spatial subsumption closure —
/// with that evidence absent, `sparq_policy`'s matching is exactly the exact-IRI base
/// case the rules implement, so the two paths cannot silently diverge on it. Convert
/// with [`to_request`](N3Request::to_request) to run the SAME request through the Rust
/// evaluator (that is what the differential suite does).
///
/// # Examples
///
/// ```
/// use sparq_solid::odrl_bridge::N3Request;
/// let req = N3Request::new("http://www.w3.org/ns/odrl/2/read")
///     .on("https://pod.ex/notes/n1")
///     .by("https://alice.ex/card#me")
///     .at("2026-07-01T00:00:00Z");
/// let rust_req = req.to_request();
/// assert_eq!(rust_req.action, "http://www.w3.org/ns/odrl/2/read");
/// assert_eq!(rust_req.target.as_deref(), Some("https://pod.ex/notes/n1"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct N3Request {
    /// The requested `odrl:` action IRI.
    pub action: String,
    /// The target asset/graph IRI, if any (no target ⇒ nothing can materialize).
    pub target: Option<String>,
    /// The requesting party (WebID), if any. Doubles as the default
    /// `odrl:recipient` evidence, mirroring [`Request::by`].
    pub party: Option<String>,
    /// The evaluation-time evidence (`odrl:dateTime`), as an admissible
    /// `xsd:dateTime`/`xsd:date` lexical (see the module notes on the accepted
    /// lexical space). `None` ⇒ time-gated rules are unprovable (fail-closed).
    pub at: Option<String>,
    /// Context evidence keyed by `leftOperand` IRI — [`Value::Iri`]/[`Value::Str`]
    /// only (numeric/dateTime evidence is outside the subset; the evaluation time
    /// goes in [`at`](N3Request::at)).
    pub evidence: BTreeMap<String, Value>,
    /// Duty action IRIs the caller asserts discharged.
    pub discharged: BTreeSet<String>,
}

impl N3Request {
    /// A request for `action` with no target/party/evidence yet.
    pub fn new(action: impl Into<String>) -> N3Request {
        N3Request { action: action.into(), ..N3Request::default() }
    }

    /// Set the target asset/graph IRI (chainable).
    pub fn on(mut self, target: impl Into<String>) -> N3Request {
        self.target = Some(target.into());
        self
    }

    /// Set the requesting party (WebID) IRI (chainable).
    pub fn by(mut self, party: impl Into<String>) -> N3Request {
        self.party = Some(party.into());
        self
    }

    /// Set the evaluation-time evidence (chainable) — the `odrl:dateTime` value
    /// time-window constraints are checked against, mirroring [`Request::at`].
    pub fn at(mut self, instant: impl Into<String>) -> N3Request {
        self.at = Some(instant.into());
        self
    }

    /// Add IRI/string context evidence for a `leftOperand` dimension (chainable).
    pub fn with(mut self, left_operand: impl Into<String>, value: Value) -> N3Request {
        self.evidence.insert(left_operand.into(), value);
        self
    }

    /// Mark a duty action IRI as discharged (chainable).
    pub fn discharge(mut self, duty_action: impl Into<String>) -> N3Request {
        self.discharged.insert(duty_action.into());
        self
    }

    /// The equivalent [`sparq_policy::Request`], for running the SAME request
    /// through the Rust evaluator (the differential's other leg).
    pub fn to_request(&self) -> Request {
        let mut r = Request::new(self.action.clone());
        if let Some(t) = &self.target {
            r = r.on(t.clone());
        }
        if let Some(p) = &self.party {
            r = r.by(p.clone());
        }
        if let Some(a) = &self.at {
            r = r.at(a.clone());
        }
        for (k, v) in &self.evidence {
            r = r.with(k.clone(), v.clone());
        }
        for d in &self.discharged {
            r = r.discharge(d.clone());
        }
        r
    }
}

/// Whether a dateTime lexical is inside the FAIL-CLOSED space on which the N3
/// (`time:inSeconds`, second precision) and Rust (`parse_instant`, nanosecond
/// precision) normalizers provably agree: `YYYY-MM-DD(Thh:mm:ss(Z|±hh:mm)?)?` with
/// in-range fields, no fractional seconds, no leap second (`:60`), no negative or
/// ≥5-digit year. [FABLE-5] sq-zgbso.2 (spike finding (a)).
fn n3_datetime_admissible(s: &str) -> bool {
    let b = s.as_bytes();
    let all_digits = |r: &[u8]| r.iter().all(u8::is_ascii_digit);
    let num2 = |r: &[u8]| -> u32 { (u32::from(r[0]) - 48) * 10 + (u32::from(r[1]) - 48) };
    // date: YYYY-MM-DD with MM 01–12, DD 01–31
    if b.len() < 10
        || !all_digits(&b[0..4])
        || b[4] != b'-'
        || !all_digits(&b[5..7])
        || b[7] != b'-'
        || !all_digits(&b[8..10])
    {
        return false;
    }
    if !(1..=12).contains(&num2(&b[5..7])) || !(1..=31).contains(&num2(&b[8..10])) {
        return false;
    }
    if b.len() == 10 {
        return true; // bare xsd:date (midnight UTC on both paths)
    }
    // time: Thh:mm:ss with hh ≤ 23, mm/ss ≤ 59 (no leap second, no fraction)
    if b.len() < 19
        || b[10] != b'T'
        || !all_digits(&b[11..13])
        || b[13] != b':'
        || !all_digits(&b[14..16])
        || b[16] != b':'
        || !all_digits(&b[17..19])
        || num2(&b[11..13]) > 23
        || num2(&b[14..16]) > 59
        || num2(&b[17..19]) > 59
    {
        return false;
    }
    match &b[19..] {
        [] => true,     // no timezone: UTC on both paths
        [b'Z'] => true, // canonical UTC
        // ±hh:mm offset with hh ≤ 14, mm ≤ 59 (the XSD offset range)
        [sign, h1, h0, b':', m1, m0]
            if matches!(sign, b'+' | b'-')
                && all_digits(&[*h1, *h0])
                && all_digits(&[*m1, *m0]) =>
        {
            num2(&[*h1, *h0]) <= 14 && num2(&[*m1, *m0]) <= 59
        }
        _ => false,
    }
}

/// Whether `s` is safe to emit as an N3 IRI (`<…>`): non-empty and free of the
/// characters N-Triples forbids inside an IRIREF. Fail-closed: an unserializable IRI
/// is a loud error, never a mangled fact.
fn n3_iri_ok(s: &str) -> bool {
    !s.is_empty()
        && !s
            .chars()
            .any(|c| c <= ' ' || matches!(c, '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\'))
}

/// Whether `s` is safe to emit as a (quoted, escaped) N3 string literal: no raw
/// control characters other than tab/newline/CR (which `n3_escape` escapes).
fn n3_lit_ok(s: &str) -> bool {
    !s.chars().any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
}

/// Escape a string for a quoted N3 literal.
fn n3_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// The `oc:` operator IRI a [`sparq_policy::Operator`] serializes as.
fn oc_op_iri(op: Operator) -> String {
    let local = match op {
        Operator::Eq => "eq",
        Operator::Neq => "neq",
        Operator::Lt => "lt",
        Operator::Lteq => "lteq",
        Operator::Gt => "gt",
        Operator::Gteq => "gteq",
        Operator::IsPartOf => "isPartOf",
        Operator::IsA => "isA",
    };
    format!("{OC_NS}{local}")
}

/// Check one atomic constraint against the N3 subset (see the module notes):
/// `odrl:dateTime` gets the order/equality operators with an admissible
/// `xsd:dateTime` bound; every other dimension gets `eq/neq/isA/isPartOf` with an
/// IRI/string bound. Anything else is a loud `Err` (fail-closed).
fn n3_constraint_admissible(c: &Constraint) -> Result<(), String> {
    if !n3_iri_ok(&c.left) {
        return Err(format!("N3 path: constraint leftOperand {:?} is not a serializable IRI", c.left));
    }
    if c.left == ODRL_DATETIME {
        if !matches!(
            c.operator,
            Operator::Lt | Operator::Lteq | Operator::Gt | Operator::Gteq | Operator::Eq | Operator::Neq
        ) {
            return Err(format!(
                "N3 path: operator {:?} on odrl:dateTime is outside the stateless N3 subset",
                c.operator
            ));
        }
        match &c.right {
            Value::DateTime(s) if n3_datetime_admissible(s) => Ok(()),
            Value::DateTime(s) => Err(format!(
                "N3 path: odrl:dateTime bound {s:?} is outside the admissible lexical space \
                 (YYYY-MM-DD[Thh:mm:ss[Z|±hh:mm]], no fractional seconds/leap second/negative year) \
                 — refusing fail-closed rather than risk instant-order divergence"
            )),
            other => Err(format!(
                "N3 path: odrl:dateTime bound must be an xsd:dateTime/xsd:date literal, got {other}"
            )),
        }
    } else {
        if !matches!(c.operator, Operator::Eq | Operator::Neq | Operator::IsA | Operator::IsPartOf) {
            return Err(format!(
                "N3 path: order operator {:?} on non-dateTime dimension <{}> is outside the \
                 stateless N3 subset",
                c.operator, c.left
            ));
        }
        match &c.right {
            Value::Iri(s) if n3_iri_ok(s) => Ok(()),
            Value::Str(s) if n3_lit_ok(s) => Ok(()),
            Value::Iri(s) => Err(format!("N3 path: rightOperand IRI {s:?} is not serializable")),
            Value::Str(s) => Err(format!("N3 path: rightOperand string {s:?} is not serializable")),
            Value::Num(_) => Err(format!(
                "N3 path: numeric rightOperand on <{}> is outside the stateless N3 subset \
                 (odrl:count and numeric bounds stay on the Rust path)",
                c.left
            )),
            Value::DateTime(_) => Err(format!(
                "N3 path: dateTime rightOperand on non-dateTime dimension <{}> is outside the \
                 stateless N3 subset",
                c.left
            )),
        }
    }
}

/// Check a whole policy against the N3 subset: every rule's direct constraints
/// admissible, every compound constraint one level deep with atomic operands.
fn n3_policy_admissible(policy: &Policy) -> Result<(), String> {
    for rule in policy.permissions.iter().chain(policy.prohibitions.iter()) {
        if !n3_iri_ok(&rule.action.0) {
            return Err(format!("N3 path: rule action {:?} is not a serializable IRI", rule.action.0));
        }
        for v in [&rule.target, &rule.assignee].into_iter().flatten() {
            if !n3_iri_ok(v) {
                return Err(format!("N3 path: rule target/assignee {v:?} is not a serializable IRI"));
            }
        }
        for d in &rule.duties {
            if !n3_iri_ok(&d.action.0) {
                return Err(format!("N3 path: duty action {:?} is not a serializable IRI", d.action.0));
            }
        }
        for c in &rule.constraints {
            n3_constraint_admissible(c)?;
        }
        for lc in &rule.logical_constraints {
            for operand in &lc.operands {
                match operand {
                    ConstraintNode::Atomic(c) => n3_constraint_admissible(c)?,
                    ConstraintNode::Compound(_) => {
                        return Err(format!(
                            "N3 path: nested LogicalConstraint {} is outside the stateless N3 \
                             subset (one compound level with atomic operands is supported)",
                            lc.id
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Check an [`N3Request`] against the N3 subset: serializable IRIs, an admissible
/// evaluation-time lexical, IRI/string evidence only (and the evaluation time only
/// via [`N3Request::at`]).
fn n3_request_admissible(request: &N3Request) -> Result<(), String> {
    if !n3_iri_ok(&request.action) {
        return Err(format!("N3 path: request action {:?} is not a serializable IRI", request.action));
    }
    for v in [&request.target, &request.party].into_iter().flatten() {
        if !n3_iri_ok(v) {
            return Err(format!("N3 path: request target/party {v:?} is not a serializable IRI"));
        }
    }
    if let Some(at) = &request.at {
        if !n3_datetime_admissible(at) {
            return Err(format!(
                "N3 path: evaluation time {at:?} is outside the admissible lexical space \
                 (YYYY-MM-DD[Thh:mm:ss[Z|±hh:mm]], no fractional seconds/leap second/negative year)"
            ));
        }
    }
    for (left, v) in &request.evidence {
        if left == ODRL_DATETIME {
            return Err(
                "N3 path: supply the evaluation time via N3Request::at, not the evidence map".to_owned()
            );
        }
        if !n3_iri_ok(left) {
            return Err(format!("N3 path: evidence dimension {left:?} is not a serializable IRI"));
        }
        match v {
            Value::Iri(s) if n3_iri_ok(s) => {}
            Value::Str(s) if n3_lit_ok(s) => {}
            other => {
                return Err(format!(
                    "N3 path: evidence for <{left}> must be a serializable IRI/string, got {other} \
                     (numeric/dateTime evidence is outside the stateless N3 subset)"
                ));
            }
        }
    }
    for d in &request.discharged {
        if !n3_iri_ok(d) {
            return Err(format!("N3 path: discharged duty action {d:?} is not a serializable IRI"));
        }
    }
    Ok(())
}

/// Serialize one admissible atomic constraint node into `oc:` facts. The bound is
/// pre-lexicalized exactly as the Rust evaluator compares it (`Value::as_str()` for
/// `eq/neq/isA`, the [`is_part_of`]-identical member split for `isPartOf`, the raw
/// `xsd:dateTime` lexical for the dateTime dimension), so the rules and the evaluator
/// share ONE source of value truth.
///
/// [`is_part_of`]: sparq_policy::Operator::IsPartOf
fn n3_emit_constraint(out: &mut String, node: &str, c: &Constraint) {
    use std::fmt::Write as _;
    let _ = writeln!(out, "<{node}> <{RDF_TYPE}> <{OC_NS}Atomic> .");
    let _ = writeln!(out, "<{node}> <{OC_NS}left> <{}> .", c.left);
    let _ = writeln!(out, "<{node}> <{OC_NS}op> <{}> .", oc_op_iri(c.operator));
    if c.left == ODRL_DATETIME {
        let _ = writeln!(out, "<{node}> <{OC_NS}boundDt> \"{}\"^^<{XSD_DATETIME_IRI}> .", c.right.as_str());
    } else if c.operator == Operator::IsPartOf {
        // The same member split `sparq_policy`'s `is_part_of` applies.
        for member in c.right.as_str().split(['|', ' ', ',']).map(str::trim).filter(|s| !s.is_empty()) {
            let _ = writeln!(out, "<{node}> <{OC_NS}member> \"{}\" .", n3_escape(member));
        }
    } else {
        let _ = writeln!(out, "<{node}> <{OC_NS}boundLex> \"{}\" .", n3_escape(c.right.as_str()));
    }
}

/// Serialize an admissible `(policy, request)` pair into the `oc:` fact schema the
/// ODRL rule strata consume. Serialization works from the PARSED model — the same
/// [`Policy`] the Rust path evaluates — so the two paths consume identical semantic
/// content (the WAC/ACP `assemble_input` discipline).
fn n3_serialize(policy: &Policy, request: &N3Request) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();

    let kinds: [(&str, &[Rule], &str); 2] = [
        ("perm", &policy.permissions, "Permission"),
        ("proh", &policy.prohibitions, "Prohibition"),
    ];
    for (tag, rules, class) in kinds {
        for (i, rule) in rules.iter().enumerate() {
            let rn = format!("{OC_NODE}{tag}{i}");
            let _ = writeln!(out, "<{rn}> <{RDF_TYPE}> <{OC_NS}{class}> .");
            let _ = writeln!(out, "<{rn}> <{RDF_TYPE}> <{OC_NS}Rule> .");
            let _ = writeln!(out, "<{rn}> <{OC_NS}action> <{}> .", rule.action.0);
            match &rule.target {
                Some(t) => {
                    let _ = writeln!(out, "<{rn}> <{OC_NS}target> <{t}> .");
                }
                None => {
                    let _ = writeln!(out, "<{rn}> <{OC_NS}anyTarget> true .");
                }
            }
            match &rule.assignee {
                Some(a) => {
                    let _ = writeln!(out, "<{rn}> <{OC_NS}assignee> <{a}> .");
                }
                None => {
                    let _ = writeln!(out, "<{rn}> <{OC_NS}anyAssignee> true .");
                }
            }
            for d in &rule.duties {
                let _ = writeln!(out, "<{rn}> <{OC_NS}dutyAction> <{}> .", d.action.0);
            }
            for (j, c) in rule.constraints.iter().enumerate() {
                let cn = format!("{rn}-c{j}");
                let _ = writeln!(out, "<{rn}> <{OC_NS}constraint> <{cn}> .");
                n3_emit_constraint(&mut out, &cn, c);
            }
            for (j, lc) in rule.logical_constraints.iter().enumerate() {
                let ln = format!("{rn}-l{j}");
                let _ = writeln!(out, "<{rn}> <{OC_NS}logical> <{ln}> .");
                let combinator = match lc.operator {
                    LogicalOperator::And => "and",
                    LogicalOperator::Or => "or",
                    LogicalOperator::Xone => "xone",
                };
                let _ = writeln!(out, "<{ln}> <{OC_NS}combinator> <{OC_NS}{combinator}> .");
                if lc.operands.is_empty() {
                    let _ = writeln!(out, "<{ln}> <{OC_NS}emptyOperands> true .");
                }
                for (k, operand) in lc.operands.iter().enumerate() {
                    // admissibility guaranteed atomic operands only
                    if let ConstraintNode::Atomic(c) = operand {
                        let on = format!("{ln}-o{k}");
                        let _ = writeln!(out, "<{ln}> <{OC_NS}operand> <{on}> .");
                        n3_emit_constraint(&mut out, &on, c);
                    }
                }
            }
        }
    }

    // The request + its evidence (the state-of-the-world the constraints test).
    let rq = format!("{OC_NODE}req");
    let _ = writeln!(out, "<{rq}> <{RDF_TYPE}> <{OC_NS}Request> .");
    let _ = writeln!(out, "<{rq}> <{OC_NS}reqAction> <{}> .", request.action);
    if let Some(t) = &request.target {
        let _ = writeln!(out, "<{rq}> <{OC_NS}reqTarget> <{t}> .");
    }
    if let Some(p) = &request.party {
        let _ = writeln!(out, "<{rq}> <{OC_NS}reqParty> <{p}> .");
    }
    if let Some(at) = &request.at {
        let _ = writeln!(out, "<{rq}> <{OC_NS}atTime> \"{at}\"^^<{XSD_DATETIME_IRI}> .");
        // evidence marker: the odrl:dateTime dimension is provable
        let _ = writeln!(out, "<{OC_NODE}ev-dt> <{OC_NS}evLeft> <{ODRL_DATETIME}> .");
    }
    for (i, (left, v)) in request.evidence.iter().enumerate() {
        let en = format!("{OC_NODE}ev{i}");
        let _ = writeln!(out, "<{en}> <{OC_NS}evLeft> <{left}> .");
        let _ = writeln!(out, "<{en}> <{OC_NS}evLex> \"{}\" .", n3_escape(v.as_str()));
    }
    // The recipient-of-data defaults to the requesting party (resolve_actual's
    // `odrl:recipient` fallback) unless explicit recipient evidence was supplied.
    if !request.evidence.contains_key(ODRL_RECIPIENT) {
        if let Some(p) = &request.party {
            let en = format!("{OC_NODE}ev-recipient");
            let _ = writeln!(out, "<{en}> <{OC_NS}evLeft> <{ODRL_RECIPIENT}> .");
            let _ = writeln!(out, "<{en}> <{OC_NS}evLex> \"{}\" .", n3_escape(p));
        }
    }
    for d in &request.discharged {
        let _ = writeln!(out, "<{rq}> <{OC_NS}discharged> <{d}> .");
    }
    out
}

/// Re-serialize a stratum's ground closure as facts for the next stratum (the
/// `materialize_acp` inter-stratum seeding shape).
fn n3_closure_facts(dict: &Dict, closure: &[[sparq_core::dict::Id; 3]]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(closure.len() * 64);
    for t in closure {
        let _ = writeln!(out, "{} {} {} .", dict.term(t[0]), dict.term(t[1]), dict.term(t[2]));
    }
    out
}

/// Evaluate `policy` against `request` **as N3 rule strata** (the WAC/ACP runtime
/// `reason_n3` pattern) and materialize the derived auth-view triples — the OPT-IN
/// N3 counterpart of [`materialize_policy`], differential-locked to it by
/// `tests/odrl_n3_differential.rs`. [FABLE-5] sq-zgbso.2 (epic sq-zgbso, #1582).
///
/// The Rust path remains the default evaluator; nothing routes through this
/// function unless a caller explicitly opts in. On the stateless subset (see the
/// module notes) the derived triple set is EXACTLY the set the Rust bridge writes:
/// allow grants from matched, duty-discharged, un-prohibited permissions and
/// `auth:deny<Mode>` triples from matched prohibitions (deny-overrides). Stateful
/// `odrl:count` enforcement stays on the Rust path (mutation is not inference).
///
/// # Errors / fail-closed
///
/// - A policy whose `odrl:conflict` strategy the bridge cannot honour returns the
///   same loud *refusal* outcome as [`materialize_policy`] (nothing materialized).
/// - A policy/request outside the stateless N3 subset — a non-admissible dateTime
///   lexical, numeric operands, nested compounds, order operators on non-dateTime
///   dimensions, unserializable IRIs — returns `Err` and materializes NOTHING:
///   loud, never a silent divergence window.
/// - Within the subset the strata are themselves fail-closed: missing evidence,
///   unmapped actions, or a missing party/target derive no triple.
///
/// # Examples
///
/// ```
/// use sparq_solid::odrl_bridge::{materialize_policy_n3, N3Request};
/// use sparq_policy::parse_policy_str;
///
/// let pol = parse_policy_str(r#"
/// @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
/// <urn:pol/1> a odrl:Set ; odrl:permission [
///     odrl:action odrl:read ;
///     odrl:target <https://pod.ex/notes/n1> ;
///     odrl:assignee <https://alice.ex/card#me> ] .
/// "#, "turtle")?;
/// let req = N3Request::new("http://www.w3.org/ns/odrl/2/read")
///     .on("https://pod.ex/notes/n1")
///     .by("https://alice.ex/card#me");
///
/// let mut graph = sparq_core::Graph::new();
/// let out = materialize_policy_n3(&mut graph, &pol, &req)?;
/// assert!(out.granted);
/// # Ok::<(), String>(())
/// ```
pub fn materialize_policy_n3(
    graph: &mut Graph,
    policy: &Policy,
    request: &N3Request,
) -> Result<BridgeOutcome, String> {
    // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy FIRST — the
    //    same guard, and the same refusal outcome, as the Rust path.
    if let Some(refusal) = refuse_unimplementable_conflict(policy) {
        return Ok(refusal);
    }
    // 1. Subset admissibility — loud errors, nothing materialized.
    n3_policy_admissible(policy)?;
    n3_request_admissible(request)?;

    // 2. The four stratified reason_n3 runs (§3.5 discipline: each stratum's negated
    //    predicates are complete in the seed it receives).
    let facts = n3_serialize(policy, request);
    let mut d1 = Dict::new();
    let c1 = sparq_reason::reason_n3(&mut d1, &format!("{facts}\n{ODRL_CORE_A}"))?;
    let f1 = n3_closure_facts(&d1, &c1);
    let mut d2 = Dict::new();
    let c2 = sparq_reason::reason_n3(&mut d2, &format!("{f1}\n{ODRL_CORE_B}"))?;
    let f2 = n3_closure_facts(&d2, &c2);
    let mut d3 = Dict::new();
    let c3 = sparq_reason::reason_n3(&mut d3, &format!("{f2}\n{ODRL_CORE_C}"))?;
    let f3 = n3_closure_facts(&d3, &c3);
    let mut d4 = Dict::new();
    let c4 = sparq_reason::reason_n3(&mut d4, &format!("{f3}\n{ODRL_CORE_D}"))?;

    // 3. Extract the derived auth-view triples (`auth:*` predicate, IRI subject/object).
    let mut triples: Vec<(String, String, String)> = Vec::new();
    for t in &c4 {
        let Term::NamedNode(p) = d4.term(t[1]) else { continue };
        if !p.as_str().starts_with(AUTH_NS) {
            continue;
        }
        let (Term::NamedNode(s), Term::NamedNode(o)) = (d4.term(t[0]), d4.term(t[2])) else {
            continue;
        };
        triples.push((s.as_str().to_owned(), p.as_str().to_owned(), o.as_str().to_owned()));
    }
    triples.sort();
    triples.dedup();

    // 4. Materialize into the auth view + bridged-provenance graph (the same append
    //    path the Rust bridge uses) and report the outcome in the same shape.
    let emitted: Vec<[Term; 3]> = triples
        .iter()
        .map(|(s, p, o)| {
            [
                Term::NamedNode(NamedNode::new_unchecked(s)),
                Term::NamedNode(NamedNode::new_unchecked(p)),
                Term::NamedNode(NamedNode::new_unchecked(o)),
            ]
        })
        .collect();
    append_bridged_triples(graph, &emitted);

    let is_deny = |p: &str| p.strip_prefix(AUTH_NS).is_some_and(|l| l.starts_with("deny"));
    let grant_triple = triples.iter().find(|(_, p, _)| !is_deny(p)).cloned();
    let deny_triple = triples.iter().find(|(_, p, _)| is_deny(p)).cloned();
    let granted = grant_triple.is_some();
    let prohibited = deny_triple.is_some();
    let reasons = if granted || prohibited {
        Vec::new()
    } else {
        vec!["N3 strata derived no auth-view triple for the request (fail-closed)".to_owned()]
    };
    Ok(BridgeOutcome {
        granted,
        prohibited,
        // Mirrors materialize_policy: the mode of the operative decision (both sides
        // derive it from the REQUEST action, so one lookup serves either).
        mode: (granted || prohibited).then(|| action_to_mode(&request.action)).flatten(),
        grant_triple,
        deny_triple,
        refused: false,
        reasons,
        consumed: None,
        emitted,
    })
}

#[cfg(test)]
mod odrl_n3_tests {
    //! [FABLE-5] sq-zgbso.2 — direct unit tests for the N3-path public surface
    //! (one per new public item, per the coverage-ratchet discipline) plus the
    //! fail-closed guard edges. The corpus-scale Rust-vs-N3 differential lives in
    //! `tests/odrl_n3_differential.rs`.
    use super::*;
    use sparq_policy::parse_policy_str;

    const READ: &str = "http://www.w3.org/ns/odrl/2/read";

    #[test]
    fn n3request_builders_and_to_request_round_trip() {
        let req = N3Request::new(READ)
            .on("urn:t/1")
            .by("urn:alice")
            .at("2026-07-01T00:00:00Z")
            .with("http://www.w3.org/ns/odrl/2/purpose", Value::Iri("urn:purpose/r".into()))
            .discharge("http://www.w3.org/ns/odrl/2/anonymize");
        let r = req.to_request();
        assert_eq!(r.action, READ);
        assert_eq!(r.target.as_deref(), Some("urn:t/1"));
        assert_eq!(r.party.as_deref(), Some("urn:alice"));
        assert_eq!(
            r.request_time(),
            Some(&Value::DateTime("2026-07-01T00:00:00Z".into()))
        );
        assert!(r.discharged_duties.contains("http://www.w3.org/ns/odrl/2/anonymize"));
        assert_eq!(
            r.context.get("http://www.w3.org/ns/odrl/2/purpose"),
            Some(&Value::Iri("urn:purpose/r".into()))
        );
    }

    #[test]
    fn datetime_admissible_accepts_agreeing_forms() {
        for ok in [
            "2026-07-01T00:00:00Z",
            "2026-07-01T23:59:59+14:00",
            "2026-07-01T12:30:00-05:30",
            "2026-07-01T12:30:00", // no tz = UTC on both paths
            "2026-02-28",          // bare date = midnight UTC on both paths
        ] {
            assert!(n3_datetime_admissible(ok), "{ok} should be admissible");
        }
    }

    #[test]
    fn datetime_admissible_rejects_divergence_windows() {
        for bad in [
            "2026-07-01T00:00:00.5Z",  // fractional seconds: Rust keeps, N3 truncates
            "2026-06-30T23:59:60Z",    // leap second: Rust clamps, N3 adds
            "-0044-03-15T00:00:00Z",   // negative year
            "12026-01-01T00:00:00Z",   // ≥5-digit year
            "2026-7-1T00:00:00Z",      // non-fixed-width
            "2026-13-01T00:00:00Z",    // month out of range
            "2026-07-01T24:00:00Z",    // hour out of range
            "2026-07-01T00:00:00+15:00", // offset beyond the XSD range
            "not-a-date",
            "",
        ] {
            assert!(!n3_datetime_admissible(bad), "{bad} should be rejected");
        }
    }

    #[test]
    fn materialize_policy_n3_grants_and_appends_auth_view() {
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            <urn:pol/1> a odrl:Set ; odrl:permission [
                odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#,
            "turtle",
        )
        .expect("parses");
        let mut graph = Graph::new();
        let out = materialize_policy_n3(&mut graph, &pol, &N3Request::new(READ).on("urn:t/1").by("urn:alice"))
            .expect("in subset");
        assert!(out.granted && !out.prohibited && !out.refused);
        assert_eq!(
            out.grant_triple,
            Some(("urn:alice".into(), format!("{AUTH_NS}read"), "urn:t/1".into()))
        );
        assert_eq!(out.mode, Some(Mode::Read));
        // the triple landed in the auth view through the same append path
        assert!(named_graph_triples(&graph, AUTH_GRAPH)
            .iter()
            .any(|t| matches!(&t[1], Term::NamedNode(p) if p.as_str() == format!("{AUTH_NS}read"))));
    }

    #[test]
    fn materialize_policy_n3_deny_overrides() {
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            <urn:pol/1> a odrl:Set ;
              odrl:permission [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ;
              odrl:prohibition [ odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#,
            "turtle",
        )
        .expect("parses");
        let mut graph = Graph::new();
        let out = materialize_policy_n3(&mut graph, &pol, &N3Request::new(READ).on("urn:t/1").by("urn:alice"))
            .expect("in subset");
        assert!(!out.granted, "deny-overrides: the matching prohibition blocks the grant");
        assert!(out.prohibited);
        assert_eq!(
            out.deny_triple,
            Some(("urn:alice".into(), format!("{AUTH_NS}denyRead"), "urn:t/1".into()))
        );
    }

    #[test]
    fn materialize_policy_n3_mirrors_conflict_refusal() {
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            <urn:pol/1> a odrl:Set ; odrl:conflict odrl:perm ; odrl:permission [
                odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#,
            "turtle",
        )
        .expect("parses");
        let mut graph = Graph::new();
        let out = materialize_policy_n3(&mut graph, &pol, &N3Request::new(READ).on("urn:t/1").by("urn:alice"))
            .expect("refusal is an outcome, not an Err");
        assert!(out.refused && !out.granted && !out.prohibited);
        assert!(named_graph_triples(&graph, AUTH_GRAPH).is_empty(), "refusal materializes nothing");
    }

    #[test]
    fn materialize_policy_n3_errs_loudly_outside_the_subset() {
        let mut graph = Graph::new();
        // fractional-second dateTime bound → divergence window → Err
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            <urn:pol/1> a odrl:Set ; odrl:permission [
                odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
                odrl:constraint [ odrl:leftOperand odrl:dateTime ; odrl:operator odrl:lteq ;
                                  odrl:rightOperand "2026-12-31T00:00:00.5Z"^^xsd:dateTime ] ] ."#,
            "turtle",
        )
        .expect("parses");
        let req = N3Request::new(READ).on("urn:t/1").by("urn:alice").at("2026-07-01T00:00:00Z");
        let err = materialize_policy_n3(&mut graph, &pol, &req).expect_err("outside the subset");
        assert!(err.contains("lexical space"), "clear reason, got: {err}");

        // numeric bound (odrl:count) → Err (stateful count stays Rust)
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            <urn:pol/2> a odrl:Set ; odrl:permission [
                odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ;
                odrl:constraint [ odrl:leftOperand odrl:count ; odrl:operator odrl:lteq ;
                                  odrl:rightOperand 5 ] ] ."#,
            "turtle",
        )
        .expect("parses");
        let err = materialize_policy_n3(&mut graph, &pol, &N3Request::new(READ).on("urn:t/1").by("urn:alice"))
            .expect_err("numeric bound outside the subset");
        assert!(err.contains("subset"), "clear reason, got: {err}");
        // numeric evidence → Err
        let req = N3Request::new(READ)
            .on("urn:t/1")
            .by("urn:alice")
            .with("http://www.w3.org/ns/odrl/2/count", Value::Num(3.0));
        let pol = parse_policy_str(
            r#"@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
            <urn:pol/3> a odrl:Set ; odrl:permission [
                odrl:action odrl:read ; odrl:target <urn:t/1> ; odrl:assignee <urn:alice> ] ."#,
            "turtle",
        )
        .expect("parses");
        let err = materialize_policy_n3(&mut graph, &pol, &req).expect_err("numeric evidence");
        assert!(err.contains("subset"), "clear reason, got: {err}");
        // dateTime evidence must go through `at`
        let req = N3Request::new(READ)
            .on("urn:t/1")
            .by("urn:alice")
            .with(super::ODRL_DATETIME, Value::Str("2026-07-01T00:00:00Z".into()));
        let err = materialize_policy_n3(&mut graph, &pol, &req).expect_err("evidence-map time");
        assert!(err.contains("N3Request::at"), "clear reason, got: {err}");
        assert!(named_graph_triples(&graph, AUTH_GRAPH).is_empty(), "errors materialize nothing");
    }

    #[test]
    fn n3_escape_and_serializability_guards() {
        assert_eq!(n3_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
        assert!(n3_iri_ok("https://alice.ex/card#me"));
        assert!(!n3_iri_ok("urn:sp ace"));
        assert!(!n3_iri_ok("urn:angle>bracket"));
        assert!(!n3_iri_ok(""));
        assert!(n3_lit_ok("plain value"));
        assert!(!n3_lit_ok("nul\u{0}byte"));
    }
}

#[cfg(test)]
mod set_tighter_tests {
    //! [OPUS-4.8] sq-0q7n — `set_tighter` keeps the instant-tightest window bound when
    //! two same-side `odrl:dateTime` constraints intersect. The pre-fix lexical `<`/`>`
    //! picked the wrong bound for mixed timezone offsets (a fail-open: a wider persisted
    //! window than the constraints allow). These pin the offset-aware behavior.
    use super::set_tighter;

    #[test]
    fn upper_bound_keeps_earlier_instant_across_offsets() {
        // Two notAfter bounds: 12:00Z (= 12:00Z) and 13:00+02:00 (= 11:00Z). The EARLIER
        // instant is the offset form (11:00Z); lexically "12…Z" < "13…+02:00" so the OLD
        // code wrongly kept 12:00Z (later instant → wider window).
        let mut slot = Some("2026-06-16T12:00:00Z".to_owned());
        set_tighter(&mut slot, "2026-06-16T13:00:00+02:00", /*keep_earlier=*/ true);
        assert_eq!(slot.as_deref(), Some("2026-06-16T13:00:00+02:00"), "kept earlier instant");
    }

    #[test]
    fn lower_bound_keeps_later_instant_across_offsets() {
        // Two notBefore bounds: 12:00Z and 09:00-02:00 (= 11:00Z). The LATER instant is
        // 12:00Z; lexically "09…-02:00" < "12…Z" — verify the tighter (later) is kept.
        let mut slot = Some("2026-06-16T12:00:00Z".to_owned());
        set_tighter(&mut slot, "2026-06-16T09:00:00-02:00", /*keep_earlier=*/ false);
        assert_eq!(slot.as_deref(), Some("2026-06-16T12:00:00Z"), "kept later instant");
        // And a genuinely-later offset bound DOES replace.
        set_tighter(&mut slot, "2026-06-16T16:00:00+02:00", /*keep_earlier=*/ false); // = 14:00Z
        assert_eq!(slot.as_deref(), Some("2026-06-16T16:00:00+02:00"), "later instant wins");
    }

    #[test]
    fn unparseable_candidate_never_widens() {
        let mut slot = Some("2026-06-16T12:00:00Z".to_owned());
        set_tighter(&mut slot, "not-a-date", true);
        assert_eq!(slot.as_deref(), Some("2026-06-16T12:00:00Z"), "malformed never replaces");
    }
}

// ============================================================================
// [OPUS-4.8] sq-58mh — STATEFUL `odrl:count` enforcement wired THROUGH the bridge so a
// bridged grant SELF-RETRACTS on exhaustion (the sq-zi5w follow-up).
//
// ACP is stateless: there is no per-session usage counter to decrement, so the count
// limit cannot be a re-checked ACP *condition* the way `recipient`/`dateTime` are (a
// `ConditionalGrant` re-checks a STATELESS dimension of the session — agent / clock).
// A usage count lives OUTSIDE any one session. So count is wired through the EXISTING
// refresh/retraction ledger (sq-dpk4) instead:
//
//   - At EXERCISE, `materialize_permission_counted` calls sparq-policy's atomic
//     `evaluate_and_exercise`, which runs the real base decision AND consumes exactly
//     one unit of `odrl:count` budget on a grant. On a grant the equivalent one-shot
//     `principal auth:<mode> graph` allow is materialized (the same triple the existing
//     enforcement honours). On a base deny / exhausted / store-unavailable: NOTHING
//     (fail-closed — a denied request never burns budget).
//   - On REFRESH the tracked counted entry is re-checked READ-ONLY against the usage
//     store (`refresh_permission_counted` → `count::count_status`, which never consumes):
//     while budget remains the grant is re-emitted; once the budget is exhausted (or the
//     store is unavailable, or the base permission was withdrawn / now-prohibited) it
//     emits nothing → the grant is RETRACTED → access is GONE through the real
//     enforcement path. That is the "self-retracts on exhaustion" the bead asks for.
//
// FAIL-CLOSED throughout: an exhausted / unprovable / store-missing re-check retracts
// (never re-grants beyond a budget we cannot account for); refresh NEVER consumes a unit
// (so re-checking is idempotent and a refresh can never itself exhaust a budget).
// ============================================================================
#[cfg(feature = "count-enforcement")]
pub(crate) mod count {
    use super::{action_to_mode, append_grant, refuse_unimplementable_conflict, BridgeOutcome, AUTH_NS};
    use sparq_core::Graph;
    use sparq_policy::{
        count_status, evaluate, evaluate_and_exercise, CountStatus, Policy, Request,
        UsageCounterStore, ODRL_COUNT,
    };
    use std::sync::Arc;

    /// A cloneable, `Debug`-able handle to the injected usage-counter store, so the
    /// [`super::BridgeLedger`] (which derives `Debug`/`Clone`/`Default`) can hold one.
    /// [OPUS-4.8] sq-58mh.
    #[derive(Clone)]
    pub(crate) struct CounterHandle(pub(crate) Arc<dyn UsageCounterStore + Send + Sync>);

    impl std::fmt::Debug for CounterHandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // The store interior is opaque (a trait object); name the handle only.
            f.write_str("CounterHandle(<UsageCounterStore>)")
        }
    }

    /// The `auth:` allow-view predicate a mode grant is materialized under — the SAME
    /// predicate [`super::materialize_permission`] uses and [`crate::AuthIndex`] reads.
    fn mode_predicate(mode: crate::Mode) -> &'static str {
        match mode {
            crate::Mode::Read => "read",
            crate::Mode::Write => "write",
            crate::Mode::Append => "append",
            crate::Mode::Control => "control",
        }
    }

    /// Evaluate `policy` against `request` and, on a grant, **atomically consume** one
    /// unit of any applicable `odrl:count` budget from `store`, materializing the
    /// equivalent `principal auth:<mode> graph` allow into the `<urn:sparq:auth>` view.
    /// [OPUS-4.8] sq-58mh.
    ///
    /// This is the stateful-count entry point of the bridge: it routes the allow/deny
    /// decision through sparq-policy's [`evaluate_and_exercise`] (which runs the REAL
    /// base [`evaluate`] decision and consumes exactly one count unit on a grant), so the
    /// first *N* exercises of an "at most *N*" permission materialize a grant and the
    /// *(N+1)*th denies — and a denied / exhausted / store-unavailable exercise burns no
    /// budget and materializes NOTHING (fail-closed).
    ///
    /// The grant is materialized **only** when the exercise was granted AND the request
    /// action [`action_to_mode`]-maps AND the request names a concrete party + target —
    /// the SAME fail-closed gates as [`super::materialize_permission`]. The returned
    /// [`BridgeOutcome`] carries `consumed = Some(n)` on a count-constrained grant
    /// (`None` for an uncounted permission or a deny).
    pub(crate) fn materialize_permission_counted(
        graph: &mut Graph,
        policy: &Policy,
        request: &Request,
        store: &dyn UsageCounterStore,
    ) -> BridgeOutcome {
        // 0. Refuse (fail-closed) an unimplementable odrl:conflict strategy BEFORE the
        //    atomic exercise — a refused policy must consume no budget. [OPUS-4.8] sq-ihqbl.
        if let Some(refusal) = refuse_unimplementable_conflict(policy) {
            return refusal;
        }
        // 1. The atomic, count-aware decision — the single source of allow/deny AND the
        //    one place a unit is consumed. A base deny / exhausted / store-unavailable
        //    returns allow == false and consumes nothing.
        let exercise = evaluate_and_exercise(policy, request, store);
        if !exercise.allow {
            return BridgeOutcome::denied(exercise.reasons);
        }

        // 2. Same fail-closed mapping gates as the one-shot allow path.
        let Some(mode) = action_to_mode(&request.action) else {
            return BridgeOutcome::denied(vec![format!(
                "ODRL action <{}> has no WAC/ACP mode mapping; no grant materialized",
                request.action
            )]);
        };
        let Some(party) = request.party.as_deref() else {
            return BridgeOutcome::denied(vec![
                "ODRL Permit has no concrete party (assignee/WebID); no grant materialized"
                    .to_owned(),
            ]);
        };
        let Some(target) = request.target.as_deref() else {
            return BridgeOutcome::denied(vec![
                "ODRL Permit has no concrete target graph IRI; no grant materialized".to_owned(),
            ]);
        };

        // 3. Materialize `party auth:<mode> target` (one-shot allow shape; the count was
        //    consumed in step 1, and is re-checked read-only on refresh).
        let pred = format!("{AUTH_NS}{}", mode_predicate(mode));
        let triple = append_grant(graph, party, &pred, target);
        BridgeOutcome {
            granted: true,
            mode: Some(mode),
            grant_triple: Some((party.to_owned(), pred, target.to_owned())),
            consumed: exercise.consumed,
            emitted: vec![triple],
            ..BridgeOutcome::default()
        }
    }

    /// Re-check a tracked counted grant on refresh and re-emit it **iff** it still holds —
    /// WITHOUT consuming a unit (refresh is read-only; a refresh must never itself burn
    /// budget). [OPUS-4.8] sq-58mh.
    ///
    /// Fail-closed: the grant is re-emitted only when (a) a counter store is available,
    /// (b) the base permission STILL grants (re-evaluated with the `odrl:count`
    /// constraints stripped — the same shape [`evaluate_and_exercise`] uses for its base
    /// decision — so a withdrawn permission / now-matching prohibition denies and the
    /// grant is retracted), and (c) the granting rule's count budget is **not** exhausted
    /// ([`CountStatus::Satisfied`]/[`NotConstrained`](CountStatus::NotConstrained)).
    /// An exhausted ([`CountStatus::DefinitelyUnsatisfied`]), unprovable
    /// ([`CountStatus::Unprovable`] — store outage / malformed limit), or store-missing
    /// re-check emits NOTHING → the grant is retracted → access is GONE. This is what
    /// makes a bridged grant self-retract on exhaustion.
    pub(crate) fn refresh_permission_counted(
        graph: &mut Graph,
        policy: &Policy,
        request: &Request,
        store: Option<&CounterHandle>,
    ) -> BridgeOutcome {
        // No store ⇒ a counted entry cannot be accounted for → fail-closed retract.
        let Some(handle) = store else {
            return BridgeOutcome::denied(vec![
                "counted grant has no usage-counter store on refresh; retracted (fail-closed)"
                    .to_owned(),
            ]);
        };
        let store: &dyn UsageCounterStore = handle.0.as_ref();

        // (b) Does the BASE permission still grant? Re-evaluate with `odrl:count`
        //     stripped from the permissions (the stateless evaluator would otherwise deny
        //     for a missing count value) — exactly the base shape `evaluate_and_exercise`
        //     uses. A withdrawn permission / now-matching prohibition denies here →
        //     retract. NOTE: this never consumes; consumption only happens at exercise.
        let stripped = strip_count_constraints(policy);
        let decision = evaluate(&stripped, request);
        if !decision.allow {
            return BridgeOutcome::denied(decision.unmet_constraints);
        }

        // Locate the granting rule in the ORIGINAL policy (which still carries the count
        // constraint) so `count_status` can read its effective limit.
        let Some(rule) = decision
            .matched_rules
            .first()
            .and_then(|id| policy.permissions.iter().find(|r| &r.id == id))
        else {
            // A base grant with no identifiable rule (should not happen for a permission)
            // → nothing to count; re-emit the plain allow.
            return super::materialize_permission(graph, &stripped, request);
        };

        // (c) Read-only count check — NEVER consumes a unit on refresh.
        match count_status(rule, request, store) {
            // Budget remains, or the rule has no count limit → re-emit the allow grant.
            CountStatus::Satisfied { .. } | CountStatus::NotConstrained => {
                reemit_grant(graph, request)
            }
            // Exhausted, or unprovable (store outage / malformed) → retract (fail-closed).
            CountStatus::DefinitelyUnsatisfied { consumed, limit } => BridgeOutcome::denied(vec![
                format!(
                    "permission {} count budget exhausted ({consumed} >= {limit}); grant retracted",
                    rule.id
                ),
            ]),
            CountStatus::Unprovable => BridgeOutcome::denied(vec![format!(
                "permission {} count state unprovable on refresh; grant retracted (fail-closed)",
                rule.id
            )]),
        }
    }

    /// Re-emit the `principal auth:<mode> target` allow for `request` (the same triple
    /// [`materialize_permission_counted`] emitted), used on a still-valid count refresh.
    /// The triple is fully determined by the request (party + action→mode + target).
    fn reemit_grant(graph: &mut Graph, request: &Request) -> BridgeOutcome {
        let Some(mode) = action_to_mode(&request.action) else {
            return BridgeOutcome::denied(vec![
                "counted grant action no longer maps on refresh; not re-emitted".to_owned(),
            ]);
        };
        let (Some(party), Some(target)) = (request.party.as_deref(), request.target.as_deref())
        else {
            return BridgeOutcome::denied(vec![
                "counted grant lost its party/target on refresh; not re-emitted".to_owned(),
            ]);
        };
        let pred = format!("{AUTH_NS}{}", mode_predicate(mode));
        let triple = append_grant(graph, party, &pred, target);
        BridgeOutcome {
            granted: true,
            mode: Some(mode),
            grant_triple: Some((party.to_owned(), pred, target.to_owned())),
            emitted: vec![triple],
            ..BridgeOutcome::default()
        }
    }

    /// A copy of `policy` with every `odrl:count` constraint removed from its PERMISSION
    /// rules — the base shape the stateless [`evaluate`] sees (count is enforced against
    /// the store, not as a stateless numeric comparison). Mirrors sparq-policy's internal
    /// `strip_count_constraints` (kept here because that helper is crate-private).
    /// Prohibitions are untouched (a count on a prohibition keeps its stateless meaning).
    fn strip_count_constraints(policy: &Policy) -> Policy {
        let mut out = policy.clone();
        for rule in &mut out.permissions {
            rule.constraints.retain(|c| c.left != ODRL_COUNT);
        }
        out
    }
}
