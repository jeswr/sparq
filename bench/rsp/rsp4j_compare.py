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
#   gen-replay   — (sq-3f5ay) deterministically GENERATE the SCALED pinned replay for
#                  the matched-workload throughput leg. The 19-event oracle replay is
#                  far too small for a sustained-rate claim; a ~200 k-event file is far
#                  too large to commit, so the pin is the GENERATOR + a committed
#                  manifest (params + sha256) rather than the payload. Both engines are
#                  then driven from the identical regenerated file.
#   count-match  — the gate + envelope emitter: oracle counts vs a competitor report
#                  TSV (from bench/rsp/rsp4j/Rsp4jReplayRunner.java or ref-counts).
#                  With --oracle-runner the oracle side comes from the sparq replay-FILE
#                  runner (crates/sparq-rsp/examples/replay_runner.rs) instead of
#                  expected.tsv, which is what makes a SCALED replay (one with no
#                  expected.tsv entry) comparable — and the runner's counts are
#                  themselves cross-checked against the independent model below before
#                  any row is admitted.
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


def windows_of(ts, rng, step):
    """The window indices k >= 0 that cover `ts` (k*step <= ts < k*step + rng)."""
    lo = (ts - rng) // step + 1
    return range(max(0, lo), ts // step + 1)


def ref_window_rows(events, scenario, k):
    """Row count the scenario's query yields for window k (reference model)."""
    rng, step = SCENARIOS[scenario]
    return ref_rows(
        [e for e in events if in_window(e[0], k, rng, step)], scenario
    )


def ref_rows(win, scenario):
    """Row count for ONE window's already-selected events (the count model proper)."""
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
        # SELECT ?st ?state ?v: distinct joined (st, state, v) across the two streams.
        # Indexed by station rather than a cross product: the SCALED replay
        # (sq-3f5ay) has ~10^3 obs x ~10^2 meta per window, where the quadratic form
        # is minutes of Python. Identical result — (st, v) pairs are already
        # distinct, so each contributes exactly one tuple per distinct state of st.
        states = {}
        for (_, g, s, p, o) in win:
            if p == STATE_P:
                states.setdefault(s, set()).add(o)
        obs = {(s, o) for (_, g, s, p, o) in win if p == VALUE_P}
        return sum(len(states[st]) for (st, _v) in obs if st in states)
    if scenario == "srbench_groupby_state":
        # GROUP BY ?state after the join: distinct joined states
        meta = {(s, o) for (_, g, s, p, o) in win if p == STATE_P}
        obs = {s for (_, g, s, p, _) in win if p == VALUE_P}
        return len({state for (st, state) in meta if st in obs})
    raise ValueError("unknown scenario %s" % scenario)


def ref_counts(events, scenario):
    """Per-window reference counts. Buckets events by covering window in ONE pass
    (the naive per-window rescan is O(events x windows) — unusable on the scaled
    replay, which has ~10^5 events and ~10^2 windows)."""
    rng, step = SCENARIOS[scenario]
    n = window_count(events, rng, step)
    buckets = [[] for _ in range(n)]
    for e in events:
        for k in windows_of(e[0], rng, step):
            if k < n:
                buckets[k].append(e)
    return [ref_rows(b, scenario) for b in buckets]


# ------------------------------------------- SCALED replay generation (sq-3f5ay)
#
# The matched-workload throughput leg needs a replay big enough for a sustained
# rate. Committing a ~200 k-line TSV is not acceptable, so what is PINNED is this
# deterministic generator plus a committed manifest holding the parameters and the
# sha256 of the file they produce (bench/rsp/replay/scaled.manifest.json). Both
# engines are then driven from the identical regenerated file, and the envelope
# still carries the file's sha256 exactly as it does for the committed replays.
#
# The shape is srbench_join — NOT the 100-sensor single-stream AVG of
# examples/throughput.rs. That is a deliberate correction: the count-comparable
# surface is `srbench_join` alone (research/gap-rsp-2026-07.md), and the protocol
# INVARIANT forbids a throughput row for a workload whose windows cannot be
# count-matched. The 100-sensor scale is kept; the query shape is the one both
# engines can actually agree on.

XSD_INTEGER = "<http://www.w3.org/2001/XMLSchema#integer>"
GEN_STATES = ["NY", "CA", "TX", "FL", "WA", "IL", "MA", "OH", "GA", "AZ"]
GEN_DEFAULTS = {"sensors": 100, "windows": 100, "obs_per_sensor": 20, "range": 10, "step": 10}


def gen_srbench_lines(sensors, windows, obs_per_sensor, rng, step):
    """Deterministic scaled srbench_join-shaped replay (no RNG, no clock, no I/O).

    Per window k: every station re-announces its state at ts = k*step (exactly what
    the oracle's metadata stream does), then each station emits `obs_per_sensor`
    values spread across the window interior. Every station has exactly one state
    and its values are distinct within the window, so the srbench_join row count is
    a CLOSED FORM — `sensors * obs_per_sensor` per window — which the gate asserts
    independently of any engine.
    """
    if min(sensors, windows, obs_per_sensor) < 1:
        raise ValueError("sensors/windows/obs_per_sensor must all be >= 1")
    if rng != step:
        raise ValueError("scaled generation is tumbling-only (range must equal step)")
    if step < 2:
        raise ValueError("step must be >= 2 (ts = k*step is the metadata slot)")
    span = step - 1  # interior offsets 1 .. step-1
    for k in range(windows):
        base = k * step
        for i in range(sensors):
            yield "%d\t<http://ex/meta>\t<http://ex/st%d>\t<http://ex/state>\t<http://ex/%s>" % (
                base, i, GEN_STATES[i % len(GEN_STATES)]
            )
        for j in range(obs_per_sensor):
            ts = base + 1 + (j * span) // obs_per_sensor
            value = k * obs_per_sensor + j
            for i in range(sensors):
                yield '%d\t<http://ex/obs>\t<http://ex/st%d>\t<http://ex/value>\t"%d"^^%s' % (
                    ts, i, value, XSD_INTEGER
                )


def gen_header(p):
    """Header comment lines. Deterministic (parameters only — no clock, no commit):
    the sha256 that pins this file must not move on a re-run."""
    return [
        "# [OPUS-5] (sq-3f5ay) GENERATED scaled replay — DO NOT EDIT, DO NOT COMMIT.",
        "# Regenerate with:  bench/rsp/rsp4j_compare.py gen-replay --manifest \\",
        "#                     bench/rsp/replay/scaled.manifest.json --out <this file>",
        "#",
        "# The matched-workload leg of the bounded count-matched-replay protocol",
        "# (research/comparative-benchmarking-everything.md sec 5.2): a srbench_join-shaped",
        "# replay at 100-sensor scale, large enough for a sustained-rate measurement, driven",
        "# through BOTH sparq-rsp (examples/replay_runner.rs) and RSP4J/YASPER",
        "# (rsp4j/Rsp4jReplayRunner.java). srbench_join because it is the only",
        "# count-comparable scenario (research/gap-rsp-2026-07.md), and the protocol",
        "# INVARIANT admits no throughput row without per-window count agreement.",
        "#",
        "# Parameters (pinned in scaled.manifest.json alongside this file's sha256):",
        "#   sensors=%(sensors)d windows=%(windows)d obs_per_sensor=%(obs_per_sensor)d "
        "range=%(range)d step=%(step)d" % p,
        "#",
        "# Columns: <ts>\\t<stream-iri>\\t<subject>\\t<predicate>\\t<object>  (N-Triples terms)",
    ]


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


def cmd_gen_replay(args):
    manifest = {}
    if args.manifest and os.path.exists(args.manifest):
        with open(args.manifest, encoding="utf-8") as f:
            manifest = json.load(f)
    p = dict(GEN_DEFAULTS)
    p.update(manifest.get("params", {}))
    for key in GEN_DEFAULTS:
        override = getattr(args, key)
        if override is not None:
            p[key] = override

    lines = gen_header(p)
    lines.extend(
        gen_srbench_lines(
            p["sensors"], p["windows"], p["obs_per_sensor"], p["range"], p["step"]
        )
    )
    with open(args.out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
        f.write("\n")

    digest = sha256_file(args.out)
    events = p["windows"] * p["sensors"] * (1 + p["obs_per_sensor"])
    derived = {
        "file": os.path.basename(args.out),
        "sha256": digest,
        "events": events,
        "windows": p["windows"],
        "srbench_join_rows_per_window": p["sensors"] * p["obs_per_sensor"],
    }

    if args.write_manifest:
        if not args.manifest:
            raise ValueError("--write-manifest needs --manifest <path>")
        out = {
            "bead": "sq-3f5ay",
            "note": "PIN for the scaled matched-workload replay. The payload is NOT "
            "committed (~10^5 lines); what is pinned is the deterministic generator "
            "(rsp4j_compare.py gen-replay) plus these parameters and the sha256 of the "
            "file they produce. Regenerating with these parameters MUST reproduce the "
            "sha256 byte-for-byte, and both engines are driven from that one file.",
            "shape": "srbench_join (the only count-comparable scenario — "
            "research/gap-rsp-2026-07.md)",
            "generator": "bench/rsp/rsp4j_compare.py gen-replay",
            "params": p,
            **derived,
        }
        with open(args.manifest, "w", encoding="utf-8") as f:
            json.dump(out, f, indent=2)
            f.write("\n")
        sys.stderr.write(
            "[rsp4j-compare] wrote manifest %s (sha256 %s)\n" % (args.manifest, digest)
        )
        return 0

    pinned = manifest.get("sha256")
    if pinned and pinned != digest:
        raise ValueError(
            "generated replay sha256 %s does not match the pinned %s in %s — the "
            "generator or its parameters drifted; both engines would no longer be "
            "driven from the pinned workload" % (digest, pinned, args.manifest)
        )
    for key, want in (("events", events), ("windows", p["windows"])):
        if key in manifest and manifest[key] != want:
            raise ValueError("manifest %s=%r but generated %r" % (key, manifest[key], want))
    sys.stderr.write(
        "[rsp4j-compare] %s: %d events, %d windows, sha256 %s%s\n"
        % (args.out, events, p["windows"], digest, " (matches pin)" if pinned else " (UNPINNED)")
    )
    return 0


def runner_oracle(path):
    """Per-window oracle counts + timing rows from a sparq replay-FILE runner TSV
    (crates/sparq-rsp/examples/replay_runner.rs). This is what makes a SCALED replay
    comparable at all: it has no expected.tsv entry, so the oracle side has to come
    from actually running sparq-rsp over the same file."""
    reports, mapped, meta, timing = parse_competitor(path)
    if reports:
        raise ValueError("%s: the sparq runner emits w<k> lines, not raw reports" % path)
    if not mapped:
        raise ValueError("%s: no w<k> per-window count lines" % path)
    n = max(mapped) + 1
    if sorted(mapped) != list(range(n)):
        raise ValueError("%s: window indices are not contiguous from w0" % path)
    return [mapped[k] for k in range(n)], meta, timing


def cmd_count_match(args):
    sparq_meta, sparq_timing, cross_check = {}, [], None
    if args.oracle_runner:
        oracle, sparq_meta, sparq_timing = runner_oracle(args.oracle_runner)
        # A runner-sourced oracle has no expected.tsv behind it, so it gets the same
        # treatment the committed replays get: the INDEPENDENT count model re-derives
        # every window from the replay file and must agree. A disagreement means the
        # oracle side itself is untrustworthy — the gate refuses before comparing.
        ref = ref_counts(parse_replay(args.replay), args.scenario)
        cross_check = {
            "model": "independent per-window count model (rsp4j_compare.ref_counts)",
            "oracle_source": "sparq replay-file runner",
            "windows": len(ref),
            "agrees": ref == oracle,
        }
        if ref != oracle:
            cross_check["divergences"] = {
                "w%d" % k: {"runner": (oracle[k] if k < len(oracle) else None), "model": r}
                for k, r in enumerate(ref)
                if k >= len(oracle) or oracle[k] != r
            }
    else:
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

    if cross_check and not cross_check["agrees"]:
        verdict = "ORACLE-CROSS-CHECK-FAILED"
        all_matched = False
    elif all_matched:
        verdict = "COUNT-MATCHED-WITH-EXCLUSIONS" if excluded else "COUNT-MATCHED"
    else:
        verdict = "NOT-COUNT-MATCHED"

    # INVARIANT (sq-hmd7l.20): no throughput row without per-window count agreement,
    # and the time-model caveat is machine-attached to every emitted comparison row.
    # Engine-specific protocol caveats (sq-rpdae): a count-match can be TRUE yet not be
    # evidence of equivalent window semantics — csparql2's Esper sliding win:time agrees
    # on this replay's counts only because no triple sits in the misaligned window edge.
    # Such a caveat travels in the envelope AND on every row, never in prose alone.
    protocol_caveats = list(args.protocol_caveat or [])

    # The SIDE-BY-SIDE (sq-3f5ay): with --oracle-runner, sparq's own timing rows from
    # the SAME replay file join the competitor's in `rows`, each labelled with its
    # engine. They are subject to the identical invariant — the gate must pass first —
    # so the published pair can never be a comparison of unmatched workloads.
    rows = []
    if all_matched:
        for engine, source in (
            (sparq_meta.get("engine", "sparq-rsp"), sparq_timing),
            (meta.get("engine", args.engine), timing),
        ):
            for t in source:
                row = {
                    "engine": engine,
                    "metric": t["metric"],
                    "value": t["value"],
                    "unit": t["unit"],
                    "time_model_caveat": True,
                    "time_model_caveat_text": TIME_MODEL_CAVEAT,
                }
                if protocol_caveats:
                    row["protocol_caveats"] = protocol_caveats
                rows.append(row)

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
                "role": "oracle (clock-free deterministic per-window counts, %s)"
                % (
                    "sparq replay-file runner over the same replay"
                    if args.oracle_runner
                    else "bench/rsp/expected.tsv"
                ),
                "oracle_mode": sparq_meta.get("mode", args.mode),
                **{k: v for k, v in sparq_meta.items()},
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
            # A runner-sourced oracle lives in a scratch TSV, so name the PRODUCER
            # (which is reproducible) rather than the temp path (which is not).
            "oracle_source": (
                "crates/sparq-rsp/examples/replay_runner.rs over %s (scenario %s)"
                % (repo_rel(args.replay), args.scenario)
                if args.oracle_runner
                else "%s (scenario %s)" % (repo_rel(args.oracle), args.scenario)
            ),
            "oracle_cross_check": cross_check,
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
        "protocol_caveats": protocol_caveats,
        "rows": rows,
        "rows_note": "rows is EMPTY unless every non-excluded window count-matched the "
        "oracle; every row carries the machine-attached time-model caveat. Rows are "
        "labelled per `engine`: when both engines were driven from this one replay file "
        "(--oracle-runner) the sparq and competitor throughput rows here ARE the "
        "matched-workload side-by-side — same file, same window sequence, counts agreed "
        "first. They remain different in KIND across the two time models (see the caveat) "
        "and are trend-only unless `canonical` is true.",
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
    if verdict == "ORACLE-CROSS-CHECK-FAILED":
        sys.stderr.write(
            "[rsp4j-compare] %s: ORACLE-CROSS-CHECK-FAILED — the sparq runner's per-window "
            "counts disagree with the independent model over %s; no rows\n"
            % (args.scenario, repo_rel(args.replay))
        )
        return 3
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


SCALED_MANIFEST = os.path.join(HERE, "replay", "scaled.manifest.json")


def replay_block(path):
    events = parse_replay(path)
    digest = sha256_file(path)
    block = {
        "file": repo_rel(path),
        "sha256": digest,
        "events": len(events),
        "max_ts": max(e[0] for e in events),
        "note": "pinned export of the fixed script in crates/sparq-rsp/examples/rsp_oracle.rs; "
        "both engines are driven from this file (identical timestamped replay)",
    }
    # The SCALED replay (sq-3f5ay) is generated, not committed — its pin is the manifest.
    # Matched by SHA, not by path or header text, so the envelope's provenance claim is
    # itself a verification: this file IS the pinned workload, or it does not say so.
    if os.path.exists(SCALED_MANIFEST):
        with open(SCALED_MANIFEST, encoding="utf-8") as f:
            manifest = json.load(f)
        if manifest.get("sha256") == digest:
            block["generated"] = True
            block["pinned_by"] = repo_rel(SCALED_MANIFEST)
            block["generator_params"] = manifest.get("params")
            block["note"] = (
                "SCALED matched-workload replay, regenerated by "
                "`rsp4j_compare.py gen-replay` and sha256-verified against %s (the payload "
                "is too large to commit; the generator + parameters are the pin); both "
                "engines are driven from this one file"
            ) % repo_rel(SCALED_MANIFEST)
    return block


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

    gr = sub.add_parser("gen-replay", help="generate the pinned SCALED matched-workload replay")
    gr.add_argument(
        "--out", required=True, help="destination .ts.tsv (generated, never committed)"
    )
    gr.add_argument(
        "--manifest",
        default=os.path.join(HERE, "replay", "scaled.manifest.json"),
        help="the PIN: parameters + expected sha256 (verified unless --write-manifest)",
    )
    gr.add_argument("--write-manifest", action="store_true", help="(re)pin the manifest instead")
    for key in sorted(GEN_DEFAULTS):
        gr.add_argument("--%s" % key.replace("_", "-"), dest=key, type=int, default=None)
    gr.set_defaults(fn=cmd_gen_replay)

    cm = sub.add_parser("count-match", help="the gate: oracle vs competitor counts + envelope")
    cm.add_argument("--scenario", required=True, choices=sorted(SCENARIOS))
    cm.add_argument("--oracle", default=os.path.join(HERE, "expected.tsv"))
    cm.add_argument(
        "--oracle-runner",
        default=None,
        help="sparq replay-FILE runner TSV (crates/sparq-rsp/examples/replay_runner.rs) to "
        "use as the oracle side instead of expected.tsv — required for a SCALED replay, "
        "which has no expected.tsv entry. Its counts are cross-checked against the "
        "independent model, and its timing rows join the side-by-side.",
    )
    cm.add_argument("--mode", default="pdict", choices=["rebuild", "pdict", "delta"])
    cm.add_argument("--competitor", required=True, help="competitor runner TSV")
    cm.add_argument("--replay", required=True, help="the pinned replay file (for the envelope)")
    cm.add_argument("--engine", default="rsp4j-yasper")
    cm.add_argument("--time-scale", type=int, default=1000)
    cm.add_argument("--exclude", action="append", metavar="wK:reason")
    cm.add_argument(
        "--protocol-caveat",
        action="append",
        metavar="TEXT",
        help="engine-specific caveat qualifying what a count-match does NOT prove; "
        "attached to the envelope AND to every emitted row (repeatable)",
    )
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
