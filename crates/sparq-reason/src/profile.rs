//! Opt-in reasoning profiler: per-rule evaluation cost + a materialization progress monitor.
//!
//! [SONNET-4.6] sq-6tykl.5 — the "which rule blew up my closure?" instrument (RDFox parity).
//! Behind the non-default `profile` feature: when it is off every hook in the reasoner is
//! `cfg`'d out entirely, so the default build carries zero instrumentation code and the
//! materialised closure is byte-identical in both feature states.
//!
//! # What is measured
//!
//! A [`Profiler`] is installed for the duration of a call with [`with_profiler`] (or, for the
//! common case, via [`crate::materialize_profiled`]). While installed, the materializers report:
//!
//! - **per rule group** — [`RuleStat`]: how many times the group was evaluated
//!   (`fired_count`), how many candidate facts it emitted (`derived_count`) and how long it
//!   took (`wall_time`);
//! - **per fixpoint round** — a [`Progress`] notification to the optional callback, carrying
//!   the round number, the facts newly committed in that round and the running total;
//! - **whole run** — [`Report::rounds`], [`Report::total_derived`], [`Report::wall_time`].
//!
//! ## Honest granularity
//!
//! Attribution is per **rule GROUP**, not per individual entailment rule, because that is the
//! granularity at which this engine actually evaluates: the semi-naive driver fuses rdfs2/3/5/
//! 7/9/11 with prp-inv/symp/eqp/fp/ifp/trp into ONE per-fact emitter so the sweep can fan out
//! over rayon (see `owl::owl_rl_closure`). Splitting the fused emitter into per-rule timers
//! would change the code being measured. The group names are listed in [`rules`].
//!
//! `derived_count` counts the facts a group **emitted into the round's candidate set**, i.e.
//! BEFORE cross-rule deduplication against the closure — that is the number that finds the
//! blow-up (a rule generating millions of duplicates is exactly the offender you are hunting).
//! The net new facts of the whole run are [`Report::total_derived`]; the two are equal only
//! when no rule ever emits a duplicate.
//!
//! # Threading
//!
//! The installed profiler is **thread-local**: it observes the driving thread's phases. Work
//! that rayon fans out to worker threads is attributed to the phase that spawned it (the
//! driver blocks for the duration), which is what a cost attribution wants; the workers
//! themselves record nothing.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Write as _,
    time::{Duration, Instant},
};

/// The stable rule-group names this crate reports. Public so a CLI/GUI can pin its own
/// rendering (and so a rename is a visible API change rather than silent string drift).
pub mod rules {
    /// Building the schema (subClassOf / subPropertyOf / domain / range) view of the closure.
    pub const SCHEMA_INDEX: &str = "schema-index";
    /// Saturating the schema: the subClassOf/subPropertyOf transitive closures + domain/range.
    pub const SCHEMA_SATURATE: &str = "schema-saturate";
    /// The single parallel ABox sweep of the RDFS materializer (rdfs2/3/7/9 per assertion).
    pub const ABOX_SWEEP: &str = "abox-sweep";
    /// The fused semi-naive delta emitter: rdfs2/3/5/7/9/11 + prp-inv/symp/eqp/fp/ifp/trp +
    /// cax-eqc + scm-eqc/eqp, all fired against the previous round's delta.
    pub const DELTA_SWEEP: &str = "delta-sweep";
    /// scm-dom1/2 + scm-rng1/2 (domain/range up subClassOf, down subPropertyOf).
    pub const SCM_DOM_RNG: &str = "scm-dom/rng";
    /// The restriction / list / cardinality / key rules (cls-svf/avf/hv, cls-int, scm-uni,
    /// prp-spo2, prp-key, cls-maxc/maxqc, cls-oo).
    pub const CLASS_FEATURES: &str = "cls-*";
    /// Committing a round's candidates: canonicalize, dedup against the closure, index.
    pub const COMMIT: &str = "commit";
    /// An owl:sameAs merge and the index rebuild it forces.
    pub const SAMEAS_MERGE: &str = "eq-sameas-merge";
    /// Expanding the owl:sameAs equivalence classes back over the closure.
    pub const SAMEAS_EXPAND: &str = "eq-expand";
    /// The final deduplicate + sort of the materialised closure.
    pub const FINALIZE: &str = "finalize";
    /// Incremental maintenance: an insert's delta sweep.
    pub const INCREMENTAL_INSERT: &str = "incremental-insert";
    /// Incremental maintenance: a delete's derivation-count decrement sweep.
    pub const INCREMENTAL_DELETE: &str = "incremental-delete";
    /// Incremental maintenance: a full re-materialization fallback (TBox change).
    pub const INCREMENTAL_REBUILD: &str = "incremental-rebuild";
}

