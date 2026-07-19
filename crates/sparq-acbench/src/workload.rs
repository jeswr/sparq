//! Workload + oracle engine (W1–W4) — bead `sq-i6du2.6`.
//!
//! This module implements the four benchmark workloads defined in §2.1 of
//! `research/ac-query-benchmark.md`, each gated on the by-construction oracle:
//!
//! - **W1 decision micro-benchmark** — [`W1DecisionBatch`]: batches of
//!   `(agent, client, resource, mode)` requests checked against pre-computed expected
//!   decisions via the model oracle.
//! - **W2 access-controlled query** — [`W2QueryLane`]: closed-form expected result sets
//!   checked against the authorized-subset the oracle admits.
//! - **W3 ACL-write + invalidation churn** — [`W3ChurnScript`]: interleaved
//!   grant/revoke/policy-add writes with expected post-write decision *deltas*; a stale
//!   grant surviving a revoke is a benchmark FAILURE, not a speedup.
//! - **W4 concurrent readers** — [`W4Config`]: N threads of W1 batches (the decision
//!   sub-lane runs today); the query sub-lane is [`RunOutcome::Skipped`] with a recorded
//!   reason (see below).
//!
//! # Fail-closed harness contract (the credibility feature)
//! A benchmark an unsound implementation can *win* is worthless. Therefore:
//! - Any decision mismatch (oracle ≠ expected) ⇒ [`RunOutcome::Failed`]; a
//!   [`HarnessReport`] with any failed lane exits non-zero ([`HarnessReport::exit_code`]).
//! - Any result-set mismatch (W2) ⇒ [`RunOutcome::Failed`].
//! - Any post-churn stale grant / missing grant (W3) ⇒ [`RunOutcome::Failed`].
//! - **No timing is emitted for a lane that failed correctness** — the harness records
//!   the mismatch, never a wall-clock number, for a red lane (`§2.3`).
//! - The **anti-vacuity** fixture (`tests/workloads.rs`) proves the harness *does* fail on
//!   a deliberately-wrong expected set — the oracle's oracle. Without it, a harness that
//!   always passes would be indistinguishable from a correct one.
//!
//! # W4 query sub-lane status (re-checked 2026-07-06)
//! The design record (§2.1 W4, §4.4) gated the W4 *query* sub-lane on the `sparq-solid`
//! `&self` read-side (#1569, PR #1612). **That PR has now MERGED**, so the API-level
//! blocker is gone. However, this crate is a dev-only **oracle** crate with a load-bearing
//! *zero dependency on `sparq-core` / `sparq-engine` / `sparq-solid`* (Cargo.toml) — it
//! cannot link the system under test. Driving a live `PodStore::query_as` concurrently
//! therefore belongs to the `sparq-solid`-linked live DRIVER under `bench/ac/` — bead
//! `sq-kvvcl`, the FILES-scoped owner of that driver (surfaced by the #1627 review:
//! `sq-i6du2.7` is scripts/registry-only and `.9` is RESULTS-only, so neither actually
//! links the SUT). It is NOT this oracle crate's job. The W4 query sub-lane is consequently
//! [`RunOutcome::Skipped`] here with the accurate reason
//! [`W4_QUERY_SKIP_REASON`] — never a fabricated concurrency number.
//!
//! # File ownership
//! **Only bead `sq-i6du2.6` edits this file.**

use crate::{oracle, AcModel, Decision, GenParams, IntentRow, QueryClass, Request};

/// The recorded skip reason for the W4 query sub-lane.
///
/// The `&self` read-side (#1569 / PR #1612) has merged, so the reason is NOT "blocked on
/// an unmerged dependency". The remaining blocker is architectural: this oracle crate is
/// forbidden (by the Cargo.toml zero-dependency constraint) from linking `sparq-solid`, so
/// the concurrent live-`query_as` driver lives outside this crate — in the FILES-scoped
/// live driver bead `sq-kvvcl` under `bench/ac/`. This string is asserted in
/// `tests/workloads.rs` so the reason cannot silently drift back to a false claim.
pub const W4_QUERY_SKIP_REASON: &str =
    "W4 query sub-lane runs in the sparq-solid-linked bench/ac live driver (sq-kvvcl), \
     not this zero-dependency oracle crate; #1569/#1612 (the &self read-side) has merged, \
     so the API blocker is gone but this crate must not link the system under test";

