#!/usr/bin/env python3
"""Arm reviewed pull requests through GitHub's merge queue.

The policy is deliberately fail-closed. It refreshes every candidate immediately before
arming, rechecks all human/security holds, and supplies the refreshed head OID as a CAS
to GitHub's explicit ``enablePullRequestAutoMerge`` GraphQL mutation.
"""

# [GPT-5] Issue #447 — deterministic, hermetically self-tested auto-arm policy.
# [FABLE-5] Issue #3759 / registry#563 item 4 — idempotent gh READS (pr list/view) go
# through gh_retry.run_gh_read (bounded, transient-only retry); MUTATIONS (pr ready,
# the enablePullRequestAutoMerge CAS) stay one-shot on the un-wrapped runner. When a
# transient 5xx survives every bounded retry, the sweep exits ::warning + 0 instead of
# redding main's gate — a missed cycle is harmless, the cron covers it. Any
# non-transient error still fails loudly (GhFatalError -> GhError -> traceback).

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
from dataclasses import dataclass
from typing import Callable

import gh_retry


REVIEW_LABEL = "review:pass"
HUMAN_OR_TRUST_LABELS = frozenset(
    {
        "trust-surface",
        "needs:user",
        "review:needs-user",
        "area:sparq-zk",
        "area:sparq-mpc",
        "area:sparq-trust",
    }
)
# [FABLE-5] #3759 finding 1: a PR flipped to review:changes (e.g. by the merge-queue
# dequeued-feedback loop) is in the fix lane and must NEVER be armed — even if a stale
# review:pass is still present. This is belt-and-suspenders with the feedback workflow
# atomically REMOVING review:pass when it adds review:changes: the sweep treats
# review:changes (and review:needs) as a HARD exclusion independent of review:pass, so
# a race that momentarily left BOTH labels on the PR can never re-arm the dequeued PR
# before remediation.
REVIEW_CHANGES_LABELS = frozenset({"review:changes", "review:needs"})
PR_FIELDS = (
    "id,number,state,isDraft,headRefOid,baseRefName,labels,autoMergeRequest,"
    "mergeStateStatus"
)
ENABLE_AUTO_MERGE = """mutation($id:ID!,$oid:GitObjectID!){
  enablePullRequestAutoMerge(input:{pullRequestId:$id,expectedHeadOid:$oid,mergeMethod:SQUASH}){
    pullRequest{number headRefOid autoMergeRequest{enabledAt}}
  }
}"""


class GhError(RuntimeError):
    """A gh command failed."""


@dataclass(frozen=True)
class PullRequest:
    node_id: str
    number: int
    state: str
    is_draft: bool
    head_oid: str
    base_ref: str
    labels: frozenset[str]
    armed: bool
    merge_state: str


def parse_pr(raw: dict) -> PullRequest:
    labels = frozenset(
        str(label.get("name", "")).strip().lower()
        for label in raw.get("labels", [])
        if isinstance(label, dict)
    )
    return PullRequest(
        node_id=str(raw.get("id", "")),
        number=int(raw["number"]),
        state=str(raw.get("state", "")).upper(),
        is_draft=bool(raw.get("isDraft")),
        head_oid=str(raw.get("headRefOid", "")),
        base_ref=str(raw.get("baseRefName", "")),
        labels=labels,
        armed=raw.get("autoMergeRequest") is not None,
        merge_state=str(raw.get("mergeStateStatus", "")).upper(),
    )


