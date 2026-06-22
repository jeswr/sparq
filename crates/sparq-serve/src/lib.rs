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
/// [OPUS-4.8] (sq-bu1a) The INCREMENTAL DELTA-STREAM / point-in-time-recovery companion to the
/// Option-A base backup ([`backup`]): export the change between two same-lineage generations as a
/// self-describing delta artifact keyed off generation/writer-seq, and replay an ordered chain of
/// deltas forward onto a restored base to reach a chosen recovery point (fail-closed on a corrupt
/// or discontinuous stream). Compiled only behind the opt-in `backup` feature (default OFF). See
/// the module docs for the delta artifact format and the same-lineage boundary.
#[cfg(feature = "backup")]
pub mod backup_delta;
mod epoch;
mod footprint;
mod ring;
mod scheduler;
mod writer;

// [OPUS-4.8] sq-jluc: the response-bytes result cache (research/concurrent-serving.md
// §5 row 2 + §6.3) is OPT-IN and OFF by default — the `result-cache` cargo feature.
// The default `sparq-serve` build (and every consumer that does not turn the feature
// on) carries ZERO cache code: no module, no canonicalization, no extra dependency
// (the cache reuses the in-tree `rustc-hash` and `std`). Gating the modules — not just
// the re-exports — is what keeps the default build lean.
//
// LAYER DISTINCTION (do not conflate): `sparq-engine` ALSO has a feature named
// `result-cache`, but it is a DIFFERENT crate and a DIFFERENT layer — the engine's is
// the embedded whole-query algebra-keyed LRU inside one engine instance (caller bumps a
// version on mutation). THIS cache is the SERVING-layer response-bytes cache: keyed on
// the visibility scope and invalidated by the per-pod epoch bumps of the generation
// ring, neither of which the engine layer knows about. Cargo features are per-package,
// so the shared name is not a conflict; the module docs in `cache.rs` spell out the
// boundary.
#[cfg(feature = "result-cache")]
mod cache;
#[cfg(feature = "result-cache")]
mod canon;

#[cfg(feature = "backup")]
pub use backup::{export as backup_export, import as backup_import, BackupError, BackupMeta};
// [OPUS-4.8] (sq-bu1a) The change-stream / PITR delta companion (feature `backup`):
// `export_delta` between two same-lineage generations, `import_delta` to decode one (fail-closed),
// and `replay` to apply an ordered chain forward onto a restored base. `DELTA_MAGIC` is the
// artifact's distinguishing magic line; `Delta`/`DeltaMeta` are the decoded forms.
#[cfg(feature = "backup")]
pub use backup_delta::{
    export_delta as backup_export_delta, import_delta as backup_import_delta,
    replay as backup_replay, Delta, DeltaMeta, DELTA_MAGIC,
};
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

// [OPUS-4.8] sq-jluc: result-cache public surface (feature `result-cache`). The cache's
// invalidation `Footprint` (the per-pod / unbounded READ footprint) is a DISTINCT type
// from this crate's write-side `footprint::Footprint` (the commutativity conflict tag),
// so it is re-exported under the disambiguating name `ReadFootprint`.
#[cfg(feature = "result-cache")]
pub use cache::{
    Bytes, CacheConfig, CacheStats, Footprint as ReadFootprint, LeadGuard, LeaseOutcome,
    ResultCache, ScopeKey, WaitGuard, WaitOutcome, DEFAULT_MAX_BYTES, DEFAULT_MAX_ENTRY_BYTES,
};
#[cfg(feature = "result-cache")]
pub use canon::{canonicalize, canonicalize_renamed};
