// AUTHORED-BY Claude Opus 4.8
//! Single-range `Range: bytes=…` request handling (RFC 9110 §14).
//!
//! Pure value logic: given a `Range` header value and the resource length, compute the satisfied
//! byte interval (for a 206 + `Content-Range`), decide it is unsatisfiable (416), or decide the
//! header should be ignored and the full body returned (200).
//!
//! Scope (M2 slice): a SINGLE byte range only — `bytes=a-b`, `bytes=a-` (open-ended), and
//! `bytes=-n` (suffix). MULTIPART/multiple ranges (`multipart/byteranges`) are M2-next: a request
//! that lists more than one range is treated as "ignore the header, return 200 full" (a spec-allowed
//! response — a server MAY ignore a Range header), never a wrong partial. A syntactically invalid
//! Range is likewise ignored (RFC 9110 §14.2: "an invalid ranges-specifier … the recipient … MUST
//! ignore the Range header field").

/// The outcome of evaluating a `Range` header against a resource of known length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeOutcome {
    /// No (usable) Range — serve the full body with 200.
    Full,
    /// A single satisfiable range `[start, end]` inclusive — serve 206 + `Content-Range`.
    Satisfied { start: u64, end: u64 },
    /// The range was syntactically valid but cannot be satisfied for this length — 416.
    Unsatisfiable,
}

impl RangeOutcome {
    /// The inclusive `[start, end]` of a satisfied range, if any.
    pub fn interval(self) -> Option<(u64, u64)> {
        match self {
            RangeOutcome::Satisfied { start, end } => Some((start, end)),
            _ => None,
        }
    }
}

