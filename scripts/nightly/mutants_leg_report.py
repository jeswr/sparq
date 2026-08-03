#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4820 — MUTATION-RATCHET LEG REPORT. 🤖 SPARQ agent.
#
# WHAT THIS FIXES. The nightly `mutants-nightly-advisory` lane in ci.yml has been
# producing real failures that nothing surfaced. Three independent layers composed to
# hide them, and only the third is fixable here:
#   1. the lane is DECLARED advisory (.github/advisory-registry.json), so it cannot
#      gate — correct by design, and out of scope for this script;
#   2. every leg carries `continue-on-error: true`, so a FAILED leg cannot fail the run.
#      The run-level conclusion is decided by the OTHER jobs — which is how a run with
#      ten red legs came out `cancelled`, the same status a run killed at startup
#      reports (#4820);
#   3. the failures live inside a 51-leg matrix, so they are invisible without
#      expanding it.
# The run-level conclusion is a LOSSY AGGREGATE of the job list. This script therefore
# reads the JOB LIST — the same posture scripts/ci_selection_alarm.py adopted for the
# same reason (#4965: "a failed nightly means a nightly with genuinely-failed JOBS,
# never a nightly GitHub concluded `failure`") — and renders ONE legible verdict:
# how many legs failed, which crates they belong to, and what the lane cost.
#
# WHAT IT IS NOT: a gate. It runs inside the nightly tier only and its job is declared
# in .github/advisory-registry.json, so its red can never block a merge. Its red is a
# NOTICEABLE red: one named job instead of ten needles in a 51-leg matrix.
#
# THE SEEDING ROLLUP is the load-bearing part. A sub-sharded crate (sparq-engine 24
# shards, sparq-reason 3) can only be seeded from a COMPLETE shard set —
# scripts/mutants-gate.py refuses a missing/repeated/truncated shard outright. So one
# capped shard makes the crate's ENTIRE sweep unusable for seeding, and the per-crate
# rollup below reports exactly that: which crates this run could actually seed, and
# which burned their whole budget for nothing.
#
# COST REPORTING exists because the sizing comments in ci.yml explicitly defer to
# evidence that nobody was collecting ("RE-TUNE from the first nightly that completes —
# that run's own timings are the evidence of record"). Per-leg durations and the
# at-cap flag ARE that evidence; they are reported per run rather than committed
# anywhere, so no timing here is a canonical repo number.
#
# Usage:
#   mutants_leg_report.py --jobs-file jobs.jsonl        # jobs from the Actions API
#   mutants_leg_report.py --jobs-file - < jobs.jsonl    # or on stdin
#   mutants_leg_report.py --self-test                   # hermetic fixtures, no I/O
#
# Input is whatever `gh api repos/{owner}/{repo}/actions/runs/{id}/jobs` yields:
# a JSON object with a `jobs` array, a bare JSON array, or the JSONL that
# `--paginate --jq '.jobs[]'` emits. All three are accepted.
#
# Exit codes:
#   0  every matched leg succeeded (or was skipped) — the lane produced a clean signal
#   1  at least one leg FAILED or was cancelled — the lane's signal is incomplete
#   2  the detector itself is broken (unparseable input, or the lane name no longer
#      matches any job) — fail-LOUD, because masking a dead lane is the one thing a
#      report like this must never do
#
# Stdlib only.

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime

# The ci.yml job `name:` whose matrix legs this report covers. GitHub renders a matrix
# leg as "<job name> (<matrix values>)", so this is a PREFIX match. Pinned against the
# live workflow by scripts/tests/test_mutants_leg_report.py — a rename of the lane
# would otherwise silently reduce this script to "no legs matched" forever.
LANE_NAME = "mutation ratchet (cargo-mutants, advisory)"

# The `timeout-minutes` caps the lane's legs actually run under: the matrix default and
# the heavy-crate override. Kept in lockstep with ci.yml's `mutants-nightly-advisory`
# matrix by the same test. Used ONLY to label a leg "hit its cap" — the distinction
# between "this crate's tests are broken" and "this shard was never given enough time",
# which is the difference between a correctness fix and a re-sharding decision.
CAP_MINUTES = (120, 360)
# Half-width of the window around a cap that counts as "it ran until the cap killed
# it". Runner shutdown is not instantaneous, so exact equality would miss most of them.
CAP_TOLERANCE_MINUTES = 5.0

