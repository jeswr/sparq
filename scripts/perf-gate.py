#!/usr/bin/env python3
# [OPUS-4.8] HARD regression GATE on the DETERMINISTIC (runner-noise-immune) perf metrics.
#
# WHY THIS EXISTS
# ---------------
# bench.yml tracks perf with github-action-benchmark, whose `fail-on-alert` is a single GLOBAL
# switch: it cannot fail on ONLY the deterministic metrics while leaving the noisy wall-clock
# latencies trend-only. So fail-on-alert stays false (alert-comment only). This script is the TRUE
# hard gate: it compares the current run's deterministic metrics against the PREVIOUS committed
# point and exits non-zero on a real regression, WITHOUT ever looking at the wall-clock latencies.
#
# WHAT IT GATES (and the per-metric thresholds, all SMALLER-IS-BETTER)
# -------------------------------------------------------------------
#   store_bytes_per_triple        2%   integer memory-layout metric — barely moves run-to-run
#   store_bytes_per_triple_small  2%   second (fixed) scale, catches per-triple-overhead regressions
#   dict_bytes_per_term           2%   integer dictionary memory-layout metric
#   wasm_bundle_bytes             2%   browser bundle size — should only grow DELIBERATELY (see below)
#   parse_ns_per_byte            10%   parse COST on a fixed corpus; min-of-5 so it carries a little
#                                      wall-clock noise (measured ~3.2% run-to-run spread on the
#                                      published series) — hence a looser-but-still-real threshold.
#
# A metric REGRESSES when  current > previous * (1 + threshold).  Improvements (smaller) never fail.
# The integer byte metrics move in whole-byte steps, so a 2% threshold is effectively "integer-exact
# plus a hair" at the current scale and only trips on a genuine layout regression.
#
# HOW THE PREVIOUS VALUE IS READ
# ------------------------------
# The previous committed values are read from the github-action-benchmark history file
# `dev/bench/data.js` on the `benchmark-data` branch (the same series that feeds the Pages chart).
# The LAST entry in entries["sparq engine"] is the most-recent committed point. The gate runs BEFORE
# the action appends the current run, so "last entry" == the true previous point. A metric absent
# from the previous point (e.g. a newly added metric, or the first-ever run) has NO baseline and is
# SKIPPED with a note — it cannot regress against nothing; it establishes the baseline for next time.
#
# DELIBERATE BASELINE BUMP (e.g. a wasm bundle increase from a new feature, or a layout change that
# trades bytes for speed)
# -------------------------------------------------------------------------------------------------
# Two documented paths, no code edit required for the common case:
#   1. Per-run override env var PERF_GATE_ALLOW="metric1,metric2" — the named metrics are allowed to
#      regress for THIS run only (CI passes; the new value is then committed to benchmark-data and
#      becomes the baseline, so subsequent runs gate against it automatically). This is the intended
#      path for a one-off deliberate increase: set it on the merge commit / in a workflow input.
#   2. Loosen / retune a threshold permanently by editing THRESHOLDS below in a reviewed commit.
# Because the gate always compares against the PREVIOUS COMMITTED point (not a frozen baseline file),
# once a deliberate increase lands on main it silently becomes the new floor — no separate baseline
# file to keep in sync.
#
# USAGE
#   perf-gate.py <current-bench-results.json> <previous-data.js>
#   perf-gate.py --self-test          # unit-test the regression logic (no files needed)
# EXIT CODES: 0 = pass, 2 = regression (hard fail), 1 = usage / parse error.
#
# This gate is intentionally STRICT-on-regression / SILENT-on-improvement and only ever reads the
# five deterministic metrics, so it is safe to run on noisy GitHub-hosted runners.

import json
import os
import sys

# Per-metric regression thresholds (fractional, smaller-is-better). See header for rationale.
THRESHOLDS = {
    "store_bytes_per_triple": 0.02,
    "store_bytes_per_triple_small": 0.02,
    "dict_bytes_per_term": 0.02,
    "wasm_bundle_bytes": 0.02,
    "parse_ns_per_byte": 0.10,
}

DETERMINISTIC = set(THRESHOLDS)


def load_current(path):
    """Current run: github-action-benchmark customSmallerIsBetter JSON = list of {name,unit,value}."""
    with open(path) as fh:
        data = json.load(fh)
    return {b["name"]: float(b["value"]) for b in data if b["name"] in DETERMINISTIC}


def load_previous(path):
    """Previous committed point: last entry of entries['sparq engine'] in benchmark-data/dev/bench/data.js.

    The file is JS (`window.BENCHMARK_DATA = { ... };`). We slice from the first '{' and strip a
    trailing ';' to recover valid JSON. Returns {} if the series is empty / missing (first-ever run).
    """
    with open(path) as fh:
        txt = fh.read()
    brace = txt.find("{")
    if brace < 0:
        return {}
    obj = json.loads(txt[brace:].rstrip().rstrip(";").rstrip())
    series = obj.get("entries", {}).get("sparq engine", [])
    if not series:
        return {}
    last = series[-1]
    return {b["name"]: float(b["value"]) for b in last.get("benches", []) if b["name"] in DETERMINISTIC}


