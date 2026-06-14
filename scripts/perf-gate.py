#!/usr/bin/env python3
# [OPUS-4.8] HARD regression RATCHET on the DETERMINISTIC (runner-noise-immune) perf metrics.
#
# WHY THIS EXISTS  (sq-i1d created the gate; sq-52e turned it into a true ratchet)
# -------------------------------------------------------------------------------
# bench.yml tracks perf with github-action-benchmark, whose `fail-on-alert` is a single GLOBAL
# switch: it cannot fail on ONLY the deterministic metrics while leaving the noisy wall-clock
# latencies trend-only. So fail-on-alert stays false (alert-comment only). This script is the TRUE
# hard gate, WITHOUT ever looking at the wall-clock latencies.
#
# THE BUG THIS FIXES — "boiling frog" slow drift (sq-52e)
# -------------------------------------------------------
# The original gate compared each metric only to the PREVIOUS committed point in benchmark-data.
# A sub-threshold regression EVERY commit (e.g. +1.9% on a +2% gate) then slow-drifts UNBOUNDED and
# never trips, even though the metric ends up far worse than the best it ever achieved. This is not
# hypothetical: in the published series wasm_bundle_bytes crept 1518547 -> 1579288 (+4.0%) across
# many commits, each step under threshold — a prev-only gate waved every step through. The repo's
# CONFORMANCE ratchets (.github/workflows/ci.yml) never have this gap: they gate against a committed
# best-ever count that only tightens. This gate now does the same for perf.
#
# THE RATCHET — gate against a COMMITTED best-ever FLOOR, not the previous point
# -----------------------------------------------------------------------------
# Floors live in bench/perf-baseline.json (the source-of-record ratchet, reviewed in diffs like the
# conformance RATCHET number). A metric REGRESSES when  current > floor*(1+threshold). Because the
# floor is the best-ever value (not the drifting previous point), N successive +1.8% creeps now sum
# against a FIXED floor and trip well before they would have under prev-only gating.
#
# PER-METRIC THRESHOLDS + MODE (all SMALLER-IS-BETTER)
# ----------------------------------------------------
#   store_bytes_per_triple        2%   auto   integer memory-layout metric — barely moves run-to-run
#   store_bytes_per_triple_small  2%   auto   second (fixed) scale, catches per-triple-overhead regs
#   dict_bytes_per_term           2%   auto   integer dictionary memory-layout metric
#   wasm_bundle_bytes             2%   auto   browser bundle size — grows only DELIBERATELY (see RAISE)
#   parse_ns_per_byte            12%   noise  wall-clock-derived (~3% run-to-run on the published
#                                             series); wide NOISE band + best-of-N re-measure (below).
# The 2% byte metrics are integer-exact at the current scale, so 2% is "integer-exact plus a hair".
#
# auto  = the floor AUTO-RATCHETS DOWN on a genuine improvement (current < floor) and NEVER auto-raises.
# noise = the floor only ratchets DOWN on a SUSTAINED improvement (a supplied median/percentile of
#         recent points is below the floor), so a single noise-low can never tighten it.
#
# DETERMINISTIC vs TIMING — the metric classification that drives flake-robustness  [OPUS-4.8] (sq-dzfu)
# -----------------------------------------------------------------------------------------------------
# The classification is DATA-DRIVEN from each metric's `mode` (it is NOT a hard-coded name list):
#   * DETERMINISTIC (mode == "auto"): integer byte-count / memory-layout metrics. These are immune to
#     runner noise (the same input always yields the same bytes), so they are HARD-GATED strictly: any
#     value above floor*(1+threshold) FAILS, no re-measure. The byte-count ratchet must stay strict.
#   * TIMING (mode == "noise"): wall-clock-derived metrics (parse_ns_per_byte). On a shared GitHub
#     runner the measurement only ever INFLATES from preemption/contention — it never reads faster than
#     the true cost. So a single high reading is noise, not a regression.
# is_timing(cfg) / is_deterministic(cfg) below are the single source of truth for this split.
#
# THE FLAKE THIS FIXES — runner noise hard-failing an unrelated PR  (sq-dzfu)
# --------------------------------------------------------------------------
# parse_ns_per_byte intermittently exceeded even its wide 12% band purely from runner contention (e.g.
# 5.5888 vs floor 4.895 = +14.17%) on a PR that changed ZERO parsing code, hard-failing the merge train
# via the ci-summary aggregator. The fix is BEST-OF-N RE-MEASURE for TIMING metrics, exactly analogous
# to PR #62's coverage fix (an undercount → re-run the measurement → take the better value):
#
# BEST-OF-N RE-MEASURE (timing metrics only) — `--remeasure-cmd` + `--remeasure-k`
# --------------------------------------------------------------------------------
# When a TIMING metric exceeds its band AND a re-measure command is supplied, the gate re-runs that
# command up to K times (default 4) and takes the BEST (min — smaller is better) value across the
# original + all re-measures. It only FAILS if the BEST of all readings still exceeds the band. Because
# timing noise only inflates, best-of-N converges to the true value, so a real parse regression (slow on
# EVERY reading) still fails while a one-off noise spike is squeezed out. DETERMINISTIC metrics are
# NEVER re-measured (re-running can't change an integer byte count) — they hard-fail on the first read.
# The re-measure command must re-emit the SAME customSmallerIsBetter JSON (a {name,unit,value} list);
# the gate reads back the timing metric(s) from it. With NO --remeasure-cmd the gate degrades to the
# wide-band-only behaviour (still robust, just without the extra squeeze) — used by --self-test, which
# injects a synthetic re-measure callback instead of shelling out.
#
# AUTO-RATCHET-DOWN (the floor only tightens by itself) — see bench.yml "Auto-ratchet" step
# -----------------------------------------------------------------------------------------
# On push to main (only), AFTER the gate passes, the workflow runs `perf-gate.py --update-baseline`
# which lowers any floor a genuine improvement beat, writes bench/perf-baseline.json, and — if it
# changed — commits it back to main with [skip ci] in the message (anti-loop: the bench workflow
# ignores pushes that only touch this path / carry [skip ci], so the commit can't re-trigger itself).
#
# RAISE a floor (deliberate regression — e.g. a wasm bump from a new feature) — EXPLICIT ONLY
# -------------------------------------------------------------------------------------------
#   1. Edit bench/perf-baseline.json in a reviewed PR (the floor change is visible in the diff). OR
#   2. Per-run env PERF_GATE_ALLOW="metric1,metric2": the named metrics may regress THIS run AND are
#      RE-FLOORED to the current value by --update-baseline (so the new, higher value becomes the
#      committed floor going forward). Use this on the merge commit that intentionally regresses.
# A floor NEVER raises on its own — only these two human-in-the-loop paths can loosen it.
#
# USAGE
#   perf-gate.py <current-bench-results.json> [perf-baseline.json]   # gate (default baseline path below)
#       [--remeasure-cmd "<shell>"] [--remeasure-k N]                # best-of-N re-measure for TIMING
#                                   # metrics: on a timing regression re-run <shell> up to N times and
#                                   # take the BEST value; only fail if the best still regresses.
#   perf-gate.py --update-baseline <current-bench-results.json> [baseline.json] [--recent a,b,c]
#                                   # recompute + rewrite floors; exit 0 if changed, 3 if unchanged
#   perf-gate.py --self-test        # unit-test the ratchet logic (no files needed)
# EXIT CODES: 0 = pass / baseline changed, 2 = regression (hard fail), 1 = usage / parse error,
#             3 = --update-baseline made no change (nothing to commit).
#
# This ratchet is STRICT-on-regression / SILENT-on-improvement for the DETERMINISTIC metrics, and is
# robust to runner noise on the TIMING metrics (wide band + best-of-N re-measure), so it is safe to run
# on noisy GitHub-hosted runners — runner jitter never blocks a merge, real regressions still fail.

