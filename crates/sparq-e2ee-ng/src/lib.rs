//! # sparq-e2ee-ng — E2EE-NG profile **primitives** (capability / envelope / epoch)
//!
//! This crate implements the low-level, opt-in cryptographic primitives for the
//! **NextGraph-style E2EE-queryable profile** designed in
//! [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](https://github.com/jeswr/sparq)
//! (program `sq-tag1q`, bead `sq-tag1q.9`):
//!
//! * [`cbor`] — a minimal **deterministic** CBOR codec (RFC 8949 core
//!   deterministic encoding) with explicit, fail-closed **parser limits**;
//! * [`suite`] — **algorithm agility** with one bound v0 suite;
//! * [`keyschedule`] — a **domain-separated** HKDF-SHA-256 key schedule;
//! * [`sign`] — Ed25519 signatures for the three separated authorities;
//! * [`wrap`] — **recipient-wrapped** secrets (X25519 sealed-box shape);
//! * [`capability`] — deterministic capability encoding with **strict
//!   read/publish/admin separation** and constrained delegation;
//! * [`envelope`] — **randomized** block & commit envelopes, padded, with a
//!   `CommitId = SHA-256(canonical encrypted root envelope)`;
//! * [`epoch`] — signed **epoch transitions** (the revocation mechanism).
//!
//! Plus one **opt-in** module behind the `se` cargo feature (OFF by default):
//!
//! * `literal` — the **Profile SE** ("structure-exposed") encrypted-literal codec
//!   (`research/e2ee-queryable-options.md` §3.c): a literal *value* is AEAD-sealed
//!   into an ordinary typed literal, so an untrusted server can still evaluate the
//!   *structural* SPARQL fragment over the cleartext skeleton with no new
//!   server-side code. It is a **different, weaker disclosure posture** than the
//!   block profile above — it reveals the **full graph topology** to the server
//!   and protects the values, not the shape of the user's life — which is exactly
//!   why it is opt-in. Read that module's leakage headline before enabling the
//!   feature.
//!
//! ## Honesty & audit boundary — read first
//!
//! Every confidentiality, integrity, authorization, revocation, and convergence
//! property this crate *implements* is **designed/intended, not proven**. The
//! construction has **not** received an external cryptographic review. This crate
//! is **research-grade** and **MUST NOT** be relied on for production
//! confidentiality: production suite selection and the full soundness review are
//! gated by **`sq-qhy4`** (research doc §8.1, §9). In particular this crate does
//! **not** claim cryptographic soundness or zero-knowledge, and it does not turn
//! SPARQL into ciphertext-side evaluation — the profile keeps querying *local*
//! over decrypted, materialized state.
//!
//! The `se` module does not change that last sentence: nothing there evaluates
//! SPARQL over ciphertext either. What Profile SE moves server-side is the
//! *structural* fragment over structure that was never encrypted in the first
//! place; anything touching an encrypted value stays opaque and must be
//! post-filtered client-side after decryption.
//!
//! The primitives here are the *capability / envelope / epoch* layer only (plus
//! the opt-in `se` value codec). The sync, broker-protocol, CRDT, and
//! materialization layers of the profile (§6, §8.4–8.5) are **not** implemented in
//! this crate.
//!
//! ## Crate-boundary invariant
//!
//! Encryption and key material live **only** behind this crate (research doc §7):
//! `sparq-core`, `sparq-engine`, and `sparq-substrate` never link a cipher and do
//! not depend on this crate. It is a standalone, non-default workspace member, so
//! the default build and the wasm artifact are byte-identical with or without it.
//!
//! ## Example — seal a block and open it
//!
//! ```
//! use sparq_e2ee_ng::envelope::{seal_block_random, open_block, BlockContext, ObjectKind};
//! use sparq_e2ee_ng::ids::{RepoId, BranchId, Epoch, ObjectId, Secret32};
//!
//! let k_read = Secret32::random();
//! let ctx = BlockContext {
//!     repo: RepoId::random(),
//!     branch: BranchId::random(),
//!     epoch: Epoch(0),
//!     kind: ObjectKind::Operation,
//! };
//! let object = ObjectId::random();
//! let env = seal_block_random(&k_read, &ctx, &object, 0, 1, b"hello quads").unwrap();
//! let pt = open_block(&k_read, &ctx, &env).unwrap();
//! assert_eq!(pt, b"hello quads");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod capability;
pub mod cbor;
pub mod envelope;
pub mod epoch;
pub mod error;
pub mod ids;
pub mod keyschedule;
#[cfg(feature = "se")]
#[cfg_attr(docsrs, doc(cfg(feature = "se")))]
pub mod literal;
pub mod sign;
pub mod suite;
pub mod wrap;

pub use error::{Error, Result};
pub use suite::SUITE_V0;
