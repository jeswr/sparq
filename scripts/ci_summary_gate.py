#!/usr/bin/env python3
# ci-summary gate poll loop — the single REQUIRED branch-protection check's brain.
# [FABLE-5] Extracted from the inline bash in .github/workflows/ci-summary.yml so
# the loop is UNIT-TESTABLE (bead sq-90cv4 mandates regression tests over the gate's
# verdict semantics), and extended with the ADAPTIVE SATURATION BUDGET described
# below. The workflow header comment in ci-summary.yml remains the doctrine for WHY
# the gate exists and what it aggregates; this file is the doctrine for HOW the loop
# decides. Invoked by ci-summary.yml with env: REPO, SHA, SELF_RUN_ID (+ GH_TOKEN
# for `gh`).
#
# SEMANTICS (faithful port of the bash — sq-prg4 / sq-ipkku / sq-wjth):
#   * Discovers every check-run on the head commit EXCEPT this gate's own run
#     (matched by an ANCHORED "/runs/<SELF_RUN_ID>(/|$)" test on details_url —
#     strictly tighter than the old substring `contains`, which could in principle
#     match a longer run id sharing the prefix).
#   * pending = check-runs with status != "completed". The settle window is re-armed
#     ONLY by pending work (the sq-ipkku / #997 guard): an injection of
#     already-terminal check-runs can never starve convergence.
#   * A verdict renders only when EVERY discovered sibling is terminal, never before
#     the MIN_POLLS startup floor, and only after SETTLE_POLLS consecutive quiet
#     polls. Verdict: advisory/informational-named checks (whole-word, case-
#     insensitive) are EXCLUDED; a gating check passes iff its conclusion is
#     success/skipped/neutral; an empty stable set passes.
#   * Exhausting the loop budget with pending == 0 renders the REAL verdict on the
#     final all-terminal set (the #997 graceful timeout), never a blind RED.
#
# ADAPTIVE SATURATION BUDGET (sq-90cv4). The gate is a WAITER occupying a runner
# slot; under org-pool saturation many concurrent gates starve the very builds they
# wait on, the base budget expires with siblings still QUEUED, and the old
# unconditional `exit 1` emitted a FALSE RED on a PR whose real legs never got to
# run (the 2026-07-02 congestion collapse; memory: reference-ci-congestion-collapse).
# Runner saturation is a THROUGHPUT signal, not a hang. So: at base-budget
# exhaustion with pending work, the gate now distinguishes:
#   * STILL SETTLING — the repo's Actions queue is deep (queued workflow-run count
#     >= SAT_QUEUE_MIN) OR sibling completions are still landing (completed count
#     rose over the last PROGRESS_WINDOW polls). Keep polling, at the slower
#     SAT_INTERVAL to cut API pressure, re-checking the signal each poll, up to the
#     ABSOLUTE cap MAX_TOTAL_POLLS.
#   * GENUINE HANG — pending work with an idle queue AND no recent completions.
#     RED immediately (the old behaviour, now correctly scoped to real hangs).
# The extension NEVER changes what a verdict says: exit 0 still happens ONLY via
# render_verdict over an all-terminal set (or the stable-empty set), so a genuinely
# failing leg still fails and nothing green is synthesised. The absolute cap +
# the workflow-level timeout-minutes bound the wait — no infinite gate.
#
# FETCH-FAILURE TOLERANCE: the bash `set -e` turned ONE transient `gh api` blip
# into a gate RED. A failed poll is now skipped (state untouched) and only
# MAX_CONSEC_FETCH_FAILURES consecutive failures fail the gate. No data => no
# verdict, so this cannot create a false pass.
#
# Exit-0 paths, exhaustively: (1) render_verdict over a stable-empty set;
# (2) render_verdict over an all-terminal set with zero non-passing GATING checks
# AND every change-based-selection pre-job check green (sq-fmx4u.3: `skipped` is
# satisfied only under a successful selection; a present-but-not-success select
# REDs outright). There is no other `return 0`.
#
# Hermetic tests: scripts/tests/test_ci_summary_gate.py (stdlib-only unittest; no
# network — fetchers are injected). Run: python3 scripts/tests/test_ci_summary_gate.py

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field

ADVISORY_RE = re.compile(r"\b(advisory|informational)\b")
_PASSING = ("success", "skipped", "neutral")
# [FABLE-5] sq-fmx4u.3: the change-based test-selection pre-job (the reusable
# .github/workflows/ci-select.yml job, called from ci.yml + feature-matrix.yml).
# Its check-run name embeds this phrase; scripts/tests/test_ci_select_wiring.py
# pins the workflow job name against this regex so the two cannot drift apart.
SELECT_RE = re.compile(r"change-based test selection")