import json
import os
import sys

# Default committed ratchet location (source-of-record floor per deterministic metric).
DEFAULT_BASELINE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bench", "perf-baseline.json")

# Fallback per-metric config if a baseline file is absent (first bootstrap). The floor=None means
# "no floor yet — establish it from the current run" (cannot regress against nothing).
DEFAULT_METRICS = {
    "store_bytes_per_triple":       {"floor": None, "threshold": 0.02, "mode": "auto"},
    "store_bytes_per_triple_small": {"floor": None, "threshold": 0.02, "mode": "auto"},
    "dict_bytes_per_term":          {"floor": None, "threshold": 0.02, "mode": "auto"},
    "wasm_bundle_bytes":            {"floor": None, "threshold": 0.02, "mode": "auto"},
    "parse_ns_per_byte":            {"floor": None, "threshold": 0.12, "mode": "noise"},
}

# [OPUS-4.8] (sq-dzfu) DATA-DRIVEN metric classification — the single source of truth for the
# DETERMINISTIC-vs-TIMING split. It is derived from each metric's `mode`, NOT a hard-coded name list,
# so a metric's classification follows its baseline config and never drifts out of sync:
#   * mode "noise" => TIMING (wall-clock-derived; runner noise only ever INFLATES it). Robust handling:
#                     wide band + best-of-N re-measure (re-running squeezes out a one-off noise spike).
#   * anything else (mode "auto") => DETERMINISTIC (integer byte counts / memory layout; re-running
#                     CANNOT change the value). Hard-gated strictly: a single read above band FAILS.
# The default total: any extra `mode` value falls into DETERMINISTIC (strict) — fail-safe.
TIMING_MODES = frozenset({"noise"})
# Default number of best-of-N re-measures for a TIMING-metric regression (the workflow may override
# with --remeasure-k). 4 retries (5 readings incl. the original) reliably drowns a single noise spike;
# more would just lengthen CI for no extra robustness on a metric the harness already mins-of-5.
DEFAULT_REMEASURE_K = 4


def is_timing(cfg):
    """A TIMING (wall-clock-derived, runner-noise-prone) metric — eligible for best-of-N re-measure."""
    return cfg.get("mode") in TIMING_MODES


def is_deterministic(cfg):
    """A DETERMINISTIC (integer byte-count / memory-layout) metric — hard-gated strictly, never re-measured."""
    return not is_timing(cfg)


def load_baseline(path):
    """Read the committed ratchet floors. Returns {metric: {floor, threshold, mode}}.

    Missing file => DEFAULT_METRICS with floor=None (bootstrap: nothing to gate against yet).
    """
    if not path or not os.path.exists(path):
        return {k: dict(v) for k, v in DEFAULT_METRICS.items()}
    with open(path) as fh:
        obj = json.load(fh)
    metrics = obj.get("metrics", {})
    out = {}
    for name, cfg in metrics.items():
        out[name] = {
            "floor": (None if cfg.get("floor") is None else float(cfg["floor"])),
            "threshold": float(cfg.get("threshold", 0.02)),
            "mode": cfg.get("mode", "auto"),
        }
    # Any DEFAULT metric not present in the file is bootstrapped (floor=None).
    for name, cfg in DEFAULT_METRICS.items():
        out.setdefault(name, dict(cfg))
    return out


def write_baseline(path, baseline, preserve_comment_from=None):
    """Write floors back to disk, preserving the human _comment block if one exists."""
    comment = None
    if preserve_comment_from and os.path.exists(preserve_comment_from):
        try:
            with open(preserve_comment_from) as fh:
                comment = json.load(fh).get("_comment")
        except (OSError, ValueError):
            comment = None
    obj = {}
    if comment is not None:
        obj["_comment"] = comment
    obj["metrics"] = {
        name: {
            "floor": (None if cfg["floor"] is None else _round_floor(cfg["floor"])),
            "threshold": cfg["threshold"],
            "mode": cfg["mode"],
        }
        for name, cfg in baseline.items()
    }
    with open(path, "w") as fh:
        json.dump(obj, fh, indent=2)
        fh.write("\n")


def _round_floor(v):
    """Keep integer-valued floors as ints (byte metrics) and round noisy ones to 4 dp."""
    if float(v).is_integer():
        return int(v)
    return round(float(v), 4)


