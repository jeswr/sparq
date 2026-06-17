// [OPUS-4.8] sq-ieqz: the crate's rustdoc front page IS its crate-local README
// (`crates/sparq-serve/README.md`, the same file crates.io/docs.rs surface), so the
// two never drift. The Wave A/B design narrative (ring / sequenced writer / scheduler
// invariants, the time-travel opt-in, the library-first + no-wasm-dep guards) lives in
// that README; the per-item rustdoc on the re-exports below carries the API detail.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod applier;
mod epoch;
mod footprint;
mod ring;
mod scheduler;
mod writer;

pub use applier::{GraphApplier, DEFAULT_COMPACT_THRESHOLD};
pub use epoch::{Epoch, PodEpochs, PodId};
pub use footprint::{Footprint, TargetGraph};
pub use ring::{Generation, GenerationRing, RingConfig, TimeTravelConfig, DEFAULT_RETAIN};
pub use scheduler::{
    Cost, Lane, SchedError, Scheduler, SchedulerConfig, Ticket, DEFAULT_HEAVY_THRESHOLD, P0,
};
pub use writer::{
    ApplyUpdates, CommitGranularity, WriteError, Writer, WriterConfig, DEFAULT_MAX_BATCH,
    DEFAULT_WINDOW,
};
