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


# --- 6. protocol caveats are machine-attached, not prose (sq-rpdae) ----------
def test_protocol_caveat():
    with tempfile.TemporaryDirectory() as td:
        comp = os.path.join(td, "competitor.tsv")
        env = os.path.join(td, "envelope.json")
        r = run_cli(["ref-counts", "--replay", REPLAY_SRBENCH, "--scenario", "srbench_join",
                     "--raw-reports"])
        with open(comp, "w") as f:
            f.write(r.stdout)
            f.write("timing\trsp_replay_push_triples_per_s\t12345\ttriples_per_s\n")
        caveat = "window alignment differs: agreement is replay-contingent, not semantic"
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--replay", REPLAY_SRBENCH, "--out", env, "--protocol-caveat", caveat])
        check("caveat.exit", r.returncode, 0)
        e = json.load(open(env))
        check("caveat.in_envelope", e["protocol_caveats"], [caveat])
        # the load-bearing half: it rides on every emitted row, so a consumer reading
        # only `rows` cannot lose it
        check("caveat.on_every_row",
              all(r_.get("protocol_caveats") == [caveat] for r_ in e["rows"]), True)
        check("caveat.rows_present", len(e["rows"]) >= 1, True)
        # absent by default => no empty key noise on the existing envelopes
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--replay", REPLAY_SRBENCH, "--out", env])
        e = json.load(open(env))
        check("caveat.default_empty", e["protocol_caveats"], [])
        check("caveat.absent_from_rows",
              any("protocol_caveats" in r_ for r_ in e["rows"]), False)


# --- 7. SCALED matched-workload replay + runner-sourced oracle (sq-3f5ay) ----
#
# The scaled replay is GENERATED, not committed (~10^5 lines), so what these tests
# pin is the chain that makes it usable: the generator is deterministic and matches
# its committed manifest sha256; its srbench_join counts hit the closed form; and
# the runner-sourced oracle path refuses an oracle that disagrees with the
# independent model — otherwise a scaled throughput row could be published off a
# workload nothing had actually verified.
MANIFEST = os.path.join(HERE, "replay", "scaled.manifest.json")


def test_gen_replay_matches_manifest():
    pin = json.load(open(MANIFEST))
    with tempfile.TemporaryDirectory() as td:
        out = os.path.join(td, "scaled.ts.tsv")
        r = run_cli(["gen-replay", "--out", out])
        check("gen.exit", r.returncode, 0)
        check("gen.sha256_matches_pin", rc.sha256_file(out), pin["sha256"])
        events = rc.parse_replay(out)
        check("gen.events", len(events), pin["events"])
        # the closed form: sensors * obs_per_sensor joined rows in EVERY window
        counts = rc.ref_counts(events, "srbench_join")
        check("gen.windows", len(counts), pin["windows"])
        check("gen.rows_per_window",
              set(counts), {pin["srbench_join_rows_per_window"]})
        # a drifted parameter must be REFUSED against the pin, not silently accepted
        r = run_cli(["gen-replay", "--out", os.path.join(td, "drift.tsv"), "--sensors", "99"])
        check("gen.drift_refused", r.returncode, 2)
        check("gen.drift_names_sha", "sha256" in r.stderr, True)


