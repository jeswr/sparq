//! Planner-to-physical bridge (design §4.2).
//!
//! Phase 0 PLACEHOLDER — no logic yet. The client does NOT write a new planner: this
//! module will (1) lower the parsed query's BGP(s) into `sparq-fedplan`'s light
//! [`Bgp`](sparq_fedplan::Bgp) / `TriplePattern` / `Term` / `Var`; (2) build one
//! [`SourceDescriptor`](sparq_fedplan::SourceDescriptor) per discovered source and call
//! [`select_sources`](sparq_fedplan::select_sources) then
//! [`plan_bgp`](sparq_fedplan::plan_bgp) (with `PlanOptions::request_cost` /
//! `stream_threshold` tuned per the discovered transport); and (3) resolve the plan's
//! pattern *indices* → patterns and source *indices* → endpoint adapters (the mapping
//! the design §3.2(4) calls out as missing). The cost-based join order + bind-vs-hash
//! decision + CS star cardinality are already correct, tested, and deterministic in
//! `sparq-fedplan` — this bridge only supplies descriptors and consumes the `JoinTree`
//! (Phase 3).
//
// [OPUS-4.8] sq-s1uy (epic sq-dnko): Phase-0 skeleton module — populated in Phase 3.