def load_current(path, metric_names):
    """Current run: github-action-benchmark customSmallerIsBetter JSON = list of {name,unit,value}."""
    with open(path) as fh:
        data = json.load(fh)
    return {b["name"]: float(b["value"]) for b in data if b["name"] in metric_names}


def _parse_remeasure_stdout(stdout, metric_names):
    """[OPUS-4.8] (sq-dzfu) Defensive parse of a re-measure command's stdout into {metric: float}.

    The expected shape is the github-action-benchmark customSmallerIsBetter JSON: a LIST of
    {name, unit, value} objects. This is hardened against *valid-but-unexpected* JSON so a re-measure
    can NEVER crash the gate (the whole point of re-measure is a safety net — a broken net must degrade
    to a no-op, not a traceback). Specifically: if the payload is not a list (e.g. `{}` — a dict), or an
    entry is not a dict, or an entry lacks a `name`/`value` field, or `value` is non-numeric, that entry
    is skipped (and a non-list payload yields {} — "no improvement", keeping the prior value). Returns
    {} on any parse failure. Happy-path behaviour is identical to the previous comprehension."""
    try:
        data = json.loads(stdout)
    except ValueError:
        # The harness prints "wrote ...:" then the JSON — fall back to the first '[' onward.
        brace = stdout.find("[")
        if brace < 0:
            sys.stderr.write("perf-gate: re-measure output was not valid JSON.\n")
            return {}
        try:
            data = json.loads(stdout[brace:])
        except ValueError:
            sys.stderr.write("perf-gate: re-measure output was not valid JSON.\n")
            return {}
    # [OPUS-4.8] guard: a valid-but-unexpected payload (a dict like `{}`, a scalar, a list of non-dicts,
    # entries missing `value` or with a non-numeric `value`) must degrade to a safe no-op, never raise.
    if not isinstance(data, list):
        sys.stderr.write(
            f"perf-gate: re-measure JSON was not a list of metric objects (got {type(data).__name__}); "
            "ignoring it (no improvement applied).\n"
        )
        return {}
    out = {}
    for b in data:
        if not isinstance(b, dict):
            sys.stderr.write("perf-gate: re-measure entry was not an object; skipping it.\n")
            continue
        name = b.get("name")
        if name not in metric_names:
            continue
        if "value" not in b:
            sys.stderr.write(f"perf-gate: re-measure entry {name!r} had no 'value'; skipping it.\n")
            continue
        try:
            out[name] = float(b["value"])
        except (TypeError, ValueError):
            sys.stderr.write(
                f"perf-gate: re-measure entry {name!r} had a non-numeric value {b['value']!r}; skipping it.\n"
            )
            continue
    return out


def make_shell_remeasure(cmd, metric_names):
    """[OPUS-4.8] (sq-dzfu) Build a remeasure callback that runs `cmd` (a shell command that re-emits the
    customSmallerIsBetter JSON to stdout) and parses the metric values back out. Returns a 0-arg fn
    suitable for gate_with_remeasure; on a non-zero exit / unparseable / malformed-but-valid output it
    returns {} or skips the bad entries (treated as 'no improvement', so a broken re-measure can only
    ever leave the gate where it was — never PASS a real regression, never crash). Imported lazily so
    --self-test never needs subprocess."""
    import subprocess

    def _run():
        try:
            proc = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=600)
        except (OSError, subprocess.SubprocessError) as exc:
            sys.stderr.write(f"perf-gate: re-measure command failed to run: {exc}\n")
            return {}
        if proc.returncode != 0:
            sys.stderr.write(f"perf-gate: re-measure command exited {proc.returncode}; stderr:\n{proc.stderr}\n")
            return {}
        return _parse_remeasure_stdout(proc.stdout, metric_names)

    return _run


def load_series_metric(data_js_path, metric, n=6):
    """Return the last `n` published values of `metric` from a benchmark-data dev/bench/data.js file.

    Used to feed the NOISE-banded metric's sustained-improvement check with a median of recent points
    (so its floor never ratchets to a one-off noise-low). Returns [] if unreadable.
    """
    try:
        with open(data_js_path) as fh:
            txt = fh.read()
    except OSError:
        return []
    brace = txt.find("{")
    if brace < 0:
        return []
    try:
        obj = json.loads(txt[brace:].rstrip().rstrip(";").rstrip())
    except ValueError:
        return []
    series = obj.get("entries", {}).get("sparq engine", [])
    vals = []
    for pt in series:
        for b in pt.get("benches", []):
            if b["name"] == metric:
                vals.append(float(b["value"]))
    return vals[-n:]


def _median(xs):
    s = sorted(xs)
    k = len(s)
    if k == 0:
        return None
    mid = k // 2
    return s[mid] if k % 2 else (s[mid - 1] + s[mid]) / 2.0


def evaluate(current, baseline, allow=frozenset()):
    """Pure ratchet logic — unit-tested by --self-test.

    Gate each metric against its COMMITTED best-ever FLOOR (not the previous point), so sub-threshold
    creep can no longer slow-drift past the floor. A metric regresses when current > floor*(1+threshold).
    Returns (regressions, report_lines). `regressions` excludes allow-listed (deliberate-bump) metrics.
    """
    regressions = []
    lines = []
    for name in sorted(baseline):
        cfg = baseline[name]
        thr = cfg["threshold"]
        floor = cfg["floor"]
        mode = cfg["mode"]
        cur = current.get(name)
        if cur is None:
            lines.append(f"  - {name}: SKIP (not in current run)")
            continue
        if floor is None:
            lines.append(f"  - {name}: SKIP (no floor yet — establishes floor = {cur:g})")
            continue
        if floor <= 0:
            lines.append(f"  - {name}: SKIP (non-positive floor {floor:g})")
            continue
        limit = floor * (1 + thr)
        delta = (cur - floor) / floor * 100
        tag = "noise-band" if mode == "noise" else "ratchet"
        if cur > limit:
            if name in allow:
                status = (f"ALLOWED-REGRESSION (PERF_GATE_ALLOW), delta {delta:+.2f}% > {thr*100:.0f}% "
                          f"[{tag}] — floor will be RAISED to {cur:g}")
            else:
                status = f"REGRESSION delta {delta:+.2f}% > +{thr*100:.0f}% above best-ever floor [{tag}]"
                regressions.append((name, floor, cur, delta, thr))
        elif cur < floor:
            status = f"IMPROVEMENT delta {delta:+.2f}% (below floor) [{tag}] — floor may ratchet DOWN"
        else:
            status = f"OK delta {delta:+.2f}% (<= +{thr*100:.0f}% above floor) [{tag}]"
        lines.append(f"  - {name}: floor={floor:g} cur={cur:g} -> {status}")
    return regressions, lines


