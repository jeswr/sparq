#!/usr/bin/env python3
"""Bridge line-anchored ``VERDICT:`` review comments to the ``review:pass`` label.

WHY THIS EXISTS
---------------
The arming chain (``scripts/auto-arm.py`` -> ``scripts/rearm-sweeper.py`` ->
``scripts/batch-merge.py``) admits a PR on exactly one predicate: the ``review:pass``
LABEL.  The review lane's actual output artefact is a line-anchored ``VERDICT: pass``
COMMENT, and the standing review brief forbids review agents from touching labels.
Nothing in the repository ever ADDS ``review:pass`` — two workflows only REMOVE it — so
the comment -> label hop was performed exclusively by an out-of-repo orchestrator
session.  A review that lands while no orchestrator is watching is simply lost, and the
PR sits green, CLEAN and unarmed forever.  A second, related hole: a green PR that has
NO verdict (or whose verdict is bound to a superseded head) carries no ``review:*``
label at all, so it is invisible to every automated lane — auto-arm needs
``review:pass``, the fast-fix ring only fires on FAILING CI, the review dispatchers work
off ``review:needs``.

This bridge closes both holes and adds NO new arming authority: it only writes labels,
and the existing, already-tested auto-arm policy still performs the arm with its own
``expectedHeadOid`` CAS and its own security-label exclusions.

FAIL-CLOSED HEAD BINDING (the load-bearing property)
----------------------------------------------------
A verdict is honoured as a PASS only when ALL of the following hold; no missing,
malformed or unreadable input can ever produce a pass — or leave an older one standing:

* the comment's last non-blank line is exactly ``VERDICT: pass`` / ``VERDICT: fail``
  (optionally bold-wrapped) — a mention of the phrase anywhere else does not count;
* the comment body contains the CURRENT head SHA in FULL 40-hex form as a standalone
  token — a short prefix, or a SHA that is merely an ancestor, does NOT bind.  This is
  an OCCURRENCE test, deliberately: a head SHA quoted inside a commit URL or a
  ``diff --git`` index line also binds.  Binding is only what PAIRS a comment to a head;
  what makes it a *verdict* is the trusted author's trailing ``VERDICT:`` line;
* the comment's ``author_association`` is OWNER / MEMBER / COLLABORATOR — an unknown or
  drive-by association is not a reviewer.

Among the head-bound verdicts, the LATEST BY ``created_at`` wins, so a retraction
("...actually, VERDICT: fail") posted after a pass at the same head defeats the pass.
Ordering deliberately ignores ``updated_at``: ordering by an edit timestamp would let
someone edit an OLD pass comment to jump it ahead of a NEWER fail and defeat the
retraction.  ``created_at`` is immutable.

AMBIGUOUS VERDICTS (strict-alone is fail-OPEN in composition)
--------------------------------------------------------------
Strictness that merely DISCARDS a malformed line is fail-closed in isolation and
fail-OPEN in composition.  If a reviewer's retraction reads ``VERDICT: FAIL`` or
``VERDICT: fail (retracting my pass)``, discarding it leaves an OLDER, strictly-formatted
pass as the newest surviving verdict — and the superseded pass promotes.  Because review
agents are forbidden from touching labels, a comment is a reviewer's ONLY retraction
channel, so this is the load-bearing path.

So a last non-blank line that is verdict-SHAPED (``^VERDICT\b``, any case, optionally
bold-wrapped) but does not parse strictly yields ``AMBIGUOUS`` rather than ``None``.  An
ambiguous verdict never promotes, defeats any earlier pass at the same head, and REMOVES
an existing ``review:pass``: guessing the polarity of an unparseable retraction is worse
than refusing to arm.  A verdict-shaped line that is quoted, fenced, blockquoted or
bulleted is NOT verdict-shaped for this purpose (it is a mention, not a declaration), so
the instruction that tells reviewers what to write still cannot influence anything.

The hole is CHANNEL-independent, so PR REVIEW bodies are read too — on the same GraphQL
page as the PR itself, at no extra API call.  They are WITHHOLD-ONLY: a review body can
retract or suppress a pass, never grant one.  Reading one as a pass would extend arming
authority into a channel the standing review brief does not mandate, so including them
can only ever SHRINK the promote-set.

AUTHOR XOR REVIEWER (the second withhold-only axis)
---------------------------------------------------
Head binding pairs a verdict to a HEAD.  It says nothing about WHO wrote it, and until
this guard existed nothing in the arming chain compared the verdict's author to the PR's
authorship at all: a PR's own author could post ``VERDICT: pass`` on their own PR and the
bridge would promote it to ``review:pass``, after which ``auto-arm`` arms it.  MEASURED on
the live repository, not hypothesised — see the ``[OPUS-5]`` note below.

The identity axis used is the PR's CONTRIBUTOR SET at the reviewed head:

* the PR's author (who opened it), plus
* every commit author, co-author and committer GitHub resolved to an account on the
  commits that make up the head, plus
* every ``Co-authored-by:`` trailer whose address is a ``…@users.noreply.github.com``
  form, which names an account GitHub itself may have failed to resolve.

Logins are normalised (case-folded, a trailing ``[bot]`` stripped) because GitHub reports
the SAME App inconsistently: GraphQL ``PullRequest.author.login`` says
``sparq-orchestrator`` while ``GitActor.user.login`` and the REST comment author both say
``sparq-orchestrator[bot]``.  An un-normalised comparison would silently never match a bot
self-review — a guard that reads as present and is not.

A verdict from inside that set is WITHHOLD-ONLY, exactly like a review body: its ``fail``
or AMBIGUOUS still retracts (refusing a contributor's own retraction would be fail-open,
and nobody falsely fails their own PR), but its ``pass`` can never GRANT the attestation.
If the contributor set cannot be read COMPLETELY, no verdict may grant at all.

WHAT THIS AXIS CANNOT DISTINGUISH — stated because a guard whose limits are unwritten
gets mistaken for a stronger one.  It proves the verdict's author neither opened the PR
nor wrote any commit in it.  It does NOT prove the reviewer is an independent JUDGMENT:
two agent sessions driven by the same operator share one GitHub login, and when the code
was committed under a different identity (Claude Code commits as ``claude``, not as the
operator) the operator's own login is outside the contributor set.  Proving reviewer
independence needs an attestation the commenter cannot mint — the registry review lane's
ledger record — which reaches only the PRs that lane can enumerate.  So this guard is
NECESSARY, not SUFFICIENT, and a suppressed self-pass is reported (``::warning``) rather
than being allowed to fail silently.

ACTIONS
-------
promote   head-bound verdict is pass, no hold label, ``review:pass`` absent -> add it.
retract   head-bound verdict is fail (or AMBIGUOUS) and ``review:pass`` is present ->
          remove it.  Requires POSITIVE evidence (a head-bound verdict-shaped line from
          a trusted reviewer).  Absence of a verdict NEVER retracts, so a label an
          orchestrator applied by hand is never fought.  Deleting the pass COMMENT is
          therefore not a retraction gesture the bridge can see — post a fail instead.
flag      green + mergeable + non-draft + no head-bound verdict + no ``review:*`` label
          -> add ``review:unreviewed``.  Purely informational: no arming predicate
          anywhere consumes this label, so it can never block or cause a merge.
unflag    ``review:unreviewed`` is stale (a verdict or another ``review:*`` label
          arrived) -> remove it.
"""