/// The outcome of one workload lane run.
#[derive(Debug, Clone)]
pub enum RunOutcome {
    /// The lane passed all oracle checks.
    Passed {
        /// Number of decisions (or result rows) evaluated.
        decisions: usize,
        /// Hint at wall-clock; NON-CANONICAL on a work-box (labels required by harness).
        wall_us_indicative: u64,
    },
    /// The lane failed an oracle check.
    Failed {
        /// Human-readable description of the first mismatch found.
        mismatch: String,
    },
    /// The lane was explicitly skipped (blocked on a dependency).
    Skipped {
        /// Reason for skip (e.g. [`W4_QUERY_SKIP_REASON`]).
        reason: String,
    },
}

impl RunOutcome {
    /// Returns `true` iff the outcome is a pass. Skipped is neither pass nor fail.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, RunOutcome::Passed { .. })
    }

    /// Returns `true` iff the outcome is a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self, RunOutcome::Failed { .. })
    }

    /// Returns `true` iff the outcome is a skip.
    #[must_use]
    pub fn is_skipped(&self) -> bool {
        matches!(self, RunOutcome::Skipped { .. })
    }

    /// The mismatch description if this is a [`RunOutcome::Failed`], else `None`.
    #[must_use]
    pub fn mismatch(&self) -> Option<&str> {
        match self {
            RunOutcome::Failed { mismatch } => Some(mismatch.as_str()),
            _ => None,
        }
    }
}

// ── W1: decision micro-benchmark ──────────────────────────────────────────────────────

/// A W1 decision-batch workload: batches of `(agent, client, resource, mode)` tuples
/// checked against the by-construction oracle.
///
/// The `intents` table is the ground truth; `expected` is the generator's pre-computed
/// decision for each request (from its `ExpectedDecision` rows). The lane passes iff, for
/// every request, the *oracle re-evaluated over `intents`* equals the pre-computed
/// `expected`. This double-checks the generator against the shared oracle and gives the
/// workload engine a single decision path (`oracle::evaluate`) it can also run
/// concurrently in W4.
pub struct W1DecisionBatch {
    /// The requests to evaluate.
    pub requests: Vec<Request>,
    /// The expected decision for each request (parallel to `requests`).
    pub expected: Vec<Decision>,
    /// The model to evaluate against.
    pub model: AcModel,
    /// The by-construction intent table the oracle reads (ground truth).
    pub intents: Vec<IntentRow>,
}

impl W1DecisionBatch {
    /// Run this batch through the by-construction oracle and return a [`RunOutcome`].
    ///
    /// For each request, the model oracle is evaluated over [`Self::intents`] and compared
    /// against the parallel [`Self::expected`] entry.
    ///
    /// # Fail-closed
    /// - A `requests` / `expected` length mismatch is a harness wiring bug ⇒
    ///   [`RunOutcome::Failed`] (never silently truncated).
    /// - The FIRST `(request, model)` whose oracle decision differs from `expected` ⇒
    ///   [`RunOutcome::Failed`] with a diagnostic; **no timing** is reported for a failed
    ///   lane.
    ///
    /// # Determinism
    /// Pure function of the batch contents (no clock affects the pass/fail verdict; the
    /// indicative wall-clock is measured but never gates correctness).
    #[must_use]
    pub fn run_oracle(&self) -> RunOutcome {
        if self.requests.len() != self.expected.len() {
            return RunOutcome::Failed {
                mismatch: format!(
                    "W1 wiring bug: {} requests vs {} expected decisions (must be equal)",
                    self.requests.len(),
                    self.expected.len()
                ),
            };
        }

        let start = std::time::Instant::now();
        for (i, (request, want)) in self.requests.iter().zip(self.expected.iter()).enumerate() {
            let got = oracle::evaluate(&self.model, request, &self.intents);
            if got != *want {
                // Correctness failure: report the mismatch, NEVER a timing number.
                return RunOutcome::Failed {
                    mismatch: format!(
                        "W1 decision mismatch at index {i} ({:?}): oracle={got:?} expected={want:?} \
                         for agent={} resource={} mode={:?}",
                        self.model, request.agent, request.resource, request.mode
                    ),
                };
            }
        }
        // Correctness held: only NOW may we report an (indicative) wall-clock.
        let wall_us_indicative = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        RunOutcome::Passed {
            decisions: self.requests.len(),
            wall_us_indicative,
        }
    }
}

