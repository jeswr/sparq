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
#   * Per metric, take the MEDIAN of its last up-to-HISTORY_WINDOW (5) published VALUES — collected
#     PER METRIC across the FULL retained history (the dev/bench/data.js the workflow already
#     fetched as prev-data.js), NOT just the values inside the last 5 commits, so a sparse metric
#     is never judged from a 1-point "median". A metric needs >= MIN_HISTORY (3) values to be
#     gated at all; below that it is listed as "insufficient history — ungated".
#   * DIRECTION: the suite is customSmallerIsBetter, but `*_per_s` metrics
#     (rsp_persistentdict_triples_per_s) store RAW THROUGHPUT, where larger is BETTER — a
#     pre-existing suite inconsistency this gate compensates for by inverting the ratio
#     (median/current) for names matching `_per_s$`. (Candidate follow-up: store the inverse —
#     seconds-per-triple — so the whole suite is genuinely smaller-is-better.)
#   * FAIL-CLOSED NUMERICS: a current value or history-window value that is NaN/non-numeric/
#     negative, or a value that crosses a zero boundary (current 0 vs positive median, or vice
#     versa), HARD-FAILS with a per-metric message — never silently skipped. Current values are
#     validated BEFORE the no-history / insufficient-history early exits (a bad current on a
#     brand-new or young metric still fails the run), and every value inside a metric's median
#     window is checked individually (a negative history point cannot hide behind a positive
#     median). The one deliberate carve-out
#     (grounded in the LIVE history): current == 0 AND median == 0 is a legitimate stable value —
#     four published series are all-zero by design (geo_compliance_deficit, nlq_ask_repairs,
#     rsp_*_w{2,3}_rows, and the sub-resolution sameas_size8_closure_s), so a blanket "<= 0 fails"
#     would permanently red main; stable zeros are instead compared at ratio 1.0 and listed.
#   * DISAPPEARED METRICS: metrics with a gateable history (>= MIN_HISTORY values) that are absent
#     from this run's results are warn-listed loudly; the run HARD-FAILS only if more than
#     DISAPPEAR_FAIL_FRACTION (30%) of the gateable metrics disappeared (suite breakage) — a
#     plain rename or two must not red main.
#   * HARD FAIL (exit 1) if any gated metric is >= HARD_RATIO (2.0x) its median (direction-adjusted).
#   * UNIFORM-SHIFT EXEMPTION (exit 0 + loud warning + full table to stdout AND the job summary):
#     the run is treated as a runner-environment scaling — and the hard zone waived — ONLY if ALL
#     four conditions hold:
#       (a) >= UNIFORM_FRACTION (50%) of compared metrics are >= WATCH_RATIO (1.75x) median;
#       (b) ratio uniformity: over the >= 1.75x metrics, stdev(ratio)/mean(ratio) <= 0.15 — a
#           multiplicative environment scaling moves every wall-clock metric by ~the same factor,
#           while a hot-path code regression does not hit unrelated metric families uniformly;
#       (c) NO metric is >= UNIFORM_CAP_RATIO (4.0x) median — a catastrophic regression must never
#           hide in the herd;
#       (d) every compared DETERMINISTIC metric (byte/count/row/gate-class, classified by unit
#           with an anchored-name fallback — see DETERMINISTIC_UNITS) is within DET_TOLERANCE (1%)
#           of its median — an environment shift moves wall-clock, never deterministic outputs; if
#           any deterministic metric moved, there is NO exemption.
#     HONESTY: the four conditions make a MASKED code regression implausible, not impossible —
#     which is exactly why the exemption still emits a ::warning and the full per-metric table
#     into GITHUB_STEP_SUMMARY instead of passing quietly. The exemption never covers fail-closed
#     numeric errors or the disappeared-metrics breakage check.
#   * TRANSPARENCY: every metric >= WATCH_RATIO (1.75x) median is printed as a table either way,
#     to stdout and (when anything is notable) to GITHUB_STEP_SUMMARY — load-bearing because this
#     gate runs BEFORE the Store step: on a hard-zone failure nothing is published and Store's
#     comparison comment never fires, so this table is the only visible evidence.
#   * SUITE NAME: exact match only. If the suite key is absent from the history, warn loudly and
#     exit 0 — NEVER fall back to a different suite's series.
#   * ABSENT HISTORY (no prev-data.js / empty file / empty series): warn + exit 0 (first-run /
#     bootstrap safe; the fetch step creates an EMPTY prev-data.js when the branch has no data.js
#     yet). A NON-empty but unparsable prev-data.js hard-fails (corrupt published history).
#
# The DETERMINISTIC byte-count metrics stay strictly gated by scripts/perf-gate.py (the committed
# best-ever floor ratchet) — this gate is the noisy-timing hard zone only, and it runs ONLY where
# the Store step auto-pushes (push-to-main), so PR behavior is unchanged.
#
# USAGE
#   scripts/bench_hardzone.py --results bench-results.json --prev-data prev-data.js \
#       --suite 'sparq engine'
#   scripts/bench_hardzone.py --self-test    # hermetic; no network, no git, no repo writes
#                                            # (end-to-end fixtures use a private tempdir)
from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import sys
import tempfile
from pathlib import Path

