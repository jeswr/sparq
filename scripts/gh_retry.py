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
    a CAS mutation is how double-arms happen). :func:`assert_read_only` is
    FAIL-CLOSED: it ALLOW-LISTS the read shapes this helper accepts and raises
    :class:`GhRetryUsageError` before ever invoking ``gh`` for anything it cannot
    AFFIRMATIVELY prove is a read. A call is retried ONLY when it positively matches a
    known-safe read shape — an allow-listed read subcommand, or ``gh api`` whose
    method is GET/HEAD with an INLINE, self-evidently-read body. Anything whose intent
    cannot be read off the argv is refused (one-shot), never retried:

      - ``gh api`` auto-switches to POST when field params are given, so a REST call
        carrying fields is refused unless the method is explicitly GET/HEAD;
      - ``gh api graphql`` is accepted only when the query text is present INLINE in
        argv (``-f query=…`` / ``--field query=…``) AND contains no ``mutation``
        (or ``subscription``) operation. A file-backed or stdin body
        (``--input file.json`` / ``-F query=@file`` / ``-f query=@file`` / no inline
        query at all) is OPAQUE to this guard — it cannot be proven a read, so it is
        refused rather than retried. This closes the file-backed-mutation bypass:
        the allow-list never says "retry" on a body it did not itself inspect.

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

# A GraphQL document that starts (after leading whitespace/comments) with a mutation
# or subscription operation is never a read. We also refuse the bare keyword anywhere
# to stay conservative — an inline query naming these operations cannot be proven safe.
_MUTATION_RE = re.compile(r"\b(?:mutation|subscription)\b", re.IGNORECASE)

# Flags whose value carries the GraphQL query text inline in argv.
_QUERY_FIELD_FLAGS = ("-f", "--field", "-F", "--raw-field")


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


def _flag_values(argv: Sequence[str], flags: Sequence[str]) -> list[str]:
    """Collect the values passed to any of ``flags`` in argv (``-f v`` and ``-f=v``)."""
    values: list[str] = []
    index = 0
    argc = len(argv)
    while index < argc:
        arg = str(argv[index])
        if arg in flags:
            if index + 1 < argc:
                values.append(str(argv[index + 1]))
                index += 2
                continue
            values.append("")  # dangling flag; treated as opaque below
        else:
            for flag in flags:
                if arg.startswith(flag + "="):
                    values.append(arg.split("=", 1)[1])
                    break
        index += 1
    return values


def _graphql_uses_opaque_body(rest: Sequence[str]) -> bool:
    """True if the GraphQL body is not fully inline in argv (file-backed / stdin).

    ``gh api graphql`` can source its query from a file or stdin — ``--input file``,
    ``-F query=@file``, ``-f query=@file`` — none of which places the query text in
    argv where the mutation guard can inspect it. Such a call is OPAQUE: we cannot
    prove it is a read, so the caller must refuse (one-shot), never retry.
    """
    if any(str(arg) in ("--input", "--input=-") or str(arg).startswith("--input=")
           for arg in rest):
        return True
    for value in _flag_values(rest, _QUERY_FIELD_FLAGS):
        # `key=@file` (and the bare `@file`/`@-` form) reads the value from a file/stdin.
        payload = value.split("=", 1)[1] if "=" in value else value
        if payload.startswith("@"):
            return True
    return False


def _inline_graphql_query(rest: Sequence[str]) -> str | None:
    """Return the inline ``query=…`` text if present in argv, else ``None``.

    Only a ``query`` field whose value is present literally in argv counts; a
    ``query=@file`` reference is not inline and yields ``None`` (handled as opaque).
    """
    for value in _flag_values(rest, _QUERY_FIELD_FLAGS):
        if value.startswith("query=") and not value.startswith("query=@"):
            return value.split("=", 1)[1]
    return None


