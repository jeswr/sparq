//! The typed, panic-free **stubs** for the deferred seam phases (design record §4.3 / §8).
//!
//! Phase 1 (this bead, sq-2q1x) lands the *shape* of the untrusted-planner → MPC-routing
//! seam — real input/output types wired to the actual upstream crates — but **none of the
//! privacy-bearing logic**. Each deferred phase is a function with its real signature that
//! returns `Err(`[`SeamError::Deferred`]`)` naming the phase and the gate it waits on. The
//! functions **compile and are callable**, perform NO MPC, reveal NOTHING, and never panic
//! (no `todo!()` / `unimplemented!()`): a caller that invokes one gets an honest typed
//! "deferred" error, not a crash and not a fabricated result.
//!
//! The phases (from the design record's phased plan):
//! * [`select_private_sources`] — Phase 2: wrap [`sparq_fedplan::select_sources`] and prune by
//!   the per-source [`crate::SourcePrivacyDescriptor`] (participation / authorisation).
//! * [`route_operators`] — Phase 3: the policy-parameterised disclosed/hidden partition,
//!   emitting the [`sparq_mpc::pipeline::OperatorRouting`] vector the pipeline consumes.
//! * [`assemble_leakage_envelope`] — Phase 4: collect what the chosen routing reveals into a
//!   declared envelope for the holder/verifier dual ratification.
//!
//! NONE of these makes a soundness/privacy claim; the privacy-bearing work is deferred and
//! audit-gated (sq-9hrn / sq-qhy4). See the crate `README.md` and
//! `research/mpc-untrusted-planner-routing-design.md`.
//!
//! [OPUS-4.8] sq-2q1x.

use sparq_fedplan::{Bgp, SourceDescriptor};
use sparq_mpc::pipeline::OperatorRouting;

use crate::SourcePrivacyDescriptor;

/// Which deferred seam phase a [`SeamError::Deferred`] came from. Naming the phase in the
/// error value (not just a doc-comment) keeps the deferral auditable at the call site —
/// the same honesty discipline as `sparq-mpc`'s `MpcError::NotYetImplemented`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeamPhase {
    /// Phase 2 — privacy/authorisation-aware source selection ([`select_private_sources`]).
    SourceSelection,
    /// Phase 3 — the disclosed/hidden routing partition ([`route_operators`]).
    Routing,
    /// Phase 4 — the leakage-envelope assembly + dual ratification ([`assemble_leakage_envelope`]).
    LeakageEnvelope,
}

impl SeamPhase {
    /// A short label for the phase (for the error `Display`).
    pub fn label(self) -> &'static str {
        match self {
            SeamPhase::SourceSelection => "privacy-aware source selection (Phase 2)",
            SeamPhase::Routing => "disclosed/hidden routing partition (Phase 3)",
            SeamPhase::LeakageEnvelope => "leakage-envelope assembly + ratification (Phase 4)",
        }
    }
}

/// The seam's error type. Phase 1 has exactly one variant — [`SeamError::Deferred`] — the
/// honest typed channel every deferred-phase stub returns. It carries the [`SeamPhase`] and
/// the `gated_on` issues/audits, so a caller learns WHY the phase is unavailable rather than
/// hitting a panic or a fabricated result. (The enum is `#[non_exhaustive]` so the
/// privacy-bearing phases can add real error variants without a breaking change.)
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeamError {
    /// A deferred seam phase was invoked. `phase` names which one; `gated_on` names the
    /// milestone(s) / audit(s) that must land first. This is NOT a fabricated answer and NOT
    /// a crash — it is an honest "not yet built" the caller can match on.
    Deferred {
        /// Which deferred phase was invoked.
        phase: SeamPhase,
        /// The gate(s) the phase waits on (issues / audits), as a human-readable string.
        gated_on: &'static str,
    },
}

impl SeamError {
    /// Construct a [`SeamError::Deferred`] for `phase`.
    fn deferred(phase: SeamPhase, gated_on: &'static str) -> SeamError {
        SeamError::Deferred { phase, gated_on }
    }
}

impl std::fmt::Display for SeamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeamError::Deferred { phase, gated_on } => write!(
                f,
                "sparq-fedplan-mpc seam phase deferred: {} (gated on {})",
                phase.label(),
                gated_on
            ),
        }
    }
}

impl std::error::Error for SeamError {}

/// The output of the (deferred) privacy-aware source-selection phase: per-pattern candidate
/// sources, restricted by what each source is willing/authorised to expose. A typed
/// PLACEHOLDER for Phase 2 — its internal shape is intentionally minimal (the phase that
/// builds it will fill it from [`sparq_fedplan::PatternSources`]); it carries no data in
/// Phase 1 and is never produced (the stub returns [`SeamError::Deferred`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SelectedPrivateSources {}

/// The output of the (deferred) routing phase: the per-operator disclosed-vs-hidden routing
/// the `sparq-mpc` pipeline consumes. A typed PLACEHOLDER for Phase 3 — it wraps the real
/// [`sparq_mpc::pipeline::OperatorRouting`] vector so the output type is the genuine seam
/// shape, but the routing decision itself is deferred (the stub returns
/// [`SeamError::Deferred`], so `routing` is only ever the empty vector here).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PrivateRouting {
    /// The per-operator routing the `sparq-mpc` pipeline accepts. Empty in Phase 1 (never
    /// produced — the routing decision is deferred).
    pub routing: Vec<OperatorRouting>,
}

