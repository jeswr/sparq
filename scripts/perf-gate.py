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
#                                             series); wide NOISE band so jitter never trips it.
# The 2% byte metrics are integer-exact at the current scale, so 2% is "integer-exact plus a hair".
#
# auto  = the floor AUTO-RATCHETS DOWN on a genuine improvement (current < floor) and NEVER auto-raises.
# noise = the floor only ratchets DOWN on a SUSTAINED improvement (a supplied median/percentile of
#         recent points is below the floor), so a single noise-low can never tighten it.
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
#   perf-gate.py --update-baseline <current-bench-results.json> [baseline.json] [--recent a,b,c]
#                                   # recompute + rewrite floors; exit 0 if changed, 3 if unchanged
#   perf-gate.py --self-test        # unit-test the ratchet logic (no files needed)
# EXIT CODES: 0 = pass / baseline changed, 2 = regression (hard fail), 1 = usage / parse error,
#             3 = --update-baseline made no change (nothing to commit).
#
# This ratchet is intentionally STRICT-on-regression / SILENT-on-improvement and only ever reads the
# five deterministic metrics, so it is safe to run on noisy GitHub-hosted runners.

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


def main(argv):
    if "--self-test" in argv:
        return self_test()
    if "--update-baseline" in argv:
        rest = [a for a in argv[1:] if a != "--update-baseline"]
        return do_update(rest)
    if len(argv) < 2 or len(argv) > 3:
        sys.stderr.write(
            "usage: perf-gate.py <current-bench-results.json> [perf-baseline.json]\n"
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

    regressions, lines = evaluate(current, baseline, allow=allow)
    print("perf-gate: deterministic-metric comparison against best-ever FLOOR (smaller is better):")
    for ln in lines:
        print(ln)

    if regressions:
        print("\nperf-gate: HARD FAIL — deterministic perf regression(s) above the best-ever ratchet floor:")
        for name, floor, cur, delta, thr in regressions:
            print(f"  * {name}: floor {floor:g} -> {cur:g} ({delta:+.2f}%, threshold +{thr*100:.0f}%)")
        print(
            "\nThis is gated against the COMMITTED best-ever floor (bench/perf-baseline.json), so a\n"
            "sub-threshold creep every commit can no longer slow-drift past it. If this increase is\n"
            "DELIBERATE: either edit bench/perf-baseline.json to RAISE the floor in a reviewed PR, or\n"
            "re-run with PERF_GATE_ALLOW listing the metric(s) (e.g. PERF_GATE_ALLOW='wasm_bundle_bytes')\n"
            "— the allowed metric then passes AND its floor is re-set to the new value. See header."
        )
        return 2
    print("\nperf-gate: PASS — no deterministic perf regression above the best-ever floor.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
