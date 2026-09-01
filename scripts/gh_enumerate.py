#!/usr/bin/env python3
"""Fail-LOUD ceilings for ``gh <list> --limit N`` enumerations.

[OPUS-5] sparq-org/sparq#4985. 🤖 SPARQ agent.

``gh pr list`` / ``gh issue list --limit N`` TRUNCATE SILENTLY: when the population
exceeds ``N`` the CLI returns the newest ``N`` rows, prints no warning, and exits 0.
Any caller that ENUMERATES a population in order to make a decision therefore degrades
INVISIBLY the moment that population crosses its cap. The two shapes in this repo:

  * a WORK-QUEUE sweep (batch-merge, pr-backlog, pr-area-labels, feature-OFF
    autodeclare, promote-on-approval) stops seeing the tail of the queue, so throughput
    drops with no error to attribute it to;
  * a DEDUPE / marker search (the alarms, pr-backlog's marker set) stops seeing the very
    marker it exists to find, so the non-spam invariant FAILS OPEN and the caller mints a
    duplicate issue on every run.

This module does not hide the cap; it makes crossing it VISIBLE. Callers keep their own
``gh`` runner — every one of them is a stub seam that caller's self-test depends on — and
compose two pieces::

    argv = ["pr", "list", "--repo", repo, "--state", "open",
            "--json", fields] + gh_enumerate.limit_args(gh_enumerate.PR_CEILING)
    rows = gh_enumerate.guard(json.loads(run(argv)), "open PRs",
                              ceiling=gh_enumerate.PR_CEILING, exc=SystemExit)

:func:`limit_args` supplies a CEILING far above the live population rather than a working
cap. The ceiling is not a budget — it exists only so a runaway snapshot fails CLOSED
instead of half-reporting.

Raising the number is expected to be cost-neutral, because ``gh`` pages the underlying
GraphQL connection and stops at the last page rather than at ``--limit``: the cost of a
list call tracks the POPULATION, not the cap. That was NOT measured here (this change was
developed without network or ``gh`` access), so treat it as the reason the ceiling is safe
to raise, not as a benchmarked result. If it is ever observed to be wrong, the fix is to
lower the ceilings toward the live populations — never to drop :func:`guard`, which is
what actually makes truncation visible.

:func:`guard` trips at ``len(rows) >= ceiling``, not ``>``, because a truncated ``gh``
fetch returns EXACTLY ``ceiling`` rows: at the boundary "the population is 2000" and "the
population is larger and was cut to 2000" are indistinguishable from the response alone,
so the conservative reading is the only sound one. A population landing exactly on the
ceiling therefore raises; that is a deliberate false positive, and the fix is to raise the
ceiling deliberately rather than to widen the comparison.

This is the GraphQL-side counterpart to the REST ``gh api --paginate`` + explicit-ceiling
fetches in ``ready-issues.py::_fetch``, ``retriage.py::_fetch_label`` and
``triage-area.py::open_issues``. Those callers can use REST because they need only stock
issue fields; the PR callers here cannot, because they select ``--json`` fields that are
GraphQL projections rather than REST list columns (``autoMergeRequest`` in
``batch-merge.py``, ``changedFiles`` in ``pr-area-labels.py``, ``headRefOid`` in
``feature_off_autodeclare.py``). Hence a ceiling + assertion rather than ``--paginate``.
"""

from __future__ import annotations

import sys

# Live population on sparq-org/sparq when this module was written (2026-09-01): 93 open
# PRs, 1707 open issues. The ceilings are sized as fail-closed backstops, not forecasts.
#: Open-PR enumerations (~21x the live open-PR count).
PR_CEILING = 2000
#: Open-ISSUE enumerations. Matches the ceiling already used by ready-issues.py,
#: retriage.py and triage-area.py, so every open-issue sweep in the repo fails closed at
#: one number (~6x the live open-issue count).
ISSUE_CEILING = 10000


class EnumerationTruncated(RuntimeError):
    """A ``gh`` enumeration came back sitting on its ceiling, so it may be truncated."""


def limit_args(ceiling: int) -> list[str]:
    """The ``--limit`` argv pair for ``ceiling``.

    A function rather than an f-string at each call site so that every enumeration in the
    repo spells its ceiling the same way and `grep limit_args` enumerates them all.
    """
    if isinstance(ceiling, bool) or not isinstance(ceiling, int) or ceiling < 1:
        raise ValueError(f"ceiling must be a positive int, got {ceiling!r}")
    return ["--limit", str(ceiling)]


