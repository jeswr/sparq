#!/usr/bin/env python3
"""Re-arm reviewed pull requests whose merge-queue arm was dropped by GitHub."""

# [GPT-5.6] Issue #3675 — restore dropped arms without mistaking queued PRs for drops.
# [FABLE-5] Issue #3759 / registry#563 item 4 — idempotent gh READS (pr list, the
# GraphQL live-state QUERY) go through gh_retry.run_gh_read (bounded, transient-only
# retry); the arm MUTATION (gh pr merge --auto) stays one-shot on the un-wrapped
# runner. Exhausted transient retries end the sweep as ::warning + exit 0 (a missed
# cycle is harmless — the cron covers it) instead of redding main's gate; any
# non-transient error still fails loudly.

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
from dataclasses import dataclass
from typing import Callable

import gh_retry


PROGRAM = "rearm-sweeper"
REVIEW_ATTESTATION = "review:pass"
EXCLUDED_LABELS = frozenset(
    {
        "review:changes",
        "review:needs",
        "review:needs-user",
        "trust:untrusted",
    }
)
DEFAULT_MAX_REARMS = 10
PR_LIST_FIELDS = "number,state,isDraft,baseRefName,labels"
LIVE_QUERY = """query($owner:String!,$name:String!,$number:Int!){
  repository(owner:$owner,name:$name){
    pullRequest(number:$number){
      number
      state
      isDraft
      baseRefName
      labels(first:100){nodes{name} pageInfo{hasNextPage}}
      autoMergeRequest{enabledAt}
      mergeQueueEntry{id}
    }
  }
}"""


class GhError(RuntimeError):
    """A GitHub CLI command failed."""


@dataclass(frozen=True)
class PullRequest:
    number: int
    state: str
    is_draft: bool
    base_ref: str
    labels: frozenset[str]
    has_auto_merge: bool = False
    has_queue_entry: bool = False
    labels_truncated: bool = False


def normalized_labels(raw: object) -> frozenset[str]:
    if not isinstance(raw, list):
        return frozenset()
    return frozenset(
        str(label.get("name", "")).strip().lower()
        for label in raw
        if isinstance(label, dict) and str(label.get("name", "")).strip()
    )


def parse_list_pr(raw: dict) -> PullRequest:
    return PullRequest(
        number=int(raw["number"]),
        state=str(raw.get("state", "")).upper(),
        is_draft=bool(raw.get("isDraft")),
        base_ref=str(raw.get("baseRefName", "")),
        labels=normalized_labels(raw.get("labels")),
    )


def parse_live_pr(number: int, raw: dict | None) -> PullRequest:
    if not isinstance(raw, dict):
        return PullRequest(number, "UNKNOWN", False, "", frozenset())
    labels = raw.get("labels") or {}
    return PullRequest(
        number=int(raw.get("number", number)),
        state=str(raw.get("state", "")).upper(),
        is_draft=bool(raw.get("isDraft")),
        base_ref=str(raw.get("baseRefName", "")),
        labels=normalized_labels(labels.get("nodes")),
        has_auto_merge=raw.get("autoMergeRequest") is not None,
        has_queue_entry=raw.get("mergeQueueEntry") is not None,
        labels_truncated=bool((labels.get("pageInfo") or {}).get("hasNextPage")),
    )


def exclusion_labels(labels: frozenset[str]) -> list[str]:
    """Return every fail-closed hold label, including the whole needs:* namespace."""
    return sorted(
        label
        for label in labels
        if label.startswith("needs:") or label in EXCLUDED_LABELS
    )


