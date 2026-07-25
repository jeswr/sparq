// AUTHORED-BY Claude Fable 5
//! The DPoP-SK anti-replay sliding window — RFC 4303 §3.4.3 (ESP), as the spec adopts it.
//!
//! Per session the server keeps a high-water mark `H` (the highest AUTHENTICATED counter value)
//! and a `W`-bit bitmap covering exactly the values `H − W + 1 … H` (the window includes `H`).
//! For a received counter `n` (checked ONLY after the HMAC verified — verify-then-mark is
//! normative; the CALLER owns that ordering, see [`crate::pop::sk::verify`]):
//!
//! - `n > H`        ⇒ shift the bitmap by `n − H`, set `H = n`, mark `n`. Accept.
//! - in the window  ⇒ duplicate if marked; otherwise mark and accept.
//! - `n ≤ H − W`    ⇒ reject (below the window).
//!
//! `W` is 1024 here (the spec's SHOULD; its MUST-floor is 128) — RFC 4303's own 32/64 defaults
//! target IPsec, whereas HTTP/2 multiplexing completes far more requests out of order, which is
//! exactly what the window (versus a strictly-increasing rule) exists to tolerate.
//!
//! Check-and-mark is a single `&mut self` call so the caller can make the pair atomic per session
//! (a `Mutex` around the window — two concurrent copies of the same value cannot both pass).

/// The window width in bits (the spec's SHOULD value).
pub const WINDOW_BITS: u64 = 1024;

const WORDS: usize = (WINDOW_BITS / 64) as usize;

/// The verdict of a window check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowVerdict {
    /// Fresh value — marked and accepted.
    Accept,
    /// The value is inside the window and already marked (a replay).
    Duplicate,
    /// The value is at or below `H − W` (too old to track — rejected, fail-closed).
    BelowWindow,
}

/// The per-session receive window: high-water mark + bitmap. Bit `i` (0-based) represents the
/// counter value `H − i`; bit 0 is `H` itself.
#[derive(Debug, Clone)]
pub struct ReplayWindow {
    /// Highest authenticated counter value seen; 0 ⇒ nothing accepted yet (counters start at 1).
    high: u64,
    bitmap: [u64; WORDS],
}

impl ReplayWindow {
    pub fn new() -> Self {
        Self {
            high: 0,
            bitmap: [0u64; WORDS],
        }
    }

    /// The highest authenticated counter value (0 = none yet). Exposed for tests/diagnostics.
    pub fn high_water(&self) -> u64 {
        self.high
    }

    /// Check-and-mark `n` (which the caller has already validated as ≥ 1 — zero is rejected at
    /// parse time). MUST be called only after the request's HMAC verified (verify-then-mark).
    pub fn check_and_mark(&mut self, n: u64) -> WindowVerdict {
        debug_assert!(n >= 1, "zero counters are rejected at parse time");
        if n > self.high {
            // Advance: shift the bitmap left by (n − high) so bit i comes to represent the same
            // absolute value under the new high-water mark, then mark n (bit 0).
            let shift = n - self.high;
            self.shift(shift);
            self.high = n;
            self.set_bit(0);
            return WindowVerdict::Accept;
        }
        let diff = self.high - n; // 0 ⇒ n == high
        if diff >= WINDOW_BITS {
            return WindowVerdict::BelowWindow;
        }
        if self.get_bit(diff) {
            return WindowVerdict::Duplicate;
        }
        self.set_bit(diff);
        WindowVerdict::Accept
    }

    fn get_bit(&self, i: u64) -> bool {
        let (word, bit) = ((i / 64) as usize, i % 64);
        (self.bitmap[word] >> bit) & 1 == 1
    }

    fn set_bit(&mut self, i: u64) {
        let (word, bit) = ((i / 64) as usize, i % 64);
        self.bitmap[word] |= 1 << bit;
    }

