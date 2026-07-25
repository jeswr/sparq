#!/usr/bin/env python3
"""Re-arm reviewed pull requests whose merge-queue arm was dropped by GitHub."""

# [GPT-5.6] Issue #3675 — restore dropped arms without mistaking queued PRs for drops.
# [FABLE-5] Issue #3759 / registry#563 item 4 — idempotent gh READS (pr list, the
# GraphQL live-state QUERY) go through gh_retry.run_gh_read (bounded, transient-only
# retry); the arm MUTATION (gh pr merge --auto) stays one-shot on the un-wrapped
# runner. Exhausted transient retries end the sweep as ::warning + exit 0 (a missed
# cycle is harmless — the cron covers it) instead of redding main's gate; any
# non-transient error still fails loudly.
# [OPUS-5] Issue #3760 — fail LOUDLY ONCE when the token cannot arm at all, and never let
#   a single per-PR arm failure abort the rest of the sweep. A capability denial is NOT a
#   transient: it never routes through the #3759 ::warning + exit-0 path.
# [OPUS-5] Issue #3766 — STICKY FAILURE PRECEDENCE. Collecting the failures (above) was
#   only half the job: the collected state lived in a LOCAL of run(), so a
#   GhTransientExhausted raised while processing a LATER candidate unwound past the
#   accumulated-failure check and main()'s lenient handler reported ::warning + exit 0.
#   Reproduced: PR #3011's arm failed, PR #3012's live-state read exhausted its retries,
#   and the run exited 0 — discarding #3011's real failure. Now the outcome lives on the
#   SWEEPER (RearmSweeper.outcome), a later candidate's exhaustion is recorded as its OWN
#   warning-class per-candidate outcome without escaping the loop, and the exit status is
#   computed at the END by sweep_exit() from that final state.
#   PRECEDENCE: collected-failure > transient-exhaustion > clean.

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Callable


