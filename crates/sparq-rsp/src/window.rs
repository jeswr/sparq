//! Window operators (the RSP-QL S2R step): time-based sliding windows
//! (`RANGE w STEP s`) and count-based windows (`ROWS n`).
//!
//! # Time-window semantics (read this; the tests pin it down)
//!
//! * Windows are **half-open intervals `[start, end)`** with `start = k·step`,
//!   `end = start + range`, for `k = 0, 1, 2, …` — i.e. the window origin
//!   `t0` is fixed at timestamp `0` (RSP-QL parameterises `t0`; we don't — a
//!   documented divergence, see the crate README). A triple with timestamp `t`
//!   belongs to window `k` iff `start ≤ t < end`: the start bound is
//!   INCLUSIVE, the end bound EXCLUSIVE, so `RANGE 10 STEP 10` windows
//!   `[0,10) [10,20) …` partition the timeline with no double-counting.
//! * **Closure is watermark-driven, never wall-clock-driven.** The watermark is
//!   `max_ts_seen − max_delay` (saturating). Window `k` CLOSES — its content is
//!   frozen and reported — as soon as the watermark reaches its `end`. Time
//!   only advances through pushed timestamps, so a quiet stream closes nothing
//!   until the next push (or [`WindowedStream::flush`]).
//! * **Out-of-order arrivals** within `max_delay` land in every still-open
//!   window that covers their timestamp. A triple whose LAST covering window
//!   has already closed is dropped and counted in
//!   [`late_dropped`](WindowedStream::late_dropped); if only some earlier
//!   covering windows closed, it still enters the open ones (standard
//!   streaming-system behaviour) and is not counted as late.
//! * **Empty windows are reported.** When the watermark jumps a gap, every
//!   window in between closes with no content — DSTREAM needs this (results
//!   must be observed disappearing), and it keeps emission deterministic.
//!   Windows wholly before the first accepted triple are skipped, so a stream
//!   starting at `ts = 10⁹` does not replay a billion empties.
//! * `step > range` leaves timestamp gaps covered by no window; a triple in a
//!   gap belongs to no window and is silently ignored (it is not "late").
//!
//! # Count-window semantics
//!
//! `ROWS n` follows CQL: the window holds the last `min(n, arrivals)` triples
//! in ARRIVAL order (timestamps are carried but do not influence membership),
//! and is reported on every arrival — or every `slide`-th arrival with
//! [`WindowSpec::with_slide`]. Reported bounds are the **inclusive**
//! `[first.ts, last.ts]` of the content (empty never occurs: a count window is
//! only reported after an arrival). [`flush`](WindowedStream::flush) is a
//! no-op for count windows — there is no watermark to advance.

use std::collections::{BTreeMap, VecDeque};

use oxrdf::Term;

use crate::stream::{TimestampedTriple, TripleStream};

/// A window specification (the S2R operator of RSP-QL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSpec {
    /// `RANGE range STEP step` — sliding (or, when `step == range`, tumbling)
    /// time window. `max_delay` is the out-of-order tolerance: the watermark
    /// lags the maximum seen timestamp by this much, holding windows open for
    /// late arrivals (0 = closed at the first sight of a newer-window triple).
    Time { range: u64, step: u64, max_delay: u64 },
    /// `ROWS rows` — count window over the last `rows` arrivals, reported
    /// every `slide` arrivals (CQL default: every arrival, `slide = 1`).
    Count { rows: usize, slide: usize },
}

impl WindowSpec {
    /// `RANGE range STEP step`, no lateness tolerance.
    ///
    /// # Panics
    /// If `range == 0` or `step == 0` (a zero-width or non-advancing window
    /// is meaningless and would loop forever).
    pub fn time(range: u64, step: u64) -> Self {
        assert!(range > 0, "window RANGE must be > 0");
        assert!(step > 0, "window STEP must be > 0");
        WindowSpec::Time { range, step, max_delay: 0 }
    }

    /// `ROWS rows`, reported on every arrival.
    ///
    /// # Panics
    /// If `rows == 0`.
    pub fn count(rows: usize) -> Self {
        assert!(rows > 0, "window ROWS must be > 0");
        WindowSpec::Count { rows, slide: 1 }
    }