def evaluate(current, previous, thresholds, allow=frozenset()):
    """Pure regression logic — unit-tested by --self-test.

    Returns (regressions, report_lines). `regressions` is the list of HARD failures (excludes
    allow-listed metrics). A metric regresses when current > previous*(1+threshold).
    """
    regressions = []
    lines = []
    for name in sorted(thresholds):
        thr = thresholds[name]
        cur = current.get(name)
        prev = previous.get(name)
        if cur is None:
            lines.append(f"  - {name}: SKIP (not in current run)")
            continue
        if prev is None:
            lines.append(f"  - {name}: SKIP (no previous baseline — establishes baseline = {cur:g})")
            continue
        if prev <= 0:
            lines.append(f"  - {name}: SKIP (non-positive previous baseline {prev:g})")
            continue
        limit = prev * (1 + thr)
        delta = (cur - prev) / prev * 100
        if cur > limit:
            if name in allow:
                status = f"ALLOWED-REGRESSION (PERF_GATE_ALLOW), delta {delta:+.2f}% > {thr*100:.0f}%"
            else:
                status = f"REGRESSION delta {delta:+.2f}% > +{thr*100:.0f}%"
                regressions.append((name, prev, cur, delta, thr))
        else:
            status = f"OK delta {delta:+.2f}% (<= +{thr*100:.0f}%)"
        lines.append(f"  - {name}: prev={prev:g} cur={cur:g} -> {status}")
    return regressions, lines


def self_test():
    """Unit-test the pure regression logic on synthetic prev+current. Asserts:
    fails on a real regression, passes on noise within threshold, passes on improvement,
    skips a metric with no baseline, and honours the PERF_GATE_ALLOW override."""
    thr = THRESHOLDS

    # 1) clear regression on a byte metric (+3.26% > 2%) MUST fail.
    prev = {"store_bytes_per_triple": 92, "dict_bytes_per_term": 53, "parse_ns_per_byte": 4.9}
    cur = {"store_bytes_per_triple": 95, "dict_bytes_per_term": 53, "parse_ns_per_byte": 4.9}
    regs, _ = evaluate(cur, prev, thr)
    assert [r[0] for r in regs] == ["store_bytes_per_triple"], regs

    # 2) noise within threshold MUST pass: parse_ns_per_byte +2.86% (< 10%), byte metric +1.09% (< 2%).
    cur2 = {"store_bytes_per_triple": 93, "dict_bytes_per_term": 53, "parse_ns_per_byte": 5.04}
    regs2, _ = evaluate(cur2, prev, thr)
    assert regs2 == [], regs2

    # 3) improvement (smaller) MUST pass.
    cur3 = {"store_bytes_per_triple": 80, "dict_bytes_per_term": 40, "parse_ns_per_byte": 3.0}
    assert evaluate(cur3, prev, thr)[0] == []

    # 4) parse_ns_per_byte +12% (> 10%) MUST fail; wasm exactly at +2% boundary MUST pass.
    prev4 = {"parse_ns_per_byte": 5.0, "wasm_bundle_bytes": 1000}
    cur4 = {"parse_ns_per_byte": 5.6, "wasm_bundle_bytes": 1020}  # +12% parse, exactly +2% wasm
    regs4, _ = evaluate(cur4, prev4, thr)
    assert [r[0] for r in regs4] == ["parse_ns_per_byte"], regs4

    # 5) new metric with NO previous baseline MUST be skipped (cannot regress against nothing).
    prev5 = {"store_bytes_per_triple": 92}
    cur5 = {"store_bytes_per_triple": 92, "store_bytes_per_triple_small": 9999}
    assert evaluate(cur5, prev5, thr)[0] == []

    # 6) first-ever run: empty previous => everything skipped, no failures.
    assert evaluate({"store_bytes_per_triple": 92}, {}, thr)[0] == []

    # 7) PERF_GATE_ALLOW lets a named metric regress (deliberate bump) without failing...
    prev7 = {"wasm_bundle_bytes": 1000}
    cur7 = {"wasm_bundle_bytes": 1100}  # +10% > 2% but allowed
    regs7, _ = evaluate(cur7, prev7, thr, allow={"wasm_bundle_bytes"})
    assert regs7 == [], regs7
    # ... and WITHOUT the allowance it fails.
    assert [r[0] for r in evaluate(cur7, prev7, thr)[0]] == ["wasm_bundle_bytes"]

    # 8) non-positive previous baseline is skipped safely (no div-by-zero).
    assert evaluate({"parse_ns_per_byte": 5.0}, {"parse_ns_per_byte": 0.0}, thr)[0] == []

    print("perf-gate self-test: ALL ASSERTIONS PASSED")
    return 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    if len(argv) != 3:
        sys.stderr.write(
            "usage: perf-gate.py <current-bench-results.json> <previous-data.js>\n"
            "       perf-gate.py --self-test\n"
        )
        return 1
    cur_path, prev_path = argv[1], argv[2]
    current = load_current(cur_path)
    # A missing previous data.js (first-ever run / branch just bootstrapped) => no baseline => pass.
    if os.path.exists(prev_path):
        previous = load_previous(prev_path)
    else:
        previous = {}
        print(f"perf-gate: previous series '{prev_path}' not found — first run, no baseline to gate.")
    allow = {m.strip() for m in os.environ.get("PERF_GATE_ALLOW", "").split(",") if m.strip()}
    if allow:
        print(f"perf-gate: PERF_GATE_ALLOW set — these metrics may regress this run: {sorted(allow)}")

    regressions, lines = evaluate(current, previous, THRESHOLDS, allow=allow)
    print("perf-gate: deterministic-metric comparison (smaller is better):")
    for ln in lines:
        print(ln)

    if regressions:
        print("\nperf-gate: HARD FAIL — deterministic perf regression(s) detected:")
        for name, prev, cur, delta, thr in regressions:
            print(f"  * {name}: {prev:g} -> {cur:g} ({delta:+.2f}%, threshold +{thr*100:.0f}%)")
        print(
            "\nIf this increase is DELIBERATE, re-run with PERF_GATE_ALLOW listing the metric(s),\n"
            "e.g. PERF_GATE_ALLOW='wasm_bundle_bytes'. See scripts/perf-gate.py header for details."
        )
        return 2
    print("\nperf-gate: PASS — no deterministic perf regression.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
