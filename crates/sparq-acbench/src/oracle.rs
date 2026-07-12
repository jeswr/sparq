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
//! # Determinism (clock-free)
//! All functions are pure (no mutation, no randomness, **no wall-clock read**). Temporal
//! ODRL conditions are decided against a *pinned* evaluation instant
//! (`BENCH_EVAL_INSTANT`) threaded through `evaluate_odrl_at`, never
//! `SystemTime::now()`, so an expired-window intent stays `Deny` no matter what day the
//! benchmark is re-run. Given the same inputs the oracle always returns the same output.
//!
//! # Completeness
//! Per-row evaluation (WAC / ACP / ODRL semantics, including the clock-free temporal +
//! purpose condition evaluation) lives here; the workload-engine integration (W1–W4
//! decision batches, result-set oracles, churn delta computation) lives in `workload.rs`.
//! Both are bead `sq-i6du2.6`'s scope. The three public wrappers in `lib.rs`
//! (`oracle_wac` / `oracle_acp` / `oracle_odrl`) call the evaluators here.

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

/// The **pinned benchmark evaluation instant** ("now") that all temporal conditions are
/// evaluated against.
///
/// The oracle must be **clock-free** (no `SystemTime::now()`) so the ground truth is
/// byte-identical on every run, platform, and date — a wall-clock read would make an
/// expired-window intent flip from `Deny` to `Allow` merely by re-running the benchmark
/// on a later day. Instead we pin a single fixed instant, in the same fixed-width UTC
/// ISO-8601 form the generators emit, and every `Condition::Temporal { start, end }` is
/// decided by the half-open membership test `start ≤ NOW < end`.
///
/// Value: `2026-07-06T00:00:00Z` — the benchmark's authoring date (design record §0). It
/// sits inside the U3/U4 "current" windows and after the expired ones, so a corpus
/// deterministically exercises BOTH the in-window (`Allow`) and expired-window (`Deny`)
/// paths. This is a design decision recorded here and in the PR body; a future EC2 tier
/// that wants to sweep the clock threads its own instant through
/// [`evaluate_odrl_at`] instead of relying on this default.
pub(crate) const BENCH_EVAL_INSTANT: &str = "2026-07-06T00:00:00Z";

/// Evaluate a request against an intent table using ODRL semantics, at the pinned
/// benchmark evaluation instant ([`BENCH_EVAL_INSTANT`]).
///
/// ODRL rules:
/// - `Effect::Allow` (Permission): a matching permission grants access if all conditions
///   are satisfied.
/// - `Effect::Deny` (Prohibition): a matching prohibition denies access.
/// - If both a Permission and a Prohibition match: Permission wins (ODRL precedence).
/// - Conditions are evaluated procedurally against `now` (see [`evaluate_condition`]).
/// - Fail-closed: no matching Permission → [`Decision::Deny`].
pub(crate) fn evaluate_odrl(request: &Request, intents: &[IntentRow]) -> Decision {
    evaluate_odrl_at(request, intents, BENCH_EVAL_INSTANT)
}

