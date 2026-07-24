#!/usr/bin/env python3
"""Bounded, transient-only retry for idempotent ``gh`` READ commands.

[FABLE-5] sparq-org/sparq#3759 + registry jeswr/agent-account-registry#563 item 4.
🤖 SPARQ agent.

A minimal, stdlib-only vendored mirror of the registry's tenacity retry rule for the
sweep scripts (auto-arm.py, rearm-sweeper.py) whose periodic runs were redding main's
gate on transient GitHub 5xx blips (HTTP 502/504 at 15:22/22:42/14:29 — #3759):

  * TRANSIENT-ONLY — only platform 5xx / network-transport failures retry (the
    bounded marker list below). Any other error (401/403/404/422, GraphQL schema
    errors, auth loss) raises :class:`GhFatalError` IMMEDIATELY — non-transient
    failures must keep failing loudly.
  * BOUNDED — 3 attempts, 10s/30s backoff (the #3759-specified schedule). Exhaustion
    raises :class:`GhTransientExhausted`, a DISTINCT type so a periodic sweep's
    entrypoint can convert exactly that case into ``::warning`` + exit 0 (a missed
    cycle is harmless; the cron covers it) while everything else stays a hard red.
  * NEVER WRAPS MUTATIONS — arms/labels/merges must stay one-shot (a blind retry of
    a CAS mutation is how double-arms happen). :func:`assert_read_only` fail-closed
    ALLOW-LISTS the read shapes this helper accepts and raises
    :class:`GhRetryUsageError` before ever invoking ``gh`` otherwise. Note ``gh api``
    auto-switches to POST when field params are given, so REST calls carrying fields
    are rejected unless the method is explicitly GET; ``gh api graphql`` is accepted
    only when no query text contains a ``mutation`` operation.

Usage::

    import gh_retry
    stdout = gh_retry.run_gh_read(["pr", "list", "--repo", repo, "--json", "number"])

Self-test (hermetic — injected runner/sleeper, no gh, no network)::

    python3 scripts/gh_retry.py --self-test
"""

from __future__ import annotations

import re
import subprocess
import sys
import time
from typing import Callable, Sequence

DEFAULT_ATTEMPTS = 3
DEFAULT_DELAYS = (10.0, 30.0)

# Bounded marker list, matched case-insensitively against gh's stderr/stdout.
# Deliberately narrow: a marker earns its place by naming a GitHub platform 5xx or a
# network-transport failure observed from the runners — never an auth/validation class.
_TRANSIENT_MARKERS = (
    "http 502",
    "http 503",
    "http 504",
    "502 bad gateway",
    "503 service unavailable",
    "504 gateway timeout",
    "connection reset",
    "connection refused",
    "i/o timeout",
    "tls handshake timeout",
    "context deadline exceeded",
    "temporary failure in name resolution",
    "unexpected eof",
    "request timed out",
)

