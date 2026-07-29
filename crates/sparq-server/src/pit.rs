//! [OPUS-5] (sq-lsp7k.7) **Point-in-time reconstruction** — the pure core behind the opt-in
//! `point-in-time` feature's `?at=<instant>` pin on `/sparql`.
//!
//! The existing `?generation=N` pin (`time-travel`) serves only generations the ring still
//! holds IN MEMORY: recent, count/age-bounded, and lost on restart. This module extends the
//! reach to an ARBITRARY past instant by treating the durable CDC change stream
//! ([`sparq_serve::ChangeLog`] — a segmented, fsync'd, retention-trimmed append-only log) as
//! the persistent temporal index it already is: each record carries the commit's
//! `timestamp_unix_nanos`, the generation it committed TO, and the quad-level inserts/deletes
//! of that commit. Resolving an instant is therefore a scan of the retained records
//! (`resolve_generation`), and materialising the state at that instant is a REVERSE replay
//! of every commit after it, applied to a snapshot the ring still holds (`rewind`).
//!
//! # Why reverse replay (and not forward replay from a base backup)
//!
//! The log records DIFFS, not states, and the only durable state artifacts (the `backup`
//! family) are a separate opt-in with their own lifecycle. The ring, by contrast, ALWAYS
//! holds a recent generation. Rewinding backwards from it needs nothing but the log, so the
//! feature composes with `--change-stream` alone.
//!
//! # Soundness obligations, stated up front
//!
//! * **Reverse order is load-bearing.** Undoing a span of commits is NOT commutative: a quad
//!   deleted at generation 5 and re-inserted at generation 7 must have generation 7's insert
//!   undone FIRST, otherwise undoing 5 re-adds it and undoing 7 then removes it — leaving it
//!   absent when it was present at generation 4. `rewind` iterates the span newest-first;
//!   `rewind_is_order_sensitive` is the witness test that goes red if that is reversed.
//! * **Never approximate.** Every failure is a REFUSAL (`PitError`), never a silent
//!   substitution of a nearby instant. A trimmed horizon, an uncaptured span (an operator
//!   re-base marker), a non-contiguous chain and the CDC lag window (a published-but-unrecorded
//!   commit — `check_unrecorded_commit`) are all distinct, reported errors.
//! * **The line-set representation is exact BECAUSE it is the same serialiser.** The log's
//!   own diff (`change_stream::diff_changes`) renders quads with
//!   [`graph_to_nquads`](sparq_engine::serialize::graph_to_nquads) and compares the resulting
//!   LINES; `rewind` rebuilds the base with the same function, so a recorded `quad` string
//!   matches a base line byte-for-byte or not at all — there is no normalisation gap between
//!   the two sides.
//! * **Inherited blank-node caveat.** The change stream identifies quads by their N-Quads
//!   line, so a blank node is identified by its LABEL. That is exactly as sound (and no more)
//!   as the CDC stream itself: this module introduces no new assumption, but it also cannot
//!   repair one — a store whose blank-node labels are not stable across generations gets a
//!   correspondingly unreliable reconstruction, on the same terms as `GET /streams`.
//!
//! Cost is honest and O(graph): one serialise of the base plus one re-parse of the result per
//! reconstruction, which is why the whole feature is opt-in, runs off the async runtime, and
//! is cached by the caller for the repeat-request (pagination) case.

use std::collections::BTreeSet;

use sparq_core::Graph;
use sparq_engine::serialize::graph_to_nquads;
use sparq_serve::{ChangeOp, ChangeRecord};

/// Why a point-in-time request could not be served. Every variant is a REFUSAL — the
/// reconstruction never falls back to a different instant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PitError {
    /// The `at=` value is neither an `xsd:dateTime` / RFC-3339 lexical nor a non-negative
    /// integer count of nanoseconds since the Unix epoch. → `400`.
    Malformed,
    /// The change log retains no records at all, so no past state is reconstructable. → `410`.
    Empty,
    /// The instant predates the oldest RETAINED change record — older segments were trimmed
    /// by the retention policy, or recording started later. Carries the oldest instant that
    /// IS reconstructable, in the same nanosecond form `at=` accepts, so the client can retry
    /// at the horizon rather than guess. → `410`.
    BeforeHorizon { earliest_unix_nanos: u128 },
    /// An operator re-base marker ([`ChangeRecord::rebase`]) sits inside the span being
    /// undone: that span was never captured, so the earlier state cannot be rebuilt from the
    /// log. → `410`.
    Gap { generation: u64 },
    /// The ring has PUBLISHED a generation the log has not recorded yet — the CDC hook appends
    /// after the publish — and the requested instant resolves to the log's tail, so that commit
    /// cannot be shown to fall after it. Carries both numbers. Retryable, unlike every other
    /// refusal here. → `503`.
    Unrecorded { published: u64, recorded: u64 },
    /// The retained records do not form the contiguous chain from the target generation up to
    /// the base snapshot's generation (a dropped record, or a log that never covered it). A
    /// server-state condition, not a client error. → `500`.
    Uncovered,
    /// The reconstructed N-Quads did not re-parse. Should be unreachable (the input is this
    /// workspace's own serialiser output); reported rather than papered over. → `500`.
    Rebuild(String),
}