def gate_with_remeasure(current, baseline, allow=frozenset(), remeasure_fn=None, k=DEFAULT_REMEASURE_K):
    """[OPUS-4.8] (sq-dzfu) Best-of-N re-measure wrapper around `evaluate` for TIMING metrics.

    Runs the gate. If the ONLY thing standing between PASS and FAIL is TIMING-metric regression(s) and a
    `remeasure_fn` is supplied, re-measure those timing metrics up to `k` times, keeping the BEST (min —
    smaller is better) value seen for each, and re-gate. A DETERMINISTIC regression is NEVER re-measured
    (re-running can't change an integer byte count) and fails immediately on the first read; if a
    deterministic metric is among the regressions we don't even bother re-measuring (the run fails
    regardless), so we never waste CI minutes on a re-run that can't clear the gate.

    `remeasure_fn() -> {metric_name: float}` returns a FRESH full reading of the metrics (the production
    callback shells out to the re-measure command and parses its customSmallerIsBetter JSON; the
    self-test injects a synthetic callback). Only the values of TIMING metrics are adopted from it (and
    only when strictly better), so a re-measure can never WORSEN a metric or touch a deterministic one.

    Returns (regressions, report_lines, effective_current): `effective_current` carries the best timing
    values seen so the caller can report/feed them onward. With no `remeasure_fn`, behaves like
    `evaluate` (wide-band-only — still robust, just without the extra squeeze).
    """
    effective = dict(current)
    regressions, lines = evaluate(effective, baseline, allow=allow)
    if not regressions or remeasure_fn is None:
        return regressions, lines, effective

    # Which of the regressions are TIMING (re-measurable) vs DETERMINISTIC (strict, not re-measurable)?
    reg_names = {r[0] for r in regressions}
    timing_regs = {n for n in reg_names if is_timing(baseline.get(n, {}))}
    det_regs = reg_names - timing_regs
    if det_regs:
        # A deterministic regression fails the run no matter what a re-measure of the timing metrics
        # would show — don't spend CI minutes re-running. Return the original (strict) result.
        lines.append(
            f"  [remeasure] SKIPPED — deterministic regression(s) {sorted(det_regs)} fail strictly; "
            f"re-measuring timing metric(s) {sorted(timing_regs)} can't clear the gate."
        )
        return regressions, lines, effective
    if not timing_regs:
        return regressions, lines, effective

    lines.append(
        f"  [remeasure] TIMING regression(s) {sorted(timing_regs)} exceeded band on the first reading — "
        f"best-of-{k + 1} re-measure (noise only ever inflates a wall-clock reading)…"
    )
    for attempt in range(1, k + 1):
        fresh = remeasure_fn() or {}
        improved = []
        for name in timing_regs:
            val = fresh.get(name)
            if val is None:
                continue
            val = float(val)
            if name not in effective or val < effective[name]:
                improved.append(f"{name}={val:g}")
                effective[name] = val
        lines.append(
            f"  [remeasure] attempt {attempt}/{k}: "
            + (("new best " + ", ".join(improved)) if improved else "no improvement on the best so far")
        )
        # Re-gate with the best-so-far timing values. If the timing regressions all cleared, stop early.
        regressions, eval_lines = evaluate(effective, baseline, allow=allow)
        if not any(r[0] in timing_regs for r in regressions):
            lines.append(f"  [remeasure] timing metric(s) cleared the band after {attempt} re-measure(s).")
            lines.extend(eval_lines)
            return regressions, lines, effective
    # Best of K still over band -> a genuine (sustained-across-all-readings) timing regression.
    regressions, eval_lines = evaluate(effective, baseline, allow=allow)
    lines.append(
        f"  [remeasure] best of {k + 1} readings still exceeds the band — TIMING regression is REAL "
        f"(not runner noise) and FAILS."
    )
    lines.extend(eval_lines)
    return regressions, lines, effective


def update_baseline(current, baseline, allow=frozenset(), recent=None):
    """Compute the NEW floors after a (passing) run. Returns (new_baseline, changes).

    Rules (the floor only TIGHTENS automatically; loosening is explicit-only):
      * auto  metric, current < floor              -> ratchet DOWN to current (genuine improvement).
      * noise metric                               -> ratchet DOWN only on a SUSTAINED improvement:
            the floor lowers to median(recent) and ONLY if that median is strictly below the floor,
            so a one-off noise-low (median still at/above floor) can never tighten it. With no recent
            series supplied we fall back to `current` (best-effort; the workflow always passes one).
      * any metric in `allow` (PERF_GATE_ALLOW)    -> RE-FLOOR to current (the explicit deliberate
            raise/lower path: the new value becomes the committed floor).
      * floor is None (bootstrap)                  -> establish floor = current.
    `recent` maps metric -> list of recent published values (for the noise sustained-improvement test).
    """
    recent = recent or {}
    new = {name: dict(cfg) for name, cfg in baseline.items()}
    changes = []
    for name, cfg in new.items():
        cur = current.get(name)
        if cur is None:
            continue
        floor = cfg["floor"]
        mode = cfg["mode"]
        if name in allow:
            if floor != cur:
                changes.append((name, floor, cur, "explicit re-floor (PERF_GATE_ALLOW)"))
            cfg["floor"] = cur
            continue
        if floor is None:
            cfg["floor"] = cur
            changes.append((name, None, cur, "bootstrap (no prior floor)"))
            continue
        if mode == "noise":
            # SUSTAINED = the recent-points MEDIAN must itself be below the floor (a one-off low leaves
            # the median at/above the floor, so it does not tighten). Fall back to `current` only when
            # no recent series is supplied (the workflow always supplies one via --recent-data).
            med = _median(recent.get(name, [])) if recent.get(name) else cur
            if med < floor:
                cfg["floor"] = med
                changes.append((name, floor, med, "sustained improvement (noise floor lowered)"))
        else:  # auto
            if cur < floor:
                cfg["floor"] = cur
                changes.append((name, floor, cur, "auto-ratchet down (improvement)"))
    return new, changes