/// Evaluate a `Range` header value against a resource of `len` bytes.
///
/// Only `bytes` ranges are understood; any other unit is ignored (→ [`RangeOutcome::Full`]). See the
/// module docs for the single-range scope.
pub fn evaluate(range: Option<&str>, len: u64) -> RangeOutcome {
    let raw = match range {
        None => return RangeOutcome::Full,
        Some(r) => r.trim(),
    };

    // Must be `bytes=<ranges>`; any other unit ⇒ ignore.
    let specs = match raw.strip_prefix("bytes=") {
        Some(s) => s.trim(),
        None => return RangeOutcome::Full,
    };

    // Multiple ranges are out of scope for this slice — ignore the header (return full, never wrong).
    if specs.contains(',') {
        return RangeOutcome::Full;
    }

    let spec = specs.trim();
    let (first, last) = match spec.split_once('-') {
        Some(parts) => parts,
        None => return RangeOutcome::Full, // not a valid range-spec ⇒ ignore.
    };
    let first = first.trim();
    let last = last.trim();

    // A zero-length resource cannot satisfy any byte range.
    if len == 0 {
        // A suffix or any range against an empty resource is unsatisfiable per RFC 9110 §14.1.2.
        return RangeOutcome::Unsatisfiable;
    }

    let last_index = len - 1;

    match (first.is_empty(), last.is_empty()) {
        // `bytes=-N` — the final N bytes (suffix range).
        (true, false) => {
            let n: u64 = match last.parse() {
                Ok(n) => n,
                Err(_) => return RangeOutcome::Full, // malformed ⇒ ignore.
            };
            if n == 0 {
                // `bytes=-0` is unsatisfiable (RFC 9110 §14.1.2: a suffix-length of 0 is invalid).
                return RangeOutcome::Unsatisfiable;
            }
            let start = len.saturating_sub(n);
            RangeOutcome::Satisfied {
                start,
                end: last_index,
            }
        }
        // `bytes=A-` — from A to the end.
        (false, true) => {
            let start: u64 = match first.parse() {
                Ok(s) => s,
                Err(_) => return RangeOutcome::Full,
            };
            if start > last_index {
                return RangeOutcome::Unsatisfiable;
            }
            RangeOutcome::Satisfied {
                start,
                end: last_index,
            }
        }
        // `bytes=A-B` — A through B inclusive.
        (false, false) => {
            let start: u64 = match first.parse() {
                Ok(s) => s,
                Err(_) => return RangeOutcome::Full,
            };
            let end_req: u64 = match last.parse() {
                Ok(e) => e,
                Err(_) => return RangeOutcome::Full,
            };
            if start > end_req {
                // An inverted range is invalid ⇒ ignore the header.
                return RangeOutcome::Full;
            }
            if start > last_index {
                return RangeOutcome::Unsatisfiable;
            }
            // Clamp the end to the last byte (RFC 9110 §14.1.2: a too-large end is clamped, not 416).
            let end = end_req.min(last_index);
            RangeOutcome::Satisfied { start, end }
        }
        // `bytes=-` — both empty: malformed ⇒ ignore.
        (true, true) => RangeOutcome::Full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_range_is_full() {
        assert_eq!(evaluate(None, 10), RangeOutcome::Full);
    }

    #[test]
    fn closed_range() {
        assert_eq!(
            evaluate(Some("bytes=2-5"), 10),
            RangeOutcome::Satisfied { start: 2, end: 5 }
        );
    }

    #[test]
    fn open_ended_range_to_end() {
        assert_eq!(
            evaluate(Some("bytes=3-"), 10),
            RangeOutcome::Satisfied { start: 3, end: 9 }
        );
    }

    #[test]
    fn suffix_range() {
        assert_eq!(
            evaluate(Some("bytes=-4"), 10),
            RangeOutcome::Satisfied { start: 6, end: 9 }
        );
        // A suffix larger than the resource clamps to the whole resource.
        assert_eq!(
            evaluate(Some("bytes=-100"), 10),
            RangeOutcome::Satisfied { start: 0, end: 9 }
        );
    }

    #[test]
    fn end_is_clamped_to_last_byte() {
        assert_eq!(
            evaluate(Some("bytes=5-1000"), 10),
            RangeOutcome::Satisfied { start: 5, end: 9 }
        );
    }

    #[test]
    fn start_past_end_is_unsatisfiable() {
        assert_eq!(
            evaluate(Some("bytes=20-30"), 10),
            RangeOutcome::Unsatisfiable
        );
        assert_eq!(evaluate(Some("bytes=10-"), 10), RangeOutcome::Unsatisfiable);
    }

    #[test]
    fn zero_length_resource_is_unsatisfiable() {
        assert_eq!(evaluate(Some("bytes=0-0"), 0), RangeOutcome::Unsatisfiable);
    }

    #[test]
    fn suffix_zero_is_unsatisfiable() {
        assert_eq!(evaluate(Some("bytes=-0"), 10), RangeOutcome::Unsatisfiable);
    }

    #[test]
    fn inverted_range_is_ignored() {
        assert_eq!(evaluate(Some("bytes=8-2"), 10), RangeOutcome::Full);
    }

    #[test]
    fn non_bytes_unit_is_ignored() {
        assert_eq!(evaluate(Some("items=0-5"), 10), RangeOutcome::Full);
    }

    #[test]
    fn multiple_ranges_are_ignored_not_wrong() {
        // Out of scope for this slice — must NOT return a wrong single partial; return full.
        assert_eq!(evaluate(Some("bytes=0-1,3-4"), 10), RangeOutcome::Full);
    }

    #[test]
    fn malformed_is_ignored() {
        assert_eq!(evaluate(Some("bytes=abc-def"), 10), RangeOutcome::Full);
        assert_eq!(evaluate(Some("bytes=-"), 10), RangeOutcome::Full);
        assert_eq!(evaluate(Some("garbage"), 10), RangeOutcome::Full);
    }

    #[test]
    fn interval_accessor() {
        assert_eq!(
            RangeOutcome::Satisfied { start: 1, end: 3 }.interval(),
            Some((1, 3))
        );
        assert_eq!(RangeOutcome::Full.interval(), None);
        assert_eq!(RangeOutcome::Unsatisfiable.interval(), None);
    }
}
