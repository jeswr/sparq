// [OPUS-4.8] sq-ieqz: the crate's rustdoc front page IS its crate-local README
// (`crates/sparq-serve/README.md`, the same file crates.io/docs.rs surface), so the
// two never drift. The Wave A/B design narrative (ring / sequenced writer / scheduler
// invariants, the time-travel opt-in, the library-first + no-wasm-dep guards) lives in
// that README; the per-item rustdoc on the re-exports below carries the API detail.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod applier;
/// [OPUS-4.8] (sq-o5bi) ONLINE consistent-snapshot backup + restore for the serving store
/// — export an already-immutable pinned [`Generation`] to a single self-describing artifact
/// WHILE SERVING (no stop-the-world), and re-hydrate a [`sparq_core::Graph`] from one
/// (fail-closed on a corrupt/mismatched artifact). Compiled only behind the opt-in `backup`
/// feature (default OFF); the serving core is fully buildable without it. See the module docs
/// for the Option-A artifact format and the at-rest-encryption out-of-scope boundary.
#[cfg(feature = "backup")]
pub mod backup;
mod epoch;
mod footprint;
mod ring;
mod scheduler;
mod writer;

#[cfg(feature = "backup")]
pub use backup::{export as backup_export, import as backup_import, BackupError, BackupMeta};
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