# Conclusions that mean the leg produced a usable verdict.
OK_CONCLUSIONS = frozenset({"success", "neutral", "skipped"})
# Conclusions that mean the leg genuinely failed. Mirrors ci_selection_alarm.py's
# FAILED_CONCLUSIONS so the two detectors cannot disagree about what "failed" means.
FAILED_CONCLUSIONS = frozenset({"failure", "timed_out", "action_required"})

# Matrix values are rendered comma-separated inside the trailing parentheses; the crate
# is the one that names a workspace member and the shard is the one shaped "k/n".
_CRATE_RE = re.compile(r"^sparq-[a-z0-9-]+$")
_SHARD_RE = re.compile(r"^\d+/\d+$")


class ReportError(Exception):
    """Detector-infrastructure failure — always exits 2, never silently exit 0."""


@dataclass(frozen=True)
class Leg:
    name: str
    crate: str
    shard: str
    state: str  # ok | failed | cancelled | running
    conclusion: str
    minutes: float | None

    @property
    def capped(self) -> bool:
        """Did this leg run until its `timeout-minutes` killed it?

        The API does not report a leg's cap, so we ask whether its duration lands in a
        WINDOW around one of the configured caps. A one-sided `>= cap - tolerance` test
        is wrong and was measurably so: a 214-minute leg is far past the 120 cap while
        far short of the 360 one it actually ran under, and calling it capped would
        report a genuine test failure as a sizing problem — the exact confusion this
        flag exists to prevent.
        """
        if self.minutes is None:
            return False
        return any(abs(self.minutes - cap) <= CAP_TOLERANCE_MINUTES for cap in CAP_MINUTES)

    @property
    def label(self) -> str:
        return f"{self.crate} {self.shard}".strip() if self.shard else self.crate


@dataclass
class Report:
    legs: list[Leg] = field(default_factory=list)

    def of(self, state: str) -> list[Leg]:
        return [leg for leg in self.legs if leg.state == state]

    @property
    def runner_hours(self) -> float:
        return sum(leg.minutes or 0.0 for leg in self.legs) / 60.0

    @property
    def crates(self) -> dict[str, list[Leg]]:
        grouped: dict[str, list[Leg]] = {}
        for leg in self.legs:
            grouped.setdefault(leg.crate, []).append(leg)
        return grouped

    @property
    def unusable_crates(self) -> list[str]:
        """Crates this run cannot seed: at least one of their legs produced no verdict.

        A sub-sharded crate needs its COMPLETE shard set to seed a ceiling, so one bad
        shard disqualifies the whole crate — see scripts/mutants-gate.py --seed.
        """
        return sorted(
            crate for crate, legs in self.crates.items()
            if any(leg.state != "ok" for leg in legs)
        )


# ---------------------------------------------------------------------------
# Parsing
# ---------------------------------------------------------------------------

def parse_jobs(text: str) -> list[dict]:
    """Accept the `jobs` envelope, a bare array, or JSONL — whichever gh emitted."""
    stripped = text.strip()
    if not stripped:
        raise ReportError("no job data on the input — the API call produced nothing")
    try:
        data = json.loads(stripped)
    except json.JSONDecodeError:
        jobs = []
        for lineno, line in enumerate(stripped.splitlines(), start=1):
            line = line.strip()
            if not line:
                continue
            try:
                jobs.append(json.loads(line))
            except json.JSONDecodeError as exc:
                raise ReportError(f"line {lineno} of the job list is not JSON: {exc}") from exc
        return jobs
    if isinstance(data, dict):
        data = data.get("jobs", [])
    if not isinstance(data, list):
        raise ReportError("job data is neither a list nor a {'jobs': [...]} envelope")
    return data


def _minutes(job: dict) -> float | None:
    started, completed = job.get("started_at"), job.get("completed_at")
    if not started or not completed:
        return None
    try:
        begin = datetime.fromisoformat(str(started).replace("Z", "+00:00"))
        end = datetime.fromisoformat(str(completed).replace("Z", "+00:00"))
    except ValueError:
        return None
    return max(0.0, (end - begin).total_seconds() / 60.0)