def self_test():
    """Unit-test the pure ratchet logic on synthetic floors+current. The LOAD-BEARING case is the
    slow-drift adversarial test (#1): it is exactly the scenario the prev-only gate failed."""

    def mk(floor_map):
        return {n: {"floor": f, "threshold": (0.12 if n == "parse_ns_per_byte" else 0.02),
                    "mode": ("noise" if n == "parse_ns_per_byte" else "auto")}
                for n, f in floor_map.items()}

    # ============================================================================================
    # 1) SLOW-DRIFT ADVERSARIAL TEST (the crux — sq-52e). Simulate N commits each regressing a
    #    deterministic byte metric by JUST UNDER the per-step (prev-only) threshold (+1.8% on a 2%
    #    gate). A prev-only gate compares step-to-step and PASSES every step (1.8% < 2% each time),
    #    so the metric drifts unbounded. The best-ever RATCHET compares each step to the FIXED floor
    #    and MUST trip once cumulative drift exceeds the band.
    # --------------------------------------------------------------------------------------------
    FLOOR0 = 92.0
    STEP = 1.018  # +1.8% per commit, < the 2% per-step threshold
    N = 10
    base = mk({"store_bytes_per_triple": FLOOR0})

    # (a) The prev-only model: each step gated only against the immediately-previous point. PROVE it
    #     never trips — this is the bug.
    prev_only_tripped = False
    v = FLOOR0
    series = []
    for _ in range(N):
        nxt = v * STEP
        # prev-only regression test: nxt > prev*(1+0.02) ?
        if nxt > v * 1.02:
            prev_only_tripped = True
        series.append(nxt)
        v = nxt
    assert not prev_only_tripped, "prev-only should NEVER trip on +1.8% steps (that is the bug)"
    total_drift = (series[-1] - FLOOR0) / FLOOR0 * 100
    assert total_drift > 18.0, f"expected ~+19% cumulative drift, got {total_drift:.1f}%"

    # (b) The best-ever RATCHET: gate each drifted point against the FIXED floor. It must trip — and
    #     in fact trips at the FIRST step that crosses floor*1.02 (here step 2: 1.018^2 = +3.6%).
    ratchet_first_trip = None
    for i, val in enumerate(series, start=1):
        regs, _ = evaluate({"store_bytes_per_triple": val}, base)
        if regs:
            ratchet_first_trip = i
            break
    assert ratchet_first_trip is not None, "best-ever ratchet MUST trip on cumulative slow drift"
    # And unambiguously: the FINAL drifted point is a hard failure under the ratchet.
    regs_final, _ = evaluate({"store_bytes_per_triple": series[-1]}, base)
    assert [r[0] for r in regs_final] == ["store_bytes_per_triple"], regs_final
    print(f"  [slow-drift] prev-only gate: PASSED all {N} steps (cumulative {total_drift:.1f}% drift, "
          f"unbounded boiling-frog)")
    print(f"  [slow-drift] best-ever ratchet: TRIPPED at step {ratchet_first_trip} and on the final "
          f"point ({total_drift:.1f}% > +2% floor band) — bug fixed")

    # 2) a SINGLE big regression trips immediately.
    regs2, _ = evaluate({"store_bytes_per_triple": 100.0}, base)  # +8.7% > 2%
    assert [r[0] for r in regs2] == ["store_bytes_per_triple"], regs2

    # 3) a genuine improvement PASSES the gate and ratchets the floor DOWN.
    impr = {"store_bytes_per_triple": 80.0}
    assert evaluate(impr, base)[0] == []
    new3, ch3 = update_baseline(impr, base)
    assert new3["store_bytes_per_triple"]["floor"] == 80.0, new3
    assert any(c[0] == "store_bytes_per_triple" and c[3].startswith("auto-ratchet") for c in ch3), ch3
    # ...and a non-improving run does NOT move an auto floor.
    _, ch3b = update_baseline({"store_bytes_per_triple": 92.0}, base)
    assert ch3b == [], ch3b

    # 4) DELIBERATE raise via PERF_GATE_ALLOW: passes the gate AND re-floors to the new (higher) value.
    baseW = mk({"wasm_bundle_bytes": 1_000_000.0})
    raise_run = {"wasm_bundle_bytes": 1_100_000.0}  # +10% > 2%
    assert evaluate(raise_run, baseW, allow={"wasm_bundle_bytes"})[0] == []          # allowed -> passes
    assert [r[0] for r in evaluate(raise_run, baseW)[0]] == ["wasm_bundle_bytes"]    # without allow -> fails
    newW, chW = update_baseline(raise_run, baseW, allow={"wasm_bundle_bytes"})
    assert newW["wasm_bundle_bytes"]["floor"] == 1_100_000.0, newW
    assert any("explicit re-floor" in c[3] for c in chW), chW

    # 4b) DELIBERATE raise via reviewed baseline EDIT: a human bumps the floor; the higher current
    #     value then sits within band and passes (models editing bench/perf-baseline.json in a PR).
    edited = mk({"wasm_bundle_bytes": 1_100_000.0})  # human-raised floor committed in the PR
    assert evaluate({"wasm_bundle_bytes": 1_100_000.0}, edited)[0] == []

    # 5) NOISE metric: within-noise jitter does NOT trip, AND a one-off noise-low does NOT ratchet
    #    the floor; but a SUSTAINED improvement (recent median below floor) DOES lower it, and
    #    sustained DRIFT above the noise band DOES trip.
    baseP = mk({"parse_ns_per_byte": 4.95})
    # within-noise jitter (+3% over best-ever) << 12% band -> pass.
    assert evaluate({"parse_ns_per_byte": 5.10}, baseP)[0] == []
    # one-off noise-low: current dips but recent median is still at/above floor -> floor UNCHANGED.
    _, chN1 = update_baseline({"parse_ns_per_byte": 4.40}, baseP,
                              recent={"parse_ns_per_byte": [4.95, 4.97, 4.93, 4.96, 4.40]})
    assert chN1 == [], chN1  # median(recent)=4.955 not below floor -> no ratchet on a single low
    # sustained improvement: recent median clearly below floor -> floor lowers to the median.
    newN, chN2 = update_baseline({"parse_ns_per_byte": 4.30}, baseP,
                                 recent={"parse_ns_per_byte": [4.40, 4.35, 4.30, 4.32, 4.28]})
    assert newN["parse_ns_per_byte"]["floor"] < 4.95, newN
    assert any("sustained improvement" in c[3] for c in chN2), chN2
    # sustained DRIFT above the noise band trips (parse at +14% over floor > 12%).
    regsN, _ = evaluate({"parse_ns_per_byte": 4.95 * 1.14}, baseP)
    assert [r[0] for r in regsN] == ["parse_ns_per_byte"], regsN

    # 6) new metric with NO floor yet is SKIPPED (cannot regress against nothing) and bootstraps.
    base6 = {"store_bytes_per_triple": {"floor": 92.0, "threshold": 0.02, "mode": "auto"},
             "store_bytes_per_triple_small": {"floor": None, "threshold": 0.02, "mode": "auto"}}
    assert evaluate({"store_bytes_per_triple": 92.0, "store_bytes_per_triple_small": 9999.0}, base6)[0] == []
    new6, ch6 = update_baseline({"store_bytes_per_triple_small": 9999.0}, base6)
    assert new6["store_bytes_per_triple_small"]["floor"] == 9999.0, new6
    assert any(c[0] == "store_bytes_per_triple_small" and "bootstrap" in c[3] for c in ch6), ch6

    # 7) wasm exactly at the +2% boundary PASSES (boundary is inclusive: not strictly greater).
    base7 = mk({"wasm_bundle_bytes": 1000.0})
    assert evaluate({"wasm_bundle_bytes": 1020.0}, base7)[0] == []          # exactly +2% -> OK
    assert [r[0] for r in evaluate({"wasm_bundle_bytes": 1021.0}, base7)[0]] == ["wasm_bundle_bytes"]

    # 8) non-positive / None floor is skipped safely (no div-by-zero).
    assert evaluate({"parse_ns_per_byte": 5.0}, mk({"parse_ns_per_byte": 0.0}))[0] == []

    # 9) metric CLASSIFICATION is data-driven from `mode` (sq-dzfu), not a hard-coded name list.
    assert is_timing({"mode": "noise"}) and not is_deterministic({"mode": "noise"})
    assert is_deterministic({"mode": "auto"}) and not is_timing({"mode": "auto"})
    assert is_deterministic({"mode": "anything-else"}), "unknown mode must be DETERMINISTIC (fail-safe)"
    baseMix = mk({"parse_ns_per_byte": 4.95, "store_bytes_per_triple": 92.0})
    assert sorted(n for n, c in baseMix.items() if is_timing(c)) == ["parse_ns_per_byte"]
    assert sorted(n for n, c in baseMix.items() if is_deterministic(c)) == ["store_bytes_per_triple"]

    # ============================================================================================
    # 10) BEST-OF-N RE-MEASURE for TIMING metrics (sq-dzfu — the load-bearing flake fix). This is the
    #     exact production scenario: parse_ns_per_byte spikes to +14.17% over floor on a noisy runner
    #     (the band is 12%), but a re-measure reads the true (within-band) value. With a re-measure
    #     callback the gate must PASS; without one it would (wide-band-only) FAIL — proving the
    #     re-measure is what kills THIS flake.
    # --------------------------------------------------------------------------------------------
    baseT = mk({"parse_ns_per_byte": 4.895})
    spike = {"parse_ns_per_byte": 5.5888}  # +14.17% > 12% band — the real-world flake value
    # (a) wide-band-only (no remeasure): the spike FAILS — this is the flake we are killing.
    regsA, _, _ = gate_with_remeasure(spike, baseT, remeasure_fn=None)
    assert [r[0] for r in regsA] == ["parse_ns_per_byte"], regsA
    # (b) best-of-N: a single noisy-high reading then a good (true) reading on re-measure -> PASS.
    good_reading = {"parse_ns_per_byte": 4.95}  # within the 12% band over floor
    calls = {"n": 0}

    def remeasure_good():
        calls["n"] += 1
        return good_reading  # the re-run reads the true, within-band cost

    regsB, linesB, effB = gate_with_remeasure(spike, baseT, remeasure_fn=remeasure_good, k=4)
    assert regsB == [], regsB                              # cleared by re-measure -> PASS
    assert effB["parse_ns_per_byte"] == 4.95, effB         # adopted the better value
    assert calls["n"] == 1, "should stop re-measuring as soon as the band clears"
    assert any("cleared the band" in ln for ln in linesB), linesB

    # 11) a GENUINE timing regression (slow on the ORIGINAL and on EVERY re-measure) still FAILS — the
    #     parse-perf guard survives. Best-of-N converges to the true (regressed) value, not noise.
    bad_reading = {"parse_ns_per_byte": 6.0}  # +22.6% over floor on every reading
    bad_calls = {"n": 0}

    def remeasure_bad():
        bad_calls["n"] += 1
        return {"parse_ns_per_byte": 6.0 + 0.1 * bad_calls["n"]}  # all readings far over band

    regsC, linesC, _ = gate_with_remeasure(bad_reading, baseT, remeasure_fn=remeasure_bad, k=4)
    assert [r[0] for r in regsC] == ["parse_ns_per_byte"], regsC   # real regression still FAILS
    assert bad_calls["n"] == 4, "a real regression should exhaust all K re-measures before failing"
    assert any("REAL" in ln for ln in linesC), linesC

    # 12) a DETERMINISTIC regression is NEVER re-measured (re-running can't change a byte count): it
    #     fails strictly on the first read, and a timing re-measure is not even attempted when a
    #     deterministic metric is also regressed (no wasted CI minutes).
    baseD = mk({"store_bytes_per_triple": 92.0, "parse_ns_per_byte": 4.895})
    det_calls = {"n": 0}

    def remeasure_should_not_run():
        det_calls["n"] += 1
        return {"parse_ns_per_byte": 4.90, "store_bytes_per_triple": 92.0}

    # deterministic regressed AND timing spiked: deterministic fails strictly; re-measure is skipped.
    regsD, linesD, _ = gate_with_remeasure(
        {"store_bytes_per_triple": 100.0, "parse_ns_per_byte": 5.5888}, baseD,
        remeasure_fn=remeasure_should_not_run, k=4)
    assert "store_bytes_per_triple" in [r[0] for r in regsD], regsD
    assert det_calls["n"] == 0, "deterministic regression must NOT trigger a re-measure"
    assert any("SKIPPED" in ln for ln in linesD), linesD
    # deterministic-only regression with NO timing trip: still fails, never re-measured.
    regsD2, _, _ = gate_with_remeasure({"store_bytes_per_triple": 100.0, "parse_ns_per_byte": 4.90},
                                       baseD, remeasure_fn=remeasure_should_not_run, k=4)
    assert [r[0] for r in regsD2] == ["store_bytes_per_triple"], regsD2

    # 13) a flaky/broken re-measure command (returns {} — non-zero exit modelled) can only ever leave
    #     the gate where it was; it can NEVER turn a real regression into a PASS.
    regsE, _, _ = gate_with_remeasure(bad_reading, baseT, remeasure_fn=lambda: {}, k=3)
    assert [r[0] for r in regsE] == ["parse_ns_per_byte"], regsE

    # ============================================================================================
    # 14) [OPUS-4.8] (sq-dzfu) DEFENSIVE re-measure PARSE — a re-measure command that emits *valid* JSON
    #     of an UNEXPECTED shape must degrade to a safe no-op, NEVER raise a traceback that crashes the
    #     gate. (Copilot finding on PR #72.) `_parse_remeasure_stdout` is exactly what the shell
    #     re-measure callback uses, so testing it directly covers the production path without subprocess.
    # --------------------------------------------------------------------------------------------
    mset = {"parse_ns_per_byte"}
    # happy path is UNCHANGED: a well-formed list of metric objects parses to {name: float}.
    assert _parse_remeasure_stdout(
        '[{"name":"parse_ns_per_byte","unit":"ns/byte","value":4.95}]', mset
    ) == {"parse_ns_per_byte": 4.95}
    # valid JSON but a DICT (e.g. `{}`) — not a list -> {} (no-op), no crash.
    assert _parse_remeasure_stdout("{}", mset) == {}
    assert _parse_remeasure_stdout('{"parse_ns_per_byte": 4.95}', mset) == {}
    # valid JSON scalar / null -> {} (no-op), no crash.
    assert _parse_remeasure_stdout("42", mset) == {}
    assert _parse_remeasure_stdout("null", mset) == {}
    # a list with a NON-DICT entry is skipped (the good sibling still parses), no crash.
    assert _parse_remeasure_stdout(
        '["oops", {"name":"parse_ns_per_byte","value":4.95}]', mset
    ) == {"parse_ns_per_byte": 4.95}
    # an entry MISSING the `value` field is skipped -> {} (no-op), no crash.
    assert _parse_remeasure_stdout('[{"name":"parse_ns_per_byte","unit":"ns/byte"}]', mset) == {}
    # an entry with a NON-NUMERIC `value` is skipped -> {} (no-op), no crash.
    assert _parse_remeasure_stdout('[{"name":"parse_ns_per_byte","value":"fast"}]', mset) == {}
    assert _parse_remeasure_stdout('[{"name":"parse_ns_per_byte","value":null}]', mset) == {}
    # genuinely-unparseable text (no JSON, no '[') -> {} (no-op), no crash.
    assert _parse_remeasure_stdout("error: build failed", mset) == {}
    # the harness "wrote ...:" preamble before a good list still parses (fallback to first '[').
    assert _parse_remeasure_stdout(
        'wrote results: [{"name":"parse_ns_per_byte","value":4.95}]', mset
    ) == {"parse_ns_per_byte": 4.95}
    # END-TO-END: a malformed-but-valid re-measure ({}) feeding the gate is a safe no-op on a real
    # regression (can't crash, can't turn a regression into a PASS) — and a GOOD re-measure still
    # improves the metric (parsed straight out of customSmallerIsBetter JSON).
    regsF, _, _ = gate_with_remeasure(
        bad_reading, baseT,
        remeasure_fn=lambda: _parse_remeasure_stdout("{}", {"parse_ns_per_byte"}), k=3)
    assert [r[0] for r in regsF] == ["parse_ns_per_byte"], regsF  # malformed -> no-op, regression stands
    regsG, _, effG = gate_with_remeasure(
        spike, baseT,
        remeasure_fn=lambda: _parse_remeasure_stdout(
            '[{"name":"parse_ns_per_byte","unit":"ns/byte","value":4.95}]', {"parse_ns_per_byte"}), k=4)
    assert regsG == [], regsG                            # good re-measure cleared the band -> PASS
    assert effG["parse_ns_per_byte"] == 4.95, effG       # ...and improved the metric to the true value

    print("perf-gate self-test: ALL ASSERTIONS PASSED")
    return 0