PROG = "bench-hardzone"

HARD_RATIO = 2.0        # >= 2.0x median => hard fail (unless the uniform-shift exemption fires)
WATCH_RATIO = 1.75      # >= 1.75x median => printed in the watch table; feeds the uniform check
UNIFORM_FRACTION = 0.5  # exemption (a): >= 50% of compared metrics in the watch band
UNIFORM_MAX_CV = 0.15   # exemption (b): stdev/mean over the watch-band ratios must be <= 0.15
UNIFORM_CAP_RATIO = 4.0  # exemption (c): NO metric may be >= 4.0x median
DET_TOLERANCE = 0.01    # exemption (d): deterministic metrics must sit within 1% of median
HISTORY_WINDOW = 5      # median over the last up-to-5 published VALUES per metric
MIN_HISTORY = 3         # a metric needs >= 3 history values to be gated at all
DISAPPEAR_FAIL_FRACTION = 0.30  # > 30% of gateable metrics missing from results => suite breakage

# Deterministic (byte/count/structural) metric classification. UNIT-first because the live metric
# NAMES are trappy: bsbm_query01_count_us / lubm_*_count_us are TIMING metrics whose names contain
# "count", store_bytes_per_triple carries "bytes" mid-name, and rsp_persistentdict_triples_per_s
# carries "triples" but is wall-clock throughput. The unit set below is the exact deterministic
# unit vocabulary observed in the published benchmark-data history (bytes/count/chars/fixtures/
# gates/milli/rows/triples); us / s / ns/byte / ratio / triples_per_s are the noisy families.
# The anchored-name fallback only fires when a row carries no unit at all.
DETERMINISTIC_UNITS = {"bytes", "count", "chars", "fixtures", "gates", "milli", "rows", "triples"}
DETERMINISTIC_NAME_RE = re.compile(r"(_bytes|_count|_chars|_deficit|_gates|_rows|_triples)$")

# Direction: `*_per_s` metrics store raw throughput => LARGER is better (ratio = median/current).
LARGER_IS_BETTER_RE = re.compile(r"_per_s$")


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def _is_num(v) -> bool:
    """A finite real number (bool excluded — json true/false must not pass as 1/0)."""
    return isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v)


def parse_data_js(text: str) -> dict:
    """Parse a github-action-benchmark data.js payload (window.BENCHMARK_DATA = {...};)."""
    m = re.search(r"window\.BENCHMARK_DATA\s*=\s*", text)
    if m:
        text = text[m.end():]
    text = text.strip().rstrip(";").strip()
    return json.loads(text)


def history_values(series: list[dict], window: int = HISTORY_WINDOW) -> dict[str, list]:
    """Per-metric last up-to-`window` published VALUES across the FULL retained series.

    `series` is entries[suite] from data.js: a list of {commit, date, benches:[{name,value,..}]}.
    Entries are ordered by date; every entry contributes, and the window is applied PER METRIC to
    the metric's own value sequence — a metric published in only some entries still collects up to
    `window` values from across the whole retained history (the old per-commit windowing judged
    sparse metrics from as little as ONE point). Values are kept RAW (including non-numeric junk)
    so evaluate() can fail closed on garbage that lands inside a metric's window.
    """
    ordered = sorted(series, key=lambda e: e.get("date", 0))
    vals: dict[str, list] = {}
    for entry in ordered:
        for b in entry.get("benches", []):
            name = b.get("name")
            if name is not None:
                vals.setdefault(name, []).append(b.get("value"))
    return {n: v[-window:] for n, v in vals.items()}


def is_deterministic(name: str, unit: str) -> bool:
    if unit:
        return unit in DETERMINISTIC_UNITS
    return bool(DETERMINISTIC_NAME_RE.search(name))


