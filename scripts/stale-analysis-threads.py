#!/usr/bin/env python3
"""Resolve review threads whose filing analysis is no longer running."""

# [OPUS-5] 🤖 SPARQ agent — issue #4542.
#
# THE DEAD END THIS ENDS
# ----------------------
# The `main` ruleset sets `required_review_thread_resolution: true`
# (docs/branch-protection.md § Required reviews). Review threads here are filed by an
# ANALYSIS far more often than by a human: in #4542's census of 119 open PRs (2026-07-27)
# EVERY unresolved review thread in the repository was CodeQL's `github-advanced-security`.
# An analysis normally supersedes its own threads: it re-runs on the next head, the
# finding is gone, the thread is outdated and can be resolved.
#
# Disable that analysis and the second half of the loop disappears while the first half's
# output stays on the PR forever. Measured 2026-07-27 (#4542): `codeql.yml` was
# `disabled_manually` with a last run of 2026-07-18T14:01Z, and PR #3451 sat BLOCKED with
# ten unresolved `github-advanced-security` threads, armed for auto-merge for 33 hours,
# every individual CI signal green. Nothing reported a problem, because from every other
# lane's point of view nothing WAS wrong.
#
# #4534's stuck-arm sweep (scripts/rearm-sweeper.py) already makes that shape VISIBLE:
# such a PR is classified `blocked-threads`, disarmed, parked, and given a receipt naming
# `threads-resolved` as the condition that un-parks it. Visible is not the same as
# exitable — nothing in the repository could satisfy `threads-resolved`. This script is
# the exit, and it is deliberately the CLASS fix (issue #4542 option 3) rather than a
# one-off cleanup of #3451: the mechanism re-arms itself the moment ANY thread-opening
# analysis is enabled and later disabled.
#
# THE ONE INVARIANT EVERYTHING ELSE RESTS ON
# ------------------------------------------
#     A thread is resolved ONLY when the analysis that filed it is PROVABLY NOT RUNNING.
#
# "Provably" needs TWO reads to agree, because DISABLING A WORKFLOW ONLY STOPS FUTURE
# TRIGGERS: a run that started (or was queued) before the switch flipped keeps going, and
# that run still owns its threads and can still supersede them. So the sweep is authorised
# only when the Actions API reports that workflow's `state` as one of the disabled states
# AND its run census reports every run it has ever started as finished. `active`, or a
# disabled workflow with an unfinished run, means the analysis CAN supersede its own
# thread, so this script must do nothing and leave the thread to the tool that owns it.
# Anything else — a 404, an unreadable response, an unrecognised state, a run census that
# cannot be read or does not add up — is UNKNOWN and also does nothing. There is no
# configuration in which an ambiguous read causes a resolve; that is the self-test's
# headline guard (its "THE HEADLINE GUARD" block: a LIVE analysis must not even enumerate
# PRs, and inverting the DEAD filter reds it).
#
# The other rails, each pinned by the self-test:
#   * EVERY comment in the thread must be authored by a declared dead analysis. One human
#     reply — or one comment from a live/undeclared author — disqualifies the thread. A
#     truncated comment page means the authors are not fully known, which disqualifies it
#     too (fail open, never guess).
#   * A PR is swept WHOLE or not at all. If its review threads overflow the enumeration
#     page, the set of threads is not fully known, so every thread on that PR is skipped
#     (counted, never resolved) rather than half-sweeping a PR that stays blocked anyway.
#   * Only threads that are still unresolved are touched; resolving is idempotent, so a
#     second tick finds nothing and posts nothing.
#   * Every mutation is bounded per run (`--max-resolves`) and every skipped thread is
#     still classified and COUNTED, so the census is total over what was read.
#   * Every resolve is receipted: one comment per PR per analysis, naming that analysis,
#     the workflow state that authorised the sweep, and every thread it resolved. A
#     resolve that lands with no receipt is a FAILURE (exit 1), never a silent success.
#
# WHAT IT IS NOT. Resolving a THREAD is not dismissing a FINDING. The ruleset's
# `code_scanning` rule (CodeQL alerts, `errors_and_warnings`) is a separate gate and is
# untouched by this script; re-enabling the workflow re-files anything still live on the
# next analysis. This script only removes a merge block that no tool can any longer clear.
#
# NOT A GATE: schedule / workflow_dispatch only (.github/workflows/stale-analysis-threads.yml),
# never on a PR head, so it registers no check-run and cannot enter the `ci-summary / gate`
# verdict. Its hermetic `--self-test` runs on every PR from docs-quality.yml's quick-gates.

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

sys.path.insert(0, str(Path(__file__).resolve().parent))
try:  # pragma: no cover - exercised by the degraded path, not by the self-test
    import gh_retry
except ModuleNotFoundError:  # pragma: no cover
    # Same policy as scripts/rearm-sweeper.py's degraded mode: losing the resilience
    # helper costs RETRIES, never correctness, and a read failure then fails LOUD.
    gh_retry = None

PROGRAM = "stale-analysis-threads"
MARKER = "> 🤖 SPARQ agent"


# --------------------------------------------------------------------------------------
# The declaration. Widening this table is the whole blast radius of this script, so it is
# a reviewable literal rather than anything inferred at runtime.
# --------------------------------------------------------------------------------------
@dataclass(frozen=True)
class Analysis:
    """A thread-opening analysis, bound to the workflow whose runs produce it."""

    login: str      # the comment author login these threads carry
    workflow: str   # the workflow FILE whose `state` decides whether it still runs
    name: str       # what to call it in the receipt


ANALYSES: tuple[Analysis, ...] = (
    Analysis("github-advanced-security", "codeql.yml", "CodeQL code scanning"),
)

# Actions API workflow states. `active` is the ONLY state that means "can still supersede
# its own threads"; the disabled family is a NECESSARY, not sufficient, condition for a
# resolve (the run census below is the other half).
STATE_ACTIVE = "active"
DISABLED_STATES = frozenset({"disabled_manually", "disabled_inactivity", "disabled_fork"})

# The only run `status` that means a run has finished. Every other value the Actions API
# reports (queued / in_progress / waiting / requested / pending, plus anything GitHub adds
# later) means the run may still be writing threads, so the census counts runs by asking
# for the COMPLETED total and the total total: unfinished = total - completed. That
# subtraction is closed under new statuses, which a per-status allow-list would not be.
RUN_STATUS_COMPLETED = "completed"

