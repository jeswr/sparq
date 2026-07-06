//! Metamorphic + differential **logic-bug harness for SPARQL engines** — TLP and NoREC
//! re-derived for SPARQL's three-valued effective-boolean-value semantics, a
//! differential oracle over `sparq-difftest`, a seeded deterministic case generator,
//! SPARQL-protocol drivers, and the found-bug ledger format.
//!
//! [FABLE-5] Bead `sq-gum8.6` (epic `sq-gum8`, paper P2a → draft bead `sq-gum8.7`).
//! Research gap this machinery targets: dedicated logic-bug testing exists for SQL
//! (SQLancer: TLP, NoREC, PQS), Datalog (queryFuzz, DLSmith), and graph query languages
//! (GDsmith, Gamera for Cypher/Gremlin), but **not** for SPARQL engines — the closest
//! prior work (SparqLog, 2023) reported wrong results incidentally while comparing a
//! translation pipeline, not via a dedicated metamorphic oracle. The SPARQL-specific
//! re-derivation is genuinely different from the SQL original because SPARQL's third
//! truth state is an evaluation *error*, not a `NULL` value — see [`tlp`] for the full
//! case analysis (the paper's core claim).
//!
//! # Layout
//!
//! * [`tlp`] — Ternary Logic Partitioning under SPARQL three-valued EBV semantics
//!   (`FILTER(c)` / `FILTER(!c)` / the reified error branch).
//! * [`norec`] — the non-optimizable-rewrite cardinality oracle (predicate moved to
//!   projection position).
//! * [`differential`] — cross-engine comparison, reusing `sparq-difftest`'s
//!   engine-independent comparators as a dependency (that crate is not modified here).
//! * [`generate`] — seeded, wall-clock-free, reproducible query/data generation.
//! * [`engine`] — the engine-under-test trait, the in-process sparq driver, and the
//!   deliberately-injected wrong-result mutant the self-tests must flag.
//! * [`ledger`] — the found-bug record format (upstream issue link + confirmation
//!   status REQUIRED per entry).
//! * `driver` (cargo feature `protocol-drivers`, off by default) — HTTP drivers for
//!   external engines over the SPARQL 1.1 Protocol. The bug-hunting campaign runs
//!   OFF-CI against live endpoints; CI runs only network-free self-tests.
//!
//! # Verdict discipline (fail-closed)
//!
//! Every oracle returns a [`Verdict`] that strictly separates **wrong-result**
//! violations from **engine failures** (see [`verdict`]); an engine error is never
//! silently passed and never counted as a logic bug. The oracle self-tests include a
//! mutation check: a wrapper that silently removes a row from filtered results
//! ([`engine::FilterDropsRow`]) must be flagged by TLP, NoREC, *and* the differential
//! oracle — a suite that cannot flag its own seeded bug proves nothing.
//!
//! # Honest scope
//!
//! This crate is the *instrument*, not the campaign: bug counts belong to the (off-CI)
//! campaign and enter the [`ledger`] only with an upstream issue link and a
//! developer-confirmation status. The oracles' scope preconditions (deterministic
//! predicates, no `EXISTS`, plain `SELECT` at the oracle level, no blank-node
//! projection cross-engine) are documented in the respective modules and enforced by
//! the generator's grammar.

pub mod differential;
pub mod engine;
pub mod generate;
pub mod ledger;
pub mod norec;
pub mod tlp;
pub mod verdict;

#[cfg(feature = "protocol-drivers")]
pub mod driver;

pub use differential::check_differential;
pub use engine::{FilterDropsRow, InProcessSparq, SparqlEngine};
pub use generate::{generate_case, GeneratedCase, SplitMix64};
pub use ledger::{from_jsonl, to_jsonl, validate_entry, BugClass, ConfirmationStatus, FoundBug};
pub use norec::{check_norec, norec_queries, NorecQueries};
pub use tlp::{check_tlp, tlp_queries, TlpQueries};
pub use verdict::{EngineFailure, FailureKind, OracleKind, Verdict, Violation};