# [OPUS-5] sparq-org/sparq — maintainer observation 2026-07-26: "there seem to be a
# number of green, ready PRs that are not in the merge queue".  Measured cause: the
# review write-path (a VERDICT comment) and the arming read-predicate (the review:pass
# label) were joined only by a human.  registry#700 records the same mismatch class.
#
# [OPUS-5] 2026-07-26, SELF-APPROVAL.  Census over all 119 open PRs (paginated,
# totalCount cross-checked): 16 head-bound verdict comments from a trusted association;
# 11 of the 16 were posted by the PR's OWN author.  Running THIS module's `decide()`
# unmodified against live sparq#4331 (non-draft, MERGEABLE, no review:* label, its only
# head-bound verdict a `pass` from its own author `jeswr`) returned
# `Decision(action='promote', ...)` — one cron tick from an unreviewed self-merge.  The
# comment author and the PR author being the same login is not an edge case here: every
# local agent session, author or reviewer, holds the same token.
#
# Two rejected fixes, both rejected on MEASUREMENT:
#   * `comment.user.login != pr.user.login` alone.  Non-vacuous for the 33 jeswr-authored
#     PRs (it catches all 11) but UNCONDITIONALLY TRUE for the 86 bot-authored ones — the
#     App opens the PR and never comments, so the check cannot fire there at all.
#   * commit authorship at the head alone.  Fires on only 3 of the 16, because Claude Code
#     commits as `claude` while the reviewing session comments as `jeswr`; it misses the
#     self-passes on #3765, #2278 and #4331 entirely.
# Neither is safe alone; their UNION is what the contributor set below computes, and even
# that is necessary-not-sufficient (see the module docstring).

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Callable, Iterable, Sequence

import gh_retry


PROGRAM = "verdict-bridge"
REVIEW_ATTESTATION = "review:pass"
UNREVIEWED_LABEL = "review:unreviewed"

# Mirrors auto-arm.py's HUMAN_OR_TRUST_LABELS + REVIEW_CHANGES_LABELS.  Promoting a PR
# that auto-arm would refuse anyway would only paint a misleading attestation onto it.
HOLD_LABELS = frozenset(
    {
        "trust-surface",
        "trust:untrusted",
        "needs:user",
        "review:needs-user",
        "review:changes",
        "review:needs",
        "area:sparq-zk",
        "area:sparq-mpc",
        "area:sparq-trust",
    }
)
# Every reviewer identity GitHub reports for a comment author. Anything outside this set
# (CONTRIBUTOR, FIRST_TIME_CONTRIBUTOR, NONE, MANNEQUIN, absent, misspelt) is not a
# reviewer for our purposes and its verdict is discarded.
TRUSTED_ASSOCIATIONS = frozenset({"OWNER", "MEMBER", "COLLABORATOR"})

VERDICT_RE = re.compile(r"^\s*(?:\*\*)?VERDICT:\s*(pass|fail)(?:\*\*)?\s*$")
# `Co-authored-by: Name <1234+login@users.noreply.github.com>` / `<login@users.noreply…>`.
# GitHub resolves most co-authors to an account on the GraphQL `authors` connection; this
# recovers the ones it did not, which would otherwise be missing from the contributor set.
COAUTHOR_NOREPLY_RE = re.compile(
    r"^\s*co-authored-by:.*<(?:[0-9]+\+)?([A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?)"
    r"@users\.noreply\.github\.com>\s*$",
    re.IGNORECASE | re.MULTILINE,
)
# Verdict-SHAPED but not strictly parseable: "VERDICT: FAIL", "VERDICT: fail (retracting
# my pass)", "VERDICT - pass".  Anchored at the START of the last non-blank line, so a
# blockquoted (`> `), fenced, bulleted or back-ticked MENTION is not verdict-shaped and
# keeps reading as "no verdict", exactly as before.
VERDICT_SHAPE_RE = re.compile(r"^\s*(?:\*\*|__)?VERDICT\b", re.IGNORECASE)
FULL_SHA_RE = re.compile(r"(?<![0-9a-fA-F])([0-9a-fA-F]{40})(?![0-9a-fA-F])")

# A trusted, head-bound, verdict-shaped line whose polarity cannot be read.  Never a
# pass, and it defeats an earlier pass — see the module docstring.
AMBIGUOUS = "ambiguous"

# Heads here reach ~1181 check-runs (12 pages).  A bound keeps a pathological or looping
# response from spinning the sweep forever; exceeding it FAILS the read, never admits it.
MAX_CHECK_RUN_PAGES = 60