# gh subcommand pairs this helper accepts as reads (fail-closed: everything not
# listed — pr merge/ready/edit/comment, issue edit, api POST… — is refused).
_READ_SUBCOMMANDS = frozenset(
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

_MUTATION_RE = re.compile(r"\bmutation\b", re.IGNORECASE)


class GhRetryUsageError(ValueError):
    """The caller tried to wrap a non-read ``gh`` invocation. Never retried."""


class GhFatalError(RuntimeError):
    """A non-transient ``gh`` failure — the caller must fail loudly."""


class GhTransientExhausted(RuntimeError):
    """Transient failures persisted through every bounded attempt."""


def is_transient(detail: str) -> bool:
    """True iff the failure text matches the bounded transient-marker list."""
    lowered = detail.lower()
    return any(marker in lowered for marker in _TRANSIENT_MARKERS)


def _explicit_method(argv: Sequence[str]) -> str | None:
    for index, arg in enumerate(argv):
        if arg in ("-X", "--method"):
            if index + 1 < len(argv):
                return str(argv[index + 1]).upper()
            return ""
        if arg.startswith("--method="):
            return arg.split("=", 1)[1].upper()
    return None


def _has_field_params(argv: Sequence[str]) -> bool:
    field_flags = ("-f", "-F", "--field", "--raw-field", "--input")
    return any(
        arg in field_flags or any(arg.startswith(flag + "=") for flag in field_flags[2:])
        for arg in argv
    )


def assert_read_only(argv: Sequence[str]) -> None:
    """Fail-closed guard: raise :class:`GhRetryUsageError` unless argv is a read."""
    if not argv:
        raise GhRetryUsageError("empty gh argv")
    head = str(argv[0])
    if head == "api":
        rest = [str(arg) for arg in argv[1:]]
        method = _explicit_method(rest)
        if "graphql" in rest:
            if any(_MUTATION_RE.search(arg) for arg in rest):
                raise GhRetryUsageError(
                    "refusing to wrap a GraphQL mutation — arms/mutations stay one-shot"
                )
            return
        if method not in (None, "GET"):
            raise GhRetryUsageError(
                f"refusing to wrap gh api with method {method or '<missing>'} — "
                "mutations stay one-shot"
            )
        if method is None and _has_field_params(rest):
            raise GhRetryUsageError(
                "refusing to wrap gh api with field params and no explicit GET "
                "(gh auto-switches to POST) — mutations stay one-shot"
            )
        return
    if tuple(str(arg) for arg in argv[:2]) in _READ_SUBCOMMANDS:
        return
    raise GhRetryUsageError(
        f"gh {' '.join(str(a) for a in argv[:2])} is not an allow-listed read — "
        "mutations/arm calls must stay one-shot (never wrapped in retries)"
    )


def run_gh_read(
    argv: Sequence[str],
    *,
    attempts: int = DEFAULT_ATTEMPTS,
    delays: Sequence[float] = DEFAULT_DELAYS,
    run: Callable[..., subprocess.CompletedProcess] = subprocess.run,
    sleep: Callable[[float], None] = time.sleep,
    log: Callable[[str], None] | None = None,
) -> str:
    """Run an idempotent ``gh`` READ with bounded, transient-only retries.

    Returns stdout on success. Raises :class:`GhRetryUsageError` (not a read),
    :class:`GhFatalError` (non-transient failure, first occurrence, never retried),
    or :class:`GhTransientExhausted` (transient failure on every bounded attempt).
    """
    assert_read_only(argv)
    if attempts < 1:
        raise GhRetryUsageError("attempts must be >= 1")
    emit = log or (lambda message: print(message, file=sys.stderr))
    command = ["gh", *[str(arg) for arg in argv]]
    label = " ".join(command[:4])
    for attempt in range(1, attempts + 1):
        result = run(command, capture_output=True, text=True, check=False)
        if result.returncode == 0:
            return result.stdout
        detail = (
            (result.stderr or "").strip()
            or (result.stdout or "").strip()
            or "unknown gh failure"
        )
        if not is_transient(detail):
            raise GhFatalError(f"{label} failed (non-transient): {detail}")
        if attempt == attempts:
            raise GhTransientExhausted(
                f"{label} failed transiently on every attempt "
                f"({attempts} attempts): {detail}"
            )
        delay = float(delays[min(attempt - 1, len(delays) - 1)]) if delays else 0.0
        emit(
            f"[gh-retry] transient failure (attempt {attempt}/{attempts}), "
            f"retrying in {delay:g}s: {label}: {detail}"
        )
        sleep(delay)
    raise AssertionError("unreachable")  # pragma: no cover


# --------------------------------------------------------------------------- tests
class _FakeRun:
    """Scripted subprocess.run stand-in: pops one (rc, stdout, stderr) per call."""

    def __init__(self, outcomes: list[tuple[int, str, str]]) -> None:
        self.outcomes = list(outcomes)
        self.calls: list[list[str]] = []

    def __call__(self, command, **_kwargs) -> subprocess.CompletedProcess:
        self.calls.append(list(command))
        rc, out, err = self.outcomes.pop(0)
        return subprocess.CompletedProcess(command, rc, stdout=out, stderr=err)


def self_test() -> None:
    read = ["pr", "list", "--repo", "sparq-org/sparq", "--json", "number"]

    # Transient then success: exactly one retry, first-delay backoff, stdout returned.
    fake = _FakeRun([(1, "", "HTTP 504 Gateway Timeout"), (0, "[]", "")])
    sleeps: list[float] = []
    out = run_gh_read(read, run=fake, sleep=sleeps.append, log=lambda _m: None)
    assert out == "[]", out
    assert len(fake.calls) == 2, fake.calls
    assert fake.calls[0][0] == "gh", fake.calls
    assert sleeps == [10.0], sleeps

    # Fatal (non-transient) fails LOUDLY on the first attempt — no retry, no sleep.
    fake = _FakeRun([(1, "", "HTTP 404: Not Found (repos/x)")])
    sleeps = []
    try:
        run_gh_read(read, run=fake, sleep=sleeps.append, log=lambda _m: None)
    except GhFatalError as error:
        assert "non-transient" in str(error), error
    else:
        raise AssertionError("HTTP 404 must raise GhFatalError")
    assert len(fake.calls) == 1, fake.calls
    assert sleeps == [], sleeps

    # Bound: persistent transients exhaust after exactly `attempts` invocations with
    # the #3759 backoff schedule, raising the DISTINCT exhaustion type.
    fake = _FakeRun([(1, "", "connection reset by peer")] * 3)
    sleeps = []
    try:
        run_gh_read(read, run=fake, sleep=sleeps.append, log=lambda _m: None)
    except GhTransientExhausted as error:
        assert "3 attempts" in str(error), error
    else:
        raise AssertionError("persistent transient must raise GhTransientExhausted")
    assert len(fake.calls) == 3, fake.calls
    assert sleeps == [10.0, 30.0], sleeps
    assert not isinstance(GhTransientExhausted("x"), GhFatalError)

    # Case-insensitive transient classification; auth/validation classes are fatal.
    assert is_transient("HTTP 502 Bad Gateway")
    assert is_transient("Post https://api.github.com/graphql: i/o TIMEOUT")
    assert not is_transient("HTTP 403: Resource not accessible by integration")
    assert not is_transient("HTTP 422: Validation Failed")
    assert not is_transient("GraphQL: Could not resolve to a node")

    # Mutation guard: every arm/label/merge shape is refused BEFORE gh is invoked.
    refused = [
        ["pr", "merge", "1", "--auto"],
        ["pr", "ready", "1"],
        ["pr", "edit", "1", "--add-label", "review:changes"],
        ["pr", "comment", "1", "--body", "x"],
        ["api", "graphql", "-f", "query=mutation($id:ID!){enablePullRequestAutoMerge}"],
        ["api", "repos/o/r/issues/1/labels", "-f", "labels[]=x"],
        ["api", "-X", "POST", "repos/o/r/issues"],
        ["api", "--method=DELETE", "repos/o/r/labels/x"],
        [],
    ]
    for argv in refused:
        boom = _FakeRun([(0, "", "")])
        try:
            run_gh_read(argv, run=boom, sleep=lambda _s: None)
        except GhRetryUsageError:
            assert boom.calls == [], (argv, boom.calls)
        else:
            raise AssertionError(f"must refuse to wrap mutation shape: {argv}")

    # Allowed read shapes pass the guard (GraphQL queries, explicit-GET REST, views).
    for argv in (
        ["api", "graphql", "-f", "query=query($n:Int!){repository{pullRequest}}"],
        ["api", "repos/o/r/actions/runs?event=merge_group"],
        ["api", "-X", "GET", "search/issues", "-f", "q=is:open"],
        ["pr", "view", "1", "--json", "state"],
        ["run", "list", "--limit", "5"],
    ):
        assert_read_only(argv)

    print("gh-retry self-test: PASS")


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        self_test()
        return 0
    print(__doc__)
    print("This is a library — import it; the only CLI entrypoint is --self-test.")
    return 2


if __name__ == "__main__":
    sys.exit(main())
