//! # sparq-e2ee-ng-broker — the **opaque broker** for the E2EE-NG profile
//!
//! The server side of the NextGraph-style E2EE-queryable profile designed in
//! [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](https://github.com/jeswr/sparq)
//! (program `sq-tag1q`, bead `sq-tag1q.18`). It stores and routes **opaque**
//! encrypted blocks and topics for clients built on
//! [`sparq_e2ee_ng`](https://docs.rs/sparq-e2ee-ng), speaking the versioned
//! client/broker messages in [`sparq_e2ee_ng::broker_protocol`] (design §8.4).
//!
//! * [`broker`] — the clock-free [`Broker`] state machine: negotiation, routing,
//!   pinning, subscription, have/want sync, block operations, publication, epoch
//!   advance, limits, and retention;
//! * [`store`] — opaque block storage (exact envelope bytes, idempotent puts);
//! * [`log`] — [`log::LogRecord`], a metadata-safe log line that structurally
//!   cannot carry ciphertext, plaintext, or a secret.
//!
//! The `sparq-e2ee-ng-brokerd` binary wraps [`Broker`] in a length-prefixed CBOR
//! TCP listener.
//!
//! ## Honesty & audit boundary — read first
//!
//! This crate is **research-grade** and **externally UNAUDITED**. Every
//! confidentiality, integrity, authorization, and revocation property of the
//! profile it serves is **designed/intended, not proven**; production suite
//! selection and the full soundness review are gated by **`sq-qhy4`** (design
//! §8.1, §9). Nothing here is claimed cryptographically sound or private.
//!
//! In particular the design's own disclosure ledger (§5) is explicit that a
//! *conforming* broker still observes transport facts, topic membership,
//! subscription and publication patterns, message timing/ordering/sizes, opaque
//! identifiers, and storage volume — so this crate **MUST NOT** be described as
//! hiding access patterns, membership, volume, or timing. It is also not trusted
//! for integrity or availability: it can omit, delay, replay, reorder, or
//! equivocate, and clients validate envelopes, signatures, causal closure, and
//! CRDT rules locally.
//!
//! The reference binary implements **no transport authentication and no TLS**;
//! it is intended to run behind an authenticated, encrypted transport.
//!
//! ## Crate-boundary invariant
//!
//! Design §7: *"A separate opt-in broker binary/crate stores opaque blocks and
//! topics; it MUST not link the query engine."* This crate depends on
//! `sparq-e2ee-ng` and nothing else in the workspace — no `sparq-core`, no
//! `sparq-engine`, no `sparq-substrate` — and `tests/boundary.rs` proves it from
//! the resolved dependency graph rather than asserting it in prose. It is opt-in
//! by construction: nothing in the workspace depends on it, so the default build
//! and the wasm artifact are byte-identical with or without it.
//!
//! ## Example — negotiate, open, store, publish
//!
//! ```
//! use sparq_e2ee_ng::broker_protocol::{hello_v0, AdmissionGrant, OpenRepo, Request, Response};
//! use sparq_e2ee_ng::capability::Validity;
//! use sparq_e2ee_ng::ids::{Epoch, OverlayId, PeerId, TopicId};
//! use sparq_e2ee_ng::sign::SecretSigningKey;
//! use sparq_e2ee_ng::suite::SUITE_V0;
//! use sparq_e2ee_ng_broker::{Broker, BrokerConfig};
//! use sparq_e2ee_ng_broker::log::NullLog;
//!
//! let mut broker = Broker::new(BrokerConfig::default(), NullLog);
//! let session = broker.open_session();
//! let now = 1_800_000_000;
//!
//! // 1. Negotiate: version, suite, header mode, limits.
//! let ack = broker.handle(session, now, Request::Hello(hello_v0(1 << 20)), 0);
//! assert!(matches!(ack, Response::HelloAck(_)));
//!
//! // 2. Open a routing context under an admin-signed admission grant. Note what
//! //    is NOT here: no repo id, no branch id, no read secret.
//! let admin = SecretSigningKey::generate();
//! let publisher = SecretSigningKey::generate();
//! let topic = TopicId::random();
//! let mut grant = AdmissionGrant {
//!     topic,
//!     epoch: Epoch(0),
//!     suite: SUITE_V0.to_string(),
//!     admin_pub: admin.public().to_bytes(),
//!     publisher_pub: Some(publisher.public().to_bytes()),
//!     validity: Validity { not_before: 0, not_after: u64::MAX },
//!     admin_sig: None,
//! };
//! grant.sign(&admin)?;
//! let open = Request::OpenRepo(OpenRepo {
//!     overlay: OverlayId::random(),
//!     topic,
//!     epoch: Epoch(0),
//!     peer: PeerId::random(),
//!     auth: Some(grant),
//! });
//! assert!(matches!(broker.handle(session, now, open, 0), Response::Ok));
//! # Ok::<(), sparq_e2ee_ng::Error>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod broker;
pub mod log;
pub mod store;

pub use broker::{Broker, BrokerConfig, SessionId};
pub use store::GcReport;
