#!/usr/bin/env python3
# [FABLE-5] (sq-hmd7l.20) Bounded count-matched-replay RSP comparability protocol —
# the comparator half of the RSP4J/YASPER harness.
#
# Protocol (research/comparative-benchmarking-everything.md sec 5.2, adopted over the
# older blanket NOT-COMPARABLE verdict): drive the external RSP engine with the
# IDENTICAL pinned timestamped replay used by sparq's clock-free oracle
# (bench/rsp/replay/*.ts.tsv, exported from crates/sparq-rsp/examples/rsp_oracle.rs),
# require PER-WINDOW RESULT-COUNT AGREEMENT with the deterministic oracle
# (bench/rsp/expected.tsv) FIRST, and only then admit any timing row — every emitted
# comparison row carries a MACHINE-ATTACHED time-model caveat (`time_model_caveat`,
# not just prose). Windows that cannot be count-matched are excluded and the
# exclusion is reported in the envelope.
#
# Subcommands
#   ref-counts   — independent per-window row-count reference evaluator over a pinned
#                  replay file. Two roles: (a) the FIDELITY GUARD tying the exported
#                  replay files to bench/rsp/expected.tsv (and transitively to the
#                  in-code oracle script, which run.sh pins to expected.tsv), and
#                  (b) the smoke stand-in for a competitor count stream so the whole
#                  count-match path is exercised without a JVM.
#   count-match  — the gate + envelope emitter: oracle counts vs a competitor report
#                  TSV (from bench/rsp/rsp4j/Rsp4jReplayRunner.java or ref-counts).
#
# stdlib-only (matches the scripts/bench-adapters/ house style). Exit codes:
#   0 = COUNT-MATCHED (possibly with declared/protocol exclusions, all reported)
#   3 = NOT-COUNT-MATCHED (envelope still written, with zero timing rows)
#   2 = usage / input error
import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

TIME_MODEL_CAVEAT = (
    "TIME-MODEL CAVEAT: sparq-rsp is clock-free — a window closes on the pushed-"
    "timestamp watermark, so its output is a pure function of the (triple, ts) replay. "
    "RSP4J/YASPER is a service-engine framework with SECRET/C-SPARQL report semantics, "
    "driven here in its event-time (TimeImpl) configuration with the identical "
    "timestamped replay plus declared boundary heartbeats; a deployed RSP service runs "
    "wall-clock windows. Per-window result-count agreement is required before any "
    "timing is admitted, and timing remains different in KIND across the two execution "
    "models — never read it as a raw head-to-head. See "
    "research/comparative-benchmarking-everything.md sec 5.2."
)

VALUE_P = "<http://ex/value>"
IN_P = "<http://ex/in>"
STATE_P = "<http://ex/state>"


# --------------------------------------------------------------------- replay


def parse_replay(path):
    """Parse a pinned `.ts.tsv` replay: (ts, stream, s, p, o) tuples, ts ascending."""
    events = []
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) != 5:
                raise ValueError(
                    "%s:%d: expected 5 tab-separated columns, got %d"
                    % (path, lineno, len(parts))
                )
            ts = int(parts[0])
            events.append((ts, parts[1], parts[2], parts[3], parts[4]))
    if events != sorted(events, key=lambda e: e[0]):
        raise ValueError("%s: events are not sorted by ts" % path)
    return events


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


# ------------------------------------------------------- reference evaluation
#
# Independent per-window row-count models of the oracle scenarios (row COUNTS only,
# mirroring the SPARQL semantics of crates/sparq-rsp/examples/rsp_oracle.rs). Window
# k spans [k*step, k*step+range); the oracle closes every window whose start is at or
# before the last replay timestamp (trailing windows close on flush).

SCENARIOS = {
    # name: (range, step)
    "tumbling_avg": (10, 10),
    "sliding_sum": (20, 10),
    "tumbling_groupby_join": (20, 20),
    "srbench_join": (10, 10),
    "srbench_groupby_state": (10, 10),
}


def window_count(events, rng, step):
    """Number of windows the clock-free oracle closes: every k with k*step <= max ts."""
    max_ts = max(e[0] for e in events)
    return max_ts // step + 1


def in_window(ts, k, rng, step):
    return k * step <= ts < k * step + rng


