//! Capability discovery (design §4.1).
//!
//! Phase 0 PLACEHOLDER — no logic yet. This module will, per endpoint source: GET
//! `/.well-known/void` and the Service Description document (`GET /sparql` with no
//! query), parse the VoID + `scs:` characteristic-set N-Triples into a
//! [`SourceDescriptor`](sparq_fedplan::SourceDescriptor) via the EXISTING
//! `from_void_nt` seam (the producer/consumer match already exists end-to-end:
//! sparq-server `to_void_with_cs` → this client's `from_void_nt`), and parse SD into a
//! `Capability` (the one genuinely-new parser this layer needs — the server has only the
//! writer). When nothing is published it falls back to FedX-style ASK probes for bound
//! patterns and to TPF per-fragment count metadata. `discover()` is cached so the hot
//! path pays nothing (Phase 1).
//
// [OPUS-4.8] sq-s1uy (epic sq-dnko): Phase-0 skeleton module — populated in Phase 1.