def _split_recent_arg(argv):
    """Pull an optional `--recent metric=a,b,c;metric2=d,e` arg out of argv for --update-baseline."""
    recent = {}
    rest = []
    it = iter(argv)
    for a in it:
        if a == "--recent":
            spec = next(it, "")
            for part in spec.split(";"):
                if "=" in part:
                    m, vals = part.split("=", 1)
                    recent[m.strip()] = [float(x) for x in vals.split(",") if x.strip()]
        else:
            rest.append(a)
    return recent, rest


def do_update(argv):
    """--update-baseline path (push-to-main only). Recompute floors, write file, report changes.
    Exit 0 if the baseline changed (workflow then commits it), 3 if nothing changed."""
    recent, rest = _split_recent_arg(argv)
    if not rest:
        sys.stderr.write("usage: perf-gate.py --update-baseline <current.json> [baseline.json] "
                         "[--recent metric=a,b,c] [--recent-data data.js]\n")
        return 1
    cur_path = rest[0]
    base_path = rest[1] if len(rest) > 1 else DEFAULT_BASELINE
    baseline = load_baseline(base_path)
    # If a benchmark-data data.js was passed via --recent-data, derive the noise metrics' recent series.
    if "--recent-data" in argv:
        djs = argv[argv.index("--recent-data") + 1]
        for name, cfg in baseline.items():
            if cfg["mode"] == "noise":
                vals = load_series_metric(djs, name)
                if vals:
                    recent[name] = vals
    current = load_current(cur_path, set(baseline))
    allow = {m.strip() for m in os.environ.get("PERF_GATE_ALLOW", "").split(",") if m.strip()}
    new, changes = update_baseline(current, baseline, allow=allow, recent=recent)
    if not changes:
        print("perf-gate --update-baseline: no floor changes (nothing to commit).")
        return 3
    print("perf-gate --update-baseline: floor changes:")
    for name, old, newv, why in changes:
        old_s = "none" if old is None else f"{old:g}"
        print(f"  * {name}: {old_s} -> {newv:g}  ({why})")
    write_baseline(base_path, new, preserve_comment_from=base_path)
    print(f"perf-gate --update-baseline: wrote {base_path}")
    return 0


