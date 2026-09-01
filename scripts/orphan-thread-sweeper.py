#!/usr/bin/env python3
"""Resolve review threads filed by an analysis that no longer runs on this repository."""

# [OPUS-5] Issue #4542 — THE PERMANENT, SILENT MERGE BLOCKER.
#
# The `main` ruleset sets `required_review_thread_resolution: true`, so every review
# thread on a pull request must be resolved before it can merge. Most threads on this
# repository are filed by an ANALYSIS, not by a person — and an analysis is a workflow,
# which can be turned off. CodeQL has been `disabled_manually` since 2026-07-18.
#
# Those two facts compose into a merge blocker with NO AUTOMATED EXIT. A thread filed
# before the analysis was disabled can never be re-scanned, superseded or dismissed by the
# tool that filed it, because that tool no longer runs. The pull request stays BLOCKED
# forever while every individual CI signal on it is green, so nothing reports a problem.
# MEASURED in #4542 over 119 open PRs on 2026-07-27: one PR (#3451) with ten unresolved
# threads, all `github-advanced-security`, armed for auto-merge for 33 hours with `gate`
# green. Filed for the STRUCTURAL reason, not the volume — the population is currently one.
#
# #4534's stuck-arm sweep already makes that shape VISIBLE — it classifies the PR
# `blocked-threads`, disarms it, parks it and writes a receipt naming `threads-resolved` as
# the condition that ends the park. Visible is not the same as exited: nothing can satisfy
# `threads-resolved`. This sweep is the missing exit, and it is deliberately the NARROW one
# of the three options in #4542 (the other two being "re-enable CodeQL", owned by PR #3427,
# and "drop the ruleset rule", which weakens a real protection). It fixes the CLASS: the
# same dead end re-arms itself the moment any thread-filing analysis is enabled and later
# disabled again.
#
# WHAT MAKES THIS SAFE TO AUTOMATE — every one of these is a hard precondition, and each
# has a self-test that goes red if it is removed. This tool RESOLVES A SECURITY THREAD, so
# the bar is that a maintainer reading the audit trail afterwards would have done the same:
#
#   1. REGISTERED AUTHOR ONLY. The thread's author must be a login in `ORPHANABLE_ANALYSES`
#      below, which maps it to the WORKFLOW FILE whose state decides orphanhood. An analysis
#      that is not backed by a workflow in this repository (Copilot code review, a human
#      reviewer) must never appear in that map — nothing could ever prove it stopped
#      running — and scripts/tests/test_orphan_thread_sweeper_wiring.py reds if one does.
#   2. NOBODY ELSE HAS SPOKEN. If the thread carries a comment from any second author —
#      a human, or another bot — it is a conversation, not an orphan, and is left alone.
#   3. THE FINDING IS THE ONE WE MAPPED. `github-advanced-security` is the identity behind
#      more than one analysis, so the registry entry also carries a body marker the finding
#      text must contain. A dependency-review thread is not orphaned by CodeQL being off.
#   4. THE ANALYSIS IS ACTUALLY OFF, READ LIVE. The mapped workflow's state comes from the
#      Actions API on every run — never from a constant, a doc, or an assumption.
#   5. IT HAS NOT RUN SINCE THE THREAD WAS FILED. If the analysis ran after the finding was
#      filed and the thread is still open, the analysis HAD its chance to supersede it and
#      did not. That is a live finding a person owns, not an orphan.
#   6. THE RATIONALE IS POSTED BEFORE THE RESOLUTION. Each thread gets a reply naming the
#      analysis, its state, its last run and this issue; only then is it resolved. A reply
#      that fails ABORTS that thread's resolution — there is no path that resolves silently.
#   7. BOUNDED PER TICK, and every non-action is COUNTED, so a census exists whether or not
#      anything was mutated.
#
# NOT A GATE. Schedule / workflow_dispatch only, never on a pull-request head. Reads are
# one-shot, which is why this script deliberately does not carry the arm sweeps' gh_retry
# helper: nothing here is worth more than an hour's delay. A failed ENUMERATION ends the
# tick loudly and mutates nothing; a failed WORKFLOW-STATE read downgrades every thread that
# analysis backs to `unreadable`, which also resolves nothing. Both exit non-zero, and so
# does any MUTATION failure — including a `resolveReviewThread` that returns HTTP 200
# without actually resolving the thread.

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable, Iterable

PROGRAM = "orphan-thread-sweeper"

# The self-identification every automated comment on this repository carries.
RECEIPT_MARKER = "> 🤖 SPARQ agent"

# The issue this policy implements, quoted in every comment it posts so a reader lands on
# the reasoning rather than on a bare bot action.
POLICY_ISSUE = "https://github.com/sparq-org/sparq/issues/4542"


@dataclass(frozen=True)
class Analysis:
    """One thread-filing analysis, and the workflow whose state decides orphanhood."""

    # The workflow FILE NAME as the Actions API addresses it. Its `state` is read live.
    workflow: str
    # A substring the FINDING TEXT must contain. `github-advanced-security` is the author
    # of several distinct analyses' threads; without this, disabling one of them would make
    # the others' threads look orphaned. Narrow by construction.
    marker: str