class GhError(RuntimeError):
    """A GitHub CLI command failed."""


@dataclass(frozen=True)
class Verdict:
    """A line-anchored, head-bound review verdict."""

    value: str  # "pass" | "fail" | AMBIGUOUS
    author: str
    created_at: str
    comment_id: int


@dataclass(frozen=True)
class PullRequest:
    number: int
    state: str
    is_draft: bool
    base_ref: str
    head_sha: str
    mergeable: str
    gate_conclusion: str | None
    labels: frozenset[str] = field(default_factory=frozenset)
    # NORMALISED logins of everyone who authored this PR or any commit at its head.  A
    # verdict from inside this set may withhold but never grant — see the docstring.
    contributors: frozenset[str] = field(default_factory=frozenset)
    # False when the commit list was truncated, so the set above is a LOWER BOUND and
    # disjointness cannot be proven.  Refuses every grant; never blocks a retraction.
    contributors_complete: bool = True


@dataclass(frozen=True)
class Decision:
    action: str  # promote | retract | flag | unflag | none
    reason: str
    # True when a head-bound, trusted `VERDICT: pass` at this head was DISCARDED because
    # its author is in the PR's contributor set.  Drives the ::warning that stops a
    # refused self-approval from being a silent non-event.
    self_review: bool = False


def normalize_login(login: object) -> str:
    """Case-fold a GitHub login and strip a trailing ``[bot]``.

    Load-bearing, not cosmetic.  GitHub reports one App under TWO spellings: GraphQL
    ``PullRequest.author.login`` gives ``sparq-orchestrator`` while ``GitActor.user.login``
    and the REST issue-comment author both give ``sparq-orchestrator[bot]``.  Comparing
    the raw strings would make an App's self-review permanently invisible to this guard.
    """
    text = str(login or "").strip().lower()
    if text.endswith("[bot]"):
        text = text[: -len("[bot]")]
    return text


def commit_contributors(node: dict) -> tuple[frozenset[str], bool]:
    """(normalised contributor logins, completeness) for one GraphQL PR node.

    Returns ``complete=False`` when the commit connection was truncated: the set is then
    a LOWER BOUND, so a contributor could be missing from it and disjointness cannot be
    proven.  Callers must refuse to GRANT on an incomplete set — never merely warn.
    """
    people: set[str] = set()
    people.add(normalize_login((node.get("author") or {}).get("login")))
    commits = node.get("commits") or {}
    nodes = commits.get("nodes") or []
    for entry in nodes:
        commit = (entry or {}).get("commit") or {}
        for author in ((commit.get("authors") or {}).get("nodes") or []):
            people.add(normalize_login(((author or {}).get("user") or {}).get("login")))
        people.add(
            normalize_login(((commit.get("committer") or {}).get("user") or {}).get("login"))
        )
        for match in COAUTHOR_NOREPLY_RE.finditer(str(commit.get("message") or "")):
            people.add(normalize_login(match.group(1)))
    total = commits.get("totalCount")
    complete = isinstance(total, int) and total <= len(nodes)
    return frozenset(p for p in people if p), complete


def is_contributor(login: object, pr: PullRequest) -> bool:
    """True when this comment author may not GRANT: a contributor, or an unreadable set.

    Fail-closed on BOTH unknowns.  An empty/absent comment author is treated as a
    contributor because an unattributable comment cannot be shown to be independent, and
    an incomplete contributor set makes EVERY author unprovable.
    """
    if not pr.contributors_complete:
        return True
    who = normalize_login(login)
    return not who or who in pr.contributors


def hold_labels(labels: Iterable[str]) -> list[str]:
    """Every fail-closed hold, including the whole ``needs:`` namespace."""
    return sorted(
        label
        for label in labels
        if label in HOLD_LABELS or label.startswith("needs:")
    )


def trailing_verdict(body: str | None) -> str | None:
    """``"pass"`` / ``"fail"`` / ``AMBIGUOUS`` for the LAST non-blank line, else None.

    Line-anchored on purpose: the brief mandates the verdict be the final line, and a
    substring search for "VERDICT: pass" matches the INSTRUCTION that tells reviewers to
    write it (the exact false-pass this project has already been bitten by).

    A verdict-SHAPED final line that does not parse strictly returns ``AMBIGUOUS``, NOT
    None.  Returning None would let an older, well-formed pass survive a malformed
    retraction at the same head and promote it — strict-and-discard is fail-open in
    composition.  See the module docstring.
    """
    if not body:
        return None
    for line in reversed(body.splitlines()):
        if not line.strip():
            continue
        match = VERDICT_RE.match(line)
        if match:
            return match.group(1).lower()
        return AMBIGUOUS if VERDICT_SHAPE_RE.match(line) else None
    return None


def binds_head(body: str | None, head_sha: str) -> bool:
    """True iff the body cites the CURRENT head as a standalone full 40-hex SHA.

    An OCCURRENCE test by design (a SHA inside a commit URL or a diff header binds too):
    it PAIRS a comment with a head.  What makes the comment a verdict is the trusted
    author's trailing ``VERDICT:`` line, which this does not and need not attest.
    """
    if not body or len(head_sha) != 40:
        return False
    target = head_sha.lower()
    return any(m.lower() == target for m in FULL_SHA_RE.findall(body))


