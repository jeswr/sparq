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
#   * NOISE FLOOR (UNIT-AWARE): only a metric whose UNIT is in the FLOOR_EXEMPT_UNITS
#     allow-list ("us" micros, "milli" — each with a measured per-unit floor) AND whose
#     median-of-window is below that unit's floor can never hard-fail the gate ALONE — at that
#     scale the measured honest run-to-run bands EXCEED the 2.0x hard threshold (empirical
#     bands at the FLOOR_EXEMPT_UNITS constant below). EVERY other unit — "s" seconds,
#     "ns_per_byte", "ratio", `*_per_s` throughputs, deterministic units, and anything
#     UNKNOWN — is NEVER floor-exempt regardless of magnitude (a stable 10x on a 0.4 s load
#     IS a real regression; unknown units fail toward gated). Floor-exempt is NOT ungated: the
#     metric still prints in the watch table flagged "under noise floor — watch only", still
#     emits a ::warning at >= HARD_RATIO, the Store step's soft comparison comment still
#     fires, and every floor-exempt hit at >= HARD_RATIO is written to the --report-out JSON,
#     which the Soft-zone triage step (scripts/bench-triage.py --hardzone-report) merges into
#     the rolling deduped `bench-flake` issue — the DURABLE trail: a persistent sub-floor
#     regression rebases the median after a few accepted points, so the issue trail is the
#     record that survives. Floor-exempt metrics are also EXCLUDED from every uniform-shift
#     computation (breadth/uniformity/cap) — their degenerate small-integer ratios would
#     distort all three. DETERMINISTIC metrics (bytes/count/gates/rows/...) are NEVER
#     floor-exempt — a 2x move on a deterministic output is real at any scale.
#   * UNIFORM-SHIFT EXEMPTION (exit 0 + loud warning + full table to stdout AND the job summary):
#     the run is treated as a runner-environment scaling — and the hard zone waived — ONLY if ALL
#     four conditions hold (computed over the GATED, i.e. non-floor-exempt, metrics):
#       (a) >= UNIFORM_FRACTION (50%) of gated metrics are >= WATCH_RATIO (1.75x) median;
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
import contextlib
import io
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

# NOISE FLOOR — UNIT-AWARE ALLOW-LIST of (unit, floor) pairs. A metric is floor-exempt
# (watch-only, never a hard fail alone, excluded from the uniform-shift computations) ONLY if
# its unit appears here AND its median-of-window is below that unit's floor. Floors are
# justified PER-UNIT by the measured honest run-to-run bands in the published benchmark-data
# history (2026-07-23 replay):
#   * "us" (micros, floor 20.0): µs-scale timing metrics have HONEST noise bands exceeding the
#     2.0x hard threshold. Within the honest pre-red-era oldest-10 points alone, max/min bands:
#     sameas_size32_query_us 2.55x (values ~4-9µs), deeptax_d1000_query_us 3.25x (~4-13µs),
#     lubm_q14_count_us 2.45x. Live replays: the honest head fails sameas_size32_query_us at
#     2.14x (9 vs median 4.2); another honest entry fails deeptax_d1000_query_us at 2.00x
#     (8 vs 4). Every observed false-fail metric has a median <~15 units; 20.0 covers them
#     with margin while keeping every >= 20µs timing metric fully hard-gated.
#   * "milli" (floor 20.0): the empirically-degenerate HNSW recall family —
#     vectors_hnsw_recall_at10 honestly oscillates 1<->4 (4.0x band) at integer-milli
#     resolution; its stable siblings (vectors_diskann_recall_at10 ~34,
#     vectors_pq_recall_at10 ~22) sit ABOVE the floor and stay hard-gated.
# EVERY unit NOT in this allow-list is NEVER floor-exempt regardless of magnitude. That
# includes "s" seconds (nine published series — load_s, text_build_s, rdfs_infer_s, ... at
# ~0.4-15 s — where a stable 10x IS a real regression; a raw magnitude-only floor wrongly
# exempted all of them), "ns_per_byte", "ratio", the `*_per_s` throughputs, every
# deterministic unit, and any UNKNOWN/absent unit: unknown units FAIL TOWARD GATED.
# DETERMINISTIC metrics are never floor-exempt even in an allow-listed unit.
FLOOR_EXEMPT_UNITS: dict[str, float] = {"us": 20.0, "milli": 20.0}