@dataclass
class Config:
    """Loop tunables. Prod values mirror the previous inline bash (INTERVAL /
    MIN_POLLS / SETTLE_POLLS / BASE_POLLS == the old 110-attempt cap) plus the
    sq-90cv4 adaptive-extension knobs. Tests inject tiny values."""

    self_run_id: str = ""
    interval: int = 20          # seconds between polls (base phase)
    min_polls: int = 3          # startup-race floor: no verdict before 3 polls
    settle_polls: int = 2       # all-terminal must hold this many consecutive polls
    base_polls: int = 110       # base budget: 110 x 20s ~= 37 min (the old hard cap)
    sat_interval: int = 40      # slower poll cadence during the saturation extension
    max_total_polls: int = 155  # absolute cap: 45 extension polls x 40s = +30 min
    sat_queue_min: int = 5      # queued workflow-runs in the repo => saturation
    progress_window: int = 15   # polls over which a completed-count rise = progress
    max_consec_fetch_failures: int = 5
    summary_path: str = field(default_factory=lambda: os.environ.get("GITHUB_STEP_SUMMARY", ""))


class FetchError(RuntimeError):
    """A poll's API fetch failed (transient or otherwise)."""


def is_advisory(name: str) -> bool:
    """Whole-word advisory/informational match (sq-wjth): excludes the standalone
    words (hyphen-/paren-/comma-delimited too) but NOT substrings — notably
    "cargo-deny (advisories, ...)" is plural and GATES."""
    return bool(ADVISORY_RE.search(name.lower()))


def is_select(name: str) -> bool:
    """[FABLE-5] sq-fmx4u.3: is this check-run the change-based test-selection
    pre-job? Matched by the stable phrase in its job name (case-insensitive).
    If the name ever drifts, detection degrades to select_runs == [] — i.e. the
    PRE-selection semantics (skipped is unconditionally satisfied), never a
    false RED; the wiring inspection test pins the name so it does not drift."""
    return bool(SELECT_RE.search(name.lower()))


def is_self(run: dict, self_run_id: str) -> bool:
    """True iff this check-run belongs to THIS gate's workflow run. Anchored to
    /runs/<id>(/|$) so a longer sibling run id sharing the prefix can't match."""
    if not self_run_id:
        return False
    url = run.get("details_url") or run.get("html_url") or ""
    return bool(re.search(rf"/runs/{re.escape(self_run_id)}(/|$)", url))


def _emit(line: str, summary_path: str = "") -> None:
    print(line, flush=True)
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as fh:
            fh.write(line + "\n")


def render_verdict(runs: list[dict], summary_path: str = "") -> int:
    """Shared by the clean-converge, graceful-timeout, and post-extension paths, so
    every path applies IDENTICAL gating semantics. Returns the process exit code.

    SELECTION SEMANTICS ([FABLE-5] sq-fmx4u.3, design §5.3): a `skipped`
    conclusion is satisfied ONLY when the change-based selection pre-job
    (is_select) succeeded — a skip is trustworthy iff the thing that decided to
    skip ran to a successful conclusion. Concretely:
      * every select check-run present must have conclusion == "success";
        anything else (failure, cancelled, skipped, neutral, stale) REDs the
        gate outright, even if every other sibling is green — an unobservable
        selection means the skips on this commit are unattributable (§4.3);
      * with select green (or absent — e.g. a pre-selection sibling set, where
        no skip was produced by selection), `skipped` stays non-failing exactly
        as before. Absent-select degradation is deliberately the PRE-sq-fmx4u.3
        behaviour, never a new failure mode.
    A job that FAILED still fails the gate regardless of selection — selection
    can only ever decide whether a SKIP is satisfied, never mask a failure."""
    total = len(runs)
    if total == 0:
        _emit("ci-summary: no sibling checks to aggregate (stable empty set) — passing.", summary_path)
        return 0
    gating = [r for r in runs if not is_advisory(r.get("name", ""))]
    excluded = total - len(gating)
    # Selection pre-job health — searched over ALL runs (not just gating) so a
    # hypothetical advisory-renamed select could still never green-light a skip.
    select_runs = [r for r in runs if is_select(r.get("name", ""))]
    select_ok = all(r.get("conclusion") == "success" for r in select_runs)
    skipped_ct = sum(1 for r in gating if r.get("conclusion") == "skipped")
    if not select_ok:
        _emit(
            f"### ci-summary: FAILED — the change-based test-selection pre-job did not "
            f"succeed, so the {skipped_ct} skipped gating check(s) on this commit cannot "
            f"be attributed to a sound selection (fail-closed, sq-fmx4u.3 / design §4.3).",
            summary_path,
        )
        for r in select_runs:
            _emit(f"- ✗ {r.get('name')}: {r.get('conclusion') or 'incomplete'}", summary_path)
        print("::error::ci-summary failed — the selection pre-job must conclude success.")
        return 1

    def _satisfied(r: dict) -> bool:
        c = r.get("conclusion")
        if c == "skipped":
            return select_ok  # always True past the gate above; kept explicit so a
            # future refactor that moves this check cannot silently trust a skip.
        return c in _PASSING

    failed = [r for r in gating if not _satisfied(r)]
    if failed:
        _emit(
            f"### ci-summary: FAILED — {len(failed)} non-passing gating check(s) of "
            f"{len(gating)} gating ({excluded} advisory check(s) excluded)",
            summary_path,
        )
        for r in failed:
            _emit(f"- ✗ {r.get('name')}: {r.get('conclusion') or 'incomplete'}", summary_path)
        print("::error::ci-summary failed — see the non-passing gating checks above.")
        return 1
    _emit(
        f"### ci-summary: PASSED — all {len(gating)} gating check(s) green (or skipped/neutral); "
        f"{excluded} advisory check(s) excluded; set stable.",
        summary_path,
    )
    if select_runs:
        _emit(
            f"selection: {len(gating) - skipped_ct} of {len(gating)} gating check(s) ran, "
            f"{skipped_ct} skipped (selection and/or path-filter; selection pre-job succeeded).",
            summary_path,
        )
    return 0