/// The output of the (deferred) leakage-envelope phase: the declared set of facts the chosen
/// routing reveals, for the holder/verifier dual ratification. A typed PLACEHOLDER for Phase 4
/// — its internal fields are filled by the phase that assembles it; it carries no data in
/// Phase 1 and is never produced (the stub returns [`SeamError::Deferred`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LeakageEnvelope {}

/// **Phase 2 (deferred)** — privacy/authorisation-aware source selection.
///
/// The built phase will run [`sparq_fedplan::select_sources`] over `bgp` + `sources` and then
/// prune the per-pattern candidates by each source's [`SourcePrivacyDescriptor`] (dropping a
/// source that will not participate or is not authorised). **This skeleton performs none of
/// that** — it returns [`SeamError::Deferred`]. It runs NO MPC and reveals NOTHING.
///
/// The arguments are taken (and named with `_`) so the seam's real input shape is fixed now;
/// the signature will not change when the phase is implemented.
pub fn select_private_sources(
    _bgp: &Bgp,
    _sources: &[SourceDescriptor],
    _privacy: &[SourcePrivacyDescriptor],
) -> Result<SelectedPrivateSources, SeamError> {
    Err(SeamError::deferred(
        SeamPhase::SourceSelection,
        "sq-2q1x Phase 2 (source-selection adapter); see research/mpc-untrusted-planner-routing-design.md §4.3",
    ))
}

/// **Phase 3 (deferred)** — the disclosed/hidden routing partition.
///
/// The built phase will, per operator, route `Disclosed` when every operand is a
/// source-declared-public term ([`SourcePrivacyDescriptor::may_disclose`]) and
/// `Hidden(class)` otherwise — emitting the [`sparq_mpc::pipeline::OperatorRouting`] vector
/// the pipeline consumes. **This skeleton performs none of that** — it returns
/// [`SeamError::Deferred`]. It runs NO MPC, makes NO disclosure decision, and reveals NOTHING.
pub fn route_operators(
    _selected: &SelectedPrivateSources,
    _privacy: &[SourcePrivacyDescriptor],
) -> Result<PrivateRouting, SeamError> {
    Err(SeamError::deferred(
        SeamPhase::Routing,
        "sq-2q1x Phase 3 (disclosed/hidden routing pass); §4.3 pass 2",
    ))
}

/// **Phase 4 (deferred)** — leakage-envelope assembly + dual ratification.
///
/// The built phase will collect what every `Disclosed` route reveals into a declared envelope
/// each holder re-checks (fail-closed) against its private-column policy and the verifier
/// accepts against its acceptance policy. **This skeleton performs none of that** — it returns
/// [`SeamError::Deferred`]. It runs NO MPC and reveals NOTHING. The privacy-bearing
/// ratification is audit-gated (sq-9hrn / sq-qhy4).
pub fn assemble_leakage_envelope(
    _routing: &PrivateRouting,
    _privacy: &[SourcePrivacyDescriptor],
) -> Result<LeakageEnvelope, SeamError> {
    Err(SeamError::deferred(
        SeamPhase::LeakageEnvelope,
        "sq-2q1x Phase 4 (leakage envelope + dual ratification); audit-gated sq-9hrn / sq-qhy4",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_fedplan::SourceId;

    fn descriptor() -> SourceDescriptor {
        SourceDescriptor::builder(SourceId::new("http://s/")).build()
    }
    fn privacy() -> SourcePrivacyDescriptor {
        SourcePrivacyDescriptor::deny_all(SourceId::new("http://s/"))
    }

    #[test]
    fn source_selection_is_deferred_not_a_panic() {
        let bgp = Bgp::default();
        let err = select_private_sources(&bgp, &[descriptor()], &[privacy()]).unwrap_err();
        assert!(matches!(
            err,
            SeamError::Deferred {
                phase: SeamPhase::SourceSelection,
                ..
            }
        ));
        // The error names its gate (honest deferral, not a crash).
        assert!(format!("{}", err).contains("source selection"));
    }

    #[test]
    fn routing_is_deferred_not_a_panic() {
        let err = route_operators(&SelectedPrivateSources::default(), &[privacy()]).unwrap_err();
        assert!(matches!(
            err,
            SeamError::Deferred {
                phase: SeamPhase::Routing,
                ..
            }
        ));
        // The placeholder output type wraps the REAL pipeline routing vector (empty here).
        assert!(PrivateRouting::default().routing.is_empty());
    }

    #[test]
    fn leakage_envelope_is_deferred_not_a_panic() {
        let err =
            assemble_leakage_envelope(&PrivateRouting::default(), &[privacy()]).unwrap_err();
        match err {
            SeamError::Deferred { phase, gated_on } => {
                assert_eq!(phase, SeamPhase::LeakageEnvelope);
                // The leakage-envelope phase names the external audit gates.
                assert!(gated_on.contains("sq-9hrn"));
                assert!(gated_on.contains("sq-qhy4"));
            }
        }
    }

    #[test]
    fn phase_labels_are_distinct() {
        let labels = [
            SeamPhase::SourceSelection.label(),
            SeamPhase::Routing.label(),
            SeamPhase::LeakageEnvelope.label(),
        ];
        // All three deferred phases carry a distinct human label.
        assert_eq!(labels.iter().collect::<std::collections::HashSet<_>>().len(), 3);
    }
}