    /// Sets the out-of-order tolerance of a time window.
    ///
    /// # Panics
    /// On a count window (count windows are arrival-ordered; lateness does
    /// not apply).
    pub fn with_max_delay(self, max_delay: u64) -> Self {
        match self {
            WindowSpec::Time { range, step, .. } => WindowSpec::Time { range, step, max_delay },
            WindowSpec::Count { .. } => panic!("max_delay applies to time windows only"),
        }
    }

    /// Sets the report cadence of a count window (report every `slide`
    /// arrivals instead of every arrival).
    ///
    /// # Panics
    /// On a time window, or if `slide == 0`.
    pub fn with_slide(self, slide: usize) -> Self {
        assert!(slide > 0, "window SLIDE must be > 0");
        match self {
            WindowSpec::Count { rows, .. } => WindowSpec::Count { rows, slide },
            WindowSpec::Time { .. } => panic!("slide applies to count windows only (use STEP)"),
        }
    }
}

/// One CLOSED (time) or REPORTED (count) window: bounds + frozen content.
#[derive(Debug, Clone)]
pub struct Window {
    /// Time window: inclusive start `k·step`. Count window: `ts` of the oldest
    /// content triple.
    pub start: u64,
    /// Time window: EXCLUSIVE end `start + range`. Count window: INCLUSIVE
    /// `ts` of the newest content triple.
    pub end: u64,
    /// Content. Time windows: timestamp order (arrival order within equal
    /// timestamps). Count windows: arrival order.
    pub triples: Vec<TimestampedTriple>,
}

/// Maintains the active window content of one stream incrementally and
/// surfaces windows as they close.
///
/// Time windows buffer by timestamp in a `BTreeMap<ts, Vec<triple>>`; closing
/// window `k` is a range read `[k·step, k·step+range)` and sliding evicts every
/// entry older than the next window's start with one `split_off`. Count windows
/// are a ring buffer (`VecDeque`) capped at `rows`.
///
/// Closed windows accumulate internally; drain them with
/// [`take_closed`](Self::take_closed) (or let [`ContinuousQuery`](crate::ContinuousQuery)
/// drive the whole loop).
#[derive(Debug)]
pub struct WindowedStream {
    spec: WindowSpec,
    /// Time-window buffer: triples (in arrival order per timestamp) keyed by ts.
    /// Holds everything from the oldest OPEN window's start upwards.
    buffer: BTreeMap<u64, Vec<[Term; 3]>>,
    /// Index `k` of the oldest window not yet closed; `None` until the first
    /// accepted push fixes the starting window.
    next_close: Option<u64>,
    /// Maximum timestamp seen (watermark = `max_ts − max_delay`).
    max_ts: u64,
    /// Too-late arrivals dropped (every covering window already closed).
    late_dropped: u64,
    /// Count-window ring buffer (last `rows` arrivals).
    ring: VecDeque<TimestampedTriple>,
    /// Count-window arrival counter (drives `slide`).
    arrivals: u64,
    /// Windows closed/reported but not yet taken.
    closed: Vec<Window>,
}

impl WindowedStream {
    /// A windowed view over a scripted stream: every buffered element is
    /// pushed in order. Drain whatever already closed with
    /// [`take_closed`](Self::take_closed); keep pushing live elements with
    /// [`push`](Self::push).
    pub fn new(stream: TripleStream, spec: WindowSpec) -> Self {
        let mut ws = Self::empty(spec);
        for item in stream.into_items() {
            ws.push(item.triple, item.ts);
        }
        ws
    }

    /// A windowed view with no history (live pushes only).
    pub fn empty(spec: WindowSpec) -> Self {
        WindowedStream {
            spec,
            buffer: BTreeMap::new(),
            next_close: None,
            max_ts: 0,
            late_dropped: 0,
            ring: VecDeque::new(),
            arrivals: 0,
            closed: Vec::new(),
        }
    }

    /// The window specification this stream was built with.
    pub fn spec(&self) -> WindowSpec {
        self.spec
    }