def _split_matrix_suffix(name: str, lane: str) -> tuple[str, str]:
    """Pull (crate, shard) out of the "<lane> (sparq-engine, 9/24, 360)" leg name."""
    suffix = name[len(lane):].strip()
    if suffix.startswith("(") and suffix.endswith(")"):
        suffix = suffix[1:-1]
    crate, shard = "", ""
    for token in (part.strip() for part in suffix.split(",")):
        if not crate and _CRATE_RE.match(token):
            crate = token
        elif not shard and _SHARD_RE.match(token):
            shard = token
    return crate or (suffix or "<unnamed leg>"), shard


def build_report(jobs: list[dict], lane: str = LANE_NAME) -> Report:
    report = Report()
    for job in jobs:
        name = str(job.get("name", ""))
        if not name.startswith(lane):
            continue
        status = str(job.get("status", ""))
        conclusion = str(job.get("conclusion") or "")
        if status != "completed":
            state = "running"
            conclusion = conclusion or status or "unknown"
        elif conclusion in OK_CONCLUSIONS:
            state = "ok"
        elif conclusion in FAILED_CONCLUSIONS:
            state = "failed"
        elif conclusion == "cancelled":
            state = "cancelled"
        else:
            # An unrecognised conclusion is treated as a failure rather than ignored:
            # a leg we cannot classify has not been shown to have produced a verdict.
            state = "failed"
        crate, shard = _split_matrix_suffix(name, lane)
        report.legs.append(
            Leg(name=name, crate=crate, shard=shard, state=state,
                conclusion=conclusion or "unknown", minutes=_minutes(job))
        )
    if not report.legs:
        raise ReportError(
            f"no job on this run has a name starting with {lane!r}. Either the lane did "
            "not run, or its ci.yml `name:` was changed without updating LANE_NAME here."
        )
    return report


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------

def _was(count: int) -> str:
    return "was" if count == 1 else "were"


def _leg_word(count: int) -> str:
    return "leg" if count == 1 else "legs"


def _duration(leg: Leg) -> str:
    if leg.minutes is None:
        return "—"
    text = f"{leg.minutes:.0f} min"
    return f"{text} (at cap)" if leg.capped else text


def render(report: Report) -> str:
    failed, cancelled = report.of("failed"), report.of("cancelled")
    ok, running = report.of("ok"), report.of("running")
    lines = ["## Mutation ratchet — leg report", ""]

    if failed or cancelled:
        lines.append(
            f"**{len(failed)} of {len(report.legs)} legs FAILED"
            + (f", {len(cancelled)} {_was(len(cancelled))} cancelled" if cancelled else "")
            + ".**"
        )
    else:
        lines.append(
            f"**All {len(report.legs)} {_leg_word(len(report.legs))} produced a verdict.**")
    lines += [
        "",
        "The run-level conclusion cannot express this list: every leg carries "
        "`continue-on-error`, so none of these failures can fail the run, and the badge "
        "is decided by the other jobs. Read the counts here, not the badge.",
        "",
        f"| success | failed | cancelled | still running | total |",
        f"|--:|--:|--:|--:|--:|",
        f"| {len(ok)} | {len(failed)} | {len(cancelled)} | {len(running)} | {len(report.legs)} |",
        "",
    ]

    if failed or cancelled or running:
        lines += ["### Legs without a verdict", "",
                  "| leg | conclusion | duration |", "|---|---|---|"]
        for leg in sorted(failed + cancelled + running, key=lambda x: x.label):
            lines.append(f"| `{leg.label}` | {leg.conclusion} | {_duration(leg)} |")
        lines.append("")

    unusable = report.unusable_crates
    if unusable:
        lines += [
            "### Crates this run cannot seed",
            "",
            "A ceiling can only be seeded from a crate's COMPLETE leg set (a sub-sharded "
            "crate needs every shard — `scripts/mutants-gate.py --seed` refuses a "
            "missing or truncated one), so a single bad leg discards the crate's whole "
            "sweep:",
            "",
            ", ".join(f"`{crate}`" for crate in unusable),
            "",
        ]

    ranked = sorted((leg for leg in report.legs if leg.minutes is not None),
                    key=lambda x: x.minutes or 0.0, reverse=True)[:5]
    lines += [
        "### Cost of this run",
        "",
        f"{report.runner_hours:.1f} runner-hours across {len(report.legs)} legs.",
        "",
    ]
    if ranked:
        lines += ["| longest legs | duration |", "|---|---|"]
        lines += [f"| `{leg.label}` | {_duration(leg)} |" for leg in ranked]
        lines.append("")
    capped = [leg for leg in report.legs if leg.capped]
    if capped:
        lines += [
            f"{len(capped)} {_leg_word(len(capped))} ran to a `timeout-minutes` cap. "
            "A capped leg is a "
            "SIZING result, not a test-quality one: it yields nothing and its budget is "
            "spent in full. Re-shard, reduce the per-mutant cost, or lower the cadence — "
            "these durations are this run's evidence of record for that decision.",
            "",
        ]
    return "\n".join(lines)


