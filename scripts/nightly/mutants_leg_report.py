#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4820 — MUTATION-RATCHET LEG REPORT. 🤖 SPARQ agent.
#
# WHAT THIS FIXES. The nightly `mutants-nightly-advisory` lane in ci.yml has been
# producing real failures that nothing surfaced. FOUR independent layers composed to
# hide them:
#   1. the lane is DECLARED advisory (.github/advisory-registry.json), so it cannot
#      gate — correct by design, and out of scope for this script;
#   2. the mutation STEP carries `continue-on-error: true` AND swallows cargo-mutants'
#      exit status with `|| …`, so a leg whose sweep died — or whose ratchet `--check`
#      found a regression — still concludes `success` at JOB level. The Actions job
#      list cannot see that class of failure AT ALL, so reading conclusions alone would
#      report exactly the failures that were never hidden and miss the ones that were;
#   3. the job ALSO carries `continue-on-error: true`, so a leg killed at its
#      `timeout-minutes` cannot fail the run either. The run-level conclusion is decided
#      by the OTHER jobs — which is how a run with ten red legs came out `cancelled`,
#      the same status a run killed at startup reports (#4820);
#   4. the failures live inside a 51-leg matrix, so they are invisible without
#      expanding it.
# So this script reads TWO sources, because neither alone is sufficient:
#   • the JOB LIST, because the run-level conclusion is a LOSSY AGGREGATE of it — the
#     same posture scripts/ci_selection_alarm.py adopted for the same reason (#4965: "a
#     failed nightly means a nightly with genuinely-failed JOBS, never a nightly GitHub
#     concluded `failure`"). This is what catches layers 3 and 4: caps, cancellations
#     and infrastructure failures, which DO reach the job conclusion.
#   • the RECORDED PER-LEG OUTCOMES (`--status-dir`), because layer 2 means a job
#     conclusion of `success` proves nothing about the sweep. The mutation step now
#     writes a `leg-status.json` — cargo-mutants' real exit status, whether an
#     outcomes.json was produced, and the ratchet `--check` exit — and uploads it as its
#     own small artifact under `if: always()`, so the record SURVIVES the suppression
#     that hides it from the job conclusion. A leg claiming `success` with no recorded
#     outcome is treated as having produced NO verdict: fail-closed, because "we cannot
#     see it" and "it was fine" must never render the same.
# From those two it renders ONE legible verdict: how many legs failed, which crates
# they belong to, which ceilings regressed, and what the lane cost.
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
#   mutants_leg_report.py --jobs-file jobs.jsonl --status-dir leg-status/
#   mutants_leg_report.py --jobs-file - < jobs.jsonl    # jobs on stdin
#   mutants_leg_report.py --self-test                   # hermetic fixtures, no I/O
#
# --jobs-file is whatever `gh api repos/{owner}/{repo}/actions/runs/{id}/jobs` yields:
# a JSON object with a `jobs` array, a bare JSON array, or the JSONL that
# `--paginate --jq '.jobs[]'` emits. All three are accepted.
# --status-dir is where `gh run download --pattern 'mutants-leg-status-*'` unpacked the
# per-leg `leg-status.json` records; it is searched recursively. Omitting it reports
# from job conclusions ALONE, which cannot see a suppressed in-step failure (layer 2
# above) — the live workflow always passes it.
#
# Exit codes:
#   0  every leg produced a verdict and no seeded ceiling regressed
#   1  at least one leg FAILED, was cancelled, was SKIPPED, is still running, or has no
#      recorded outcome — the lane's signal is incomplete; or a ratchet ceiling regressed
#   2  the detector itself is broken (unparseable input, an unreadable leg-status record,
#      or the lane name no longer matches any job) — fail-LOUD, because masking a dead
#      lane is the one thing a report like this must never do
#
# Stdlib only.

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field, replace
from datetime import datetime
from pathlib import Path

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