/// Parses the `at=` parameter into nanoseconds since the Unix epoch.
///
/// Two accepted forms, disambiguated without ambiguity because an `xsd:dateTime` lexical
/// always contains a `T`:
///
/// * an **`xsd:dateTime` / RFC-3339 instant** — `2026-07-28T12:00:00Z`,
///   `2026-07-28T13:00:00+01:00`, fractional seconds allowed. A lexical with NO timezone
///   offset is read as UTC (the XSD "floating" reading has no absolute instant, and this
///   endpoint must resolve one — documented, not silent).
/// * a **non-negative integer** of nanoseconds since the Unix epoch — byte-identical to the
///   `commitTimestampNanos` field `GET /streams` emits, so a consumer that saw a record on
///   the CDC feed can time-travel to it by copying the value across.
///
/// Returns [`PitError::Malformed`] for anything else. Instants before 1970 are representable
/// (the result is negative) but can never resolve to a record, so they surface as
/// [`PitError::BeforeHorizon`] downstream rather than as a parse error.
pub(crate) fn parse_instant(raw: &str) -> Result<i128, PitError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(PitError::Malformed);
    }
    if raw.bytes().all(|b| b.is_ascii_digit()) {
        return raw.parse::<i128>().map_err(|_| PitError::Malformed);
    }
    let tl = sparq_core::temporal::Timeline::parse_datetime(raw).ok_or(PitError::Malformed)?;
    // `secs` is the LOCAL wall-clock second count; subtracting the offset yields UTC. An
    // absent offset is read as UTC (see the doc comment).
    let utc_secs = i128::from(tl.secs) - i128::from(tl.tz.unwrap_or(0));
    // `frac` is in [0, 1) by construction (`f64::fract` of the seconds field).
    let frac_nanos = (tl.frac * 1e9).round() as i128;
    Ok(utc_secs * 1_000_000_000 + frac_nanos)
}

/// Resolves an instant to the generation whose published state IS the store's state at that
/// instant: the newest retained record with `timestamp_unix_nanos <= target`.
///
/// `records` must be the retained log in ascending `seq` order (what
/// [`ChangeLog::poll`](sparq_serve::ChangeLog::poll) returns). An instant at or after the
/// newest record resolves to that newest record — the store has not changed since.
///
/// Refuses rather than approximating: an instant older than every retained record is
/// [`PitError::BeforeHorizon`], never silently clamped to the oldest one.
pub(crate) fn resolve_generation(records: &[ChangeRecord], target: i128) -> Result<u64, PitError> {
    let Some(first) = records.first() else {
        return Err(PitError::Empty);
    };
    records
        .iter()
        .rev()
        .find(|r| (r.timestamp_unix_nanos as i128) <= target)
        .map(|r| r.generation)
        .ok_or(PitError::BeforeHorizon {
            earliest_unix_nanos: first.timestamp_unix_nanos,
        })
}

/// Which generation's snapshot [`rewind`] must start from, given the retained records and the
/// ring's CURRENT generation number.
///
/// Normally the log's newest record names the current generation and the answer is simply
/// "current". The `min` matters at both edges:
///
/// * the CDC hook appends AFTER the ring publishes, so a commit can be current-but-unrecorded
///   for a moment — starting from `current` then would leave a span the log cannot undo, so
///   the base steps back to the newest RECORDED generation (which the ring still retains: the
///   concurrency floor keeps the last K ≥ 4). Stepping back keeps the REBUILD sound; whether the
///   requested instant may be answered at all in that window is [`check_unrecorded_commit`]'s
///   call;
/// * conversely a commit may land between pinning `current` and polling, putting records
///   BEYOND the pin in the log — those are simply not part of the span.
pub(crate) fn base_generation(records: &[ChangeRecord], current: u64) -> u64 {
    records
        .last()
        .map_or(current, |r| r.generation.min(current))
}