// ── W2: access-controlled query result-set oracle ─────────────────────────────────────

/// A single W2 query check: a set of *candidate* result rows the query would return over
/// the full dataset, and the set the requesting agent is *authorized* to see.
///
/// The by-construction expected result set is
/// `candidate_rows ∩ authorized_rows` — the query answer restricted to graphs the oracle
/// admits for the agent. The lane passes iff the engine-produced `produced_rows` equals
/// that expected set (as a sorted, de-duplicated multiset of N-Quads strings).
///
/// In this oracle crate the "engine-produced" rows are the generator's closed-form
/// `expected_rows` — so the lane validates the generator's result set against the oracle's
/// authorized subset. A `sparq-solid`-linked driver (`bench/ac/`) later substitutes the
/// real `query_as` output for `produced_rows` unchanged.
#[derive(Debug, Clone)]
pub struct W2QueryCheck {
    /// The W2 query class this check exercises.
    pub class: QueryClass,
    /// The access-control model.
    pub model: AcModel,
    /// Rows the query returns over the FULL (unauthorized) dataset, sorted N-Quads.
    pub candidate_rows: Vec<String>,
    /// Rows the requesting agent is authorized to observe, sorted N-Quads.
    pub authorized_rows: Vec<String>,
    /// The rows actually produced (by the generator here; by `query_as` in the driver).
    pub produced_rows: Vec<String>,
}