    /// Pushes one stream element. Any window this closes is queued for
    /// [`take_closed`](Self::take_closed).
    pub fn push(&mut self, triple: [Term; 3], ts: u64) {
        match self.spec {
            WindowSpec::Time { range, step, max_delay } => {
                self.push_time(triple, ts, range, step, max_delay)
            }
            WindowSpec::Count { rows, slide } => self.push_count(triple, ts, rows, slide),
        }
    }

    fn push_time(&mut self, triple: [Term; 3], ts: u64, range: u64, step: u64, max_delay: u64) {
        // Window k covers [k·step, k·step + range). The LAST window covering
        // ts is k_max = ts / step — valid only if ts actually falls inside it
        // (ts % step < range can fail when step > range: gap timestamps are
        // covered by no window and ignored).
        let k_max = ts / step;
        if ts - k_max * step >= range {
            return; // gap (step > range): in no window
        }
        match self.next_close {
            Some(k_next) if k_max < k_next => {
                // Every window covering ts already closed: too late.
                self.late_dropped += 1;
                return;
            }
            None => {
                // First accepted triple fixes the starting window: the FIRST
                // window covering ts (skip the all-empty prefix of the axis).
                let k_min = if ts >= range { (ts - range) / step + 1 } else { 0 };
                self.next_close = Some(k_min);
            }
            _ => {}
        }
        self.buffer.entry(ts).or_default().push(triple);
        if ts > self.max_ts {
            self.max_ts = ts;
        }
        // Close every window whose end the watermark has reached.
        let watermark = self.max_ts.saturating_sub(max_delay);
        while let Some(k) = self.next_close {
            let start = k * step;
            if watermark < start + range {
                break;
            }
            self.close_time_window(k, range, step);
        }
    }

    /// Freezes window `k`'s content, queues it, slides to `k + 1` and evicts
    /// everything older than the new oldest open window.
    fn close_time_window(&mut self, k: u64, range: u64, step: u64) {
        let start = k * step;
        let end = start + range;
        let triples = self
            .buffer
            .range(start..end)
            .flat_map(|(&ts, ts_triples)| {
                ts_triples.iter().map(move |t| TimestampedTriple { triple: t.clone(), ts })
            })
            .collect();
        self.closed.push(Window { start, end, triples });
        self.next_close = Some(k + 1);
        // Evict: nothing below the next window's start can be read again.
        self.buffer = self.buffer.split_off(&((k + 1) * step));
    }

    fn push_count(&mut self, triple: [Term; 3], ts: u64, rows: usize, slide: usize) {
        self.arrivals += 1;
        self.ring.push_back(TimestampedTriple { triple, ts });
        if self.ring.len() > rows {
            self.ring.pop_front();
        }
        if self.arrivals % slide as u64 == 0 {
            // Reported on arrival, so the ring is never empty here.
            let start = self.ring.front().expect("non-empty ring").ts;
            let end = self.ring.back().expect("non-empty ring").ts;
            self.closed.push(Window { start, end, triples: self.ring.iter().cloned().collect() });
        }
    }

    /// End-of-stream: closes every time window up to and including the last
    /// one covering `max_ts` — regardless of `max_delay` — and returns ALL
    /// pending windows (the flushed ones plus any not yet taken). No-op for
    /// count windows beyond draining. After a flush, further pushes at or
    /// below the flushed horizon count as late.
    pub fn flush(&mut self) -> Vec<Window> {
        if let WindowSpec::Time { range, step, .. } = self.spec {
            while let Some(k) = self.next_close {
                if k * step > self.max_ts {
                    break; // window starts after every triple we ever saw
                }
                self.close_time_window(k, range, step);
            }
        }
        self.take_closed()
    }

    /// Drains the windows that closed since the last call, oldest first.
    pub fn take_closed(&mut self) -> Vec<Window> {
        std::mem::take(&mut self.closed)
    }

    /// How many arrivals were dropped as too late (time windows only).
    pub fn late_dropped(&self) -> u64 {
        self.late_dropped
    }
}
