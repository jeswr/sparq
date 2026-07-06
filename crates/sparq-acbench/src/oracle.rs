//! By-construction procedural oracle (§2.3 of `research/ac-query-benchmark.md`).
//!
//! This module is the **ground truth** the system under test is judged against.
//! It is intentionally simple and must NEVER call any sparq crate — its value comes
//! from being structurally independent of the implementation under test.
//!
//! # Fail-closed invariant
//! Every public function returns [`Decision::Deny`] by default. A request is allowed
//! only when there exists a matching `Allow` intent that is not overridden by a `Deny`.
//!
//! # Determinism
//! All functions are pure (no mutation, no randomness, no time). Given the same inputs,
//! they always return the same output.
//!
//! # Completeness (B6's responsibility)
//! Full workload-engine integration (W1–W4 decision batches, result-set oracles,
//! churn delta computation) lives in `workload.rs` and `oracle.rs`'s `workload`
//! submodule (bead `sq-i6du2.6`). This file implements only the per-row evaluation
//! functions called from `lib.rs`'s public API.

use crate::{Audience, Condition, Decision, Effect, IntentRow, Request};

// ── WAC oracle ──────────────────────────────────────────────────────────────────────

/// Evaluate a request against an intent table using WAC semantics.
///
/// WAC rules:
/// - `Scope::Subtree` intents apply to the resource if the resource URI starts with
///   the intent's `resource_uri` (nearest-ancestor selection).
/// - `Condition ≠ None` intents are UNSUPPORTED in WAC and are skipped.
/// - `Effect::Deny` intents are UNSUPPORTED in WAC and are skipped.
/// - Fail-closed: absence of a matching Allow → [`Decision::Deny`].
pub(crate) fn evaluate_wac(request: &Request, intents: &[IntentRow]) -> Decision {
    use crate::Scope;

    for intent in intents {
        // WAC has no deny and no conditions — skip unsupported rows.
        if intent.effect == Effect::Deny || intent.condition != Condition::None {
            continue;
        }

        // Scope check: resource must match.
        let resource_matches = match intent.scope {
            Scope::Resource => request.resource == intent.resource_uri,
            Scope::Subtree => request.resource.starts_with(&intent.resource_uri),
        };
        if !resource_matches {
            continue;
        }

        // Mode check.
        if !mode_matches(&request.mode, &intent.mode) {
            continue;
        }

        // Audience check.
        if audience_matches_wac(request, &intent.audience) {
            return Decision::Allow;
        }
    }

    Decision::Deny // fail-closed
}

/// Check whether the request's agent/client matches a WAC audience.
fn audience_matches_wac(request: &Request, audience: &Audience) -> bool {
    match audience {
        Audience::Public => true,
        Audience::Authenticated => !request.agent.is_empty(),
        Audience::Owner => request.agent == format!("{}#owner", request.resource),
        Audience::Agent(a) => &request.agent == a,
        Audience::Group(g) => {
            // Group membership is evaluated by the generator; at oracle level,
            // the group IRI is opaque. The generator-level oracle resolves group
            // closure before calling this function (via `evaluate_wac_with_closure`).
            // For the scaffold smoke tests, treat a group match as "agent IRI contains
            // the group IRI as a prefix" — full resolution is B2–B5's responsibility.
            request.agent.starts_with(g.as_str())
        }
        Audience::ClientRestricted { agent, client } => {
            &request.agent == agent
                && request.client.as_deref() == Some(client.as_str())
        }
        Audience::AllExcept(excl) => {
            // WAC has no deny: AllExcept is approximated as "not in exclusion list".
            !excl.iter().any(|e| e == &request.agent)
        }
    }
}

// ── ACP oracle ──────────────────────────────────────────────────────────────────────

/// Evaluate a request against an intent table using ACP semantics.
///
/// ACP rules:
/// - `Effect::Deny` overrides any `Effect::Allow` (deny-wins within matching intents).
/// - `Condition ≠ None` intents are UNSUPPORTED in ACP and are skipped.
/// - `Scope::Subtree` applies to the resource if the resource URI starts with
///   the intent's `resource_uri` (cumulative-ancestor inheritance, all matching ancestors).
/// - Fail-closed: no matching Allow (or any matching Deny) → [`Decision::Deny`].
pub(crate) fn evaluate_acp(request: &Request, intents: &[IntentRow]) -> Decision {
    use crate::Scope;

    let mut any_allow = false;
    let mut any_deny = false;

    for intent in intents {
        // ACP has no usage conditions in the base spec.
        if intent.condition != Condition::None {
            continue;
        }

        let resource_matches = match intent.scope {
            Scope::Resource => request.resource == intent.resource_uri,
            Scope::Subtree => request.resource.starts_with(&intent.resource_uri),
        };
        if !resource_matches {
            continue;
        }

        if !mode_matches(&request.mode, &intent.mode) {
            continue;
        }

        if audience_matches_acp(request, &intent.audience) {
            match intent.effect {
                Effect::Allow => any_allow = true,
                Effect::Deny => any_deny = true,
            }
        }
    }

    // ACP: deny-wins. Any explicit deny overrides any allow.
    if any_deny || !any_allow {
        Decision::Deny
    } else {
        Decision::Allow
    }
}

