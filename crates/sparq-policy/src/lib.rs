#![doc = include_str!("../README.md")] // [OPUS-4.8] README is the docs.rs front page
#![forbid(unsafe_code)] // [OPUS-4.8] sq-r06h: crate has zero `unsafe`

mod eval;
pub mod model;
mod parse;

pub use eval::{
    evaluate, matched_prohibition, prohibition_status, purpose_status, Decision, ProhibitionStatus,
    PurposeMatch, Request, ODRL_PURPOSE,
};
pub use model::{Action, Constraint, Duty, Operator, Policy, Rule, Value, ODRL_NS};
pub use parse::{parse_policy, parse_policy_str};