/// Fail-closed guard for the CDC lag window — the case [`base_generation`]'s step back cannot
/// resolve on its own.
///
/// While the ring holds a published-but-unrecorded commit, [`resolve_generation`] can only answer
/// with a RECORDED generation, so an instant at or after the log's tail resolves to the state
/// BEFORE that commit. The unrecorded commit has no recorded timestamp yet, so it cannot be shown
/// to fall after the requested instant — answering would be exactly the silent approximation this
/// module refuses. The request is [`PitError::Unrecorded`] instead: retryable, because the append
/// happens within the commit that triggered it (the writer acks the update only after the hook
/// returns), so the window closes on its own.
///
/// An instant resolving STRICTLY BEFORE the newest recorded generation is provably unaffected —
/// generations commit in order, so the unrecorded commit is newer than the record that already
/// answers the instant — and is served normally. So is the opposite edge, records BEYOND
/// `current` (a commit that landed between the pin and the poll): there the log LEADS the ring
/// rather than lagging it, and the extra records are simply outside the span.
///
/// A non-empty `records` is the caller's precondition — [`resolve_generation`] refuses an empty
/// log with [`PitError::Empty`] before this runs, and an empty log is nothing this guard can say
/// anything about.
pub(crate) fn check_unrecorded_commit(
    records: &[ChangeRecord],
    current: u64,
    target_generation: u64,
) -> Result<(), PitError> {
    let Some(recorded) = records.last().map(|r| r.generation) else {
        return Ok(());
    };
    if recorded < current && target_generation >= recorded {
        return Err(PitError::Unrecorded {
            published: current,
            recorded,
        });
    }
    Ok(())
}