def run_gate(cfg: Config, fetch_runs, fetch_queue_depth, sleep_fn=time.sleep) -> int:
    """The poll loop. `fetch_runs()` -> list of {name,status,conclusion,details_url}
    dicts (raises FetchError on API failure); `fetch_queue_depth()` -> int queued
    workflow-run count for the repo, or None when unknown (API failure — the gate
    then falls back to the progress signal alone). Returns the exit code."""
    prev_names: list[str] | None = None
    stable = 0
    runs: list[dict] = []
    pending = 0
    completed_hist: list[int] = []
    consec_fetch_failures = 0
    extension_started = False

    attempt = 0
    while attempt < cfg.max_total_polls:
        attempt += 1
        try:
            raw = fetch_runs()
        except FetchError as exc:
            consec_fetch_failures += 1
            if consec_fetch_failures >= cfg.max_consec_fetch_failures:
                print(
                    f"::error::ci-summary: {consec_fetch_failures} consecutive check-run "
                    f"fetch failures — cannot observe the sibling set. Last error: {exc}"
                )
                return 1
            print(
                f"attempt {attempt}: check-run fetch failed ({exc}) — skipping this poll "
                f"({consec_fetch_failures}/{cfg.max_consec_fetch_failures} consecutive)."
            )
            sleep_fn(cfg.sat_interval if extension_started else cfg.interval)
            continue
        consec_fetch_failures = 0

        runs = [r for r in raw if not is_self(r, cfg.self_run_id)]
        total = len(runs)
        pending = sum(1 for r in runs if r.get("status") != "completed")
        completed_hist.append(total - pending)
        # Settle is a POST-TERMINAL window re-armed ONLY by pending work (sq-ipkku):
        # already-terminal injections must not starve convergence.
        stable = 0 if pending else stable + 1
        names = sorted({r.get("name", "") for r in runs})
        changed = " (name set changed)" if prev_names is not None and names != prev_names else ""
        prev_names = names
        print(
            f"attempt {attempt}: {total} check-run(s), {pending} running, "
            f"all-terminal stable for {stable}/{cfg.settle_polls} poll(s){changed}",
            flush=True,
        )

        # Clean convergence: everything terminal, held for the settle, past the floor.
        if attempt >= cfg.min_polls and pending == 0 and stable >= cfg.settle_polls:
            return render_verdict(runs, cfg.summary_path)

        # ADAPTIVE SATURATION BUDGET (sq-90cv4): at/after the base budget with work
        # still pending, extend ONLY while the evidence says throughput-starvation
        # (deep queue) or live progress — otherwise it's a genuine hang: RED.
        if attempt >= cfg.base_polls and pending > 0:
            progressing = (
                len(completed_hist) > cfg.progress_window
                and completed_hist[-1] > completed_hist[-1 - cfg.progress_window]
            )
            # [SONNET-4.6] Guard: if fetch_queue_depth() raises (e.g. subprocess.run
            # raises inside the closure before returning None), treat depth as the
            # "unknown" sentinel — never crash the gate, and never grant a saturation
            # extension on depth alone (None → saturated = False, conservative branch).
            try:
                depth = fetch_queue_depth()
            except Exception as exc:
                print(f"  (queue-depth fetch raised {exc!r} — treating depth as unknown)")
                depth = None
            saturated = depth is not None and depth >= cfg.sat_queue_min
            if not (saturated or progressing):
                print(
                    f"::error::ci-summary timed out — {pending} sibling check-run(s) never "
                    f"finished within the base budget, the Actions queue is idle "
                    f"(depth={depth if depth is not None else 'unknown'} < {cfg.sat_queue_min}) "
                    f"and no completions landed in the last {cfg.progress_window} poll(s): "
                    f"genuine hang, not a still-settling set. See the per-poll log above."
                )
                return 1
            if not extension_started:
                extension_started = True
                print(
                    f"::notice::ci-summary base budget reached with {pending} sibling(s) still "
                    f"pending, but the runner pool shows saturation/progress "
                    f"(queued runs={depth if depth is not None else 'unknown'}, "
                    f"progressing={progressing}) — this is a throughput signal, not a hang. "
                    f"Extending the wait (adaptive budget, sq-90cv4) up to poll "
                    f"{cfg.max_total_polls}."
                )
            else:
                print(
                    f"  extension: queued runs={depth if depth is not None else 'unknown'}, "
                    f"progressing={progressing} — still settling."
                )
            if attempt < cfg.max_total_polls:
                sleep_fn(cfg.sat_interval)
            continue

        if attempt < cfg.max_total_polls:
            sleep_fn(cfg.interval)

    # Absolute budget exhausted.
    if pending == 0:
        # The #997 graceful timeout: everything IS terminal, we just never got a
        # full quiet settle — render the real verdict, never a blind RED.
        print(
            "::notice::ci-summary loop budget reached with every sibling check terminal "
            "(the set kept being injected into without a full quiet settle) — rendering "
            "the verdict on the final all-terminal set."
        )
        return render_verdict(runs, cfg.summary_path)
    print(
        f"::error::ci-summary timed out — {pending} sibling check-run(s) never finished "
        f"within the ABSOLUTE budget (base + saturation extension, sq-90cv4). The runner "
        f"pool stayed saturated longer than the extension allows; re-run this gate once "
        f"the queue drains. See the per-poll log above."
    )
    return 1