def guard(rows, what: str, *, ceiling: int, exc=EnumerationTruncated):
    """Return ``rows``, or raise ``exc`` if the fetch may have been silently truncated.

    ``what`` names the population for the error message ("open PRs"). ``exc`` is the
    exception the CALLER wants raised, so each script keeps its own failure vocabulary —
    ``SystemExit`` for a sweep whose entrypoint reports it, ``AlarmError`` for the alarms
    — instead of catching and re-raising at every site.
    """
    if not isinstance(rows, list):
        raise exc(
            f"gh enumeration of {what} returned {type(rows).__name__}, not a list — "
            "refusing to treat a malformed response as an empty or complete population."
        )
    if len(rows) >= ceiling:
        raise exc(
            f"gh enumeration of {what} returned {len(rows)} rows, at its ceiling of "
            f"{ceiling}. `gh ... list --limit` truncates SILENTLY, so the tail of this "
            "population may be missing and any decision taken from it is unsound. Raise "
            "the ceiling in scripts/gh_enumerate.py deliberately."
        )
    return rows


# --------------------------------------------------------------------------------------
# Self-test
# --------------------------------------------------------------------------------------
def _self_test() -> int:
    failures = []

    def chk(name, got, want):
        ok = got == want
        print(f"{'ok  ' if ok else 'FAIL'} {name}")
        if not ok:
            failures.append(f"{name}: got {got!r}, want {want!r}")

    def raises(fn, exc_type):
        try:
            fn()
        except exc_type:
            return True
        except Exception:  # noqa: BLE001 - wrong type is a failure, not an error
            return False
        return False

    chk("limit_args emits the ceiling verbatim", limit_args(2000), ["--limit", "2000"])
    chk("limit_args rejects a zero ceiling", raises(lambda: limit_args(0), ValueError), True)
    chk("limit_args rejects a negative ceiling",
        raises(lambda: limit_args(-1), ValueError), True)
    chk("limit_args rejects a bool ceiling (True is not a row count)",
        raises(lambda: limit_args(True), ValueError), True)

    # Under the ceiling the guard is transparent: same rows, same order, same identity.
    under = [{"number": n} for n in range(9)]
    chk("guard passes an under-ceiling fetch through unchanged",
        guard(under, "open PRs", ceiling=10), under)
    chk("guard passes an empty fetch through", guard([], "open PRs", ceiling=10), [])

    # THE BOUNDARY. A truncated `gh` fetch returns exactly `ceiling` rows, so `==` must
    # raise. If this flips to `>`, silent truncation is readmitted at exactly the size it
    # actually occurs at — the whole point of the module.
    exact = [{"number": n} for n in range(10)]
    chk("guard raises when the fetch sits EXACTLY on the ceiling (the truncation case)",
        raises(lambda: guard(exact, "open PRs", ceiling=10), EnumerationTruncated), True)
    chk("guard raises above the ceiling",
        raises(lambda: guard(exact + [{}], "open PRs", ceiling=10), EnumerationTruncated),
        True)

    # The caller's own exception type, so scripts keep their failure vocabulary.
    class _CallerError(Exception):
        pass

    chk("guard raises the CALLER's exception type",
        raises(lambda: guard(exact, "open PRs", ceiling=10, exc=_CallerError), _CallerError),
        True)
    chk("guard raises the caller's type for SystemExit too",
        raises(lambda: guard(exact, "open PRs", ceiling=10, exc=SystemExit), SystemExit),
        True)

    # A malformed response must not read as "population is fine".
    chk("guard rejects a non-list response",
        raises(lambda: guard({"message": "Not Found"}, "open PRs", ceiling=10),
               EnumerationTruncated), True)
    chk("guard rejects None rather than treating it as empty",
        raises(lambda: guard(None, "open PRs", ceiling=10), EnumerationTruncated), True)

    # The message has to be actionable: it must carry the count, the ceiling and the name.
    try:
        guard(exact, "open selection-alarm issues", ceiling=10)
        msg = ""
    except EnumerationTruncated as exc:
        msg = str(exc)
    chk("truncation message names the population", "open selection-alarm issues" in msg, True)
    chk("truncation message carries the observed count", "10 rows" in msg, True)
    chk("truncation message carries the ceiling", "ceiling of 10" in msg, True)

    # The shipped ceilings must clear the live populations they were sized against, or the
    # sweep trades silent truncation for a standing red.
    chk("PR_CEILING clears the live open-PR count with headroom", PR_CEILING >= 1000, True)
    chk("ISSUE_CEILING clears the live open-issue count with headroom",
        ISSUE_CEILING >= 5000, True)

    print()
    if failures:
        print(f"gh_enumerate self-test: {len(failures)} FAILURE(S)")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("gh_enumerate self-test: all checks passed")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        raise SystemExit(_self_test())
    print(__doc__)
    print("run with --self-test to exercise the ceiling guard")