/// Reconstructs the store's state at `target_generation` by reverse-replaying every recorded
/// commit in `(target_generation, base_generation]` against `base` (the snapshot published at
/// `base_generation`).
///
/// The span must be fully and contiguously covered by `records`; anything else is a typed
/// refusal (see [`PitError`]). `target_generation == base_generation` is the identity case and
/// rebuilds `base` unchanged.
pub(crate) fn rewind(
    base: &Graph,
    base_generation: u64,
    records: &[ChangeRecord],
    target_generation: u64,
) -> Result<Graph, PitError> {
    if target_generation > base_generation {
        return Err(PitError::Uncovered);
    }

    let span: Vec<&ChangeRecord> = records
        .iter()
        .filter(|r| r.generation > target_generation && r.generation <= base_generation)
        .collect();

    // The span must start exactly one generation after the target and end exactly at the
    // base, with no hole in between — otherwise some commit in it is unknown and the rewind
    // would silently produce a state that never existed.
    if target_generation != base_generation {
        if span.first().map(|r| r.generation) != Some(target_generation + 1)
            || span.last().map(|r| r.generation) != Some(base_generation)
        {
            return Err(PitError::Uncovered);
        }
        if span
            .windows(2)
            .any(|w| w[1].generation != w[0].generation + 1)
        {
            return Err(PitError::Uncovered);
        }
    }
    // A re-base marker is an HONEST GAP: the commits it stands for were never captured, so
    // nothing before it can be rebuilt from the log.
    if let Some(gap) = span.iter().find(|r| r.rebase) {
        return Err(PitError::Gap {
            generation: gap.generation,
        });
    }

    let mut lines: BTreeSet<String> = graph_to_nquads(base)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect();
    // NEWEST FIRST — see the module docs: undoing a span is not commutative.
    for record in span.iter().rev() {
        for change in &record.changes {
            match change.op {
                // The commit inserted it, so it was absent before the commit.
                ChangeOp::Insert => {
                    lines.remove(&change.quad);
                }
                // The commit deleted it, so it was present before the commit.
                ChangeOp::Delete => {
                    lines.insert(change.quad.clone());
                }
            }
        }
    }

    let mut nquads = String::new();
    for line in &lines {
        nquads.push_str(line);
        nquads.push('\n');
    }
    Graph::load_dataset(&nquads, "nquads").map_err(PitError::Rebuild)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_serve::Change;

    fn record(seq: u64, generation: u64, ts: u128, changes: Vec<Change>) -> ChangeRecord {
        ChangeRecord {
            seq,
            generation,
            timestamp_unix_nanos: ts,
            changes,
            rebase: false,
        }
    }

    fn ins(quad: &str) -> Change {
        Change {
            op: ChangeOp::Insert,
            quad: quad.to_string(),
        }
    }

    fn del(quad: &str) -> Change {
        Change {
            op: ChangeOp::Delete,
            quad: quad.to_string(),
        }
    }

    const A: &str = "<http://ex/a> <http://ex/p> <http://ex/o> .";
    const B: &str = "<http://ex/b> <http://ex/p> <http://ex/o> .";

    fn graph_of(lines: &[&str]) -> Graph {
        Graph::load_dataset(&lines.join("\n"), "nquads").expect("fixture parses")
    }

    /// `Graph` implements neither `Debug` nor `PartialEq`, so a refusal is asserted on the
    /// extracted error rather than on the whole `Result`.
    fn refusal(result: Result<Graph, PitError>) -> PitError {
        match result {
            Ok(graph) => panic!(
                "expected a refusal, got a reconstruction of {} quads",
                subjects(&graph).len()
            ),
            Err(e) => e,
        }
    }

    fn subjects(graph: &Graph) -> Vec<String> {
        let mut out: Vec<String> = graph_to_nquads(graph)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect();
        out.sort();
        out
    }

    #[test]
    fn parse_instant_accepts_rfc3339_utc() {
        // 2026-07-28T00:00:00Z is 20662 days after the epoch (56 years x 365 + 14 leap days to
        // 2026-01-01, plus 208 days into 2026).
        let expected = 20662i128 * 86_400 * 1_000_000_000;
        assert_eq!(parse_instant("2026-07-28T00:00:00Z"), Ok(expected));
    }

    #[test]
    fn parse_instant_applies_the_timezone_offset() {
        let utc = parse_instant("2026-07-28T12:00:00Z").expect("utc parses");
        let plus_one = parse_instant("2026-07-28T13:00:00+01:00").expect("offset parses");
        assert_eq!(utc, plus_one, "the same absolute instant");
    }

    #[test]
    fn parse_instant_accepts_bare_unix_nanos() {
        assert_eq!(parse_instant("1234567890"), Ok(1_234_567_890));
    }

    #[test]
    fn parse_instant_rejects_nonsense() {
        assert_eq!(parse_instant("yesterday"), Err(PitError::Malformed));
        assert_eq!(parse_instant(""), Err(PitError::Malformed));
        assert_eq!(parse_instant("2026-07-28"), Err(PitError::Malformed));
    }

    #[test]
    fn resolve_generation_picks_the_newest_record_at_or_before_the_instant() {
        let records = vec![
            record(0, 1, 1_000, vec![]),
            record(1, 2, 2_000, vec![]),
            record(2, 3, 3_000, vec![]),
        ];
        assert_eq!(resolve_generation(&records, 2_500), Ok(2));
        assert_eq!(resolve_generation(&records, 2_000), Ok(2), "inclusive");
        assert_eq!(resolve_generation(&records, 9_999), Ok(3), "after the tail");
    }

    #[test]
    fn resolve_generation_refuses_before_the_retained_horizon() {
        let records = vec![record(7, 8, 5_000, vec![])];
        assert_eq!(
            resolve_generation(&records, 4_999),
            Err(PitError::BeforeHorizon {
                earliest_unix_nanos: 5_000
            }),
            "never clamped to the oldest retained record"
        );
        assert_eq!(resolve_generation(&[], 1), Err(PitError::Empty));
    }

    #[test]
    fn base_generation_steps_back_to_the_newest_recorded_commit() {
        let records = vec![record(0, 1, 1, vec![]), record(1, 2, 2, vec![])];
        // The log lags the ring (the hook has not appended generation 3 yet).
        assert_eq!(base_generation(&records, 3), 2);
        // A commit landed after the pin — the extra record is simply out of the span.
        assert_eq!(base_generation(&records, 1), 1);
        assert_eq!(base_generation(&[], 4), 4, "no records: current is the base");
    }

    /// The CDC lag window is a REFUSAL, not the preceding state. The ring publishes before the
    /// hook appends, so it can hold a commit no record describes; an instant at or after the
    /// log's tail may or may not include that commit, and this surface refuses rather than
    /// answering at the older one. Narrow the `>=` to `>` (or drop the guard) and the
    /// published-but-unrecorded case below goes green on exactly that silent substitution.
    #[test]
    fn check_unrecorded_commit_refuses_inside_the_cdc_lag_window() {
        let records = vec![record(0, 1, 1, vec![]), record(1, 2, 2, vec![])];
        // Steady state — the log has recorded everything the ring published.
        assert_eq!(check_unrecorded_commit(&records, 2, 2), Ok(()));
        assert_eq!(check_unrecorded_commit(&records, 2, 1), Ok(()));
        // Generation 3 is published but not appended yet: an instant resolving to the tail is at
        // or after generation 2's record, so generation 3 cannot be shown to postdate it.
        assert_eq!(
            check_unrecorded_commit(&records, 3, 2),
            Err(PitError::Unrecorded {
                published: 3,
                recorded: 2
            })
        );
        // … but an instant resolving STRICTLY before the tail is provably unaffected: generation
        // 3 commits after generation 2, which already answers that instant.
        assert_eq!(check_unrecorded_commit(&records, 3, 1), Ok(()));
        // The opposite edge — a commit landed between the pin and the poll, so the log LEADS the
        // ring. Not a lag, and not a refusal.
        assert_eq!(check_unrecorded_commit(&records, 1, 1), Ok(()));
        // An empty log is the caller's precondition (`resolve_generation` refuses it first).
        assert_eq!(check_unrecorded_commit(&[], 3, 0), Ok(()));
    }

    #[test]
    fn rewind_undoes_inserts_and_deletes() {
        // gen 1: {A}. gen 2 inserted B. gen 3 deleted A.
        let base = graph_of(&[B]);
        let records = vec![
            record(0, 2, 2_000, vec![ins(B)]),
            record(1, 3, 3_000, vec![del(A)]),
        ];
        let at_gen_1 = rewind(&base, 3, &records, 1).expect("covered span rewinds");
        assert_eq!(subjects(&at_gen_1), vec![A.to_string()]);
    }

    /// THE order witness (see the module docs). A quad deleted at generation 2 and re-inserted
    /// at generation 3 must be PRESENT at generation 1. Undoing the span oldest-first instead
    /// of newest-first re-adds it at step one and then removes it at step two, yielding an
    /// absent quad — a state that never existed. Flip the `.rev()` in `rewind` and this test
    /// goes red.
    #[test]
    fn rewind_is_order_sensitive() {
        // gen 1: {A}. gen 2 deleted A. gen 3 re-inserted A. Base (gen 3) therefore holds A.
        let base = graph_of(&[A]);
        let records = vec![
            record(0, 2, 2_000, vec![del(A)]),
            record(1, 3, 3_000, vec![ins(A)]),
        ];
        let at_gen_1 = rewind(&base, 3, &records, 1).expect("covered span rewinds");
        assert_eq!(
            subjects(&at_gen_1),
            vec![A.to_string()],
            "A was present at generation 1; a commutative (order-blind) undo loses it"
        );
    }

    #[test]
    fn rewind_identity_at_the_base_generation() {
        let base = graph_of(&[A, B]);
        let records = vec![record(0, 3, 3_000, vec![ins(B)])];
        let same = rewind(&base, 3, &records, 3).expect("identity rewinds");
        assert_eq!(subjects(&same), subjects(&base));
    }

    #[test]
    fn rewind_refuses_an_uncovered_span() {
        let base = graph_of(&[A]);
        // The span (1, 4] needs generations 2, 3 and 4; generation 3 is missing.
        let records = vec![
            record(0, 2, 2_000, vec![ins(A)]),
            record(1, 4, 4_000, vec![ins(B)]),
        ];
        assert_eq!(refusal(rewind(&base, 4, &records, 1)), PitError::Uncovered);
        // Nothing at all covering the span.
        assert_eq!(refusal(rewind(&base, 4, &[], 1)), PitError::Uncovered);
        // A target ahead of the base is not a rewind at all.
        assert_eq!(refusal(rewind(&base, 2, &records, 5)), PitError::Uncovered);
    }

    #[test]
    fn rewind_refuses_across_a_rebase_gap() {
        let base = graph_of(&[A]);
        let mut gap = record(1, 3, 3_000, vec![]);
        gap.rebase = true;
        let records = vec![record(0, 2, 2_000, vec![ins(A)]), gap];
        assert_eq!(
            refusal(rewind(&base, 3, &records, 1)),
            PitError::Gap { generation: 3 },
            "an uncaptured span is refused, never replayed as 'no changes'"
        );
    }
}