# THE REGISTRY. Adding an entry is a deliberate, reviewed act: it grants this sweep the
# authority to resolve that author's threads once the named workflow is off. Two rules the
# self-test pins:
#   * the value's `workflow` must be a workflow FILE in this repository, so the eligibility
#     question is always answerable from the Actions API, and
#   * an analysis with no workflow behind it (Copilot code review is the live example — it
#     is a repository setting, not a workflow) MUST NOT be added: nothing could ever prove
#     it stopped running, so it could never be orphaned, so an entry for it would only ever
#     be a way to resolve live review threads.
ORPHANABLE_ANALYSES: dict[str, Analysis] = {
    "github-advanced-security": Analysis(workflow="codeql.yml", marker="CodeQL"),
}

# Actions workflow states in which GitHub schedules NO run on any trigger. `active` and
# every unknown state fail closed to "the analysis is live".
DISABLED_STATES = frozenset({"disabled_manually", "disabled_inactivity"})

ACTION_NONE = "none"
ACTION_RESOLVE = "resolve"

# The class enum, TOTAL over the unresolved-thread population and closed the same way
# rearm-sweeper.py's is: this dict is the single source of truth for both the class set and
# its routing, so a class added without routing is a KeyError at import rather than a
# silent no-op at 03:00.
CLASS_ACTIONS: dict[str, str] = {
    # --- not eligible. Counted, never touched. ----------------------------------------
    "already-resolved": ACTION_NONE,     # nothing to do; the enumeration filters these too
    "other-participant": ACTION_NONE,    # somebody else replied: a conversation, not an orphan
    "unregistered-author": ACTION_NONE,  # not a registered analysis (incl. every human)
    "marker-mismatch": ACTION_NONE,      # registered author, but not the mapped analysis
    "analysis-live": ACTION_NONE,        # the workflow still runs — it owns its own thread
    "analysis-ran-since": ACTION_NONE,   # it ran after the filing and did not supersede
    "unreadable": ACTION_NONE,           # short/absent page or unreadable workflow state
    # --- the one eligible class. --------------------------------------------------------
    "orphaned": ACTION_RESOLVE,
}

# The bound. Small on purpose: this mutation is a security-thread resolution, so a backlog
# that all becomes eligible on one tick drains over several hourly ticks under a maintainer's
# eye rather than in one unreviewable burst. Every deferred thread is still CLASSIFIED and
# COUNTED.
DEFAULT_MAX_RESOLUTIONS = 10

# Per-page sizes for the enumeration. A thread page that overflows costs COVERAGE (the tail
# waits for a later tick); a COMMENT page that overflows costs ELIGIBILITY (the thread is
# classified `unreadable`, because a hidden page could hold the human reply that rule 2
# refuses on). Both failures are in the safe direction.
PR_PAGE_SIZE = 10
THREAD_PAGE_SIZE = 50
COMMENT_PAGE_SIZE = 20
MAX_PR_PAGES = 40

THREADS_QUERY = """query($owner:String!,$name:String!,$base:String!,$cursor:String){
  repository(owner:$owner,name:$name){
    pullRequests(states:OPEN,baseRefName:$base,first:%(prs)d,after:$cursor,
                 orderBy:{field:UPDATED_AT,direction:DESC}){
      pageInfo{hasNextPage endCursor}
      nodes{
        number
        reviewThreads(first:%(threads)d){
          pageInfo{hasNextPage}
          nodes{
            id
            isResolved
            isOutdated
            path
            comments(first:%(comments)d){
              pageInfo{hasNextPage}
              nodes{databaseId author{login} body createdAt}
            }
          }
        }
      }
    }
  }
}""" % {"prs": PR_PAGE_SIZE, "threads": THREAD_PAGE_SIZE, "comments": COMMENT_PAGE_SIZE}

RESOLVE_MUTATION = (
    "mutation($id:ID!){resolveReviewThread(input:{threadId:$id})"
    "{thread{id isResolved}}}"
)


class GhError(RuntimeError):
    """A `gh` invocation failed."""


@dataclass(frozen=True)
class WorkflowState:
    """The live Actions state of one workflow, plus when it last produced a run."""

    state: str
    # Epoch seconds of the newest run on ANY ref/event; 0 means "never ran". 0 makes the
    # workflow look infinitely stale, which is the correct direction here: a workflow that
    # has never run cannot have superseded anything.
    last_run_epoch: int


@dataclass(frozen=True)
class Thread:
    """One review thread. Pure data: no gh handle, no clock, no I/O."""

    id: str
    pr: int
    path: str
    is_resolved: bool
    is_outdated: bool
    # Distinct comment author logins, `[bot]` suffix stripped, in first-seen order.
    authors: tuple[str, ...]
    first_body: str
    # REST id of the first comment — what a reply is addressed to. None means the thread
    # was read without one, which is unreadable rather than eligible.
    first_comment_id: int | None
    created_epoch: int
    comments_truncated: bool


def iso_epoch(value: object) -> int:
    """ISO-8601 Z timestamp -> epoch seconds; 0 when absent or unparseable."""
    if not isinstance(value, str) or not value:
        return 0
    text = value.replace("+00:00", "Z")
    try:
        return int(
            datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ")
            .replace(tzinfo=timezone.utc)
            .timestamp()
        )
    except ValueError:
        return 0


