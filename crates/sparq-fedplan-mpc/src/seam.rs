//! The seam's shared error/phase types and the **still-deferred** phase stubs (design record
//! §4.3 / §8).
//!
//! Phase 1 (sq-2q1x) landed the *shape* of the untrusted-planner → MPC-routing seam — real
//! input/output types wired to the actual upstream crates — with **none of the privacy-bearing
//! logic**. Phase 2 (sq-fix4) makes the **source-selection adapter** real (it lives in
//! [`crate::selection`]). Phase 3 (sq-i1wh2) makes the **disclosed/hidden routing pass** real
//! (it lives in [`crate::routing`]). Phase 4 stays a deferred stub here: a function with its real
//! signature returning `Err(`[`SeamError::Deferred`]`)` naming the phase and the gate it waits on.
//! The stub **compiles and is callable**, performs NO MPC, reveals NOTHING, and never panics (no
//! `todo!()` / `unimplemented!()`): a caller gets an honest typed "deferred" error, not a crash
//! and not a fabricated result.
//!
//! The phases (from the design record's phased plan):
//! * [`select_private_sources`](crate::select_private_sources) — Phase 2 (**implemented**, sq-fix4):
//!   wraps [`sparq_fedplan::select_sources`] and prunes by the per-source
//!   [`crate::SourcePrivacyDescriptor`] (participation / authorisation). See [`crate::selection`].
//! * [`route_operators`](crate::route_operators) — Phase 3 (**implemented**, sq-i1wh2): the
//!   policy-parameterised disclosed/hidden partition, emitting the
//!   [`sparq_mpc::pipeline::OperatorRouting`] vector the pipeline consumes. See [`crate::routing`].
//! * [`assemble_leakage_envelope`] — Phase 4 (deferred): collect what the chosen routing reveals
//!   into a declared envelope for the holder/verifier dual ratification.
//!
//! NONE of these makes a soundness/privacy claim; the privacy-bearing work is deferred and
//! audit-gated (sq-9hrn / sq-qhy4). See the crate `README.md` and
//! `research/mpc-untrusted-planner-routing-design.md`.
//!
//! [OPUS-4.8] sq-2q1x / sq-fix4 / sq-i1wh2.

use sparq_mpc::pipeline::OperatorRouting;

use crate::SourcePrivacyDescriptor;

/// Which deferred seam phase a [`SeamError::Deferred`] came from. Naming the phase in the
/// error value (not just a doc-comment) keeps the deferral auditable at the call site —
/// the same honesty discipline as `sparq-mpc`'s `MpcError::NotYetImplemented`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeamPhase {
    /// Phase 2 — privacy/authorisation-aware source selection
    /// ([`select_private_sources`](crate::select_private_sources)).
    SourceSelection,
    /// Phase 3 — the disclosed/hidden routing partition
    /// ([`route_operators`](crate::route_operators)).
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

/// The seam's error type. [`SeamError::Deferred`] is the honest typed channel every *deferred*
/// phase stub returns (it carries the [`SeamPhase`] and the `gated_on` issues/audits, so a caller
/// learns WHY the phase is unavailable rather than hitting a panic or a fabricated result).
/// [`SeamError::DescriptorMismatch`] is the first *real* error variant — the implemented
/// source-selection adapter (Phase 2) and the routing pass (Phase 3) both return it on an
/// ambiguous / inconsistent privacy declaration. (The enum is `#[non_exhaustive]` so later phases
/// can add variants without a breaking change.)
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
    /// A privacy descriptor set was inconsistent — e.g. two
    /// [`crate::SourcePrivacyDescriptor`]s declared for the same source id (an ambiguous
    /// authorisation policy). The adapter **refuses** (fail-closed) rather than guessing which
    /// one binds. Returned by [`select_private_sources`](crate::select_private_sources).
    DescriptorMismatch {
        /// Which phase raised it ([`SeamPhase::SourceSelection`] for Phase 2,
        /// [`SeamPhase::Routing`] for Phase 3).
        phase: SeamPhase,
        /// The offending source id.
        source_id: String,
        /// A short human-readable explanation of the inconsistency.
        detail: &'static str,
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
            SeamError::DescriptorMismatch {
                phase,
                source_id,
                detail,
            } => write!(
                f,
                "sparq-fedplan-mpc {} privacy-descriptor mismatch for source {}: {}",
                phase.label(),
                source_id,
                detail
            ),
        }
    }
}

impl std::error::Error for SeamError {}

/// The output of the **Phase-3 routing pass** ([`route_operators`](crate::route_operators)): the
/// per-operator disclosed-vs-hidden routing the `sparq-mpc` pipeline consumes. It wraps the real
/// [`sparq_mpc::pipeline::OperatorRouting`] vector so the output is the genuine seam shape — the
/// structure the pipeline today receives hand-written. The routing decision is computed by the
/// pass (default-deny, policy-parameterised); see [`crate::routing`].
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PrivateRouting {
    /// The per-operator routing the `sparq-mpc` pipeline accepts, in input-operator order.
    pub routing: Vec<OperatorRouting>,
}

/// The output of the (deferred) leakage-envelope phase: the declared set of facts the chosen
/// routing reveals, for the holder/verifier dual ratification. A typed PLACEHOLDER for Phase 4
/// — its internal fields are filled by the phase that assembles it; it carries no data in
/// Phase 1 and is never produced (the stub returns [`SeamError::Deferred`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LeakageEnvelope {}

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

    fn privacy() -> SourcePrivacyDescriptor {
        SourcePrivacyDescriptor::deny_all(SourceId::new("http://s/"))
    }

    // Phase 2 (`select_private_sources`) is IMPLEMENTED — its tests live in `crate::selection`.
    // Phase 3 (`route_operators`) is IMPLEMENTED — its tests live in `crate::routing`.
    // Phase 4 is still a deferred stub; the tests below pin its honest deferral.

    #[test]
    fn leakage_envelope_is_deferred_not_a_panic() {
        let err = assemble_leakage_envelope(&PrivateRouting::default(), &[privacy()]).unwrap_err();
        match err {
            SeamError::Deferred { phase, gated_on } => {
                assert_eq!(phase, SeamPhase::LeakageEnvelope);
                // The leakage-envelope phase names the external audit gates.
                assert!(gated_on.contains("sq-9hrn"));
                assert!(gated_on.contains("sq-qhy4"));
            }
            other => panic!(
                "expected a Deferred leakage-envelope error, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn phase_labels_are_distinct() {
        let labels = [
            SeamPhase::SourceSelection.label(),
            SeamPhase::Routing.label(),
            SeamPhase::LeakageEnvelope.label(),
        ];
        // All three seam phases carry a distinct human label.
        assert_eq!(
            labels
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn descriptor_mismatch_displays_the_offending_source() {
        // The new Phase-2 error variant renders the source id + phase in its Display.
        let err = SeamError::DescriptorMismatch {
            phase: SeamPhase::SourceSelection,
            source_id: "http://dup/".to_string(),
            detail: "duplicate descriptor",
        };
        let shown = format!("{}", err);
        assert!(shown.contains("http://dup/"));
        assert!(shown.contains("source selection"));
    }
}