/// Aggregate observations for one rule group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleStat {
    /// Stable rule-group name — one of the constants in [`rules`].
    pub rule: &'static str,
    /// How many times the group was evaluated (once per fixpoint round it ran).
    pub fired_count: usize,
    /// Candidate facts the group emitted, BEFORE cross-rule dedup (see the module docs).
    pub derived_count: usize,
    /// Wall time spent evaluating the group.
    pub wall_time: Duration,
}

/// Materialization progress notification, delivered once per fixpoint round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// 1-based round number.
    pub round: usize,
    /// Facts newly committed to the closure in THIS round.
    pub derived_count: usize,
    /// Facts committed so far across all rounds.
    pub total_derived: usize,
    /// Time since the profiler was installed.
    pub elapsed: Duration,
}

/// Completed instrumentation report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    stats: Vec<RuleStat>,
    rounds: usize,
    total_derived: usize,
    wall_time: Duration,
}

impl Report {
    /// Statistics in stable rule-group-name order.
    pub fn stats(&self) -> &[RuleStat] {
        &self.stats
    }

    /// Fixpoint rounds executed. A single-pass materializer (RDFS, or the OWL-RL fast paths)
    /// reports exactly one.
    pub fn rounds(&self) -> usize {
        self.rounds
    }

    /// Net new facts committed to the closure across the whole run.
    pub fn total_derived(&self) -> usize {
        self.total_derived
    }

    /// Wall time the profiler was installed for.
    pub fn wall_time(&self) -> Duration {
        self.wall_time
    }

    /// The `n` greatest offenders: most candidate facts emitted, ties broken by wall time then
    /// by name so the ordering is deterministic.
    pub fn top(&self, n: usize) -> Vec<&RuleStat> {
        let mut v: Vec<_> = self.stats.iter().collect();
        v.sort_by(|a, b| {
            b.derived_count
                .cmp(&a.derived_count)
                .then_with(|| b.wall_time.cmp(&a.wall_time))
                .then_with(|| a.rule.cmp(b.rule))
        });
        v.truncate(n);
        v
    }

    /// A fixed-width top-`n` offender table for a CLI (`sparq-cli reason --profile`).
    /// Deterministic apart from the timings themselves.
    pub fn to_text_summary(&self, n: usize) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "reasoning profile: {} round(s), {} fact(s) derived, {:.3}ms",
            self.rounds,
            self.total_derived,
            self.wall_time.as_secs_f64() * 1e3
        );
        let _ = writeln!(
            s,
            "  {:<20} {:>8} {:>12} {:>12}",
            "rule", "fired", "emitted", "ms"
        );
        for st in self.top(n) {
            let _ = writeln!(
                s,
                "  {:<20} {:>8} {:>12} {:>12.3}",
                st.rule,
                st.fired_count,
                st.derived_count,
                st.wall_time.as_secs_f64() * 1e3
            );
        }
        s
    }

    /// The machine-readable surface a server / GUI renders. Hand-rolled (this crate carries no
    /// serde dependency); keys are stable and snake_case, matching `sparq-introspect`'s
    /// `to_json` convention. Rule names are the [`rules`] constants, so no escaping is needed —
    /// the writer nonetheless escapes `"` and `\` so the output is valid JSON by construction.
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\"rounds\":");
        let _ = write!(s, "{}", self.rounds);
        let _ = write!(s, ",\"total_derived\":{}", self.total_derived);
        let _ = write!(s, ",\"wall_time_ms\":{:.3}", self.wall_time.as_secs_f64() * 1e3);
        s.push_str(",\"rules\":[");
        for (i, st) in self.stats.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("{\"rule\":\"");
            for c in st.rule.chars() {
                if c == '"' || c == '\\' {
                    s.push('\\');
                }
                s.push(c);
            }
            let _ = write!(
                s,
                "\",\"fired_count\":{},\"derived_count\":{},\"wall_time_ms\":{:.3}}}",
                st.fired_count,
                st.derived_count,
                st.wall_time.as_secs_f64() * 1e3
            );
        }
        s.push_str("]}");
        s
    }
}