def classify_verdicts(
    comments: Sequence[dict], head_sha: str, pr: PullRequest | None = None
) -> tuple[list[Verdict], list[Verdict]]:
    """``(countable, self_passes)`` — every head-bound verdict from a trusted reviewer.

    ``countable`` is sorted oldest-first by immutable ``created_at`` (then comment id) so
    that editing an older comment can never reorder it ahead of a newer retraction.

    TWO withhold-only axes remove a ``pass`` from ``countable`` while leaving a ``fail``
    or AMBIGUOUS in place.  Both are one-directional on purpose: they can only ever SHRINK
    the promote-set, and a retraction from a withheld source must still land or the guard
    would be fail-open in composition.

    * ``_channel == "review"`` — a PR REVIEW body rather than an issue comment.  Reading
      one as a pass would extend arming authority into a channel the standing review
      brief does not mandate.
    * the author is in ``pr.contributors`` (or that set is unreadable) — author XOR
      reviewer.  These land in ``self_passes`` so the caller can REPORT the refusal
      instead of letting a suppressed self-approval vanish silently.

    ``pr=None`` disables only the second axis; it exists so the head-binding, ordering and
    association guards stay directly testable in isolation.
    """
    countable: list[Verdict] = []
    self_passes: list[Verdict] = []
    for comment in comments:
        if not isinstance(comment, dict):
            continue
        value = trailing_verdict(comment.get("body"))
        if value is None:
            continue
        association = str(comment.get("author_association") or "").upper()
        if association not in TRUSTED_ASSOCIATIONS:
            continue
        if not binds_head(comment.get("body"), head_sha):
            continue
        author = (comment.get("user") or {}).get("login")
        found = Verdict(
            value=value,
            author=str(author or "?"),
            created_at=str(comment.get("created_at") or ""),
            comment_id=int(comment.get("id") or 0),
        )
        if value == "pass":
            if comment.get("_channel") == "review":
                continue
            if pr is not None and is_contributor(author, pr):
                self_passes.append(found)
                continue
        countable.append(found)
    countable.sort(key=lambda v: (v.created_at, v.comment_id))
    self_passes.sort(key=lambda v: (v.created_at, v.comment_id))
    return countable, self_passes


def head_bound_verdict(
    comments: Sequence[dict], head_sha: str, pr: PullRequest | None = None
) -> Verdict | None:
    """The LATEST COUNTABLE head-bound verdict, or None."""
    countable, _ = classify_verdicts(comments, head_sha, pr)
    return countable[-1] if countable else None


def is_green_and_ready(pr: PullRequest) -> bool:
    """Green, mergeable and open for business — the population the maintainer sees."""
    return (
        not pr.is_draft
        and pr.mergeable.upper() == "MERGEABLE"
        and (pr.gate_conclusion or "").lower() == "success"
    )


def decide(pr: PullRequest, comments: Sequence[dict]) -> Decision:
    """Pure core. No I/O — every guard below is unit-tested in isolation.

    NOTE ON SCOPE: ``promote`` deliberately does NOT consult ``gate_conclusion`` or
    ``mergeable`` — it only attests that a review happened, and ``auto-arm``'s
    ``--auto`` holds a red or conflicting PR out of the queue on its own.
    ``is_green_and_ready`` therefore guards ``flag`` (visibility) only.
    """
    if pr.state.upper() != "OPEN":
        return Decision("none", f"not-open ({pr.state or 'UNKNOWN'})")
    if not pr.base_ref:
        return Decision("none", "unknown-base")

    countable, self_passes = classify_verdicts(comments, pr.head_sha, pr)
    verdict = countable[-1] if countable else None

    def out(action: str, reason: str) -> Decision:
        """Stamp every exit with the self-review flag so no refusal exits silently."""
        if self_passes:
            who = ", ".join(sorted({f"@{v.author}" for v in self_passes}))
            detail = (
                "contributor set unreadable"
                if not pr.contributors_complete
                else f"{who} contributed to this head"
            )
            reason = f"{reason}; SELF-REVIEW pass by {who} DISCARDED ({detail})"
        return Decision(action, reason, self_review=bool(self_passes))

    has_attestation = REVIEW_ATTESTATION in pr.labels
    flagged = UNREVIEWED_LABEL in pr.labels

    if verdict is None:
        # NEVER retract on absence: an orchestrator-applied label has no comment behind
        # it, and removing it would fight the human lane and un-arm a real review.
        if flagged:
            return out("none", "flagged, still unreviewed")
        other_review_label = any(
            label.startswith("review:") for label in pr.labels
        )
        if other_review_label or has_attestation:
            return out("none", "already in a review lane")
        if is_green_and_ready(pr):
            return out("flag", "green + mergeable but invisible to every lane")
        return out("none", "no head-bound verdict")

    if verdict.value == AMBIGUOUS:
        # A trusted reviewer wrote a verdict-SHAPED line at THIS head that does not
        # parse.  Reading it as "no verdict" would resurrect an older pass (fail-open);
        # guessing its polarity would arm on a coin flip.  Refuse both: never promote,
        # and drop an attestation this line may well be retracting.
        if has_attestation:
            return out(
                "retract",
                f"ambiguous verdict line by {verdict.author} supersedes "
                f"{REVIEW_ATTESTATION} — polarity unreadable, refusing to arm",
            )
        if (
            not flagged
            and is_green_and_ready(pr)
            and not any(label.startswith("review:") for label in pr.labels)
        ):
            return out(
                "flag", f"ambiguous verdict line by {verdict.author} — needs a re-review"
            )
        return out(
            "none", f"ambiguous verdict line by {verdict.author} — polarity unreadable"
        )

    if flagged:
        return out("unflag", f"verdict arrived ({verdict.value})")

    if verdict.value == "fail":
        if has_attestation:
            return out(
                "retract",
                f"head-bound fail by {verdict.author} supersedes {REVIEW_ATTESTATION}",
            )
        return out("none", "head-bound fail, no attestation to retract")

    # verdict.value == "pass"
    holds = hold_labels(pr.labels)
    if holds:
        return out("none", f"hold ({', '.join(holds)})")
    if has_attestation:
        return out("none", "already attested")
    if pr.is_draft:
        # auto-arm un-drafts on the label; do not attest a draft the author still owns.
        return out("none", "draft")
    return out(
        "promote", f"head-bound pass by {verdict.author} at {pr.head_sha[:12]}"
    )


# --------------------------------------------------------------------------- I/O