def test_oracle_runner_path():
    """The scaled leg's gate, exercised on the SMALL committed replay (same code path,
    no 21 MB temp file): oracle counts sourced from a replay-runner-shaped TSV."""
    with tempfile.TemporaryDirectory() as td:
        oracle_tsv = os.path.join(td, "sparq.tsv")
        comp = os.path.join(td, "competitor.tsv")
        env = os.path.join(td, "envelope.json")
        with open(oracle_tsv, "w") as f:
            f.write("meta\tengine\tsparq-rsp\n")
            counts = rc.ref_counts(rc.parse_replay(REPLAY_SRBENCH), "srbench_join")
            for k, rows in enumerate(counts):
                f.write("w%d\t%d\n" % (k, rows))
            f.write("timing\trsp_replay_push_triples_per_s\t777\ttriples_per_s\n")
        r = run_cli(["ref-counts", "--replay", REPLAY_SRBENCH, "--scenario", "srbench_join",
                     "--raw-reports"])
        with open(comp, "w") as f:
            # relabel the ref-evaluator stand-in as the engine it stands in for, so the
            # side-by-side assertion below reads on real engine labels
            for line in r.stdout.splitlines():
                if line.startswith("meta\tengine\t"):
                    line = "meta\tengine\trsp4j-yasper"
                f.write(line + "\n")
            f.write("timing\trsp_replay_push_triples_per_s\t555\ttriples_per_s\n")
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--oracle-runner", oracle_tsv, "--replay", REPLAY_SRBENCH, "--out", env])
        check("runner.exit", r.returncode, 0)
        e = json.load(open(env))
        check("runner.verdict", e["verdict"], "COUNT-MATCHED")
        check("runner.cross_check_agrees", e["count_match"]["oracle_cross_check"]["agrees"], True)
        # the SIDE-BY-SIDE: both engines' throughput, engine-labelled, both caveated
        by_engine = {r_["engine"]: r_["value"] for r_ in e["rows"]
                     if r_["metric"] == "rsp_replay_push_triples_per_s"}
        check("runner.side_by_side", by_engine, {"sparq-rsp": "777", "rsp4j-yasper": "555"})
        check("runner.all_rows_caveated",
              all(r_["time_model_caveat"] is True for r_ in e["rows"]), True)

        # INVARIANT: an oracle that disagrees with the independent model publishes NOTHING
        with open(oracle_tsv) as f:
            bad = f.read().replace("w0\t3", "w0\t4", 1)
        check("runner.corruption_applied", "w0\t4" in bad, True)
        with open(oracle_tsv, "w") as f:
            f.write(bad)
        r = run_cli(["count-match", "--scenario", "srbench_join", "--competitor", comp,
                     "--oracle-runner", oracle_tsv, "--replay", REPLAY_SRBENCH, "--out", env])
        check("runner.bad_oracle.exit", r.returncode, 3)
        e = json.load(open(env))
        check("runner.bad_oracle.verdict", e["verdict"], "ORACLE-CROSS-CHECK-FAILED")
        check("runner.bad_oracle.no_rows", e["rows"], [])
        check("runner.bad_oracle.divergence_reported",
              e["count_match"]["oracle_cross_check"]["divergences"],
              {"w0": {"runner": 4, "model": 3}})


def test_windows_of():
    """The bucketing used by ref_counts must select exactly the covering windows —
    it replaced an O(events x windows) rescan, so an off-by-one here would silently
    change every scaled count."""
    for rng, step, ts, want in [
        (10, 10, 0, [0]), (10, 10, 9, [0]), (10, 10, 10, [1]),
        (20, 10, 0, [0]), (20, 10, 15, [0, 1]), (20, 10, 25, [1, 2]),
        (20, 20, 19, [0]), (20, 20, 20, [1]),
    ]:
        got = list(rc.windows_of(ts, rng, step))
        check("windows_of(ts=%d,rng=%d,step=%d)" % (ts, rng, step), got, want)
        # cross-check against the naive predicate the bucketing replaced
        naive = [k for k in range(0, ts // step + 1) if rc.in_window(ts, k, rng, step)]
        check("windows_of.agrees_with_naive(ts=%d,rng=%d)" % (ts, rng), got, naive)


def main():
    test_parse_replay()
    test_ref_counts_match_oracle()
    test_map_reports()
    test_gate()
    test_cli_end_to_end()
    test_protocol_caveat()
    test_windows_of()
    test_gen_replay_matches_manifest()
    test_oracle_runner_path()
    print("\n%d passed, %d failed" % (PASS, FAIL))
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
