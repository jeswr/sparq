//! Physical federation operators (design §4.4).
//!
//! Phase 0 PLACEHOLDER — no logic yet. This module will interpret a
//! [`JoinTree`](sparq_fedplan::JoinTree) into physical operators over the
//! [`SolutionStream`](crate::stream) boundary:
//!
//! * `JoinAlgo::Bind` → a bind-join operator (gather a block of upstream bindings, push
//!   via VALUES/brTPF, stream matches) — `try_bound_join_service` generalised from
//!   "accumulate ALL blocks then join" to "emit per block as it returns";
//! * `JoinAlgo::Hash` / `JoinAlgo::Streaming` → feed each side's `SolutionStream` into
//!   the REUSED [`StreamJoin`](sparq_fedplan::StreamJoin) (non-blocking symmetric hash
//!   join + bounded spill) — the single new piece is a stream feeder replacing
//!   `run_streaming`'s `&[Tuple]` slices, plus the light-`Tuple` ↔ engine-`Bindings`
//!   bridge so engine-evaluated local results and remote sub-results can be joined;
//! * `Local` leaf → `sparq-engine` local BGP evaluation on the in-process `Graph`.
//!
//! Concurrent fan-out issues independent leaf/group fetches in parallel; first results
//! stream as soon as the earliest source responds. Async/runtime choice stays INSIDE
//! this opt-in crate (design §7) — never leaking into core/engine (Phases 3/5).
//
// [OPUS-4.8] sq-s1uy (epic sq-dnko): Phase-0 skeleton module — populated in Phases 3/5.