def run_gh(argv: list[str]) -> str:
    result = subprocess.run(["gh", *argv], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh failure"
        raise GhError(f"gh {' '.join(argv[:3])} failed: {detail}")
    return result.stdout


def run_gh_read(argv: list[str]) -> str:
    """Idempotent READ path: bounded transient-only retries (#3759).

    Non-transient failures re-raise as :class:`GhError` so the existing per-PR
    error handling keeps working; exhausted transients propagate as
    :class:`gh_retry.GhTransientExhausted` for the ::warning + exit-0 sweep policy.
    """
    try:
        return gh_retry.run_gh_read(argv)
    except gh_retry.GhFatalError as error:
        raise GhError(str(error)) from error


class RearmSweeper:
    def __init__(
        self,
        repo: str,
        default_branch: str,
        *,
        max_rearms: int = DEFAULT_MAX_REARMS,
        gh: Callable[[list[str]], str] = run_gh,
        gh_read: Callable[[list[str]], str] | None = None,
        log: Callable[[str], None] = print,
    ) -> None:
        if not 1 <= max_rearms <= DEFAULT_MAX_REARMS:
            raise ValueError(f"max_rearms must be between 1 and {DEFAULT_MAX_REARMS}")
        self.repo = repo
        self.default_branch = default_branch
        self.max_rearms = max_rearms
        self.gh = gh
        # Idempotent READS may go through a retrying runner (#3759); the arm
        # MUTATION always uses the one-shot `gh` runner above.
        self.gh_read = gh_read if gh_read is not None else gh
        self.log = log
        try:
            self.owner, self.name = repo.split("/", 1)
        except ValueError as error:
            raise ValueError("repo must be OWNER/REPOSITORY") from error
        if not self.owner or not self.name or "/" in self.name:
            raise ValueError("repo must be OWNER/REPOSITORY")

    def decision(self, pr: PullRequest, *, live: bool) -> str | None:
        # State is deliberately first: closed/merged PRs may report other fields as UNKNOWN.
        if pr.state != "OPEN":
            return f"not-open ({pr.state or 'UNKNOWN'})"
        if pr.is_draft:
            return "draft"
        if pr.base_ref != self.default_branch:
            return f"non-default-base ({pr.base_ref or 'UNKNOWN'})"
        if live and pr.labels_truncated:
            return "live label set exceeds query page"
        if REVIEW_ATTESTATION not in pr.labels:
            return "review:pass attestation absent"
        excluded = exclusion_labels(pr.labels)
        if excluded:
            return f"hard exclusion ({', '.join(excluded)})"
        if live and pr.has_queue_entry:
            return "live mergeQueueEntry"
        if live and pr.has_auto_merge:
            return "live autoMergeRequest"
        return None

    def list_candidates(self) -> list[PullRequest]:
        raw = json.loads(
            self.gh_read(
                [
                    "pr",
                    "list",
                    "--repo",
                    self.repo,
                    "--state",
                    "open",
                    "--label",
                    REVIEW_ATTESTATION,
                    "--limit",
                    "1000",
                    "--json",
                    PR_LIST_FIELDS,
                ]
            )
        )
        if not isinstance(raw, list):
            raise GhError("gh pr list returned a non-list response")
        return [parse_list_pr(item) for item in raw]

    def live_state(self, number: int) -> PullRequest:
        response = json.loads(
            self.gh_read(
                [
                    "api",
                    "graphql",
                    "-f",
                    f"query={LIVE_QUERY}",
                    "-f",
                    f"owner={self.owner}",
                    "-f",
                    f"name={self.name}",
                    "-F",
                    f"number={number}",
                ]
            )
        )
        if response.get("errors"):
            raise GhError(f"GraphQL returned errors: {response['errors']}")
        repository = (response.get("data") or {}).get("repository") or {}
        return parse_live_pr(number, repository.get("pullRequest"))

    def arm(self, number: int) -> None:
        # No merge-method flag: the repository's merge queue chooses the strategy.
        self.gh(["pr", "merge", str(number), "--repo", self.repo, "--auto"])

    def emit(self, number: int, verdict: str, detail: str) -> None:
        self.log(f"[{PROGRAM}] PR #{number}: {verdict} — {detail}")

    def run(self) -> int:
        attempts = 0
        errors = 0
        candidates = self.list_candidates()
        self.log(
            f"[{PROGRAM}] found {len(candidates)} open PR(s) returned for "
            f"label {REVIEW_ATTESTATION}; re-arm limit={self.max_rearms}"
        )
        for snapshot in candidates:
            reason = self.decision(snapshot, live=False)
            if reason:
                self.emit(snapshot.number, "SKIP", reason)
                continue

            try:
                current = self.live_state(snapshot.number)
            except (GhError, json.JSONDecodeError) as error:
                self.emit(snapshot.number, "SKIP", f"live-state query failed ({error})")
                errors += 1
                continue

            reason = self.decision(current, live=True)
            if reason:
                self.emit(current.number, "SKIP", reason)
                continue

            # Both fields are absent here: and only here is the arm considered dropped.
            if attempts >= self.max_rearms:
                self.emit(current.number, "SKIP", "per-run re-arm limit reached")
                continue
            attempts += 1
            try:
                self.arm(current.number)
            except GhError as error:
                # The arm MUTATION failed — that failure is the primary status. The
                # follow-up live-state read is a diagnostic to classify an API race,
                # and it is retriable: it can raise GhError, a decode error, OR
                # GhTransientExhausted. [FABLE-5] #3759 finding 5: the diagnostic's
                # OWN exhaustion must never escape here — if it did, main's lenient
                # `except GhTransientExhausted -> exit 0` would convert a genuine
                # failed arm into a false success. Catch exhaustion too; on any
                # inconclusive diagnostic, record the arm failure as a real error.
                try:
                    raced = self.live_state(current.number)
                    race_reason = self.decision(raced, live=True)
                except (GhError, json.JSONDecodeError, gh_retry.GhTransientExhausted):
                    race_reason = None
                if race_reason:
                    self.emit(current.number, "SKIP", f"arm raced: {race_reason}")
                else:
                    self.emit(current.number, "SKIP", f"re-arm failed ({error})")
                    errors += 1
                continue
            self.emit(current.number, "ARMED", "dropped auto-merge request restored")

        self.log(
            f"[{PROGRAM}] complete: candidates={len(candidates)} "
            f"re-arm-attempts={attempts} errors={errors}"
        )
        return errors


def fixture(
    number: int,
    *,
    state: str = "OPEN",
    draft: bool = False,
    base: str = "main",
    labels: tuple[str, ...] = (REVIEW_ATTESTATION,),
    armed: bool = False,
    queued: bool = False,
) -> dict:
    return {
        "number": number,
        "state": state,
        "isDraft": draft,
        "baseRefName": base,
        "labels": [{"name": label} for label in labels],
        "autoMergeRequest": {"enabledAt": "2026-07-21T00:00:00Z"} if armed else None,
        "mergeQueueEntry": {"id": f"MQE_{number}"} if queued else None,
    }


class FakeGh:
    def __init__(self, snapshots: list[dict], live: dict[int, dict]) -> None:
        self.snapshots = copy.deepcopy(snapshots)
        self.live = copy.deepcopy(live)
        self.calls: list[list[str]] = []

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(argv)
        if argv[:2] == ["pr", "list"]:
            return json.dumps(self.snapshots)
        if argv[:2] == ["api", "graphql"]:
            number = int(
                next(arg.split("=", 1)[1] for arg in argv if arg.startswith("number="))
            )
            pr = self.live.get(number)
            if pr is not None:
                pr = copy.deepcopy(pr)
                pr["labels"] = {
                    "nodes": pr["labels"],
                    "pageInfo": {"hasNextPage": False},
                }
            return json.dumps({"data": {"repository": {"pullRequest": pr}}})
        if argv[:2] == ["pr", "merge"]:
            return ""
        raise AssertionError(f"unexpected fake gh call: {argv}")


def arm_calls(fake: FakeGh) -> list[list[str]]:
    return [call for call in fake.calls if call[:2] == ["pr", "merge"]]


def exercise(
    *prs: dict,
    live_prs: tuple[dict, ...] | None = None,
    max_rearms: int = DEFAULT_MAX_REARMS,
):
    snapshots = [
        {
            key: copy.deepcopy(value)
            for key, value in pr.items()
            if key != "mergeQueueEntry" and key != "autoMergeRequest"
        }
        for pr in prs
    ]
    current = live_prs if live_prs is not None else prs
    fake = FakeGh(snapshots, {int(pr["number"]): pr for pr in current})
    messages: list[str] = []
    errors = RearmSweeper(
        "sparq-org/sparq", "main", max_rearms=max_rearms, gh=fake, log=messages.append
    ).run()
    return fake, messages, errors


def self_test() -> None:
    expected_exact_exclusions = {
        "review:changes",
        "review:needs",
        "review:needs-user",
        "trust:untrusted",
    }
    assert EXCLUDED_LABELS == expected_exact_exclusions, EXCLUDED_LABELS

    # Mutation tripwire (a): autoMergeRequest is null while a queue entry is live.
    queued = fixture(3678, queued=True)
    fake, messages, errors = exercise(fixture(3678), live_prs=(queued,))
    assert errors == 0, messages
    assert not arm_calls(fake), fake.calls
    assert any("SKIP" in line and "mergeQueueEntry" in line for line in messages), (
        messages
    )
    query_call = next(call for call in fake.calls if call[:2] == ["api", "graphql"])
    query_text = next(arg for arg in query_call if arg.startswith("query="))
    assert "autoMergeRequest{" in query_text, query_text
    assert "mergeQueueEntry{" in query_text, query_text

    # Mutation tripwire (b): needs:* is a hard exclusion, independent of queue state.
    held = fixture(3682, labels=(REVIEW_ATTESTATION, "needs:user"))
    fake, messages, errors = exercise(fixture(3682), live_prs=(held,))
    assert errors == 0, messages
    assert not arm_calls(fake), fake.calls
    assert any("SKIP" in line and "needs:user" in line for line in messages), messages

    # Mutation tripwire (c): a reviewed PR with neither live field must be re-armed.
    dropped = fixture(3675)
    fake, messages, errors = exercise(dropped)
    assert errors == 0, messages
    assert len(arm_calls(fake)) == 1, fake.calls
    assert arm_calls(fake)[0] == [
        "pr",
        "merge",
        "3675",
        "--repo",
        "sparq-org/sparq",
        "--auto",
    ], arm_calls(fake)
    assert any("ARMED" in line for line in messages), messages

    # CI state alone is never an arm verdict: live removal of review:pass must stop it.
    unattested = fixture(105, labels=())
    fake, messages, errors = exercise(fixture(105), live_prs=(unattested,))
    assert errors == 0, messages
    assert not arm_calls(fake), fake.calls
    assert any("review:pass attestation absent" in line for line in messages), messages
    list_call = next(call for call in fake.calls if call[:2] == ["pr", "list"])
    label_arg = list_call.index("--label")
    assert list_call[label_arg + 1] == REVIEW_ATTESTATION, list_call

    # Every named exclusion is fail-closed, including one added after enumeration.
    for label in (*sorted(EXCLUDED_LABELS), "needs:security"):
        held = fixture(100, labels=(REVIEW_ATTESTATION, label))
        fake, messages, _ = exercise(
            fixture(100),
            live_prs=(held,),
        )
        assert not arm_calls(fake), (label, fake.calls)
        assert any("hard exclusion" in line for line in messages), (label, messages)
    for pr, expected in (
        (fixture(101, armed=True), "autoMergeRequest"),
        (fixture(102, state="CLOSED", base="", queued=True), "not-open"),
        (fixture(103, draft=True), "draft"),
        (fixture(104, base="stacked-base"), "non-default-base"),
    ):
        fake, messages, _ = exercise(
            fixture(pr["number"]),
            live_prs=(pr,),
        )
        assert not arm_calls(fake), fake.calls
        assert any(expected in line for line in messages), messages

    # The hard bound limits commands, while every excess candidate still gets a decision.
    fake, messages, errors = exercise(fixture(201), fixture(202), max_rearms=1)
    assert errors == 0, messages
    assert len(arm_calls(fake)) == 1, fake.calls
    assert any("PR #202: SKIP" in line and "limit" in line for line in messages), (
        messages
    )

    # [FABLE-5] #3759: READS route through gh_read (and really are reads — the retry
    # helper's fail-closed guard accepts them); the arm MUTATION stays on the
    # one-shot runner, so a retry wrapper can never double-fire it.
    dropped = fixture(3759)
    fake = FakeGh(
        [{k: v for k, v in dropped.items() if k not in ("mergeQueueEntry", "autoMergeRequest")}],
        {3759: dropped},
    )
    read_calls: list[list[str]] = []

    def spy_read(argv: list[str]) -> str:
        read_calls.append(argv)
        gh_retry.assert_read_only(argv)
        return fake(argv)

    RearmSweeper(
        "sparq-org/sparq", "main", gh=fake, gh_read=spy_read, log=lambda _m: None
    ).run()
    assert [c[:2] for c in read_calls] == [["pr", "list"], ["api", "graphql"]], (
        read_calls
    )
    assert [c[:2] for c in arm_calls(fake)] == [["pr", "merge"]], fake.calls
    assert all(c[:2] != ["pr", "merge"] for c in read_calls), read_calls

    # [FABLE-5] #3759 finding 5: an arm MUTATION failure must be recorded as a real
    # error even when the follow-up race-diagnostic READ (live_state) exhausts its
    # transient retries. If the diagnostic's GhTransientExhausted escaped run(), main's
    # lenient handler would turn a genuinely failed arm into a false success. Here the
    # arm raises GhError; the post-arm live_state raises GhTransientExhausted.
    dropped = fixture(4001)
    snapshot = {
        k: v for k, v in dropped.items() if k not in ("mergeQueueEntry", "autoMergeRequest")
    }

    class _ArmFailsDiagExhausts:
        def __init__(self) -> None:
            self.armed = False
            self.calls: list[list[str]] = []

        def __call__(self, argv: list[str]) -> str:
            self.calls.append(argv)
            if argv[:2] == ["pr", "list"]:
                return json.dumps([snapshot])
            if argv[:2] == ["api", "graphql"]:
                if self.armed:
                    # The post-arm diagnostic read exhausts transient retries.
                    raise gh_retry.GhTransientExhausted(
                        "gh api graphql: HTTP 504 (3 attempts)"
                    )
                pr = copy.deepcopy(dropped)
                pr["labels"] = {
                    "nodes": pr["labels"],
                    "pageInfo": {"hasNextPage": False},
                }
                return json.dumps({"data": {"repository": {"pullRequest": pr}}})
            if argv[:2] == ["pr", "merge"]:
                self.armed = True
                raise GhError("simulated arm failure")
            raise AssertionError(f"unexpected fake gh call: {argv}")

    fake_ad = _ArmFailsDiagExhausts()
    messages = []
    errors = RearmSweeper(
        "sparq-org/sparq", "main", gh=fake_ad, gh_read=fake_ad, log=messages.append
    ).run()
    assert errors == 1, (errors, messages)
    assert any("re-arm failed" in line for line in messages), messages
    # The diagnostic exhaustion did NOT escape run() (no exception propagated here).

    # [FABLE-5] #3759 finding 5: EVENT-mode exhausted transient on the enumeration read
    # exits NON-zero (no per-PR backstop); SWEEP-mode is the lenient ::warning + 0.
    import io
    from contextlib import redirect_stderr, redirect_stdout

    def _exhausted_read(_argv: list[str]) -> str:
        raise gh_retry.GhTransientExhausted("gh pr list: HTTP 504 (3 attempts)")

    original_run_gh_read = globals()["run_gh_read"]

    class _ModeHarness:
        """Drive main() with a stubbed enumeration read that exhausts transients."""

        def __init__(self, mode: str) -> None:
            self.mode = mode

        def __enter__(self):
            globals()["run_gh_read"] = _exhausted_read
            self._argv = sys.argv
            sys.argv = [
                "rearm-sweeper.py", "--repo", "sparq-org/sparq", "--mode", self.mode,
            ]
            return self

        def __exit__(self, *exc):
            globals()["run_gh_read"] = original_run_gh_read
            sys.argv = self._argv
            return False

    out, err = io.StringIO(), io.StringIO()
    with _ModeHarness("event"), redirect_stdout(out), redirect_stderr(err):
        event_code = main()
    assert event_code == 1, event_code
    assert "event-mode" in err.getvalue(), err.getvalue()

    out, err = io.StringIO(), io.StringIO()
    with _ModeHarness("sweep"), redirect_stdout(out), redirect_stderr(err):
        sweep_code = main()
    assert sweep_code == 0, sweep_code
    assert "::warning" in out.getvalue(), out.getvalue()

    print("rearm-sweeper self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", help="OWNER/REPOSITORY to sweep")
    parser.add_argument("--default-branch", default="main")
    parser.add_argument("--max-rearms", type=int, default=DEFAULT_MAX_REARMS)
    parser.add_argument(
        "--mode",
        choices=("sweep", "event"),
        default="sweep",
        help=(
            "sweep: periodic cron (exhausted transient READS on the enumeration path "
            "=> ::warning + exit 0, the cron backstop covers a missed cycle). event: a "
            "single-PR event-driven invocation with no per-PR backstop — an exhausted "
            "transient exits non-zero so the run is visibly red. This workflow runs "
            "sweep-only today; the flag keeps the exit contract explicit and testable."
        ),
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required unless --self-test is used")
    try:
        sweeper = RearmSweeper(
            args.repo, args.default_branch, max_rearms=args.max_rearms,
            gh_read=run_gh_read,
        )
        return 1 if sweeper.run() else 0
    except gh_retry.GhTransientExhausted as error:
        # [FABLE-5] #3759: only the periodic + idempotent SWEEP may swallow a missed
        # cycle on a transient platform 5xx (the cron backstop covers it). An
        # EVENT-driven invocation has no per-PR backstop, so it fails loudly (finding
        # 5). A failed arm MUTATION never reaches here as GhTransientExhausted — its
        # own diagnostic read cannot overwrite it (see run()).
        if args.mode == "event":
            print(
                f"[{PROGRAM}] fatal: event-mode exhausted transient retries: {error}",
                file=sys.stderr,
            )
            return 1
        print(
            f"::warning title={PROGRAM} skipped a cycle on transient GitHub API "
            f"failures::{error} — bounded retries exhausted; the next cron run "
            "covers this sweep, so this run reports success."
        )
        return 0
    except (GhError, ValueError, json.JSONDecodeError) as error:
        print(f"[{PROGRAM}] fatal: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