# Deterministic (byte/count/structural) metric classification. UNIT-first because the live metric
# NAMES are trappy: bsbm_query01_count_us / lubm_*_count_us are TIMING metrics whose names contain
# "count", store_bytes_per_triple carries "bytes" mid-name, and rsp_persistentdict_triples_per_s
# carries "triples" but is wall-clock throughput. The unit set below is the exact deterministic
# unit vocabulary observed in the published benchmark-data history (bytes/count/chars/fixtures/
# gates/rows/triples); us / s / ns/byte / ratio / triples_per_s / milli are the noisy families.
# "milli" is NOISY, not deterministic: it carries the three vectors_*_recall_at10 series, and the
# live history shows vectors_hnsw_recall_at10 honestly oscillating 1<->4 (randomized HNSW build
# at degenerate integer-milli resolution) — a measurement, not a deterministic output. Its stable
# siblings (diskann ~34, pq ~22) sit ABOVE the milli floor (FLOOR_EXEMPT_UNITS) and stay
# hard-gated as noisy metrics.
# The anchored-name fallback only fires when a row carries no unit at all.
DETERMINISTIC_UNITS = {"bytes", "count", "chars", "fixtures", "gates", "rows", "triples"}
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


def _floor_desc() -> str:
    """Compact rendering of the unit-aware floor allow-list for report/annotation text."""
    return ", ".join(f"{u} < {f:g}" for u, f in sorted(FLOOR_EXEMPT_UNITS.items()))