def annotations(report: Report) -> list[str]:
    """Workflow annotations — what shows up in the run header without expanding a matrix."""
    out = []
    for leg in sorted(report.of("failed"), key=lambda x: x.label):
        out.append(f"::error title=mutation ratchet leg failed::{leg.label} — "
                   f"{leg.conclusion} after {_duration(leg)}")
    for leg in sorted(report.of("cancelled"), key=lambda x: x.label):
        out.append(f"::warning title=mutation ratchet leg cancelled::{leg.label} — "
                   f"cancelled after {_duration(leg)}; no verdict produced")
    return out


# ---------------------------------------------------------------------------
# Self-test (hermetic — a broken detector must red before it is trusted)
# ---------------------------------------------------------------------------

def _job(name: str, conclusion: str, minutes: float | None = 10.0,
         status: str = "completed") -> dict:
    job = {"name": name, "status": status, "conclusion": conclusion}
    if minutes is not None:
        job["started_at"] = "2026-08-01T00:00:00Z"
        hours, mins = divmod(int(minutes), 60)
        job["completed_at"] = f"2026-08-01T{hours:02d}:{mins:02d}:00Z"
    return job


_FIXTURE = [
    {"name": "build + test (stable)", "status": "completed", "conclusion": "failure"},
    _job(f"{LANE_NAME} (sparq-canon)", "success", 40),
    _job(f"{LANE_NAME} (sparq-mpc, 360)", "failure", 214),
    _job(f"{LANE_NAME} (sparq-engine, 1/24, 360)", "success", 100),
    _job(f"{LANE_NAME} (sparq-engine, 9/24, 360)", "failure", 359),
    _job(f"{LANE_NAME} (sparq-engine, 10/24, 360)", "cancelled", 360),
    _job(f"{LANE_NAME} (sparq-reason, 1/3)", "success", 118),
]


