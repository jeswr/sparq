#![doc = include_str!("../README.md")] // [OPUS-4.8] README is the docs.rs front page
#![forbid(unsafe_code)] // [OPUS-4.8] sq-r06h: crate has zero `unsafe`

// [OPUS-4.8] sq-zi5w: stateful `odrl:count` enforcement, gated on `count-enforcement`
// (OFF by default — the default build carries zero counter-store code or runtime cost).
#[cfg(feature = "count-enforcement")]
pub mod count;

mod eval;
pub mod model;
mod parse;

pub use eval::{
    datetime_status, evaluate, matched_prohibition, prohibition_status, purpose_status,
    recipient_status, DateTimeMatch, Decision, ProhibitionStatus, PurposeMatch, RecipientMatch,
    Request, ODRL_COUNT, ODRL_DATETIME, ODRL_PURPOSE, ODRL_RECIPIENT,
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
