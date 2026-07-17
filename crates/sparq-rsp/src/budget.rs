//! [SONNET-4.6] sq-xqu: per-registered-query budget plumbing — the
//! [`QueryBudget`] a continuous query applies to EVERY window evaluation,
//! plus the optional per-window timeout that refreshes the budget's deadline
//! at each evaluation start (a stored `deadline` is an ABSOLUTE instant, which
//! is the wrong shape for a long-lived registered query).

use sparq_engine::QueryBudget;

/// The registered budget configuration shared by all continuous-query forms.
///
/// `base` is applied to every window evaluation as-is: the row / byte caps and
/// the cancellation flag are naturally per-evaluation limits, while a
/// `deadline` inside it stays absolute (once it passes, every later window
/// fails). `per_window_timeout` is the relative form embedders actually want:
/// at each window evaluation start the effective deadline becomes
/// `now + timeout`, bounding the worst-case cost of ONE window.
#[derive(Debug, Clone, Default)]
pub(crate) struct BudgetSpec {
    pub base: QueryBudget,
    #[cfg(not(target_arch = "wasm32"))]
    pub per_window_timeout: Option<std::time::Duration>,
}

impl BudgetSpec {
    /// The effective [`QueryBudget`] for one window evaluation: `base`, with
    /// the deadline refreshed from `per_window_timeout` (measured from NOW,
    /// the moment this window's evaluation starts). A timeout so large that
    /// `now + timeout` is unrepresentable can never trip, so it is treated as
    /// unlimited: `base`'s own absolute deadline (if any) is kept instead of
    /// panicking on `Instant` overflow.
    pub fn window_budget(&self) -> QueryBudget {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(t) = self.per_window_timeout {
            let mut b = self.base.clone();
            if let Some(deadline) = std::time::Instant::now().checked_add(t) {
                b.deadline = Some(deadline);
            }
            return b;
        }
        self.base.clone()
    }
}