# ----------------------------- live (gh-backed) wiring -----------------------------


def _gh_json_lines(args: list[str]) -> list[dict]:
    # [SONNET-4.6] Wrap subprocess.run so FileNotFoundError / TimeoutExpired / OSError
    # (e.g. `gh` not on PATH) are converted into FetchError, routing them into the
    # existing bounded-retry / skip-this-poll tolerance in run_gate exactly as a
    # non-zero exit code does — no raw crash, no false pass.
    try:
        proc = subprocess.run(["gh", "api", *args], capture_output=True, text=True)
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
        raise FetchError(f"subprocess raised: {exc}") from exc
    if proc.returncode != 0:
        raise FetchError(proc.stderr.strip()[:300] or f"gh api exited {proc.returncode}")
    out = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if line:
            out.append(json.loads(line))
    return out


def make_fetch_runs(repo: str, sha: str):
    def fetch() -> list[dict]:
        return _gh_json_lines(
            [
                f"repos/{repo}/commits/{sha}/check-runs",
                "--paginate",
                "--jq",
                ".check_runs[] | {name, status, conclusion, details_url, html_url}",
            ]
        )

    return fetch


def make_fetch_queue_depth(repo: str):
    """Queued workflow-run count for the repo — the saturation signal. Returns None
    (unknown) on any failure so a permissions/API blip degrades to progress-only,
    never crashes the gate. Needs `actions: read` on the workflow token."""

    def fetch():
        proc = subprocess.run(
            [
                "gh",
                "api",
                f"repos/{repo}/actions/runs?status=queued&per_page=1",
                "--jq",
                ".total_count",
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            print(f"  (queue-depth fetch failed: {proc.stderr.strip()[:200]} — treating as unknown)")
            return None
        try:
            return int(proc.stdout.strip())
        except ValueError:
            return None

    return fetch


def main() -> int:
    repo = os.environ.get("REPO", "")
    sha = os.environ.get("SHA", "")
    self_run_id = os.environ.get("SELF_RUN_ID", "")
    if not repo or not sha or not self_run_id:
        print("::error::ci-summary: REPO, SHA and SELF_RUN_ID must all be set.")
        return 1
    cfg = Config(self_run_id=self_run_id)
    return run_gate(cfg, make_fetch_runs(repo, sha), make_fetch_queue_depth(repo))


if __name__ == "__main__":
    sys.exit(main())