/// Evaluate a request against an intent table using ODRL semantics, at an explicit,
/// caller-supplied evaluation instant `now` (fixed-width UTC ISO-8601, e.g.
/// `"2026-07-06T00:00:00Z"`).
///
/// This is the clock-free core: `now` is threaded in, never read from the system clock,
/// so the result is a pure function of `(request, intents, now)`. [`evaluate_odrl`] is the
/// thin wrapper that pins `now` to [`BENCH_EVAL_INSTANT`]; an alternate-clock lane (e.g. a
/// U4 embargo sweep) calls this directly.
pub(crate) fn evaluate_odrl_at(request: &Request, intents: &[IntentRow], now: &str) -> Decision {
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

        if !evaluate_condition(&intent.condition, now) {
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

/// The **purpose declared by a request** for the benchmark's purpose-of-use evaluation.
///
/// A request carries no purpose field in the current [`Request`] IR (the generators do not
/// yet declare one — bead `sq-i6du2.4`/`.5`), so a `Condition::Purpose` constraint is
/// evaluated against this single pinned "authorized purpose" that the corpus grants for.
/// A purpose-constrained permission is therefore satisfied iff its purpose equals this
/// pinned value — modelling "this agent's session is running under the granted purpose".
/// Any other purpose is a deny (fail-closed), so a wrong-purpose ground truth is caught.
///
/// This is a design decision (recorded in the PR body): it keeps purpose evaluation
/// non-vacuous — a mismatched purpose denies — without inventing a `Request.purpose` field
/// that would break the sibling generator branches. When those branches add a declared
/// request purpose, this constant becomes the fixture default and the real field threads
/// through the same way `now` does for temporal windows.
pub(crate) const BENCH_GRANTED_PURPOSE: &str = "https://sparq.dev/vocab/purpose#reporting";

/// Evaluate an ODRL usage condition against a pinned evaluation instant `now`.
///
/// Clock-free by construction: `now` is a fixed-width UTC ISO-8601 string passed in by the
/// caller (see [`BENCH_EVAL_INSTANT`]); nothing here reads the system clock, so the result
/// is a pure function of `(condition, now)`.
///
/// - `Condition::None` — always satisfied.
/// - `Condition::Temporal { start, end }` — satisfied iff `start ≤ now < end` (half-open
///   window). Comparison is lexicographic, which is exact for the fixed-width `Z`-suffixed
///   UTC ISO-8601 form the generators emit (same width, same zone ⇒ lexical order equals
///   chronological order). An **expired** window (`now ≥ end`) or a **not-yet-open** one
///   (`now < start`) is therefore a deny — the correct ground truth for a retention /
///   embargo constraint.
/// - `Condition::Purpose(p)` — satisfied iff `p` equals the pinned
///   [`BENCH_GRANTED_PURPOSE`]; any other purpose denies (fail-closed).
/// - `Condition::Count(n)` — satisfied iff `n > 0` (a zero-count grant admits nothing).
/// - `Condition::And(a, b)` — conjunction; both sub-conditions must hold.
///
/// # Fail-closed
/// A malformed temporal window (a bound that is not parseable as the expected fixed-width
/// form, detected by a length check) returns `false` (deny) rather than silently admitting.
pub(crate) fn evaluate_condition(condition: &Condition, now: &str) -> bool {
    match condition {
        Condition::None => true,
        Condition::Temporal { start, end } => temporal_in_window(start, end, now),
        Condition::Purpose(p) => p == BENCH_GRANTED_PURPOSE,
        Condition::Count(n) => *n > 0,
        Condition::And(a, b) => {
            evaluate_condition(a, now) && evaluate_condition(b, now)
        }
    }
}

/// Half-open temporal-window membership test: `start ≤ now < end`.
///
/// The three arguments are all fixed-width `Z`-suffixed UTC ISO-8601 instants
/// (`YYYY-MM-DDTHH:MM:SSZ`). For that canonical form, lexicographic `str` ordering is
/// identical to chronological ordering, so no date parsing (and no timezone / DST logic,
/// and no external crate) is needed — this is what keeps the oracle simple and trustworthy.
///
/// # Fail-closed
/// If any of the three instants is not the expected 20-character fixed-width form, the
/// window is treated as unsatisfiable (`false`) — a malformed bound must never silently
/// admit access.
fn temporal_in_window(start: &str, end: &str, now: &str) -> bool {
    // Canonical form is exactly `YYYY-MM-DDTHH:MM:SSZ` = 20 chars. A differing width would
    // break the lexicographic ⇔ chronological equivalence, so reject it fail-closed.
    const CANONICAL_LEN: usize = 20;
    if start.len() != CANONICAL_LEN || end.len() != CANONICAL_LEN || now.len() != CANONICAL_LEN {
        return false;
    }
    start <= now && now < end
}

// ── Shared helpers ───────────────────────────────────────────────────────────────────

/// Check that the requested modes are all covered by the intent's modes.
/// The intent must grant AT LEAST the requested modes.
fn mode_matches(requested: &crate::AccessMode, granted: &crate::AccessMode) -> bool {
    (!requested.read || granted.read)
        && (!requested.write || granted.write)
        && (!requested.control || granted.control)
}

// ── Workload-engine model dispatch (bead `sq-i6du2.6`) ────────────────────────────────

/// Dispatch a request to the by-construction oracle for a given [`AcModel`].
///
/// This is the single entry point the workload engine (`workload.rs`, W1/W3/W4) uses to
/// obtain the ground-truth decision for a `(request, model)` pair. It is a thin router
/// over [`evaluate_wac`], [`evaluate_acp`], and [`evaluate_odrl`] so the workload engine
/// never re-implements per-model semantics (single source of truth for the oracle).
///
/// **Structural independence**: like the three underlying evaluators, this function reads
/// only the intent table — it never links or calls any sparq crate. That independence is
/// the whole point of the benchmark: an unsound system-under-test must *disagree* with
/// this oracle and thereby FAIL the harness.
///
/// **Determinism**: pure function of `(model, request, intents)`.
#[must_use]
pub(crate) fn evaluate(
    model: &crate::AcModel,
    request: &Request,
    intents: &[IntentRow],
) -> Decision {
    match model {
        crate::AcModel::Wac => evaluate_wac(request, intents),
        crate::AcModel::Acp => evaluate_acp(request, intents),
        crate::AcModel::Odrl => evaluate_odrl(request, intents),
    }
}

// ── Direct unit tests for the clock-free condition evaluator (bead `sq-i6du2.6`) ──────
//
// These pin the condition-eval semantics at the unit level (per-branch, exact value) so a
// comparison/boundary mutant in the temporal window test or a stubbed-out condition arm
// goes red WITHOUT needing a full generator corpus. The integration lane in
// `tests/workloads.rs` covers the model-differentiating dispatch; this covers the leaf
// evaluator's arms directly.
#[cfg(test)]
mod tests {
    use super::{
        BENCH_EVAL_INSTANT, BENCH_GRANTED_PURPOSE, evaluate_condition, temporal_in_window,
    };
    use crate::Condition;

    #[test]
    fn none_condition_is_always_satisfied() {
        assert!(evaluate_condition(&Condition::None, BENCH_EVAL_INSTANT));
    }

    #[test]
    fn temporal_window_half_open_boundaries() {
        // start ≤ now < end. Same-width canonical instants ⇒ lexical == chronological.
        assert!(temporal_in_window(
            "2026-01-01T00:00:00Z",
            "2027-01-01T00:00:00Z",
            "2026-07-06T00:00:00Z"
        ));
        // now == start is IN (inclusive lower bound).
        assert!(temporal_in_window(
            "2026-07-06T00:00:00Z",
            "2027-01-01T00:00:00Z",
            "2026-07-06T00:00:00Z"
        ));
        // now == end is OUT (exclusive upper bound).
        assert!(!temporal_in_window(
            "2026-01-01T00:00:00Z",
            "2026-07-06T00:00:00Z",
            "2026-07-06T00:00:00Z"
        ));
        // expired (now ≥ end).
        assert!(!temporal_in_window(
            "2020-01-01T00:00:00Z",
            "2021-01-01T00:00:00Z",
            "2026-07-06T00:00:00Z"
        ));
        // not yet open (now < start).
        assert!(!temporal_in_window(
            "2099-01-01T00:00:00Z",
            "2100-01-01T00:00:00Z",
            "2026-07-06T00:00:00Z"
        ));
    }

    #[test]
    fn temporal_malformed_bound_fails_closed() {
        // A non-canonical width breaks the lexical⇔chronological equivalence ⇒ deny.
        assert!(!temporal_in_window("2026", "2027-01-01T00:00:00Z", BENCH_EVAL_INSTANT));
        assert!(!temporal_in_window(
            "2026-01-01T00:00:00Z",
            "not-a-date",
            BENCH_EVAL_INSTANT
        ));
        assert!(!temporal_in_window(
            "2026-01-01T00:00:00Z",
            "2027-01-01T00:00:00Z",
            "now"
        ));
    }

    #[test]
    fn temporal_condition_uses_pinned_instant() {
        let valid = Condition::Temporal {
            start: "2026-01-01T00:00:00Z".to_string(),
            end: "2027-01-01T00:00:00Z".to_string(),
        };
        let expired = Condition::Temporal {
            start: "2020-01-01T00:00:00Z".to_string(),
            end: "2021-01-01T00:00:00Z".to_string(),
        };
        assert!(evaluate_condition(&valid, BENCH_EVAL_INSTANT));
        assert!(!evaluate_condition(&expired, BENCH_EVAL_INSTANT));
    }

    #[test]
    fn purpose_condition_matches_only_granted_purpose() {
        assert!(evaluate_condition(
            &Condition::Purpose(BENCH_GRANTED_PURPOSE.to_string()),
            BENCH_EVAL_INSTANT
        ));
        assert!(!evaluate_condition(
            &Condition::Purpose("https://sparq.dev/vocab/purpose#marketing".to_string()),
            BENCH_EVAL_INSTANT
        ));
    }

    #[test]
    fn count_condition_zero_denies_positive_admits() {
        assert!(!evaluate_condition(&Condition::Count(0), BENCH_EVAL_INSTANT));
        assert!(evaluate_condition(&Condition::Count(1), BENCH_EVAL_INSTANT));
    }

    #[test]
    fn and_condition_is_conjunction() {
        let both_true = Condition::And(
            Box::new(Condition::Purpose(BENCH_GRANTED_PURPOSE.to_string())),
            Box::new(Condition::Temporal {
                start: "2026-01-01T00:00:00Z".to_string(),
                end: "2027-01-01T00:00:00Z".to_string(),
            }),
        );
        assert!(evaluate_condition(&both_true, BENCH_EVAL_INSTANT));

        // One expired arm makes the conjunction false.
        let one_expired = Condition::And(
            Box::new(Condition::Purpose(BENCH_GRANTED_PURPOSE.to_string())),
            Box::new(Condition::Temporal {
                start: "2020-01-01T00:00:00Z".to_string(),
                end: "2021-01-01T00:00:00Z".to_string(),
            }),
        );
        assert!(!evaluate_condition(&one_expired, BENCH_EVAL_INSTANT));

        // Wrong purpose also makes it false.
        let wrong_purpose = Condition::And(
            Box::new(Condition::Purpose("https://sparq.dev/vocab/purpose#other".to_string())),
            Box::new(Condition::Temporal {
                start: "2026-01-01T00:00:00Z".to_string(),
                end: "2027-01-01T00:00:00Z".to_string(),
            }),
        );
        assert!(!evaluate_condition(&wrong_purpose, BENCH_EVAL_INSTANT));
    }
}
