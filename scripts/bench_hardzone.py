#!/usr/bin/env python3
# [FABLE-5] Hard-zone benchmark gate — median-of-history regression check (bench.yml, main pushes).
#
# WHY — github-action-benchmark's fail-on-alert compares each metric against the SINGLE previous
# benchmark-data point. On shared GitHub-hosted runners a one-off slow-runner environment shift
# flips many unrelated µs-scale timing metrics uniformly "worse" and hard-fails the run, while the
# very next push (back on a normal runner) goes green — a noise flap, not a regression signal
# (live 24h flap on main pushes 2026-07-22/23, e.g. red run 29988505852 between green neighbours).
# This script replaces that single-point hard zone with a deterministic MEDIAN-OF-HISTORY gate:
#
#   * Per metric, take the MEDIAN of its last up-to-HISTORY_WINDOW (5) published points from the
#     benchmark-data history (the dev/bench/data.js the workflow already fetched as prev-data.js).
#     A single slow-runner point cannot move a median-of-5, so a one-off environment shift no
#     longer reds main — while a REAL regression (the current run sustained above its history)
#     still hard-fails.
#   * HARD FAIL (exit 1) only if any compared metric is >= HARD_RATIO (2.0x) its median.
#   * UNIFORM-SHIFT EXEMPTION (exit 0 + loud warning): if >= UNIFORM_FRACTION (50%) of ALL
#     compared metrics are simultaneously >= WATCH_RATIO (1.75x) median, the whole run is a
#     runner-environment shift — a real code regression never degrades every unrelated metric
#     across every suite at once — so the gate refuses to red main on it (it warns loudly instead;
#     the soft-zone triage step files the bench-flake issue).
#   * TRANSPARENCY: every metric >= WATCH_RATIO (1.75x) median is printed as a table either way.
#   * ABSENT HISTORY (no prev-data.js / empty series / no overlapping metrics): warn + exit 0
#     (first-run safe; nothing to compare is not a failure).
#
# The DETERMINISTIC byte-count metrics stay strictly gated by scripts/perf-gate.py (the committed
# best-ever floor ratchet) — this gate is the noisy-timing hard zone only, and it runs ONLY where
# the Store step auto-pushes (push-to-main), so PR behavior is unchanged.
#
# USAGE
#   scripts/bench_hardzone.py --results bench-results.json --prev-data prev-data.js \
#       --suite 'sparq engine'
#   scripts/bench_hardzone.py --self-test          # hermetic; no network, no git, no writes
from __future__ import annotations

import argparse
import json
import re
import statistics
import sys
from pathlib import Path

PROG = "bench-hardzone"

HARD_RATIO = 2.0        # >= 2.0x median => hard fail (unless the uniform-shift exemption fires)
WATCH_RATIO = 1.75      # >= 1.75x median => printed in the watch table; feeds the uniform check
UNIFORM_FRACTION = 0.5  # >= 50% of compared metrics in the watch band => environment shift, exempt
HISTORY_WINDOW = 5      # median over the last up-to-5 published points per metric


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def parse_data_js(text: str) -> dict:
    """Parse a github-action-benchmark data.js payload (window.BENCHMARK_DATA = {...};)."""
    m = re.search(r"window\.BENCHMARK_DATA\s*=\s*", text)
    if m:
        text = text[m.end():]
    text = text.strip().rstrip(";").strip()
    return json.loads(text)


def history_values(series: list[dict], window: int = HISTORY_WINDOW) -> dict[str, list[float]]:
    """Per-metric values from the last up-to-`window` entries of a benchmark-data series.

    `series` is entries[suite] from data.js: a list of {commit, date, benches:[{name,value,..}]}.
    Entries are ordered by date; only the most recent `window` entries contribute. A metric absent
    from some of those entries simply contributes fewer points (median over what exists).
    """
    ordered = sorted(series, key=lambda e: e.get("date", 0))[-window:]
    vals: dict[str, list[float]] = {}
    for entry in ordered:
        for b in entry.get("benches", []):
            name = b.get("name")
            v = b.get("value")
            if name is not None and isinstance(v, (int, float)):
                vals.setdefault(name, []).append(float(v))
    return vals