LIVE = "live"
DEAD = "dead"
UNKNOWN = "unknown"

# Thread classifications. TOTAL over every unresolved thread read, so the census cannot
# hide a thread behind an unnamed branch.
SKIP_RESOLVED = "already-resolved"
SKIP_NO_COMMENTS = "no-comments"
SKIP_AUTHORS_TRUNCATED = "authors-truncated"
SKIP_PR_THREADS_TRUNCATED = "pr-threads-truncated"
SKIP_OTHER_AUTHOR = "not-a-dead-analysis"
ELIGIBLE = "eligible"

DEFAULT_MAX_RESOLVES = 20
DEFAULT_MAX_PAGES = 20
PR_PAGE_SIZE = 20

OPEN_PR_THREADS_QUERY = """query($owner:String!,$name:String!,$cursor:String){
  repository(owner:$owner,name:$name){
    pullRequests(states:OPEN,first:20,after:$cursor,orderBy:{field:UPDATED_AT,direction:DESC}){
      pageInfo{hasNextPage endCursor}
      nodes{
        number
        reviewThreads(first:50){
          pageInfo{hasNextPage}
          nodes{
            id
            isResolved
            isOutdated
            path
            line
            originalLine
            comments(first:10){
              pageInfo{hasNextPage}
              nodes{author{login}}
            }
          }
        }
      }
    }
  }
}"""

RESOLVE_MUTATION = """mutation($threadId:ID!){
  resolveReviewThread(input:{threadId:$threadId}){thread{id isResolved}}
}"""


class GhError(RuntimeError):
    """A ``gh`` invocation failed."""


if gh_retry is not None:  # pragma: no cover - trivial alias
    GhTransientExhausted = gh_retry.GhTransientExhausted
else:  # pragma: no cover

    class GhTransientExhausted(RuntimeError):
        """Never raised without gh_retry: with no retries there is nothing to exhaust."""


