#!/usr/bin/env python3
# [FABLE-5] (sq-hmd7l.20) Unit tests for the bounded count-matched-replay comparator
# (rsp4j_compare.py). stdlib-only + FAST (no JVM, no cargo) — the house style of
# scripts/bench-adapters/test_adapters.py.
#
# The load-bearing assertions:
#   1. FIDELITY GUARD — the independent reference evaluator re-derives every
#      scenario's per-window row counts from the PINNED REPLAY FILES and they must
#      equal bench/rsp/expected.tsv (which run.sh pins to the in-code oracle script).
#      This chains replay-file -> expected.tsv -> rsp_oracle.rs: a drifted export
#      fails here loudly.
#   2. The GATE INVARIANT — no timing row unless every non-excluded window
#      count-matches, and the time-model caveat is machine-attached to every row.
#      Both directions are exercised (a corrupted count stream MUST fail the gate).
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import rsp4j_compare as rc  # noqa: E402

EXPECTED = os.path.join(HERE, "expected.tsv")
REPLAY_SINGLE = os.path.join(HERE, "replay", "single_window.ts.tsv")
REPLAY_SRBENCH = os.path.join(HERE, "replay", "srbench.ts.tsv")

PASS = 0
FAIL = 0


def check(name, got, want):
    global PASS, FAIL
    if got == want:
        PASS += 1
        print("ok   %-58s = %r" % (name, got))
    else:
        FAIL += 1
        print("FAIL %-58s got %r want %r" % (name, got, want))


# --- 1. replay parsing -------------------------------------------------------
def test_parse_replay():
    single = rc.parse_replay(REPLAY_SINGLE)
    srbench = rc.parse_replay(REPLAY_SRBENCH)
    check("replay.single.events", len(single), 13)
    check("replay.single.max_ts", max(e[0] for e in single), 41)
    check("replay.srbench.events", len(srbench), 19)
    check("replay.srbench.max_ts", max(e[0] for e in srbench), 35)
    check("replay.srbench.streams", {e[1] for e in srbench},
          {"<http://ex/obs>", "<http://ex/meta>"})


# --- 2. FIDELITY GUARD: reference evaluator vs expected.tsv ------------------
def test_ref_counts_match_oracle():
    single = rc.parse_replay(REPLAY_SINGLE)
    srbench = rc.parse_replay(REPLAY_SRBENCH)
    for scenario, events in [
        ("tumbling_avg", single),
        ("sliding_sum", single),
        ("tumbling_groupby_join", single),
        ("srbench_join", srbench),
        ("srbench_groupby_state", srbench),
    ]:
        want = rc.oracle_counts(EXPECTED, scenario)
        got = rc.ref_counts(events, scenario)
        check("fidelity.%s" % scenario, got, want)
    # the three EvalModes are pinned identical in expected.tsv — assert that premise
    for mode in ("rebuild", "pdict", "delta"):
        check("oracle.sliding_sum.%s" % mode,
              rc.oracle_counts(EXPECTED, "sliding_sum", mode), [2, 1, 2, 2, 1])


# --- 3. report -> window mapping --------------------------------------------
def test_map_reports():
    # tumbling 10/10, scale 1000: heartbeat report at C*1000+1 evaluates window C-10..C
    mapped, dupes, leading = rc.map_reports_to_windows(
        [(10001, 3), (20001, 2), (30001, 2)], 10, 10, 1000
    )
    check("map.tumbling.mapped", mapped, {0: 3, 1: 2, 2: 2})
    check("map.tumbling.dupes", dupes, [])
    # a real element at t_e=21000 evaluates the same window as a heartbeat at 20001
    mapped, dupes, _ = rc.map_reports_to_windows([(20001, 2), (21000, 2)], 10, 10, 1000)
    check("map.same_window_agreeing_reports", (mapped, dupes), ({1: 2}, []))
    # DISAGREEING duplicate reports for one window must be flagged, not silently merged
    _, dupes, _ = rc.map_reports_to_windows([(20001, 2), (21000, 5)], 10, 10, 1000)
    check("map.conflicting_reports_flagged", len(dupes), 1)
    # C-SPARQL partial leading windows (k < 0) are protocol exclusions: sliding 20/10,
    # a report at t_e=10001 evaluates [-10,10) -> k=-1
    mapped, _, leading = rc.map_reports_to_windows([(10001, 2)], 20, 10, 1000)
    check("map.leading_partial_excluded", (mapped, len(leading)), ({}, 1))