def run_gh(argv: list[str]) -> str:
    result = subprocess.run(
        ["gh", *argv], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh failure"
        raise GhError(f"gh {' '.join(argv[:3])} failed: {detail}")
    return result.stdout


def run_gh_read(argv: list[str]) -> str:
    """Idempotent READ path: bounded transient-only retries (#3759).

    Non-transient failures re-raise as :class:`GhError` so every existing per-PR
    handler keeps working; exhausted transients propagate as
    :class:`gh_retry.GhTransientExhausted` for the ::warning + exit-0 sweep policy.
    """
    try:
        return gh_retry.run_gh_read(argv)
    except gh_retry.GhFatalError as error:
        raise GhError(str(error)) from error


class AutoArmer:
    def __init__(
        self,
        repo: str,
        default_branch: str,
        gh: Callable[[list[str]], str] = run_gh,
        log: Callable[[str], None] = print,
        *,
        gh_read: Callable[[list[str]], str] | None = None,
    ) -> None:
        self.repo = repo
        self.default_branch = default_branch
        self.gh = gh
        # Idempotent READS may go through a retrying runner (#3759); mutations
        # (pr ready, the arm CAS) always use the one-shot `gh` runner above.
        self.gh_read = gh_read if gh_read is not None else gh
        self.log = log

    def list_open(self) -> list[PullRequest]:
        raw = json.loads(
            self.gh_read(
                [
                    "pr",
                    "list",
                    "--repo",
                    self.repo,
                    "--state",
                    "open",
                    "--limit",
                    "1000",
                    "--json",
                    PR_FIELDS,
                ]
            )
        )
        return [parse_pr(item) for item in raw]

    def refresh(self, number: int) -> PullRequest:
        raw = json.loads(
            self.gh_read(
                [
                    "pr",
                    "view",
                    str(number),
                    "--repo",
                    self.repo,
                    "--json",
                    PR_FIELDS,
                ]
            )
        )
        return parse_pr(raw)

    @staticmethod
    def security_labels(pr: PullRequest) -> list[str]:
        return sorted(pr.labels & HUMAN_OR_TRUST_LABELS)

    def skip_reason(self, pr: PullRequest, *, require_review: bool = True) -> str | None:
        if pr.state != "OPEN":
            return f"not-open ({pr.state or 'unknown'})"
        blocked = self.security_labels(pr)
        if blocked:
            return f"security-surface ({', '.join(blocked)})"
        # [FABLE-5] #3759 finding 1: review:changes is a HARD exclusion checked BEFORE
        # review:pass, so a PR in the fix lane is never armed even with a stale
        # review:pass still on it (a re-arm before remediation is the exact bug).
        changes = sorted(pr.labels & REVIEW_CHANGES_LABELS)
        if changes:
            return f"changes-requested ({', '.join(changes)})"
        if pr.base_ref != self.default_branch:
            return f"non-default-base ({pr.base_ref or 'unknown'})"
        if require_review and REVIEW_LABEL not in pr.labels:
            return "review-label-removed"
        if pr.armed:
            return "already-armed"
        return None

    def mark_ready(self, pr: PullRequest) -> bool:
        try:
            self.gh(["pr", "ready", str(pr.number), "--repo", self.repo])
        except GhError as error:
            self.log(f"PR #{pr.number}: skipped draft (gh pr ready failed: {error})")
            return False
        self.log(f"PR #{pr.number}: draft -> marked ready")
        return True

    def arm(self, pr: PullRequest) -> None:
        self.gh(
            [
                "api",
                "graphql",
                "-f",
                f"query={ENABLE_AUTO_MERGE}",
                "-f",
                f"id={pr.node_id}",
                "-f",
                f"oid={pr.head_oid}",
            ]
        )

    def classify_mutation_race(self, attempted: PullRequest) -> str | None:
        """Classify a failed CAS without ever retrying the mutation in this sweep."""
        current = self.refresh(attempted.number)
        reason = self.skip_reason(current)
        if reason:
            return reason
        if current.is_draft:
            return "draft"
        if current.head_oid != attempted.head_oid:
            return f"head-moved ({attempted.head_oid[:12]} -> {current.head_oid[:12]})"
        return None

    def process(self, initial: PullRequest) -> None:
        reason = self.skip_reason(initial)
        if reason:
            self.log(f"PR #{initial.number}: skipped {reason}")
            return

        if initial.is_draft and not self.mark_ready(initial):
            return

        try:
            current = self.refresh(initial.number)
        except GhError as error:
            self.log(f"PR #{initial.number}: skipped state-refresh-failed ({error})")
            return

        reason = self.skip_reason(current)
        if reason:
            self.log(f"PR #{initial.number}: skipped {reason}")
            return
        if current.is_draft:
            self.log(f"PR #{initial.number}: skipped draft")
            return
        if not current.node_id or not current.head_oid:
            self.log(f"PR #{initial.number}: skipped incomplete-live-state")
            return
        if current.head_oid != initial.head_oid:
            self.log(
                f"PR #{initial.number}: skipped head-moved "
                f"({initial.head_oid[:12]} -> {current.head_oid[:12]})"
            )
            return

        try:
            self.arm(current)
        except GhError as mutation_error:
            # The arm mutation FAILED — this error is the primary run status and must
            # survive whatever the follow-up diagnostic read does. The diagnostic
            # read (classify_mutation_race -> refresh) is itself retriable, so it can
            # raise GhError *or* GhTransientExhausted; if it exhausts transient
            # retries we must NOT let that exhaustion propagate, or run_sweep's
            # lenient GhTransientExhausted handler would convert a genuine arm failure
            # into a false success (::warning + exit 0). Suppress the diagnostic's own
            # failure and re-raise the ORIGINAL mutation error. (#3759 finding 5.)
            try:
                race = self.classify_mutation_race(current)
            except (GhError, gh_retry.GhTransientExhausted) as diag_error:
                self.log(
                    f"PR #{initial.number}: arm failed and the race diagnostic could "
                    f"not resolve ({diag_error}); propagating the original arm error."
                )
                raise mutation_error from mutation_error
            if race:
                self.log(f"PR #{initial.number}: skipped {race}")
                return
            raise mutation_error
        self.log(f"PR #{initial.number}: armed head {current.head_oid[:12]}")

    def run(self) -> None:
        for pr in self.list_open():
            if REVIEW_LABEL in pr.labels:
                self.process(pr)


def run_sweep(
    armer: AutoArmer, log: Callable[[str], None] = print, *, mode: str = "sweep"
) -> int:
    """Run the armer once. Exit policy depends on the INVOCATION MODE (#3759 finding 5).

    ``mode="sweep"`` (the periodic cron) — exhausted TRANSIENT retries on the READ
    path => ``::warning`` + exit 0: the sweep is idempotent and cron-covered, so a
    missed cycle must not red main's gate.

    ``mode="event"`` (the label-triggered auto-arm on a single PR) — there is NO cron
    backstop for *this* PR right now, so the lenient exit does NOT apply: an exhausted
    transient (or any other failure) exits NON-zero so the arm attempt is visibly red
    and gets retried. Only the periodic sweep may swallow a transient.

    A failed arm MUTATION always propagates as ``GhError`` in either mode (the
    diagnostic read's own exhaustion can never overwrite it — see ``process``)."""
    if mode not in ("sweep", "event"):
        raise ValueError(f"mode must be 'sweep' or 'event', got {mode!r}")
    try:
        armer.run()
    except gh_retry.GhTransientExhausted as error:
        if mode == "event":
            # Event-driven arm: no per-PR cron backstop — fail loudly so it is retried.
            raise GhError(
                f"event-mode auto-arm exhausted transient retries: {error}"
            ) from error
        log(
            "::warning title=auto-arm skipped a cycle on transient GitHub API "
            f"failures::{error} — bounded retries exhausted; a missed sweep is "
            "harmless (the cron backstop covers it), so this run reports success."
        )
        return 0
    return 0


def fixture(
    *,
    number: int = 447,
    labels: tuple[str, ...] = (REVIEW_LABEL,),
    draft: bool = False,
    armed: bool = False,
    oid: str = "a" * 40,
    base: str = "main",
    merge_state: str = "CLEAN",
) -> dict:
    return {
        "id": f"PR_kwDO{number}",
        "number": number,
        "state": "OPEN",
        "isDraft": draft,
        "headRefOid": oid,
        "baseRefName": base,
        "labels": [{"name": label} for label in labels],
        "autoMergeRequest": {"enabledAt": "2026-07-19T00:00:00Z"} if armed else None,
        "mergeStateStatus": merge_state,
    }


class FakeGh:
    def __init__(
        self,
        initial: dict,
        *,
        views: list[dict] | None = None,
        mutation_error: bool = False,
    ) -> None:
        self.initial = copy.deepcopy(initial)
        self.views = [copy.deepcopy(view) for view in (views or [initial])]
        self.mutation_error = mutation_error
        self.calls: list[list[str]] = []

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(argv)
        if argv[:2] == ["pr", "list"]:
            return json.dumps([self.initial])
        if argv[:2] == ["pr", "ready"]:
            return ""
        if argv[:2] == ["pr", "view"]:
            view = self.views.pop(0) if len(self.views) > 1 else self.views[0]
            return json.dumps(view)
        if argv[:2] == ["api", "graphql"]:
            if self.mutation_error:
                raise GhError("simulated expectedHeadOid CAS rejection")
            return json.dumps({"data": {"enablePullRequestAutoMerge": {}}})
        raise AssertionError(f"unexpected fake gh call: {argv}")


def exercise(initial: dict, **fake_kwargs) -> tuple[FakeGh, list[str]]:
    fake = FakeGh(initial, **fake_kwargs)
    messages: list[str] = []
    AutoArmer("sparq-org/sparq", "main", fake, messages.append).run()
    return fake, messages


def graphql_calls(fake: FakeGh) -> list[list[str]]:
    return [call for call in fake.calls if call[:2] == ["api", "graphql"]]


def self_test() -> None:
    expected_guards = {
        "trust-surface",
        "needs:user",
        "review:needs-user",
        "area:sparq-zk",
        "area:sparq-mpc",
        "area:sparq-trust",
    }
    assert HUMAN_OR_TRUST_LABELS == expected_guards, HUMAN_OR_TRUST_LABELS

    # Reviewed, live, non-draft: arm once with the explicit method and head-OID CAS.
    clean = fixture()
    assert parse_pr(clean).merge_state == "CLEAN", clean
    fake, messages = exercise(clean)
    mutations = graphql_calls(fake)
    assert len(mutations) == 1, mutations
    mutation = mutations[0]
    assert "expectedHeadOid:$oid" in " ".join(mutation), mutation
    assert "mergeMethod:SQUASH" in " ".join(mutation), mutation
    assert f"oid={clean['headRefOid']}" in mutation, mutation
    assert any("armed head" in message for message in messages), messages

    # Every named human/trust area is fail-closed before any mutation.
    for label in sorted(expected_guards):
        fake, messages = exercise(fixture(labels=(REVIEW_LABEL, label)))
        assert not graphql_calls(fake), (label, fake.calls)
        assert any("security-surface" in message for message in messages), (label, messages)

    # [FABLE-5] #3759 finding 1: review:changes is a HARD exclusion. A dequeued PR that
    # still carries a STALE review:pass (a race left both on) must NEVER be armed — the
    # fix-lane label wins. Both review-hold labels are checked independent of review:pass.
    for hold in ("review:changes", "review:needs"):
        fake, messages = exercise(fixture(labels=(REVIEW_LABEL, hold)))
        assert not graphql_calls(fake), (hold, fake.calls)
        assert any("changes-requested" in message for message in messages), (hold, messages)
    # It also stops an arm when review:changes appears only on the live refresh.
    fake, messages = exercise(clean, views=[fixture(labels=(REVIEW_LABEL, "review:changes"))])
    assert not graphql_calls(fake), fake.calls
    assert any("changes-requested" in message for message in messages), messages

    # Existing auto-merge arms are idempotent no-ops.
    fake, messages = exercise(fixture(armed=True))
    assert not graphql_calls(fake), fake.calls
    assert any("already-armed" in message for message in messages), messages

    # A hold, review removal, or competing arm that appears on refresh stays fail-closed.
    transitions = (
        (fixture(labels=(REVIEW_LABEL, "trust-surface")), "security-surface"),
        (fixture(labels=()), "review-label-removed"),
        (fixture(armed=True), "already-armed"),
    )
    for refreshed, expected_reason in transitions:
        fake, messages = exercise(clean, views=[refreshed])
        assert not graphql_calls(fake), (expected_reason, fake.calls)
        assert any(expected_reason in message for message in messages), (
            expected_reason,
            messages,
        )

    # Draft workers become ready, are refreshed, then are armed exactly once.
    draft = fixture(draft=True)
    ready = fixture(draft=False)
    fake, messages = exercise(draft, views=[ready])
    assert any(call[:2] == ["pr", "ready"] for call in fake.calls), fake.calls
    assert len(graphql_calls(fake)) == 1, fake.calls
    assert any("draft -> marked ready" in message for message in messages), messages

    # A push between the list snapshot and the live refresh is skipped before mutation.
    moved = fixture(oid="b" * 40)
    fake, messages = exercise(clean, views=[moved])
    assert not graphql_calls(fake), fake.calls
    assert any("head-moved" in message for message in messages), messages

    # A moved head makes the GraphQL CAS fail; classify it and never retry this sweep.
    fake, messages = exercise(
        clean, views=[clean, moved], mutation_error=True
    )
    assert len(graphql_calls(fake)) == 1, fake.calls
    assert any("head-moved" in message for message in messages), messages

    # Stacked PRs must never be armed into their non-default base.
    fake, messages = exercise(fixture(base="feature-base"))
    assert not graphql_calls(fake), fake.calls
    assert any("non-default-base" in message for message in messages), messages

    # [FABLE-5] #3759: READS route through gh_read; MUTATIONS stay on the one-shot
    # runner (the retry helper itself refuses them — see gh_retry.assert_read_only).
    fake = FakeGh(clean)
    read_calls: list[list[str]] = []

    def spy_read(argv: list[str]) -> str:
        read_calls.append(argv)
        gh_retry.assert_read_only(argv)  # every read the armer issues IS a read
        return fake(argv)

    AutoArmer(
        "sparq-org/sparq", "main", fake, lambda _m: None, gh_read=spy_read
    ).run()
    assert [c[:2] for c in read_calls] == [["pr", "list"], ["pr", "view"]], read_calls
    direct = [c for c in fake.calls if c not in read_calls]
    assert [c[:2] for c in direct] == [["api", "graphql"]], fake.calls

    # Exhausted transient retries abort the sweep as ::warning + exit 0 (a missed
    # cycle is harmless); the arm mutation is never attempted on partial state.
    def exhausted(_argv: list[str]) -> str:
        raise gh_retry.GhTransientExhausted("gh pr list: HTTP 504 (3 attempts)")

    fake = FakeGh(clean)
    warnings: list[str] = []
    code = run_sweep(
        AutoArmer("sparq-org/sparq", "main", fake, lambda _m: None, gh_read=exhausted),
        log=warnings.append,
    )
    assert code == 0, code
    assert not fake.calls, fake.calls
    assert warnings and warnings[0].startswith("::warning"), warnings

    # A NON-transient failure still fails loudly: GhError escapes run_sweep untouched.
    def fatal(_argv: list[str]) -> str:
        raise GhError("gh pr list failed: HTTP 403 forbidden")

    try:
        run_sweep(
            AutoArmer("sparq-org/sparq", "main", fake, lambda _m: None, gh_read=fatal),
            log=warnings.append,
        )
    except GhError:
        pass
    else:
        raise AssertionError("non-transient errors must not be swallowed")

    # [FABLE-5] #3759 finding 5 (BLOCKING): a real arm MUTATION failure must be the
    # run's status even when the follow-up race-diagnostic READ exhausts transient
    # retries. Here the arm CAS fails (GhError) and the diagnostic refresh raises
    # GhTransientExhausted; the ORIGINAL mutation error must propagate, NOT be
    # rewritten into a lenient exit-0 success.
    fake = FakeGh(clean, mutation_error=True)

    def read_lists_then_exhausts(argv: list[str]) -> str:
        # The pre-arm reads (pr list, the pre-arm pr view) succeed; the diagnostic
        # refresh AFTER the failed arm is the one that exhausts transients.
        if argv[:2] == ["pr", "view"] and getattr(read_lists_then_exhausts, "armed", False):
            raise gh_retry.GhTransientExhausted("gh pr view: HTTP 504 (3 attempts)")
        if argv[:2] == ["api", "graphql"]:
            read_lists_then_exhausts.armed = True  # type: ignore[attr-defined]
        return fake(argv)

    def gh_marks_arm(argv: list[str]) -> str:
        if argv[:2] == ["api", "graphql"]:
            read_lists_then_exhausts.armed = True  # type: ignore[attr-defined]
        return fake(argv)

    read_lists_then_exhausts.armed = False  # type: ignore[attr-defined]
    try:
        run_sweep(
            AutoArmer(
                "sparq-org/sparq",
                "main",
                gh_marks_arm,
                lambda _m: None,
                gh_read=read_lists_then_exhausts,
            ),
            log=lambda _m: None,
        )
    except GhError as error:
        assert "CAS rejection" in str(error), error
    else:
        raise AssertionError(
            "a failed arm mutation must NOT be masked as success when the follow-up "
            "diagnostic read exhausts its transient retries"
        )

    # [FABLE-5] #3759 finding 5: EVENT-mode (label-triggered) arm must NOT use the
    # sweep's lenient exit — an exhausted transient exits NON-zero (there is no
    # per-PR cron backstop), so the arm is visibly red and retried.
    def exhausted_event(_argv: list[str]) -> str:
        raise gh_retry.GhTransientExhausted("gh pr list: HTTP 504 (3 attempts)")

    fake = FakeGh(clean)
    try:
        run_sweep(
            AutoArmer(
                "sparq-org/sparq", "main", fake, lambda _m: None, gh_read=exhausted_event
            ),
            log=lambda _m: None,
            mode="event",
        )
    except GhError as error:
        assert "event-mode" in str(error), error
    else:
        raise AssertionError("event-mode exhausted transient must exit non-zero")

    # The SAME exhausted transient in SWEEP mode is still the lenient ::warning + 0.
    fake = FakeGh(clean)
    warnings = []
    code = run_sweep(
        AutoArmer(
            "sparq-org/sparq", "main", fake, lambda _m: None, gh_read=exhausted_event
        ),
        log=warnings.append,
        mode="sweep",
    )
    assert code == 0, code
    assert warnings and warnings[0].startswith("::warning"), warnings

    print("auto-arm self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", help="owner/repository to sweep")
    parser.add_argument("--default-branch", default="main")
    parser.add_argument(
        "--mode",
        choices=("sweep", "event"),
        default="sweep",
        help=(
            "sweep: periodic cron (exhausted transient reads => ::warning + exit 0, "
            "the cron backstop covers a missed cycle). event: label-triggered arm on a "
            "single PR (no per-PR backstop — any failure, incl. exhausted transients, "
            "exits non-zero so the arm is visibly red and retried)."
        ),
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required unless --self-test is used")
    return run_sweep(
        AutoArmer(args.repo, args.default_branch, gh_read=run_gh_read), mode=args.mode
    )


if __name__ == "__main__":
    sys.exit(main())