# Conclusions that mean the leg RAN TO COMPLETION, and so could have produced a usable
# verdict. `skipped` is deliberately NOT here: a skipped leg never executed, so it has no
# outcomes.json, cannot contribute a shard to a seeding rollup, and must not be counted
# under "produced a verdict" — that reads as good news for a sweep that did not happen.
# It gets its own state below rather than being folded into the failures, because a
# skipped leg is a scheduling fact, not a test-quality one.
OK_CONCLUSIONS = frozenset({"success", "neutral"})
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
class LegStatus:
    """A leg's OWN record of what its mutation step did, written by ci.yml.

    This exists because the job conclusion cannot carry it: the step is
    `continue-on-error` and swallows cargo-mutants' exit status, so a dead sweep and a
    clean one both conclude `success`. The step writes this record and uploads it under
    `if: always()`, which is what makes the outcome survive that suppression.

    `mutants_exit` is recorded but NOT treated as failure on its own: cargo-mutants
    exits nonzero when mutants merely SURVIVE, which is the lane's normal advisory
    result and the very thing it is measuring. What proves a leg produced a verdict is
    `outcomes` — an outcomes.json actually landed — and what proves the verdict was
    clean is `ratchet_exit`.
    """

    crate: str
    shard: str
    outcomes: bool
    mutants_exit: int | None = None
    ratchet_exit: int | None = None

    @property
    def key(self) -> tuple[str, str]:
        return (self.crate, self.shard)


