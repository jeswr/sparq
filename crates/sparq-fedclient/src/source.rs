//! Source-type abstraction (design §4.1).
//!
//! Phase 0 PLACEHOLDER — no logic yet. This module will hold the `SourceType` enum
//! (`Endpoint` | `BrTpf` | `Tpf` | `Local`), the `Capability` descriptor (which FILTER
//! ops / aggregates / property-paths / ORDER-LIMIT a source can evaluate, its bind-join
//! mode — VALUES vs `maxMpR(n)` vs none, and its advertised result formats), and the
//! `FederatedSource` trait:
//!
//! ```ignore
//! pub trait FederatedSource {
//!     fn discover(&self) -> Result<(Capability, Option<SourceDescriptor>), FedError>;
//!     fn execute(&self, sub: &SubQuery) -> SolutionStream;
//! }
//! ```
//!
//! REUSE: the `Endpoint` adapter (Phase 2) wraps `sparq-engine`'s existing SRJ
//! `Transport` + SSRF `EgressFilterResolver`; `Local` evaluates the sub-BGP directly
//! through `sparq-engine` on the in-process `Graph`. The brTPF/TPF adapters land in
//! Phase 6.
//
// [OPUS-4.8] sq-s1uy (epic sq-dnko): Phase-0 skeleton module — populated in Phases 2/6.