def evaluate(current: list[dict], hist: dict[str, list]) -> tuple[int, dict]:
    """Pure gate logic — unit-tested by --self-test. No I/O, no env access.

    `current` is this run's bench-results.json rows ({name,value,unit}); `hist` is the per-metric
    history value lists (see history_values). Returns (exit_code, report) where report carries:
      compared      — number of ratio-gated metrics
      rows          — [(name, cur, median, n, ratio, deterministic, larger_better)] all compared
      watch / hard  — the >= WATCH_RATIO / >= HARD_RATIO subsets of rows
      invalid       — [(name, reason)] fail-closed numeric errors (any entry => exit 1)
      ungated       — [(name, n)] metrics with 1..MIN_HISTORY-1 history values (listed, not gated)
      new           — metric names present in results with NO history at all
      stable_zero   — metrics compared as legitimate 0 == 0
      disappeared   — gateable metrics absent from this run's results
      disappeared_frac / uniform_shift / exemption_reasons
    """
    rows: list[tuple] = []
    invalid: list[tuple[str, str]] = []
    ungated: list[tuple[str, int]] = []
    new: list[str] = []
    stable_zero: list[str] = []

    current_by_name: dict[str, dict] = {}
    for row in current:
        name = row.get("name")
        if name is not None:
            current_by_name[name] = row

    gateable = {n for n, pts in hist.items() if len(pts) >= MIN_HISTORY}
    disappeared = sorted(n for n in gateable if n not in current_by_name)
    disappeared_frac = (len(disappeared) / len(gateable)) if gateable else 0.0

    for name in sorted(current_by_name):
        row = current_by_name[name]
        val = row.get("value")
        unit = row.get("unit") or ""
        # FAIL-CLOSED, pre-history: validate the current value BEFORE the no-history /
        # insufficient-history early exits — a NaN/non-numeric/negative current on a brand-new
        # or young metric must still fail the run, not slide out through "new"/"ungated".
        if not _is_num(val):
            invalid.append((name, f"current value {val!r} is not a finite number — fail-closed"))
            continue
        val = float(val)
        if val < 0:
            invalid.append((name, f"negative current measurement ({val:g}) — fail-closed"))
            continue
        pts = hist.get(name)
        if not pts:
            new.append(name)
            continue
        if len(pts) < MIN_HISTORY:
            ungated.append((name, len(pts)))
            continue
        # FAIL-CLOSED, per-window value: every value inside the median window must itself be a
        # finite non-negative number — a negative history point must not hide behind a positive
        # median. Zeros stay legal here: the stable-zero carve-out below needs all-zero windows.
        bad_hist = [p for p in pts if not _is_num(p) or p < 0]
        if bad_hist:
            invalid.append((name, f"history window contains non-finite/negative value(s) "
                                  f"{bad_hist!r} — fail-closed (published history is corrupt "
                                  f"for this metric)"))
            continue
        med = float(statistics.median(pts))
        larger_better = bool(LARGER_IS_BETTER_RE.search(name))
        det = is_deterministic(name, unit)
        if val == 0 and med == 0:
            # Stable zero — legitimate for deterministic zero-counts and the sub-resolution
            # timing series that live history proves are all-zero by design. Compared, not skipped.
            stable_zero.append(name)
            ratio = 1.0
        elif val == 0 or med == 0:
            invalid.append((name, f"zero-boundary change (current {val:g} vs median {med:g}) — "
                                  f"ratio undefined; fail-closed"))
            continue
        else:
            # customSmallerIsBetter suite, EXCEPT `*_per_s` throughput (larger is better): invert.
            ratio = (med / val) if larger_better else (val / med)
        rows.append((name, val, med, len(pts), ratio, det, larger_better))

    compared = len(rows)
    watch = [r for r in rows if r[4] >= WATCH_RATIO]
    hard = [r for r in rows if r[4] >= HARD_RATIO]

    # Uniform-shift exemption — ALL four conditions must hold (see header).
    uniform = False
    exemption_reasons: list[str] = []
    if compared and watch:
        frac = len(watch) / compared
        cond_a = frac >= UNIFORM_FRACTION
        if not cond_a:
            exemption_reasons.append(
                f"(a) breadth: only {len(watch)}/{compared} compared metrics >= "
                f"{WATCH_RATIO}x (< {UNIFORM_FRACTION:.0%})")
        ratios = [r[4] for r in watch]
        if len(ratios) >= 2:
            cv = statistics.stdev(ratios) / statistics.mean(ratios)
            cond_b = cv <= UNIFORM_MAX_CV
            if not cond_b:
                exemption_reasons.append(
                    f"(b) uniformity: stdev/mean over the watch-band ratios is {cv:.3f} "
                    f"(> {UNIFORM_MAX_CV}) — the shift is not a uniform scaling")
        else:
            cond_b = False
            exemption_reasons.append(
                "(b) uniformity: a single shifted metric is not an environment-wide scaling")
        worst = max(r[4] for r in rows)
        cond_c = worst < UNIFORM_CAP_RATIO
        if not cond_c:
            exemption_reasons.append(
                f"(c) cap: worst ratio {worst:.2f}x >= {UNIFORM_CAP_RATIO}x — a catastrophic "
                f"regression never hides in the herd")
        det_moved = [r for r in rows if r[5] and abs(r[4] - 1.0) > DET_TOLERANCE]
        cond_d = not det_moved
        if not cond_d:
            names = ", ".join(f"{r[0]} ({r[4]:.3f}x)" for r in det_moved[:5])
            exemption_reasons.append(
                f"(d) deterministic drift: {len(det_moved)} deterministic metric(s) moved "
                f"> {DET_TOLERANCE:.0%} of median ({names}) — an environment shift moves "
                f"wall-clock, never deterministic outputs")
        uniform = cond_a and cond_b and cond_c and cond_d

    fail = False
    if invalid:
        fail = True
    if disappeared_frac > DISAPPEAR_FAIL_FRACTION:
        fail = True
    if hard and not uniform:
        fail = True

    report = {
        "compared": compared, "rows": rows, "watch": watch, "hard": hard,
        "invalid": invalid, "ungated": ungated, "new": new, "stable_zero": stable_zero,
        "disappeared": disappeared, "disappeared_frac": disappeared_frac,
        "uniform_shift": uniform, "exemption_reasons": exemption_reasons,
    }
    return (1 if fail else 0), report