def evaluate(current: list[dict], hist: dict[str, list]) -> tuple[int, dict]:
    """Pure gate logic — unit-tested by --self-test. No I/O, no env access.

    `current` is this run's bench-results.json rows ({name,value,unit}); `hist` is the per-metric
    history value lists (see history_values). Returns (exit_code, report) where report carries:
      compared      — number of ratio-compared metrics (gated + floor-exempt)
      rows          — [(name, cur, median, n, ratio, deterministic, larger_better, floor_exempt)]
      watch         — the >= WATCH_RATIO subset of rows (floor-exempt rows included, flagged)
      hard          — the >= HARD_RATIO subset of GATED rows (floor-exempt can never hard-fail)
      floor_exempt  — rows whose UNIT is in FLOOR_EXEMPT_UNITS with median under that unit's
                      floor (watch-only; unit-aware allow-list, unknown units stay gated)
      floor_exempt_hard — floor-exempt rows at >= HARD_RATIO, in the soft-triage comparison
                      shape ({name,current,previous,ratio_pct,unit,note}; previous = the
                      MEDIAN-of-history) — the durable-trail payload written by --report-out
                      and merged into the deduped bench-flake issue by bench-triage.py
      gated_compared / gated_watch — counts over the GATED (non-floor-exempt) rows, backing
                      the uniform-shift annotation (gated numerator/denominator)
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
    units: dict[str, str] = {}

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
        # NOISE FLOOR — UNIT-AWARE: exempt ONLY allow-listed units ("us"/"milli") whose median
        # sits under that unit's measured floor (see FLOOR_EXEMPT_UNITS). Every other unit —
        # "s" seconds, "ns_per_byte", "ratio", `*_per_s`, unknown — is gated regardless of
        # magnitude (unknown units fail toward gated), and deterministic metrics are NEVER
        # exempt (belt-and-braces: no deterministic unit is allow-listed).
        unit_floor = FLOOR_EXEMPT_UNITS.get(unit)
        floor_exempt = (not det) and unit_floor is not None and med < unit_floor
        rows.append((name, val, med, len(pts), ratio, det, larger_better, floor_exempt))
        units[name] = unit

    compared = len(rows)
    # Floor-exempt rows stay in the watch TABLE (flagged "under noise floor — watch only") but
    # are excluded from the hard set AND from every uniform-shift computation below — their
    # degenerate small-integer ratios would distort breadth (a), uniformity (b) and the cap (c)
    # alike (the honest sub-floor history reaches 4.0x on pure noise).
    gated_rows = [r for r in rows if not r[7]]
    floor_exempt_rows = [r for r in rows if r[7]]
    watch = [r for r in rows if r[4] >= WATCH_RATIO]
    hard = [r for r in gated_rows if r[4] >= HARD_RATIO]
    # DURABLE TRAIL — floor-exempt rows at/above HARD_RATIO never red the gate, and after a few
    # accepted points the regressed median REBASES, erasing the gate-side evidence. Emit them in
    # the soft-triage comparison-row shape (`previous` = the MEDIAN-of-history, tagged in `note`)
    # so bench-triage.py --mode soft --hardzone-report merges them into the rolling deduped
    # bench-flake issue (see --report-out).
    floor_exempt_hard = [
        {"name": r[0], "current": r[1], "previous": r[2],
         "ratio_pct": round(r[4] * 100.0, 1), "unit": units.get(r[0], ""),
         "note": f"floor-exempt hard-band (>= {HARD_RATIO:g}x median-of-history; "
                 f"watch-only for the gate)"}
        for r in floor_exempt_rows if r[4] >= HARD_RATIO
    ]

    # Uniform-shift exemption — ALL four conditions must hold (see header), computed over the
    # GATED (non-floor-exempt) rows only.
    uniform = False
    exemption_reasons: list[str] = []
    gated_watch = [r for r in gated_rows if r[4] >= WATCH_RATIO]
    if gated_rows and gated_watch:
        frac = len(gated_watch) / len(gated_rows)
        cond_a = frac >= UNIFORM_FRACTION
        if not cond_a:
            exemption_reasons.append(
                f"(a) breadth: only {len(gated_watch)}/{len(gated_rows)} gated metrics >= "
                f"{WATCH_RATIO}x (< {UNIFORM_FRACTION:.0%}; floor-exempt metrics excluded)")
        ratios = [r[4] for r in gated_watch]
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
        worst = max(r[4] for r in gated_rows)
        cond_c = worst < UNIFORM_CAP_RATIO
        if not cond_c:
            exemption_reasons.append(
                f"(c) cap: worst gated ratio {worst:.2f}x >= {UNIFORM_CAP_RATIO}x — a "
                f"catastrophic regression never hides in the herd")
        # Deterministic rows are never floor-exempt, so gated_rows covers every one of them.
        det_moved = [r for r in gated_rows if r[5] and abs(r[4] - 1.0) > DET_TOLERANCE]
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
        "floor_exempt": floor_exempt_rows, "floor_exempt_hard": floor_exempt_hard,
        "gated_compared": len(gated_rows), "gated_watch": len(gated_watch),
        "invalid": invalid, "ungated": ungated, "new": new, "stable_zero": stable_zero,
        "disappeared": disappeared, "disappeared_frac": disappeared_frac,
        "uniform_shift": uniform, "exemption_reasons": exemption_reasons,
    }
    return (1 if fail else 0), report


def _table_lines(entries: list[tuple], markdown: bool) -> list[str]:
    """Render [(name, cur, med, n, ratio, det, larger_better, floor_exempt)] rows as a table."""
    out = []
    if markdown:
        out.append("| metric | current | median (n) | ratio | direction | gating |")
        out.append("|---|---:|---:|---:|---|---|")
        for name, cur, med, n, ratio, _det, lb, floor in sorted(entries, key=lambda t: -t[4]):
            gating = "under noise floor — watch only" if floor else "hard-zone"
            out.append(f"| `{name}` | {cur:.6g} | {med:.6g} (n={n}) | {ratio:.2f}x |"
                       f" {'larger-is-better' if lb else 'smaller-is-better'} | {gating} |")
    else:
        out.append(f"  {'metric':<60} {'current':>14} {'median(n)':>18} {'ratio':>7}")
        for name, cur, med, n, ratio, _det, lb, floor in sorted(entries, key=lambda t: -t[4]):
            suffix = "  [larger-is-better]" if lb else ""
            if floor:
                suffix += "  [under noise floor — watch only]"
            out.append(f"  {name:<60} {cur:>14.4g} {med:>13.4g}(n={n}) {ratio:>6.2f}x{suffix}")
    return out


def render_report(report: dict, markdown: bool) -> list[str]:
    """Human-readable report lines (plain for stdout, markdown for GITHUB_STEP_SUMMARY)."""
    h2 = "## " if markdown else ""
    lines: list[str] = [f"{h2}bench hard zone — median-of-history gate"]
    lines.append(f"{report['compared']} metrics compared; {len(report['watch'])} >= "
                 f"{WATCH_RATIO}x median; {len(report['hard'])} gated >= {HARD_RATIO}x median.")
    if report["watch"]:
        lines.append("")
        lines.append(f"**Watch band (>= {WATCH_RATIO}x median):**" if markdown
                     else f"metrics >= {WATCH_RATIO}x their median of the last up-to-"
                          f"{HISTORY_WINDOW} published values:")
        lines.extend(_table_lines(report["watch"], markdown))
    if report["floor_exempt"]:
        floor_watch = sum(1 for r in report["floor_exempt"] if r[4] >= WATCH_RATIO)
        lines.append(f"{len(report['floor_exempt'])} noisy metric(s) under their unit's noise "
                     f"floor (allow-list: {_floor_desc()}): watch-only, never a hard fail alone "
                     f"(measured honest bands at that scale exceed {HARD_RATIO}x); "
                     f"{floor_watch} currently >= {WATCH_RATIO}x median (flagged in the table).")
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
        # GATED numbers only — the exemption predicate is computed over the gated
        # (non-floor-exempt) rows, so the annotation must report exactly those counts:
        # total-row numbers would misstate the breadth and "no metric reached 4x" would be
        # FALSE whenever a floor-exempt row honestly spiked past the cap.
        print(f"::warning title=bench hard zone — uniform environment shift, not gating::"
              f"{report['gated_watch']} of {report['gated_compared']} GATED metrics "
              f"(floor-exempt excluded) are >= {WATCH_RATIO}x their median-of-history "
              f"simultaneously, the shifted ratios are uniform (stdev/mean <= "
              f"{UNIFORM_MAX_CV}), no GATED metric reached {UNIFORM_CAP_RATIO}x, and every "
              f"deterministic metric is within {DET_TOLERANCE:.0%} of its median. Treated as "
              f"a runner-environment scaling and NOT failing the run — a masked code "
              f"regression is implausible under these four conditions, not impossible, so "
              f"review the table in the job summary.")
    for name, reason in report["invalid"]:
        print(f"::error title=bench hard zone — invalid measurement::{name}: {reason}")
    if report["disappeared_frac"] > DISAPPEAR_FAIL_FRACTION:
        print(f"::error title=bench hard zone — suite breakage::"
              f"{len(report['disappeared'])} gateable metrics "
              f"({report['disappeared_frac']:.0%}) are missing from this run's results — more "
              f"than {DISAPPEAR_FAIL_FRACTION:.0%} of the gated suite disappeared at once, "
              f"which is a bench-suite breakage, not a rename. See the disappeared list above.")
    floor_hot = [r for r in report["floor_exempt"] if r[4] >= HARD_RATIO]
    if floor_hot:
        names = ", ".join(f"{r[0]} ({r[4]:.2f}x, median {r[2]:.6g})" for r in floor_hot)
        print(f"::warning title=bench hard zone — under noise floor, watch only::{names} "
              f"at/above {HARD_RATIO}x median, but the median is under the unit's noise floor "
              f"(allow-list: {_floor_desc()}) — measured honest run-to-run bands at that scale "
              f"exceed {HARD_RATIO}x, so this cannot fail the gate alone. It stays visible "
              f"here, in the watch table, in the Store step's comparison comment, AND it is "
              f"routed via the --report-out JSON into the soft-triage deduped bench-flake "
              f"issue — the durable trail that survives the median rebasing.")
    if report["hard"] and not report["uniform_shift"]:
        for name, cur, med, n, ratio, _det, lb, _floor in report["hard"]:
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
    ap.add_argument("--report-out", default=None,
                    help="write a machine-readable report JSON here (carries the "
                         "floor_exempt_hard rows bench-triage.py --hardzone-report merges "
                         "into the deduped bench-flake issue — the durable trail for "
                         "sub-floor watch-only regressions); optional")
    return ap


def run_gate(results_path: str, prev_data_path: str, suite: str,
             report_out: str | None = None) -> int:
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
    if report_out:
        # Written on pass AND fail (the step may fail on another metric while a floor-exempt
        # row still needs its durable trail); the Soft-zone triage step runs `if: always()`.
        Path(report_out).write_text(json.dumps(
            {"suite": suite, "hard_ratio": HARD_RATIO,
             "floor_exempt_units": FLOOR_EXEMPT_UNITS,
             "floor_exempt_hard": report["floor_exempt_hard"]},
            indent=2) + "\n", encoding="utf-8")
        log(f"report JSON ({len(report['floor_exempt_hard'])} floor-exempt hard-band row(s)) "
            f"written to {report_out}")
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

    # Baseline 5-commit history: four metrics stable at 100.0 — deliberately ABOVE NOISE_FLOOR
    # so these fixtures keep exercising the hard zone proper (sub-floor behavior is tested in
    # its own section below).
    series = _series([[("a", 100.0), ("b", 100.0), ("c", 100.0), ("d", 100.0)]] * 5)

    # 1. Median semantics: window trims to the last 5 VALUES per metric.
    hist = history_values(_series([[("m", 100.0)], [("m", 1.0)], [("m", 2.0)], [("m", 3.0)],
                                   [("m", 4.0)], [("m", 5.0)]]))
    check(hist["m"] == [1.0, 2.0, 3.0, 4.0, 5.0], "history window keeps only the last 5 values")
    check(statistics.median(hist["m"]) == 3.0, "median of 5-value history is the middle value")
    hist_even = history_values(_series([[("m", 1.0)], [("m", 2.0)], [("m", 3.0)], [("m", 8.0)]]))
    check(statistics.median(hist_even["m"]) == 2.5, "median of an even count averages the middle two")
    code, rep = evaluate(_cur([("m", 150.0)]), history_values(
        _series([[("m", 100.0)], [("m", 100.0)], [("m", 100.0)], [("m", 100.0)],
                 [("m", 300.0)]])))
    check(code == 0 and not rep["hard"],
          "single outlier history point does not move the median (1.5x vs median passes)")

    # 2. SPARSE-metric per-metric windowing: a metric present in only 5 of 10 commits still
    #    collects all 5 of its values (the old per-commit window judged it from 1 point).
    sparse_pts: list[list[tuple]] = []
    for i in range(10):
        commit: list[tuple] = [("dense", 100.0)]
        if i % 2 == 0:
            commit.append(("sparse", 100.0 + 10 * i))  # 100,120,140,160,180 at commits 0,2,4,6,8
        sparse_pts.append(commit)
    hist_sparse = history_values(_series(sparse_pts))
    check(hist_sparse["sparse"] == [100.0, 120.0, 140.0, 160.0, 180.0],
          "sparse metric collects its last 5 values across the FULL retained history")
    code, rep = evaluate(_cur([("sparse", 150.0), ("dense", 100.0)]), hist_sparse)
    check(code == 0 and rep["compared"] == 2,
          "sparse metric is gated against its own 5-value median (140.0), not a 1-point window")

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
    lone = _cur([("a", 210.0), ("b", 102.0), ("c", 99.0), ("d", 100.0)])
    code, rep = evaluate(lone, history_values(series))
    check(code == 1 and not rep["uniform_shift"] and len(rep["hard"]) == 1
          and rep["hard"][0][0] == "a",
          "single sustained >= 2.0x regression hard-fails (no uniform exemption)")
    watch_only = _cur([("a", 180.0), ("b", 100.0), ("c", 100.0), ("d", 100.0)])
    code, rep = evaluate(watch_only, history_values(series))
    check(code == 0 and len(rep["watch"]) == 1 and not rep["hard"],
          "a lone watch-band metric (1.8x, < 2.0x) is reported but does not fail")

    # 9. Uniform-shift exemption POSITIVE: broad (3/4 >= 1.75x), uniform ratios (cv <= 0.15),
    #    nothing >= 4x, no deterministic metric moved => exempt, exit 0.
    shifted = _cur([("a", 190.0), ("b", 195.0), ("c", 205.0), ("d", 110.0)])
    code, rep = evaluate(shifted, history_values(series))
    check(code == 0 and rep["uniform_shift"] and len(rep["watch"]) == 3 and len(rep["hard"]) == 1,
          "uniform environment shift (3/4 >= 1.75x, uniform, capped, no det drift) is exempted")

    # 10. Uniformity rejection: broad shift with NON-uniform ratios => NOT exempt => fail.
    ragged = _cur([("a", 180.0), ("b", 250.0), ("c", 350.0), ("d", 100.0)])  # 1.8/2.5/3.5
    code, rep = evaluate(ragged, history_values(series))
    check(code == 1 and not rep["uniform_shift"]
          and any("(b) uniformity" in r for r in rep["exemption_reasons"]),
          "broad but NON-uniform shift (cv > 0.15) is NOT exempt — hard fail stands")

    # 11. The 4x cap ISOLATED: conditions (a) breadth, (b) uniformity and (d) deterministic all
    #     PASS — ratios 4.05/4.1/4.08/4.06 put 4/4 metrics in the watch band with CV ~0.005 and
    #     no deterministic metrics — so ONLY the cap (c) blocks the exemption. Removing the cap
    #     from the predicate would flip exactly this check red.
    capped = _cur([("a", 405.0), ("b", 410.0), ("c", 408.0), ("d", 406.0)])
    code, rep = evaluate(capped, history_values(series))
    check(code == 1 and not rep["uniform_shift"] and len(rep["hard"]) == 4
          and len(rep["exemption_reasons"]) == 1
          and "(c) cap" in rep["exemption_reasons"][0],
          "a >= 4x metric is the SOLE exemption blocker ((a)/(b)/(d) pass) — hard fail stands")
    #     Control: the SAME uniform shape kept below the cap (all ~1.9x) IS exempt — proving the
    #     cap is the discriminating condition, not breadth/uniformity/deterministic drift.
    under_cap = _cur([("a", 190.0), ("b", 195.0), ("c", 192.0), ("d", 191.0)])
    code, rep = evaluate(under_cap, history_values(series))
    check(code == 0 and rep["uniform_shift"] and not rep["exemption_reasons"],
          "same uniform shape below the cap IS exempt — the 4x cap is the discriminating condition")

    # 12. Deterministic-metric drift blocks the exemption: same broad uniform timing shift, but a
    #     byte-count metric moved > 1% => NOT exempt => fail.
    det_series = [{"date": i, "benches": [
        {"name": "a", "value": 100.0, "unit": "us"},
        {"name": "b", "value": 100.0, "unit": "us"},
        {"name": "c", "value": 100.0, "unit": "us"},
        {"name": "wasm_bundle_bytes", "value": 1000.0, "unit": "bytes"},
    ]} for i in range(5)]
    det_cur = [{"name": "a", "value": 190.0, "unit": "us"},
               {"name": "b", "value": 195.0, "unit": "us"},
               {"name": "c", "value": 205.0, "unit": "us"},
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
    #     "milli" is NOISY: vectors_hnsw_recall_at10 honestly oscillates 1<->4 in the live
    #     history (randomized HNSW build at degenerate resolution) — a measurement, not a count.
    check(is_deterministic("wasm_bundle_bytes", "bytes")
          and not is_deterministic("bsbm_query01_count_us", "us")
          and not is_deterministic("rsp_persistentdict_triples_per_s", "triples_per_s")
          and not is_deterministic("vectors_hnsw_recall_at10", "milli")
          and is_deterministic("zk_compose_filter_f64_gates", ""),
          "deterministic classification: unit-first, anchored-name fallback only when unit absent")

    # 14. NOISE FLOOR — sub-floor "us" timing metric at 3.0x median: watch-only, NEVER a hard
    #     fail alone (measured honest µs-scale bands exceed 2x — see FLOOR_EXEMPT_UNITS).
    #     MUTATION CHECK vs check 15: zeroing the "us" floor in FLOOR_EXEMPT_UNITS (or removing
    #     the entry) must turn exactly THIS red.
    sub = history_values(_series([[("tiny_query_us", 4.0)]] * 5))
    code, rep = evaluate(_cur([("tiny_query_us", 12.0)]), sub)
    check(code == 0 and not rep["hard"] and len(rep["floor_exempt"]) == 1
          and len(rep["watch"]) == 1 and rep["watch"][0][0] == "tiny_query_us",
          "sub-floor timing metric at 3.0x median is watch-only — no hard fail")
    check("under noise floor — watch only" in "\n".join(render_report(rep, markdown=False)),
          "sub-floor watch row is flagged 'under noise floor — watch only' in the report")
    #     DURABLE TRAIL: the same hit is emitted in floor_exempt_hard in the soft-triage
    #     comparison shape (previous = median-of-history, tagged "floor-exempt hard-band") —
    #     the exact contract bench-triage.py --hardzone-report consumes.
    check(rep["floor_exempt_hard"] == [{
              "name": "tiny_query_us", "current": 12.0, "previous": 4.0, "ratio_pct": 300.0,
              "unit": "us",
              "note": f"floor-exempt hard-band (>= {HARD_RATIO:g}x median-of-history; "
                      f"watch-only for the gate)"}],
          "floor-exempt hard-band hit is emitted in the soft-triage comparison shape")
    code, rep = evaluate(_cur([("tiny_query_us", 7.0)]), sub)  # 1.75x: watch, below hard band
    check(code == 0 and len(rep["floor_exempt"]) == 1 and not rep["floor_exempt_hard"],
          "a sub-floor row below the hard band is NOT routed to the durable trail")

    # 15. The floor is the DISCRIMINATOR: the same 3.0x shape with its median ABOVE the floor
    #     hard-fails; and a sub-floor DETERMINISTIC metric at 2.0x still hard-fails (the floor
    #     never applies to deterministic outputs — a 2x on a count/bytes/gates value is real).
    over = history_values(_series([[("big_query_us", 40.0)]] * 5))
    code, rep = evaluate(_cur([("big_query_us", 120.0)]), over)
    check(code == 1 and len(rep["hard"]) == 1 and not rep["floor_exempt"],
          "the same 3.0x shape with median above the floor hard-fails")
    det_small = history_values([{"date": i, "benches": [
        {"name": "solid_fixture_count", "value": 5.0, "unit": "count"}]} for i in range(5)])
    code, rep = evaluate([{"name": "solid_fixture_count", "value": 10.0, "unit": "count"}],
                         det_small)
    check(code == 1 and len(rep["hard"]) == 1 and not rep["floor_exempt"],
          "sub-floor DETERMINISTIC metric at 2.0x still hard-fails (floor never applies)")

    # 15b. UNIT-AWARE floor ([FABLE-5] round-2 review): the floor is an explicit per-unit
    #      allow-list ("us"/"milli"), NOT a raw magnitude comparison. The published history
    #      carries nine `s`-unit series (load_s, text_build_s, rdfs_infer_s, ... at ~0.4-15 s)
    #      whose RAW medians sit under 20 — a raw floor wrongly exempted ALL of them, so a
    #      stable 0.4s -> 4s (10x) load regression would pass and rebase the median.
    #      MUTATION CHECK: adding "s" to FLOOR_EXEMPT_UNITS must turn exactly THIS red.
    load_hist = history_values(_series([[("load_s", 0.4)]] * 5))
    code, rep = evaluate(_cur([("load_s", 4.0)], unit="s"), load_hist)
    check(code == 1 and len(rep["hard"]) == 1 and rep["hard"][0][0] == "load_s"
          and not rep["floor_exempt"],
          "s-unit metric (median 0.4 s) at 10x median HARD-FAILS — seconds are never "
          "floor-exempt regardless of magnitude")
    #      Every other non-allow-listed unit with a small median stays gated too — including
    #      an UNKNOWN unit and a missing unit (allow-list fails toward gated, never open).
    for u in ("ns_per_byte", "ratio", "mystery_unit", ""):
        code, rep = evaluate(_cur([("small_metric", 12.0)], unit=u),
                             history_values(_series([[("small_metric", 4.0)]] * 5)))
        check(code == 1 and len(rep["hard"]) == 1 and not rep["floor_exempt"],
              f"unit {u!r} (median 4) at 3.0x median hard-fails — not in the floor allow-list")
    #      "milli" IS allow-listed: the empirically-degenerate HNSW recall family (honest
    #      1<->4 oscillation at integer-milli resolution) stays watch-only under the floor.
    code, rep = evaluate(_cur([("vectors_hnsw_recall_at10", 4.0)], unit="milli"),
                         history_values(_series([[("vectors_hnsw_recall_at10", 1.0)]] * 5)))
    check(code == 0 and not rep["hard"] and len(rep["floor_exempt"]) == 1
          and len(rep["floor_exempt_hard"]) == 1,
          "milli-unit metric (median 1) at 4.0x is floor-exempt (HNSW recall family) and "
          "routed to the durable trail")

    # 16. Uniform-shift computations EXCLUDE floor-exempt metrics — degenerate sub-floor ratios
    #     must distort neither breadth (a) nor uniformity (b) nor the cap (c).
    mix_series = [{"date": i, "benches":
                   [{"name": n, "value": 100.0, "unit": "us"} for n in "abcd"]
                   + [{"name": f"tiny{j}_us", "value": 4.0, "unit": "us"} for j in range(5)]}
                  for i in range(5)]
    #     4 gated metrics uniformly ~2.05x (a hard fail without the exemption) + 5 sub-floor
    #     metrics unmoved: excluding the floor-exempt rows, breadth is 4/4 => exempt (exit 0);
    #     counting them, breadth would be 4/9 (44% < 50%) => condition (a) fails => exit 1.
    mix_cur = _cur([("a", 210.0), ("b", 205.0), ("c", 208.0), ("d", 195.0)]
                   + [(f"tiny{j}_us", 4.0) for j in range(5)])
    code, rep = evaluate(mix_cur, history_values(mix_series))
    check(code == 0 and rep["uniform_shift"] and len(rep["floor_exempt"]) == 5,
          "breadth excludes floor-exempt metrics (4/4 gated uniform => exempt; 4/9 would fail)")
    #     A sub-floor metric spiking 4.0x (the honest vectors_hnsw 1<->4 shape) must block
    #     neither the cap (c) nor uniformity (b) for a genuine environment shift.
    mix_cur2 = _cur([("a", 210.0), ("b", 205.0), ("c", 208.0), ("d", 195.0), ("tiny0_us", 16.0)]
                    + [(f"tiny{j}_us", 4.0) for j in range(1, 5)])
    code, rep = evaluate(mix_cur2, history_values(mix_series))
    check(code == 0 and rep["uniform_shift"],
          "a 4.0x sub-floor spike blocks neither the cap nor uniformity (excluded from both)")

    # 16b. The uniform-shift ::warning reports GATED numerator/denominator + "no GATED metric
    #      reached" wording — the predicate is gated-rows-only, so total-row numbers would
    #      misstate the breadth (5/9 here) and "no metric reached 4x" would be FALSE (the
    #      floor-exempt tiny0_us honestly sits at exactly 4.0x in this fixture).
    buf = io.StringIO()
    saved_summary = os.environ.pop("GITHUB_STEP_SUMMARY", None)  # keep the CI summary clean
    try:
        with contextlib.redirect_stdout(buf):
            emit_report(rep)
    finally:
        if saved_summary is not None:
            os.environ["GITHUB_STEP_SUMMARY"] = saved_summary
    out = buf.getvalue()
    check("4 of 4 GATED metrics" in out and "no GATED metric reached" in out,
          "uniform-shift annotation reports the gated numerator/denominator + GATED wording")
    check("5 of 9" not in out and "no metric reached" not in out,
          "uniform-shift annotation no longer reports total-row numbers")

    # 17. Argument handling: the parser accepts the documented flags on a fixture path.
    ns = build_parser().parse_args(
        ["--results", "fixtures/bench-results.json", "--prev-data", "fixtures/prev-data.js",
         "--suite", "sparq engine"])
    check(ns.results == "fixtures/bench-results.json"
          and ns.prev_data == "fixtures/prev-data.js" and ns.suite == "sparq engine",
          "argument parser maps --results/--prev-data/--suite onto the expected namespace")

    # 18. End-to-end run_gate over real files in a private tempdir: suite-name mismatch must
    #     warn + exit 0 (never fall back to another suite), exact match must gate.
    with tempfile.TemporaryDirectory(prefix="bench-hardzone-selftest-") as td:
        results = Path(td) / "bench-results.json"
        results.write_text(json.dumps(_cur([("a", 210.0), ("b", 100.0), ("c", 100.0),
                                            ("d", 100.0)])), encoding="utf-8")
        prev = Path(td) / "prev-data.js"
        payload = {"entries": {"sparq engine": _series(
            [[("a", 100.0), ("b", 100.0), ("c", 100.0), ("d", 100.0)]] * 5)}}
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
        # --report-out: the run writes the machine-readable report JSON carrying the
        # floor-exempt hard-band rows (the soft-triage durable-trail hand-off).
        sub_results = Path(td) / "sub-results.json"
        sub_results.write_text(json.dumps(_cur([("tiny_us", 12.0)])), encoding="utf-8")
        sub_prev = Path(td) / "sub-prev.js"
        sub_prev.write_text("window.BENCHMARK_DATA = " + json.dumps(
            {"entries": {"sparq engine": _series([[("tiny_us", 4.0)]] * 5)}}) + ";",
            encoding="utf-8")
        report_path = Path(td) / "hardzone-report.json"
        rc = run_gate(str(sub_results), str(sub_prev), "sparq engine", str(report_path))
        payload = json.loads(report_path.read_text(encoding="utf-8"))
        check(rc == 0 and [r["name"] for r in payload["floor_exempt_hard"]] == ["tiny_us"]
              and "floor-exempt hard-band" in payload["floor_exempt_hard"][0]["note"]
              and payload["floor_exempt_hard"][0]["previous"] == 4.0,
              "--report-out writes the floor-exempt hard-band rows for the soft-triage trail")

    if failures:
        log(f"self-test FAILED ({len(failures)}/{checks} checks): " + "; ".join(failures))
        return 1
    log(f"self-test OK ({checks} checks)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    args = build_parser().parse_args(argv)
    return run_gate(args.results, args.prev_data, args.suite, args.report_out)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
