//! Opt-in materialization profiling.
//!
//! Implements the core instrumentation proposed in
//! `research/reasoner-program-differential-and-instrumentation.md`.

use sparq_core::dict::Id;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

/// Aggregate observations for one rule or fused rule group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleStat {
    /// Stable rule name.
    pub rule: &'static str,
    /// Evaluation count.
    pub fired_count: usize,
    /// Newly committed facts.
    pub derived_count: usize,
    /// Wall time spent evaluating the rule.
    pub wall_time: Duration,
}

/// Materialization progress notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub round: usize,
    pub derived_count: usize,
}

/// Completed instrumentation report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    stats: Vec<RuleStat>,
}
impl Report {
    /// Statistics in stable rule-name order.
    pub fn stats(&self) -> &[RuleStat] {
        &self.stats
    }
    /// The `n` greatest sources of derived facts.
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
}

/// Accumulates materialization observations.
#[derive(Default)]
pub struct Profiler {
    stats: BTreeMap<&'static str, RuleStat>,
    progress: Option<Box<dyn FnMut(Progress) + Send>>,
}
impl Profiler {
    /// Create a profiler without a progress callback.
    pub fn new() -> Self {
        Self::default()
    }
    /// Create a profiler that reports progress.
    pub fn with_progress(callback: impl FnMut(Progress) + Send + 'static) -> Self {
        Self {
            stats: BTreeMap::new(),
            progress: Some(Box::new(callback)),
        }
    }
    pub(crate) fn observe(
        &mut self,
        rule: &'static str,
        f: impl FnOnce(&mut Vec<[Id; 3]>) -> usize,
        triples: &mut Vec<[Id; 3]>,
    ) -> usize {
        let start = Instant::now();
        let added = f(triples);
        let s = self.stats.entry(rule).or_insert(RuleStat {
            rule,
            fired_count: 0,
            derived_count: 0,
            wall_time: Duration::ZERO,
        });
        s.fired_count += 1;
        s.derived_count += added;
        s.wall_time += start.elapsed();
        if let Some(cb) = &mut self.progress {
            cb(Progress {
                round: 1,
                derived_count: added,
            });
        }
        added
    }
    pub(crate) fn finish(self) -> Report {
        Report {
            stats: self.stats.into_values().collect(),
        }
    }
}