def _table_lines(entries: list[tuple], markdown: bool) -> list[str]:
    """Render [(name, cur, med, n, ratio, det, larger_better)] rows as a table."""
    out = []
    if markdown:
        out.append("| metric | current | median (n) | ratio | direction |")
        out.append("|---|---:|---:|---:|---|")
        for name, cur, med, n, ratio, _det, lb in sorted(entries, key=lambda t: -t[4]):
            out.append(f"| `{name}` | {cur:.6g} | {med:.6g} (n={n}) | {ratio:.2f}x |"
                       f" {'larger-is-better' if lb else 'smaller-is-better'} |")
    else:
        out.append(f"  {'metric':<60} {'current':>14} {'median(n)':>18} {'ratio':>7}")
        for name, cur, med, n, ratio, _det, lb in sorted(entries, key=lambda t: -t[4]):
            suffix = "  [larger-is-better]" if lb else ""
            out.append(f"  {name:<60} {cur:>14.4g} {med:>13.4g}(n={n}) {ratio:>6.2f}x{suffix}")
    return out


def render_report(report: dict, markdown: bool) -> list[str]:
    """Human-readable report lines (plain for stdout, markdown for GITHUB_STEP_SUMMARY)."""
    h2 = "## " if markdown else ""
    lines: list[str] = [f"{h2}bench hard zone — median-of-history gate"]
    lines.append(f"{report['compared']} metrics compared; {len(report['watch'])} >= "
                 f"{WATCH_RATIO}x median; {len(report['hard'])} >= {HARD_RATIO}x median.")
    if report["watch"]:
        lines.append("")
        lines.append(f"**Watch band (>= {WATCH_RATIO}x median):**" if markdown
                     else f"metrics >= {WATCH_RATIO}x their median of the last up-to-"
                          f"{HISTORY_WINDOW} published values:")
        lines.extend(_table_lines(report["watch"], markdown))
    if report["invalid"]:
        lines.append("")
        lines.append("**Fail-closed numeric errors (each one fails the run):**" if markdown
                     else "fail-closed numeric errors (each one fails the run):")
        lines.extend(f"  - `{n}`: {reason}" if markdown else f"  {n}: {reason}"
                     for n, reason in report["invalid"])
    if report["disappeared"]:
        lines.append("")
        frac = report["disappeared_frac"]
        verdict = ("FAIL — suite breakage" if frac > DISAPPEAR_FAIL_FRACTION
                   else "warn only (below the "
                        f"{DISAPPEAR_FAIL_FRACTION:.0%} suite-breakage threshold)")
        lines.append(f"**Disappeared metrics** ({len(report['disappeared'])}, "
                     f"{frac:.0%} of gateable — {verdict}):" if markdown else
                     f"DISAPPEARED metrics ({len(report['disappeared'])}, {frac:.0%} of "
                     f"gateable — {verdict}):")
        lines.extend(f"  - `{n}`" if markdown else f"  {n}" for n in report["disappeared"])
    if report["ungated"]:
        lines.append("")
        names = ", ".join(f"{n} (n={c})" for n, c in report["ungated"])
        lines.append(f"insufficient history — ungated (need >= {MIN_HISTORY} values): {names}")
    if report["new"]:
        shown = ", ".join(report["new"][:15])
        more = f" (+{len(report['new']) - 15} more)" if len(report["new"]) > 15 else ""
        lines.append(f"new metrics with no history yet (ungated): {shown}{more}")
    if report["stable_zero"]:
        lines.append(f"stable-zero metrics compared at ratio 1.0: "
                     f"{', '.join(report['stable_zero'])}")
    if report["uniform_shift"]:
        lines.append("")
        lines.append("UNIFORM-SHIFT EXEMPTION applied: all four conditions hold (breadth, ratio "
                     "uniformity, no >= 4x outlier, deterministic metrics unmoved). This makes a "
                     "masked code regression implausible — NOT impossible — hence this loud "
                     "record instead of a quiet pass.")
    elif report["exemption_reasons"]:
        lines.append("")
        lines.append("uniform-shift exemption NOT applied:")
        lines.extend(f"  - {r}" for r in report["exemption_reasons"])
    return lines