@dataclass(frozen=True)
class Leg:
    name: str
    crate: str
    shard: str
    state: str  # ok | failed | cancelled | skipped | running
    conclusion: str
    minutes: float | None
    # A SEEDED crate whose survivor count now exceeds its committed ceiling. The leg
    # still produced a complete verdict, so it stays `ok` and its crate stays seedable —
    # this is a test-QUALITY red, reported separately from "no verdict".
    ratchet_regressed: bool = False

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
    # Whether the per-leg recorded outcomes were overlaid. Rendered prose depends on it:
    # WITHOUT them these counts are job conclusions alone, which cannot see a suppressed
    # in-step failure, and saying otherwise would be the same overclaim the records fix.
    statuses_applied: bool = False

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
    def regressed(self) -> list[Leg]:
        return [leg for leg in self.legs if leg.ratchet_regressed]

    @property
    def unusable_crates(self) -> list[str]:
        """Crates this run cannot seed: at least one of their legs produced no verdict.

        A sub-sharded crate needs its COMPLETE shard set to seed a ceiling, so one bad
        shard disqualifies the whole crate — see scripts/mutants-gate.py --seed. A
        SKIPPED leg counts against its crate for the same reason a failed one does: it
        contributed no outcomes.json, so the shard set is incomplete either way.
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
        elif conclusion == "skipped":
            state = "skipped"
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


def load_leg_statuses(directory: str) -> list[LegStatus]:
    """Read the `leg-status.json` records `gh run download` unpacked into `directory`.

    One file per leg, each in its own artifact-named subdirectory, so the search is
    recursive. An unreadable or malformed record is a DETECTOR failure (exit 2), not a
    leg failure: these are files this repo writes to a fixed shape, so a bad one means
    the recording side broke — and silently dropping it would restore exactly the
    blindness this argument exists to remove.
    """
    root = Path(directory)
    if not root.is_dir():
        raise ReportError(
            f"--status-dir {directory!r} is not a directory. The report is wired to "
            "consume the per-leg outcomes that survive `continue-on-error`; without "
            "them it cannot see a suppressed in-step failure at all."
        )
    statuses: list[LegStatus] = []
    for path in sorted(root.rglob("leg-status.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            raise ReportError(f"{path}: recorded leg outcome is unreadable: {exc}") from exc
        if not isinstance(data, dict) or "crate" not in data:
            raise ReportError(f"{path}: not a leg-status record (no 'crate' key)")
        statuses.append(LegStatus(
            crate=str(data.get("crate", "")),
            shard=str(data.get("shard", "")),
            outcomes=bool(data.get("outcomes", False)),
            mutants_exit=data.get("mutants_exit"),
            ratchet_exit=data.get("ratchet_exit"),
        ))
    return statuses


def apply_leg_statuses(report: Report, statuses: list[LegStatus]) -> None:
    """Overlay each leg's OWN recorded outcome onto its job conclusion, in place.

    Only legs the job list calls `ok` are touched. A leg that already failed, was
    cancelled, was skipped or is still running is left alone: its job conclusion is
    already telling the truth, and such a leg is EXPECTED to have recorded nothing
    (a runner killed at `timeout-minutes` never reaches the step that writes the record).

    The rule for an `ok` leg is fail-closed — it keeps that state only if it recorded an
    outcomes.json. No record at all means the mutation step did not reach its end, which
    `continue-on-error` then reported as `success`; that is the precise failure class the
    job list cannot see, so treating it as a pass would defeat the whole mechanism.
    """
    report.statuses_applied = True
    by_key = {status.key: status for status in statuses}
    for index, leg in enumerate(report.legs):
        if leg.state != "ok":
            continue
        status = by_key.get((leg.crate, leg.shard))
        if status is None:
            report.legs[index] = replace(
                leg, state="failed",
                conclusion=f"{leg.conclusion} (no outcome recorded — the mutation step "
                           "never reached its end; `continue-on-error` reported it as "
                           "success)")
        elif not status.outcomes:
            report.legs[index] = replace(
                leg, state="failed",
                conclusion=f"{leg.conclusion} (no outcomes.json; cargo-mutants exit "
                           f"{status.mutants_exit} — suppressed by `continue-on-error`)")
        elif status.ratchet_exit:
            report.legs[index] = replace(leg, ratchet_regressed=True)


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
    ok, running, skipped = report.of("ok"), report.of("running"), report.of("skipped")
    incomplete = failed + cancelled + skipped + running
    lines = ["## Mutation ratchet — leg report", ""]

    if incomplete:
        extra = []
        if cancelled:
            extra.append(f"{len(cancelled)} {_was(len(cancelled))} cancelled")
        if skipped:
            extra.append(f"{len(skipped)} {_was(len(skipped))} skipped")
        if running:
            extra.append(f"{len(running)} {_was(len(running))} still running")
        lines.append(
            f"**{len(failed)} of {len(report.legs)} legs FAILED"
            + (", " + ", ".join(extra) if extra else "")
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
        # Do not claim to see through the in-step suppression unless we actually did.
        ("Nor can a leg's own conclusion be trusted alone — the mutation step is "
         "`continue-on-error` and swallows cargo-mutants' exit status, so a leg whose "
         "sweep died still concludes `success`. The counts below fold in each leg's "
         "recorded outcome, which survives that."
         if report.statuses_applied else
         "**These counts are job conclusions ALONE.** The mutation step is "
         "`continue-on-error` and swallows cargo-mutants' exit status, so a leg whose "
         "sweep died still concludes `success` and is counted here as a success. Run "
         "with `--status-dir` to fold in the per-leg recorded outcomes."),
        "",
        f"| success | failed | cancelled | skipped | still running | total |",
        f"|--:|--:|--:|--:|--:|--:|",
        f"| {len(ok)} | {len(failed)} | {len(cancelled)} | {len(skipped)} "
        f"| {len(running)} | {len(report.legs)} |",
        "",
    ]

    if incomplete:
        lines += ["### Legs without a verdict", "",
                  "| leg | conclusion | duration |", "|---|---|---|"]
        for leg in sorted(incomplete, key=lambda x: x.label):
            lines.append(f"| `{leg.label}` | {leg.conclusion} | {_duration(leg)} |")
        lines.append("")

    regressed = report.regressed
    if regressed:
        lines += [
            "### Ratchet regressions",
            "",
            "These legs completed and their crate is still seedable, but "
            "`scripts/mutants-gate.py --check` found MORE surviving mutants than the "
            "committed ceiling — a test-quality regression the step summary reported "
            "into a `continue-on-error` step nobody reads:",
            "",
            ", ".join(f"`{leg.label}`" for leg in sorted(regressed, key=lambda x: x.label)),
            "",
        ]

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
    for leg in sorted(report.of("skipped"), key=lambda x: x.label):
        out.append(f"::warning title=mutation ratchet leg skipped::{leg.label} — "
                   "skipped; it never ran, so it produced no outcomes and its crate "
                   "cannot be seeded from this run")
    for leg in sorted(report.regressed, key=lambda x: x.label):
        out.append(f"::error title=mutation ratchet regression::{leg.label} — more "
                   "surviving mutants than its committed ceiling (mutants-gate --check)")
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

    # A SKIPPED leg never executed, so it produced no outcomes.json — counting it under
    # "success" would report a sweep that did not happen as good news and would leave its
    # crate seedable from a shard set that is missing a shard.
    skipped = build_report([_job(f"{LANE_NAME} (sparq-canon)", "success", 40),
                            _job(f"{LANE_NAME} (sparq-parse)", "skipped", 1)])
    check("skipped state", skipped.of("skipped")[0].label, "sparq-parse")
    check("skipped not ok", len(skipped.of("ok")), 1)
    check("skipped verdict", verdict(skipped), 1)
    check("skipped unseedable", skipped.unusable_crates, ["sparq-parse"])
    check("skipped text", "1 was skipped" in render(skipped), True)
    check("skipped annotated", len(annotations(skipped)), 1)

    # THE SUPPRESSED-FAILURE CASE — the shape the live lane actually produces, and the
    # one job conclusions alone cannot express. The mutation step is `continue-on-error`
    # and swallows cargo-mutants' exit status with `|| …`, so ALL THREE of these legs
    # conclude `success` in the Actions job list. Only their recorded outcomes separate
    # them. A fixture that manufactured `conclusion: failure` here would test a path the
    # suppression means never happens.
    suppressed = build_report([
        _job(f"{LANE_NAME} (sparq-canon)", "success", 40),        # genuinely clean
        _job(f"{LANE_NAME} (sparq-mpc, 360)", "success", 214),    # sweep died mid-run
        _job(f"{LANE_NAME} (sparq-parse)", "success", 51),        # ceiling regressed
        _job(f"{LANE_NAME} (sparq-reason, 1/3)", "success", 118),  # recorded nothing
    ])
    check("suppressed all ok before overlay", len(suppressed.of("ok")), 4)
    check("suppressed verdict before overlay", verdict(suppressed), 0)
    # Blind is allowed; blind while claiming otherwise is not.
    check("blind report says it is blind",
          "These counts are job conclusions ALONE." in render(suppressed), True)
    apply_leg_statuses(suppressed, [
        LegStatus("sparq-canon", "", outcomes=True, mutants_exit=0, ratchet_exit=0),
        # cargo-mutants exit 1 and no outcomes.json: nothing to check, nothing to seed.
        LegStatus("sparq-mpc", "", outcomes=False, mutants_exit=1, ratchet_exit=0),
        # A COMPLETE sweep whose ratchet --check exited 1 — more survivors than the
        # committed ceiling. The verdict exists, so the crate stays seedable.
        LegStatus("sparq-parse", "", outcomes=True, mutants_exit=2, ratchet_exit=1),
        # sparq-reason 1/3 deliberately absent — the step never reached the record.
    ])
    by_label = {leg.label: leg for leg in suppressed.legs}
    check("suppressed canon still ok", by_label["sparq-canon"].state, "ok")
    check("suppressed mpc failed", by_label["sparq-mpc"].state, "failed")
    check("suppressed unrecorded failed", by_label["sparq-reason 1/3"].state, "failed")
    # A regression is red but NOT a missing verdict: the sweep completed.
    check("suppressed parse still ok", by_label["sparq-parse"].state, "ok")
    check("suppressed parse regressed", by_label["sparq-parse"].ratchet_regressed, True)
    check("suppressed verdict", verdict(suppressed), 1)
    check("suppressed unseedable", suppressed.unusable_crates,
          ["sparq-mpc", "sparq-reason"])
    suppressed_text = render(suppressed)
    for needle in ("2 of 4 legs FAILED", "Ratchet regressions", "no outcomes.json",
                   "no outcome recorded"):
        if needle not in suppressed_text:
            failures.append(f"suppressed-failure report is missing {needle!r}")
    # cargo-mutants exiting nonzero is NOT itself a failure — it exits nonzero whenever
    # mutants merely survive, which is the lane's normal result. Only a missing
    # outcomes.json or a failed ratchet check reds a leg.
    survivors = build_report([_job(f"{LANE_NAME} (sparq-canon)", "success", 40)])
    apply_leg_statuses(survivors, [
        LegStatus("sparq-canon", "", outcomes=True, mutants_exit=2, ratchet_exit=0)])
    check("survivors are not a failure", verdict(survivors), 0)

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
    """0 iff every leg produced a clean verdict; 1 when it did not.

    A SKIPPED leg counts as incomplete: it never executed, so it produced no outcomes and
    cannot support a seeding rollup. A ratchet regression counts too — that leg's verdict
    is complete, and it is red.
    """
    incomplete = (report.of("failed") + report.of("cancelled")
                  + report.of("skipped") + report.of("running"))
    return 1 if incomplete or report.regressed else 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jobs-file", help="job list from the Actions API ('-' for stdin)")
    parser.add_argument("--status-dir",
                        help="directory of downloaded per-leg leg-status.json records; "
                             "these survive the step's `continue-on-error`, which the "
                             "job conclusions do not")
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
        if args.status_dir:
            apply_leg_statuses(report, load_leg_statuses(args.status_dir))
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