def ref_window_rows(events, scenario, k):
    """Row count the scenario's query yields for window k (reference model)."""
    rng, step = SCENARIOS[scenario]
    win = [e for e in events if in_window(e[0], k, rng, step)]
    if scenario == "tumbling_avg":
        # single-group AVG: one aggregate row per window, including empty windows
        return 1
    if scenario == "sliding_sum":
        # SUM per sensor GROUP BY ?s: one row per distinct subject with a value triple
        return len({s for (_, _, s, p, _) in win if p == VALUE_P})
    if scenario == "tumbling_groupby_join":
        # room join GROUP BY ?room: distinct rooms of subjects that also have a value
        valued = {s for (_, _, s, p, _) in win if p == VALUE_P}
        return len({o for (_, _, s, p, o) in win if p == IN_P and s in valued})
    if scenario == "srbench_join":
        # SELECT ?st ?state ?v: distinct joined (st, state, v) across the two streams
        meta = {(s, o) for (_, g, s, p, o) in win if p == STATE_P}
        obs = {(s, o) for (_, g, s, p, o) in win if p == VALUE_P}
        return len({(st, state, v) for (st, v) in obs for (st2, state) in meta if st == st2})
    if scenario == "srbench_groupby_state":
        # GROUP BY ?state after the join: distinct joined states
        meta = {(s, o) for (_, g, s, p, o) in win if p == STATE_P}
        obs = {s for (_, g, s, p, _) in win if p == VALUE_P}
        return len({state for (st, state) in meta if st in obs})
    raise ValueError("unknown scenario %s" % scenario)


def ref_counts(events, scenario):
    rng, step = SCENARIOS[scenario]
    return [ref_window_rows(events, scenario, k) for k in range(window_count(events, rng, step))]


# ----------------------------------------------------------------- the oracle