/// Accumulates materialization observations. Install it around a materialization with
/// [`with_profiler`] or [`crate::materialize_profiled_with`].
#[derive(Default)]
pub struct Profiler {
    stats: BTreeMap<&'static str, RuleStat>,
    progress: Option<Box<dyn FnMut(Progress) + Send>>,
    rounds: usize,
    total_derived: usize,
    started: Option<Instant>,
}

impl Profiler {
    /// Create a profiler without a progress callback.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a profiler that reports [`Progress`] once per fixpoint round of a batch
    /// materialization, and once per *incrementally maintained* insert/delete. A mutation that
    /// falls back to a full rebuild instead ticks the rounds of the batch run it delegates to
    /// — [`rules::INCREMENTAL_REBUILD`] in [`Report::stats`] is what identifies that case.
    ///
    /// The callback runs on the materializing thread with the profiler temporarily detached, so
    /// it must not itself start a nested materialization; keep it short (a channel send, a
    /// counter bump, a log line).
    pub fn with_progress(callback: impl FnMut(Progress) + Send + 'static) -> Self {
        Self {
            progress: Some(Box::new(callback)),
            ..Self::default()
        }
    }

    fn entry(&mut self, rule: &'static str) -> &mut RuleStat {
        self.stats.entry(rule).or_insert(RuleStat {
            rule,
            fired_count: 0,
            derived_count: 0,
            wall_time: Duration::ZERO,
        })
    }

    fn finish(self) -> Report {
        Report {
            wall_time: self.started.map_or(Duration::ZERO, |t| t.elapsed()),
            rounds: self.rounds,
            total_derived: self.total_derived,
            stats: self.stats.into_values().collect(),
        }
    }
}

thread_local! {
    /// The profiler observing THIS thread, if any. `None` (the overwhelmingly common state)
    /// makes every hook below a single thread-local read.
    static ACTIVE: RefCell<Option<Profiler>> = const { RefCell::new(None) };
}

/// Restores the previously-installed profiler on scope exit, including on unwind.
struct Restore(Option<Profiler>);
impl Drop for Restore {
    fn drop(&mut self) {
        ACTIVE.with(|a| *a.borrow_mut() = self.0.take());
    }
}

/// Run `f` with `profiler` installed on this thread, returning its value and the [`Report`].
///
/// Any reasoning this thread drives inside `f` is instrumented — batch materialization
/// ([`crate::materialize`]) and incremental maintenance
/// ([`crate::MaterializedGraph`] / [`crate::MaterializedOwlGraph`] inserts and deletes) alike.
/// Nesting is supported: the outer profiler is restored on exit and observes nothing that the
/// inner one observed.
///
/// ```
/// # use sparq_core::dict::Dict;
/// # use sparq_reason::{profile, Profile};
/// let mut dict = Dict::new();
/// let mut triples = Vec::new();
/// let (added, report) = profile::with_profiler(profile::Profiler::new(), || {
///     sparq_reason::materialize(Profile::Rdfs, &mut dict, &mut triples)
/// });
/// assert_eq!(added, 0);
/// assert_eq!(report.total_derived(), 0);
/// ```
pub fn with_profiler<R>(profiler: Profiler, f: impl FnOnce() -> R) -> (R, Report) {
    let mut profiler = profiler;
    profiler.started = Some(Instant::now());
    let prev = ACTIVE.with(|a| a.borrow_mut().replace(profiler));
    let restore = Restore(prev);
    let out = f();
    // Take OUR profiler before `restore` puts the previous one back.
    let mine = ACTIVE.with(|a| a.borrow_mut().take());
    drop(restore);
    (out, mine.map_or_else(Report::default, Profiler::finish))
}

