#!/usr/bin/env python3
"""Single source of the review lane's VISIBILITY predicate, and the DISPOSITION of every
PR that predicate excludes.

[OPUS-5] 🤖 SPARQ agent. Issue #4677 (measured on live sparq, 2026-07-27).

WHY THIS MODULE EXISTS
======================
sparq has no in-repo verdict PRODUCER. Every review a sparq PR receives is dispatched by
the REGISTRY (`jeswr/agent-account-registry`), whose `enumerate_review_items` admits a PR
only if its head ref matches ``^sparq-agent/issue-([1-9][0-9]*)-``, its head repo is the
target repo, and its author is the worker App bot.

`scripts/verdict-bridge.py` — which lives HERE — labelled every green, mergeable,
verdict-less PR ``review:unreviewed`` with **no reachability condition at all**. So a
labeller in this repo enrolled PRs into a lane the reviewer in the OTHER repo structurally
cannot see. MEASURED: all four PRs carrying ``review:unreviewed`` failed the registry's
head-ref gate, including #4460 — the FIRST crates.io release PR — which is authored by the
correct App bot and is still invisible purely because release-plz names its branch
``release-plz-<timestamp>``. The string ``review:unreviewed`` appears nowhere in the
registry codebase, so nothing was ever going to review them.

A PR that is labelled but never reviewed is WORSE than one never labelled: the label makes
it look enrolled, and every per-run success rate the lane reports is computed over a
population that excludes it entirely — the lane can report fully healthy while this class
grows without bound.

THE GENERALISABLE RULE, which this module is the mechanism for: when a pipeline has a
component that WRITES state and a component that does the WORK, their visibility
predicates must be THE SAME OBJECT, not two copies that agree today. This module is that
object; `verdict-bridge.py` (the labeller) and `review_lane_alarm.py` (the detector) both
import it, and neither may re-derive it.

WHAT IT DOES NOT DO
===================
It does not make an unreachable PR reviewable. Widening the registry's allowlist is not
available: reviewer selection there INVERTS ``impl_provider`` to guarantee cross-provider
review, and the release-plz / dependabot classes have no implementing model — admitting one
would require FABRICATING a provider, which makes the cross-provider assertion vacuous
rather than merely weaker (registry#916). So the excluded classes need a DISPOSITION, and
this module names it rather than leaving it implicit.

DISPOSITIONS
------------
``lane``           the registry review lane can enumerate this PR; a review is coming.
                   Label: ``review:unreviewed`` — "enrolled, waiting".
``hand-dispatch``  no automated producer can enumerate it, but a head-bound
                   ``VERDICT: pass`` comment from a hand-dispatched reviewer WOULD promote
                   and arm it normally. Label: ``review:unreachable``.
``maintainer``     no automated producer can enumerate it AND standing repository policy
                   already says its merge is a maintainer's decision:
                     * the release-plz Release PR — never armed by ANY automated path
                       (#1135, `scripts/release_pr_guard.py`, consulted here rather than
                       re-implemented), because a crates.io version cannot be unpublished;
                     * dependabot — `scripts/pr-backlog.py` closes a RED bump and reroutes
                       it to an issue, and leaves a GREEN one untouched for the maintainer;
                       `scripts/batch-merge.py` never touches a ``dependabot/*`` branch.
                   Label: ``review:maintainer-owned``.

The disposition labels are INFORMATIONAL in exactly the sense ``review:unreviewed`` already
was: no arming predicate consumes them (pinned by
`scripts/tests/test_verdict_bridge.py::TestCrossPolicyConsistency`), so labelling can
neither block nor cause a merge. What they change is that the label now tells the truth
about who — if anyone — is coming.

stdlib-only apart from the sibling `release_pr_guard`, because every importer runs under a
sparse checkout with no dependencies. The import is HARD (no fail-soft stub): a missing
guard would silently demote the Release PR from ``maintainer`` to ``hand-dispatch``, i.e.
advertise that a hand-dispatched verdict could arm it. Failing the run loudly is the only
acceptable direction.

    python3 scripts/review_lane_visibility.py --self-test   # hermetic, no network
"""

from __future__ import annotations

import argparse
import re
import sys

import release_pr_guard

# The registry review lane's own admission regex, replicated VERBATIM from the registry's
# `scripts/dispatch-claim.py` (master @ 2026-07-26). Kept byte-identical on purpose: every
# claim made from this module is "the registry lane can / cannot see this PR", so the test
# must be the registry's test and not a paraphrase of it. If the registry ever loosens the
# gate, this constant moves in the same breath — pinned by
# `scripts/tests/test_review_lane_alarm.py::test_the_regex_is_byte_identical_to_the_registry_gate`.
REGISTRY_HEAD_REF_RE = re.compile(r"^sparq-agent/issue-([1-9][0-9]*)-")