def assert_read_only(argv: Sequence[str]) -> None:
    """Fail-closed guard: raise :class:`GhRetryUsageError` unless argv is a read."""
    if not argv:
        raise GhRetryUsageError("empty gh argv")
    head = str(argv[0])
    if head == "api":
        rest = [str(arg) for arg in argv[1:]]
        method = _explicit_method(rest)
        if "graphql" in rest:
            # FAIL-CLOSED: a GraphQL call is retriable ONLY if we can read the whole
            # query text off argv and prove it carries no mutation/subscription. A
            # file-backed or stdin body (`--input`, `-F query=@file`, `-f query=@file`,
            # or no inline query at all) is opaque — refuse it rather than retry.
            if _graphql_uses_opaque_body(rest):
                raise GhRetryUsageError(
                    "refusing to wrap gh api graphql with a file-backed/stdin body "
                    "(--input / query=@file / stdin) — its query text is not inline in "
                    "argv, so it cannot be proven a read; mutations stay one-shot"
                )
            inline_query = _inline_graphql_query(rest)
            if inline_query is None:
                raise GhRetryUsageError(
                    "refusing to wrap gh api graphql with no inline `query=` text — "
                    "cannot prove it is a read; mutations stay one-shot"
                )
            if _MUTATION_RE.search(inline_query):
                raise GhRetryUsageError(
                    "refusing to wrap a GraphQL mutation/subscription — "
                    "arms/mutations stay one-shot"
                )
            return
        if method not in (None, "GET", "HEAD"):
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
        # [FABLE-5] #3759 finding 6: FAIL-CLOSED on file-backed / stdin GraphQL bodies.
        # The query text is not inline in argv, so it CANNOT be proven a read — the
        # substring-only guard used to accept these (the file could hold a mutation).
        ["api", "graphql", "--input", "payload.json"],
        ["api", "graphql", "--input=payload.json"],
        ["api", "graphql", "--input", "-"],  # stdin body
        ["api", "graphql", "-F", "query=@file.graphql"],
        ["api", "graphql", "-f", "query=@file.graphql"],
        ["api", "graphql", "--field", "query=@q.gql"],
        # An inline query naming a mutation/subscription operation is still refused.
        ["api", "graphql", "-f", "query=mutation Arm { enableAutoMerge { id } }"],
        ["api", "graphql", "-F", "query=subscription S { x }"],
        # graphql with no inline `query=` at all is opaque — refuse (cannot prove read).
        ["api", "graphql", "-f", "owner=sparq-org"],
        ["api", "graphql"],
    ]
    for argv in refused:
        boom = _FakeRun([(0, "", "")])
        try:
            run_gh_read(argv, run=boom, sleep=lambda _s: None)
        except GhRetryUsageError:
            assert boom.calls == [], (argv, boom.calls)
        else:
            raise AssertionError(f"must refuse to wrap mutation shape: {argv}")

    # Allowed read shapes pass the guard (INLINE GraphQL queries, explicit-GET REST,
    # views). The rearm-sweeper's live-state query is an inline `-f query=query(...)`.
    for argv in (
        ["api", "graphql", "-f", "query=query($n:Int!){repository{pullRequest}}"],
        ["api", "graphql", "--field", "query=query{viewer{login}}", "-F", "n=1"],
        ["api", "repos/o/r/actions/runs?event=merge_group"],
        ["api", "-X", "GET", "search/issues", "-f", "q=is:open"],
        ["api", "-X", "HEAD", "repos/o/r"],
        ["pr", "view", "1", "--json", "state"],
        ["run", "list", "--limit", "5"],
    ):
        assert_read_only(argv)

    # A file-backed body next to an inline non-query field is still opaque (the query
    # itself is file-sourced) and must be refused — not smuggled through by the field.
    try:
        assert_read_only(["api", "graphql", "-F", "query=@q.gql", "-f", "n=1"])
    except GhRetryUsageError:
        pass
    else:
        raise AssertionError("file-backed graphql query must be refused")

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