def normalize_login(raw: object) -> str:
    """`github-advanced-security[bot]` and `github-advanced-security` are one identity.

    GraphQL's `Bot.login` and REST's `user.login` disagree about the `[bot]` suffix for the
    same actor, so a registry keyed on one form would silently match nothing on the other —
    and "matches nothing" here reads as "no orphans found", which is invisible.
    """
    if not isinstance(raw, str):
        return ""
    login = raw.strip()
    return login[: -len("[bot]")] if login.endswith("[bot]") else login


def parse_thread(pr: int, raw: object) -> Thread | None:
    """One `reviewThreads.nodes[]` entry -> a `Thread`, or None when it is not one."""
    if not isinstance(raw, dict):
        return None
    thread_id = raw.get("id")
    if not isinstance(thread_id, str) or not thread_id:
        return None
    comments = raw.get("comments") or {}
    nodes = comments.get("nodes")
    nodes = nodes if isinstance(nodes, list) else []
    authors: list[str] = []
    for node in nodes:
        if not isinstance(node, dict):
            continue
        login = normalize_login((node.get("author") or {}).get("login"))
        # An author GitHub cannot name (a deleted account renders as null) is NOT the
        # registered analysis, so it must still occupy a slot in `authors` — otherwise a
        # thread with one analysis comment and one anonymous reply would look single-authored
        # and become eligible. "" is deliberately kept as a distinct participant.
        if login not in authors:
            authors.append(login)
    first = nodes[0] if nodes and isinstance(nodes[0], dict) else {}
    ident = first.get("databaseId")
    body = first.get("body")
    return Thread(
        id=thread_id,
        pr=pr,
        path=str(raw.get("path") or ""),
        is_resolved=bool(raw.get("isResolved")),
        is_outdated=bool(raw.get("isOutdated")),
        authors=tuple(authors),
        first_body=body if isinstance(body, str) else "",
        first_comment_id=ident if isinstance(ident, int) else None,
        created_epoch=iso_epoch(first.get("createdAt")),
        comments_truncated=bool((comments.get("pageInfo") or {}).get("hasNextPage")),
    )


def classify(thread: Thread, states: dict[str, WorkflowState | None]) -> str:
    """Which class this thread is in. PURE — the whole safety argument lives here.

    `states` maps a workflow file name to its live state, or to None when the state could
    not be read. Every unknown maps to a NON-acting class.
    """
    if thread.is_resolved:
        return "already-resolved"
    if thread.comments_truncated or thread.first_comment_id is None or not thread.authors:
        return "unreadable"
    if len(thread.authors) != 1:
        # Rule 2. A second voice — human or bot — makes this a conversation somebody owns.
        return "other-participant"
    analysis = ORPHANABLE_ANALYSES.get(thread.authors[0])
    if analysis is None:
        return "unregistered-author"
    if analysis.marker not in thread.first_body:
        # Rule 3. Registered author, different analysis: not orphaned by THIS workflow.
        return "marker-mismatch"
    state = states.get(analysis.workflow)
    if state is None:
        return "unreadable"
    if state.state not in DISABLED_STATES:
        # Rule 4, fail-closed: `active` AND every state this script does not recognise.
        return "analysis-live"
    if state.last_run_epoch and thread.created_epoch and (
        state.last_run_epoch >= thread.created_epoch
    ):
        # Rule 5. It ran after the filing and left the thread standing.
        return "analysis-ran-since"
    if not thread.created_epoch:
        # An unreadable filing time cannot satisfy rule 5, so it cannot be eligible.
        return "unreadable"
    return "orphaned"


