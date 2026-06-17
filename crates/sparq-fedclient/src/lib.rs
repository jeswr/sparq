//! sparq-fedclient: a **streaming federation CLIENT** over heterogeneous remote RDF
//! sources — the query *consumer* half of federation (epic **sq-dnko** / **sq-3183**,
//! Phase 0 bead **sq-s1uy**). See `research/federation-client-design.md` for the full
//! design (§4 architecture, §6 phased build plan, §7 honest risks).
//!
//! Given one SPARQL query and a set of heterogeneous remote sources — full SPARQL
//! endpoints, bindings-restricted (brTPF) servers, plain TPF servers, and the *local*
//! sparq engine — this crate (when complete) **discovers** each source's capability,
//! **plans** a federated execution that pushes the most precise sub-query each source
//! can answer (REUSING the [`sparq-fedplan`](sparq_fedplan) cost-based planner), and
//! **streams** results back through non-blocking federation operators.
//!
//! # Phase 0 — what this is, and what it is NOT
//!
//! This is the **compiling skeleton only** (design §6, Phase 0). It establishes:
//!
//! * the **public module layout** the design §4 names ([`source`], [`discovery`],
//!   [`planner`], [`pushdown`], [`operators`], [`stream`]) — each currently an empty,
//!   `todo!()`-free placeholder module;
//! * the **opt-in feature** ([`fedclient`](#opt-in-hard-constraint), OFF by default);
//! * the **dependency-boundary proof** (below) — the load-bearing deliverable of Phase 0.
//!
//! There is **NO federation logic yet**: discovery, the source adapters, the planner
//! bridge, capability-aware pushdown, and the streaming operators land in Phases 1-7
//! (each a future bead under epic sq-dnko). The modules exist so the public surface is
//! visible and the boundary is provable *before* any logic is written.
//!
//! # Opt-in (hard constraint)
//!
//! The whole client is behind the **`fedclient` cargo feature, OFF by default**, and the
//! crate is a standalone workspace member with `publish = false`. `sparq-core` and
//! `sparq-engine` **never** depend on it — the dependency arrow points one-way *into*
//! the engine (the client reuses the engine's SERVICE transport + SSRF guard + local
//! eval), so the default engine build and the WASM artifact are byte-identical with or
//! without `sparq-fedclient`. A build that does not enable `fedclient` compiles an empty
//! crate (mirrors [`sparq-fedplan`](sparq_fedplan)'s `fedplan` feature).
//!
//! # The dependency boundary (load-bearing, enforced)
//!
//! Phase 0's actual point is to **prove the boundary before any logic exists**. Two
//! complementary checks enforce that neither `sparq-core` nor `sparq-engine` ever gains a
//! dependency edge *to* `sparq-fedclient`, in both feature states:
//!
//! * `scripts/fedclient-boundary-guard.sh` — a CI step (in `feature-matrix.yml`) that
//!   inverts the dependency graph (`cargo tree -i sparq-fedclient`) and fails if
//!   `sparq-core` or `sparq-engine` appears as a dependent;
//! * `tests/boundary.rs` — a hermetic `cargo metadata` test asserting the same invariant
//!   from inside the test suite, so it gates even off the CI script.
//!
//! Both must FAIL if a future edit introduces such an edge.
//!
//! [OPUS-4.8] sq-s1uy — flagged for Fable re-review when available.
#![forbid(unsafe_code)]
// [OPUS-4.8] sq-s1uy: crate has zero `unsafe`.
// When `fedclient` is off the crate is intentionally empty; when on, the Phase-0 module
// stubs are placeholders with no public items yet, so silence the expected dead-code/
// unused-import lints until the logic phases populate them.
#![cfg_attr(not(feature = "fedclient"), allow(dead_code, unused_imports))]
#![cfg_attr(feature = "fedclient", allow(dead_code, unused_imports))]

// ─── Public module layout (design §4) ──────────────────────────────────────────────
// Each module is a Phase-0 placeholder. The doc comment on each records WHICH design
// section it realises and WHICH existing sparq seam that phase reuses, so the layout is
// self-documenting before any logic exists. All are gated behind `fedclient`.