def run_gh(argv: list[str]) -> str:
    result = subprocess.run(["gh", *argv], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh failure"
        raise GhError(f"gh {' '.join(argv[:3])} failed: {detail}")
    return result.stdout


def run_gh_read(argv: list[str]) -> str:
    try:
        return gh_retry.run_gh_read(argv)
    except gh_retry.GhFatalError as error:
        raise GhError(str(error)) from error


PR_LIST_QUERY = """query($owner:String!,$name:String!,$cursor:String){
  repository(owner:$owner,name:$name){
    pullRequests(states:OPEN,first:50,after:$cursor){
      pageInfo{hasNextPage endCursor}
      totalCount
      nodes{
        number state isDraft baseRefName headRefOid mergeable
        author{login}
        labels(first:100){nodes{name} pageInfo{hasNextPage}}
        commits(last:250){
          totalCount
          nodes{commit{
            message
            authors(first:20){nodes{user{login}}}
            committer{user{login}}
          }}
        }
        reviews(last:30){nodes{
          databaseId body authorAssociation submittedAt author{login}
        }}
      }
    }
  }
}"""


class VerdictBridge:
    def __init__(
        self,
        repo: str,
        default_branch: str,
        *,
        gh: Callable[[list[str]], str] = run_gh,
        gh_read: Callable[[list[str]], str] | None = None,
        log: Callable[[str], None] = print,
        dry_run: bool = False,
        max_writes: int = 25,
    ) -> None:
        if "/" not in repo or repo.count("/") != 1 or not all(repo.split("/")):
            raise ValueError("repo must be OWNER/REPOSITORY")
        self.repo = repo
        self.owner, self.name = repo.split("/", 1)
        self.default_branch = default_branch
        self.gh = gh
        self.gh_read = gh_read if gh_read is not None else gh
        self.log = log
        self.dry_run = dry_run
        self.max_writes = max_writes

    # -- reads --------------------------------------------------------------

    def list_open(self) -> list[dict]:
        """Every open PR, EXPLICITLY paged and cross-checked against totalCount.

        `gh api --paginate` concatenates one JSON object per page with no separator, so
        a single json.loads over it raises; page by cursor instead and assert the count.
        """
        nodes: list[dict] = []
        cursor: str | None = None
        total: int | None = None
        while True:
            argv = [
                "api",
                "graphql",
                "-f",
                f"query={PR_LIST_QUERY}",
                "-F",
                f"owner={self.owner}",
                "-F",
                f"name={self.name}",
            ]
            if cursor:
                argv += ["-F", f"cursor={cursor}"]
            payload = json.loads(self.gh_read(argv))
            if payload.get("errors"):
                raise GhError(f"GraphQL returned errors: {payload['errors']}")
            page = ((payload.get("data") or {}).get("repository") or {}).get(
                "pullRequests"
            )
            if not page:
                raise GhError("GraphQL response carried no pullRequests connection")
            total = page.get("totalCount") if total is None else total
            nodes.extend(page.get("nodes") or [])
            info = page.get("pageInfo") or {}
            if not info.get("hasNextPage"):
                break
            cursor = info.get("endCursor")
            if not cursor:
                raise GhError("hasNextPage without an endCursor")
        if total is not None and len(nodes) != total:
            raise GhError(f"paged {len(nodes)} PRs but totalCount was {total}")
        return nodes

    def check_runs(self, sha: str) -> list[dict]:
        """Every check-run for ``sha``, EXPLICITLY paged and de-duplicated by id.

        PRs here carry hundreds of check-runs; an unpaginated read silently truncates
        and can hide the `gate` run entirely, which reads as "no gate" -> not green.

        The cross-check against ``total_count`` is ONE-SIDED, and that asymmetry is
        load-bearing.  A live sweep measured ``paged 479 check-runs, total_count=473`` on
        sparq#4074: runs were being CREATED between page reads, so the list GREW under
        pagination.  Reading MORE than the first page's ``total_count`` is benign — the
        risk this guard exists for is TRUNCATION, which reads FEWER.  A two-sided
        equality check turned a routine mid-sweep re-run into a skipped PR plus a red
        workflow every cycle.  Growth also repeats entries across page boundaries, hence
        the de-duplication by run id.
        """
        runs: list[dict] = []
        seen: set = set()
        total: int | None = None
        page = 1
        while True:
            payload = json.loads(
                self.gh_read(
                    [
                        "api",
                        f"/repos/{self.repo}/commits/{sha}/check-runs"
                        f"?per_page=100&page={page}",
                    ]
                )
            )
            total = payload.get("total_count") if total is None else total
            batch = payload.get("check_runs") or []
            for run in batch:
                if isinstance(run, dict) and run.get("id") not in seen:
                    seen.add(run.get("id"))
                    runs.append(run)
            # Terminate on a SHORT page, never on `len(runs) >= total_count`: a
            # total_count read before a re-run started is an UNDERCOUNT, and stopping on
            # it would truncate exactly when the list is growing — hiding the newest
            # `gate` run, which is the one that matters.
            if len(batch) < 100:
                break
            page += 1
            if page > MAX_CHECK_RUN_PAGES:
                raise GhError(f"{sha}: check-runs exceeded {MAX_CHECK_RUN_PAGES} pages")
        if total is not None and len(runs) < total:
            # TRUNCATION — the `gate` run may be in the part we never read.  Fail closed.
            raise GhError(f"{sha}: paged only {len(runs)} check-runs, total_count={total}")
        return runs

    def gate_conclusion(self, sha: str) -> str | None:
        """Newest-run-per-name resolution of the `gate` check.

        A cancelled or superseded twin shares the name; taking any-run would read a
        stale result as the live one.
        """
        newest: tuple[str, int] | None = None
        chosen: dict | None = None
        for run in self.check_runs(sha):
            if run.get("name") != "gate":
                continue
            key = (str(run.get("started_at") or ""), int(run.get("id") or 0))
            if newest is None or key > newest:
                newest, chosen = key, run
        if chosen is None or chosen.get("status") != "completed":
            return None
        return chosen.get("conclusion")

    def comments(self, number: int) -> list[dict]:
        out: list[dict] = []
        page = 1
        while True:
            batch = json.loads(
                self.gh_read(
                    [
                        "api",
                        f"/repos/{self.repo}/issues/{number}/comments"
                        f"?per_page=100&page={page}",
                    ]
                )
            )
            if not isinstance(batch, list):
                raise GhError(f"PR #{number}: comments response was not a list")
            out.extend(batch)
            if len(batch) < 100:
                break
            page += 1
        return out

    # -- writes -------------------------------------------------------------

    @staticmethod
    def review_withholdings(node: dict) -> list[dict]:
        """PR REVIEW bodies, normalised onto the issue-comment shape.

        Carried on the SAME GraphQL page as the PR itself, so this costs no extra API
        call.  Tagged ``_channel="review"``, which makes them WITHHOLD-ONLY in
        ``head_bound_verdict``: they can retract or suppress a pass, never grant one.
        A reviewer who posts a pass as a comment and then withdraws it in a review body
        would otherwise leave the superseded pass standing — the same composition hole
        as an unparseable retraction, through a different channel.
        """
        out: list[dict] = []
        for review in ((node.get("reviews") or {}).get("nodes") or []):
            if not isinstance(review, dict) or not review.get("submittedAt"):
                continue  # a PENDING review has not been submitted; it is not evidence.
            out.append(
                {
                    "_channel": "review",
                    "id": review.get("databaseId") or 0,
                    "body": review.get("body"),
                    "author_association": review.get("authorAssociation"),
                    "created_at": review.get("submittedAt"),
                    "user": review.get("author") or {},
                }
            )
        return out

    def edit_labels(self, number: int, *, add: str = "", remove: str = "") -> None:
        argv = ["pr", "edit", str(number), "--repo", self.repo]
        if add:
            argv += ["--add-label", add]
        if remove:
            argv += ["--remove-label", remove]
        self.gh(argv)

    # -- driver -------------------------------------------------------------

    def parse_node(self, node: dict, gate: str | None) -> PullRequest:
        labels = node.get("labels") or {}
        if (labels.get("pageInfo") or {}).get("hasNextPage"):
            # Fail closed: an unseen label could be a hold.
            raise GhError(f"PR #{node.get('number')}: label set exceeds one page")
        contributors, complete = commit_contributors(node)
        return PullRequest(
            number=int(node["number"]),
            state=str(node.get("state") or "").upper(),
            is_draft=bool(node.get("isDraft")),
            base_ref=str(node.get("baseRefName") or ""),
            head_sha=str(node.get("headRefOid") or ""),
            mergeable=str(node.get("mergeable") or ""),
            gate_conclusion=gate,
            labels=frozenset(
                str(item.get("name", "")).strip().lower()
                for item in (labels.get("nodes") or [])
                if isinstance(item, dict) and str(item.get("name", "")).strip()
            ),
            contributors=contributors,
            contributors_complete=complete,
        )

    def run(self) -> int:
        errors = 0
        writes = 0
        self_reviews = 0
        nodes = self.list_open()
        self.log(f"[{PROGRAM}] {len(nodes)} open PR(s); dry_run={self.dry_run}")
        for node in nodes:
            number = node.get("number")
            try:
                if str(node.get("baseRefName") or "") != self.default_branch:
                    continue
                head = str(node.get("headRefOid") or "")
                # Cheap pre-filter: only PRs that could possibly need a gate read.
                gate = self.gate_conclusion(head) if head else None
                pr = self.parse_node(node, gate)
                evidence = self.comments(pr.number) + self.review_withholdings(node)
                decision = decide(pr, evidence)
            except (GhError, json.JSONDecodeError, KeyError, ValueError) as error:
                self.log(f"[{PROGRAM}] PR #{number}: SKIP inspect-failed ({error})")
                errors += 1
                continue

            # VISIBLE TERMINAL STATE.  Emitted BEFORE the `none` short-circuit and
            # independently of the action, because the whole point of this guard is that
            # a refused self-approval usually produces NO label change at all — and a
            # refusal that writes nothing and says nothing is indistinguishable from
            # never having looked.  ::warning surfaces it in the Actions run summary; the
            # `flag` action puts `review:unreviewed` on the PR itself, which is what
            # review-lane-alarm censuses as a blind spot.
            if decision.self_review:
                self_reviews += 1
                self.log(
                    f"::warning title={PROGRAM} refused a self-review::PR #{pr.number}: "
                    f"{decision.reason}"
                )
            if decision.action == "none":
                continue
            self.log(
                f"[{PROGRAM}] PR #{pr.number}: {decision.action.upper()} — "
                f"{decision.reason}"
            )
            if self.dry_run:
                continue
            if writes >= self.max_writes:
                self.log(f"[{PROGRAM}] PR #{pr.number}: SKIP per-run write cap reached")
                continue
            add, remove = {
                "promote": (REVIEW_ATTESTATION, UNREVIEWED_LABEL),
                "retract": ("", REVIEW_ATTESTATION),
                "flag": (UNREVIEWED_LABEL, ""),
                "unflag": ("", UNREVIEWED_LABEL),
            }[decision.action]
            if decision.action == "promote" and UNREVIEWED_LABEL not in pr.labels:
                remove = ""
            try:
                self.edit_labels(pr.number, add=add, remove=remove)
                writes += 1
            except GhError as error:
                self.log(f"[{PROGRAM}] PR #{pr.number}: label edit failed ({error})")
                errors += 1
        self.log(
            f"[{PROGRAM}] complete: writes={writes} errors={errors} "
            f"self_reviews_refused={self_reviews}"
        )
        return errors


def run_bridge(bridge: VerdictBridge, log: Callable[[str], None] = print) -> int:
    """Exhausted TRANSIENT reads end the cycle as ::warning + 0 — the cron covers it.

    Any NON-transient failure still propagates and reds the run.
    """
    try:
        errors = bridge.run()
    except gh_retry.GhTransientExhausted as error:
        log(
            f"::warning title={PROGRAM} skipped a cycle on transient GitHub API "
            f"failures::{error} — bounded retries exhausted; the cron backstop covers "
            "a missed cycle, so this run reports success."
        )
        return 0
    return 1 if errors else 0


# --------------------------------------------------------------------------- tests


def comment(
    body: str,
    *,
    association: str = "OWNER",
    created_at: str = "2026-07-26T10:00:00Z",
    cid: int = 1,
    user: str = "reviewer",
) -> dict:
    return {
        "id": cid,
        "body": body,
        "author_association": association,
        "created_at": created_at,
        "updated_at": created_at,
        "user": {"login": user},
    }


HEAD = "a" * 40
OTHER = "b" * 40


def pr_fixture(**overrides) -> PullRequest:
    base = {
        "number": 4200,
        "state": "OPEN",
        "is_draft": False,
        "base_ref": "main",
        "head_sha": HEAD,
        "mergeable": "MERGEABLE",
        "gate_conclusion": "success",
        "labels": frozenset(),
    }
    base.update(overrides)
    base["labels"] = frozenset(base["labels"])
    return PullRequest(**base)


def self_test() -> None:
    passing = comment(f"Reviewed {HEAD}.\n\nVERDICT: pass")
    failing = comment(f"Reviewed {HEAD}.\n\nVERDICT: fail", cid=2)

    # A head-bound pass on a clean PR promotes.
    assert decide(pr_fixture(), [passing]).action == "promote"

    # Head binding: a verdict citing a DIFFERENT sha does not bind, so the PR reads as
    # unreviewed (and, being green, gets flagged) rather than promoted.
    stale = comment(f"Reviewed {OTHER}.\n\nVERDICT: pass")
    assert decide(pr_fixture(), [stale]).action == "flag", "stale sha must not promote"
    # A short prefix of the head is NOT a binding.
    short = comment(f"Reviewed {HEAD[:12]}.\n\nVERDICT: pass")
    assert decide(pr_fixture(), [short]).action == "flag", "prefix must not bind"
    # A 40-hex token embedded in a longer hex run is not a standalone SHA.
    glued = comment(f"Reviewed {HEAD}cafe.\n\nVERDICT: pass")
    assert decide(pr_fixture(), [glued]).action == "flag", "glued sha must not bind"

    # Line anchoring: the phrase inside a quoted INSTRUCTION is not a verdict.
    quoted = comment(
        f"Reviewed {HEAD}. The brief says to end with `VERDICT: pass` or "
        "`VERDICT: fail`.\n\nStill working on it."
    )
    assert decide(pr_fixture(), [quoted]).action == "flag", "substring must not pass"
    assert trailing_verdict("VERDICT: pass\n\ntrailing prose") is None
    # A MENTION (quoted / blockquoted / fenced / bulleted) is not verdict-shaped.
    for mention in ("> VERDICT: pass", "- VERDICT: pass", "`VERDICT: pass`", "```"):
        assert trailing_verdict(mention) is None, mention

    # AMBIGUITY: a verdict-SHAPED final line that does not parse must NOT read as "no
    # verdict" — that would leave an older, well-formed pass as the newest verdict.
    for shaped in (
        "VERDICT: FAIL",
        "VERDICT: fail (retracting my pass)",
        "**VERDICT: Fail**",
        "VERDICT - pass",
        "verdict: unclear",
    ):
        assert trailing_verdict(shaped) == AMBIGUOUS, shaped
    stale_pass = comment(
        f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
    )
    for shaped in ("VERDICT: FAIL", "VERDICT: fail (retracting my pass)"):
        retraction = comment(
            f"{HEAD}\n\n{shaped}", created_at="2026-02-01T00:00:00Z", cid=2
        )
        combo = [stale_pass, retraction]
        assert head_bound_verdict(combo, HEAD).value == AMBIGUOUS, shaped
        assert decide(pr_fixture(), combo).action != "promote", shaped
        # ... and it REMOVES an attestation it may be retracting.
        held = decide(pr_fixture(labels={REVIEW_ATTESTATION}), combo)
        assert held.action == "retract", (shaped, held)
    # An ambiguous line from an UNTRUSTED author is still ignored entirely (no DoS on
    # arming by a drive-by "VERDICT: FAIL").
    drive_by_shaped = comment(
        f"{HEAD}\n\nVERDICT: FAIL", association="NONE", created_at="2026-02-01T00:00:00Z", cid=3
    )
    assert decide(pr_fixture(), [stale_pass, drive_by_shaped]).action == "promote"

    # Provenance: an untrusted association is not a reviewer.
    for association in ("CONTRIBUTOR", "FIRST_TIME_CONTRIBUTOR", "NONE", "", "owner "):
        drive_by = comment(f"Reviewed {HEAD}.\n\nVERDICT: pass", association=association)
        assert decide(pr_fixture(), [drive_by]).action == "flag", association

    # RETRACTION: a later head-bound fail defeats an earlier pass at the same head.
    early = comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-07-26T10:00:00Z", cid=1)
    later = comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-07-26T11:00:00Z", cid=2)
    assert decide(pr_fixture(), [early, later]).action != "promote"
    assert decide(pr_fixture(), [later, early]).action != "promote", "order-independent"
    # ... and it REMOVES an attestation that is already on the PR.
    retract = decide(pr_fixture(labels={REVIEW_ATTESTATION}), [early, later])
    assert retract.action == "retract", retract
    # Editing the OLD pass must not reorder it ahead of the newer fail.
    edited = dict(early, updated_at="2026-07-26T23:00:00Z")
    assert decide(pr_fixture(), [edited, later]).action != "promote", "edit reorders"
    # A newer PASS after a fail does re-attest.
    newer_pass = comment(
        f"{HEAD}\n\nVERDICT: pass", created_at="2026-07-26T12:00:00Z", cid=3
    )
    assert decide(pr_fixture(), [later, newer_pass]).action == "promote"

    # Holds are fail-closed, matching auto-arm's exclusion set.
    for hold in sorted(HOLD_LABELS) + ["needs:design", "needs:anything"]:
        held = decide(pr_fixture(labels={hold}), [passing])
        assert held.action == "none", (hold, held)
        assert "hold" in held.reason, (hold, held)

    # Never retract on ABSENCE of evidence — a hand-applied label survives.
    kept = decide(pr_fixture(labels={REVIEW_ATTESTATION}), [])
    assert kept.action == "none", kept
    kept_stale = decide(pr_fixture(labels={REVIEW_ATTESTATION}), [stale])
    assert kept_stale.action == "none", kept_stale

    # Idempotence: an already-attested PR is not re-promoted.
    assert decide(pr_fixture(labels={REVIEW_ATTESTATION}), [passing]).action == "none"

    # Drafts and non-open PRs are never attested.
    assert decide(pr_fixture(is_draft=True), [passing]).action == "none"
    assert decide(pr_fixture(state="MERGED"), [passing]).action == "none"

    # FLAG: the invisible population — green, mergeable, non-draft, no review label.
    assert decide(pr_fixture(), []).action == "flag"
    #   ... but not a draft, not a conflicting PR, not a red/absent gate,
    #   and not one already in a review lane.
    assert decide(pr_fixture(is_draft=True), []).action == "none"
    assert decide(pr_fixture(mergeable="CONFLICTING"), []).action == "none"
    assert decide(pr_fixture(gate_conclusion="failure"), []).action == "none"
    assert decide(pr_fixture(gate_conclusion=None), []).action == "none"
    for lane in ("review:needs", "review:changes", "review:needs-user", "review:pass"):
        assert decide(pr_fixture(labels={lane}), []).action == "none", lane

    # UNFLAG: the informational label is dropped as soon as a verdict binds.
    assert decide(pr_fixture(labels={UNREVIEWED_LABEL}), [passing]).action == "unflag"
    assert decide(pr_fixture(labels={UNREVIEWED_LABEL}), [failing]).action == "unflag"
    assert decide(pr_fixture(labels={UNREVIEWED_LABEL}), []).action == "none"

    # The informational label must never be a hold (it would deadlock the arm lane).
    assert UNREVIEWED_LABEL not in HOLD_LABELS
    assert not hold_labels({UNREVIEWED_LABEL})

    # Malformed comment payloads are ignored, never treated as a pass.
    assert head_bound_verdict([None, 7, {}, {"body": None}], HEAD) is None  # type: ignore[list-item]

    # A PR REVIEW body may WITHHOLD but never GRANT.
    review_fail = dict(
        comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-02-01T00:00:00Z", cid=9),
        _channel="review",
    )
    review_pass = dict(
        comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-02-01T00:00:00Z", cid=9),
        _channel="review",
    )
    old_pass = comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z")
    assert decide(pr_fixture(), [old_pass, review_fail]).action != "promote"
    assert (
        decide(pr_fixture(labels={REVIEW_ATTESTATION}), [old_pass, review_fail]).action
        == "retract"
    )
    # ... and a review body ALONE never grants the attestation.
    assert decide(pr_fixture(), [review_pass]).action == "flag"
    # A review fail cannot be overridden by an even newer review pass either.
    newest_review_pass = dict(
        comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-03-01T00:00:00Z", cid=10),
        _channel="review",
    )
    assert decide(pr_fixture(), [old_pass, review_fail, newest_review_pass]).action != "promote"

    # AUTHOR XOR REVIEWER.  A verdict from the PR's own contributor set may WITHHOLD but
    # never GRANT.  The control arm is load-bearing: with the guard deleted BOTH arms
    # promote and the second assertion fires, so this is non-vacuous for the same-login
    # case that is the whole reason the guard exists.
    contrib = pr_fixture(contributors=frozenset({"jeswr", "claude"}))
    self_pass = comment(f"{HEAD}\n\nVERDICT: pass", user="jeswr")
    indep_pass = comment(f"{HEAD}\n\nVERDICT: pass", user="reviewer")
    assert decide(contrib, [indep_pass]).action == "promote", "control arm must promote"
    refused = decide(contrib, [self_pass])
    assert refused.action != "promote", refused
    assert refused.self_review and "SELF-REVIEW" in refused.reason, refused
    # ... and the bot-suffix spelling difference does not launder it.
    app_contrib = pr_fixture(contributors=frozenset({"sparq-orchestrator"}))
    app_self = comment(f"{HEAD}\n\nVERDICT: pass", user="sparq-orchestrator[bot]")
    assert decide(app_contrib, [app_self]).action != "promote"
    # ... a contributor's FAIL still retracts (withhold-only, never a DoS on retraction).
    own_fail = comment(f"{HEAD}\n\nVERDICT: fail", user="jeswr")
    held = pr_fixture(
        labels={REVIEW_ATTESTATION}, contributors=frozenset({"jeswr", "claude"})
    )
    assert decide(held, [own_fail]).action == "retract"
    # ... an unreadable contributor set refuses every grant, but not a retraction.
    torn = pr_fixture(contributors_complete=False)
    assert decide(torn, [indep_pass]).action != "promote"
    assert (
        decide(
            pr_fixture(labels={REVIEW_ATTESTATION}, contributors_complete=False),
            [comment(f"{HEAD}\n\nVERDICT: fail", user="reviewer")],
        ).action
        == "retract"
    )
    # ... and a clean PR does not report a phantom self-review.
    assert not decide(contrib, [indep_pass]).self_review

    print(f"{PROGRAM} self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser(description="Bridge VERDICT comments to labels.")
    parser.add_argument("--repo", help="owner/repository to sweep")
    parser.add_argument("--default-branch", default="main")
    parser.add_argument("--dry-run", action="store_true", help="report, write nothing")
    parser.add_argument("--max-writes", type=int, default=25)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return 0
    if not args.repo:
        parser.error("--repo is required unless --self-test is used")
    return run_bridge(
        VerdictBridge(
            args.repo,
            args.default_branch,
            gh_read=run_gh_read,
            dry_run=args.dry_run,
            max_writes=args.max_writes,
        )
    )


if __name__ == "__main__":
    sys.exit(main())