# Dispositions. `lane` is the ONLY one that means "a review producer exists for this PR".
LANE = "lane"
HAND_DISPATCH = "hand-dispatch"
MAINTAINER = "maintainer"

# The label each disposition earns while a PR is green, mergeable and carries no verdict.
UNREVIEWED_LABEL = "review:unreviewed"
UNREACHABLE_LABEL = "review:unreachable"
MAINTAINER_LABEL = "review:maintainer-owned"
FLAG_LABEL_BY_DISPOSITION = {
    LANE: UNREVIEWED_LABEL,
    HAND_DISPATCH: UNREACHABLE_LABEL,
    MAINTAINER: MAINTAINER_LABEL,
}
FLAG_LABELS = frozenset(FLAG_LABEL_BY_DISPOSITION.values())

# dependabot opens under `dependabot[bot]` (REST) / `app/dependabot` (gh) / `dependabot`
# (GraphQL) and always branches `dependabot/<ecosystem>/…`. Either signal alone is enough:
# the branch prefix is reserved by dependabot itself, and the login cannot be spoofed by a
# branch rename. Same normalisation as `scripts/pr-backlog.py::is_dependabot`.
DEPENDABOT_LOGINS = frozenset({"dependabot", "dependabot-preview"})
DEPENDABOT_BRANCH_PREFIX = "dependabot/"

# (colour, one-line description) per label. The reify step in
# `.github/workflows/verdict-bridge.yml` renders these with `--labels` instead of
# restating them, so the label a PR carries and the meaning a human reads off it cannot
# drift from the policy that applied it.
LABEL_SPECS = {
    UNREVIEWED_LABEL: (
        "ededed",
        "Green, no head-bound VERDICT, and the review lane CAN see it (informational)",
    ),
    UNREACHABLE_LABEL: (
        "d4c5f9",
        "Green, but no review producer can see it — needs a hand-dispatched review",
    ),
    MAINTAINER_LABEL: (
        "fbca04",
        "Unreachable by the review lane; a maintainer merges it (release-plz #1135 / dependabot)",
    ),
}
# GitHub rejects a label description longer than this (HTTP 422), which would fail the
# reify step and, with it, every labelling run. Pinned in the self-test because the
# descriptions are prose and prose grows.
MAX_LABEL_DESCRIPTION = 100


def normalize_login(login: object) -> str:
    """Normalise a GitHub actor login. Same rule as `release_pr_guard.normalize_login`,
    delegated rather than restated so the two can never disagree."""
    return release_pr_guard.normalize_login(login)


def login_is_app(login: object) -> bool:
    """True iff a REST-shaped login names an App (``x[bot]``).

    SHAPE ADAPTER, not the predicate. GitHub reports one App under three spellings —
    ``x[bot]`` (REST), ``app/x`` (gh) and a bare ``x`` (GraphQL) — so "is this an App?"
    cannot be answered from a normalised login at all, and each caller has to derive it
    from whatever ITS response shape carries (GraphQL callers read
    ``author { __typename }``). Only the derivation is per-caller; the conjunction below
    is shared.
    """
    return str(login or "").strip().endswith("[bot]")


def lane_reachable(
    *, head_ref: object, head_repo: object, author_is_app: bool, repo: str
) -> bool:
    """True iff the registry review lane's `enumerate_review_items` can EVER enumerate
    this PR. False => no review producer in existence will ever see it.

    Only the three AUTHOR-SIDE structural gates are replicated. The registry additionally
    requires a provenance record, but that is a live registry read no sparq workflow has a
    token for, and it can only ever NARROW admission — so answering True here is the
    CONSERVATIVE direction for a detector (it under-reports the blind spot) and the
    conservative direction for a labeller too (it never invents an enrolment that the
    registry would have refused for a reason we cannot see; it can only inherit one).
    """
    if not REGISTRY_HEAD_REF_RE.match(str(head_ref or "")):
        return False
    if str(head_repo or "") != repo:
        return False
    return bool(author_is_app)


def is_dependabot(*, head_ref: object = None, author_login: object = None) -> bool:
    """True iff this PR is a dependabot bump (by author OR by reserved branch prefix)."""
    if normalize_login(author_login) in DEPENDABOT_LOGINS:
        return True
    return str(head_ref or "").strip().lower().startswith(DEPENDABOT_BRANCH_PREFIX)