impl W2QueryCheck {
    /// The by-construction expected result set: authorized ∩ candidate, sorted+deduped.
    #[must_use]
    pub fn expected_rows(&self) -> Vec<String> {
        let authorized: std::collections::BTreeSet<&String> = self.authorized_rows.iter().collect();
        let mut out: Vec<String> = self
            .candidate_rows
            .iter()
            .filter(|r| authorized.contains(r))
            .cloned()
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Compare the produced rows against the by-construction expected set.
    ///
    /// # Fail-closed
    /// Any element the engine returned that the agent is NOT authorized to see (an
    /// over-share) — or any authorized row the engine dropped — is a [`RunOutcome::Failed`].
    #[must_use]
    fn check(&self) -> RunOutcome {
        let expected = self.expected_rows();
        let mut produced = self.produced_rows.clone();
        produced.sort();
        produced.dedup();

        if produced != expected {
            // Diagnose the first divergence for a legible failure message.
            let over: Vec<&String> = produced.iter().filter(|r| !expected.contains(r)).collect();
            let under: Vec<&String> = expected.iter().filter(|r| !produced.contains(r)).collect();
            return RunOutcome::Failed {
                mismatch: format!(
                    "W2 {:?}/{:?} result-set mismatch: {} over-shared row(s) (e.g. {:?}), \
                     {} missing row(s) (e.g. {:?})",
                    self.class,
                    self.model,
                    over.len(),
                    over.first(),
                    under.len(),
                    under.first()
                ),
            };
        }
        RunOutcome::Passed {
            decisions: expected.len(),
            wall_us_indicative: 0,
        }
    }
}

/// A W2 lane: a sequence of [`W2QueryCheck`]s across the four query classes.
pub struct W2QueryLane {
    /// The per-query checks in this lane.
    pub checks: Vec<W2QueryCheck>,
}

impl W2QueryLane {
    /// Run every query check; the lane fails on the FIRST mismatch (fail-closed).
    ///
    /// A passing lane reports the total number of result rows validated; a failing lane
    /// reports only the mismatch (no timing).
    #[must_use]
    pub fn run_oracle(&self) -> RunOutcome {
        let start = std::time::Instant::now();
        let mut rows = 0usize;
        for check in &self.checks {
            match check.check() {
                RunOutcome::Passed { decisions, .. } => rows += decisions,
                failed @ RunOutcome::Failed { .. } => return failed,
                RunOutcome::Skipped { reason } => return RunOutcome::Skipped { reason },
            }
        }
        let wall_us_indicative = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        RunOutcome::Passed {
            decisions: rows,
            wall_us_indicative,
        }
    }
}

// ── W3: ACL-write + invalidation churn ────────────────────────────────────────────────

/// A single ACL-write operation in a W3 churn script.
///
/// Each write mutates the intent table; the oracle re-evaluates over the *post-write*
/// table, so a grant that should have been revoked but survives (a stale grant) is caught
/// as a decision mismatch — the whole point of the invalidation lane.
#[derive(Debug, Clone)]
pub enum AclWrite {
    /// Add (grant) an intent row.
    Grant(IntentRow),
    /// Remove every intent row equal to this one (revoke). Idempotent if absent.
    Revoke(IntentRow),
}

/// The expected decision for one probe after a specific write step.
///
/// A W3 probe is checked against the intent table *as of* [`Self::after_step`] writes
/// applied — that is what makes a stale grant observable: the probe expects `Deny` after a
/// revoke, and if the table (or a system under test) still grants, the lane fails.
#[derive(Debug, Clone)]
pub struct ChurnProbe {
    /// The request to evaluate.
    pub request: Request,
    /// The number of writes that must have been applied before this probe is checked.
    pub after_step: usize,
    /// The expected decision at that point.
    pub expected: Decision,
}

/// A W3 churn script: an ordered list of writes interleaved with expected-decision probes.
pub struct W3ChurnScript {
    /// The intent table BEFORE any write is applied.
    pub initial_intents: Vec<IntentRow>,
    /// The ordered writes to apply.
    pub writes: Vec<AclWrite>,
    /// Probes checked at their `after_step` checkpoints.
    pub probes: Vec<ChurnProbe>,
    /// The model to evaluate against.
    pub model: AcModel,
}

impl W3ChurnScript {
    /// Replay the churn script, checking every probe against the intent table as of its
    /// checkpoint.
    ///
    /// # Fail-closed
    /// - A probe whose `after_step` exceeds the number of writes is a wiring bug ⇒
    ///   [`RunOutcome::Failed`].
    /// - The FIRST probe whose oracle decision (over the post-step table) differs from
    ///   `expected` — a stale or missing grant — ⇒ [`RunOutcome::Failed`], no timing.
    ///
    /// # Determinism
    /// Writes are applied in order; the intent table is rebuilt deterministically at each
    /// checkpoint (no dependence on iteration order beyond the given `writes` sequence).
    #[must_use]
    pub fn run_oracle(&self) -> RunOutcome {
        // Validate probe checkpoints up-front (a probe past the end is a wiring bug).
        for (i, probe) in self.probes.iter().enumerate() {
            if probe.after_step > self.writes.len() {
                return RunOutcome::Failed {
                    mismatch: format!(
                        "W3 wiring bug: probe {i} after_step={} > {} total writes",
                        probe.after_step,
                        self.writes.len()
                    ),
                };
            }
        }

        let start = std::time::Instant::now();
        let mut checked = 0usize;
        // For each distinct checkpoint we need, materialise the table and check its probes.
        // We rebuild from `initial_intents` for each checkpoint: O(writes²) but tiny, and
        // guarantees no cross-checkpoint state leakage.
        for step in 0..=self.writes.len() {
            let table = self.table_after(step);
            for probe in self.probes.iter().filter(|p| p.after_step == step) {
                let got = oracle::evaluate(&self.model, &probe.request, &table);
                if got != probe.expected {
                    return RunOutcome::Failed {
                        mismatch: format!(
                            "W3 stale/missing grant after step {step} ({:?}): oracle={got:?} \
                             expected={:?} for agent={} resource={}",
                            self.model, probe.expected, probe.request.agent, probe.request.resource
                        ),
                    };
                }
                checked += 1;
            }
        }

        if checked != self.probes.len() {
            // Every probe must have been reached; an unreached probe means the harness
            // silently skipped a check — treat as a failure, not a pass.
            return RunOutcome::Failed {
                mismatch: format!(
                    "W3 wiring bug: {checked} of {} probes checked (some checkpoint unreached)",
                    self.probes.len()
                ),
            };
        }
        let wall_us_indicative = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        RunOutcome::Passed {
            decisions: checked,
            wall_us_indicative,
        }
    }

