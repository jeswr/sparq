#![doc = include_str!("../README.md")] // [OPUS-4.8] README is the docs.rs front page
#![forbid(unsafe_code)] // [OPUS-4.8] sq-r06h: crate has zero `unsafe`

// [OPUS-4.8] sq-zi5w: stateful `odrl:count` enforcement, gated on `count-enforcement`
// (OFF by default — the default build carries zero counter-store code or runtime cost).
#[cfg(feature = "count-enforcement")]
pub mod count;

// [OPUS-4.8] sq-5z1q: a CROSS-PROCESS `UsageCounterStore` (a file + OS lockfile) — the
// distributed-atomicity follow-up to sq-zi5w's in-process `InMemoryCounterStore`. Same
// feature gate, same `try_consume` contract, std-only (no new deps, no `unsafe`).
#[cfg(feature = "count-enforcement")]
pub mod count_file;

// [OPUS-4.8] sq-zabv: static (request-free) policy analysis — permission/prohibition
// conflict + policy containment/refinement, on the query-containment comparison
// semantics. Always compiled (no feature gate); it adds no deps and no core cost.
mod compare;
mod eval;
pub mod model;
mod parse;

pub use compare::{contains, detect_conflicts, Conflict, Containment, Overlap};
pub use eval::{
    cmp_datetime, datetime_status, evaluate, matched_prohibition, prohibition_status,
    purpose_status, recipient_status, spatial_status, DateTimeMatch, Decision, ProhibitionStatus,
    PurposeMatch, RecipientMatch, Request, SpatialMatch, ODRL_COUNT, ODRL_DATETIME, ODRL_PURPOSE,
    ODRL_RECIPIENT, ODRL_SPATIAL,
};
pub use model::{Action, Constraint, Duty, Operator, Policy, Rule, Value, ODRL_NS};
pub use parse::{parse_policy, parse_policy_str};

// [OPUS-4.8] sq-zi5w: re-export the count-enforcement surface at the crate root when the
// feature is on (matches how the stateless evaluator surface is flat-exported).
#[cfg(feature = "count-enforcement")]
pub use count::{
    count_status, evaluate_and_exercise, ConsumeResult, CountKey, CountStatus, ExerciseDecision,
    InMemoryCounterStore, UsageCounterStore,
};

// [OPUS-4.8] sq-5z1q: the cross-process file-backed store, flat-exported alongside the
// in-memory one (a drop-in for the same `UsageCounterStore` seam).
#[cfg(feature = "count-enforcement")]
pub use count_file::FileCounterStore;
