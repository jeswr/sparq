// ─── Crate-level documentation ──────────────────────────────────────────────────────
// [OPUS-4.8] sq-2q1x: the crate doc is split per feature state (mirroring sparq-fedplan's
// sq-gxx7 fix) so `cargo doc` is clean under `-D rustdoc::broken_intra_doc_links` in BOTH
// states: the ON narrative intra-doc-links the gated public items (which exist only when
// the feature is on); the default (feature OFF) build serves a concise, link-free doc.
#![cfg_attr(
    feature = "fedplan-mpc",
    doc = r#"sparq-fedplan-mpc: the opt-in **glue** between cost-based federated source
selection (`sparq-fedplan`) and MPC-over-federated-SPARQL routing (`sparq-mpc`) — the
untrusted-planner → MPC-routing seam (beads sq-2q1x + sq-fix4, epics sq-pwr / sq-0jsc;
`research/mpc-untrusted-planner-routing-design.md`).

# What is delivered so far

Phase 1 (sq-2q1x) landed the **seam scaffold**; Phase 2 (sq-fix4) lands the
**source-selection adapter**. Phases 3–4 remain typed, panic-free, deferred stubs.

* [`SourcePrivacyDescriptor`] — the per-source privacy declaration the routing pass reads:
  which predicates a source is **willing to disclose in the clear**, the opaque attestation-key
  id its graph is signed under, and a reserved participation/authorisation field. Its posture is
  **default-DENY**: a predicate is treated as PRIVATE unless the source explicitly marks it
  disclosable. See [`SourcePrivacyDescriptor::may_disclose`] for the load-bearing invariant.
* [`select_private_sources`] (**Phase 2, implemented**) — wraps
  [`sparq_fedplan::select_sources`] and **prunes** any source that has declared it will not
  participate (the descriptor's authorisation field), surfacing the retained per-pattern
  candidate set plus an audit trail of what was pruned and why. The **only** prune rule is
  participation/authorisation — a source holding a *private* predicate stays a candidate (its
  in-clear-vs-MPC routing is the Phase-3 decision), so the adapter is recall-safe and loses no
  answers. It is plumbing — **source-selection routing, not a cryptographic guarantee**: it runs
  NO MPC and reveals nothing it was not already given in the clear. See [`selection`].
* [`SeamPhase`] + [`SeamError`] — the shared error/phase types plus the still-deferred Phase-3
  ([`route_operators`]) and Phase-4 ([`assemble_leakage_envelope`]) stubs, each returning
  `Err(`[`SeamError::Deferred`]`)` naming its gate. They **compile and are callable**; they
  perform NO MPC and reveal NOTHING.

# What this crate does NOT do (honest boundary)

It performs **no** MPC, **no** secret-sharing, and runs **no** privacy-bearing logic — the
Phase-2 adapter only routes the caller's own (public) descriptors. It makes **no** soundness,
privacy, or security claim. The privacy-bearing phases (the disclosed/hidden routing decision,
the leakage-envelope assembly + dual ratification, and the authenticated-input attestation) are
**deferred** and **audit-gated**: the MPC estate is research-grade, honest-majority semi-honest
only, and is **not** externally audited — the external accredited-cryptographer sign-off
(sq-qhy4) and the collaborative-coZK re-audit (sq-9hrn) are pending. Do not present anything here
as providing a privacy or soundness guarantee. See `README.md` and the design record for the
full caveat.

# Opt-in (hard constraint)

The whole surface is behind the **`fedplan-mpc` cargo feature, OFF by default**, and the crate
is a standalone `publish = false` workspace member. `sparq-core` / `sparq-engine` never depend
on it, so the default engine build and the WASM artifact are byte-identical with or without
it; a build that does not enable `fedplan-mpc` compiles an empty crate and pulls in neither
`sparq-fedplan` nor `sparq-mpc`. Neither upstream gains a cross-dependency on the other.

[OPUS-4.8] sq-2q1x / sq-fix4 — flagged for Fable re-review."#
)]
// Default (feature OFF) build: a concise, link-free crate doc. None of the gated items above
// exist in this build, so the doc is plain text only (no intra-doc links). [OPUS-4.8] sq-2q1x.
#![cfg_attr(
    not(feature = "fedplan-mpc"),
    doc = r#"sparq-fedplan-mpc: the opt-in glue between cost-based federated source selection
(`sparq-fedplan`) and MPC-over-federated-SPARQL routing (`sparq-mpc`) — the untrusted-planner →
MPC-routing seam (beads sq-2q1x + sq-fix4). See `README.md` and `skills/mpc/SKILL.md` for the
full design.

**This is the default build with the `fedplan-mpc` feature OFF, so the crate is empty** — the
whole surface (the `SourcePrivacyDescriptor`, the Phase-2 `select_private_sources` adapter, and
the deferred-phase stubs) is gated behind the **`fedplan-mpc` cargo feature, OFF by default**.
Build with `--features fedplan-mpc` to see the seam API. The crate is a standalone
`publish = false` workspace member; `sparq-core` / `sparq-engine` never depend on it, and a
feature-off build pulls in neither `sparq-fedplan` nor `sparq-mpc`. The crate performs no MPC and
makes no privacy/soundness claim; the privacy-bearing phases are deferred and audit-gated
(sq-9hrn / sq-qhy4).

[OPUS-4.8] sq-2q1x / sq-fix4 — flagged for Fable re-review."#
)]
#![forbid(unsafe_code)]
// [OPUS-4.8] sq-2q1x: crate has zero `unsafe`.
// [OPUS-4.8] sq-2q1x: lock the crate-doc split in — `broken_intra_doc_links` is a rustdoc-only
// lint (fires under `cargo doc`, never `cargo build`/`clippy`/`test`), so denying it keeps the
// existing build/clippy/test gates untouched while making a future broken link a hard error.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(not(feature = "fedplan-mpc"), allow(dead_code, unused_imports))]

// The whole seam surface is feature-gated; a feature-off build compiles an empty crate.
#[cfg(feature = "fedplan-mpc")]
mod privacy;
// [OPUS-4.8] sq-fix4: Phase 2 source-selection adapter lives in its own module (mirroring the
// upstream `sparq-fedplan::selection` split).
#[cfg(feature = "fedplan-mpc")]
mod seam;
#[cfg(feature = "fedplan-mpc")]
pub mod selection;

#[cfg(feature = "fedplan-mpc")]
pub use privacy::{Disclosability, SourcePrivacyDescriptor, SourcePrivacyDescriptorBuilder};
// [OPUS-4.8] sq-fix4: Phase 2 — the implemented privacy/authorisation-aware source-selection
// adapter and its result types.
#[cfg(feature = "fedplan-mpc")]
pub use seam::{
    assemble_leakage_envelope, route_operators, LeakageEnvelope, PrivateRouting, SeamError,
    SeamPhase,
};
#[cfg(feature = "fedplan-mpc")]
pub use selection::{
    select_private_sources, PrivateCandidate, PrivatePatternSources, PruneReason, PrunedCandidate,
    SelectedPrivateSources,
};