def _self_test() -> int:
    failures: list[str] = []

    def check(label: str, got, want) -> None:
        if got != want:
            failures.append(f"{label}: got {got!r}, want {want!r}")

    report = build_report(_FIXTURE)
    # The non-lane job must not be counted, and every lane leg must be.
    check("legs counted", len(report.legs), 6)
    check("failed", sorted(leg.label for leg in report.of("failed")),
          ["sparq-engine 9/24", "sparq-mpc"])
    check("cancelled", [leg.label for leg in report.of("cancelled")], ["sparq-engine 10/24"])
    check("ok", len(report.of("ok")), 3)
    # A crate is unseedable if ANY of its legs missed — sparq-engine has one good shard.
    check("unusable crates", report.unusable_crates, ["sparq-engine", "sparq-mpc"])
    # A leg that ran to its cap is labelled as such; one well under it is not.
    by_label = {leg.label: leg for leg in report.legs}
    check("capped 9/24", by_label["sparq-engine 9/24"].capped, True)
    check("capped 10/24", by_label["sparq-engine 10/24"].capped, True)
    check("not capped canon", by_label["sparq-canon"].capped, False)
    # REGRESSION (found by reproducing the lane end-to-end): a leg far past the 120
    # cap but far short of the 360 one it ran under is NOT capped. A one-sided
    # `>= cap - tolerance` test called this one capped and mislabelled a genuine
    # test failure as a sizing problem.
    check("not capped mpc", by_label["sparq-mpc"].capped, False)
    # sparq-reason's 118-minute leg sits inside the 120-minute cap's tolerance.
    check("capped reason 1/3", by_label["sparq-reason 1/3"].capped, True)
    check("runner hours", round(report.runner_hours, 2), round(1191 / 60, 2))
    check("annotations", len(annotations(report)), 3)
    text = render(report)
    for needle in ("2 of 6 legs FAILED", "sparq-engine 9/24", "Crates this run cannot seed"):
        if needle not in text:
            failures.append(f"rendered report is missing {needle!r}")
    check("verdict", verdict(report), 1)

    # A clean run exits 0 and says so.
    clean = build_report([_job(f"{LANE_NAME} (sparq-canon)", "success", 40)])
    check("clean verdict", verdict(clean), 0)
    check("clean text", "All 1 leg produced a verdict." in render(clean), True)

    # A still-running leg is neither a pass nor silently dropped.
    live = build_report([_job(f"{LANE_NAME} (sparq-canon)", "", None, status="in_progress")])
    check("running state", live.legs[0].state, "running")
    check("running verdict", verdict(live), 1)

    # Every accepted input shape must parse to the same jobs.
    envelope = json.dumps({"jobs": _FIXTURE})
    bare = json.dumps(_FIXTURE)
    jsonl = "\n".join(json.dumps(job) for job in _FIXTURE)
    for label, payload in (("envelope", envelope), ("bare", bare), ("jsonl", jsonl)):
        check(f"parse {label}", len(parse_jobs(payload)), len(_FIXTURE))

    # Fail-LOUD: a renamed lane must not read as "nothing to report".
    for label, payload in (("renamed lane", [_job("some other job", "success")]),
                           ("empty input", [])):
        try:
            build_report(payload)
            failures.append(f"{label}: expected ReportError, got a report")
        except ReportError:
            pass
    try:
        parse_jobs("not json at all")
        failures.append("garbage input: expected ReportError")
    except ReportError:
        pass

    for failure in failures:
        print(f"SELF-TEST FAIL: {failure}", file=sys.stderr)
    print(f"self-test: {'FAILED' if failures else 'ok'} ({len(failures)} failure(s))")
    return 2 if failures else 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def verdict(report: Report) -> int:
    """0 iff every leg produced a verdict; 1 when the lane's signal is incomplete."""
    incomplete = report.of("failed") + report.of("cancelled") + report.of("running")
    return 1 if incomplete else 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jobs-file", help="job list from the Actions API ('-' for stdin)")
    parser.add_argument("--lane-name", default=LANE_NAME,
                        help="ci.yml job name whose matrix legs are reported")
    parser.add_argument("--summary-file", default=os.environ.get("GITHUB_STEP_SUMMARY"),
                        help="append the markdown report here (default: $GITHUB_STEP_SUMMARY)")
    parser.add_argument("--self-test", action="store_true",
                        help="run the hermetic fixtures and exit")
    args = parser.parse_args(argv)

    if args.self_test:
        return _self_test()
    if not args.jobs_file:
        parser.error("--jobs-file is required (or use --self-test)")

    try:
        if args.jobs_file == "-":
            raw = sys.stdin.read()
        else:
            with open(args.jobs_file, encoding="utf-8") as handle:
                raw = handle.read()
        report = build_report(parse_jobs(raw), args.lane_name)
    except (ReportError, OSError) as exc:
        print(f"::error title=mutation ratchet leg report broken::{exc}")
        print(f"mutation ratchet leg report: {exc}", file=sys.stderr)
        return 2

    text = render(report)
    print(text)
    for annotation in annotations(report):
        print(annotation)
    if args.summary_file:
        try:
            with open(args.summary_file, "a", encoding="utf-8") as handle:
                handle.write(text + "\n")
        except OSError as exc:  # a summary we cannot write is not a lane failure
            print(f"::warning::could not write the step summary: {exc}")
    return verdict(report)


if __name__ == "__main__":
    sys.exit(main())