def emit_report(report: dict) -> None:
    """Print the report to stdout, and to GITHUB_STEP_SUMMARY when anything is notable.

    Load-bearing: this gate runs BEFORE the Store step, so on a hard-zone failure the run
    publishes nothing and Store's comparison comment never fires — this output is the only
    visible evidence of what tripped.
    """
    for line in render_report(report, markdown=False):
        print(line)
    notable = (report["watch"] or report["hard"] or report["invalid"]
               or report["disappeared"] or report["uniform_shift"])
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if notable and summary_path:
        with open(summary_path, "a", encoding="utf-8") as f:
            f.write("\n".join(render_report(report, markdown=True)) + "\n")
    if report["uniform_shift"]:
        print(f"::warning title=bench hard zone — uniform environment shift, not gating::"
              f"{len(report['watch'])} of {report['compared']} compared metrics are >= "
              f"{WATCH_RATIO}x their median-of-history simultaneously, the shifted ratios are "
              f"uniform (stdev/mean <= {UNIFORM_MAX_CV}), no metric reached "
              f"{UNIFORM_CAP_RATIO}x, and every deterministic metric is within "
              f"{DET_TOLERANCE:.0%} of its median. Treated as a runner-environment scaling and "
              f"NOT failing the run — a masked code regression is implausible under these four "
              f"conditions, not impossible, so review the table in the job summary.")
    for name, reason in report["invalid"]:
        print(f"::error title=bench hard zone — invalid measurement::{name}: {reason}")
    if report["disappeared_frac"] > DISAPPEAR_FAIL_FRACTION:
        print(f"::error title=bench hard zone — suite breakage::"
              f"{len(report['disappeared'])} gateable metrics "
              f"({report['disappeared_frac']:.0%}) are missing from this run's results — more "
              f"than {DISAPPEAR_FAIL_FRACTION:.0%} of the gated suite disappeared at once, "
              f"which is a bench-suite breakage, not a rename. See the disappeared list above.")
    if report["hard"] and not report["uniform_shift"]:
        for name, cur, med, n, ratio, _det, lb in report["hard"]:
            direction = "median/current (larger-is-better)" if lb else "current/median"
            print(f"::error title=bench hard zone::{name} is {ratio:.2f}x its "
                  f"median-of-{n} history ({direction}: {cur:.6g} vs median {med:.6g}) — "
                  f"at/above the {HARD_RATIO}x hard threshold, and the uniform-shift exemption "
                  f"does not apply. Treating as a real regression.")


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog=PROG, description=__doc__)
    ap.add_argument("--results", required=True, help="this run's bench-results.json")
    ap.add_argument("--prev-data", required=True,
                    help="previously-published data.js (prev-data.js; absent => warn + pass)")
    ap.add_argument("--suite", default="sparq engine",
                    help="data.js entries suite name (EXACT match; absent => warn + pass)")
    return ap


def run_gate(results_path: str, prev_data_path: str, suite: str) -> int:
    rp = Path(results_path)
    if not rp.exists():
        print(f"::error title=bench hard zone::results file {results_path} is absent — the "
              f"benchmark step should have written it on this path; fail-closed.")
        return 1
    try:
        current = json.loads(rp.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, ValueError) as e:
        print(f"::error title=bench hard zone::could not parse {results_path} ({e}) — "
              f"fail-closed (this run's own artifact is corrupt).")
        return 1
    if not isinstance(current, list):
        current = current.get("benches", [])
    pp = Path(prev_data_path)
    if not pp.exists():
        print(f"::warning title=bench hard zone — no history::{prev_data_path} is absent "
              f"(benchmark-data branch missing or empty — first run). Nothing to compare; passing.")
        return 0
    text = pp.read_text(encoding="utf-8")
    if not text.strip():
        # The fetch step creates an EMPTY prev-data.js when the branch has no data.js yet
        # (`git show ... > prev-data.js` truncates before git show fails) — bootstrap, not corrupt.
        print(f"::warning title=bench hard zone — no history::{prev_data_path} is empty "
              f"(benchmark-data branch has no data.js yet — bootstrap). Nothing to compare; "
              f"passing.")
        return 0
    try:
        data = parse_data_js(text)
    except (json.JSONDecodeError, ValueError) as e:
        print(f"::error title=bench hard zone::could not parse non-empty {prev_data_path} "
              f"({e}) — fail-closed (the published history is corrupt; investigate the "
              f"benchmark-data branch).")
        return 1
    entries = data.get("entries", {})
    # EXACT suite match only — never fall back to a different suite's series.
    series = entries.get(suite)
    if series is None:
        print(f"::warning title=bench hard zone — suite not found::suite '{suite}' is not in "
              f"{prev_data_path} (available: {sorted(entries)}). Refusing to compare against a "
              f"different suite's series; nothing to compare; passing.")
        return 0
    if not series:
        print(f"::warning title=bench hard zone — empty history::no published points for suite "
              f"'{suite}' in {prev_data_path}; nothing to compare; passing.")
        return 0
    code, report = evaluate(current, history_values(series))
    emit_report(report)
    if code == 0 and not report["hard"] and not report["uniform_shift"]:
        log(f"hard zone clean: {report['compared']} metrics compared against their "
            f"median-of-history; {len(report['watch'])} in the watch band, none failing.")
    return code


# ── self-test (hermetic: inline fixtures; no network, no git, no repo writes) ────────────────────
def _series(points: list[list[tuple]]) -> list[dict]:
    """Build a data.js-shaped series from [[(name, value), ...] per commit] (oldest first)."""
    return [
        {"date": i, "benches": [{"name": n, "value": v, "unit": "us"} for n, v in benches]}
        for i, benches in enumerate(points)
    ]


def _cur(rows: list[tuple], unit: str = "us") -> list[dict]:
    return [{"name": n, "value": v, "unit": unit} for n, v in rows]