/// Start timing a rule group — `None` when no profiler is installed, which makes
/// [`phase`] a no-op and costs one thread-local read instead of a clock call.
pub(crate) fn mark() -> Option<Instant> {
    ACTIVE.with(|a| a.borrow().is_some()).then(Instant::now)
}

/// Record one evaluation of `rule` lasting since `since`.
pub(crate) fn phase(rule: &'static str, since: Option<Instant>) {
    let Some(start) = since else { return };
    let elapsed = start.elapsed();
    ACTIVE.with(|a| {
        if let Some(p) = a.borrow_mut().as_mut() {
            let st = p.entry(rule);
            st.fired_count += 1;
            st.wall_time += elapsed;
        }
    });
}

/// Record `n` candidate facts emitted by `rule`.
pub(crate) fn derived(rule: &'static str, n: usize) {
    ACTIVE.with(|a| {
        if let Some(p) = a.borrow_mut().as_mut() {
            p.entry(rule).derived_count += n;
        }
    });
}

/// Record the completion of one fixpoint round that committed `committed` new facts, and
/// notify the progress callback.
pub(crate) fn round(committed: usize) {
    // The callback is detached across the call so a re-entrant hook cannot hit the RefCell.
    let notify = ACTIVE.with(|a| {
        let mut slot = a.borrow_mut();
        let p = slot.as_mut()?;
        p.rounds += 1;
        p.total_derived += committed;
        let progress = Progress {
            round: p.rounds,
            derived_count: committed,
            total_derived: p.total_derived,
            elapsed: p.started.map_or(Duration::ZERO, |t| t.elapsed()),
        };
        Some((p.progress.take()?, progress))
    });
    if let Some((mut callback, progress)) = notify {
        callback(progress);
        ACTIVE.with(|a| {
            if let Some(p) = a.borrow_mut().as_mut() {
                p.progress = Some(callback);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hooks are no-ops with no profiler installed (so a `profile`-feature build that
    /// never installs one still materialises without recording anything).
    #[test]
    fn hooks_are_inert_without_an_installed_profiler() {
        assert!(mark().is_none());
        phase(rules::COMMIT, None);
        derived(rules::COMMIT, 99);
        round(7);
        let (_, report) = with_profiler(Profiler::new(), || ());
        assert_eq!(report.rounds(), 0, "the pre-install round must not be counted");
        assert!(report.stats().is_empty());
    }

    #[test]
    fn top_orders_by_emitted_then_time_then_name() {
        let (_, report) = with_profiler(Profiler::new(), || {
            derived(rules::COMMIT, 5);
            derived(rules::DELTA_SWEEP, 50);
            derived(rules::SCM_DOM_RNG, 50);
            phase(rules::SCM_DOM_RNG, Some(Instant::now()));
        });
        let top = report.top(2);
        assert_eq!(top.len(), 2);
        // Equal emitted counts: the one with measured wall time wins the tie.
        assert_eq!(top[0].rule, rules::SCM_DOM_RNG);
        assert_eq!(top[1].rule, rules::DELTA_SWEEP);
        assert_eq!(report.top(99).len(), 3, "top() clamps to the stats it has");
    }

    #[test]
    fn progress_callback_sees_every_round_with_running_totals() {
        use std::sync::{Arc, Mutex};
        let seen: Arc<Mutex<Vec<Progress>>> = Arc::default();
        let sink = Arc::clone(&seen);
        let profiler = Profiler::with_progress(move |p| sink.lock().unwrap().push(p));
        let (_, report) = with_profiler(profiler, || {
            round(4);
            round(3);
            round(0);
        });
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 3, "one notification per round");
        assert_eq!(
            seen.iter().map(|p| p.round).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            seen.iter().map(|p| p.total_derived).collect::<Vec<_>>(),
            vec![4, 7, 7],
            "total_derived is the running sum"
        );
        assert_eq!(report.rounds(), 3);
        assert_eq!(report.total_derived(), 7);
    }

    #[test]
    fn fired_and_wall_time_accumulate_per_rule() {
        let (_, report) = with_profiler(Profiler::new(), || {
            for _ in 0..3 {
                let t = mark();
                assert!(t.is_some(), "mark() is live while a profiler is installed");
                phase(rules::DELTA_SWEEP, t);
            }
        });
        let st = &report.stats()[0];
        assert_eq!(st.rule, rules::DELTA_SWEEP);
        assert_eq!(st.fired_count, 3);
        assert_eq!(st.derived_count, 0, "timing alone emits nothing");
    }

    #[test]
    fn nested_profilers_do_not_leak_into_each_other() {
        let (inner, outer) = with_profiler(Profiler::new(), || {
            derived(rules::COMMIT, 1);
            let (_, inner) = with_profiler(Profiler::new(), || derived(rules::DELTA_SWEEP, 10));
            derived(rules::COMMIT, 1);
            inner
        });
        assert_eq!(inner.stats().len(), 1);
        assert_eq!(inner.stats()[0].rule, rules::DELTA_SWEEP);
        assert_eq!(outer.stats().len(), 1, "the inner group must not reach the outer report");
        assert_eq!(outer.stats()[0].derived_count, 2);
    }

    #[test]
    fn an_unwind_restores_the_previous_profiler() {
        let (_, outer) = with_profiler(Profiler::new(), || {
            let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_profiler(Profiler::new(), || panic!("boom"));
            }));
            assert!(panicked.is_err());
            // The outer profiler must be installed again, not lost with the unwound inner one.
            derived(rules::COMMIT, 3);
        });
        assert_eq!(outer.stats().len(), 1);
        assert_eq!(outer.stats()[0].derived_count, 3);
    }

    #[test]
    fn json_surface_is_stable_and_parseable() {
        let (_, report) = with_profiler(Profiler::new(), || {
            derived(rules::DELTA_SWEEP, 12);
            round(5);
        });
        let json = report.to_json();
        assert!(json.starts_with("{\"rounds\":1,\"total_derived\":5,\"wall_time_ms\":"), "{}", json);
        assert!(
            json.contains("\"rules\":[{\"rule\":\"delta-sweep\",\"fired_count\":0,\"derived_count\":12,"),
            "{}",
            json
        );
        assert!(json.ends_with("]}"), "{}", json);
        // Balanced braces/brackets — a cheap well-formedness check without a JSON dep.
        assert_eq!(json.matches('{').count(), json.matches('}').count());
        assert_eq!(json.matches('[').count(), json.matches(']').count());
    }

    #[test]
    fn text_summary_lists_the_top_offenders_only() {
        let (_, report) = with_profiler(Profiler::new(), || {
            derived(rules::DELTA_SWEEP, 900);
            derived(rules::COMMIT, 3);
            derived(rules::SCM_DOM_RNG, 2);
            round(4);
        });
        let text = report.to_text_summary(1);
        assert!(text.contains("1 round(s), 4 fact(s) derived"), "{}", text);
        assert!(text.contains("delta-sweep"), "{}", text);
        assert!(!text.contains("scm-dom/rng"), "top(1) must not spill: {}", text);
    }
}