# --------------------------------------------------------------------------------------
# [OPUS-5] Issue #3776 — A MISSING RESILIENCE HELPER MUST NEVER BRICK AN ARM SWEEP.
#
# THE OUTAGE was in the SIBLING script: auto-arm.yml runs on `pull_request`, so its WORKFLOW
# FILE (and therefore its file-by-file `sparse-checkout` manifest) comes from the PR's own
# ref while the SCRIPT comes from `ref: default_branch`. #3766 added `import gh_retry` and
# the matching manifest entry in one commit — atomic on main, not across refs — so every
# stale PR ref got the new script without gh_retry.py and died with
# `ModuleNotFoundError: No module named 'gh_retry'` on a GATING check (sparq #3434, run
# 30143852994). See scripts/auto-arm.py for the full account.
#
# THIS script is NOT exposed to that ref skew today: rearm-sweeper.yml triggers only on
# `schedule` / `workflow_dispatch`, so its workflow file and its `ref: default_branch`
# checkout are always the same commit, and it is explicitly NOT A GATE. The guard is here
# for the two ways that changes: adding any per-ref trigger to rearm-sweeper.yml, or adding
# a second sibling import without remembering the manifest. Same class, same remedy — a
# missing resilience helper must degrade, not abort.
#
# THE DEGRADATION, EXPLICITLY. gh_retry is a RESILIENCE helper (bounded, transient-only
# retry for idempotent READS). Losing it must cost RETRIES — never the arm:
#   * reads become ONE-SHOT via _DegradedGhRetry.run_gh_read. The load-bearing work is
#     unchanged: candidates are still enumerated, every skip rule is still evaluated, a
#     draft is still marked ready, and the arm mutation is still issued.
#   * FAIL-CLOSED classification. With no retries there is nothing to exhaust, so a read
#     failure raises GhFatalError (loud red). It is NEVER reported as GhTransientExhausted:
#     that type is precisely what #3759 converts into ::warning + exit 0, so synthesising
#     it here would turn a 403 into a false success. Degraded mode therefore trades
#     transient tolerance for LOUDNESS, which is the safe direction — and strictly better
#     than the guaranteed red it replaces.
#   * assert_read_only stays a REAL fail-closed guard, not a no-op, so the self-test
#     assertion "every read this script issues IS a read" cannot go vacuous when degraded.
#   * exactly ONE loud ::warning at import time names the cause and the remedy.
# Deliberately DUPLICATED into the sibling arm script — each workflow sparse-checks out only
# its own script, so neither may import a shared helper (that very constraint is what caused
# this outage). scripts/tests/test_arm_capability_wiring.py pins the two copies together and
# proves the degraded path still mutates.
class _DegradedGhRetry:
    """One-shot stand-in for scripts/gh_retry.py when that file was not checked out."""

    # Mirrors gh_retry._READ_SUBCOMMANDS.
    READ_SUBCOMMANDS = frozenset(
        {
            ("pr", "list"),
            ("pr", "view"),
            ("pr", "checks"),
            ("pr", "status"),
            ("issue", "list"),
            ("issue", "view"),
            ("run", "list"),
            ("run", "view"),
            ("label", "list"),
            ("search", "prs"),
            ("search", "issues"),
        }
    )
    MUTATION_RE = re.compile(r"\b(?:mutation|subscription)\b", re.IGNORECASE)
    QUERY_FIELD_FLAGS = ("-f", "--field", "-F", "--raw-field")
    FIELD_FLAGS = ("-f", "-F", "--field", "--raw-field", "--input")

    class GhRetryUsageError(ValueError):
        """argv could not be PROVEN a read. Refused, never wrapped."""

    class GhFatalError(RuntimeError):
        """A non-retried ``gh`` failure — the caller must fail loudly."""

    class GhTransientExhausted(RuntimeError):
        """Never raised by this stand-in: with zero retries there is nothing to exhaust.

        Kept so ``except gh_retry.GhTransientExhausted`` clauses stay valid (and so the
        self-test can still raise it through an injected runner).
        """

    @classmethod
    def _field_values(cls, rest: list[str], flags: tuple[str, ...]) -> list[str]:
        values: list[str] = []
        index = 0
        while index < len(rest):
            arg = rest[index]
            if arg in flags:
                values.append(rest[index + 1] if index + 1 < len(rest) else "")
                index += 2
                continue
            for flag in flags:
                if arg.startswith(flag + "="):
                    values.append(arg.split("=", 1)[1])
                    break
            index += 1
        return values

    @classmethod
    def assert_read_only(cls, argv) -> None:
        """Fail-closed: refuse anything not AFFIRMATIVELY provable as a read."""
        rest = [str(arg) for arg in argv]
        if not rest:
            raise cls.GhRetryUsageError("empty gh argv")
        if rest[0] != "api":
            if tuple(rest[:2]) in cls.READ_SUBCOMMANDS:
                return
            raise cls.GhRetryUsageError(
                f"gh {' '.join(rest[:2])} is not an allow-listed read — "
                "mutations/arm calls stay one-shot"
            )
        tail = rest[1:]
        method = None
        for index, arg in enumerate(tail):
            if arg in ("-X", "--method"):
                method = tail[index + 1].upper() if index + 1 < len(tail) else ""
                break
            if arg.startswith("--method="):
                method = arg.split("=", 1)[1].upper()
                break
        if "graphql" in tail:
            inline = None
            for value in cls._field_values(tail, cls.QUERY_FIELD_FLAGS):
                if value.startswith("query=") and not value.startswith("query=@"):
                    inline = value.split("=", 1)[1]
            if inline is None:
                raise cls.GhRetryUsageError(
                    "refusing gh api graphql with no inline `query=` text (file-backed / "
                    "stdin bodies are opaque) — cannot prove it is a read"
                )
            if cls.MUTATION_RE.search(inline):
                raise cls.GhRetryUsageError(
                    "refusing a GraphQL mutation/subscription — arms stay one-shot"
                )
            return
        if method not in (None, "GET", "HEAD"):
            raise cls.GhRetryUsageError(
                f"refusing gh api with method {method or '<missing>'}"
            )
        if method is None and any(
            arg in cls.FIELD_FLAGS
            or any(arg.startswith(flag + "=") for flag in cls.FIELD_FLAGS[2:])
            for arg in tail
        ):
            raise cls.GhRetryUsageError(
                "refusing gh api with field params and no explicit GET "
                "(gh auto-switches to POST)"
            )

    @classmethod
    def run_gh_read(cls, argv, *, run=subprocess.run) -> str:
        """Do the read ONCE. No retry, and no transient classification (see above).

        ``run`` is injectable so the wiring test can prove a failing read raises
        GhFatalError and NEVER the lenient GhTransientExhausted.
        """
        cls.assert_read_only(argv)
        command = ["gh", *[str(arg) for arg in argv]]
        result = run(command, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            detail = (
                (result.stderr or "").strip()
                or (result.stdout or "").strip()
                or "unknown gh failure"
            )
            raise cls.GhFatalError(
                f"{' '.join(command[:4])} failed (degraded: scripts/gh_retry.py was not "
                f"checked out, so this read was NOT retried): {detail}"
            )
        return result.stdout


try:
    import gh_retry

    GH_RETRY_DEGRADED = False
except ImportError as _gh_retry_missing:  # pragma: no cover - see the wiring test
    gh_retry = _DegradedGhRetry  # type: ignore[assignment]
    GH_RETRY_DEGRADED = True
    print(
        "::warning title=rearm-sweeper running WITHOUT transient-retry (scripts/gh_retry.py "
        f"not checked out)::{_gh_retry_missing} — this run's idempotent gh READS are "
        "ONE-SHOT: a GitHub 5xx blip will red it instead of being retried. Arming itself "
        "is UNAFFECTED and proceeds. CAUSE: this script came from the default branch "
        "while the `sparse-checkout` manifest came from an older workflow snapshot on the "
        "PR ref, which does not list scripts/gh_retry.py (sparq #3776 / #3766). REMEDY: "
        "rebase the PR onto the default branch, or re-run once the PR ref carries a "
        "workflow file that sparse-checks out the whole scripts/ directory."
    )


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

# --------------------------------------------------------------------------------------
# [OPUS-5] #3760 — ARM CAPABILITY.
#
# LIVE ROOT CAUSE. Sweeper run 30033315483 (2026-07-23T18:21Z) failed arming PR #3454
# (OPEN, non-draft, CLEAN, review:pass) with
#     GraphQL: Resource not accessible by integration (enablePullRequestAutoMerge)
# while auto-arm run 30027010495 (2026-07-23T16:52Z, ~90 minutes earlier, SAME repo, SAME
# plain GITHUB_TOKEN, no App token) armed two PRs successfully. The ONLY difference was the
# job `permissions:` block: auto-arm.yml declared `contents: write`, rearm-sweeper.yml
# declared `contents: read`. Enabling auto-merge is a repository WRITE operation, so
# `contents: read` is denied with exactly this error. It is NOT a maintainer-only setting:
# `allow_auto_merge` is already true on the repository and the `main` ruleset carries no
# integration restriction.
#
# PROBE DESIGN — what is and is NOT a usable capability signal (all measured):
#   * Repository.autoMergeAllowed IS usable: it is the repository "Allow auto-merge"
#     setting, readable read-only, and false means no token can ever arm here.
#   * Repository.viewerPermission is NOT usable: GitHub documents it as "will return null
#     if authenticated as a GitHub App", and the Actions GITHUB_TOKEN *is* an App
#     installation token. Kept in the query as diagnostics only — never gate on it.
#   * PullRequest.viewerCanEnableAutoMerge is NOT a token-capability signal: measured
#     false, under an ADMIN user token, for already-armed (#2521), queued (#3764), draft
#     and merged PRs. It conflates PR state with permission, so gating on it would refuse
#     legitimate arms.
#   * A mutation against a bogus node id is NOT a probe: an authorized token gets
#     NOT_FOUND ("Could not resolve to a node ..."), so the denial class cannot be
#     distinguished without actually being denied.
# Hence: the startup probe fails loud on the signals it can read decisively, stays
# INCONCLUSIVE (never a false red) otherwise, and the first real denial from the mutation
# itself is promoted to a capability verdict — one ::error, sweep stopped, exit 1 once.
CAPABILITY_QUERY = """query($owner:String!,$name:String!){
  repository(owner:$owner,name:$name){
    autoMergeAllowed
    viewerPermission
  }
  viewer{login}
}"""

# Substrings that mark a token-capability denial rather than a per-PR condition. Matched
# case-insensitively against the whole gh/GraphQL failure text. Deliberately disjoint from
# the #3759 transient set: a denial is permanent and must never be swallowed as a transient.
ARM_DENIAL_MARKERS = (
    "resource not accessible by integration",
    "must have write access",
    "must have admin access",
    "not authorized to enable auto-merge",
    "auto merge is not allowed for this repository",
    "auto-merge is not allowed for this repository",
)

# One remediation string, naming every exact thing a human can flip, in fix order.
ARM_REMEDIATION = (
    "the arm token cannot call enablePullRequestAutoMerge. Fix in this order: "
    "(1) .github/workflows/rearm-sweeper.yml MUST declare `permissions:` with BOTH "
    "`contents: write` and `pull-requests: write` — enabling auto-merge is a repository "
    "WRITE operation and `contents: read` is denied with exactly this error (live proof: "
    "sweeper run 30033315483 failed with contents: read while auto-arm run 30027010495 "
    "armed successfully with contents: write on the same repo and the same GITHUB_TOKEN "
    "90 minutes earlier); "
    "(2) repository Settings -> General -> Pull Requests -> 'Allow auto-merge' must be ON "
    "(GraphQL Repository.autoMergeAllowed must be true); "
    "(3) if the repository's `main` ruleset gains a restriction on which integrations may "
    "write, github-actions[bot] must be allowed or the App path used instead; "
    "(4) to arm as the sparq-orchestrator App instead of GITHUB_TOKEN, set the "
    "repository/organization secrets ORCHESTRATOR_APP_ID and ORCHESTRATOR_APP_PRIVATE_KEY "
    "(both are currently UNSET, so the mint step is skipped and the job falls back to "
    "GITHUB_TOKEN)."
)

CAN_ARM = "can-arm"
CANNOT_ARM = "cannot-arm"
INCONCLUSIVE = "inconclusive"


def is_arm_denial(text: str) -> bool:
    """True when a failure text is a token-capability denial, not a per-PR condition."""
    lowered = str(text).lower()
    return any(marker in lowered for marker in ARM_DENIAL_MARKERS)


@dataclass(frozen=True)
class CapabilityVerdict:
    status: str
    detail: str

    @property
    def blocks_sweep(self) -> bool:
        """Only a decisive cannot-arm blocks; inconclusive must never red the run."""
        return self.status == CANNOT_ARM


@dataclass
class SweepOutcome:
    candidates: int = 0
    attempts: int = 0
    armed: int = 0
    # (number, reason) for every PR whose arm was attempted and failed.
    arm_failures: list[tuple[int, str]] = field(default_factory=list)
    # Live-state query failures: not arm failures, but still surfaced and still red
    # (this preserves the pre-#3760 `errors` contract for the diagnostic read).
    state_failures: list[tuple[int, str]] = field(default_factory=list)
    capability: str = INCONCLUSIVE
    # [OPUS-5] #3766 — warning-class per-candidate outcomes: (number, detail) for every
    # candidate whose own bounded read retries were exhausted. These NEVER downgrade a
    # collected failure; they only matter when they are the run's ONLY problem.
    transient_exhaustions: list[tuple[int, str]] = field(default_factory=list)
    # Whole-sweep transient exhaustion (the enumeration read, or a backstop catch).
    sweep_transient: str | None = None

    @property
    def capability_failed(self) -> bool:
        return self.capability == CANNOT_ARM

    @property
    def hard_failed(self) -> bool:
        """A COLLECTED failure: the verdict that outranks every later transient (#3766)."""
        return bool(self.capability_failed or self.arm_failures or self.state_failures)

    @property
    def transient_detail(self) -> str | None:
        """The first transient exhaustion, or None. Warning-class — never a verdict."""
        if self.sweep_transient:
            return self.sweep_transient
        if self.transient_exhaustions:
            number, reason = self.transient_exhaustions[0]
            return f"PR #{number}: {reason}"
        return None

    @property
    def exit_code(self) -> int:
        """Never exit 0 while an arm failed — a silent 0 is how #3760 hid for days.

        Deliberately independent of ``transient_exhaustions`` — a transient's exit is
        MODE-dependent (#3759) and is applied by :func:`sweep_exit`, whereas a collected
        failure is non-zero in every mode.
        """
        return 1 if self.hard_failed else 0


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
        # [OPUS-5] #3760 latches: the fleet must see exactly ONE ::error per run, never
        # one per PR, and the sweep must stop once the token is known to be unable to arm.
        self.capability_error_emitted = False
        self.capability_lost = False
        # [OPUS-5] #3766: the collected outcome lives HERE, not only in a local of run(),
        # so an exception escaping run() can never discard an already-earned failure.
        self.outcome = SweepOutcome()
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

    def probe_arm_capability(self) -> CapabilityVerdict:
        """Read-only startup probe: can this token arm at all in this repository?

        Decisive only in the directions it can actually read (see the ARM CAPABILITY note
        above). Anything unreadable is INCONCLUSIVE and must not red the run — the real
        mutation is the backstop and its denial is promoted to a capability verdict. An
        exhausted #3759 transient is likewise inconclusive: the sweep's own retrying reads
        remain the authority on whether the cycle can proceed.
        """
        try:
            raw = self.gh_read(
                [
                    "api",
                    "graphql",
                    "-f",
                    f"query={CAPABILITY_QUERY}",
                    "-f",
                    f"owner={self.owner}",
                    "-f",
                    f"name={self.name}",
                ]
            )
        except gh_retry.GhTransientExhausted as error:
            return CapabilityVerdict(
                INCONCLUSIVE, f"capability query exhausted transient retries ({error})"
            )
        except GhError as error:
            if is_arm_denial(str(error)):
                return CapabilityVerdict(
                    CANNOT_ARM, f"the arm token was denied by GitHub ({error})"
                )
            return CapabilityVerdict(
                INCONCLUSIVE, f"capability query failed, assuming armable ({error})"
            )
        try:
            response = json.loads(raw)
        except json.JSONDecodeError as error:
            return CapabilityVerdict(
                INCONCLUSIVE, f"capability query returned non-JSON ({error})"
            )
        if response.get("errors"):
            detail = json.dumps(response["errors"], sort_keys=True)
            if is_arm_denial(detail):
                return CapabilityVerdict(
                    CANNOT_ARM, f"the arm token was denied by GitHub ({detail})"
                )
            return CapabilityVerdict(
                INCONCLUSIVE, f"capability query returned errors ({detail})"
            )
        data = response.get("data") or {}
        repository = data.get("repository")
        if not isinstance(repository, dict):
            return CapabilityVerdict(
                INCONCLUSIVE, "capability query returned no repository node"
            )
        # Diagnostics only — null for App tokens, so it can never be a gate.
        login = str((data.get("viewer") or {}).get("login") or "unknown")
        permission = repository.get("viewerPermission")
        allowed = repository.get("autoMergeAllowed")
        if allowed is False:
            return CapabilityVerdict(
                CANNOT_ARM,
                "repository setting 'Allow auto-merge' is OFF "
                "(GraphQL Repository.autoMergeAllowed=false)",
            )
        if allowed is not True:
            return CapabilityVerdict(
                INCONCLUSIVE,
                f"autoMergeAllowed not reported (token={login}, "
                f"viewerPermission={permission})",
            )
        return CapabilityVerdict(
            CAN_ARM,
            f"autoMergeAllowed=true (token={login}, viewerPermission={permission}); "
            "write scope is confirmed by the first arm",
        )

    def fail_capability(self, detail: str) -> None:
        """Emit the single ::error for this run and stop attempting further arms."""
        self.capability_lost = True
        if self.capability_error_emitted:
            return
        self.capability_error_emitted = True
        self.log(
            f"::error title={PROGRAM} cannot enable auto-merge::{detail} — "
            f"{ARM_REMEDIATION}"
        )

    def run(self) -> SweepOutcome:
        # [OPUS-5] #3766: publish the outcome on the SWEEPER before doing any work, so
        # every failure collected below survives any exception that escapes this method and
        # the exit status can be computed at the END from this state (see sweep_exit).
        outcome = self.outcome = SweepOutcome()
        verdict = self.probe_arm_capability()
        outcome.capability = verdict.status
        self.log(f"[{PROGRAM}] arm-capability probe: {verdict.status} — {verdict.detail}")
        if verdict.blocks_sweep:
            # Fail ONCE, before touching any PR: a broken token would otherwise fail
            # identically on every candidate, every ten minutes, forever. This is NOT a
            # transient, so it never reaches the #3759 ::warning + exit-0 path.
            self.fail_capability(f"startup arm-capability probe failed: {verdict.detail}")
            self.log(
                f"[{PROGRAM}] complete: candidates=0 re-arm-attempts=0 armed=0 "
                f"arm-failures=0 state-failures=0 capability={verdict.status}"
            )
            return outcome

        try:
            candidates = self.list_candidates()
        except gh_retry.GhTransientExhausted as error:
            # Whole-sweep transient: nothing has been collected yet, so this is a plain
            # missed cycle. Recorded, not raised — the exit is decided at the end.
            outcome.sweep_transient = str(error)
            self.log(
                f"[{PROGRAM}] candidate enumeration exhausted transient retries ({error})"
            )
            candidates = []
        outcome.candidates = len(candidates)
        self.log(
            f"[{PROGRAM}] found {len(candidates)} open PR(s) returned for "
            f"label {REVIEW_ATTESTATION}; re-arm limit={self.max_rearms}"
        )
        for snapshot in candidates:
            # [OPUS-5] #3766 (BLOCKING, reproduced by cross-provider review): every read in
            # this body is retriable, so a LATER candidate's exhausted transient could
            # escape the loop, unwind run(), and land in main()'s lenient
            # `except GhTransientExhausted -> ::warning + exit 0` — discarding an EARLIER
            # candidate's collected arm failure (PR #3011 arm-failed, PR #3012 live-state
            # exhausted, run reported success). The exhaustion is now this candidate's OWN
            # warning-class outcome: the sweep continues and the verdict #3011 already
            # earned is untouchable.
            try:
                if self.capability_lost:
                    self.emit(
                        snapshot.number,
                        "SKIP",
                        "arm capability lost this run (see the single ::error above)",
                    )
                    continue

                reason = self.decision(snapshot, live=False)
                if reason:
                    self.emit(snapshot.number, "SKIP", reason)
                    continue

                try:
                    current = self.live_state(snapshot.number)
                except (GhError, json.JSONDecodeError) as error:
                    self.emit(
                        snapshot.number, "SKIP", f"live-state query failed ({error})"
                    )
                    outcome.state_failures.append((snapshot.number, str(error)))
                    continue

                reason = self.decision(current, live=True)
                if reason:
                    self.emit(current.number, "SKIP", reason)
                    continue

                # Both fields are absent here: and only here is the arm considered dropped.
                if outcome.attempts >= self.max_rearms:
                    self.emit(current.number, "SKIP", "per-run re-arm limit reached")
                    continue
                outcome.attempts += 1
                try:
                    self.arm(current.number)
                except gh_retry.GhTransientExhausted as exhausted:
                    # [OPUS-5] #3766, fail CLOSED. The arm runs on the ONE-SHOT runner,
                    # which never raises this today (gh_retry refuses to wrap mutations) —
                    # but if it ever did, the mutation's outcome would be UNKNOWN, and an
                    # unknown arm must be a per-PR FAILURE, never the lenient
                    # warning-class outcome a transient normally gets.
                    detail = (
                        "re-arm failed (transient exhaustion on the arm mutation "
                        f"itself: {exhausted})"
                    )
                    self.emit(current.number, "ARM-FAILED", detail)
                    outcome.arm_failures.append((current.number, detail))
                    continue
                except GhError as error:
                    message = str(error)
                    # [OPUS-5] #3760: a capability denial is NOT a per-PR condition — every
                    # remaining candidate would fail identically. Record it, stop arming,
                    # and surface exactly ONE ::error naming what to change.
                    if is_arm_denial(message):
                        self.emit(
                            current.number,
                            "ARM-FAILED",
                            f"arm capability denied ({message})",
                        )
                        outcome.arm_failures.append(
                            (current.number, f"arm capability denied ({message})")
                        )
                        outcome.capability = CANNOT_ARM
                        self.fail_capability(
                            f"arming PR #{current.number} was denied: {message}"
                        )
                        continue
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
                    except (
                        GhError,
                        json.JSONDecodeError,
                        gh_retry.GhTransientExhausted,
                    ):
                        race_reason = None
                    if race_reason:
                        self.emit(current.number, "SKIP", f"arm raced: {race_reason}")
                    else:
                        # Collect and CONTINUE — one bad PR must never abort the sweep.
                        self.emit(
                            current.number, "ARM-FAILED", f"re-arm failed ({message})"
                        )
                        outcome.arm_failures.append(
                            (current.number, f"re-arm failed ({message})")
                        )
                    continue
            except gh_retry.GhTransientExhausted as error:
                outcome.transient_exhaustions.append((snapshot.number, str(error)))
                self.emit(
                    snapshot.number,
                    "SKIP",
                    f"transient-exhausted ({error}); the sweep continues and this "
                    "cannot downgrade an earlier arm failure",
                )
                continue
            outcome.armed += 1
            self.emit(current.number, "ARMED", "dropped auto-merge request restored")

        for number, reason in outcome.arm_failures:
            self.log(f"[{PROGRAM}] arm-failure summary: PR #{number} — {reason}")
        for number, reason in outcome.state_failures:
            self.log(f"[{PROGRAM}] state-failure summary: PR #{number} — {reason}")
        for number, reason in outcome.transient_exhaustions:
            self.log(f"[{PROGRAM}] transient-exhaustion summary: PR #{number} — {reason}")
        self.log(
            f"[{PROGRAM}] complete: candidates={outcome.candidates} "
            f"re-arm-attempts={outcome.attempts} armed={outcome.armed} "
            f"arm-failures={len(outcome.arm_failures)} "
            f"state-failures={len(outcome.state_failures)} "
            f"transient-exhaustions={len(outcome.transient_exhaustions)} "
            f"capability={outcome.capability}"
        )
        return outcome


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


CAPABILITY_RESPONSE = {
    "data": {
        "repository": {"autoMergeAllowed": True, "viewerPermission": None},
        "viewer": {"login": "github-actions[bot]"},
    }
}
# The exact text GitHub returned in run 30033315483 — the #3760 regression anchor.
DENIAL_TEXT = (
    "gh pr merge 3454 failed: GraphQL: Resource not accessible by integration "
    "(enablePullRequestAutoMerge)"
)


def capability_payload(auto_merge_allowed: bool | None) -> str:
    payload = copy.deepcopy(CAPABILITY_RESPONSE)
    payload["data"]["repository"]["autoMergeAllowed"] = auto_merge_allowed
    return json.dumps(payload)


def is_capability_query(argv: list[str]) -> bool:
    return argv[:2] == ["api", "graphql"] and any(
        "autoMergeAllowed" in arg for arg in argv
    )


class FakeGh:
    def __init__(
        self,
        snapshots: list[dict],
        live: dict[int, dict],
        *,
        auto_merge_allowed: bool | None = True,
        capability_error: str | None = None,
        arm_errors: dict[int, str] | None = None,
    ) -> None:
        self.snapshots = copy.deepcopy(snapshots)
        self.live = copy.deepcopy(live)
        self.auto_merge_allowed = auto_merge_allowed
        self.capability_error = capability_error
        self.arm_errors = dict(arm_errors or {})
        self.calls: list[list[str]] = []

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(argv)
        if argv[:2] == ["pr", "list"]:
            return json.dumps(self.snapshots)
        if is_capability_query(argv):
            if self.capability_error is not None:
                raise GhError(self.capability_error)
            return capability_payload(self.auto_merge_allowed)
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
            failure = self.arm_errors.get(int(argv[2]))
            if failure is not None:
                raise GhError(failure)
            return ""
        raise AssertionError(f"unexpected fake gh call: {argv}")


def arm_calls(fake: FakeGh) -> list[list[str]]:
    return [call for call in fake.calls if call[:2] == ["pr", "merge"]]


def probe_query_calls(fake: FakeGh) -> list[list[str]]:
    return [call for call in fake.calls if is_capability_query(call)]


def exercise(
    *prs: dict,
    live_prs: tuple[dict, ...] | None = None,
    max_rearms: int = DEFAULT_MAX_REARMS,
    auto_merge_allowed: bool | None = True,
    capability_error: str | None = None,
    arm_errors: dict[int, str] | None = None,
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
    fake = FakeGh(
        snapshots,
        {int(pr["number"]): pr for pr in current},
        auto_merge_allowed=auto_merge_allowed,
        capability_error=capability_error,
        arm_errors=arm_errors,
    )
    messages: list[str] = []
    outcome = RearmSweeper(
        "sparq-org/sparq", "main", max_rearms=max_rearms, gh=fake, log=messages.append
    ).run()
    return fake, messages, outcome


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
    fake, messages, outcome = exercise(fixture(3678), live_prs=(queued,))
    assert outcome.exit_code == 0, messages
    assert not arm_calls(fake), fake.calls
    assert any("SKIP" in line and "mergeQueueEntry" in line for line in messages), (
        messages
    )
    query_call = next(
        call
        for call in fake.calls
        if call[:2] == ["api", "graphql"] and not is_capability_query(call)
    )
    query_text = next(arg for arg in query_call if arg.startswith("query="))
    assert "autoMergeRequest{" in query_text, query_text
    assert "mergeQueueEntry{" in query_text, query_text

    # Mutation tripwire (b): needs:* is a hard exclusion, independent of queue state.
    held = fixture(3682, labels=(REVIEW_ATTESTATION, "needs:user"))
    fake, messages, outcome = exercise(fixture(3682), live_prs=(held,))
    assert outcome.exit_code == 0, messages
    assert not arm_calls(fake), fake.calls
    assert any("SKIP" in line and "needs:user" in line for line in messages), messages

    # Mutation tripwire (c): a reviewed PR with neither live field must be re-armed.
    dropped = fixture(3675)
    fake, messages, outcome = exercise(dropped)
    assert outcome.exit_code == 0, messages
    assert outcome.armed == 1, outcome
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
    fake, messages, outcome = exercise(fixture(105), live_prs=(unattested,))
    assert outcome.exit_code == 0, messages
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
    fake, messages, outcome = exercise(fixture(201), fixture(202), max_rearms=1)
    assert outcome.exit_code == 0, messages
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
    # [OPUS-5] #3760: the capability probe is itself an idempotent READ, so it routes
    # through gh_read (the retrying runner) and precedes the enumeration.
    assert [c[:2] for c in read_calls] == [
        ["api", "graphql"],
        ["pr", "list"],
        ["api", "graphql"],
    ], read_calls
    assert is_capability_query(read_calls[0]), read_calls[0]
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
            if is_capability_query(argv):
                return capability_payload(True)
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
    outcome = RearmSweeper(
        "sparq-org/sparq", "main", gh=fake_ad, gh_read=fake_ad, log=messages.append
    ).run()
    assert outcome.exit_code == 1, (outcome, messages)
    assert len(outcome.arm_failures) == 1, outcome
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

    # ---------------------------------------------------------------------------------
    # [OPUS-5] #3760 — ARM CAPABILITY + PER-PR FAILURE ISOLATION.
    # ---------------------------------------------------------------------------------

    # The live #3760 error text must classify as a CAPABILITY denial, and near-misses
    # must NOT — a race/CAS/transient stays per-PR, and the denial set must stay disjoint
    # from #3759's transient set (a denial must never be swallowed as ::warning + 0).
    assert is_arm_denial(DENIAL_TEXT), DENIAL_TEXT
    assert is_arm_denial(DENIAL_TEXT.upper()), "denial matching must be case-insensitive"
    for benign in (
        "gh pr merge 1 failed: GraphQL: Head branch was modified. Review and try again",
        "gh pr merge 1 failed: HTTP 502: 502 Bad Gateway",
        "gh pr merge 1 failed: HTTP 504: Gateway Timeout",
        "gh pr merge 1 failed: Pull request is in unstable status",
    ):
        assert not is_arm_denial(benign), benign

    # (1) PROBE — can-arm: the sweep proceeds, the verdict is logged, exit 0, and the
    # probe really runs BEFORE any arm so a broken token never touches a PR.
    fake, messages, outcome = exercise(fixture(3001))
    assert outcome.capability == CAN_ARM, outcome
    assert outcome.exit_code == 0, messages
    assert len(arm_calls(fake)) == 1, fake.calls
    assert any("arm-capability probe: can-arm" in line for line in messages), messages
    assert len(probe_query_calls(fake)) == 1, fake.calls
    assert fake.calls.index(probe_query_calls(fake)[0]) < min(
        index for index, call in enumerate(fake.calls) if call[:2] == ["pr", "merge"]
    ), fake.calls

    # (2) PROBE — cannot-arm (repository setting OFF): ONE ::error naming the exact
    # setting, ZERO PRs touched (not even enumerated), exit 1.
    fake, messages, outcome = exercise(fixture(3002), auto_merge_allowed=False)
    assert outcome.capability == CANNOT_ARM, outcome
    assert outcome.exit_code == 1, messages
    assert not arm_calls(fake), fake.calls
    assert not [call for call in fake.calls if call[:2] == ["pr", "list"]], fake.calls
    emitted = [line for line in messages if line.startswith("::error")]
    assert len(emitted) == 1, emitted
    assert "Allow auto-merge" in emitted[0], emitted
    assert "contents: write" in emitted[0], emitted
    assert "ORCHESTRATOR_APP_ID" in emitted[0], emitted

    # (3) PROBE — cannot-arm (the token itself is denied): same single loud error.
    fake, messages, outcome = exercise(
        fixture(3003),
        capability_error="gh api graphql failed: Resource not accessible by integration",
    )
    assert outcome.capability == CANNOT_ARM, outcome
    assert outcome.exit_code == 1, messages
    assert not arm_calls(fake), fake.calls
    assert len([line for line in messages if line.startswith("::error")]) == 1, messages

    # (4) PROBE — inconclusive must NEVER red the run or stop the sweep: a transient 502,
    # a repository that does not report the field, and an exhausted #3759 retry all pass.
    for kwargs in (
        {"capability_error": "gh api graphql failed: HTTP 502: 502 Bad Gateway"},
        {"auto_merge_allowed": None},
    ):
        fake, messages, outcome = exercise(fixture(3004), **kwargs)
        assert outcome.capability == INCONCLUSIVE, (kwargs, outcome)
        assert outcome.exit_code == 0, (kwargs, messages)
        assert len(arm_calls(fake)) == 1, (kwargs, fake.calls)
        assert not [line for line in messages if line.startswith("::error")], messages

    exhausting = FakeGh([], {})

    def _probe_exhausts(argv: list[str]) -> str:
        if is_capability_query(argv):
            raise gh_retry.GhTransientExhausted("gh api graphql: HTTP 504 (3 attempts)")
        return exhausting(argv)

    probe_verdict = RearmSweeper(
        "sparq-org/sparq", "main", gh=exhausting, gh_read=_probe_exhausts,
        log=lambda _line: None,
    ).probe_arm_capability()
    assert probe_verdict.status == INCONCLUSIVE, probe_verdict
    assert "transient" in probe_verdict.detail, probe_verdict

    # (5) PER-PR ARM FAILURE — a non-capability failure on the FIRST PR must not abort
    # the sweep: the second PR is still armed, the failure is summarised, exit is 1.
    fake, messages, outcome = exercise(
        fixture(3011),
        fixture(3012),
        arm_errors={3011: "gh pr merge 3011 failed: Pull request is in unstable status"},
    )
    assert [call[2] for call in arm_calls(fake)] == ["3011", "3012"], fake.calls
    assert outcome.armed == 1, outcome
    assert [number for number, _ in outcome.arm_failures] == [3011], outcome
    assert outcome.exit_code == 1, messages
    assert any("PR #3011: ARM-FAILED" in line for line in messages), messages
    assert any("PR #3012: ARMED" in line for line in messages), messages
    assert any("arm-failure summary: PR #3011" in line for line in messages), messages
    assert any(
        "arm-failures=1" in line and "armed=1" in line for line in messages
    ), messages
    # A per-PR failure is NOT a capability failure, so it emits no ::error.
    assert not [line for line in messages if line.startswith("::error")], messages

    # (6) MID-SWEEP CAPABILITY DENIAL — the #3760 error on the first PR must stop the
    # sweep after ONE ::error (never one per PR) and still exit non-zero.
    fake, messages, outcome = exercise(
        fixture(3021),
        fixture(3022),
        fixture(3023),
        arm_errors={number: DENIAL_TEXT for number in (3021, 3022, 3023)},
    )
    assert [call[2] for call in arm_calls(fake)] == ["3021"], fake.calls
    assert outcome.capability == CANNOT_ARM, outcome
    assert outcome.armed == 0, outcome
    assert [number for number, _ in outcome.arm_failures] == [3021], outcome
    assert outcome.exit_code == 1, messages
    emitted = [line for line in messages if line.startswith("::error")]
    assert len(emitted) == 1, emitted
    assert "contents: write" in emitted[0], emitted
    assert sum("arm capability lost" in line for line in messages) == 2, messages

    # (7) EXIT SEMANTICS — an arm failure must never exit 0, and the pre-#3760
    # live-state `errors` contract still reds the run.
    assert SweepOutcome().exit_code == 0
    assert SweepOutcome(arm_failures=[(1, "x")]).exit_code == 1
    assert SweepOutcome(state_failures=[(1, "x")]).exit_code == 1
    assert SweepOutcome(capability=CANNOT_ARM).exit_code == 1
    assert SweepOutcome(capability=INCONCLUSIVE).exit_code == 0

    # (8) The standalone probe entry point must agree with the in-sweep verdict, in both
    # directions, and a cannot-arm probe must exit non-zero exactly once.
    blocked = RearmSweeper(
        "sparq-org/sparq",
        "main",
        gh=FakeGh([], {}, auto_merge_allowed=False),
        log=lambda _line: None,
    )
    assert blocked.probe_arm_capability().status == CANNOT_ARM
    assert probe_arm_capability_exit(blocked) == 1
    healthy = RearmSweeper(
        "sparq-org/sparq", "main", gh=FakeGh([], {}), log=lambda _line: None
    )
    assert healthy.probe_arm_capability().status == CAN_ARM
    assert probe_arm_capability_exit(healthy) == 0

    # A capability failure is NOT a transient: --mode sweep must still exit 1 (it must
    # never be converted into the #3759 ::warning + exit-0 missed cycle).
    class _CannotArmHarness:
        def __enter__(self):
            self._argv = sys.argv
            globals()["run_gh_read"] = lambda argv: capability_payload(False)
            sys.argv = ["rearm-sweeper.py", "--repo", "sparq-org/sparq", "--mode", "sweep"]
            return self

        def __exit__(self, *exc):
            globals()["run_gh_read"] = original_run_gh_read
            sys.argv = self._argv
            return False

    out, err = io.StringIO(), io.StringIO()
    with _CannotArmHarness(), redirect_stdout(out), redirect_stderr(err):
        cannot_arm_code = main()
    assert cannot_arm_code == 1, cannot_arm_code
    assert "::error" in out.getvalue(), out.getvalue()
    assert "::warning" not in out.getvalue(), out.getvalue()

    # ---------------------------------------------------------------------------------
    # [OPUS-5] #3766 — STICKY FAILURE PRECEDENCE:
    #     collected-failure > transient-exhaustion > clean.
    # The false pass this fixes (reproduced by the cross-provider review): PR #3011's arm
    # FAILED and was collected, then PR #3012's live-state read exhausted its bounded
    # retries; that exhaustion unwound run() and main()'s lenient handler reported
    # ::warning + exit 0 — discarding a real arm failure. The collected verdict must be
    # untouchable by anything that happens for a LATER candidate.
    # ---------------------------------------------------------------------------------

    def strip_live(pr: dict) -> dict:
        return {
            key: value
            for key, value in pr.items()
            if key not in ("mergeQueueEntry", "autoMergeRequest")
        }

    def exhausting_read(base: FakeGh, number: int) -> Callable[[list[str]], str]:
        """A gh_read whose live-state query for ONE candidate exhausts its retries."""

        def read(argv: list[str]) -> str:
            if (
                argv[:2] == ["api", "graphql"]
                and not is_capability_query(argv)
                and f"number={number}" in argv
            ):
                raise gh_retry.GhTransientExhausted(
                    "gh api graphql: HTTP 504 (3 attempts)"
                )
            return base(argv)

        return read

    def sequence(*, arm_fails: bool):
        """PR #3011 (optionally arm-failing), then PR #3012 whose live read exhausts."""
        first_pr, second_pr = fixture(3011), fixture(3012)
        base = FakeGh(
            [strip_live(first_pr), strip_live(second_pr)],
            {3011: first_pr, 3012: second_pr},
            arm_errors=(
                {3011: "gh pr merge 3011 failed: Pull request is in unstable status"}
                if arm_fails
                else None
            ),
        )
        lines: list[str] = []
        sweeper = RearmSweeper(
            "sparq-org/sparq",
            "main",
            gh=base,
            gh_read=exhausting_read(base, 3012),
            log=lines.append,
        )
        # run() must RETURN here: an ESCAPING exhaustion is exactly the #3766 false pass.
        return lines, sweeper.run()

    # (9) THE #3766 SEQUENCE — arm failure on A, exhausted transient on a LATER B.
    lines, outcome = sequence(arm_fails=True)
    assert [number for number, _ in outcome.arm_failures] == [3011], outcome
    assert [number for number, _ in outcome.transient_exhaustions] == [3012], outcome
    assert outcome.hard_failed and outcome.exit_code == 1, outcome
    for mode in ("sweep", "event"):
        assert sweep_exit(outcome, mode, lines.append, lines.append) == 1, (mode, lines)
    assert any("PR #3011: ARM-FAILED" in line for line in lines), lines
    assert any("arm-failure summary: PR #3011" in line for line in lines), lines
    assert any(
        "transient-exhausted" in line and "3012" in line for line in lines
    ), lines
    assert any("precedence: collected-failure" in line for line in lines), lines
    # The lenient ::warning must NOT be emitted while a real failure stands.
    assert not [line for line in lines if line.startswith("::warning")], lines

    # (10) CONTROL — the lenient policy is intact where it IS correct: a sweep whose ONLY
    # problem is an exhausted transient still exits 0 with exactly one ::warning, and the
    # candidates around it are still armed.
    lines, outcome = sequence(arm_fails=False)
    assert outcome.armed == 1 and not outcome.hard_failed, outcome
    assert [number for number, _ in outcome.transient_exhaustions] == [3012], outcome
    warned: list[str] = []
    assert sweep_exit(outcome, "sweep", warned.append, warned.append) == 0, warned
    assert len(warned) == 1 and warned[0].startswith("::warning"), warned
    assert "3012" in warned[0], warned
    assert any("PR #3011: ARMED" in line for line in lines), lines

    # (11) CONTROL — event mode is unchanged: no per-PR cron backstop, so the same
    # transient-only outcome fails loudly on the stderr channel.
    loud: list[str] = []
    assert sweep_exit(outcome, "event", loud.append, loud.append) == 1, loud
    assert any("event-mode" in line for line in loud), loud

    # (12) The precedence rule itself, at the seam.
    assert SweepOutcome().transient_detail is None
    assert SweepOutcome(sweep_transient="HTTP 504").transient_detail == "HTTP 504"
    assert (
        SweepOutcome(transient_exhaustions=[(1, "HTTP 504")]).transient_detail
        == "PR #1: HTTP 504"
    )
    assert SweepOutcome(transient_exhaustions=[(1, "HTTP 504")]).exit_code == 0
    dominated = SweepOutcome(
        arm_failures=[(1, "unstable")], transient_exhaustions=[(2, "HTTP 504")]
    )
    assert dominated.hard_failed and dominated.exit_code == 1, dominated
    for mode in ("sweep", "event"):
        assert sweep_exit(dominated, mode, lambda _l: None, lambda _l: None) == 1, mode

    # (13) FAIL CLOSED — an exhausted transient raised by the ARM MUTATION itself leaves the
    # arm's outcome UNKNOWN, so it must be a per-PR failure (exit 1), never a lenient
    # warning-class missed cycle. The one-shot runner cannot raise this today; the handler
    # exists so a future refactor cannot open a second false-pass route.
    only = fixture(3031)
    mutation_base = FakeGh([strip_live(only)], {3031: only})

    def gh_arm_exhausts(argv: list[str]) -> str:
        if argv[:2] == ["pr", "merge"]:
            raise gh_retry.GhTransientExhausted("gh pr merge: HTTP 504 (3 attempts)")
        return mutation_base(argv)

    lines = []
    outcome = RearmSweeper(
        "sparq-org/sparq", "main", gh=gh_arm_exhausts, gh_read=mutation_base,
        log=lines.append,
    ).run()
    assert [number for number, _ in outcome.arm_failures] == [3031], outcome
    assert not outcome.transient_exhaustions, outcome
    assert sweep_exit(outcome, "sweep", lines.append, lines.append) == 1, lines
    assert not [line for line in lines if line.startswith("::warning")], lines
    assert any(
        "transient exhaustion on the arm mutation" in line for line in lines
    ), lines

    # (14) END TO END through main() — the precedence must hold for the real entry point,
    # not only for the seam the assertions above call directly.
    original_run_gh = globals()["run_gh"]

    class _StickyPrecedenceHarness:
        """A collected arm failure on PR #3011 must dominate PR #3012's exhaustion."""

        def __enter__(self):
            first_pr, second_pr = fixture(3011), fixture(3012)
            fake = FakeGh(
                [strip_live(first_pr), strip_live(second_pr)],
                {3011: first_pr, 3012: second_pr},
                arm_errors={
                    3011: "gh pr merge 3011 failed: Pull request is in unstable status"
                },
            )
            self._argv = sys.argv
            globals()["run_gh"] = fake
            globals()["run_gh_read"] = exhausting_read(fake, 3012)
            sys.argv = [
                "rearm-sweeper.py", "--repo", "sparq-org/sparq", "--mode", "sweep",
            ]
            return self

        def __exit__(self, *exc):
            globals()["run_gh"] = original_run_gh
            globals()["run_gh_read"] = original_run_gh_read
            sys.argv = self._argv
            return False

    out, err = io.StringIO(), io.StringIO()
    with _StickyPrecedenceHarness(), redirect_stdout(out), redirect_stderr(err):
        sticky_code = main()
    assert sticky_code == 1, (sticky_code, out.getvalue(), err.getvalue())
    assert "arm-failure summary: PR #3011" in out.getvalue(), out.getvalue()
    assert "::warning" not in out.getvalue(), out.getvalue()

    print("rearm-sweeper self-test: PASS")


def sweep_exit(
    outcome: SweepOutcome,
    mode: str,
    log: Callable[[str], None] = print,
    fail: Callable[[str], None] | None = None,
) -> int:
    """Decide the run's exit status from the FINAL collected state (#3766).

    PRECEDENCE — ``collected-failure > transient-exhaustion > clean``:

    1. A COLLECTED arm/state/capability failure DOMINATES, in either mode. Nothing that
       happens while processing a LATER candidate — an exhausted transient above all — can
       downgrade a verdict an EARLIER candidate already earned. This check is reached from
       the accumulated state, never short-circuited by an exception.
    2. TRANSIENT EXHAUSTION alone (no collected failure anywhere) keeps #3759's intended
       lenient policy: ``sweep`` reports ``::warning`` + exit 0, because a missed cycle is
       harmless and the next cron run re-covers it. ``event`` has no per-PR backstop, so it
       fails loudly instead.
    3. CLEAN is 0.
    """
    if fail is None:

        def fail(message: str) -> None:
            print(message, file=sys.stderr)

    if outcome.hard_failed:
        if outcome.transient_detail:
            log(
                f"[{PROGRAM}] a transient exhaustion also occurred "
                f"({outcome.transient_detail}) but a collected failure outranks it "
                "(precedence: collected-failure > transient-exhaustion > clean): exit 1"
            )
        return outcome.exit_code
    if outcome.transient_detail:
        # [FABLE-5] #3759: only the periodic + idempotent SWEEP may swallow a missed cycle
        # on a transient platform 5xx (the cron backstop covers it). An EVENT-driven
        # invocation has no per-PR backstop, so it fails loudly (finding 5).
        if mode == "event":
            fail(
                f"[{PROGRAM}] fatal: event-mode exhausted transient retries: "
                f"{outcome.transient_detail}"
            )
            return 1
        log(
            f"::warning title={PROGRAM} skipped a cycle on transient GitHub API "
            f"failures::{outcome.transient_detail} — bounded retries exhausted; the next "
            "cron run covers this sweep, so this run reports success."
        )
        return 0
    return 0


def probe_arm_capability_exit(sweeper: RearmSweeper) -> int:
    """Run ONLY the startup probe: 0 = armable (or inconclusive), 1 = provably not.

    Wired as its own workflow step so the job reds at second three with one actionable
    ::error instead of burning a sweep and reporting a per-PR SKIP every ten minutes.
    """
    verdict = sweeper.probe_arm_capability()
    sweeper.log(f"[{PROGRAM}] arm-capability probe: {verdict.status} — {verdict.detail}")
    if verdict.blocks_sweep:
        sweeper.fail_capability(f"startup arm-capability probe failed: {verdict.detail}")
        return 1
    return 0


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
    parser.add_argument(
        "--probe-arm-capability",
        action="store_true",
        help="only verify the token can enable auto-merge here; arm nothing",
    )
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required unless --self-test is used")
    sweeper: RearmSweeper | None = None
    try:
        sweeper = RearmSweeper(
            args.repo, args.default_branch, max_rearms=args.max_rearms,
            gh=run_gh, gh_read=run_gh_read,
        )
        if args.probe_arm_capability:
            return probe_arm_capability_exit(sweeper)
        outcome = sweeper.run()
    except gh_retry.GhTransientExhausted as error:
        # [OPUS-5] #3766 STICKY BACKSTOP. run() already records a per-candidate exhaustion
        # without unwinding; this is defence in depth for any read that is ever added
        # OUTSIDE that guarded loop. The outcome is read back OFF THE SWEEPER, so everything
        # already collected still reaches sweep_exit below — an exception can no longer
        # short-circuit past the accumulated-failure check and report a false success.
        outcome = sweeper.outcome if sweeper is not None else SweepOutcome()
        outcome.sweep_transient = outcome.sweep_transient or str(error)
    except (GhError, ValueError, json.JSONDecodeError) as error:
        print(f"[{PROGRAM}] fatal: {error}", file=sys.stderr)
        return 1
    # The exit is computed HERE, at the end, from the final collected state:
    # collected-failure > transient-exhaustion > clean.
    return sweep_exit(outcome, args.mode)


if __name__ == "__main__":
    sys.exit(main())