def oracle_counts(expected_tsv, scenario, mode="pdict"):
    """Per-window oracle counts from bench/rsp/expected.tsv.

    Single-window scenarios are read at one EvalMode (default pdict) — expected.tsv
    pins all three modes to identical counts, so the choice is immaterial; SRBench
    scenarios have a single metric family.
    """
    if scenario.startswith("srbench_"):
        prefix = "rsp_srbench_%s_w" % scenario[len("srbench_"):]
    else:
        prefix = "rsp_%s_%s_w" % (scenario, mode)
    by_k = {}
    with open(expected_tsv, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            metric, value = line.split("\t")
            if metric.startswith(prefix) and metric.endswith("_rows"):
                k = int(metric[len(prefix):-len("_rows")])
                by_k[k] = int(value)
    if not by_k:
        raise ValueError("no oracle metrics for scenario %s in %s" % (scenario, expected_tsv))
    n = max(by_k) + 1
    if sorted(by_k) != list(range(n)):
        raise ValueError("oracle window indices for %s are not contiguous" % scenario)
    return [by_k[k] for k in range(n)]


# ------------------------------------------------------------ competitor TSVs


def parse_competitor(path):
    """Parse a competitor runner TSV.

    Line kinds (tab-separated):
      report\t<t_e>\t<distinct_rows>   — one report the engine emitted at time t_e
      w<k>\t<rows>                     — pre-mapped per-window count (ref-counts output)
      meta\t<key>\t<value>             — engine/version/policy metadata
      timing\t<metric>\t<value>\t<unit> — timing metrics (admitted only after the gate)
    """
    reports, mapped, meta, timing = [], {}, {}, []
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t")
            if parts[0] == "report" and len(parts) == 3:
                reports.append((int(parts[1]), int(parts[2])))
            elif parts[0] == "meta" and len(parts) == 3:
                meta[parts[1]] = parts[2]
            elif parts[0] == "timing" and len(parts) == 4:
                timing.append({"metric": parts[1], "value": parts[2], "unit": parts[3]})
            elif parts[0].startswith("w") and len(parts) == 2:
                mapped[int(parts[0][1:])] = int(parts[1])
            else:
                raise ValueError("%s:%d: unrecognised line %r" % (path, lineno, line))
    return reports, mapped, meta, timing


def map_reports_to_windows(reports, rng, step, time_scale):
    """Map raw (t_e, rows) engine reports to oracle window indices.

    A report at t_e evaluates the newest window already closed at t_e, i.e. the max k
    with k*step + range <= t_e (in scaled time). Windows with k < 0 are C-SPARQL's
    partial leading windows (its SECRET scope opens windows starting before t0) — the
    oracle's window sequence starts at k = 0, so they are protocol exclusions,
    returned separately for honest reporting.
    """
    mapped, dupes, leading = {}, [], []
    for t_e, rows in reports:
        k = (t_e - rng * time_scale) // (step * time_scale)
        if k < 0:
            leading.append({"t_e": t_e, "rows": rows})
            continue
        if k in mapped and mapped[k] != rows:
            dupes.append({"window": "w%d" % k, "t_e": t_e, "rows": rows, "prev": mapped[k]})
            continue
        mapped[k] = rows
    return mapped, dupes, leading


# ----------------------------------------------------------------- the gate


def count_match(oracle, competitor_by_k, declared_exclusions, missing_as_zero):
    """Window-for-window gate. Returns (windows, excluded, extra, all_matched)."""
    windows, excluded = {}, {}
    all_matched = True
    for k, want in enumerate(oracle):
        wk = "w%d" % k
        if wk in declared_exclusions:
            excluded[wk] = {"oracle": want, "reason": declared_exclusions[wk]}
            continue
        if k in competitor_by_k:
            got = competitor_by_k[k]
            policy = None
        elif missing_as_zero:
            got = 0
            policy = "missing_report_as_zero (an RSTREAM of an empty relation emits no elements)"
        else:
            got = None
            policy = "missing"
        entry = {"oracle": want, "competitor": got, "match": got == want}
        if policy:
            entry["policy"] = policy
        windows[wk] = entry
        all_matched = all_matched and entry["match"]
    extra = {
        "w%d" % k: rows for k, rows in sorted(competitor_by_k.items()) if k >= len(oracle)
    }
    return windows, excluded, extra, all_matched


# ------------------------------------------------------------------ commands


def cmd_ref_counts(args):
    events = parse_replay(args.replay)
    rng, step = SCENARIOS[args.scenario]
    counts = ref_counts(events, args.scenario)
    print("meta\tengine\tref-evaluator (bench/rsp/rsp4j_compare.py, count-model only)")
    for k, rows in enumerate(counts):
        if args.raw_reports:
            # emit as raw engine-style reports (t_e = window close + 1, scaled);
            # zero-count windows are OMITTED like a real engine's RSTREAM of an
            # empty relation, so the missing-as-zero policy path is exercised
            if rows == 0:
                continue
            t_e = (k * step + rng) * args.time_scale + 1
            print("report\t%d\t%d" % (t_e, rows))
        else:
            print("w%d\t%d" % (k, rows))
    return 0


def cmd_count_match(args):
    oracle = oracle_counts(args.oracle, args.scenario, args.mode)
    reports, mapped, meta, timing = parse_competitor(args.competitor)
    rng, step = SCENARIOS[args.scenario]
    dupes, leading = [], []
    if reports:
        raw_mapped, dupes, leading = map_reports_to_windows(reports, rng, step, args.time_scale)
        # pre-mapped lines (if any) take precedence over raw reports for the same k
        raw_mapped.update(mapped)
        mapped = raw_mapped

    declared = {}
    for spec in args.exclude or []:
        wk, _, reason = spec.partition(":")
        declared[wk] = reason or "declared exclusion (no reason given)"

    windows, excluded, extra, all_matched = count_match(
        oracle, mapped, declared, not args.no_missing_as_zero
    )
    all_matched = all_matched and not dupes

    if all_matched:
        verdict = "COUNT-MATCHED-WITH-EXCLUSIONS" if excluded else "COUNT-MATCHED"
    else:
        verdict = "NOT-COUNT-MATCHED"

    # INVARIANT (sq-hmd7l.20): no throughput row without per-window count agreement,
    # and the time-model caveat is machine-attached to every emitted comparison row.
    rows = []
    if all_matched:
        for t in timing:
            rows.append(
                {
                    "engine": meta.get("engine", args.engine),
                    "metric": t["metric"],
                    "value": t["value"],
                    "unit": t["unit"],
                    "time_model_caveat": True,
                    "time_model_caveat_text": TIME_MODEL_CAVEAT,
                }
            )

    envelope = {
        "gather": "rsp-bounded-count-matched-replay (sq-hmd7l.20)",
        "design_record": "research/comparative-benchmarking-everything.md sec 5.2",
        "suite": "rsp-throughput",
        "canonical": bool(args.canonical),
        "canonical_note": args.canonical_note
        or "NON-canonical box: counts are deterministic and machine-independent; any "
        "timing rows are trend-only until re-gathered on the quiet EC2 reference box.",
        "git_commit": git_commit(),
        "scenario": args.scenario,
        "window": {"range": rng, "step": step, "time_scale": args.time_scale},
        "replay": replay_block(args.replay),
        "engines": {
            "sparq": {
                "role": "oracle (clock-free deterministic per-window counts, bench/rsp/expected.tsv)",
                "oracle_mode": args.mode,
            },
            args.engine: {"role": "competitor", **{k: v for k, v in meta.items()}},
        },
        "policies": {
            "missing_report_as_zero": not args.no_missing_as_zero,
            "leading_partial_windows": "excluded-by-protocol (C-SPARQL SECRET scope opens "
            "windows before t0; the oracle window sequence starts at k=0)",
            "declared_exclusions": declared,
        },
        "count_match": {
            "oracle_source": "%s (scenario %s)"
            % (repo_rel(args.oracle), args.scenario),
            "windows": windows,
            "excluded_windows": excluded,
            "extra_competitor_windows": extra,
            "leading_partial_reports": leading,
            "conflicting_reports": dupes,
            "all_matched": all_matched,
            "matched": sum(1 for w in windows.values() if w["match"]),
            "mismatched": sum(1 for w in windows.values() if not w["match"]),
            "excluded": len(excluded),
        },
        "verdict": verdict,
        "rows": rows,
        "rows_note": "rows is EMPTY unless every non-excluded window count-matched the "
        "oracle; every row carries the machine-attached time-model caveat.",
        "time_model_caveat": True,
        "time_model_caveat_text": TIME_MODEL_CAVEAT,
    }
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump(envelope, f, indent=2)
            f.write("\n")
    else:
        json.dump(envelope, sys.stdout, indent=2)
        sys.stdout.write("\n")

    n = len(windows)
    if all_matched:
        sys.stderr.write(
            "[rsp4j-compare] %s: %d/%d windows count-matched (%d excluded) — %d timing row(s) admitted\n"
            % (args.scenario, envelope["count_match"]["matched"], n, len(excluded), len(rows))
        )
        return 0
    sys.stderr.write(
        "[rsp4j-compare] %s: NOT-COUNT-MATCHED (%d mismatch(es), %d conflicting report(s)) — no timing rows\n"
        % (args.scenario, envelope["count_match"]["mismatched"], len(dupes))
    )
    return 3


def repo_rel(path):
    return os.path.relpath(path, os.path.join(HERE, "..", ".."))


def replay_block(path):
    events = parse_replay(path)
    return {
        "file": repo_rel(path),
        "sha256": sha256_file(path),
        "events": len(events),
        "max_ts": max(e[0] for e in events),
        "note": "pinned export of the fixed script in crates/sparq-rsp/examples/rsp_oracle.rs; "
        "both engines are driven from this file (identical timestamped replay)",
    }


def git_commit():
    try:
        return (
            subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=HERE, capture_output=True, text=True, timeout=10
            ).stdout.strip()
            or None
        )
    except Exception:
        return None


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    rc = sub.add_parser("ref-counts", help="independent per-window count reference evaluator")
    rc.add_argument("--replay", required=True)
    rc.add_argument("--scenario", required=True, choices=sorted(SCENARIOS))
    rc.add_argument("--raw-reports", action="store_true", help="emit engine-style report lines")
    rc.add_argument("--time-scale", type=int, default=1000)
    rc.set_defaults(fn=cmd_ref_counts)

    cm = sub.add_parser("count-match", help="the gate: oracle vs competitor counts + envelope")
    cm.add_argument("--scenario", required=True, choices=sorted(SCENARIOS))
    cm.add_argument("--oracle", default=os.path.join(HERE, "expected.tsv"))
    cm.add_argument("--mode", default="pdict", choices=["rebuild", "pdict", "delta"])
    cm.add_argument("--competitor", required=True, help="competitor runner TSV")
    cm.add_argument("--replay", required=True, help="the pinned replay file (for the envelope)")
    cm.add_argument("--engine", default="rsp4j-yasper")
    cm.add_argument("--time-scale", type=int, default=1000)
    cm.add_argument("--exclude", action="append", metavar="wK:reason")
    cm.add_argument("--no-missing-as-zero", action="store_true")
    cm.add_argument("--canonical", action="store_true")
    cm.add_argument("--canonical-note", default=None)
    cm.add_argument("--out", default=None, help="envelope JSON path (default stdout)")
    cm.set_defaults(fn=cmd_count_match)

    args = ap.parse_args(argv)
    try:
        return args.fn(args)
    except (ValueError, OSError) as e:
        sys.stderr.write("[rsp4j-compare] ERROR: %s\n" % e)
        return 2


if __name__ == "__main__":
    sys.exit(main())
