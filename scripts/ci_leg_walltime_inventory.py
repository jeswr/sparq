#!/usr/bin/env python3
"""Per-leg CI wall-time inventory — the steering data for sq-6vshe (§8). 🤖 SPARQ agent.

Bead sq-6vshe.7 (a). Design: research/ci-structural-speedup.md §8, and the record this
instrument exists to fill in, research/ci-leg-walltime-inventory.md.

WHAT THIS IS. `sq-6vshe.7` asks for "a per-leg wall-time inventory decomposed into
constant overhead (queue/checkout/toolchain/cache restore) vs compile vs test-run — this
table is the steering data for the whole program". That table cannot be authored; it has
to be MEASURED, and nothing in this repo measured it. `scripts/ci_execution_latency_alarm.py`
reads the same Actions API but answers a different question (is a lane failing to fire /
overrunning?) and keeps no per-step decomposition; `ci_summary_gate.py` is gate logic.
This script is the missing instrument: it pulls completed runs, decomposes every job's
wall-clock into named buckets from the per-step timings the API already returns, and emits
the steering table.

WHY A STEP-NAME CLASSIFIER, AND WHAT THAT COSTS. The Actions API gives each step a name,
`started_at` and `completed_at` — and nothing else. There is no machine-readable "this
step was a compile". So the split is inferred from step NAMES (`classify_step`), which is
an approximation, and the honest consequence is designed in rather than hidden:

  * every step that no rule matches lands in OTHER, never silently in a flattering bucket;
  * the report names the top unmatched steps whenever there are any, so a table whose
    classification has decayed says so on its face;
  * the decay signal is the UNCLASSIFIED share — the wall-clock of steps no rule matched —
    which is tracked separately from the OTHER bucket. OTHER is a WORKLOAD category and
    also holds deliberately-recognised guard/ratchet/report steps; a leg that spends real
    time in those is fully classified and must not read as decayed. Conflating the two
    would make a healthy classifier look broken (and vice versa: a large recognised gate
    step would mask genuine decay by dominating the same percentage);
  * `--max-unclassified-pct` turns the unclassified share into a hard exit for unattended
    use.

Read an UNCLASSIFIED share above a few percent as "this leg is not classified yet", not
as "this leg has no compile". A large OTHER share with a zero unclassified share means
something else entirely: the leg really does spend its time in gates and reports.

WHAT IS DELIBERATELY OUT OF SCOPE. Coverage-instrumented jobs are `sq-piapk`'s topology,
not this bead's, and are EXCLUDED by default (`--include-coverage` to opt in). The
exclusion is by job-name predicate (`is_coverage_job`) so the two beads' inventories
cannot silently overlap.

BUCKETS. Per job:
  queue        job.started_at - job.created_at  (runner pickup; not a step)
  setup        runner set-up/teardown, checkout, toolchain, tool install, cache
               restore/save, artifact upload/download, disk reclaim  <- the per-leg
               CONSTANT TAX the bead is trying to fold away
  fixture      pinned external test-corpus fetches (W3C suites etc.) — a real, separable
               constant tax that only the conformance legs pay
  compile      cargo build / check / nextest archive / clippy / rustdoc / wasm-pack build
  doctest      the `cargo test --doc` lane — broken out because sq-6vshe.7(d) is
               explicitly a doctest-cost audit and nextest never runs doctests
  test         actual test execution (nextest run, cargo test, wasm-pack test, pytest, …)
  other        ratchet/guard/report scripts (recognised), plus anything unmatched
  unaccounted  job wall-clock minus the sum of its steps (runner overhead the step list
               does not itemise) — reported, never redistributed

The unmatched slice of OTHER is ALSO tracked on its own (`unmatched_secs` per leg,
`unclassified_pct` overall) — that, not the OTHER share, is the classifier-decay metric.

USAGE
  # measure (needs `gh` authenticated; writes a reproducible dump)
  ci_leg_walltime_inventory.py --repo owner/name --runs 20 --dump-json /tmp/jobs.json

  # re-analyse offline, no network, from that dump
  ci_leg_walltime_inventory.py --from-json /tmp/jobs.json --format markdown

  # hermetic self-test (no gh, no network)
  ci_leg_walltime_inventory.py --self-test

EXIT CODES. 0 clean; 1 a declared threshold was breached (--max-unclassified-pct); 2 the
instrument could not measure — no runs, no jobs, gh failure, unreadable dump. Reporting a
table over an empty population is the one thing it must never do.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
from collections import defaultdict

DEFAULT_REPO = "sparq-org/sparq"

# Non-coverage gating lanes this bead steers. Additional lanes via --workflow.
DEFAULT_WORKFLOWS = ["ci.yml", "feature-matrix.yml"]

QUEUE, SETUP, FIXTURE, COMPILE, DOCTEST, TEST, OTHER = (
    "queue", "setup", "fixture", "compile", "doctest", "test", "other",
)
BUCKETS = [SETUP, FIXTURE, COMPILE, DOCTEST, TEST, OTHER]

# The unmatched slice of OTHER, carried as its own field. NOT a bucket — it is a subset of
# OTHER, so adding it to BUCKETS would double-count against `unaccounted` and `wall`.
UNMATCHED = "unmatched_secs"

# --- step-name classification -------------------------------------------------------
#
# ORDER IS LOAD-BEARING: the first matching rule wins. Two orderings in particular are
# not accidental and must not be "tidied":
#   * `install cargo-nextest` is a TOOL INSTALL, not a test run — the install rules sit
#     above every cargo rule, or half the setup tax would be booked as test time.
#   * `Test-presence gate` / `Shard-partition guard` are shell gates that happen to start
#     with a test word — the gate/guard/ratchet rule sits above the test rules.
#
# Each entry is (rule-name, compiled pattern, bucket). Patterns run against the
# lowercased step name.
_RULES: list[tuple[str, re.Pattern, str]] = [
    # runner-managed pseudo-steps the API always emits
    ("runner-setup", re.compile(r"^set up job$"), SETUP),
    ("runner-teardown", re.compile(r"^(complete job|stop containers)$"), SETUP),
    ("post-step", re.compile(r"^post "), SETUP),
    ("checkout", re.compile(r"checkout"), SETUP),
    # toolchain + tooling install
    ("toolchain", re.compile(r"^rust \(|rustup|toolchain|llvm-tools"), SETUP),
    ("tool-install", re.compile(r"^install |install-action|setup[- ]node|setup[- ]python|"
                                r"^node |^python \("), SETUP),
    ("cache", re.compile(r"cache|swatinem"), SETUP),
    ("free-disk", re.compile(r"free .*disk|disk[- ]guard"), SETUP),
    ("artifact", re.compile(r"^(upload|download)\b"), SETUP),
    # pinned external corpora — a conformance-only constant tax, separable from setup
    ("fixture-fetch", re.compile(r"^fetch |^(save|restore) the pinned |"
                                 r"test suites? \(pinned\)"), FIXTURE),
    ("restore", re.compile(r"^restore "), SETUP),
    # scripts/gates that do no compiling and run no tests
    ("script-gate", re.compile(r"\bgate\b|\bguard\b|ratchet|scoreboard|^summarize$|"
                               r"^emit |^aggregate |^enforce |report$|^detect |^decide |"
                               r"^assert |^compute |^file the |^commit \+ push|"
                               r"^assemble |^resolve |^report |^measure |"
                               r"^loud failure|coverage report"), OTHER),
    # doctests — own bucket: sq-6vshe.7(d), and nextest never runs these
    ("doctest", re.compile(r"doctest|--doc\b|test --doc"), DOCTEST),
    # compile-shaped work
    ("archive-build", re.compile(r"nextest archive"), COMPILE),
    ("lint", re.compile(r"clippy|rustfmt|rustdoc"), COMPILE),
    ("build", re.compile(r"^build |^cargo check|cargo build|instrumented (engine )?"
                         r"objects|wasm-pack build|maturin"), COMPILE),
    # test execution
    ("test-run", re.compile(r"nextest|cargo test|^test\b|wasm-pack test|pytest|npm test|"
                            r"headless wasm tests|smoke test|selftest|^run .*"
                            r"(suite|lane|oracle|test|arm|floor)"), TEST),
]

# Coverage-instrumented jobs belong to sq-piapk, not this bead. Matched on job name.
_COVERAGE_JOB = re.compile(r"coverage|profraw|instrumented|mutation|mutants", re.I)


def classify_step(name: str) -> tuple[str, str]:
    """Return (rule-name, bucket) for a step name. Unmatched -> ('unmatched', OTHER)."""
    low = (name or "").strip().lower()
    for rule, pattern, bucket in _RULES:
        if pattern.search(low):
            return rule, bucket
    return "unmatched", OTHER


def is_coverage_job(job_name: str) -> bool:
    return bool(_COVERAGE_JOB.search(job_name or ""))


# --- decomposition ------------------------------------------------------------------

def _ts(value: str | None) -> dt.datetime | None:
    if not value:
        return None
    try:
        return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def _span(start: str | None, end: str | None) -> float:
    a, b = _ts(start), _ts(end)
    if a is None or b is None:
        return 0.0
    return max(0.0, (b - a).total_seconds())


def decompose_job(job: dict) -> dict | None:
    """Decompose ONE Actions job object into the bucket split.

    Returns None for a job that never ran (skipped / no start), which must not be
    averaged in — a skipped leg costs nothing and would drag every mean toward zero.
    """
    if job.get("conclusion") == "skipped" or not job.get("started_at"):
        return None
    wall = _span(job.get("started_at"), job.get("completed_at"))
    if wall <= 0:
        return None
    buckets = dict.fromkeys(BUCKETS, 0.0)
    rules: dict[str, float] = defaultdict(float)
    unmatched: dict[str, float] = defaultdict(float)
    steps = job.get("steps") or []
    for step in steps:
        secs = _span(step.get("started_at"), step.get("completed_at"))
        if secs <= 0:
            continue
        rule, bucket = classify_step(step.get("name", ""))
        buckets[bucket] += secs
        rules[rule] += secs
        if rule == "unmatched":
            unmatched[(step.get("name") or "").strip()] += secs
    accounted = sum(buckets.values())
    return {
        "workflow": job.get("workflow_name") or "?",
        "leg": job.get("name") or "?",
        "run_id": job.get("run_id"),
        "wall": wall,
        QUEUE: _span(job.get("created_at"), job.get("started_at")),
        **buckets,
        # the decay signal: the slice of OTHER that no rule matched. A subset of OTHER,
        # deliberately not a bucket of its own.
        UNMATCHED: sum(unmatched.values()),
        # never redistributed: runner overhead the step list does not itemise
        "unaccounted": max(0.0, wall - accounted),
        "rules": dict(rules),
        "unmatched": dict(unmatched),
    }


def percentile(values: list[float], q: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    idx = min(len(ordered) - 1, max(0, int(round(q * (len(ordered) - 1)))))
    return ordered[idx]


def aggregate(jobs: list[dict], *, include_coverage: bool = False) -> dict:
    """Aggregate decomposed jobs into per-leg statistics keyed (workflow, leg)."""
    legs: dict[tuple[str, str], dict] = {}
    unmatched_total: dict[str, float] = defaultdict(float)
    skipped_coverage = 0
    for job in jobs:
        name = job.get("name") or ""
        if not include_coverage and is_coverage_job(name):
            skipped_coverage += 1
            continue
        row = decompose_job(job)
        if row is None:
            continue
        key = (row["workflow"], row["leg"])
        acc = legs.setdefault(key, {
            "workflow": row["workflow"], "leg": row["leg"], "n": 0,
            "walls": [],
            "sums": dict.fromkeys([QUEUE, *BUCKETS, UNMATCHED, "unaccounted"], 0.0),
            "rules": defaultdict(float),
        })
        acc["n"] += 1
        acc["walls"].append(row["wall"])
        for field in (QUEUE, *BUCKETS, UNMATCHED, "unaccounted"):
            acc["sums"][field] += row[field]
        for rule, secs in row["rules"].items():
            acc["rules"][rule] += secs
        for step_name, secs in row["unmatched"].items():
            unmatched_total[step_name] += secs

    rows = []
    for acc in legs.values():
        n = acc["n"]
        means = {k: v / n for k, v in acc["sums"].items()}
        rows.append({
            "workflow": acc["workflow"], "leg": acc["leg"], "n": n,
            "p50": percentile(acc["walls"], 0.50),
            "p90": percentile(acc["walls"], 0.90),
            "mean_wall": sum(acc["walls"]) / n,
            "mean": means,
            "rules": {k: v / n for k, v in acc["rules"].items()},
        })
    rows.sort(key=lambda r: r["p50"], reverse=True)

    total_wall = sum(r["mean_wall"] * r["n"] for r in rows)
    total_other = sum(r["mean"][OTHER] * r["n"] for r in rows)
    total_unmatched = sum(r["mean"][UNMATCHED] * r["n"] for r in rows)
    return {
        "legs": rows,
        "leg_count": len(rows),
        "job_samples": sum(r["n"] for r in rows),
        "coverage_jobs_excluded": skipped_coverage,
        # WORKLOAD share of the OTHER bucket — gates/ratchets/reports AND unmatched work.
        # A high value is not by itself a classifier problem.
        "other_pct": (100.0 * total_other / total_wall) if total_wall else 0.0,
        # CLASSIFIER-DECAY share — only the steps no rule matched. This is what
        # --max-unclassified-pct thresholds.
        "unclassified_pct": (100.0 * total_unmatched / total_wall) if total_wall else 0.0,
        "unmatched_steps": sorted(unmatched_total.items(), key=lambda kv: -kv[1]),
    }


# --- rendering ----------------------------------------------------------------------

def _fmt(seconds: float) -> str:
    return f"{seconds:.0f}s" if seconds < 600 else f"{seconds / 60:.1f}m"


def render_markdown(stats: dict, *, top: int = 0) -> str:
    rows = stats["legs"][:top] if top else stats["legs"]
    out = [
        "| leg | n | p50 | p90 | queue | setup | fixture | compile | doctest | test | other | unacct |",
        "|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|",
    ]
    for r in rows:
        m = r["mean"]
        out.append(
            f"| `{r['workflow']}` / {r['leg']} | {r['n']} | {_fmt(r['p50'])} | "
            f"{_fmt(r['p90'])} | {_fmt(m[QUEUE])} | {_fmt(m[SETUP])} | {_fmt(m[FIXTURE])} | "
            f"{_fmt(m[COMPILE])} | {_fmt(m[DOCTEST])} | {_fmt(m[TEST])} | {_fmt(m[OTHER])} | "
            f"{_fmt(m['unaccounted'])} |"
        )
    out.append("")
    out.append(
        f"{stats['leg_count']} legs over {stats['job_samples']} job samples; "
        f"{stats['coverage_jobs_excluded']} coverage jobs excluded (sq-piapk's topology). "
        f"OTHER workload share of measured wall-clock: {stats['other_pct']:.1f}% "
        f"(recognised gate/ratchet/report steps plus anything unmatched). "
        f"Unclassified share (steps NO rule matched — the classifier-decay signal): "
        f"{stats['unclassified_pct']:.1f}%."
    )
    if stats["unmatched_steps"]:
        out.append("")
        out.append("Top step names NO rule matched (extend `_RULES` before trusting the split):")
        for name, secs in stats["unmatched_steps"][:10]:
            out.append(f"  - {_fmt(secs)} total — {name!r}")
    return "\n".join(out)


# --- fetching -----------------------------------------------------------------------

def _gh(args: list[str]):
    try:
        out = subprocess.run(
            ["gh", *args], check=True, capture_output=True, text=True, timeout=180,
        ).stdout
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        raise SystemExit(f"FAIL-LOUD: `gh {' '.join(args)}` failed: {exc}")
    try:
        return json.loads(out)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"FAIL-LOUD: `gh {' '.join(args)}` returned non-JSON: {exc}")


def fetch_jobs(repo: str, workflows: list[str], event: str, runs: int) -> list[dict]:
    """Completed-run jobs for the named workflows. Fails loud on an empty population."""
    jobs: list[dict] = []
    for workflow in workflows:
        payload = _gh(["api", "-X", "GET",
                       f"/repos/{repo}/actions/workflows/{workflow}/runs",
                       "-f", f"event={event}", "-f", "status=completed",
                       "-f", f"per_page={max(1, min(runs, 100))}"])
        for run in payload.get("workflow_runs", [])[:runs]:
            page = _gh(["api", "-X", "GET",
                        f"/repos/{repo}/actions/runs/{run['id']}/jobs",
                        "-f", "per_page=100"])
            jobs.extend(page.get("jobs", []))
    return jobs


# --- self-test ----------------------------------------------------------------------

def _step(name: str, start: str, end: str) -> dict:
    return {"name": name, "started_at": start, "completed_at": end}


def _self_test() -> int:
    failures: list[str] = []
    checks_run = 0

    def check(name: str, condition: bool) -> None:
        nonlocal checks_run
        checks_run += 1
        if not condition:
            failures.append(name)

    # KNOWN-ANSWER: step names copied VERBATIM from .github/workflows/ci.yml. If the
    # classifier drifts, this table reds rather than the report quietly mis-bucketing.
    known = {
        "Set up job": SETUP,
        "Complete job": SETUP,
        "Post Run actions/checkout@v7.0.0": SETUP,
        "Run actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0": SETUP,
        "Rust (preinstalled stable)": SETUP,
        "Rust (pinned MSRV toolchain 1.88)": SETUP,
        "Ensure llvm-tools-preview on the pinned toolchain": SETUP,
        "Install cargo-nextest": SETUP,          # NOT a test run — ordering guard
        "Cache cargo + target": SETUP,
        "Free runner disk before download + test": SETUP,
        "Upload nextest archive": SETUP,
        "Download nextest archive": SETUP,
        "Fetch W3C SPARQL test suites (pinned)": FIXTURE,
        "Fetch inference test suites (pinned)": FIXTURE,
        "Build + archive test binaries (nextest archive)": COMPILE,
        "clippy (deny warnings)": COMPILE,
        "rustdoc (deny warnings, workspace — all features)": COMPILE,
        "cargo check on the declared MSRV": COMPILE,
        "Build sparq-wasm for the browser target": COMPILE,
        "Doctests (nextest does not run these; run once, off the warm build)": DOCTEST,
        "Test (nextest, shard bulk 1/3)": TEST,
        "Run conformance suites": TEST,
        "Run SHACL core + SHACL-SPARQL suites": TEST,
        "Headless wasm tests (wasm-pack test --node)": TEST,
        "Smoke test (docker run + /health + ASK query)": TEST,
        "Test-presence gate (fast, no compile)": OTHER,   # gate, not a test run
        "Shard-partition guard (fast, no compile)": OTHER,
        "Enforce ratchet (pass count must not regress)": OTHER,
    }
    for step_name, expected in known.items():
        rule, bucket = classify_step(step_name)
        check(f"classify {step_name!r} -> {expected} (got {bucket} via {rule})",
              bucket == expected)

    # NON-VACUITY: an unknown name must land in OTHER as 'unmatched', so the classifier
    # cannot pass this suite by bucketing everything into one place.
    rule, bucket = classify_step("Frobnicate the widget")
    check("unknown step -> unmatched/OTHER", (rule, bucket) == ("unmatched", OTHER))
    check("not every name is SETUP", classify_step("Run conformance suites")[1] != SETUP)

    # coverage exclusion (sq-piapk non-overlap)
    check("coverage job detected", is_coverage_job("coverage (measure shard 1/4)"))
    check("mutation job detected", is_coverage_job("mutation ratchet (sparq-core)"))
    check("test shard NOT coverage", not is_coverage_job("test (load-aware shard bulk 1/3)"))

    # decomposition arithmetic
    job = {
        "name": "test (load-aware shard bulk 1/3)", "workflow_name": "ci",
        "conclusion": "success", "run_id": 1,
        "created_at": "2026-07-01T00:00:00Z",
        "started_at": "2026-07-01T00:00:10Z",
        "completed_at": "2026-07-01T00:04:10Z",   # 240s wall
        "steps": [
            _step("Set up job", "2026-07-01T00:00:10Z", "2026-07-01T00:00:20Z"),      # 10 setup
            _step("Install cargo-nextest", "2026-07-01T00:00:20Z", "2026-07-01T00:00:50Z"),  # 30 setup
            _step("Download nextest archive", "2026-07-01T00:00:50Z", "2026-07-01T00:01:10Z"),  # 20 setup
            _step("Test (nextest, shard bulk 1/3)", "2026-07-01T00:01:10Z", "2026-07-01T00:03:10Z"),  # 120 test
            _step("Frobnicate", "2026-07-01T00:03:10Z", "2026-07-01T00:03:20Z"),      # 10 other
        ],
    }
    row = decompose_job(job)
    check("wall 240s", abs(row["wall"] - 240.0) < 0.01)
    check("queue 10s", abs(row[QUEUE] - 10.0) < 0.01)
    check("setup 60s", abs(row[SETUP] - 60.0) < 0.01)
    check("test 120s", abs(row[TEST] - 120.0) < 0.01)
    check("other 10s", abs(row[OTHER] - 10.0) < 0.01)
    check("unmatched 10s", abs(row[UNMATCHED] - 10.0) < 0.01)
    # 240 wall - (60+120+10) accounted = 50s the step list does not itemise
    check("unaccounted 50s", abs(row["unaccounted"] - 50.0) < 0.01)
    check("unmatched named", "Frobnicate" in row["unmatched"])

    # a skipped leg is NOT a zero-cost sample: it must be dropped, not averaged in
    check("skipped job dropped", decompose_job({**job, "conclusion": "skipped"}) is None)
    check("unstarted job dropped", decompose_job({**job, "started_at": None}) is None)

    # aggregation + coverage exclusion end to end
    cov = {**job, "name": "coverage (measure shard 1/4)"}
    stats = aggregate([job, job, cov])
    check("one leg after coverage exclusion", stats["leg_count"] == 1)
    check("two samples", stats["job_samples"] == 2)
    check("coverage excluded counted", stats["coverage_jobs_excluded"] == 1)
    check("other_pct ~4.2", abs(stats["other_pct"] - (100 * 10 / 240)) < 0.1)
    check("unclassified_pct ~4.2 (the OTHER here IS unmatched)",
          abs(stats["unclassified_pct"] - (100 * 10 / 240)) < 0.1)
    check("include_coverage opt-in works",
          aggregate([job, cov], include_coverage=True)["leg_count"] == 2)

    # DISCRIMINATION: a RECOGNISED gate step is real OTHER workload but is NOT classifier
    # decay. If unclassified_pct ever tracks the whole OTHER bucket again, this reds.
    gated = {**job, "steps": [
        _step("Set up job", "2026-07-01T00:00:10Z", "2026-07-01T00:00:20Z"),          # 10 setup
        _step("Enforce ratchet (pass count must not regress)",
              "2026-07-01T00:00:20Z", "2026-07-01T00:02:20Z"),                        # 120 other
    ]}
    gstats = aggregate([gated])
    check("recognised gate grows OTHER", gstats["other_pct"] > 45.0)
    check("recognised gate leaves unclassified at zero", gstats["unclassified_pct"] == 0.0)
    check("recognised gate names no unmatched step", gstats["unmatched_steps"] == [])

    # percentile edges
    check("p50 of one value", abs(percentile([7.0], 0.5) - 7.0) < 1e-9)
    check("p90 picks the tail", percentile([1.0, 2.0, 3.0, 100.0], 0.9) == 100.0)
    check("percentile of empty is 0", percentile([], 0.5) == 0.0)

    # rendering never crashes and states its own uncertainty
    md = render_markdown(stats)
    check("markdown names the leg", "bulk 1/3" in md)
    check("markdown states unclassified share", "Unclassified share" in md)
    check("markdown states OTHER workload share", "OTHER workload share" in md)
    check("markdown lists the unmatched step", "Frobnicate" in md)

    for failure in failures:
        print(f"FAIL: {failure}")
    print(f"{checks_run - len(failures)} checks passed, {len(failures)} failed")
    return 1 if failures else 0


# --- CLI ----------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Per-leg CI wall-time inventory (sq-6vshe.7)")
    parser.add_argument("--repo", default=DEFAULT_REPO)
    parser.add_argument("--workflow", action="append", default=None,
                        help=f"repeatable; default {DEFAULT_WORKFLOWS}")
    parser.add_argument("--event", default="pull_request")
    parser.add_argument("--runs", type=int, default=20)
    parser.add_argument("--from-json", help="analyse a saved jobs dump; no network")
    parser.add_argument("--dump-json", help="write the fetched jobs for offline re-analysis")
    parser.add_argument("--include-coverage", action="store_true",
                        help="opt in to coverage jobs (sq-piapk's topology; off by default)")
    parser.add_argument("--format", choices=["markdown", "json"], default="markdown")
    parser.add_argument("--top", type=int, default=0, help="only the N slowest legs")
    parser.add_argument("--max-unclassified-pct", type=float, default=None,
                        help="exit 1 if the share of wall-clock in steps NO rule matched "
                             "exceeds this (classifier decay; NOT the OTHER workload share)")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return _self_test()

    if args.from_json:
        try:
            with open(args.from_json, encoding="utf-8") as handle:
                jobs = json.load(handle)
        except (OSError, json.JSONDecodeError) as exc:
            print(f"FAIL-LOUD: cannot read {args.from_json}: {exc}", file=sys.stderr)
            return 2
        if not isinstance(jobs, list):
            print(f"FAIL-LOUD: {args.from_json} is not a list of job objects", file=sys.stderr)
            return 2
    else:
        jobs = fetch_jobs(args.repo, args.workflow or DEFAULT_WORKFLOWS,
                          args.event, args.runs)

    if not jobs:
        print("FAIL-LOUD: empty job population — refusing to report a table over nothing",
              file=sys.stderr)
        return 2

    if args.dump_json:
        with open(args.dump_json, "w", encoding="utf-8") as handle:
            json.dump(jobs, handle)

    stats = aggregate(jobs, include_coverage=args.include_coverage)
    if not stats["legs"]:
        print("FAIL-LOUD: every job was skipped or excluded — nothing measured",
              file=sys.stderr)
        return 2

    if args.format == "json":
        print(json.dumps(stats, indent=2, default=float))
    else:
        print(render_markdown(stats, top=args.top))

    if (args.max_unclassified_pct is not None
            and stats["unclassified_pct"] > args.max_unclassified_pct):
        print(f"\nFAIL: unclassified share {stats['unclassified_pct']:.1f}% exceeds "
              f"--max-unclassified-pct {args.max_unclassified_pct}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