def run_gh(argv: list[str]) -> str:
    """One-shot MUTATION path. Never retried — a resolve/comment is not idempotent."""
    result = subprocess.run(["gh", *argv], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh failure"
        raise GhError(f"gh {' '.join(argv[:3])} failed: {detail}")
    return result.stdout


def run_gh_read(argv: list[str]) -> str:
    """Idempotent READ path: bounded transient-only retries when gh_retry is present."""
    if gh_retry is None:  # pragma: no cover - degraded mode, see the header
        return run_gh(argv)
    try:
        return gh_retry.run_gh_read(argv)
    except gh_retry.GhFatalError as error:
        raise GhError(str(error)) from error


# --------------------------------------------------------------------------------------
# Pure decisions. No I/O, no clock, no gh handle — every one of these is a fixture table
# in the self-test.
# --------------------------------------------------------------------------------------
def normalize_login(login: object) -> str:
    """`GitHub-Advanced-Security[bot]` -> `github-advanced-security`.

    GraphQL reports a Bot author's login with and without the `[bot]` suffix depending on
    the node type, and login comparison is case-insensitive on GitHub.
    """
    text = str(login or "").strip().lower()
    if text.endswith("[bot]"):
        text = text[: -len("[bot]")]
    return text


def workflow_verdict(state: object) -> str:
    """LIVE / DEAD / UNKNOWN from an Actions API workflow `state`.

    UNKNOWN is the fail-open bucket and covers everything not explicitly declared: an
    unreadable response, a 404, and any state GitHub adds after this was written. Only
    DEAD authorises a resolve.
    """
    text = str(state or "").strip().lower()
    if text == STATE_ACTIVE:
        return LIVE
    if text in DISABLED_STATES:
        return DEAD
    return UNKNOWN


def parse_run_total(raw: object) -> int | None:
    """`total_count` off one workflow-runs page, or None if the page is unreadable.

    None is the FAIL-CLOSED answer and is decision-bearing: a census we cannot read is not
    evidence that nothing is running. `bool` is rejected explicitly because it is an `int`.
    """
    if not isinstance(raw, dict):
        return None
    total = raw.get("total_count")
    if isinstance(total, bool) or not isinstance(total, int) or total < 0:
        return None
    return total


def newest_run(raw: object) -> dict | None:
    """The newest run on a workflow-runs page (they come newest-first), if it is readable."""
    runs = raw.get("workflow_runs") if isinstance(raw, dict) else None
    if not isinstance(runs, list) or not runs or not isinstance(runs[0], dict):
        return None
    return runs[0]


def parse_run_created(raw: object) -> str | None:
    """The newest run's `created_at`. Receipt evidence only — NEVER decision-bearing."""
    run = newest_run(raw)
    created = run.get("created_at") if run else None
    return str(created) if created else None


def parse_run_status(raw: object) -> str | None:
    """The newest run's `status`, normalized. None = not readable, which is FAIL-CLOSED."""
    run = newest_run(raw)
    if run is None:
        return None
    status = run.get("status")
    if not isinstance(status, str) or not status.strip():
        return None
    return status.strip().lower()


@dataclass(frozen=True)
class Thread:
    """One review thread, reduced to what the decision needs."""

    id: str
    pr: int
    is_resolved: bool
    is_outdated: bool
    path: str
    line: int | None
    authors: tuple[str, ...]          # normalized comment author logins, in order
    authors_truncated: bool           # the comment page overflowed: authors NOT fully known

    def location(self) -> str:
        path = self.path or "(file-level)"
        return f"{path}:{self.line}" if self.line else path


def classify_thread(thread: Thread, dead_logins: frozenset[str]) -> str:
    """Why this thread is (not) sweepable. Total: every thread gets exactly one label."""
    if thread.is_resolved:
        return SKIP_RESOLVED
    if not thread.authors:
        # No readable author is not evidence of a dead analysis.
        return SKIP_NO_COMMENTS
    if thread.authors_truncated:
        # A hidden later comment could be a human's. Refuse rather than guess.
        return SKIP_AUTHORS_TRUNCATED
    if not all(author in dead_logins for author in thread.authors):
        return SKIP_OTHER_AUTHOR
    return ELIGIBLE


def parse_threads(pr_node: object) -> tuple[list[Thread], bool]:
    """(threads, threads_truncated) for one PR node of the enumeration query."""
    if not isinstance(pr_node, dict):
        return [], False
    number = pr_node.get("number")
    if not isinstance(number, int):
        return [], False
    review_threads = pr_node.get("reviewThreads") or {}
    truncated = bool((review_threads.get("pageInfo") or {}).get("hasNextPage"))
    nodes = review_threads.get("nodes")
    threads: list[Thread] = []
    for node in nodes if isinstance(nodes, list) else []:
        if not isinstance(node, dict) or not node.get("id"):
            continue
        comments = node.get("comments") or {}
        comment_nodes = comments.get("nodes")
        authors: list[str] = []
        for comment in comment_nodes if isinstance(comment_nodes, list) else []:
            if not isinstance(comment, dict):
                continue
            author = comment.get("author")
            login = normalize_login(author.get("login") if isinstance(author, dict) else None)
            # An author GitHub cannot render (deleted account) reads as "" and is NOT a
            # declared analysis, so it disqualifies the thread via SKIP_OTHER_AUTHOR.
            authors.append(login)
        line = node.get("line")
        if not isinstance(line, int):
            line = node.get("originalLine")
        threads.append(
            Thread(
                id=str(node["id"]),
                pr=number,
                is_resolved=bool(node.get("isResolved")),
                is_outdated=bool(node.get("isOutdated")),
                path=str(node.get("path") or ""),
                line=line if isinstance(line, int) else None,
                authors=tuple(authors),
                authors_truncated=bool((comments.get("pageInfo") or {}).get("hasNextPage")),
            )
        )
    return threads, truncated


@dataclass(frozen=True)
class AnalysisState:
    """What the Actions API said about one declared analysis, and when."""

    analysis: Analysis
    verdict: str
    state: str          # the raw `state` string, or the reason it is unknown
    last_run: str | None = None


def receipt_comment(state: AnalysisState, threads: list[Thread]) -> str:
    """The per-PR receipt. Names the authority for the sweep and every thread it touched."""
    last_run = f", last run {state.last_run}" if state.last_run else ""
    rows = "\n".join(f"| `{thread.location()}` |" for thread in threads)
    return (
        f"{MARKER}\n\n"
        f"**Stale-analysis review threads resolved — {len(threads)}.**\n\n"
        f"`{state.analysis.login}` ({state.analysis.name}) filed the threads below, and the "
        f"analysis that files them is not running: workflow `{state.analysis.workflow}` "
        f"reports state `{state.state}`{last_run}. A thread from an analysis that no longer "
        f"runs can never be superseded by a re-scan, so under the `main` ruleset's "
        f"`required_review_thread_resolution` it is a permanent merge block with no "
        f"automated exit (sparq-org/sparq#4542).\n\n"
        f"| resolved thread |\n|---|\n{rows}\n\n"
        f"This resolves the THREAD, not the finding. The code-scanning alert gate (the "
        f"ruleset's `code_scanning` rule) is untouched, and re-enabling "
        f"`{state.analysis.workflow}` re-files anything still live on its next analysis. "
        f"Un-resolve any thread you want kept open.\n\n"
        f"Swept by `.github/workflows/stale-analysis-threads.yml` "
        f"(`scripts/{Path(__file__).name}`)."
    )


# --------------------------------------------------------------------------------------
# The sweep.
# --------------------------------------------------------------------------------------
@dataclass
class Sweeper:
    repo: str
    gh: Callable[[list[str]], str] = run_gh
    gh_read: Callable[[list[str]], str] | None = None
    log: Callable[[str], None] = print
    dry_run: bool = False
    max_resolves: int = DEFAULT_MAX_RESOLVES
    max_pages: int = DEFAULT_MAX_PAGES
    analyses: tuple[Analysis, ...] = ANALYSES

    counts: dict[str, int] = field(default_factory=dict)
    failures: list[str] = field(default_factory=list)
    resolved: int = 0
    deferred: int = 0
    truncated_enumeration: bool = False
    truncated_prs: set[int] = field(default_factory=set)

    def __post_init__(self) -> None:
        try:
            self.owner, self.name = self.repo.split("/", 1)
        except ValueError as error:
            raise ValueError("repo must be OWNER/REPOSITORY") from error
        if not self.owner or not self.name or "/" in self.name:
            raise ValueError("repo must be OWNER/REPOSITORY")
        if self.gh_read is None:
            self.gh_read = self.gh

    # ---- reads -------------------------------------------------------------------------

    def _read_json(self, argv: list[str]) -> object:
        assert self.gh_read is not None
        return json.loads(self.gh_read(argv))

    def analysis_state(self, analysis: Analysis) -> AnalysisState:
        """Is this analysis still running? The whole authority for every mutation below.

        DEAD requires BOTH halves: a disabled workflow `state` (no NEW run can start) and a
        run census proving no started run is still going (nothing OLD is still writing).
        """
        try:
            raw = self._read_json(
                ["api", f"repos/{self.owner}/{self.name}/actions/workflows/{analysis.workflow}"]
            )
        except GhError as error:
            # Unreadable is UNKNOWN, and UNKNOWN never resolves anything.
            return AnalysisState(analysis, UNKNOWN, f"unreadable: {error}")
        except json.JSONDecodeError as error:
            return AnalysisState(analysis, UNKNOWN, f"unparseable: {error}")
        state = raw.get("state") if isinstance(raw, dict) else None
        text = str(state or "unreported")
        verdict = workflow_verdict(state)
        if verdict != DEAD:
            return AnalysisState(analysis, verdict, text)
        census = self.run_census(analysis)
        if census is None:
            # A census we could not read is not proof of quiescence. Fail closed.
            return AnalysisState(analysis, UNKNOWN, f"{text}, run census unreadable")
        unfinished, last_run = census
        if unfinished:
            # Disabled, but a run started before the switch flipped is still going and
            # still owns its threads — it can supersede them itself.
            return AnalysisState(
                analysis, LIVE, f"{text}, {unfinished} run(s) not finished", last_run
            )
        return AnalysisState(analysis, DEAD, text, last_run)

    def run_census(self, analysis: Analysis) -> tuple[int, str | None] | None:
        """(unfinished run count, newest run's `created_at`), or None if unreadable.

        TWO independent signals, and either one alone blocks the sweep:

        * the COUNTS — every run this workflow ever started (`total_count`) minus the ones
          reported `completed`. Counted server-side, so no page bound can hide a run, and
          the subtraction is closed under statuses GitHub adds later;
        * the NEWEST run's own `status`, read off the page. `total_count` is an aggregate
          this script cannot verify, so it is not trusted alone: a run that is still going
          is overwhelmingly the newest one, and its status is a fact on the page.

        The two reads cannot race a new run into existence because the caller has already
        established that the workflow is disabled. Any unreadable, malformed or incoherent
        answer (unparseable page, missing `total_count`, an unreadable status on a run that
        exists, completed > total) returns None, which the caller treats as UNKNOWN.
        """
        every = self._run_page(analysis)
        if every is None:
            return None
        completed = self._run_page(analysis, status=RUN_STATUS_COMPLETED)
        if completed is None:
            return None
        total, last_run, newest_status = every
        done, _, _ = completed
        if done > total:
            return None
        unfinished = total - done
        if total:
            if newest_status is None:
                return None  # a run exists but its status is unreadable: fail closed.
            if newest_status != RUN_STATUS_COMPLETED:
                unfinished = max(unfinished, 1)
        return unfinished, last_run

    def _run_page(
        self, analysis: Analysis, status: str | None = None
    ) -> tuple[int, str | None, str | None] | None:
        """One runs page as (total_count, newest created_at, newest status). None = unreadable."""
        query = "per_page=1" + (f"&status={status}" if status else "")
        try:
            raw = self._read_json(
                [
                    "api",
                    f"repos/{self.owner}/{self.name}/actions/workflows/"
                    f"{analysis.workflow}/runs?{query}",
                ]
            )
        except (GhError, json.JSONDecodeError):
            return None
        total = parse_run_total(raw)
        if total is None:
            return None
        return total, parse_run_created(raw), parse_run_status(raw)

    def open_pr_threads(self) -> dict[int, list[Thread]]:
        """Every open PR's review threads, bounded by `max_pages`."""
        by_pr: dict[int, list[Thread]] = {}
        cursor: str | None = None
        for _ in range(self.max_pages):
            argv = [
                "api", "graphql",
                "-f", f"query={OPEN_PR_THREADS_QUERY}",
                "-f", f"owner={self.owner}",
                "-f", f"name={self.name}",
            ]
            if cursor:
                argv += ["-f", f"cursor={cursor}"]
            response = self._read_json(argv)
            if not isinstance(response, dict) or response.get("errors"):
                raise GhError("GraphQL returned errors while enumerating review threads")
            repository = (response.get("data") or {}).get("repository") or {}
            pulls = repository.get("pullRequests") or {}
            for node in pulls.get("nodes") or []:
                threads, truncated = parse_threads(node)
                if truncated:
                    # More threads than one page: the sweep is partial, and says so.
                    self.truncated_enumeration = True
                if threads:
                    by_pr.setdefault(threads[0].pr, []).extend(threads)
                    if truncated:
                        # ...and this PR's thread set is not fully known, so run() must
                        # sweep NONE of it. Nested review-thread pagination is not
                        # implemented, so the tail is unreachable, not merely deferred.
                        self.truncated_prs.add(threads[0].pr)
            page = pulls.get("pageInfo") or {}
            if not page.get("hasNextPage"):
                return by_pr
            cursor = str(page.get("endCursor") or "")
            if not cursor:
                break
        self.truncated_enumeration = True
        self.log(
            f"::warning title={PROGRAM} enumeration truncated::stopped after "
            f"{self.max_pages} page(s) of {PR_PAGE_SIZE} open PR(s) — the counts below "
            "cover only what was read."
        )
        return by_pr

    # ---- mutations ---------------------------------------------------------------------

    def resolve_thread(self, thread: Thread) -> bool:
        payload = self.gh(
            [
                "api", "graphql",
                "-f", f"query={RESOLVE_MUTATION}",
                "-f", f"threadId={thread.id}",
            ]
        )
        try:
            response = json.loads(payload)
        except json.JSONDecodeError as error:
            raise GhError(f"resolveReviewThread returned unparseable JSON: {error}") from error
        if not isinstance(response, dict) or response.get("errors"):
            raise GhError(f"resolveReviewThread returned errors for {thread.id}")
        resolved = (
            ((response.get("data") or {}).get("resolveReviewThread") or {}).get("thread") or {}
        ).get("isResolved")
        if resolved is not True:
            raise GhError(f"resolveReviewThread did not resolve {thread.id}")
        return True

    def post_receipt(self, pr: int, body: str) -> None:
        self.gh(["pr", "comment", str(pr), "--repo", self.repo, "--body", body])

    # ---- the run -----------------------------------------------------------------------

    def count(self, key: str, amount: int = 1) -> None:
        self.counts[key] = self.counts.get(key, 0) + amount

    def dead_analyses(self) -> list[AnalysisState]:
        """The declared analyses that are PROVABLY not running. Usually empty."""
        states = [self.analysis_state(analysis) for analysis in self.analyses]
        for state in states:
            self.log(
                f"[{PROGRAM}] {state.analysis.workflow}: {state.verdict} "
                f"(state={state.state})"
            )
        return [state for state in states if state.verdict == DEAD]

    def run(self) -> None:
        dead = self.dead_analyses()
        if not dead:
            # THE INVARIANT. A live (or unknown) analysis owns its own threads; this
            # script has no business touching them and stops before reading a single PR.
            self.log(
                f"[{PROGRAM}] no declared analysis is provably disabled — nothing to sweep."
            )
            return
        # Keyed by the NORMALIZED login, because that is what a thread's author reads as.
        by_login = {normalize_login(state.analysis.login): state for state in dead}
        dead_logins = frozenset(by_login)

        for pr, threads in sorted(self.open_pr_threads().items()):
            if pr in self.truncated_prs:
                # A PR is swept WHOLE or not at all, and its thread set overflowed the
                # page — so the threads past the first page cannot even be read. Count
                # them (the census stays total) and resolve none of them.
                self.count(SKIP_PR_THREADS_TRUNCATED, len(threads))
                self.log(
                    f"::warning title={PROGRAM} PR threads truncated::#{pr} has more "
                    "review threads than one page — swept none of them."
                )
                continue
            eligible: list[Thread] = []
            for thread in threads:
                verdict = classify_thread(thread, dead_logins)
                self.count(verdict)
                if verdict == ELIGIBLE:
                    eligible.append(thread)
            if not eligible:
                continue
            self.sweep_pr(pr, eligible, by_login)

        self.log(
            f"[{PROGRAM}] resolved={self.resolved} deferred={self.deferred} "
            f"failures={len(self.failures)} counts={json.dumps(self.counts, sort_keys=True)}"
        )

    def sweep_pr(self, pr: int, eligible: list[Thread], by_login: dict[str, AnalysisState]) -> None:
        """Resolve this PR's eligible threads, one receipt per analysis that filed them."""
        budget = self.max_resolves - self.resolved
        if budget <= 0:
            self.deferred += len(eligible)
            self.log(f"[{PROGRAM}] #{pr}: {len(eligible)} thread(s) deferred (budget spent)")
            return
        if len(eligible) > budget:
            # Partial sweep of a PR is pointless — it stays blocked and re-comments next
            # tick — so defer the whole PR to a later run instead.
            self.deferred += len(eligible)
            self.log(f"[{PROGRAM}] #{pr}: {len(eligible)} thread(s) deferred (budget {budget})")
            return
        # authors[0] OPENED the thread, so it names the analysis a receipt must credit.
        # With two disabled analyses on one PR each gets its own receipt rather than one
        # comment misattributing half the threads.
        groups: dict[str, list[Thread]] = {}
        for thread in eligible:
            groups.setdefault(normalize_login(thread.authors[0]), []).append(thread)
        for login, threads in sorted(groups.items()):
            self.sweep_group(pr, by_login[login], threads)

    def sweep_group(self, pr: int, state: AnalysisState, eligible: list[Thread]) -> None:
        if self.dry_run:
            self.deferred += len(eligible)
            self.log(
                f"[{PROGRAM}] DRY-RUN #{pr}: would resolve {len(eligible)} thread(s) filed by "
                f"{state.analysis.login} ({state.analysis.workflow}={state.state}): "
                + ", ".join(thread.location() for thread in eligible)
            )
            return
        done: list[Thread] = []
        for thread in eligible:
            try:
                self.resolve_thread(thread)
            except GhError as error:
                # Collected, never fatal: the remaining PRs are still swept and the exit
                # status is computed from this list at the end (sticky-failure precedence).
                self.failures.append(f"#{pr} {thread.location()}: {error}")
                self.log(f"::error title={PROGRAM} resolve failed::#{pr} {thread.id}: {error}")
                continue
            done.append(thread)
            self.resolved += 1
        if not done:
            return
        try:
            self.post_receipt(pr, receipt_comment(state, done))
        except GhError as error:
            # A resolve with no receipt is unauditable — that is a failure, not a nit.
            # No early `return` here: the collected failure IS the outcome, and
            # scripts/check-sticky-failure-exits.py reads a bare `return` in a
            # failure-recording handler as a swallowed failure.
            self.failures.append(f"#{pr} receipt: {error}")
            self.log(f"::error title={PROGRAM} receipt failed::#{pr}: {error}")
        else:
            self.log(
                f"[{PROGRAM}] #{pr}: resolved {len(done)} stale {state.analysis.login} thread(s)"
            )


def sweep_exit(sweeper: Sweeper, transient: bool) -> int:
    """PRECEDENCE: collected-failure > transient-exhaustion > clean (mirrors #3766)."""
    if sweeper.failures:
        sweeper.log(
            f"::error title={PROGRAM} failed::{len(sweeper.failures)} mutation(s) failed: "
            + "; ".join(sweeper.failures[:5])
        )
        return 1
    if transient:
        sweeper.log(
            f"::warning title={PROGRAM} skipped::transient GitHub failures exhausted the "
            "bounded retries — the next scheduled tick covers this run."
        )
    return 0


# ======================================================================================
# Self-test. Hermetic: every gh seam is INJECTED at construction, so no code path here
# can reach subprocess/network (scripts/selftest_write_guard.py enforces that statically,
# and `repo` has no default so nothing can inherit a live target by omission).
# ======================================================================================
def _workflow_payload(state: str) -> str:
    return json.dumps({"id": 1, "name": "codeql", "state": state})


def _runs_payload(created: str, total: int = 1, status: str | None = "completed") -> str:
    run = {"created_at": created}
    if status is not None:
        run["status"] = status
    return json.dumps({"total_count": total, "workflow_runs": [run] if total else []})


def _thread_node(node_id, *, authors, resolved=False, path="a.rs", line=7,
                 comments_truncated=False):
    return {
        "id": node_id,
        "isResolved": resolved,
        "isOutdated": False,
        "path": path,
        "line": line,
        "originalLine": line,
        "comments": {
            "pageInfo": {"hasNextPage": comments_truncated},
            "nodes": [{"author": {"login": login}} for login in authors],
        },
    }


def _pr_node(number, threads, *, threads_truncated=False):
    return {
        "number": number,
        "reviewThreads": {
            "pageInfo": {"hasNextPage": threads_truncated},
            "nodes": threads,
        },
    }


def _graphql_payload(pr_nodes, *, has_next=False, cursor=""):
    return json.dumps(
        {
            "data": {
                "repository": {
                    "pullRequests": {
                        "pageInfo": {"hasNextPage": has_next, "endCursor": cursor},
                        "nodes": pr_nodes,
                    }
                }
            }
        }
    )


def _resolve_payload(thread_id, resolved=True):
    return json.dumps(
        {"data": {"resolveReviewThread": {"thread": {"id": thread_id, "isResolved": resolved}}}}
    )


class FakeGh:
    """Records every argv and answers from canned payloads. No subprocess, ever."""

    def __init__(self, *, workflow_state="disabled_manually", pages=None,
                 resolve_error=None, comment_error=None, last_run="2026-07-18T14:01:00Z",
                 runs_total=1, runs_completed=1, runs_error=None, newest_status="completed"):
        self.workflow_state = workflow_state
        self.pages = list(pages or [])
        self.resolve_error = resolve_error
        self.comment_error = comment_error
        self.last_run = last_run
        # The run census: how many runs this workflow ever started, and how many finished.
        # Equal totals = quiescent; runs_total > runs_completed = something is still going.
        self.runs_total = runs_total
        self.runs_completed = runs_completed
        self.runs_error = runs_error
        self.newest_status = newest_status
        self.calls: list[list[str]] = []
        self.page_index = 0

    # -- helpers used by the assertions ---------------------------------------------
    def mutations(self) -> list[list[str]]:
        return [
            argv for argv in self.calls
            if argv[:2] == ["pr", "comment"] or any("mutation" in a for a in argv)
        ]

    def resolves(self) -> list[list[str]]:
        return [argv for argv in self.calls if any("resolveReviewThread" in a for a in argv)]

    def comments(self) -> list[list[str]]:
        return [argv for argv in self.calls if argv[:2] == ["pr", "comment"]]

    def reads(self) -> list[list[str]]:
        return [argv for argv in self.calls if argv not in self.mutations()]

    # -- the seam --------------------------------------------------------------------
    def __call__(self, argv: list[str]) -> str:
        self.calls.append(list(argv))
        joined = " ".join(argv)
        if argv[:2] == ["pr", "comment"]:
            if self.comment_error:
                raise GhError(self.comment_error)
            return ""
        if "resolveReviewThread" in joined:
            if self.resolve_error:
                raise GhError(self.resolve_error)
            thread_id = next(a.split("=", 1)[1] for a in argv if a.startswith("threadId="))
            return _resolve_payload(thread_id)
        if "/runs?" in joined:
            if self.runs_error:
                raise GhError(self.runs_error)
            if f"status={RUN_STATUS_COMPLETED}" in joined:
                return _runs_payload(self.last_run, self.runs_completed)
            return _runs_payload(self.last_run, self.runs_total, self.newest_status)
        if "actions/workflows/" in joined:
            if self.workflow_state is None:
                raise GhError("HTTP 404: Not Found")
            return _workflow_payload(self.workflow_state)
        if "query=" in joined:
            page = self.pages[self.page_index] if self.page_index < len(self.pages) else \
                _graphql_payload([])
            self.page_index += 1
            return page
        raise AssertionError(f"unexpected gh call: {argv}")


def _sweeper(fake: FakeGh, **kw) -> Sweeper:
    return Sweeper(repo="fixture/repo", gh=fake, log=lambda *_: None, **kw)


CODEQL = "github-advanced-security"


def self_test() -> None:
    # ---- pure decisions ---------------------------------------------------------------
    assert normalize_login("GitHub-Advanced-Security[bot]") == "github-advanced-security"
    assert normalize_login(" CodeQL ") == "codeql"
    assert normalize_login(None) == ""

    assert workflow_verdict("active") == LIVE
    for state in DISABLED_STATES:
        assert workflow_verdict(state) == DEAD, state
    # Everything undeclared is UNKNOWN — including a state GitHub invents later.
    for state in (None, "", "queued", "disabled_something_new", 7):
        assert workflow_verdict(state) == UNKNOWN, state

    # The run census is decision-bearing, so an unreadable page reads as None, not 0.
    assert parse_run_total({"total_count": 0}) == 0
    assert parse_run_total({"total_count": 12}) == 12
    for bad in (None, [], {}, {"total_count": None}, {"total_count": "3"},
                {"total_count": True}, {"total_count": -1}):
        assert parse_run_total(bad) is None, bad
    assert parse_run_created(json.loads(_runs_payload("2026-07-18T14:01:00Z"))) \
        == "2026-07-18T14:01:00Z"
    assert parse_run_created(json.loads(_runs_payload("x", total=0))) is None
    assert parse_run_created({"workflow_runs": [{}]}) is None
    assert parse_run_status(json.loads(_runs_payload("x", status=" In_Progress "))) \
        == "in_progress"
    assert parse_run_status(json.loads(_runs_payload("x"))) == RUN_STATUS_COMPLETED
    for bad in (None, {"workflow_runs": []}, {"workflow_runs": [{}]},
                {"workflow_runs": [{"status": ""}]}, {"workflow_runs": [{"status": 7}]}):
        assert parse_run_status(bad) is None, bad

    dead = frozenset({CODEQL})
    bot = Thread("t", 1, False, False, "a.rs", 7, (CODEQL,), False)
    assert classify_thread(bot, dead) == ELIGIBLE
    assert classify_thread(Thread("t", 1, True, False, "a.rs", 7, (CODEQL,), False), dead) \
        == SKIP_RESOLVED
    assert classify_thread(Thread("t", 1, False, False, "a.rs", 7, (), False), dead) \
        == SKIP_NO_COMMENTS
    assert classify_thread(Thread("t", 1, False, False, "a.rs", 7, (CODEQL,), True), dead) \
        == SKIP_AUTHORS_TRUNCATED
    # One human reply disqualifies the whole thread, in either position.
    assert classify_thread(Thread("t", 1, False, False, "a.rs", 7, (CODEQL, "jeswr"), False),
                           dead) == SKIP_OTHER_AUTHOR
    assert classify_thread(Thread("t", 1, False, False, "a.rs", 7, ("jeswr", CODEQL), False),
                           dead) == SKIP_OTHER_AUTHOR
    # A deleted-account author reads as "" and is not a declared analysis.
    assert classify_thread(Thread("t", 1, False, False, "a.rs", 7, ("",), False), dead) \
        == SKIP_OTHER_AUTHOR

    threads, truncated = parse_threads(
        _pr_node(3451, [_thread_node("T1", authors=[CODEQL + "[bot]"], line=None)],
                 threads_truncated=True)
    )
    assert truncated and len(threads) == 1
    assert threads[0].authors == (CODEQL,) and threads[0].pr == 3451
    assert threads[0].line == 7 or threads[0].line is None  # originalLine fallback
    assert parse_threads({"number": None}) == ([], False)

    # ---- THE HEADLINE GUARD: a LIVE analysis is never swept ----------------------------
    # Delete or invert the `verdict == DEAD` filter in dead_analyses() and this goes red.
    page = _graphql_payload([_pr_node(3451, [
        _thread_node("T1", authors=[CODEQL]),
        _thread_node("T2", authors=[CODEQL]),
    ])])
    live = FakeGh(workflow_state="active", pages=[page])
    sweeper = _sweeper(live)
    sweeper.run()
    assert live.mutations() == [], live.mutations()
    assert live.page_index == 0, "a live analysis must not even enumerate PRs"
    assert sweeper.resolved == 0 and sweeper.failures == []
    assert sweep_exit(sweeper, transient=False) == 0

    # An UNKNOWN state (404 / unreadable) is equally inert.
    unknown = FakeGh(workflow_state=None, pages=[page])
    sweeper = _sweeper(unknown)
    sweeper.run()
    assert unknown.mutations() == [] and sweeper.resolved == 0
    for state in ("", "queued"):
        probe = FakeGh(workflow_state=state, pages=[page])
        _sweeper(probe).run()
        assert probe.mutations() == [], state

    # ---- THE SECOND HALF OF THE INVARIANT: disabled is not the same as quiescent -------
    # `disabled_manually` only stops the NEXT trigger. A run queued or in progress when the
    # switch flipped still owns its threads, so the sweep must not start. Delete the
    # `if unfinished:` branch in analysis_state() and this goes red.
    busy = FakeGh(pages=[page], runs_total=4, runs_completed=3)
    sweeper = _sweeper(busy)
    sweeper.run()
    assert busy.mutations() == [], busy.mutations()
    assert busy.page_index == 0, "an unfinished run must not even enumerate PRs"
    assert sweeper.resolved == 0 and sweeper.failures == []
    # Counts that AGREE are not enough either: `total_count` is a server-side aggregate
    # this script cannot verify, so the newest run's own status must also say it finished.
    # Delete the `if newest_status != RUN_STATUS_COMPLETED` branch and this goes red.
    for status in ("in_progress", "queued", "waiting", "some_status_github_adds_later"):
        running = FakeGh(pages=[page], newest_status=status)
        sweeper = _sweeper(running)
        sweeper.run()
        assert running.mutations() == [] and running.page_index == 0, status
        assert sweeper.resolved == 0, status
        # ...and it is LIVE, not UNKNOWN: the analysis is running, it was not unreadable.
        assert sweeper.analysis_state(ANALYSES[0]).verdict == LIVE, status
    # An unreadable or malformed census fails closed identically — as UNKNOWN, because
    # nothing was learned. Reporting it LIVE would claim a fact the read never gave us.
    for census in (dict(runs_error="HTTP 500"), dict(runs_total=0, runs_completed=1),
                   dict(newest_status=None)):
        blind = FakeGh(pages=[page], **census)
        sweeper = _sweeper(blind)
        sweeper.run()
        assert blind.mutations() == [] and blind.page_index == 0, census
        assert sweeper.resolved == 0, census
        assert sweeper.analysis_state(ANALYSES[0]).verdict == UNKNOWN, census
    # ...while a disabled workflow whose every run has COMPLETED does sweep.
    quiet = FakeGh(pages=[page], runs_total=57, runs_completed=57)
    sweeper = _sweeper(quiet)
    sweeper.run()
    assert sweeper.resolved == 2, sweeper.resolved

    # ---- the sweep itself --------------------------------------------------------------
    fake = FakeGh(pages=[page])
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.resolved == 2, sweeper.resolved
    assert sweeper.counts == {ELIGIBLE: 2}, sweeper.counts
    assert [a for a in fake.resolves()], "threads must actually be resolved"
    assert len(fake.comments()) == 1, "exactly one receipt per PR"
    body = fake.comments()[0][fake.comments()[0].index("--body") + 1]
    assert body.startswith(MARKER), body
    assert "disabled_manually" in body and "codeql.yml" in body, body
    assert "2026-07-18T14:01:00Z" in body, body
    assert "a.rs:7" in body, body
    assert "code_scanning" in body and "#4542" in body, body
    assert sweep_exit(sweeper, transient=False) == 0

    # Mixed population: only the all-bot unresolved threads are touched, and the census
    # is TOTAL over what was read.
    mixed = _graphql_payload([
        _pr_node(10, [
            _thread_node("A", authors=[CODEQL]),
            _thread_node("B", authors=[CODEQL, "jeswr"]),
            _thread_node("C", authors=[CODEQL], resolved=True),
            _thread_node("D", authors=[CODEQL], comments_truncated=True),
        ]),
        _pr_node(11, [_thread_node("E", authors=["copilot-pull-request-reviewer"])]),
    ])
    fake = FakeGh(pages=[mixed])
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.resolved == 1, sweeper.resolved
    assert sweeper.counts == {
        ELIGIBLE: 1, SKIP_OTHER_AUTHOR: 2, SKIP_RESOLVED: 1, SKIP_AUTHORS_TRUNCATED: 1,
    }, sweeper.counts
    assert [argv for argv in fake.comments() if "11" in argv] == [], "PR 11 gets no comment"

    # Two disabled analyses on one PR get one receipt EACH — a single comment would
    # misattribute half the threads to the wrong workflow.
    other = Analysis("some-other-analysis", "other.yml", "Some Other analysis")

    class TwoWorkflowGh(FakeGh):
        def __call__(self, argv):
            joined = " ".join(argv)
            if "actions/workflows/other.yml" in joined:
                self.calls.append(list(argv))
                # Disabled AND quiescent — every run it ever started reports completed.
                if "/runs?" in joined:
                    return _runs_payload(self.last_run)
                return _workflow_payload("disabled_inactivity")
            return super().__call__(argv)

    fake = TwoWorkflowGh(pages=[_graphql_payload([_pr_node(40, [
        _thread_node("X", authors=[CODEQL]),
        _thread_node("Y", authors=[other.login]),
    ])])])
    sweeper = _sweeper(fake, analyses=(ANALYSES[0], other))
    sweeper.run()
    assert sweeper.resolved == 2, sweeper.resolved
    bodies = [argv[argv.index("--body") + 1] for argv in fake.comments()]
    assert len(bodies) == 2, bodies
    assert sum("codeql.yml" in body for body in bodies) == 1, bodies
    assert sum("other.yml" in body for body in bodies) == 1, bodies
    # ...and a receipt names ONLY the analysis whose threads it reports.
    assert not any("codeql.yml" in body and "other.yml" in body for body in bodies), bodies

    # A declared login written with the `[bot]` suffix still matches its threads.
    suffixed = Analysis(CODEQL + "[bot]", "codeql.yml", "CodeQL code scanning")
    fake = FakeGh(pages=[page])
    sweeper = _sweeper(fake, analyses=(suffixed,))
    sweeper.run()
    assert sweeper.resolved == 2 and sweeper.failures == [], sweeper.failures

    # ---- dry-run mutates nothing ------------------------------------------------------
    fake = FakeGh(pages=[page])
    sweeper = _sweeper(fake, dry_run=True)
    sweeper.run()
    assert fake.mutations() == [] and sweeper.resolved == 0
    assert sweeper.counts == {ELIGIBLE: 2} and sweeper.deferred == 2

    # ---- bounds ------------------------------------------------------------------------
    fake = FakeGh(pages=[page])
    sweeper = _sweeper(fake, max_resolves=1)
    sweeper.run()
    assert sweeper.resolved == 0 and sweeper.deferred == 2, "a PR is swept whole or not at all"
    assert fake.mutations() == []

    two_prs = _graphql_payload([
        _pr_node(20, [_thread_node("P", authors=[CODEQL])]),
        _pr_node(21, [_thread_node("Q", authors=[CODEQL])]),
    ])
    fake = FakeGh(pages=[two_prs])
    sweeper = _sweeper(fake, max_resolves=1)
    sweeper.run()
    assert sweeper.resolved == 1 and sweeper.deferred == 1, (sweeper.resolved, sweeper.deferred)

    # Pagination stops at max_pages and SAYS SO rather than reporting a clean sweep.
    logged: list[str] = []
    fake = FakeGh(pages=[_graphql_payload([], has_next=True, cursor="c1")] * 3)
    sweeper = Sweeper(repo="fixture/repo", gh=fake, log=logged.append, max_pages=2)
    sweeper.run()
    assert sweeper.truncated_enumeration
    assert any("enumeration truncated" in line for line in logged), logged
    assert fake.page_index == 2, fake.page_index

    # A per-PR thread page overflow fails CLOSED: nested review-thread pagination is not
    # implemented, so the threads past the page are unreachable, and a half-swept PR stays
    # blocked anyway. Every thread read is still counted, so the census stays total.
    fake = FakeGh(pages=[_graphql_payload([
        _pr_node(30, [_thread_node("R", authors=[CODEQL])], threads_truncated=True),
        _pr_node(31, [_thread_node("S", authors=[CODEQL])]),
    ])])
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.truncated_enumeration and sweeper.truncated_prs == {30}
    assert sweeper.counts == {SKIP_PR_THREADS_TRUNCATED: 1, ELIGIBLE: 1}, sweeper.counts
    # ...and only the whole-known PR was touched.
    assert sweeper.resolved == 1, sweeper.resolved
    assert [argv for argv in fake.comments() if "30" in argv] == [], "PR 30 gets no comment"
    assert fake.resolves() and "S" in " ".join(fake.resolves()[0]), fake.resolves()

    # ---- failures are collected, never swallowed ---------------------------------------
    fake = FakeGh(pages=[two_prs], resolve_error="Resource not accessible by integration")
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.resolved == 0 and len(sweeper.failures) == 2, sweeper.failures
    assert fake.comments() == [], "no receipt when nothing was resolved"
    assert sweep_exit(sweeper, transient=False) == 1
    # ...and a failure still exits non-zero when a transient ALSO happened (precedence).
    assert sweep_exit(sweeper, transient=True) == 1

    # A resolve that lands with no receipt is a failure, not a silent success.
    fake = FakeGh(pages=[page], comment_error="HTTP 403")
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.resolved == 2 and len(sweeper.failures) == 1, sweeper.failures
    assert sweep_exit(sweeper, transient=False) == 1

    # A mutation that reports the thread still unresolved is a failure, not a success.
    class LyingGh(FakeGh):
        def __call__(self, argv):
            if any("resolveReviewThread" in a for a in argv):
                self.calls.append(list(argv))
                return _resolve_payload("T1", resolved=False)
            return super().__call__(argv)

    fake = LyingGh(pages=[page])
    sweeper = _sweeper(fake)
    sweeper.run()
    assert sweeper.resolved == 0 and len(sweeper.failures) == 2, sweeper.failures
    assert sweep_exit(sweeper, transient=False) == 1

    # A clean run with only transient read exhaustion is a WARNING, not a red.
    clean = _sweeper(FakeGh(pages=[_graphql_payload([])]))
    clean.run()
    assert sweep_exit(clean, transient=True) == 0

    # ---- every READ this script issues IS a read (fail-closed, per gh_retry) -----------
    fake = FakeGh(pages=[page])
    sweeper = _sweeper(fake)
    sweeper.run()
    if gh_retry is not None:
        for argv in fake.reads():
            gh_retry.assert_read_only(argv)

    # ---- construction ------------------------------------------------------------------
    for bad in ("", "no-slash", "a/b/c", "/name", "owner/"):
        try:
            Sweeper(repo=bad, gh=FakeGh())
        except ValueError:
            continue
        raise AssertionError(f"accepted a malformed repo: {bad!r}")

    print(f"{PROGRAM}: self-test OK")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", help="OWNER/REPOSITORY to sweep")
    parser.add_argument(
        "--max-resolves", type=int, default=DEFAULT_MAX_RESOLVES,
        help="upper bound on threads resolved per run (bounded blast radius)",
    )
    parser.add_argument(
        "--max-pages", type=int, default=DEFAULT_MAX_PAGES,
        help=f"upper bound on open-PR pages read ({PR_PAGE_SIZE} PRs per page)",
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="classify and report only — no thread is resolved and no comment is posted",
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required")

    sweeper = Sweeper(
        repo=args.repo,
        dry_run=args.dry_run,
        max_resolves=args.max_resolves,
        max_pages=args.max_pages,
    )
    transient = False
    try:
        sweeper.run()
    except GhTransientExhausted as error:
        # A missed cycle is harmless — the cron covers it. A collected failure is not,
        # which is why sweep_exit() applies the precedence rather than this handler.
        transient = True
        sweeper.log(f"[{PROGRAM}] transient GitHub failure: {error}")
    return sweep_exit(sweeper, transient)


if __name__ == "__main__":
    sys.exit(main())