    /// The intent table after applying the first `step` writes to `initial_intents`.
    fn table_after(&self, step: usize) -> Vec<IntentRow> {
        let mut table = self.initial_intents.clone();
        for write in self.writes.iter().take(step) {
            match write {
                AclWrite::Grant(row) => table.push(row.clone()),
                AclWrite::Revoke(row) => table.retain(|r| r != row),
            }
        }
        table
    }
}

// ── W4: concurrent readers ────────────────────────────────────────────────────────────

/// Configuration for a W4 concurrent-reader run.
pub struct W4Config {
    /// Number of parallel reader threads.
    pub n_threads: usize,
    /// Number of W1 batches per thread.
    pub batches_per_thread: usize,
    /// The model to evaluate against.
    pub model: AcModel,
    /// Scale factor (from `GenParams::sf`).
    pub sf: u32,
}

/// The two-part outcome of a W4 run: the decision sub-lane (runs today) and the query
/// sub-lane (skipped in this crate — see [`W4_QUERY_SKIP_REASON`]).
#[derive(Debug, Clone)]
pub struct W4Outcome {
    /// Concurrent decision sub-lane (N threads of W1 batches via the oracle).
    pub decision_lane: RunOutcome,
    /// Concurrent query sub-lane — [`RunOutcome::Skipped`] in this oracle crate.
    pub query_lane: RunOutcome,
}

impl W4Config {
    /// Run the W4 concurrent-reader workload against a shared intent table.
    ///
    /// The decision sub-lane spawns [`Self::n_threads`] threads, each running
    /// [`Self::batches_per_thread`] copies of the given `batch` through the by-construction
    /// oracle concurrently over a shared (`Arc`) intent table — the oracle is a pure
    /// `&`-read, so concurrent readers are sound by construction (this mirrors the
    /// `sparq-solid` `&self` `decide_batch` read-side). Any thread that observes a decision
    /// mismatch fails the whole sub-lane (fail-closed). If ALL threads agree with the
    /// single-threaded oracle, the concurrent result is byte-identical (determinism under
    /// concurrency — a correctness property, not a timing one).
    ///
    /// The query sub-lane is [`RunOutcome::Skipped`] here; see [`W4_QUERY_SKIP_REASON`].
    ///
    /// # Panics
    /// Does not panic on a worker thread panic — a panicked join is converted to a
    /// [`RunOutcome::Failed`] so the harness stays fail-closed.
    #[must_use]
    pub fn run(&self, batch: &W1DecisionBatch) -> W4Outcome {
        W4Outcome {
            decision_lane: self.run_decision_lane(batch),
            query_lane: RunOutcome::Skipped {
                reason: W4_QUERY_SKIP_REASON.to_string(),
            },
        }
    }

    /// Convenience: run W4 with a batch derived from a [`GenParams`]-sized synthetic
    /// probe. Retained for the scaffold `run(&params)` shape; delegates to
    /// [`Self::run`] with an empty batch (which trivially passes) so callers that only
    /// need the *skip* semantics of the query sub-lane can call it without a corpus.
    #[must_use]
    pub fn run_params(&self, _params: &GenParams) -> W4Outcome {
        let empty = W1DecisionBatch {
            requests: vec![],
            expected: vec![],
            model: self.model.clone(),
            intents: vec![],
        };
        self.run(&empty)
    }