# --- 4. the count-match gate -------------------------------------------------
def test_gate():
    oracle = [3, 2, 2, 0]
    # full agreement, w3 via missing-as-zero (empty RSTREAM emits nothing)
    win, exc, extra, ok = rc.count_match(oracle, {0: 3, 1: 2, 2: 2}, {}, True)
    check("gate.match_with_missing_zero", ok, True)
    check("gate.w3_policy_recorded", "policy" in win["w3"], True)
    # a mismatching window fails the gate
    win, _, _, ok = rc.count_match(oracle, {0: 3, 1: 9, 2: 2}, {}, True)
    check("gate.mismatch_fails", ok, False)
    check("gate.mismatch_window_recorded", win["w1"]["match"], False)
    # a DECLARED exclusion is honoured and reported, remaining windows still gate
    win, exc, _, ok = rc.count_match(oracle, {0: 3, 2: 2}, {"w1": "engine cannot close"}, True)
    check("gate.declared_exclusion", (ok, list(exc)), (True, ["w1"]))
    # extra trailing competitor windows are surfaced, not silently dropped
    _, _, extra, _ = rc.count_match(oracle, {0: 3, 1: 2, 2: 2, 7: 1}, {}, True)
    check("gate.extra_windows_reported", extra, {"w7": 1})
    # without missing-as-zero a missing window is a mismatch
    _, _, _, ok = rc.count_match(oracle, {0: 3, 1: 2, 2: 2}, {}, False)
    check("gate.strict_missing_fails", ok, False)


# --- 5. end-to-end CLI: envelope + INVARIANT ---------------------------------
def run_cli(args):
    return subprocess.run(
        [sys.executable, os.path.join(HERE, "rsp4j_compare.py")] + args,
        capture_output=True, text=True, timeout=60,
    )


def test_cli_end_to_end():
    with tempfile.TemporaryDirectory() as td:
        comp = os.path.join(td, "competitor.tsv")
        env = os.path.join(td, "envelope.json")
        # competitor counts derived INDEPENDENTLY from the pinned replay file
        r = run_cli(["ref-counts", "--replay", REPLAY_SRBENCH, "--scenario", "srbench_join",
                     "--raw-reports"])
        check("cli.ref_counts.exit", r.returncode, 0)
        with open(comp, "w") as f:
            f.write(r.stdout)
            f.write("timing\trsp_replay_push_triples_per_s\t12345\ttriples_per_s\n")
        # the smoke path: count-match one scenario window-for-window vs the oracle
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--replay", REPLAY_SRBENCH, "--out", env])
        check("cli.count_match.exit", r.returncode, 0)
        e = json.load(open(env))
        check("cli.verdict", e["verdict"], "COUNT-MATCHED")
        check("cli.windows_matched", e["count_match"]["matched"], 4)
        check("cli.w3_zero_matched", e["count_match"]["windows"]["w3"],
              {"oracle": 0, "competitor": 0, "match": True,
               "policy": "missing_report_as_zero (an RSTREAM of an empty relation "
                         "emits no elements)"})
        # INVARIANT half 1: caveat machine-attached to every row + the envelope
        check("cli.rows_admitted", len(e["rows"]), 1)
        check("cli.rows_caveat", all(r_["time_model_caveat"] is True for r_ in e["rows"]), True)
        check("cli.envelope_caveat", e["time_model_caveat"], True)
        check("cli.replay_sha_present", len(e["replay"]["sha256"]), 64)

        # INVARIANT half 2: a corrupted count stream -> gate fails, NO timing rows
        with open(comp) as f:
            lines = f.read().splitlines()
        corrupted = [ln.replace("\t3", "\t4", 1) if ln.startswith("report\t10001") else ln
                     for ln in lines]
        check("cli.corruption_applied", corrupted != lines, True)
        with open(comp, "w") as f:
            f.write("\n".join(corrupted) + "\n")
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--replay", REPLAY_SRBENCH, "--out", env])
        check("cli.corrupted.exit", r.returncode, 3)
        e = json.load(open(env))
        check("cli.corrupted.verdict", e["verdict"], "NOT-COUNT-MATCHED")
        check("cli.corrupted.no_timing_rows", e["rows"], [])


def main():
    test_parse_replay()
    test_ref_counts_match_oracle()
    test_map_reports()
    test_gate()
    test_cli_end_to_end()
    print("\n%d passed, %d failed" % (PASS, FAIL))
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
