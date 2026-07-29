// AUTHORED-BY Claude Opus 4.8
//! Web Access Control (WAC) authorization.
//!
//! The local, in-Rust WAC engine — a semantic port of prod-solid-server `src/authz/` (NOT a code
//! copy). It supersedes the interim pre-WAC posture in [`crate::ldp::handler`] with real per-resource
//! `.acl` evaluation:
//!
//! - [`mode`] — the four access modes + the HTTP-method → required-mode mapping (`.acl` ⇒ Control).
//! - [`acl`] — rule-matching: a parsed `.acl` graph (`oxrdf::Triple`s, parsed via `oxttl`/`oxjsonld`,
//!   NEVER hand-parsed) → the modes granted to a requester under an `accessTo`/`default` scope.
//! - [`wac`] — the authorizer: own-ACL-else-nearest-ancestor-`acl:default` resolution (child→root,
//!   fail-closed), the allow/deny [`Decision`] with the 401-vs-403 split, and the
//!   effective-permissions computation for `WAC-Allow`.
//! - [`wac_allow`] — the `WAC-Allow` response-header serialiser.
//!
//! The opt-in `access_profile` module is a different contract: the strict ODRL access profile of
//! the LWS spec (`access-profile-odrl1`), a pure port of that spec's normative access-decision rule
//! set measured against its vendored test-vectors. It is NOT part of the WAC decision path.
//!
//! ## Architecture note (the maintainer's directive)
//! In the production architecture WAC evaluation is SPARQ-authoritative (the ACL graph in SPARQ is the
//! source of truth, gated on `sparq#992`). In this slice — which runs on the in-memory store doubles —
//! the engine reads each `.acl` resource THROUGH the [`Store`](crate::store::Store) seam and evaluates
//! it locally. When the SPARQ access-control design lands, the per-resource decision moves behind the
//! same [`WacAuthorizer`] seam (ask SPARQ for the decision instead of reading +
//! evaluating the `.acl` here) with no change to the handler wiring.

pub mod acl;
// [SONNET-4.6] sq-gg0qq.6: the opt-in strict ODRL access profile (`access-profile-odrl1`) — a Rust
// port of the LWS spec's normative access-decision rule set, gated by the vendored `lws-spec/`
// test-vectors. A PURE library function over already-decoded documents (no I/O, no store reach),
// deliberately NOT wired into `ldp::handler`: enabling the feature changes no request's outcome.
// Native-only, mirroring the ODRL gate below.
#[cfg(all(feature = "access-profile-odrl1", not(target_arch = "wasm32")))]
pub mod access_profile;
pub mod mode;
// The opt-in ODRL policy gate seam on the read/query path (`odrl-authz`, sq-elg47) — see the
// module's own docs. Native-only: the wasm request core keeps the policy layer out.
#[cfg(all(feature = "odrl-authz", not(target_arch = "wasm32")))]
pub mod odrl;
// [OPUS-5] sq-hed3q: the opt-in trust-graph admission seam (`trust-graph`) — a PURE library
// function, deliberately NOT wired into the LDP handler, so enabling the feature changes no
// request's outcome. Native-only, mirroring the ODRL gate: the wasm request core keeps the
// trust-graph estate out of its dependency graph.
#[cfg(all(feature = "trust-graph", not(target_arch = "wasm32")))]
pub mod trust_admit;
pub mod wac;
pub mod wac_allow;

pub use acl::{AclScope, Requester};
pub use mode::{is_acl_auxiliary_suffix, is_acl_resource, mode_for_operation, AccessMode};
pub use wac::{AclCandidate, Decision, ReadDecision, WacAuthorizer};
pub use wac_allow::{wac_allow_header, EffectivePermissions};