/// Check whether the request's agent/client matches an ACP audience.
fn audience_matches_acp(request: &Request, audience: &Audience) -> bool {
    match audience {
        Audience::Public => true,
        Audience::Authenticated => !request.agent.is_empty(),
        Audience::Owner => request.agent == format!("{}#owner", request.resource),
        Audience::Agent(a) => &request.agent == a,
        Audience::Group(g) => {
            // Same placeholder as WAC oracle; full group closure resolved by generators.
            request.agent.starts_with(g.as_str())
        }
        Audience::ClientRestricted { agent, client } => {
            &request.agent == agent
                && request.client.as_deref() == Some(client.as_str())
        }
        Audience::AllExcept(excl) => {
            !excl.iter().any(|e| e == &request.agent)
        }
    }
}

// ── ODRL oracle ─────────────────────────────────────────────────────────────────────

/// Evaluate a request against an intent table using ODRL semantics.
///
/// ODRL rules:
/// - `Effect::Allow` (Permission): a matching permission grants access if all conditions
///   are satisfied.
/// - `Effect::Deny` (Prohibition): a matching prohibition denies access.
/// - If both a Permission and a Prohibition match: Permission wins (ODRL precedence).
/// - Conditions are evaluated procedurally (see [`evaluate_condition`]).
/// - Fail-closed: no matching Permission → [`Decision::Deny`].
pub(crate) fn evaluate_odrl(request: &Request, intents: &[IntentRow]) -> Decision {
    use crate::Scope;

    let mut any_permission = false;

    for intent in intents {
        let resource_matches = match intent.scope {
            Scope::Resource => request.resource == intent.resource_uri,
            Scope::Subtree => request.resource.starts_with(&intent.resource_uri),
        };
        if !resource_matches {
            continue;
        }

        if !mode_matches(&request.mode, &intent.mode) {
            continue;
        }

        if !audience_matches_odrl(request, &intent.audience) {
            continue;
        }

        if !evaluate_condition(&intent.condition) {
            continue;
        }

        if intent.effect == Effect::Allow {
            any_permission = true;
        }
    }

    // ODRL: Permission overrides Prohibition when both match.
    // Fail-closed: no Permission → Deny regardless of whether Prohibition matched.
    if any_permission {
        Decision::Allow
    } else {
        Decision::Deny
    }
}

/// Check whether the request's agent/client matches an ODRL assignee.
fn audience_matches_odrl(request: &Request, audience: &Audience) -> bool {
    match audience {
        Audience::Public => true,
        Audience::Authenticated => !request.agent.is_empty(),
        Audience::Owner => request.agent == format!("{}#owner", request.resource),
        Audience::Agent(a) => &request.agent == a,
        Audience::Group(g) => {
            // ODRL PartyCollection: group membership resolved by generators.
            request.agent.starts_with(g.as_str())
        }
        Audience::ClientRestricted { agent, client } => {
            &request.agent == agent
                && request.client.as_deref() == Some(client.as_str())
        }
        Audience::AllExcept(excl) => {
            !excl.iter().any(|e| e == &request.agent)
        }
    }
}

/// Evaluate an ODRL usage condition.
///
/// In the scaffold, conditions involving time/purpose/count are evaluated
/// against fixed test-time values. Full runtime condition evaluation with
/// real `dateTime` windows is B6's responsibility.
///
/// # Fail-closed
/// Any unrecognised or malformed condition returns `false` (deny).
pub(crate) fn evaluate_condition(condition: &Condition) -> bool {
    match condition {
        // Scaffold: None, Temporal, and Purpose all return true (always-satisfied stubs).
        // B6 supplies real clock-free temporal evaluation and purpose-matching.
        Condition::None | Condition::Temporal { .. } | Condition::Purpose(_) => true,
        Condition::Count(n) => {
            // Scaffold: count > 0 is satisfied; 0 is deny.
            *n > 0
        }
        Condition::And(a, b) => {
            evaluate_condition(a) && evaluate_condition(b)
        }
    }
}

// ── Shared helpers ───────────────────────────────────────────────────────────────────

/// Check that the requested modes are all covered by the intent's modes.
/// The intent must grant AT LEAST the requested modes.
fn mode_matches(requested: &crate::AccessMode, granted: &crate::AccessMode) -> bool {
    (!requested.read || granted.read)
        && (!requested.write || granted.write)
        && (!requested.control || granted.control)
}