    /// Run the concurrent decision sub-lane over a shared intent table.
    fn run_decision_lane(&self, batch: &W1DecisionBatch) -> RunOutcome {
        use std::sync::Arc;
        use std::thread;

        if batch.requests.len() != batch.expected.len() {
            return RunOutcome::Failed {
                mismatch: format!(
                    "W4 wiring bug: {} requests vs {} expected decisions",
                    batch.requests.len(),
                    batch.expected.len()
                ),
            };
        }

        // Shared, immutable state: the oracle is a pure read, so an Arc suffices — no lock.
        let intents = Arc::new(batch.intents.clone());
        let requests = Arc::new(batch.requests.clone());
        let expected = Arc::new(batch.expected.clone());
        let model = Arc::new(batch.model.clone());

        let start = std::time::Instant::now();
        let mut handles = Vec::with_capacity(self.n_threads);
        for _ in 0..self.n_threads {
            let intents = Arc::clone(&intents);
            let requests = Arc::clone(&requests);
            let expected = Arc::clone(&expected);
            let model = Arc::clone(&model);
            let batches = self.batches_per_thread;
            handles.push(thread::spawn(move || -> Result<usize, String> {
                let mut n = 0usize;
                for _ in 0..batches {
                    for (i, (req, want)) in requests.iter().zip(expected.iter()).enumerate() {
                        let got = oracle::evaluate(&model, req, &intents);
                        if got != *want {
                            return Err(format!(
                                "W4 concurrent decision mismatch at index {i}: oracle={got:?} \
                                 expected={want:?} for agent={} resource={}",
                                req.agent, req.resource
                            ));
                        }
                        n += 1;
                    }
                }
                Ok(n)
            }));
        }

        let mut total = 0usize;
        for handle in handles {
            match handle.join() {
                Ok(Ok(n)) => total += n,
                Ok(Err(mismatch)) => return RunOutcome::Failed { mismatch },
                Err(_) => {
                    return RunOutcome::Failed {
                        mismatch: "W4 worker thread panicked (fail-closed)".to_string(),
                    };
                }
            }
        }
        let wall_us_indicative = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        RunOutcome::Passed {
            decisions: total,
            wall_us_indicative,
        }
    }
}

// ── Harness aggregator (fail-closed exit code) ────────────────────────────────────────

/// One lane's result in a full harness run, labelled with its workload + model.
#[derive(Debug, Clone)]
pub struct LaneReport {
    /// A short label, e.g. `"W1/personal/WAC"`.
    pub label: String,
    /// The lane's outcome.
    pub outcome: RunOutcome,
}

/// The aggregate report over all lanes in a harness run.
///
/// The harness is **fail-closed**: [`Self::exit_code`] is non-zero if ANY lane failed.
/// A `Skipped` lane (e.g. the W4 query sub-lane) is neither a pass nor a fail and does
/// not, by itself, make the run non-zero — but it is recorded so no skip is silent.
#[derive(Debug, Clone, Default)]
pub struct HarnessReport {
    /// The per-lane results, in run order.
    pub lanes: Vec<LaneReport>,
}

impl HarnessReport {
    /// Create an empty report.
    #[must_use]
    pub fn new() -> Self {
        Self { lanes: Vec::new() }
    }

    /// Record one lane's outcome.
    pub fn record(&mut self, label: impl Into<String>, outcome: RunOutcome) {
        self.lanes.push(LaneReport {
            label: label.into(),
            outcome,
        });
    }

    /// `true` iff any recorded lane is a [`RunOutcome::Failed`].
    #[must_use]
    pub fn any_failed(&self) -> bool {
        self.lanes.iter().any(|l| l.outcome.is_failure())
    }

    /// The process exit code: `1` if any lane failed, else `0` (fail-closed).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        i32::from(self.any_failed())
    }

    /// The number of failed lanes.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.lanes.iter().filter(|l| l.outcome.is_failure()).count()
    }

    /// The number of skipped lanes.
    #[must_use]
    pub fn skipped_count(&self) -> usize {
        self.lanes.iter().filter(|l| l.outcome.is_skipped()).count()
    }
}
