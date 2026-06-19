// ─── Crate-level documentation ──────────────────────────────────────────────────────
// [OPUS-4.8] sq-2q1x: the crate doc is split per feature state (mirroring sparq-fedplan's
// sq-gxx7 fix) so `cargo doc` is clean under `-D rustdoc::broken_intra_doc_links` in BOTH
// states: the ON narrative intra-doc-links the gated public items (which exist only when
// the feature is on); the default (feature OFF) build serves a concise, link-free doc.
#![cfg_attr(
    feature = "fedplan-mpc",
    doc = r#"sparq-fedplan-mpc: the opt-in **glue** between cost-based federated source
selection (`sparq-fedplan`) and MPC-over-federated-SPARQL routing (`sparq-mpc`) — bead
sq-2q1x, the first (skeleton) slice of the untrusted-planner → MPC-routing seam (epics
sq-pwr / sq-0jsc; `research/mpc-untrusted-planner-routing-design.md`).

# What this PHASE delivers (skeleton only)

This bead lands the **seam scaffold**, not the protocol:

* [`SourcePrivacyDescriptor`] — the per-source privacy declaration that the (later) routing
  pass reads: which predicates a source is **willing to disclose in the clear**, the opaque
  attestation-key id its graph is signed under, and a reserved participation/authorisation
  field. Its posture is **default-DENY**: a predicate is treated as PRIVATE (MPC-routed,
  never disclosed) unless the source has explicitly marked it disclosable. See
  [`SourcePrivacyDescriptor::may_disclose`] for the load-bearing invariant.
* [`SeamPhase`] + [`SeamError`] — the typed, panic-free stubs for the deferred phases
  ([`select_private_sources`], [`route_operators`], [`assemble_leakage_envelope`]). Each
  returns `Err(`[`SeamError::Deferred`]`)` naming the phase and the gate it waits on. They
  **compile and are callable**; they perform NO MPC and reveal NOTHING.

# What this phase does NOT do (honest boundary)

It performs **no** MPC, **no** secret-sharing, and runs **no** privacy-bearing logic. It
makes **no** soundness, privacy, or security claim. The privacy-bearing phases
(source-selection pruning, the disclosed/hidden routing decision, the leakage-envelope
assembly + dual ratification, and the authenticated-input attestation) are **deferred** and
**audit-gated**: the MPC estate is research-grade, honest-majority semi-honest only, and is
**not** externally audited — the external accredited-cryptographer sign-off (sq-qhy4) and the
collaborative-coZK re-audit (sq-9hrn) are pending. Do not present anything here as providing
a privacy or soundness guarantee. See `README.md` and the design record for the full caveat.

# Opt-in (hard constraint)

The whole surface is behind the **`fedplan-mpc` cargo feature, OFF by default**, and the crate
is a standalone `publish = false` workspace member. `sparq-core` / `sparq-engine` never depend
on it, so the default engine build and the WASM artifact are byte-identical with or without
it; a build that does not enable `fedplan-mpc` compiles an empty crate and pulls in neither
`sparq-fedplan` nor `sparq-mpc`. Neither upstream gains a cross-dependency on the other.

[OPUS-4.8] sq-2q1x — flagged for Fable re-review."#
)]
// Default (feature OFF) build: a concise, link-free crate doc. None of the gated items above
// exist in this build, so the doc is plain text only (no intra-doc links). [OPUS-4.8] sq-2q1x.
#![cfg_attr(
    not(feature = "fedplan-mpc"),
    doc = r#"sparq-fedplan-mpc: the opt-in glue between cost-based federated source selection
(`sparq-fedplan`) and MPC-over-federated-SPARQL routing (`sparq-mpc`) — bead sq-2q1x, the
first (skeleton) slice of the untrusted-planner → MPC-routing seam. See `README.md` and
`skills/federated-planning/SKILL.md` for the full design.

**This is the default build with the `fedplan-mpc` feature OFF, so the crate is empty** — the
whole surface (the `SourcePrivacyDescriptor` and the typed deferred-phase stubs) is gated
behind the **`fedplan-mpc` cargo feature, OFF by default**. Build with `--features fedplan-mpc`
to see the seam API. The crate is a standalone `publish = false` workspace member; `sparq-core`
/ `sparq-engine` never depend on it, and a feature-off build pulls in neither `sparq-fedplan`
nor `sparq-mpc`. This phase is a SKELETON — it performs no MPC and makes no privacy/soundness
claim; the privacy-bearing phases are deferred and audit-gated (sq-9hrn / sq-qhy4).

[OPUS-4.8] sq-2q1x — flagged for Fable re-review."#
)]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-2q1x: crate has zero `unsafe`.
// [OPUS-4.8] sq-2q1x: lock the crate-doc split in — `broken_intra_doc_links` is a rustdoc-only
// lint (fires under `cargo doc`, never `cargo build`/`clippy`/`test`), so denying it keeps the
// existing build/clippy/test gates untouched while making a future broken link a hard error.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(not(feature = "fedplan-mpc"), allow(dead_code, unused_imports))]

// The whole seam surface is feature-gated; a feature-off build compiles an empty crate.
#[cfg(feature = "fedplan-mpc")]
mod privacy;
#[cfg(feature = "fedplan-mpc")]
mod seam;

#[cfg(feature = "fedplan-mpc")]
pub use privacy::{Disclosability, SourcePrivacyDescriptor, SourcePrivacyDescriptorBuilder};
#[cfg(feature = "fedplan-mpc")]
pub use seam::{
    assemble_leakage_envelope, route_operators, select_private_sources, LeakageEnvelope,
    PrivateRouting, SeamError, SeamPhase, SelectedPrivateSources,
};