def _split_remeasure_args(argv):
    """[OPUS-4.8] (sq-dzfu) Pull optional `--remeasure-cmd <shell>` / `--remeasure-k N` out of argv so
    the positional <current.json> [baseline.json] parsing is unaffected. Returns (cmd, k, rest)."""
    cmd = None
    k = DEFAULT_REMEASURE_K
    rest = []
    it = iter(argv)
    for a in it:
        if a == "--remeasure-cmd":
            cmd = next(it, None)
        elif a == "--remeasure-k":
            try:
                k = int(next(it, DEFAULT_REMEASURE_K))
            except (TypeError, ValueError):
                k = DEFAULT_REMEASURE_K
        else:
            rest.append(a)
    return cmd, max(0, k), rest


def main(argv):
    if "--self-test" in argv:
        return self_test()
    if "--update-baseline" in argv:
        rest = [a for a in argv[1:] if a != "--update-baseline"]
        return do_update(rest)
    remeasure_cmd, remeasure_k, argv = _split_remeasure_args(argv)
    if len(argv) < 2 or len(argv) > 3:
        sys.stderr.write(
            "usage: perf-gate.py <current-bench-results.json> [perf-baseline.json]\n"
            "                    [--remeasure-cmd '<shell>'] [--remeasure-k N]\n"
            "       perf-gate.py --update-baseline <current.json> [baseline.json] [--recent ...]\n"
            "       perf-gate.py --self-test\n"
        )
        return 1
    cur_path = argv[1]
    base_path = argv[2] if len(argv) == 3 else DEFAULT_BASELINE
    baseline = load_baseline(base_path)
    if not os.path.exists(base_path):
        print(f"perf-gate: baseline '{base_path}' not found — bootstrapping floors from this run.")
    current = load_current(cur_path, set(baseline))
    allow = {m.strip() for m in os.environ.get("PERF_GATE_ALLOW", "").split(",") if m.strip()}
    if allow:
        print(f"perf-gate: PERF_GATE_ALLOW set — these metrics may regress this run (floor re-set): {sorted(allow)}")

    # [OPUS-4.8] (sq-dzfu) Best-of-N re-measure for TIMING metrics: if a timing metric trips the band,
    # re-run the (cheap) re-measure command up to K times and keep the BEST value — runner noise only
    # ever inflates a wall-clock reading, so a one-off spike is squeezed out while a real regression
    # (slow on every reading) still fails. DETERMINISTIC byte metrics are never re-measured.
    remeasure_fn = None
    if remeasure_cmd:
        timing = sorted(n for n, c in baseline.items() if is_timing(c))
        print(f"perf-gate: best-of-{remeasure_k + 1} re-measure ARMED for TIMING metric(s) {timing} "
              f"(cmd: {remeasure_cmd!r})")
        remeasure_fn = make_shell_remeasure(remeasure_cmd, set(baseline))

    regressions, lines, _eff = gate_with_remeasure(
        current, baseline, allow=allow, remeasure_fn=remeasure_fn, k=remeasure_k)
    print("perf-gate: metric comparison against best-ever FLOOR (smaller is better; "
          "DETERMINISTIC byte counts hard-gated, TIMING metrics best-of-N robust):")
    for ln in lines:
        print(ln)

    if regressions:
        has_timing = any(is_timing(baseline.get(r[0], {})) for r in regressions)
        kind = "perf" if has_timing else "deterministic perf"
        print(f"\nperf-gate: HARD FAIL — {kind} regression(s) above the best-ever ratchet floor:")
        for name, floor, cur, delta, thr in regressions:
            klass = "TIMING" if is_timing(baseline.get(name, {})) else "DETERMINISTIC"
            print(f"  * {name} [{klass}]: floor {floor:g} -> {cur:g} ({delta:+.2f}%, threshold +{thr*100:.0f}%)")
        if has_timing:
            print(
                "\nA TIMING metric above is a REAL regression: it stayed over the band across the original\n"
                "reading AND every best-of-N re-measure, so it is NOT runner noise (noise only inflates a\n"
                "single reading; the best of N drowns it out)."
            )
        print(
            "\nThis is gated against the COMMITTED best-ever floor (bench/perf-baseline.json), so a\n"
            "sub-threshold creep every commit can no longer slow-drift past it. If this increase is\n"
            "DELIBERATE: either edit bench/perf-baseline.json to RAISE the floor in a reviewed PR, or\n"
            "re-run with PERF_GATE_ALLOW listing the metric(s) (e.g. PERF_GATE_ALLOW='wasm_bundle_bytes')\n"
            "— the allowed metric then passes AND its floor is re-set to the new value. See header."
        )
        return 2
    print("\nperf-gate: PASS — no perf regression above the best-ever floor "
          "(deterministic metrics strict; timing metrics robust to runner noise).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