def self_test() -> int:
    failures: list[str] = []
    checks = 0

    def check(cond: bool, what: str) -> None:
        nonlocal checks
        checks += 1
        if cond:
            log(f"self-test ok: {what}")
        else:
            failures.append(what)
            log(f"self-test FAIL: {what}")

    # Baseline 5-commit history: four metrics stable at 10.0.
    series = _series([[("a", 10.0), ("b", 10.0), ("c", 10.0), ("d", 10.0)]] * 5)

    # 1. Median semantics: window trims to the last 5 VALUES per metric.
    hist = history_values(_series([[("m", 100.0)], [("m", 1.0)], [("m", 2.0)], [("m", 3.0)],
                                   [("m", 4.0)], [("m", 5.0)]]))
    check(hist["m"] == [1.0, 2.0, 3.0, 4.0, 5.0], "history window keeps only the last 5 values")
    check(statistics.median(hist["m"]) == 3.0, "median of 5-value history is the middle value")
    hist_even = history_values(_series([[("m", 1.0)], [("m", 2.0)], [("m", 3.0)], [("m", 8.0)]]))
    check(statistics.median(hist_even["m"]) == 2.5, "median of an even count averages the middle two")
    code, rep = evaluate(_cur([("m", 15.0)]), history_values(
        _series([[("m", 10.0)], [("m", 10.0)], [("m", 10.0)], [("m", 10.0)], [("m", 30.0)]])))
    check(code == 0 and not rep["hard"],
          "single outlier history point does not move the median (1.5x vs median passes)")

    # 2. SPARSE-metric per-metric windowing: a metric present in only 5 of 10 commits still
    #    collects all 5 of its values (the old per-commit window judged it from 1 point).
    sparse_pts: list[list[tuple]] = []
    for i in range(10):
        commit: list[tuple] = [("dense", 10.0)]
        if i % 2 == 0:
            commit.append(("sparse", 10.0 + i))  # values 10,12,14,16,18 at commits 0,2,4,6,8
        sparse_pts.append(commit)
    hist_sparse = history_values(_series(sparse_pts))
    check(hist_sparse["sparse"] == [10.0, 12.0, 14.0, 16.0, 18.0],
          "sparse metric collects its last 5 values across the FULL retained history")
    code, rep = evaluate(_cur([("sparse", 15.0), ("dense", 10.0)]), hist_sparse)
    check(code == 0 and rep["compared"] == 2,
          "sparse metric is gated against its own 5-value median (14.0), not a 1-point window")

    # 3. Insufficient history (< 3 values) => listed as ungated, not silently gated or failed.
    two_pt = history_values(_series([[("young", 10.0)], [("young", 10.0)]]))
    code, rep = evaluate(_cur([("young", 100.0)]), two_pt)
    check(code == 0 and rep["compared"] == 0 and rep["ungated"] == [("young", 2)],
          "a metric with only 2 history values is ungated + listed (even at 10x)")

    # 4. Absent history: no overlapping metrics => pass (exit 0), nothing compared.
    code, rep = evaluate(_cur([("new_metric", 42.0)]), {})
    check(code == 0 and rep["compared"] == 0 and rep["new"] == ["new_metric"],
          "absent history passes with nothing compared; metric listed as new")

    # 5. Fail-closed numerics: NaN / non-numeric / negative current, NaN in the history
    #    window, and zero-boundary changes all hard-fail with a per-metric record.
    code, rep = evaluate(_cur([("a", float("nan"))]), history_values(series))
    check(code == 1 and len(rep["invalid"]) == 1, "NaN current value fails closed")
    code, rep = evaluate([{"name": "a", "value": "fast", "unit": "us"}], history_values(series))
    check(code == 1 and len(rep["invalid"]) == 1, "non-numeric current value fails closed")
    code, rep = evaluate(_cur([("a", -1.0)]), history_values(series))
    check(code == 1 and len(rep["invalid"]) == 1, "negative current value fails closed")
    bad_hist = history_values(_series([[("a", 10.0)], [("a", 10.0)],
                                       [("a", float("nan"))], [("a", 10.0)]]))
    code, rep = evaluate(_cur([("a", 10.0)]), bad_hist)
    check(code == 1 and len(rep["invalid"]) == 1, "NaN inside the history window fails closed")
    #    Pre-history validation: a bad CURRENT value must fail even when the metric has no or
    #    young history — never slide out through the "new"/"ungated" early exits.
    code, rep = evaluate(_cur([("brandnew", float("nan"))]), {})
    check(code == 1 and len(rep["invalid"]) == 1 and not rep["new"],
          "NaN current on a NO-history metric fails closed (not silently listed as new)")
    young_hist = history_values(_series([[("young", 10.0)], [("young", 10.0)]]))
    code, rep = evaluate([{"name": "young", "value": "junk", "unit": "us"}], young_hist)
    check(code == 1 and len(rep["invalid"]) == 1 and not rep["ungated"],
          "non-numeric current on a young-history metric fails closed (not silently ungated)")
    code, rep = evaluate(_cur([("fresh", -3.0)]), {})
    check(code == 1 and len(rep["invalid"]) == 1 and not rep["new"],
          "negative current on a no-history metric fails closed")
    #    Per-window negativity: a negative history point must not hide behind a positive median.
    neg_hist = history_values(_series([[("a", -1.0)], [("a", 10.0)], [("a", 10.0)]]))
    code, rep = evaluate(_cur([("a", 10.0)]), neg_hist)
    check(code == 1 and len(rep["invalid"]) == 1,
          "negative history value inside the window fails closed despite a positive median")
    code, rep = evaluate(_cur([("a", 0.0)]), history_values(series))
    check(code == 1 and len(rep["invalid"]) == 1,
          "current 0 vs positive median is a zero-boundary change — fails closed")
    zero_series = history_values(_series([[("z", 0.0)]] * 5))
    code, rep = evaluate(_cur([("z", 5.0)]), zero_series)
    check(code == 1 and len(rep["invalid"]) == 1,
          "positive current vs zero median is a zero-boundary change — fails closed")
    code, rep = evaluate(_cur([("z", 0.0)]), zero_series)
    check(code == 0 and rep["stable_zero"] == ["z"] and rep["compared"] == 1,
          "stable zero (current 0, median 0) is legitimate — compared at ratio 1.0")

    # 6. Disappeared metrics: warn below the 30% threshold, hard-fail above it.
    code, rep = evaluate(_cur([("a", 10.0), ("b", 10.0), ("c", 10.0)]), history_values(series))
    check(code == 0 and rep["disappeared"] == ["d"] and rep["disappeared_frac"] == 0.25,
          "1 of 4 gateable metrics disappeared (25%) => loud warn-list, no fail")
    code, rep = evaluate(_cur([("a", 10.0), ("b", 10.0)]), history_values(series))
    check(code == 1 and rep["disappeared"] == ["c", "d"],
          "2 of 4 gateable metrics disappeared (50% > 30%) => suite breakage, hard fail")

    # 7. `_per_s` direction inversion (throughput: larger is better).
    thr = history_values(_series([[("rsp_x_triples_per_s", 1000.0)]] * 5))
    code, rep = evaluate(_cur([("rsp_x_triples_per_s", 500.0)], unit="triples_per_s"), thr)
    check(code == 1 and len(rep["hard"]) == 1 and abs(rep["hard"][0][4] - 2.0) < 1e-9,
          "_per_s throughput halving is a 2.0x regression (ratio inverted) — hard fail")
    code, rep = evaluate(_cur([("rsp_x_triples_per_s", 2000.0)], unit="triples_per_s"), thr)
    check(code == 0 and not rep["watch"],
          "_per_s throughput doubling is an improvement (ratio 0.5) — passes")

    # 8. Single-metric hard fail: one metric >= 2.0x median, the rest normal => exit 1.
    lone = _cur([("a", 21.0), ("b", 10.2), ("c", 9.9), ("d", 10.0)])
    code, rep = evaluate(lone, history_values(series))
    check(code == 1 and not rep["uniform_shift"] and len(rep["hard"]) == 1
          and rep["hard"][0][0] == "a",
          "single sustained >= 2.0x regression hard-fails (no uniform exemption)")
    watch_only = _cur([("a", 18.0), ("b", 10.0), ("c", 10.0), ("d", 10.0)])
    code, rep = evaluate(watch_only, history_values(series))
    check(code == 0 and len(rep["watch"]) == 1 and not rep["hard"],
          "a lone watch-band metric (1.8x, < 2.0x) is reported but does not fail")

    # 9. Uniform-shift exemption POSITIVE: broad (3/4 >= 1.75x), uniform ratios (cv <= 0.15),
    #    nothing >= 4x, no deterministic metric moved => exempt, exit 0.
    shifted = _cur([("a", 19.0), ("b", 19.5), ("c", 20.5), ("d", 11.0)])
    code, rep = evaluate(shifted, history_values(series))
    check(code == 0 and rep["uniform_shift"] and len(rep["watch"]) == 3 and len(rep["hard"]) == 1,
          "uniform environment shift (3/4 >= 1.75x, uniform, capped, no det drift) is exempted")

    # 10. Uniformity rejection: broad shift with NON-uniform ratios => NOT exempt => fail.
    ragged = _cur([("a", 18.0), ("b", 25.0), ("c", 35.0), ("d", 10.0)])  # ratios 1.8/2.5/3.5
    code, rep = evaluate(ragged, history_values(series))
    check(code == 1 and not rep["uniform_shift"]
          and any("(b) uniformity" in r for r in rep["exemption_reasons"]),
          "broad but NON-uniform shift (cv > 0.15) is NOT exempt — hard fail stands")

    # 11. The 4x cap ISOLATED: conditions (a) breadth, (b) uniformity and (d) deterministic all
    #     PASS — ratios 4.05/4.1/4.08/4.06 put 4/4 metrics in the watch band with CV ~0.005 and
    #     no deterministic metrics — so ONLY the cap (c) blocks the exemption. Removing the cap
    #     from the predicate would flip exactly this check red.
    capped = _cur([("a", 40.5), ("b", 41.0), ("c", 40.8), ("d", 40.6)])
    code, rep = evaluate(capped, history_values(series))
    check(code == 1 and not rep["uniform_shift"] and len(rep["hard"]) == 4
          and len(rep["exemption_reasons"]) == 1
          and "(c) cap" in rep["exemption_reasons"][0],
          "a >= 4x metric is the SOLE exemption blocker ((a)/(b)/(d) pass) — hard fail stands")
    #     Control: the SAME uniform shape kept below the cap (all ~1.9x) IS exempt — proving the
    #     cap is the discriminating condition, not breadth/uniformity/deterministic drift.
    under_cap = _cur([("a", 19.0), ("b", 19.5), ("c", 19.2), ("d", 19.1)])
    code, rep = evaluate(under_cap, history_values(series))
    check(code == 0 and rep["uniform_shift"] and not rep["exemption_reasons"],
          "same uniform shape below the cap IS exempt — the 4x cap is the discriminating condition")

    # 12. Deterministic-metric drift blocks the exemption: same broad uniform timing shift, but a
    #     byte-count metric moved > 1% => NOT exempt => fail.
    det_series = [{"date": i, "benches": [
        {"name": "a", "value": 10.0, "unit": "us"},
        {"name": "b", "value": 10.0, "unit": "us"},
        {"name": "c", "value": 10.0, "unit": "us"},
        {"name": "wasm_bundle_bytes", "value": 1000.0, "unit": "bytes"},
    ]} for i in range(5)]
    det_cur = [{"name": "a", "value": 19.0, "unit": "us"},
               {"name": "b", "value": 19.5, "unit": "us"},
               {"name": "c", "value": 20.5, "unit": "us"},
               {"name": "wasm_bundle_bytes", "value": 1030.0, "unit": "bytes"}]  # +3%
    code, rep = evaluate(det_cur, history_values(det_series))
    check(code == 1 and not rep["uniform_shift"]
          and any("(d) deterministic" in r for r in rep["exemption_reasons"]),
          "a moved deterministic metric (bytes +3%) blocks the exemption — hard fail stands")
    det_cur_ok = [dict(r) for r in det_cur]
    det_cur_ok[3]["value"] = 1000.0  # deterministic metric unmoved => exemption restored
    code, rep = evaluate(det_cur_ok, history_values(det_series))
    check(code == 0 and rep["uniform_shift"],
          "same shift with the deterministic metric unmoved IS exempt (control case)")

    # 13. Deterministic classification is unit-first (name traps are real: *_count_us is timing).
    check(is_deterministic("wasm_bundle_bytes", "bytes")
          and not is_deterministic("bsbm_query01_count_us", "us")
          and not is_deterministic("rsp_persistentdict_triples_per_s", "triples_per_s")
          and is_deterministic("zk_compose_filter_f64_gates", ""),
          "deterministic classification: unit-first, anchored-name fallback only when unit absent")

    # 14. Argument handling: the parser accepts the documented flags on a fixture path.
    ns = build_parser().parse_args(
        ["--results", "fixtures/bench-results.json", "--prev-data", "fixtures/prev-data.js",
         "--suite", "sparq engine"])
    check(ns.results == "fixtures/bench-results.json"
          and ns.prev_data == "fixtures/prev-data.js" and ns.suite == "sparq engine",
          "argument parser maps --results/--prev-data/--suite onto the expected namespace")

    # 15. End-to-end run_gate over real files in a private tempdir: suite-name mismatch must
    #     warn + exit 0 (never fall back to another suite), exact match must gate.
    with tempfile.TemporaryDirectory(prefix="bench-hardzone-selftest-") as td:
        results = Path(td) / "bench-results.json"
        results.write_text(json.dumps(_cur([("a", 21.0), ("b", 10.0), ("c", 10.0),
                                            ("d", 10.0)])), encoding="utf-8")
        prev = Path(td) / "prev-data.js"
        payload = {"entries": {"sparq engine": _series(
            [[("a", 10.0), ("b", 10.0), ("c", 10.0), ("d", 10.0)]] * 5)}}
        prev.write_text("window.BENCHMARK_DATA = " + json.dumps(payload) + ";",
                        encoding="utf-8")
        check(run_gate(str(results), str(prev), "some other suite") == 0,
              "suite-name mismatch warns + exits 0 (no cross-suite fallback)")
        check(run_gate(str(results), str(prev), "sparq engine") == 1,
              "exact suite match gates for real (2.1x lone regression fails end-to-end)")
        empty = Path(td) / "empty-prev.js"
        empty.write_text("", encoding="utf-8")
        check(run_gate(str(results), str(empty), "sparq engine") == 0,
              "empty prev-data.js (bootstrap fetch artifact) warns + exits 0")
        corrupt = Path(td) / "corrupt-prev.js"
        corrupt.write_text("window.BENCHMARK_DATA = {not json", encoding="utf-8")
        check(run_gate(str(results), str(corrupt), "sparq engine") == 1,
              "non-empty unparsable prev-data.js fails closed (corrupt published history)")

    if failures:
        log(f"self-test FAILED ({len(failures)}/{checks} checks): " + "; ".join(failures))
        return 1
    log(f"self-test OK ({checks} checks)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    args = build_parser().parse_args(argv)
    return run_gate(args.results, args.prev_data, args.suite)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