def evaluate(current: list[dict], hist: dict[str, list[float]]) -> tuple[int, dict]:
    """Pure gate logic — unit-tested by --self-test.

    `current` is this run's bench-results.json rows ({name,value,unit}); `hist` is the per-metric
    history value lists (see history_values). Returns (exit_code, report) where report carries
    {compared, watch: [(name, cur, median, n, ratio)...], hard: [...same...], uniform_shift}.
    """
    watch: list[tuple[str, float, float, int, float]] = []
    hard: list[tuple[str, float, float, int, float]] = []
    compared = 0
    for row in current:
        name = row.get("name")
        val = row.get("value")
        if name is None or not isinstance(val, (int, float)):
            continue
        past = hist.get(name)
        if not past:
            continue  # brand-new metric — no history to compare against
        med = statistics.median(past)
        if med <= 0:
            continue  # cannot form a ratio (degenerate/zero baseline)
        compared += 1
        ratio = float(val) / med
        entry = (name, float(val), med, len(past), ratio)
        if ratio >= WATCH_RATIO:
            watch.append(entry)
        if ratio >= HARD_RATIO:
            hard.append(entry)
    if compared == 0:
        log("no overlapping metrics with history — nothing to compare; passing.")
        return 0, {"compared": 0, "watch": [], "hard": [], "uniform_shift": False}
    if watch:
        log(f"metrics >= {WATCH_RATIO}x their median of the last {HISTORY_WINDOW} points "
            f"({len(watch)}/{compared} compared):")
        log(f"  {'metric':<60} {'current':>14} {'median(n)':>18} {'ratio':>7}")
        for name, cur, med, n, ratio in sorted(watch, key=lambda t: -t[4]):
            log(f"  {name:<60} {cur:>14.4g} {med:>13.4g}(n={n}) {ratio:>6.2f}x")
    uniform = len(watch) / compared >= UNIFORM_FRACTION
    if uniform:
        # An across-the-board shift is the runner environment, never a real code regression in
        # every unrelated metric at once — refuse to red main on it, loudly.
        print(f"::warning title=bench hard zone — uniform environment shift, not gating::"
              f"{len(watch)} of {compared} compared metrics are >= {WATCH_RATIO}x their "
              f"median-of-{HISTORY_WINDOW} history simultaneously. A uniform shift across "
              f"unrelated metrics is a runner-environment change, not a code regression — "
              f"NOT failing the run. See the per-metric table in the step log.")
        return 0, {"compared": compared, "watch": watch, "hard": hard, "uniform_shift": True}
    if hard:
        for name, cur, med, n, ratio in hard:
            print(f"::error title=bench hard zone::{name} is {ratio:.2f}x its "
                  f"median-of-{n} history ({cur:.6g} vs median {med:.6g}) — at/above the "
                  f"{HARD_RATIO}x hard threshold, and the shift is NOT uniform across metrics "
                  f"(so it is not a runner-environment artifact). Treating as a real regression.")
        return 1, {"compared": compared, "watch": watch, "hard": hard, "uniform_shift": False}
    log(f"hard zone clean: {compared} metrics compared against their median-of-history; "
        f"{len(watch)} in the watch band, none at/above {HARD_RATIO}x.")
    return 0, {"compared": compared, "watch": watch, "hard": hard, "uniform_shift": False}


def run_gate(results_path: str, prev_data_path: str, suite: str) -> int:
    rp = Path(results_path)
    if not rp.exists():
        log(f"results file {results_path} is absent — nothing to gate; passing.")
        return 0
    current = json.loads(rp.read_text(encoding="utf-8"))
    if not isinstance(current, list):
        current = current.get("benches", [])
    pp = Path(prev_data_path)
    if not pp.exists():
        print(f"::warning title=bench hard zone — no history::{prev_data_path} is absent "
              f"(benchmark-data branch missing or empty — first run). Nothing to compare; passing.")
        return 0
    try:
        data = parse_data_js(pp.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, ValueError) as e:
        print(f"::warning title=bench hard zone — unparsable history::could not parse "
              f"{prev_data_path} ({e}); nothing to compare; passing.")
        return 0
    entries = data.get("entries", {})
    series = entries.get(suite) or (next(iter(entries.values())) if entries else [])
    if not series:
        print(f"::warning title=bench hard zone — empty history::no published points for suite "
              f"'{suite}' in {prev_data_path}; nothing to compare; passing.")
        return 0
    code, _report = evaluate(current, history_values(series))
    return code


# ── self-test (hermetic: inline fixtures; no network, no git, no writes) ─────────────────────────
def _series(points: list[list[tuple[str, float]]]) -> list[dict]:
    """Build a data.js-shaped series from [[(name, value), ...] per commit] (oldest first)."""
    return [
        {"date": i, "benches": [{"name": n, "value": v, "unit": "us"} for n, v in benches]}
        for i, benches in enumerate(points)
    ]


