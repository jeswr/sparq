// [OPUS-4.8] sq-jxl0: single-source the crate overview from README.md so crates.io
// (package.readme) and the docs.rs front page render identical content. The README's
// quickstart fence is a compiled doctest (hidden `#`-scaffolding for the pushed triple).
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod eval;
mod multi;
mod query;
mod rspql;
mod stream;
mod window;

pub use eval::EvalMode;
pub use multi::ContinuousMultiQuery;
pub use query::{
    AskResult, ContinuousAsk, ContinuousConstruct, ContinuousQuery, GraphResult, R2S, WindowResult,
};
pub use rspql::{RspqlQuery, WindowDecl};
pub use stream::{Timestamped, TimestampedTriple, TripleStream};
pub use window::{Window, WindowSpec, WindowedStream};