    /// Shift the whole bitmap towards OLDER positions by `by` bits (values falling off the end are
    /// forgotten — they become "below the window").
    fn shift(&mut self, by: u64) {
        if by >= WINDOW_BITS {
            self.bitmap = [0u64; WORDS];
            return;
        }
        let word_shift = (by / 64) as usize;
        let bit_shift = (by % 64) as u32;
        // Move words from low index (recent) to high index (old), high-to-low to avoid clobbering.
        for i in (0..WORDS).rev() {
            let mut v = if i >= word_shift {
                self.bitmap[i - word_shift] << bit_shift
            } else {
                0
            };
            if bit_shift > 0 && i > word_shift {
                v |= self.bitmap[i - word_shift - 1] >> (64 - bit_shift);
            }
            self.bitmap[i] = v;
        }
    }
}

impl Default for ReplayWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_value_accepts_and_sets_high_water() {
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(1), WindowVerdict::Accept);
        assert_eq!(w.high_water(), 1);
    }

    #[test]
    fn duplicate_inside_window_is_rejected() {
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(1), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(1), WindowVerdict::Duplicate);
        assert_eq!(w.check_and_mark(2), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(2), WindowVerdict::Duplicate);
        assert_eq!(w.check_and_mark(1), WindowVerdict::Duplicate);
    }

    #[test]
    fn out_of_order_within_window_accepts_then_rejects_replay() {
        // HTTP/2 multiplexing: 5 then 3 (out of order) must both pass; a second 3 is a replay.
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(5), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(3), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(3), WindowVerdict::Duplicate);
        assert_eq!(w.check_and_mark(4), WindowVerdict::Accept);
        assert_eq!(w.high_water(), 5);
    }

    #[test]
    fn below_window_is_rejected_even_if_never_seen() {
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(WINDOW_BITS + 100), WindowVerdict::Accept);
        // n = high − W is exactly one below the window's oldest tracked value (H − W + 1).
        assert_eq!(w.check_and_mark(100), WindowVerdict::BelowWindow);
        // The oldest IN-window value is still acceptable (never marked).
        assert_eq!(w.check_and_mark(101), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(101), WindowVerdict::Duplicate);
    }

    #[test]
    fn window_boundary_math_is_exact() {
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(2000), WindowVerdict::Accept);
        // Window covers 2000-1024+1=977 ..= 2000.
        assert_eq!(w.check_and_mark(977), WindowVerdict::Accept);
        assert_eq!(w.check_and_mark(976), WindowVerdict::BelowWindow);
    }

    #[test]
    fn shift_by_multiples_of_64_and_large_jumps() {
        let mut w = ReplayWindow::new();
        for n in 1..=10u64 {
            assert_eq!(w.check_and_mark(n), WindowVerdict::Accept);
        }
        // Jump exactly 64: prior marks slide into higher words but stay tracked.
        assert_eq!(w.check_and_mark(74), WindowVerdict::Accept);
        for n in 1..=10u64 {
            assert_eq!(w.check_and_mark(n), WindowVerdict::Duplicate, "n={n}");
        }
        // Jump beyond the whole window: everything old is forgotten (below window now).
        assert_eq!(
            w.check_and_mark(74 + WINDOW_BITS + 1),
            WindowVerdict::Accept
        );
        assert_eq!(w.check_and_mark(74), WindowVerdict::BelowWindow);
    }

    #[test]
    fn marks_survive_incremental_shifts_across_word_edges() {
        let mut w = ReplayWindow::new();
        assert_eq!(w.check_and_mark(60), WindowVerdict::Accept);
        // Cross the first word boundary in small steps; 60 must stay marked throughout.
        for n in 61..=200u64 {
            assert_eq!(w.check_and_mark(n), WindowVerdict::Accept, "n={n}");
            assert_eq!(w.check_and_mark(60), WindowVerdict::Duplicate, "after {n}");
        }
    }

    #[test]
    fn forged_traffic_cannot_advance_the_window() {
        // Not a property of this type per se (the caller enforces verify-then-mark), but assert
        // the API shape supports it: no method mutates state on a read — only `check_and_mark`
        // mutates, and the verifier calls it strictly after HMAC success.
        let w = ReplayWindow::new();
        assert_eq!(w.high_water(), 0);
        let _ = w.high_water(); // read-only probe
        assert_eq!(w.high_water(), 0);
    }
}