def maintainer_owned_reason(
    *, head_ref: object, author_login: object = None, title: object = None
) -> str | None:
    """A non-empty reason iff standing policy already makes this PR a maintainer's merge.

    The release axis DELEGATES to `release_pr_guard.arm_block_reason`, the single
    predicate every automated arming path already consults — a second copy keyed on
    "looks like release-plz" is exactly the drift this module exists to prevent. It is
    fail-CLOSED (an unreadable branch, or a bot PR with an unknown title, reads as the
    Release PR), which is the right direction here too: over-attributing a PR to the
    maintainer costs a label, under-attributing it advertises an automated review that is
    never coming.
    """
    if is_dependabot(head_ref=head_ref, author_login=author_login):
        return (
            "dependabot bump — pr-backlog closes a red bump and leaves a green one for "
            "the maintainer; no automated review lane covers it"
        )
    return release_pr_guard.arm_block_reason(
        head_ref=head_ref, author_login=author_login, title=title
    )


def disposition(
    *,
    head_ref: object,
    head_repo: object,
    author_is_app: bool,
    repo: str,
    author_login: object = None,
    title: object = None,
) -> str:
    """The disposition of a PR: who, if anyone, is going to review it.

    Reachability is decided FIRST. A PR the registry lane can enumerate is enrolled even
    if it also looks release-ish, because the lane will in fact produce a verdict for it —
    and mislabelling an enrolled PR as maintainer-owned would hide a real review.
    """
    if lane_reachable(
        head_ref=head_ref, head_repo=head_repo, author_is_app=author_is_app, repo=repo
    ):
        return LANE
    if maintainer_owned_reason(
        head_ref=head_ref, author_login=author_login, title=title
    ):
        return MAINTAINER
    return HAND_DISPATCH


def render_label_rows() -> list[str]:
    """``name<TAB>colour<TAB>description`` per flag label, for the reify step.

    TAB-separated because every description contains spaces and one contains a comma;
    the workflow reads it with ``IFS=$'\\t'``, so a description can never be split into
    an extra field or interpreted as another argument.
    """
    return [
        "\t".join((name, *LABEL_SPECS[name])) for name in sorted(LABEL_SPECS)
    ]


def flag_label(disposition_value: str) -> str:
    """The label a green, verdict-less PR with this disposition should carry."""
    try:
        return FLAG_LABEL_BY_DISPOSITION[disposition_value]
    except KeyError:  # pragma: no cover - guarded by the self-test below
        raise ValueError(f"unknown disposition {disposition_value!r}") from None


