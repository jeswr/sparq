// AUTHORED-BY Claude Opus 4.8
//! The LDP / Solid Protocol request path (M1 + the M2 verb-completion slice).
//!
//! - [`target`] — request-URL / LDP-target parsing (pure value logic).
//! - [`cors`] — the Solid Protocol CORS middleware (reflective origin + credentials, preflight,
//!   exposed headers, case-sensitive `Vary: Origin`) — a hand-rolled axum middleware.
//! - [`content`] — Turtle / JSON-LD classification, RDF validation, re-serialisation, and `Accept`
//!   content negotiation.
//! - [`conditional`] — `If-Match` / `If-None-Match` precondition evaluation over the strong ETag.
//! - [`range`] — single and multipart `Range: bytes=…` request handling (206 / 416). [GPT-5.6]
//! - [`patch`] — the Solid N3-Patch engine (`text/n3`): insert/delete plus the `solid:where`
//!   variable solver (basic-graph-pattern matching with the spec's exactly-one-solution rule).
//! - [`handler`] — the GET / HEAD / PUT / POST / DELETE / PATCH axum handlers over the
//!   [`crate::store::Store`] seam.
//!
//! Full WAC authorization is IMPLEMENTED: every resource read/mutation handler here
//! (GET / HEAD / PUT / POST / DELETE / PATCH) authorizes through [`crate::authz`] (per-resource
//! `.acl` evaluation, own-ACL-else-nearest-ancestor `acl:default` resolution, the four modes, the
//! 401-vs-403 split, `WAC-Allow`), and the decision runs BEFORE any existence probe so a denial
//! never discloses whether the target exists. **OPTIONS is the deliberate exception**: it is NOT
//! auth-gated, because it returns only static method/media-type metadata (`Allow`, `Accept-Post`,
//! `Accept-Patch`) for the surface — it neither reads content nor probes whether the target exists —
//! and it is the path the CORS preflight rides on (see [`handler::options_handler`]).
//! `application/sparql-update` PATCH now supports the INSERT/DELETE DATA subset (see [`patch`]).
//!
//! M2-next (clearly seamed, not implemented): `acl:agentGroup` membership resolution (recognised but
//! NEVER matching — fail-closed) and the N-Triples/N-Quads/N3 read formats (see [`content`]).

pub mod conditional;
pub mod content;
pub mod cors;
pub mod handler;
pub mod patch;
pub mod public_read_skip;
pub mod range;
pub mod target;