def _cur(rows: list[tuple[str, float]]) -> list[dict]:
    return [{"name": n, "value": v, "unit": "us"} for n, v in rows]


def self_test() -> int:
    failures: list[str] = []

    def check(cond: bool, what: str) -> None:
        if cond:
            log(f"self-test ok: {what}")
        else:
            failures.append(what)
            log(f"self-test FAIL: {what}")

    # 1. median computation: window trims to the last 5 points; median over what remains.
    hist = history_values(_series([[("m", 100.0)], [("m", 1.0)], [("m", 2.0)], [("m", 3.0)],
                                   [("m", 4.0)], [("m", 5.0)]]))
    check(hist["m"] == [1.0, 2.0, 3.0, 4.0, 5.0], "history window keeps only the last 5 points")
    check(statistics.median(hist["m"]) == 3.0, "median of 5-point history is the middle point")
    hist_even = history_values(_series([[("m", 1.0)], [("m", 2.0)], [("m", 3.0)], [("m", 8.0)]]))
    check(statistics.median(hist_even["m"]) == 2.5, "median of an even count averages the middle two")
    # A single outlier point cannot move a median-of-5 into failing territory.
    hist_outlier = history_values(_series([[("m", 10.0)], [("m", 10.0)], [("m", 10.0)],
                                           [("m", 10.0)], [("m", 30.0)]]))
    code, rep = evaluate(_cur([("m", 15.0)]), history_values(
        _series([[("m", 10.0)], [("m", 10.0)], [("m", 10.0)], [("m", 10.0)], [("m", 30.0)]])))
    check(statistics.median(hist_outlier["m"]) == 10.0 and code == 0 and not rep["hard"],
          "single outlier history point does not move the median (1.5x vs median passes)")

    # 2. absent history: no overlapping metrics => pass (exit 0), nothing compared.
    code, rep = evaluate(_cur([("new_metric", 42.0)]), {})
    check(code == 0 and rep["compared"] == 0, "absent history passes with nothing compared")

    # 3. uniform-shift exemption: >= 50% of compared metrics >= 1.75x median => exit 0, flagged.
    series = _series([[("a", 10.0), ("b", 10.0), ("c", 10.0), ("d", 10.0)]] * 5)
    shifted = _cur([("a", 19.0), ("b", 19.5), ("c", 20.5), ("d", 11.0)])  # 3/4 >= 1.75x, 1 >= 2.0x
    code, rep = evaluate(shifted, history_values(series))
    check(code == 0 and rep["uniform_shift"] and len(rep["watch"]) == 3 and len(rep["hard"]) == 1,
          "uniform environment shift (3/4 metrics >= 1.75x) is exempted despite a >= 2.0x metric")

    # 4. single-metric hard fail: one metric >= 2.0x median, the rest normal => exit 1.
    lone = _cur([("a", 21.0), ("b", 10.2), ("c", 9.9), ("d", 10.0)])  # only a regresses (2.1x)
    code, rep = evaluate(lone, history_values(series))
    check(code == 1 and not rep["uniform_shift"] and len(rep["hard"]) == 1
          and rep["hard"][0][0] == "a",
          "single sustained >= 2.0x regression hard-fails (no uniform exemption)")
    # 4b. watch band alone (>= 1.75x but < 2.0x, minority of metrics) does NOT fail.
    watch_only = _cur([("a", 18.0), ("b", 10.0), ("c", 10.0), ("d", 10.0)])
    code, rep = evaluate(watch_only, history_values(series))
    check(code == 0 and len(rep["watch"]) == 1 and not rep["hard"],
          "a lone watch-band metric (1.8x, < 2.0x) is reported but does not fail")

    if failures:
        log(f"self-test FAILED ({len(failures)}): " + "; ".join(failures))
        return 1
    log("self-test OK")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    ap = argparse.ArgumentParser(prog=PROG, description=__doc__)
    ap.add_argument("--results", required=True, help="this run's bench-results.json")
    ap.add_argument("--prev-data", required=True,
                    help="previously-published data.js (prev-data.js; absent => warn + pass)")
    ap.add_argument("--suite", default="sparq engine", help="data.js entries suite name")
    args = ap.parse_args(argv)
    return run_gate(args.results, args.prev_data, args.suite)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