/// §4.1 — **source-type abstraction**: the `SourceType` enum (Endpoint | BrTpf | Tpf |
/// Local) and the `FederatedSource` trait (`discover()` + `execute(&SubQuery) ->
/// SolutionStream`), the sparq analogue of Comunica's `IQuerySource`. Phase 2 wires the
/// Endpoint adapter over the engine's existing SRJ transport; Phase 6 adds brTPF/TPF.
#[cfg(feature = "fedclient")]
pub mod source;
// [OPUS-4.8] sq-rsxf: re-export the Phase-2 §4.1 source surface at the crate root.
#[cfg(feature = "fedclient")]
pub use source::{
    is_forbidden_ip, BindJoin, BrTpfSource, Capability, EgressGuard, Endpoint, FedError,
    FederatedSource, FilterClass, Interface, LocalSource, SourceType, SubQuery, TpfSource,
    Transport,
};
// [OPUS-4.8] sq-2qze: re-export the Phase-6 brTPF/TPF fragment surface at the crate root.
#[cfg(feature = "fedclient")]
pub use source::{
    FragBinding, FragPattern, FragTerm, FragTriple, FragmentPage, FragmentTransport, PatternTerm,
};

/// §4.1 — **capability discovery** (Phase 1, bead sq-nfxl): GET `/.well-known/void` + the
/// Service Description per endpoint, parse VoID+`scs:` via the existing
/// [`SourceDescriptor::from_void_nt`](sparq_fedplan::SourceDescriptor) seam (REUSED), parse
/// SD into a [`Capability`](discovery::Capability) (the one genuinely-new client-side parser),
/// with a FedX-style ASK-probe fallback when nothing is published. Every fetch is behind an
/// SSRF-guarded [`Fetcher`](discovery::Fetcher) seam (default-deny private/internal IPs).
#[cfg(feature = "fedclient")]
pub mod discovery;

/// §4.2 — **planner bridge**: lower the parsed query's BGP into `sparq-fedplan`'s light
/// `Bgp`/`TriplePattern`/`Term`/`Var`, build one `SourceDescriptor` per discovered source,
/// call [`select_sources`](sparq_fedplan::select_sources) +
/// [`plan_bgp`](sparq_fedplan::plan_bgp), and resolve plan pattern/source *indices* to
/// patterns / endpoint adapters (Phase 3). The client does NOT write a new planner.
#[cfg(feature = "fedclient")]
pub mod planner;
// [OPUS-4.8] sq-j27p: re-export the Phase-3 planner-bridge surface at the crate root.
#[cfg(feature = "fedclient")]
pub use planner::{lower_leaf, pattern_vars, ResolveError, SourceResolver};

/// §4.3 — **capability-aware pushdown**: per leaf / FedX exclusive group, build the
/// MOST PRECISE sub-query a source can answer (projection + common-variable-checked
/// filters + ORDER/LIMIT when the capability covers them), reusing
/// `render_values_block` for VALUES bind-join. Anything a source cannot evaluate is kept
/// local (Phase 4). Pushdown only ever NARROWS a source's result — correctness-preserving.
#[cfg(feature = "fedclient")]
pub mod pushdown;

/// §4.4 — **physical federation operators**: interpret a `JoinTree` into operators —
/// `Bind` → VALUES/brTPF bind-join, `Hash`/`Streaming` → the reused
/// [`StreamJoin`](sparq_fedplan::StreamJoin), `Local` → `sparq-engine` local eval — over
/// the async [`stream::SolutionStream`] boundary, with concurrent fan-out (Phase 3/5).
#[cfg(feature = "fedclient")]
pub mod operators;
// [OPUS-4.8] sq-j27p: re-export the Phase-3 materialised single-source interpreter surface.
#[cfg(feature = "fedclient")]
pub use operators::{
    materialize_single_source, parse_srj, solutions_equal, InterpError, Relation,
};

/// §4.4 — the **`SolutionStream`** abstraction the client owns at its boundary (the
/// engine stays materialised, §7). Backpressured + bounded (Rust ownership + explicit
/// buffer bounds + the reused StreamJoin spill — no GC heap). Adapters yield it;
/// operators consume and produce it (Phase 5).
#[cfg(feature = "fedclient")]
pub mod stream;