def render_run_time(epoch: int) -> str:
    if not epoch:
        return "never"
    return (
        datetime.fromtimestamp(epoch, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    )


def thread_reply(thread: Thread, analysis: Analysis, state: WorkflowState) -> str:
    """The audit trail, posted BEFORE the resolution and addressed to the finding itself."""
    return (
        f"{RECEIPT_MARKER}\n\n"
        f"**Resolved by `{PROGRAM}`: the analysis that filed this thread no longer runs.**\n\n"
        f"- author: `{thread.authors[0]}` (registered analysis, workflow "
        f"`.github/workflows/{analysis.workflow}`)\n"
        f"- workflow state: `{state.state}` — GitHub schedules no run on any trigger\n"
        f"- last run of that workflow: {render_run_time(state.last_run_epoch)}\n"
        f"- this thread was filed: {render_run_time(thread.created_epoch)}, and the analysis "
        "has not run since, so it can never re-scan, supersede or dismiss its own finding\n\n"
        "The finding text above is unchanged and this resolution is reversible — click "
        "**Unresolve conversation** to put it back. Nothing here says the finding was wrong; "
        "it says the tool that filed it was switched off, and the `main` ruleset's "
        "`required_review_thread_resolution` would otherwise block this pull request forever "
        f"with no automated exit. Policy: {POLICY_ISSUE}"
    )


def pr_receipt(
    pr: int, resolved: list[Thread], analysis: Analysis, state: WorkflowState
) -> str:
    """One visible, machine-readable receipt per pull request per tick."""
    listed = "\n".join(
        f"- `{thread.path or '<file unknown>'}`"
        + (" (outdated)" if thread.is_outdated else "")
        for thread in resolved
    )
    payload = {
        "program": PROGRAM,
        "policy_issue": 4542,
        "pr": pr,
        "analysis": resolved[0].authors[0],
        "workflow": analysis.workflow,
        "workflow_state": state.state,
        "workflow_last_run": render_run_time(state.last_run_epoch),
        "resolved_thread_ids": [thread.id for thread in resolved],
    }
    return (
        f"{RECEIPT_MARKER}\n\n"
        f"**`{PROGRAM}` resolved {len(resolved)} orphaned review thread(s).**\n\n"
        f"They were filed by `{resolved[0].authors[0]}`, whose workflow "
        f"`.github/workflows/{analysis.workflow}` is `{state.state}` (last run "
        f"{render_run_time(state.last_run_epoch)}) — so it can never re-scan or supersede "
        "them, and the ruleset's `required_review_thread_resolution` would keep this pull "
        "request BLOCKED with no automated exit. Each thread carries its own reply with the "
        "evidence, the findings are untouched, and every resolution is reversible.\n\n"
        f"{listed}\n\n"
        "```json\n" + json.dumps(payload, indent=2, sort_keys=True) + "\n```\n\n"
        f"Policy: {POLICY_ISSUE}"
    )


def run_gh(argv: list[str]) -> str:
    proc = subprocess.run(["gh", *argv], capture_output=True, text=True)
    if proc.returncode != 0:
        raise GhError(
            f"gh {' '.join(argv[:3])} failed ({proc.returncode}): "
            f"{(proc.stderr or proc.stdout).strip()[:400]}"
        )
    return proc.stdout


class OrphanThreadSweeper:
    """Enumerate unresolved threads, classify them, and resolve only the orphans."""

    def __init__(
        self,
        repo: str,
        default_branch: str = "main",
        *,
        max_resolutions: int = DEFAULT_MAX_RESOLUTIONS,
        gh: Callable[[list[str]], str] = run_gh,
        log: Callable[[str], None] = print,
        dry_run: bool = False,
    ) -> None:
        self.repo = repo
        self.default_branch = default_branch
        self.max_resolutions = max_resolutions
        self.gh = gh
        self.log = log
        # OBSERVE-ONLY. Exists so the census can be taken against the live repository before
        # any resolution is enabled. The scheduled workflow step MUST NOT pass it — pinned by
        # scripts/tests/test_orphan_thread_sweeper_wiring.py, because a sweep permanently
        # stuck in dry-run reports beautifully and removes no dead end.
        self.dry_run = dry_run
        self.counts: dict[str, int] = {}
        self.resolved = 0
        self.deferred = 0
        self.errors = 0
        self.observed = 0
        self.truncated_prs: list[int] = []
        self._states: dict[str, WorkflowState | None] = {}
        try:
            self.owner, self.name = repo.split("/", 1)
        except ValueError as error:
            raise ValueError("repo must be OWNER/REPOSITORY") from error
        if not self.owner or not self.name or "/" in self.name:
            raise ValueError("repo must be OWNER/REPOSITORY")

    # ---- reads -------------------------------------------------------------------------

    def _read_json(self, argv: list[str]) -> object:
        return json.loads(self.gh(argv))

    def workflow_state(self, workflow: str) -> WorkflowState | None:
        """Rule 4 + rule 5, read LIVE from the Actions API and memoised for the tick."""
        if workflow in self._states:
            return self._states[workflow]
        state: WorkflowState | None = None
        try:
            meta = self._read_json(
                ["api", f"repos/{self.repo}/actions/workflows/{workflow}"]
            )
            if isinstance(meta, dict) and isinstance(meta.get("state"), str):
                runs = self._read_json(
                    [
                        "api",
                        f"repos/{self.repo}/actions/workflows/{workflow}/runs?per_page=1",
                    ]
                )
                rows = (runs or {}).get("workflow_runs") if isinstance(runs, dict) else None
                newest = 0
                if isinstance(rows, list) and rows and isinstance(rows[0], dict):
                    newest = iso_epoch(rows[0].get("created_at"))
                state = WorkflowState(state=meta["state"], last_run_epoch=newest)
        except (GhError, json.JSONDecodeError) as error:
            self.errors += 1
            self.log(
                f"::warning title={PROGRAM} workflow state unreadable::"
                f"{workflow}: {error} — every thread it backs is classified `unreadable` "
                "this tick."
            )
        self._states[workflow] = state
        return state

    def unresolved_threads(self) -> list[Thread]:
        """Every UNRESOLVED thread on every open PR against the default branch."""
        threads: list[Thread] = []
        cursor: str | None = None
        for _page in range(MAX_PR_PAGES):
            argv = [
                "api", "graphql",
                "-f", f"query={THREADS_QUERY}",
                "-f", f"owner={self.owner}",
                "-f", f"name={self.name}",
                "-f", f"base={self.default_branch}",
            ]
            if cursor:
                argv += ["-f", f"cursor={cursor}"]
            response = self._read_json(argv)
            if not isinstance(response, dict) or response.get("errors"):
                raise GhError("GraphQL returned errors while listing review threads")
            page = ((response.get("data") or {}).get("repository") or {}).get(
                "pullRequests"
            ) or {}
            for node in page.get("nodes") or []:
                if not isinstance(node, dict):
                    continue
                number = node.get("number")
                if not isinstance(number, int):
                    continue
                self.observed += 1
                raw_threads = node.get("reviewThreads") or {}
                if (raw_threads.get("pageInfo") or {}).get("hasNextPage"):
                    self.truncated_prs.append(number)
                for entry in raw_threads.get("nodes") or []:
                    thread = parse_thread(number, entry)
                    if thread is not None and not thread.is_resolved:
                        threads.append(thread)
            info = page.get("pageInfo") or {}
            if not info.get("hasNextPage"):
                return threads
            cursor = info.get("endCursor")
            if not isinstance(cursor, str) or not cursor:
                return threads
        self.log(
            f"::warning title={PROGRAM} enumeration truncated::"
            f"stopped after {MAX_PR_PAGES} pages of open pull requests — the census below "
            "covers only what was read."
        )
        return threads

    # ---- mutations. One-shot; a failure is loud and never silently retried. -------------

    def reply(self, pr: int, comment_id: int, body: str) -> None:
        # `POST /repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies` —
        # the documented "reply to a review comment" route. The pull number is part of the
        # path, so it is read from the thread rather than inferred.
        self.gh(
            [
                "api", "-X", "POST",
                f"repos/{self.repo}/pulls/{pr}/comments/{comment_id}/replies",
                "-f", f"body={body}",
            ]
        )

    def resolve(self, thread_id: str) -> None:
        # The RESPONSE is checked, not just the exit status. A GraphQL mutation can return
        # HTTP 200 with an `errors` array, and a receipt that claims a resolution which did
        # not happen is worse than no receipt: it would report the dead end as exited while
        # the pull request stayed BLOCKED.
        raw = self.gh(
            [
                "api", "graphql",
                "-f", f"query={RESOLVE_MUTATION}",
                "-f", f"id={thread_id}",
            ]
        )
        try:
            response = json.loads(raw)
        except json.JSONDecodeError as error:
            raise GhError(f"resolveReviewThread returned unparseable JSON: {error}") from error
        if not isinstance(response, dict) or response.get("errors"):
            raise GhError(f"resolveReviewThread returned errors for {thread_id}")
        thread = (
            ((response.get("data") or {}).get("resolveReviewThread") or {}).get("thread")
            or {}
        )
        if thread.get("isResolved") is not True:
            raise GhError(
                f"resolveReviewThread did not report {thread_id} resolved: {thread!r}"
            )

    def comment(self, pr: int, body: str) -> None:
        self.gh(["pr", "comment", str(pr), "--repo", self.repo, "--body", body])

    def resolve_one(self, thread: Thread) -> bool:
        """Rule 6: the rationale is posted FIRST, and a failed reply resolves nothing."""
        analysis = ORPHANABLE_ANALYSES[thread.authors[0]]
        state = self._states[analysis.workflow]
        assert state is not None  # `orphaned` is unreachable with an unreadable state
        if self.dry_run:
            self.log(
                f"[{PROGRAM}] PR #{thread.pr} thread {thread.id}: DRY-RUN would resolve "
                f"({thread.path})"
            )
            return False
        self.reply(
            thread.pr, thread.first_comment_id or 0, thread_reply(thread, analysis, state)
        )
        self.resolve(thread.id)
        return True

    # ---- the sweep ---------------------------------------------------------------------

    def run(self) -> int:
        try:
            threads = self.unresolved_threads()
        except (GhError, json.JSONDecodeError) as error:
            self.log(f"::error title={PROGRAM} enumeration failed::{error}")
            return 1
        self.log(
            f"[{PROGRAM}] {len(threads)} unresolved thread(s) across {self.observed} open "
            f"PR(s) against {self.default_branch}; max-resolutions={self.max_resolutions}"
        )
        # Pre-read every registered analysis's state once, so the classification of a whole
        # tick rests on ONE observation of the world rather than on a state that could flip
        # halfway through the loop.
        for analysis in ORPHANABLE_ANALYSES.values():
            self.workflow_state(analysis.workflow)

        # Keyed by (pull request, analysis) so a pull request carrying findings from two
        # registered analyses gets one correctly-attributed receipt per analysis rather
        # than a single receipt naming whichever happened to be first.
        eligible: dict[tuple[int, str], list[Thread]] = {}
        selected = 0
        for thread in threads:
            klass = classify(thread, self._states)
            self.counts[klass] = self.counts.get(klass, 0) + 1
            if CLASS_ACTIONS[klass] == ACTION_NONE:
                continue
            if selected >= self.max_resolutions:
                self.deferred += 1
                continue
            selected += 1
            eligible.setdefault((thread.pr, thread.authors[0]), []).append(thread)

        for (pr, _author), pending in sorted(eligible.items()):
            done: list[Thread] = []
            for thread in pending:
                try:
                    if self.resolve_one(thread):
                        done.append(thread)
                        self.resolved += 1
                except GhError as error:
                    self.errors += 1
                    self.log(
                        f"::error title={PROGRAM} resolution failed::PR #{pr} thread "
                        f"{thread.id}: {error}"
                    )
            if done and not self.dry_run:
                analysis = ORPHANABLE_ANALYSES[done[0].authors[0]]
                state = self._states[analysis.workflow]
                assert state is not None
                try:
                    self.comment(pr, pr_receipt(pr, done, analysis, state))
                except GhError as error:
                    self.errors += 1
                    self.log(
                        f"::error title={PROGRAM} receipt failed::PR #{pr}: {error}"
                    )
        self.report()
        return 1 if self.errors else 0

    def report(self) -> None:
        breakdown = " ".join(
            f"{name}={self.counts.get(name, 0)}" for name in sorted(CLASS_ACTIONS)
        )
        self.log(f"[{PROGRAM}] census: {breakdown}")
        if self.truncated_prs:
            self.log(
                f"::warning title={PROGRAM} thread page truncated::"
                + ", ".join(f"#{n}" for n in sorted(set(self.truncated_prs)))
                + f" carry more than {THREAD_PAGE_SIZE} review threads — the tail waits "
                "for a later tick."
            )
        self.log(
            f"[{PROGRAM}] complete: observed-prs={self.observed} "
            f"classified={sum(self.counts.values())} resolved={self.resolved} "
            f"deferred={self.deferred} errors={self.errors}"
            + (" (DRY-RUN: nothing was mutated)" if self.dry_run else "")
        )


# ======================================================================================
# Self-test. Stdlib only, no gh, no network — it runs as a HARD step in docs-quality.yml
# so a change to this policy is validated on the PR that makes it, not at 03:00 on the
# cron. Every safety rule in the header has a case here that RESOLVES NOTHING.
# ======================================================================================

CODEQL_BODY = "## CodeQL / Hard-coded cryptographic value used as a nonce"
NOW = iso_epoch("2026-08-01T00:00:00Z")


def _thread(**kw) -> Thread:
    base = dict(
        id="T_ok",
        pr=3451,
        path="crates/sparq-trust/src/expression.rs",
        is_resolved=False,
        is_outdated=False,
        authors=("github-advanced-security",),
        first_body=CODEQL_BODY,
        first_comment_id=99,
        created_epoch=iso_epoch("2026-07-10T00:00:00Z"),
        comments_truncated=False,
    )
    base.update(kw)
    return Thread(**base)  # type: ignore[arg-type]


DISABLED = WorkflowState(state="disabled_manually", last_run_epoch=iso_epoch(
    "2026-07-01T00:00:00Z"
))


def graphql_page(prs: Iterable[dict], *, has_next: bool = False, cursor: str = "") -> str:
    return json.dumps(
        {
            "data": {
                "repository": {
                    "pullRequests": {
                        "pageInfo": {"hasNextPage": has_next, "endCursor": cursor},
                        "nodes": list(prs),
                    }
                }
            }
        }
    )


def graphql_pr(number: int, threads: Iterable[dict], *, threads_next: bool = False) -> dict:
    return {
        "number": number,
        "reviewThreads": {
            "pageInfo": {"hasNextPage": threads_next},
            "nodes": list(threads),
        },
    }


def graphql_thread(
    ident: str,
    *,
    resolved: bool = False,
    logins: tuple[str, ...] = ("github-advanced-security",),
    body: str = CODEQL_BODY,
    created: str = "2026-07-10T00:00:00Z",
    comments_next: bool = False,
    database_id: int | None = 99,
) -> dict:
    return {
        "id": ident,
        "isResolved": resolved,
        "isOutdated": False,
        "path": "crates/sparq-trust/src/expression.rs",
        "comments": {
            "pageInfo": {"hasNextPage": comments_next},
            "nodes": [
                {
                    "databaseId": database_id if index == 0 else database_id,
                    "author": {"login": login},
                    "body": body if index == 0 else "thanks",
                    "createdAt": created,
                }
                for index, login in enumerate(logins)
            ],
        },
    }


class FakeGh:
    """Records every argv and answers the reads the sweep issues."""

    def __init__(
        self,
        pages: list[str],
        *,
        workflow_state: str = "disabled_manually",
        last_run: str | None = "2026-07-01T00:00:00Z",
        fail: Callable[[list[str]], bool] = lambda argv: False,
        resolve_reports: bool = True,
    ) -> None:
        self.pages = list(pages)
        self.workflow_state = workflow_state
        self.last_run = last_run
        self.fail = fail
        # What `resolveReviewThread` claims about the thread afterwards. False models the
        # HTTP-200-but-nothing-happened response the mutation can return.
        self.resolve_reports = resolve_reports
        self.calls: list[list[str]] = []

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(list(argv))
        if self.fail(argv):
            raise GhError("injected failure")
        joined = " ".join(argv)
        if "actions/workflows" in joined and "/runs" in joined:
            runs = [{"created_at": self.last_run}] if self.last_run else []
            return json.dumps({"workflow_runs": runs})
        if "actions/workflows" in joined:
            return json.dumps({"state": self.workflow_state})
        if "graphql" in argv and any("resolveReviewThread" in a for a in argv):
            thread_id = next(a for a in argv if a.startswith("id=")).removeprefix("id=")
            return json.dumps(
                {
                    "data": {
                        "resolveReviewThread": {
                            "thread": {"id": thread_id, "isResolved": self.resolve_reports}
                        }
                    }
                }
            )
        if "graphql" in argv:
            return self.pages.pop(0) if self.pages else graphql_page([])
        return "{}"

    def kinds(self) -> list[str]:
        out = []
        for argv in self.calls:
            joined = " ".join(argv)
            if "resolveReviewThread" in joined:
                out.append("resolve")
            elif "/replies" in joined:
                out.append("reply")
            elif argv[:2] == ["pr", "comment"]:
                out.append("receipt")
        return out


def sweeper(fake: FakeGh, **kw) -> OrphanThreadSweeper:
    return OrphanThreadSweeper(
        "sparq-org/sparq", "main", gh=fake, log=lambda _m: None, **kw
    )


def self_test() -> None:  # noqa: C901 — a flat table of independent cases reads better here
    # ---- the classifier: one case per safety rule, each RESOLVING NOTHING. -------------
    states = {"codeql.yml": DISABLED}
    assert classify(_thread(), states) == "orphaned"

    # Rule 2 — a second participant, human or bot, ends eligibility.
    assert classify(
        _thread(authors=("github-advanced-security", "jeswr")), states
    ) == "other-participant"
    assert classify(
        _thread(authors=("github-advanced-security", "copilot-pull-request-reviewer")),
        states,
    ) == "other-participant"
    # ...including an author GitHub could not name, which must occupy its own slot.
    assert classify(_thread(authors=("github-advanced-security", "")), states) == (
        "other-participant"
    )

    # Rule 1 — every unregistered author, which is every human reviewer.
    assert classify(_thread(authors=("jeswr",)), states) == "unregistered-author"
    assert classify(
        _thread(authors=("copilot-pull-request-reviewer",)), states
    ) == "unregistered-author"

    # Rule 3 — registered author, a DIFFERENT analysis's finding.
    assert classify(
        _thread(first_body="Dependency review found a vulnerable package"), states
    ) == "marker-mismatch"

    # Rule 4 — fail closed on `active` AND on any state this script does not recognise.
    for live in ("active", "disabled_fork", "", "ACTIVE", "disabled_manually_maybe"):
        assert classify(
            _thread(), {"codeql.yml": WorkflowState(state=live, last_run_epoch=0)}
        ) == "analysis-live", live

    # Rule 5 — it ran after the filing and left the thread standing.
    ran_since = {
        "codeql.yml": WorkflowState(
            state="disabled_manually", last_run_epoch=iso_epoch("2026-07-18T14:01:00Z")
        )
    }
    assert classify(_thread(), ran_since) == "analysis-ran-since"
    # ...and a workflow that never ran at all cannot have superseded anything.
    assert classify(
        _thread(), {"codeql.yml": WorkflowState(state="disabled_manually", last_run_epoch=0)}
    ) == "orphaned"

    # Unreadable inputs are never eligible.
    assert classify(_thread(comments_truncated=True), states) == "unreadable"
    assert classify(_thread(first_comment_id=None), states) == "unreadable"
    assert classify(_thread(authors=()), states) == "unreadable"
    assert classify(_thread(created_epoch=0), states) == "unreadable"
    assert classify(_thread(), {"codeql.yml": None}) == "unreadable"
    assert classify(_thread(), {}) == "unreadable"

    # The class enum is closed and TOTAL: every class routes.
    for name in CLASS_ACTIONS:
        assert CLASS_ACTIONS[name] in (ACTION_NONE, ACTION_RESOLVE), name
    assert sum(1 for a in CLASS_ACTIONS.values() if a == ACTION_RESOLVE) == 1

    # The registry may only name workflows that exist, or rule 4 is unanswerable.
    root = __file__.rsplit("/scripts/", 1)[0]
    for login, analysis in ORPHANABLE_ANALYSES.items():
        assert login == normalize_login(login), login
        assert analysis.marker, login
        path = Path(root) / ".github" / "workflows" / analysis.workflow
        assert path.is_file(), f"{login} maps to a missing workflow: {analysis.workflow}"

    # ---- parsing ----------------------------------------------------------------------
    parsed = parse_thread(3451, graphql_thread("T_a"))
    assert parsed is not None and parsed.authors == ("github-advanced-security",)
    assert parsed.first_comment_id == 99 and parsed.first_body == CODEQL_BODY
    assert normalize_login("github-advanced-security[bot]") == "github-advanced-security"
    assert parse_thread(1, {"id": ""}) is None and parse_thread(1, "nope") is None
    # A `[bot]`-suffixed login is the SAME participant, not a second one.
    bot_suffixed = parse_thread(
        1, graphql_thread("T_b", logins=("github-advanced-security[bot]",))
    )
    assert bot_suffixed is not None
    assert classify(bot_suffixed, states) == "orphaned"

    # ---- the sweep, end to end ---------------------------------------------------------
    fake = FakeGh([graphql_page([graphql_pr(3451, [graphql_thread("T_1")])])])
    sweep = sweeper(fake)
    assert sweep.run() == 0
    assert sweep.counts == {"orphaned": 1}, sweep.counts
    assert sweep.resolved == 1 and sweep.errors == 0
    # Rule 6: the rationale is posted BEFORE the resolution, and the PR gets one receipt.
    assert fake.kinds() == ["reply", "resolve", "receipt"], fake.kinds()
    receipt = next(a for a in fake.calls if a[:2] == ["pr", "comment"])[-1]
    assert RECEIPT_MARKER in receipt and "codeql.yml" in receipt and "4542" in receipt
    reply_call = next(a for a in fake.calls if "/replies" in " ".join(a))
    assert "repos/sparq-org/sparq/pulls/3451/comments/99/replies" in reply_call, reply_call
    assert "disabled_manually" in reply_call[-1] and "Unresolve conversation" in reply_call[-1]

    # An already-resolved thread handed straight to the classifier still routes nowhere.
    assert classify(_thread(is_resolved=True), states) == "already-resolved"

    # An already-resolved thread is never even a candidate.
    fake = FakeGh([graphql_page([graphql_pr(1, [graphql_thread("T_r", resolved=True)])])])
    sweep = sweeper(fake)
    assert sweep.run() == 0 and sweep.counts == {} and fake.kinds() == []

    # A live analysis mutates nothing, and is still COUNTED.
    fake = FakeGh(
        [graphql_page([graphql_pr(1, [graphql_thread("T_l")])])], workflow_state="active"
    )
    sweep = sweeper(fake)
    assert sweep.run() == 0 and sweep.counts == {"analysis-live": 1}
    assert fake.kinds() == []

    # An unreadable workflow state mutates nothing and is reported as an error.
    fake = FakeGh(
        [graphql_page([graphql_pr(1, [graphql_thread("T_u")])])],
        fail=lambda argv: "actions/workflows" in " ".join(argv),
    )
    sweep = sweeper(fake)
    assert sweep.run() == 1 and sweep.counts == {"unreadable": 1} and fake.kinds() == []

    # THE BOUND is real: eligible threads beyond the cap are deferred, not resolved.
    fake = FakeGh(
        [graphql_page([graphql_pr(7, [graphql_thread(f"T_{i}") for i in range(5)])])]
    )
    sweep = sweeper(fake, max_resolutions=2)
    assert sweep.run() == 0
    assert sweep.resolved == 2 and sweep.deferred == 3, (sweep.resolved, sweep.deferred)
    assert fake.kinds().count("resolve") == 2

    # DRY-RUN classifies and resolves nothing.
    fake = FakeGh([graphql_page([graphql_pr(1, [graphql_thread("T_d")])])])
    sweep = sweeper(fake, dry_run=True)
    assert sweep.run() == 0 and sweep.counts == {"orphaned": 1}
    assert sweep.resolved == 0 and fake.kinds() == []

    # A FAILED REPLY RESOLVES NOTHING — the audit trail is a precondition, not a courtesy.
    fake = FakeGh(
        [graphql_page([graphql_pr(1, [graphql_thread("T_f")])])],
        fail=lambda argv: "/replies" in " ".join(argv),
    )
    sweep = sweeper(fake)
    assert sweep.run() == 1
    assert "resolve" not in fake.kinds() and "receipt" not in fake.kinds()
    assert sweep.resolved == 0 and sweep.errors == 1

    # A mutation that reports HTTP 200 and does NOT resolve the thread is an ERROR, and
    # the pull request must not receive a receipt claiming a dead end that is still there.
    fake = FakeGh(
        [graphql_page([graphql_pr(1, [graphql_thread("T_n")])])], resolve_reports=False
    )
    sweep = sweeper(fake)
    assert sweep.run() == 1 and sweep.resolved == 0 and sweep.errors == 1
    assert "receipt" not in fake.kinds(), fake.kinds()

    # Pagination: a second page is followed, and a truncated THREAD page is reported.
    fake = FakeGh(
        [
            graphql_page(
                [graphql_pr(1, [graphql_thread("T_p1")], threads_next=True)],
                has_next=True,
                cursor="c1",
            ),
            graphql_page([graphql_pr(2, [graphql_thread("T_p2")])]),
        ]
    )
    sweep = sweeper(fake)
    assert sweep.run() == 0 and sweep.resolved == 2 and sweep.observed == 2
    assert sweep.truncated_prs == [1]
    assert any("cursor=c1" in " ".join(argv) for argv in fake.calls)

    # A failed enumeration is loud and exits non-zero without mutating anything.
    fake = FakeGh([], fail=lambda argv: "graphql" in argv)
    sweep = sweeper(fake)
    assert sweep.run() == 1 and fake.kinds() == []

    # Malformed repo arguments are refused before any read.
    for bad in ("", "no-slash", "a/b/c", "/x", "y/"):
        try:
            OrphanThreadSweeper(bad)
        except ValueError:
            continue
        raise AssertionError(f"accepted a malformed repo: {bad!r}")

    print(f"[{PROGRAM}] self-test OK")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", help="OWNER/REPOSITORY to sweep")
    parser.add_argument("--default-branch", default="main")
    parser.add_argument(
        "--max-resolutions",
        type=int,
        default=DEFAULT_MAX_RESOLUTIONS,
        help="per-tick bound on resolutions (the remainder is deferred to the next tick)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="classify and report, mutate nothing (census only)",
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required unless --self-test is given")
    if args.max_resolutions < 1:
        parser.error("--max-resolutions must be at least 1")
    return OrphanThreadSweeper(
        args.repo,
        args.default_branch,
        max_resolutions=args.max_resolutions,
        dry_run=args.dry_run,
    ).run()


if __name__ == "__main__":
    sys.exit(main())