# --------------------------------------------------------------------------- self-test
def self_test() -> int:
    failures: list[str] = []
    repo = "jeswr/sparq"

    def check(label: str, condition: bool) -> None:
        if not condition:
            failures.append(label)
        print(f"  [{'PASS' if condition else 'FAIL'}] {label}")

    def disp(**kwargs) -> str:
        base = dict(head_repo=repo, repo=repo, author_is_app=True)
        base.update(kwargs)
        return disposition(**base)

    # --- the one shape the registry lane CAN see (a real worker branch, 2026-07-26).
    check(
        "worker-App PR on a sparq-agent/issue- branch is lane-reachable",
        disp(head_ref="sparq-agent/issue-2908-30221671021-1",
             author_login="sparq-orchestrator[bot]") == LANE,
    )

    # --- THE FOUR MEASURED PRs FROM #4677: every one of them must NOT read as enrolled.
    measured = [
        # (#3798, #4193) interactive agent sessions -> a hand-dispatched verdict can arm.
        ("ci/auto-arm-workflows-permission", "jeswr", "fix: auto-arm permissions",
         False, HAND_DISPATCH),
        ("research/knowledge-management-strategy", "jeswr", "docs: knowledge management",
         False, HAND_DISPATCH),
        # (#4460) the FIRST crates.io release PR: correct App bot, wrong branch shape.
        ("release-plz-2026-07-27T02-19-35Z", "sparq-orchestrator[bot]",
         "chore: release v0.2.0", True, MAINTAINER),
        # (#4488) dependabot bumping pinned action SHAs.
        ("dependabot/github_actions/actions-minor-1234", "dependabot[bot]",
         "chore(deps): bump actions", True, MAINTAINER),
    ]
    for ref, login, title, is_app, expected in measured:
        check(
            f"#4677 measured PR on {ref!r} disposes as {expected}",
            disp(head_ref=ref, author_login=login, title=title, author_is_app=is_app)
            == expected,
        )
        check(
            f"{ref!r} is NOT enrolled in the registry lane",
            disp(head_ref=ref, author_login=login, title=title, author_is_app=is_app)
            != LANE,
        )

    # --- each reachability gate is individually load-bearing (AND, not OR).
    worker = dict(head_ref="sparq-agent/issue-1-worker", author_login="x[bot]")
    check("a fork head is unreachable", disp(**worker, head_repo="attacker/sparq") != LANE)
    check("a non-App author is unreachable", disp(**worker, author_is_app=False) != LANE)
    check(
        "issue-0 is unreachable (the registry regex demands [1-9][0-9]*)",
        disp(head_ref="sparq-agent/issue-0-worker", author_login="x[bot]") != LANE,
    )
    check(
        "a bare `sparq-agent/issue-1` with no trailing `-` is unreachable",
        disp(head_ref="sparq-agent/issue-1", author_login="x[bot]") != LANE,
    )

    # --- the maintainer axis is the LIVE release guard, not a copy of it.
    check(
        "the release axis delegates to release_pr_guard",
        maintainer_owned_reason(head_ref="release-plz-main", author_login="jeswr")
        == release_pr_guard.arm_block_reason(
            head_ref="release-plz-main", author_login="jeswr"
        ),
    )
    check(
        "an indeterminate head branch is maintainer-owned (fail-closed)",
        disp(head_ref="", author_login="jeswr", author_is_app=False) == MAINTAINER,
    )
    check(
        "dependabot is maintainer-owned by BRANCH alone",
        disp(head_ref="dependabot/cargo/serde-1.0.220", author_login="jeswr",
             title="chore(deps): bump serde", author_is_app=False) == MAINTAINER,
    )
    check(
        "dependabot is maintainer-owned by LOGIN alone",
        disp(head_ref="deps/serde", author_login="app/dependabot",
             title="chore(deps): bump serde", author_is_app=True) == MAINTAINER,
    )
    check(
        "an ordinary human PR is hand-dispatch, not maintainer-owned",
        disp(head_ref="fix/readiness-visibility", author_login="jeswr",
             title="fix: readiness visibility", author_is_app=False) == HAND_DISPATCH,
    )

    # --- reachability WINS over the maintainer axis: never hide a review that is coming.
    check(
        "an enrolled PR with a release-ish title is still lane",
        disp(head_ref="sparq-agent/issue-42-release-notes", author_login="x[bot]",
             title="chore: release v9.9.9") == LANE,
    )

    # --- label mapping is total, distinct, and never collides with the attestation.
    check(
        "every disposition maps to a distinct label",
        len(FLAG_LABELS) == 3
        and {flag_label(d) for d in (LANE, HAND_DISPATCH, MAINTAINER)} == set(FLAG_LABELS),
    )
    check(
        "no flag label is the review:pass attestation",
        "review:pass" not in FLAG_LABELS,
    )
    check(
        "every label carries a colour + description for the reify step",
        set(LABEL_SPECS) == set(FLAG_LABELS)
        and all(len(spec) == 2 and all(spec) for spec in LABEL_SPECS.values()),
    )
    check(
        f"every description fits GitHub's {MAX_LABEL_DESCRIPTION}-char cap "
        "(a 422 here fails the whole labelling run)",
        all(
            len(description) <= MAX_LABEL_DESCRIPTION
            for _colour, description in LABEL_SPECS.values()
        ),
    )
    check(
        "every colour is a 6-digit hex with no leading '#'",
        all(
            len(colour) == 6 and all(c in "0123456789abcdef" for c in colour)
            for colour, _description in LABEL_SPECS.values()
        ),
    )
    check(
        "--labels renders one TAB-separated row per label, with no stray tab",
        [row.split("\t")[0] for row in render_label_rows()] == sorted(FLAG_LABELS)
        and all(len(row.split("\t")) == 3 for row in render_label_rows()),
    )

    # --- the shape adapter reads all three spellings the way each caller sees them.
    check("login_is_app('x[bot]')", login_is_app("x[bot]"))
    check("not login_is_app('jeswr')", not login_is_app("jeswr"))
    check("not login_is_app(None)", not login_is_app(None))

    print(
        f"review_lane_visibility self-test: {'FAILED' if failures else 'OK'}"
        + (f" ({len(failures)} failure(s))" if failures else "")
    )
    return 1 if failures else 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--labels",
        action="store_true",
        help="print `name<TAB>colour<TAB>description` for every flag label",
    )
    args = parser.parse_args(argv)
    if args.labels:
        for row in render_label_rows():
            print(row)
        return 0
    if not args.self_test:
        parser.error("this module is a library; run it with --self-test or --labels")
    return self_test()


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
