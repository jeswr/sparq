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
//! `use` subsumes every action in the ODRL hierarchy, so materializing it as a
//! single WAC mode would have to pick the widest, violating fail-closed. A caller
//! that wants `use → Read` should request `odrl:read` explicitly (a `use` permission
//! in the policy still *grants* a `read` request — `odrl:use` permits any action in
//! the evaluator — so the bridge maps the **request** action, which is concrete).
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

use crate::authindex::Mode;
use crate::{AUTH_GRAPH, AUTH_NS};
use oxrdf::{NamedNode, Term};
use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_policy::{evaluate, matched_prohibition, Policy, Request};

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
    /// Human-readable reason a grant/deny was NOT materialized (the ODRL decision's
    /// caveats, an unmapped action, or a missing party/target). Empty on success.
    pub reasons: Vec<String>,
}

impl BridgeOutcome {
    fn denied(reasons: Vec<String>) -> BridgeOutcome {
        BridgeOutcome { reasons, ..BridgeOutcome::default() }
    }
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
    append_grant(graph, party, &pred, target);

    BridgeOutcome {
        granted: true,
        mode: Some(mode),
        grant_triple: Some((party.to_owned(), pred, target.to_owned())),
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
    append_grant(graph, party, &pred, target);

    BridgeOutcome {
        prohibited: true,
        mode: Some(mode),
        deny_triple: Some((party.to_owned(), pred, target.to_owned())),
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
    let allow = materialize_permission(graph, policy, request);
    let deny = materialize_prohibition(graph, policy, request);

    let mut reasons = allow.reasons;
    reasons.extend(deny.reasons);
    BridgeOutcome {
        granted: allow.granted,
        prohibited: deny.prohibited,
        // Under deny-overrides the deny is the operative decision when present.
        mode: deny.mode.or(allow.mode),
        grant_triple: allow.grant_triple,
        deny_triple: deny.deny_triple,
        reasons,
    }
}

/// Append a single `subject predicate object` triple to the `<urn:sparq:auth>`
/// named graph, preserving the triples already there (the WAC/ACP grants). Rebuilds
/// the auth sub-graph from its existing triples + the new one via [`Graph::from_parts`].
fn append_grant(graph: &mut Graph, subject: &str, predicate: &str, object: &str) {
    let s = Term::NamedNode(NamedNode::new_unchecked(subject));
    let p = Term::NamedNode(NamedNode::new_unchecked(predicate));
    let o = Term::NamedNode(NamedNode::new_unchecked(object));
    let auth_name = Term::NamedNode(NamedNode::new_unchecked(AUTH_GRAPH));

    // Collect the existing auth-view triples (if any) as terms, add the grant, then
    // re-intern into a fresh sub-graph dictionary (matches install_auth_view).
    let mut terms: Vec<[Term; 3]> = match graph.named.iter().find(|(n, _)| *n == auth_name) {
        Some((_, sub)) => crate::loader::graph_triples(sub),
        None => Vec::new(),
    };
    // Idempotent: don't duplicate an identical grant triple.
    let new_triple = [s, p, o];
    if !terms.contains(&new_triple) {
        terms.push(new_triple);
    }

    let mut dict = Dict::new();
    let ids: Vec<[sparq_core::dict::Id; 3]> = terms
        .iter()
        .map(|t| [dict.intern(&t[0]), dict.intern(&t[1]), dict.intern(&t[2])])
        .collect();
    let auth = Graph::from_parts(dict, ids);

    if let Some(slot) = graph.named.iter_mut().find(|(n, _)| *n == auth_name) {
        slot.1 = auth;
    } else {
        graph.named.push((auth_name, auth));
    }
}
