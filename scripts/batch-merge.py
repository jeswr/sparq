#!/usr/bin/env python3
# Omnibus merge-queue overflow batcher (scheduled / event-driven, NON-gating).
#
# WHAT THIS IS
# ------------
# The deterministic policy core of .github/workflows/batch-merge.yml. The merge queue on
# `main` (ALLGREEN, max_entries_to_merge=8) drains individually-armed worker PRs one queue
# window at a time. This script batches ALL reviewed, armed worker PRs into ONE
# integration ("omnibus") PR so a single queue slot lands every reviewed constituent at
# once; a broken batch is recursively BISECTED (rule 6) until the culprit is isolated.
# It makes NO model calls: pure, testable policy over a GitHub/git state snapshot.
#
# POLICY (maintainer-directed, 2026-07-17; v2 batch-everything + bisection 2026-07-18)
# ------------------------------------------------------------------------------------
#  1. ELIGIBILITY. A constituent is an OPEN PR whose head branch matches ^sparq-agent/,
#     carrying label `review:pass` AND an ACTIVE auto-merge arm enabled by
#     app/sparq-orchestrator. NEVER touched: release-plz PRs, dependabot/* branches,
#     drafts, PRs labeled needs:user or trust:*, or any non-sparq-agent branch.
#  2. BATCH EVERYTHING (v2). QUEUE_RESERVE (0) eligible PRs are left to the queue
#     individually; ALL the rest — i.e. every eligible PR — batch into one omnibus,
#     ascending by PR number. Fewer than MIN_CONSTITUENTS (2) -> no-op (a lone armed PR
#     merges fine through the queue on its own). Background: the merge queue's
#     max_entries_to_merge stays 8, but one omnibus is ONE queue entry however many
#     constituents it carries, so the batch is uncapped — broken-code risk is handled by
#     recursive bisection (rule 6), not by capping batch size.
#  3. INTEGRATION BRANCH. sparq-omnibus/<utcstamp> fresh off origin/main; sequentially
#     `git merge --no-ff` each overflow branch (ascending PR number). A conflicting
#     constituent is SKIPPED (it stays individually armed) and recorded in the PR body.
#     If fewer than MIN_CONSTITUENTS merge cleanly the branch is deleted and no PR opens.
#  4. ONE PR. Title `omnibus: <n> reviewed worker PRs`; body = SPARQ-agent self-ID +
#     machine marker (constituent PR numbers) + constituent table (PR, issue, crate) +
#     one `Closes #<issue>` line per constituent target issue. The PR is armed with
#     `gh pr merge --auto` (strategy is chosen by the merge queue). The sparq-omnibus/
#     prefix keeps it OUT of the worker review loop: the registry's enumerator
#     (dispatch-claim.py) admits only heads matching ^sparq-agent/issue-<n>- and keys
#     state off review:* labels, so the omnibus carries NO review:* label ever.
#  5. CONSTITUENT CLOSURE. For every MERGED omnibus PR (found via the body marker), any
#     still-open constituent whose branch now adds NOTHING vs origin/main (merge-tree of
#     origin/main + branch == origin/main's tree) is closed with a comment linking the
#     omnibus. Its issue closes via the omnibus's `Closes #` refs. Idempotent: closed
#     constituents drop out of the open set; a constituent with post-omnibus commits is
#     NOT empty and is left alone.
#  6. FAILURE PATH (v2: recursive bisection). Open-omnibus triage is ORDERED,
#     most-exculpatory first — a conflict must NEVER implicate constituents:
#       (i)   CONFLICTING with main, checked FIRST — even when a concluded gate
#             failure is ALSO present (main moved under the omnibus, so that verdict
#             was evidence against a stale base, not against any constituent): v1
#             close only — never bisection, never quarantine; close-and-recreate
#             rebuilds a fresh root off current main in the same run.
#       (ii)  head `gate` CONCLUDED in strict FAILURE on a DEFINITIVELY MERGEABLE
#             head — live because the omnibus is pushed/created with the
#             sparq-orchestrator App token, so head pull_request CI fires on it like
#             on any worker PR. This is the ONLY constituent-fault signal: the
#             omnibus BISECTS (below). Mergeability UNKNOWN/unreported is NOT
#             definitive (after main advances GitHub reports UNKNOWN before
#             resolving — possibly to CONFLICTING, which must never implicate
#             anyone): the omnibus WAITS untouched that run; only the age-expiry
#             NON-culpable close may act on a persistently-unresolved omnibus.
#       (iii) head `gate` concluded cancelled / timed_out, OR the omnibus is OLDER
#             than MAX_OMNIBUS_AGE_HOURS (stamp parsed from its branch name — the
#             liveness backstop for every head-invisible failure mode: a merge-group
#             `gate` failure on the queue's synthetic ref, a dropped auto-merge arm,
#             checks that never reported). INFRA-shaped, NOT a code verdict: v1
#             close only — never bisection (under a sustained outage the split
#             cascade would double the node count every pass, 2->4->8->16, then
#             expire the singleton leaves and flood the queue) — and NO same-run
#             root rebuild; a new root waits out ROOT_COOLDOWN_HOURS (below).
#     Bisection (case ii only): CAUSAL = the failing omnibus's marker constituents
#     whose code is IN the failed head and not cleanly excluded — the only clean
#     exclusion is positive evidence the code already LANDED (empty vs main).
#     SURVIVORS = the CAUSAL constituents that are also still OPEN PRs, still
#     armed-eligible (per-PR re-check of rule 1). ELIGIBILITY IS NOT CAUSALITY: a
#     constituent that became draft/unarmed/relabeled/closed WITHOUT landing still
#     has its code in the failed head — it may be the actual culprit, so it BLOCKS
#     single-survivor conviction (below) even though it cannot be re-batched. Then:
#       * >=2 survivors, depth < DEPTH_CAP: close the parent (the comment NAMES both
#         child branches) + delete its branch, and create TWO child omnibuses (first /
#         second half of the survivors, ascending order preserved) at depth+1 with the
#         SAME creation machinery as a root. Sibling children co-exist (children are
#         exempt from the one-open-omnibus rule below); a singleton CHILD is legal —
#         MIN_CONSTITUENTS applies to ROOT creation only, and a singleton child is
#         exactly how a culprit gets isolated.
#       * exactly 1 survivor, the parent's head `gate` CONCLUDED in strict FAILURE,
#         AND the CAUSAL set is exactly that survivor: THE CULPRIT. It is NOT closed. It is commented (machine marker
#         `sparq-omnibus-culprit:v1`), converted to DRAFT (`gh pr ready --undo`) and
#         disarmed — in THAT order (crash safety, 6b). Draft + unarmed + review:pass is
#         the registry's `stranded` posture, which escalates loudly to a human
#         (needs:user). The culprit's OWN head gate stays green — head CI does not
#         re-run when main advances, and the failure evidence lived on the now-closed
#         child-omnibus head — so a gate-keyed ci-fix admission can never see it;
#         stranded IS the guaranteed-closure handoff. The parent omnibus closes as in
#         v1.
#       * exactly 1 survivor but a LARGER causal set (a code-bearing sibling became
#         ineligible without landing): conviction is UNSOUND — the parent closes and
#         a fresh singleton RE-TEST child carrying only the survivor is created off
#         current main (its failure IS sound evidence). When no child can be created
#         (hygiene-only, depth cap, truncated census) the non-culpable v1 close runs
#         instead: never quarantine on ambiguous causality.
#       * exactly 1 survivor on a NON-verdict failure: unreachable by construction
#         since only case (ii) computes survivors at all, but kept as an in-code
#         belt-and-braces guard — if bisectability is ever widened again, a survivor
#         must still only be CONVICTED by a strict concluded head-gate FAILURE
#         (v1 close; survivor stays individually armed).
#       * 0 survivors: plain close (everything already landed elsewhere).
#       * depth >= DEPTH_CAP with >=2 survivors: v1 fallback — close, constituents stay
#         individually armed, loud log, no same-run root rebuild.
#     TRUNCATED LISTING (open_list_truncated: the open-PR listing saturated its fetch
#     limit): incomplete data must never drive a destructive act. Absence from the
#     list must never convict anything, so every survivor census degrades to a v1
#     close (no bisection, no quarantine); an omitted LIVE omnibus would also be
#     misclassified as stale and a new root would break the one-tree guarantee, so
#     stale-branch deletion (rule 8) AND new-root creation are BOTH suppressed for
#     that run (fail-safe: do nothing destructive, retry on complete data).
#     ROOT COOLDOWN (durable, cross-run; the infra-flood backstop): the batcher is
#     stateless, so the cooldown state is read back from GitHub's closed-PR record —
#     any sparq-omnibus PR closed UNMERGED within the last ROOT_COOLDOWN_HOURS
#     suppresses new-ROOT creation. Combined with (iii)'s no-same-run-rebuild, a
#     persistently failing environment converges to at most ~one root per
#     (MAX_OMNIBUS_AGE_HOURS + ROOT_COOLDOWN_HOURS) instead of a rebuild flood.
#     Bisection CHILDREN are exempt (the cooldown damps re-rooting, never culprit
#     isolation). In --hygiene-only, children cannot be created (no App token, see
#     TOKENS), so a bisectable parent v1-closes instead; the culprit path (comment +
#     draft + disarm) still runs. A dying CONFLICTING or bisected omnibus does NOT
#     suppress a new ROOT in the same run (close-and-recreate), but an infra-closed
#     one does, and freshly spawned children always do — one omnibus TREE at a time,
#     so one bad batch can never wedge the batcher.
#  6b. CRASH SAFETY (by construction). Batching never closes or disarms a constituent
#     except the isolated culprit; an omnibus is an integration COPY of individually
#     armed PRs. If a run dies between closing a failed parent and creating its
#     children, the constituents remain individually armed (exactly v1 semantics) and
#     the next cron builds a fresh root — no state is lost, nothing wedges. Partial
#     child creation self-heals the same way: an orphan child is a normal open omnibus
#     (it merges or re-enters this failure path), and the missing sibling's
#     constituents stay individually armed and re-batch once the tree drains. The
#     culprit mutation order (comment -> draft -> disarm) is load-bearing: a crash or
#     raised gh failure after only the comment leaves the culprit ARMED, so the next
#     run re-isolates it and repeats the sequence idempotently (a duplicate comment is
#     harmless); once the draft conversion lands, GitHub itself drops the auto-merge
#     arm, so the terminal stranded posture is already reached even if the explicit
#     disarm never runs. No cut point can leave a disarmed-but-unmarked PR — the arm
#     is only ever stripped AFTER the durable breadcrumb exists.
#  7. RE-ARM LIVENESS. GitHub DROPS the auto-merge arm when a merge group fails. Each
#     run, an open, young (under the age bound), MERGEABLE omnibus with NO active arm is
#     re-armed idempotently (`gh pr merge --auto`) — a transient queue failure gets
#     retried; a persistent one hits the age bound and is closed.
#  8. STALE HYGIENE. Any sparq-omnibus/* remote branch with no open PR is deleted —
#     SKIPPED entirely when the open-PR listing may be truncated (rule 6: an omitted
#     live omnibus must not lose its branch). Branch deletion is BEST-EFFORT
#     everywhere (an already-deleted branch is success; a persistent delete failure
#     logs loudly but never blocks the child/root creation planned after it).
#  9. PROVENANCE. A PR is classified as an omnibus RECORD — for the open (close /
#     bisect / quarantine / root-suppression), merged (constituent closure), cooldown
#     and stale-branch legs alike — ONLY when its head is SAME-REPOSITORY and its
#     author is the orchestrator App (the pipeline's trusted identity,
#     sparq-orchestrator[bot]). A fork or non-App PR with a spoofed sparq-omnibus/
#     prefix is ignored with a loud log: it can never draft/disarm a constituent nor
#     wedge root creation. Its branch (if same-repo) is still protected from stale
#     deletion while the PR is open — deleting under ANY open PR is destructive.
#
# TOKENS. The omnibus branch/PR MUST be pushed + created with the sparq-orchestrator App
# installation token: GITHUB_TOKEN-created refs/PRs get their workflow events SUPPRESSED,
# so ci-summary would never run on the omnibus head, the required `gate` context would
# never report, and the merge queue would never ADMIT the PR (admission requires the
# required checks to pass on the head; merge-group evaluation happens only after
# admission) — the PR would wedge open forever (empirically: GITHUB_TOKEN-created PR
# #1084, zero check-runs, BLOCKED since 2026-06-21). When the App credentials are absent
# the workflow runs this script with --hygiene-only: closure/failure/re-arm/stale legs
# still run, but NO new omnibus is created.
#
# DESIGN: policy is a PURE FUNCTION plan(state) -> [Action]; an Action carries the exact
# `gh`/`git` argv it maps to. `live` mode gathers the state via real gh/git then executes
# the plan; `--self-test` runs plan() over fixtures with gh AND git STUBBED so no live
# mutation can happen from a test. Fixtures cover the DO-NOTHING cases explicitly and the
# asserts compare exact argv (flip any one expectation and the suite goes red).
#
# USAGE
#   scripts/batch-merge.py --repo owner/repo                 # live run (App token in GH_TOKEN)
#   scripts/batch-merge.py --repo owner/repo --dry-run       # print plan, no mutations
#   scripts/batch-merge.py --repo owner/repo --hygiene-only  # no new omnibus (no App token)
#   scripts/batch-merge.py --self-test                       # hermetic; gh + git stubbed
#
# Exit 0 on success; non-zero only on a real error or a failed self-test.
import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone

PROG = "batch-merge"

# v2 batch-everything: how many eligible PRs are left to the merge queue individually
# before batching. 0 = EVERY eligible armed worker PR joins the omnibus — broken-code
# risk is handled by recursive bisection (DEPTH_CAP below), not by capping the batch.
# (Background, still accurate: the GitHub merge queue's max_entries_to_merge is 8, but an
# omnibus occupies ONE queue entry regardless of how many constituents it carries.)
QUEUE_RESERVE = 0
# An omnibus with a single constituent is pure overhead — a ROOT requires at least two.
# Bisection CHILDREN are exempt: a singleton child is exactly how a culprit is isolated.
MIN_CONSTITUENTS = 2
# Liveness bound: an omnibus that has not MERGED this many hours after creation has no
# route to merge (merge-group failure / dropped arm / checks never reported — all
# invisible on the PR head) and is closed so it cannot suppress batching indefinitely.
# The queue drains a window well inside this bound; the value trades retry opportunity
# (re-arm, §7) against worst-case queue churn from a persistently-failing omnibus.
MAX_OMNIBUS_AGE_HOURS = 4
# Bisection recursion bound: a parent FAILING at this depth with >=2 survivors falls back
# to a v1 close (constituents stay individually armed) instead of splitting further.
# 2^8 = 256 constituents of headroom — far beyond any realistic armed backlog.
DEPTH_CAP = 8
# A CONCLUDED `gate` in any of these states closes an omnibus, but ONLY the strict
# "failure" verdict is a constituent-fault signal (rule 6): it alone may BISECT, and
# it alone may CONVICT a singleton survivor. "cancelled" (a human-cancelled run) and
# "timed_out" (runner starvation) are infra-shaped — the same class as age-expiry —
# and take the v1-close path: never bisection, never a disarmed constituent.
GATE_FAILING_CONCLUSIONS = ("failure", "timed_out", "cancelled")
# Root-creation cooldown (rule 6, infra-flood backstop): while ANY sparq-omnibus PR
# was closed UNMERGED within this many hours, no new ROOT is created. Durable across
# runs (read back from GitHub's closed-PR record — the batcher itself is stateless);
# bisection children are exempt. Constituents stay individually armed throughout, so
# the cooldown costs only batching throughput, never loses work.
ROOT_COOLDOWN_HOURS = 2
# Open-PR listing fetch bound. If the listing SATURATES this limit it may be truncated:
# a live constituent absent from a truncated list must never be classified as
# closed/merged (that census feeds DESTRUCTIVE decisions — quarantine picks "the one
# survivor"), so plan() degrades every bisection decision to a v1 close for that run.
OPEN_PR_LIST_LIMIT = 1000

OMNIBUS_PREFIX = "sparq-omnibus/"
WORKER_BRANCH_RE = re.compile(r"^sparq-agent/")
# Worker heads encode their target issue: sparq-agent/issue-<N>-<run_id>-<attempt>.
WORKER_ISSUE_RE = re.compile(r"^sparq-agent/issue-([1-9][0-9]*)-")
ORCHESTRATOR_APP = "sparq-orchestrator"
# Machine markers embedded in every omnibus PR body; closure + bisection key on them.
# New bodies always write v2 (with a bisection depth); v1 bodies (pre-bisection) still
# parse, with depth=0 implied.
MARKER_V1_RE = re.compile(r"<!--\s*sparq-omnibus:v1\s+constituents=([0-9,]+)\s*-->")
MARKER_V2_RE = re.compile(
    r"<!--\s*sparq-omnibus:v2\s+constituents=([0-9,]+)\s+depth=([0-9]+)\s*-->")
# Leading UTC creation stamp of an omnibus branch name (children carry a suffix).
STAMP_RE = re.compile(r"^([0-9]{8}T[0-9]{6}Z)")

SELF_ID = "> 🤖 SPARQ agent"


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def author_handle(login: str) -> str:
    # gh reports app authors as "app/<name>", "<name>[bot]" (REST-shaped, the form
    # retriage.py's TRUSTED_BOT pins) or bare — normalise all three to "<name>".
    h = (login or "").split("/")[-1].lower()
    return h[: -len("[bot]")] if h.endswith("[bot]") else h


def is_release_plz(pr: dict) -> bool:
    return (
        author_handle(pr.get("author_login", "")) == "github-actions"
        and pr.get("title", "").lower().startswith("chore: release")
    )


def has_excluded_label(pr: dict) -> bool:
    for lbl in pr.get("labels", []):
        low = (lbl or "").lower()
        if low == "needs:user" or low.startswith("trust:"):
            return True
    return False


def armed_by_orchestrator(pr: dict) -> bool:
    """True iff the PR carries an ACTIVE auto-merge request enabled by the orchestrator App."""
    am = pr.get("auto_merge")
    if not isinstance(am, dict):
        return False
    return author_handle(str(am.get("enabled_by", ""))) == ORCHESTRATOR_APP


def is_trusted_omnibus(pr: dict) -> bool:
    """True iff this PR may be CLASSIFIED as an omnibus record at all.

    Three provenance legs, ALL required — the same same-repo + App-identity trust the
    rest of the pipeline applies (retriage.py's TRUSTED_BOT / the enumerator's
    same-repo head rule): (1) the sparq-omnibus/ head prefix, (2) a SAME-REPOSITORY
    head (is_cross_repository must be explicitly False — a fork can name its branch
    anything), (3) orchestrator-App authorship (the only identity that ever creates an
    omnibus, see TOKENS). A fork or non-App PR with a spoofed sparq-omnibus/ prefix
    must NEVER drive any omnibus classification — open (close / bisect / QUARANTINE /
    root-suppression), merged (constituent closure), cooldown, or stale-branch —
    because that would let an outsider draft/disarm innocent constituents or wedge
    root creation."""
    return (pr.get("head_ref", "").startswith(OMNIBUS_PREFIX)
            and pr.get("is_cross_repository") is False
            and author_handle(pr.get("author_login", "")) == ORCHESTRATOR_APP)


def issue_of(pr: dict):
    """The constituent's target issue number, from its worker head ref (None if unparseable)."""
    m = WORKER_ISSUE_RE.match(pr.get("head_ref", ""))
    return int(m.group(1)) if m else None


def crate_of(pr: dict) -> str:
    """Best-effort crate column: first changed crates/<name>/ path, else area:<x> label, else em-dash."""
    for path in pr.get("files", []):
        m = re.match(r"^crates/([^/]+)/", path or "")
        if m:
            return m.group(1)
    for lbl in sorted(pr.get("labels", [])):
        if lbl.lower().startswith("area:"):
            return lbl.split(":", 1)[1]
    return "—"


def eligible_constituents(prs: list) -> list:
    """The armed, reviewed worker PRs the batcher may consider — sorted by number ASCENDING."""
    out = []
    for pr in prs:
        head = pr.get("head_ref", "")
        if not WORKER_BRANCH_RE.match(head):
            continue  # covers dependabot/*, release-plz, and every other branch family
        if pr.get("is_draft"):
            continue
        if is_release_plz(pr) or has_excluded_label(pr):
            continue
        if "review:pass" not in [(lbl or "").lower() for lbl in pr.get("labels", [])]:
            continue
        if not armed_by_orchestrator(pr):
            continue
        out.append(pr)
    return sorted(out, key=lambda p: p["number"])


def parse_marker(body: str):
    """(constituent PR numbers, bisection depth) from an omnibus body marker.

    Parses BOTH marker generations: v2 carries an explicit depth; a v1 marker implies
    depth=0 (a pre-bisection root). ([], 0) when no marker is present."""
    m = MARKER_V2_RE.search(body or "")
    if m:
        return [int(x) for x in m.group(1).split(",") if x], int(m.group(2))
    m = MARKER_V1_RE.search(body or "")
    if m:
        return [int(x) for x in m.group(1).split(",") if x], 0
    return [], 0


def constituents_of_marker(body: str) -> list:
    """Constituent PR numbers recorded in an omnibus body marker ([] when absent)."""
    return parse_marker(body)[0]


def omnibus_age_hours(head_ref: str, now: datetime):
    """Hours since the omnibus branch's creation stamp.

    Branch names are sparq-omnibus/<utcstamp> for a root and
    sparq-omnibus/<utcstamp>-p<parentPR><a|b> for a bisection child — the LEADING stamp
    is the creation time either way. None on an unparseable stamp — the caller treats
    that as EXPIRED (an omnibus branch we cannot date must never become immortal)."""
    rest = head_ref[len(OMNIBUS_PREFIX):] if head_ref.startswith(OMNIBUS_PREFIX) else ""
    m = STAMP_RE.match(rest)
    if not m:
        return None
    created = datetime.strptime(m.group(1), "%Y%m%dT%H%M%SZ").replace(tzinfo=timezone.utc)
    return (now - created).total_seconds() / 3600.0


def gate_conclusion(pr: dict):
    """The FAILING conclusion of the authoritative ci-summary `gate` check ("failure" /
    "timed_out" / "cancelled"), or None when the gate is missing, non-terminal, or green.

    Live for an omnibus because its branch/PR are pushed + created with the App token,
    so head pull_request CI (ci-summary -> `gate`) fires on it. A merge-group failure
    reports on the queue's synthetic ref instead — the age bound covers that mode.
    A missing / in-progress / queued gate is NOT a failure (never act on a non-terminal
    gate — same posture as pr-backlog.py). The caller distinguishes the strict
    "failure" verdict (the ONLY bisect/convict signal) from the infra-shaped
    cancelled / timed_out (v1 close only — see GATE_FAILING_CONCLUSIONS)."""
    for c in pr.get("checks", []):
        if (c.get("name") or "").strip().lower() != "gate":
            continue
        if (c.get("status") or "").lower() == "completed":
            concl = (c.get("conclusion") or "").lower()
            if concl in GATE_FAILING_CONCLUSIONS:
                return concl
    return None


# --------------------------------------------------------------------------------------
# Actions: each carries the exact argv the live runner executes. `tool` is "gh" or "git".
# --------------------------------------------------------------------------------------
@dataclass
class Action:
    kind: str
    tool: str
    argv: list
    note: str = ""
    # A merge step may fail on conflict; the runner then aborts the merge + records a skip.
    may_conflict: bool = False
    constituent: int = 0
    # Creation metadata (push-branch / open-pr): the fewest clean merges worth pushing
    # (MIN_CONSTITUENTS for a root, 1 for a bisection child) and the body's depth/parent
    # the runner re-templates over the clean-merge survivors.
    min_ok: int = MIN_CONSTITUENTS
    depth: int = 0
    parent: int = 0


def omnibus_branch(now: datetime, suffix: str = "") -> str:
    # Bisection children suffix the parent PR number + half (-p<N>a / -p<N>b) so two
    # siblings — and the children of two parents dying in one run — never collide.
    return OMNIBUS_PREFIX + now.strftime("%Y%m%dT%H%M%SZ") + suffix


def omnibus_title(n: int, depth: int = 0) -> str:
    """Telemetry: the title always carries the constituent count (depth lives in the body)."""
    base = f"omnibus: {n} reviewed worker PRs"
    return base if depth == 0 else f"{base} (bisect depth {depth})"


def omnibus_body(constituents: list, skipped: list, depth: int = 0, parent: int = 0) -> str:
    """The omnibus PR body: self-ID, v2 machine marker (constituents + bisection depth),
    constituent table, parent PR reference (children only), Closes lines."""
    nums = ",".join(str(p["number"]) for p in constituents)
    if depth == 0:
        intro = (
            f"{SELF_ID} — omnibus batcher (`scripts/batch-merge.py`). This PR batches ALL "
            f"reviewed, orchestrator-armed worker PRs into one merge-queue entry. Every "
            f"constituent already carries `review:pass`; its issue closes below. A "
            f"conflicting constituent was skipped and stays individually armed.")
    else:
        intro = (
            f"{SELF_ID} — omnibus batcher (`scripts/batch-merge.py`). BISECTION child "
            f"(depth {depth}) of failed omnibus PR #{parent}: one half of its surviving "
            f"constituents, landing independently — a failing half keeps halving until "
            f"the culprit is isolated. Every constituent stays individually armed.")
    lines = [
        intro,
        "",
        f"<!-- sparq-omnibus:v2 constituents={nums} depth={depth} -->",
        "",
        "| PR | Issue | Crate |",
        "|---|---|---|",
    ]
    for p in constituents:
        issue = issue_of(p)
        issue_cell = f"#{issue}" if issue else "—"
        lines.append(f"| #{p['number']} | {issue_cell} | {crate_of(p)} |")
    if skipped:
        lines += ["", "Skipped (merge conflict — still individually armed): "
                  + ", ".join(f"#{n}" for n in skipped)]
    lines.append("")
    for p in constituents:
        issue = issue_of(p)
        if issue:
            lines.append(f"Closes #{issue}")
    lines += ["", "Constituent PRs: " + ", ".join(f"#{p['number']}" for p in constituents)]
    return "\n".join(lines)


def close_constituent_comment(omnibus_num: int) -> str:
    return (
        f"{SELF_ID} — this PR's changes landed on `main` via omnibus PR #{omnibus_num} "
        f"(its branch now adds nothing vs `main`). Closing; the target issue closes via "
        f"the omnibus's `Closes #` references."
    )


def close_failed_omnibus_comment(reason: str) -> str:
    return (
        f"{SELF_ID} — closing this omnibus: {reason}. Every constituent PR remains "
        f"individually armed, so nothing is lost — the queue drains them as before.\n\n"
        f"<!-- sparq-omnibus-failure:v1 bisection=v2 -->"
    )


def bisect_close_comment(reason: str, child_a: str, child_b: str) -> str:
    """Parent-close comment for a bisected omnibus. MUST name both child branches — the
    self-test mutation check pins this so the wiring cannot silently be dropped."""
    return (
        f"{SELF_ID} — closing this omnibus: {reason}. BISECTING: its surviving "
        f"constituents were split in half into two child omnibuses — `{child_a}` and "
        f"`{child_b}` — each armed to land independently; a failing half keeps halving "
        f"until the culprit is isolated. Every constituent PR remains individually armed "
        f"throughout.\n\n"
        f"<!-- sparq-omnibus-failure:v1 bisection=v2 -->"
    )


def retest_close_comment(reason: str, child: str, survivor: int, blockers: list) -> str:
    """Parent-close comment for the AMBIGUOUS-CAUSALITY singleton re-test: conviction
    on the parent's verdict is impossible while a sibling's code is still in the
    failed head, so the sole eligible survivor gets a fresh singleton child instead."""
    sibs = ", ".join(f"#{n}" for n in sorted(blockers))
    return (
        f"{SELF_ID} — closing this omnibus: {reason}. NOT convicting sole still-"
        f"eligible constituent #{survivor}: sibling constituent(s) {sibs} are no "
        f"longer eligible (drafted / disarmed / relabeled / closed unmerged) but "
        f"their code is STILL in this failed head, so the failure cannot be pinned "
        f"on #{survivor}. RE-TESTING instead: fresh singleton child omnibus "
        f"`{child}` carries ONLY #{survivor} — a failure there is sound evidence "
        f"against it alone. Every constituent PR remains individually armed "
        f"throughout.\n\n"
        f"<!-- sparq-omnibus-failure:v1 bisection=v2 -->"
    )


def quarantine_comment(parent_num: int, reason: str) -> str:
    # The handoff contract is the registry's `stranded` posture (draft + unarmed +
    # review:pass -> loud needs:user escalation), NOT its ci-fix admission: ci-fix
    # keys on a CONCLUDED gate failure on the culprit's OWN current head, but head CI
    # does not re-run when main advances and the failure evidence lived on the
    # (now-closed) child-omnibus head — a quarantined culprit's own gate is green, so
    # a gate-keyed admission can never see it. The trailing machine marker is the
    # durable breadcrumb any enumerator can key on.
    return (
        f"{SELF_ID} — bisection isolated this PR as the sole surviving constituent of "
        f"omnibus PR #{parent_num}, which failed ({reason}). This PR is the presumptive "
        f"CULPRIT: it is converted to DRAFT and its auto-merge arm is disabled; the PR "
        f"itself stays open. Draft + unarmed + review:pass is the registry's `stranded` "
        f"posture, which escalates to a human (needs:user) — this PR's own head gate is "
        f"still green (head CI does not re-run when `main` advances; the failing "
        f"evidence lived on the now-closed child omnibus), so the stranded escalation "
        f"is the guaranteed-closure path.\n\n"
        f"<!-- sparq-omnibus-culprit:v1 parent={parent_num} reason=gate-failure -->"
    )


def creation_actions(repo: str, branch: str, constituents: list, depth: int = 0,
                     parent: int = 0, min_ok: int = MIN_CONSTITUENTS) -> list:
    """The shared omnibus-creation machinery (root AND bisection children): fetch, fresh
    branch off origin/main, per-constituent no-ff merges (conflict-skippable), push, PR
    with the v2 marker, arm. The runner re-templates the title/body over the clean-merge
    survivors and aborts the push when fewer than min_ok merged cleanly."""
    acts = [
        Action("fetch", "git",
               ["fetch", "origin", "main"] + [p["head_ref"] for p in constituents]),
        Action("create-branch", "git",
               ["checkout", "-B", branch, "origin/main"], note=branch),
    ]
    for p in constituents:
        acts.append(Action(
            "merge", "git",
            ["merge", "--no-ff", "--no-edit", "-m",
             f"omnibus: merge PR #{p['number']} ({p['head_ref']})",
             f"origin/{p['head_ref']}"],
            may_conflict=True, constituent=p["number"],
            note=f"constituent #{p['number']}"))
    acts.append(Action("push-branch", "git", ["push", "origin", branch], note=branch,
                       min_ok=min_ok))
    acts.append(Action(
        "open-pr", "gh",
        ["pr", "create", "--repo", repo, "--base", "main", "--head", branch,
         "--title", omnibus_title(len(constituents), depth),
         "--body", omnibus_body(constituents, [], depth=depth, parent=parent)],
        note=f"{len(constituents)} constituents (depth {depth})",
        min_ok=min_ok, depth=depth, parent=parent))
    # Arm: strategy is chosen by the merge queue, so no method flag.
    acts.append(Action("arm", "gh", ["pr", "merge", branch, "--repo", repo, "--auto"],
                       note=branch))
    return acts


# --------------------------------------------------------------------------------------
# The pure policy. `state` is a plain-dict snapshot:
#   repo: "owner/name"
#   now:  aware UTC datetime
#   open_prs: [{number, head_ref, title, author_login, labels[], is_draft,
#               auto_merge{enabled_by}|None, files[], body, checks[], mergeable,
#               is_cross_repository}]   (is_cross_repository: fork-head marker — an
#                                        omnibus record additionally requires it to
#                                        be explicitly False, see is_trusted_omnibus)
#   merged_omnibus: [{number, body}]                    (merged sparq-omnibus/* PRs)
#   empty_vs_main: {pr_number: bool}                    (branch adds nothing vs main)
#   remote_omnibus_branches: ["sparq-omnibus/..."]      (live remote refs)
#   batch_enabled: bool                                 (False = --hygiene-only: never
#                                                        create a new omnibus)
#   open_list_truncated: bool                           (open-PR listing saturated its
#                                                        fetch limit: survivor censuses
#                                                        are unreliable — degrade every
#                                                        bisection decision to v1 close,
#                                                        and suppress stale-branch
#                                                        deletion + new-root creation)
#   recent_omnibus_closures: [datetime]                 (closedAt of sparq-omnibus PRs
#                                                        closed UNMERGED — the durable
#                                                        ROOT_COOLDOWN_HOURS record)
# --------------------------------------------------------------------------------------
def plan(state: dict) -> list:
    repo = state["repo"]
    actions: list = []
    open_prs = state.get("open_prs", [])
    open_by_num = {p["number"]: p for p in open_prs}

    # ---- 5. constituent closure for MERGED omnibus PRs (first: frees the backlog) ----
    closing_nums = set()  # closed below — never re-batched into a new root this run
    for om in state.get("merged_omnibus", []):
        for num in constituents_of_marker(om.get("body", "")):
            pr = open_by_num.get(num)
            if pr is None:
                continue  # already closed/merged — idempotent
            if not state.get("empty_vs_main", {}).get(num, False):
                continue  # has content main lacks (e.g. post-omnibus commits) — leave armed
            closing_nums.add(num)
            actions.append(Action(
                "close-constituent", "gh",
                ["pr", "close", str(num), "--repo", repo,
                 "--comment", close_constituent_comment(om["number"])],
                note=f"constituent #{num} contained in merged omnibus #{om['number']}"))

    # ---- 6+7. failure path v2 (recursive bisection) + re-arm liveness ----
    # PROVENANCE GATE (impersonation guard): only a trusted omnibus — same-repo head
    # + orchestrator-App author — is CLASSIFIED as one. A spoofed sparq-omnibus/
    # prefix from a fork or non-App author is ignored entirely: it never closes,
    # never bisects, never quarantines a constituent, and never suppresses a new
    # root (an outsider must not be able to wedge the batcher by opening a fork PR).
    prefixed_open = [p for p in open_prs
                     if p.get("head_ref", "").startswith(OMNIBUS_PREFIX)]
    open_omnibus = [p for p in prefixed_open if is_trusted_omnibus(p)]
    for p in prefixed_open:
        if not is_trusted_omnibus(p):
            log(f"WARNING: PR #{p['number']} spoofs the omnibus branch prefix "
                f"({p.get('head_ref', '')}) but is not a same-repository "
                f"orchestrator-App PR — never classified as an omnibus (ignored)")
    dying_branches = set()
    children_spawned = False   # freshly planned children block a new ROOT this run
    suppress_root = False      # depth-cap fallback: no same-run rebuild of a doomed root
    infra_closed = False       # an infra-shaped close this run: no same-run root rebuild
    quarantined = set()        # disarmed culprits — never re-batched into a new root
    can_create = state.get("batch_enabled", True)
    truncated = state.get("open_list_truncated", False)
    empty_vs_main = state.get("empty_vs_main", {})

    def _survivor(num: int) -> bool:
        # RE-BATCH eligibility (NOT causality — see _causal below): a marker
        # constituent survives into a child iff it is still an OPEN PR, still
        # armed-eligible (the same per-PR test as root eligibility), and NOT already
        # empty vs main. Unknown emptiness counts as content: an accidentally-empty
        # child merges harmlessly, whereas dropping live work would not.
        pr = open_by_num.get(num)
        return (pr is not None and bool(eligible_constituents([pr]))
                and not empty_vs_main.get(num, False))

    for om in open_omnibus:
        age = omnibus_age_hours(om.get("head_ref", ""), state["now"])
        expired = age is None or age > MAX_OMNIBUS_AGE_HOURS
        bisectable = False
        # Only a strict, CONCLUDED head-gate FAILURE on a DEFINITIVELY MERGEABLE
        # head is a constituent-fault signal: it alone BISECTS, and it alone may
        # CONVICT a singleton survivor (quarantine). Everything else — a conflict,
        # an UNRESOLVED (UNKNOWN/unreported) mergeability, a cancelled / timed_out
        # gate, age-expiry — v1-closes (or waits) without implicating a constituent.
        culpable = False
        gate_c = gate_conclusion(om)
        mergeable = (om.get("mergeable") or "").upper()
        if mergeable == "CONFLICTING":
            # Checked FIRST — BEFORE any gate verdict. Main moved under the omnibus,
            # so even a concluded gate FAILURE on this head was evidence against a
            # stale base, not against any constituent: v1 close only, no bisection,
            # no quarantine — close-and-recreate rebuilds a fresh root off current
            # main. Conflicts must NEVER implicate constituents.
            reason = "it conflicts with `main`"
        elif gate_c == "failure" and mergeable == "MERGEABLE":
            reason = f"its `gate` check concluded in {gate_c}"
            bisectable = True
            culpable = True
        elif gate_c == "failure":
            # INDETERMINATE mergeability (UNKNOWN / null / unreported): when main
            # advances, GitHub reports UNKNOWN before resolving — possibly to
            # CONFLICTING, and a conflict must NEVER implicate constituents, so a
            # gate failure without a DEFINITIVE MERGEABLE verdict must not either.
            # WAIT (no action this run; the still-open omnibus keeps suppressing a
            # new root); only the age-expiry NON-culpable close may act on a
            # persistently-unresolved omnibus.
            if expired:
                reason = (f"it did not merge within {MAX_OMNIBUS_AGE_HOURS}h of "
                          f"creation and its mergeability never became definitive "
                          f"(a gate failure on an unresolved base is not evidence "
                          f"against any constituent)")
                infra_closed = True
            else:
                log(f"omnibus #{om['number']}: gate concluded in failure but "
                    f"mergeability is {om.get('mergeable') or 'unreported'!r} — "
                    f"NOT definitive (main may have moved under it); waiting, no "
                    f"constituent may be implicated on indeterminate mergeability")
                continue
        elif gate_c is not None:
            # cancelled / timed_out: infra-shaped (a human-cancelled run, runner
            # starvation), NOT a code verdict — v1 close only, never bisection
            # (splitting on an outage doubles the node count every pass), and the
            # infra flag below suppresses a same-run root rebuild.
            reason = (f"its `gate` check concluded in {gate_c} (infra-shaped — "
                      f"not a verdict against any constituent)")
            infra_closed = True
        elif expired:
            # Liveness backstop: covers every head-invisible failure (merge-group gate
            # failure, dropped arm, checks never reported, unparseable stamp). Also
            # infra-shaped: v1 close only, never bisection, no same-run rebuild.
            reason = (f"it did not merge within {MAX_OMNIBUS_AGE_HOURS}h of creation "
                      f"(no route to merge — e.g. a merge-group failure, a dropped "
                      f"auto-merge arm, or checks that never reported)")
            infra_closed = True
        else:
            # §7 re-arm: GitHub drops the auto-merge arm when a merge group fails; a
            # young, mergeable, UNARMED omnibus gets its arm restored idempotently so a
            # transient queue failure is retried instead of wedging until the age bound.
            if (om.get("mergeable") or "").upper() == "MERGEABLE" and not om.get("auto_merge"):
                actions.append(Action(
                    "arm", "gh",
                    ["pr", "merge", str(om["number"]), "--repo", repo, "--auto"],
                    note=f"re-arm open omnibus #{om['number']} (arm dropped)"))
            continue
        dying_branches.add(om["head_ref"])
        nums, depth = parse_marker(om.get("body", ""))
        # CAUSALITY (distinct from eligibility): the set that can be blamed for this
        # failed head is the marker constituents whose CODE IS IN IT and is not
        # cleanly excluded. The ONLY clean exclusion is positive evidence the code
        # already landed on main (empty_vs_main True — it merged through a
        # gate-green path). A constituent that merely became INELIGIBLE — drafted,
        # disarmed, relabeled, closed unmerged, or simply absent from the open
        # listing without an emptiness proof — still has its code in the failed
        # head and may be the actual culprit.
        causal = [n for n in nums if not empty_vs_main.get(n, False)] if bisectable else []
        survivors = [n for n in causal if _survivor(n)]
        if survivors and truncated:
            # A live constituent absent from a TRUNCATED listing would be miscounted
            # as dead — and this census feeds destructive decisions (which halves to
            # build, which lone PR to convict). Degrade to the always-safe v1 close.
            log(f"open-PR listing saturated its fetch limit ({OPEN_PR_LIST_LIMIT}): "
                f"survivor census for omnibus #{om['number']} is unreliable — v1 close "
                f"instead of bisection/quarantine (constituents stay individually "
                f"armed)")
            survivors = []
        if len(survivors) >= 2 and depth >= DEPTH_CAP:
            log(f"DEPTH CAP: omnibus #{om['number']} failed at depth {depth} >= "
                f"{DEPTH_CAP} with {len(survivors)} survivors — v1 fallback (close; "
                f"constituents stay individually armed; no same-run rebuild)")
            survivors = []
            suppress_root = True
        if len(survivors) >= 2 and not can_create:
            log(f"hygiene-only: omnibus #{om['number']} would bisect into "
                f"{len(survivors)} survivors, but child creation needs the App token — "
                f"v1 close instead (constituents stay individually armed)")
            survivors = []
        if len(survivors) >= 2:
            mid = (len(survivors) + 1) // 2
            half_a, half_b = survivors[:mid], survivors[mid:]
            branch_a = omnibus_branch(state["now"], f"-p{om['number']}a")
            branch_b = omnibus_branch(state["now"], f"-p{om['number']}b")
            log(f"BISECT: omnibus #{om['number']} (depth {depth}, "
                f"{len(nums)} marker constituents, {len(survivors)} survivors) -> "
                f"children {branch_a} ({len(half_a)}) + {branch_b} ({len(half_b)}) "
                f"at depth {depth + 1}")
            actions.append(Action(
                "close-omnibus", "gh",
                ["pr", "close", str(om["number"]), "--repo", repo,
                 "--comment", bisect_close_comment(reason, branch_a, branch_b)],
                note=f"omnibus #{om['number']}: {reason} — bisecting"))
            actions.append(Action(
                "delete-branch", "git", ["push", "origin", "--delete", om["head_ref"]],
                note=f"branch of closed omnibus #{om['number']}"))
            for br, half in ((branch_a, half_a), (branch_b, half_b)):
                actions += creation_actions(
                    repo, br, [open_by_num[n] for n in half],
                    depth=depth + 1, parent=om["number"], min_ok=1)
            children_spawned = True
            continue
        if len(survivors) == 1 and not culpable:
            # Belt-and-braces conviction gate. Unreachable by construction today
            # (survivors are only computed when bisectable, and bisectable implies a
            # strict gate FAILURE) — kept so that if bisectability is ever widened
            # again, a sole survivor still can only be quarantined by a concluded
            # head-gate FAILURE verdict, never by an infra-shaped mode.
            log(f"sole survivor PR #{survivors[0]} of omnibus #{om['number']} "
                f"(depth {depth}) NOT quarantined: {reason} is not a concluded "
                f"head-gate FAILURE verdict — v1 close, survivor stays armed")
            survivors = []
        if len(survivors) == 1 and len(causal) > 1:
            # ELIGIBILITY IS NOT CAUSALITY: another marker constituent's code is
            # still in the failed head (it became ineligible / closed WITHOUT
            # landing), so the parent's gate failure cannot convict the lone
            # still-eligible survivor — the unremoved sibling may be the culprit.
            # Get FRESH evidence instead: a singleton RE-TEST child carrying only
            # the survivor, off current main — if THAT fails, its causal set is
            # exactly {survivor} and conviction is sound. When no child can be
            # created (hygiene-only / depth cap), take the non-culpable v1 close:
            # never quarantine on ambiguous causality.
            blockers = sorted(n for n in causal if n != survivors[0])
            if not can_create or depth >= DEPTH_CAP:
                log(f"AMBIGUOUS CAUSALITY: omnibus #{om['number']} (depth {depth}) "
                    f"failed with sole eligible survivor PR #{survivors[0]}, but "
                    f"code-bearing ineligible sibling(s) {blockers} are still in "
                    f"the failed head and no re-test child can be created "
                    f"(hygiene-only or depth cap) — v1 close, survivor stays armed")
                if depth >= DEPTH_CAP:
                    suppress_root = True  # same damper as the >=2 depth-cap fallback
                survivors = []
            else:
                child = omnibus_branch(state["now"], f"-p{om['number']}a")
                log(f"AMBIGUOUS CAUSALITY: omnibus #{om['number']} (depth {depth}) "
                    f"failed with sole eligible survivor PR #{survivors[0]}, but "
                    f"code-bearing ineligible sibling(s) {blockers} are still in "
                    f"the failed head — singleton RE-TEST child {child} instead of "
                    f"conviction (never quarantine on ambiguous causality)")
                actions.append(Action(
                    "close-omnibus", "gh",
                    ["pr", "close", str(om["number"]), "--repo", repo,
                     "--comment", retest_close_comment(reason, child, survivors[0],
                                                       blockers)],
                    note=f"omnibus #{om['number']}: {reason} — ambiguous causality, "
                         f"re-testing #{survivors[0]}"))
                actions.append(Action(
                    "delete-branch", "git",
                    ["push", "origin", "--delete", om["head_ref"]],
                    note=f"branch of closed omnibus #{om['number']}"))
                actions += creation_actions(
                    repo, child, [open_by_num[survivors[0]]],
                    depth=depth + 1, parent=om["number"], min_ok=1)
                children_spawned = True
                continue
        if len(survivors) == 1:
            culprit = survivors[0]
            quarantined.add(culprit)
            log(f"QUARANTINE: omnibus #{om['number']} (depth {depth}) failed its head "
                f"gate with sole survivor PR #{culprit} — comment + draft + disarm "
                f"(NOT closing); the registry's stranded escalation owns it now")
            # Ordering is load-bearing (crash safety, 6b): comment FIRST — a crash or
            # raised gh failure after it leaves the culprit ARMED, so the next run
            # re-isolates it and the repeat is idempotent. Then DRAFT: GitHub drops
            # the auto-merge arm on draft conversion, so the terminal stranded
            # posture (draft + unarmed) is reached even if the explicit disarm below
            # never runs. The belt-and-braces disarm goes LAST (non-fatal in
            # execute(): the arm is usually already gone). No cut point can leave a
            # disarmed-but-unmarked PR.
            actions.append(Action(
                "comment", "gh",
                ["pr", "comment", str(culprit), "--repo", repo,
                 "--body", quarantine_comment(om["number"], reason)],
                note=f"culprit #{culprit} quarantine notice"))
            actions.append(Action(
                "draft", "gh",
                ["pr", "ready", str(culprit), "--repo", repo, "--undo"],
                note=f"culprit #{culprit} -> draft (registry stranded posture)"))
            actions.append(Action(
                "disarm", "gh",
                ["pr", "merge", str(culprit), "--repo", repo, "--disable-auto"],
                note=f"culprit #{culprit} isolated by failed omnibus #{om['number']}"))
            # No labels are added here: draft + unarmed IS the registry's stranded
            # posture (its enumerator keys on the draft state, not on anything the
            # batcher could apply), and the comment's machine marker is the durable
            # breadcrumb. Fall through: the parent closes exactly as in v1.
        actions.append(Action(
            "close-omnibus", "gh",
            ["pr", "close", str(om["number"]), "--repo", repo,
             "--comment", close_failed_omnibus_comment(reason)],
            note=f"omnibus #{om['number']}: {reason}"))
        actions.append(Action(
            "delete-branch", "git", ["push", "origin", "--delete", om["head_ref"]],
            note=f"branch of closed omnibus #{om['number']}"))

    # ---- 8. stale hygiene: sparq-omnibus/* branches with no open PR ----
    if truncated:
        # A truncated listing can OMIT a live omnibus PR: its branch would look
        # stale here and be DELETED out from under an open PR — and the one-tree
        # check below would then also open a duplicate root. Fail-safe: nothing
        # destructive on incomplete data; a later run with a complete listing
        # does the hygiene.
        log(f"open-PR listing saturated its fetch limit ({OPEN_PR_LIST_LIMIT}): "
            f"a live omnibus may be missing from it — skipping stale-branch "
            f"deletion this run (fail-safe)")
    else:
        # Branch protection deliberately covers UNTRUSTED prefixed PRs too: a spoofed
        # PR is never CLASSIFIED as an omnibus (above), but deleting a same-repo
        # branch out from under ANY open PR is destructive — leave it to a human.
        live_branches = {p["head_ref"] for p in prefixed_open}
        for br in sorted(state.get("remote_omnibus_branches", [])):
            if br in live_branches or br in dying_branches:
                continue
            actions.append(Action("delete-branch", "git",
                                  ["push", "origin", "--delete", br],
                                  note="stale omnibus branch (no open PR)"))

    # ---- 1-4. root batching (v2: batch EVERYTHING eligible) ----
    if not can_create:
        # --hygiene-only (no App token in GH_TOKEN): a GITHUB_TOKEN-created omnibus can
        # NEVER enter the merge queue (events suppressed -> required `gate` never
        # reports -> no queue admission), so creating one would only wedge. The
        # closure / failure / re-arm / stale legs above still ran.
        log("hygiene-only mode — skipping omnibus creation (no App token)")
        return actions
    if truncated:
        # The one-open-tree check below reads the SAME truncated listing: a live
        # omnibus omitted from it would not suppress a new root, breaking the
        # one-tree guarantee. Fail-safe: no new root on incomplete data.
        log(f"open-PR listing saturated its fetch limit ({OPEN_PR_LIST_LIMIT}): "
            f"a live omnibus may be invisible, so the one-tree guarantee cannot be "
            f"verified — not opening a new root this run (fail-safe)")
        return actions
    eligible = eligible_constituents(open_prs)
    # QUEUE_RESERVE (0) eligible PRs stay individually queued; the rest — everything —
    # batch. PRs being closed this run (landed via a merged omnibus) or quarantined this
    # run (isolated culprits) are never re-batched.
    batch = [p for p in eligible[QUEUE_RESERVE:]
             if p["number"] not in closing_nums and p["number"] not in quarantined]
    if len(batch) < MIN_CONSTITUENTS:
        log(f"eligible armed worker PRs to batch: {len(batch)} < {MIN_CONSTITUENTS} — "
            f"the queue handles them individually (no batch)")
        return actions
    # One omnibus TREE in flight at a time: never stack a new ROOT while any omnibus is
    # open or freshly spawned (the closed-above ones no longer count; bisection children
    # are exempt from this rule at creation — siblings must co-exist — but once open
    # they block a new root like any other omnibus).
    still_open = [p["number"] for p in open_omnibus if p["head_ref"] not in dying_branches]
    if still_open:
        log(f"omnibus PR(s) {still_open} still open — not opening a new root")
        return actions
    if children_spawned or suppress_root:
        log("bisection children spawned / depth-cap fallback this run — no new root")
        return actions
    if infra_closed:
        # An omnibus died this run for an infra-shaped reason (cancelled / timed_out
        # gate, age-expiry). Rebuilding a root immediately would retry the exact
        # same environment that just failed — under a sustained outage that is the
        # close-and-rebuild flood. The constituents stay individually armed; a new
        # root waits out the cooldown below.
        log(f"an omnibus closed for an infra-shaped reason this run — no same-run "
            f"root rebuild (a new root waits out ROOT_COOLDOWN_HOURS="
            f"{ROOT_COOLDOWN_HOURS}h)")
        return actions
    recent = [t for t in state.get("recent_omnibus_closures", [])
              if t is not None
              and (state["now"] - t).total_seconds() / 3600.0 < ROOT_COOLDOWN_HOURS]
    if recent:
        # Durable cross-run damping (the batcher is stateless — this reads GitHub's
        # closed-PR record): a recently failure-closed omnibus means the environment
        # was churning; do not re-root until the cooldown elapses. Bisection
        # children were planned above and are deliberately exempt.
        log(f"root-creation cooldown: {len(recent)} omnibus failure-close(s) within "
            f"the last {ROOT_COOLDOWN_HOURS}h — not opening a new root (damps the "
            f"close-and-rebuild flood; constituents stay individually armed)")
        return actions

    branch = omnibus_branch(state["now"])
    # Telemetry (no-silent-caps): every creation logs its batch size.
    log(f"ROOT omnibus {branch}: batching ALL {len(batch)} eligible constituents "
        f"(batch-everything; no window slice)")
    actions += creation_actions(repo, branch, batch, depth=0, parent=0,
                                min_ok=MIN_CONSTITUENTS)
    return actions


# --------------------------------------------------------------------------------------
# Live runners. All mutation flows through run_gh/run_git so --self-test can stub both.
# --------------------------------------------------------------------------------------
def run_gh(argv: list, capture: bool = True) -> str:
    res = subprocess.run(["gh"] + argv, check=True,
                         capture_output=capture, text=True)
    return res.stdout if capture else ""


def run_git(argv: list, check: bool = True) -> int:
    return subprocess.run(["git"] + argv, check=check).returncode


def run_git_out(argv: list, check: bool = True) -> str:
    """Capture-output git (merge-tree / rev-parse / ls-remote) — a separate seam from
    run_git so --self-test can stub EVERY subprocess gather_state touches."""
    return subprocess.run(["git"] + argv, check=check,
                          capture_output=True, text=True).stdout


def gather_state(repo: str, now: datetime, batch_enabled: bool = True) -> dict:
    """Snapshot the live GitHub/git state the pure plan() consumes.

    Deliberately TWO-PHASE: one LIGHT list over all open PRs (a single query carrying
    body+statusCheckRollup for hundreds of PRs overloads the GraphQL stream), then
    targeted per-PR views only where the policy needs the heavy fields (open omnibus PRs
    and the overflow candidates). A listing that SATURATES OPEN_PR_LIST_LIMIT may be
    truncated — constituent liveness would then be unreliable — so the snapshot carries
    open_list_truncated and plan() degrades every bisection decision to a v1 close AND
    suppresses stale-branch deletion + new-root creation (fail-safe)."""
    fields = ("number,headRefName,title,author,labels,isDraft,autoMergeRequest,"
              "isCrossRepository")
    raw = json.loads(run_gh(["pr", "list", "--repo", repo, "--state", "open",
                             "--limit", str(OPEN_PR_LIST_LIMIT), "--json", fields]))
    open_list_truncated = len(raw) >= OPEN_PR_LIST_LIMIT
    if open_list_truncated:
        log(f"WARNING: open-PR listing returned {len(raw)} >= its --limit "
            f"{OPEN_PR_LIST_LIMIT} — possibly truncated; survivor censuses are "
            f"unreliable, so this run degrades bisection to v1 closes and suppresses "
            f"stale-branch deletion + new-root creation")
    open_prs = []
    for pr in raw:
        am = pr.get("autoMergeRequest")
        open_prs.append({
            "number": pr["number"],
            "head_ref": pr.get("headRefName", ""),
            "title": pr.get("title", ""),
            "author_login": (pr.get("author") or {}).get("login", ""),
            "labels": [lbl["name"] for lbl in pr.get("labels", [])],
            "is_draft": bool(pr.get("isDraft")),
            "auto_merge": ({"enabled_by": ((am.get("enabledBy") or {}).get("login", ""))}
                           if isinstance(am, dict) else None),
            "is_cross_repository": bool(pr.get("isCrossRepository")),
            "files": [],  # filled below only for overflow candidates (keeps API calls low)
            "body": "",
            "mergeable": "",   # filled below for TRUSTED omnibus PRs only
            "checks": [],      # filled below for TRUSTED omnibus PRs only
        })
    for p in open_prs:
        # Heavy fields only for TRUSTED omnibus PRs: a spoofed prefix (fork /
        # non-App author) is never classified as an omnibus, so its body/marker
        # must never feed closure, bisection, or the emptiness census.
        if not is_trusted_omnibus(p):
            continue
        try:
            heavy = json.loads(run_gh(["pr", "view", str(p["number"]), "--repo", repo,
                                       "--json", "body,mergeable,statusCheckRollup"]))
        except subprocess.CalledProcessError:
            continue  # unknown never acts: leave mergeable/checks empty for this run
        p["body"] = heavy.get("body", "")
        p["mergeable"] = heavy.get("mergeable", "")
        p["checks"] = [{"name": c.get("name", ""), "status": c.get("status", ""),
                        "conclusion": c.get("conclusion", "")}
                       for c in (heavy.get("statusCheckRollup") or []) if isinstance(c, dict)]
    eligible = eligible_constituents(open_prs)
    for p in eligible[QUEUE_RESERVE:]:
        try:
            files = json.loads(run_gh(["pr", "view", str(p["number"]), "--repo", repo,
                                       "--json", "files"]))
            p["files"] = [f.get("path", "") for f in files.get("files", [])]
        except subprocess.CalledProcessError:
            p["files"] = []  # crate column degrades to label/em-dash — non-fatal

    def _rec(raw_pr: dict) -> dict:
        # Minimal record shape for the is_trusted_omnibus provenance check.
        return {"head_ref": raw_pr.get("headRefName", ""),
                "author_login": ((raw_pr.get("author") or {}).get("login", "")),
                "is_cross_repository": bool(raw_pr.get("isCrossRepository"))}

    # MERGED omnibus records (rule 5 closure) carry the same provenance gate: a
    # spoofed sparq-omnibus/ PR that somehow merged must never close constituents.
    merged = json.loads(run_gh(["pr", "list", "--repo", repo, "--state", "merged",
                                "--limit", "50", "--search", "head:sparq-omnibus/",
                                "--json",
                                "number,headRefName,body,author,isCrossRepository"]))
    merged_omnibus = [{"number": m["number"], "body": m.get("body", "")}
                      for m in merged if is_trusted_omnibus(_rec(m))]

    # ROOT_COOLDOWN_HOURS record (rule 6): omnibus PRs closed UNMERGED recently.
    # GitHub's closed-PR list is the stateless batcher's only durable cross-run
    # memory; "closed" includes merged PRs, so filter on a null mergedAt. Same
    # provenance gate: an outsider closing spoofed fork PRs must not be able to
    # suppress root creation indefinitely.
    closed = json.loads(run_gh(["pr", "list", "--repo", repo, "--state", "closed",
                                "--limit", "30", "--search", "head:sparq-omnibus/",
                                "--json", "number,headRefName,closedAt,mergedAt,"
                                "author,isCrossRepository"]))
    recent_omnibus_closures = []
    for c in closed:
        if not is_trusted_omnibus(_rec(c)) or c.get("mergedAt"):
            continue
        try:
            recent_omnibus_closures.append(
                datetime.fromisoformat((c.get("closedAt") or "").replace("Z", "+00:00")))
        except ValueError:
            continue  # undatable close: cannot feed a time-window decision

    # Emptiness test: merging the constituent branch into main yields main's own tree
    # <=> the branch adds nothing. (A plain two/three-dot diff is wrong here: after the
    # omnibus SQUASHES onto main the constituent's merge-base is stale, so its three-dot
    # diff stays non-empty forever even though main contains every change.) Computed for
    # the constituents of MERGED omnibus PRs (closure, rule 5) AND of OPEN omnibus PRs
    # (the bisection survivor filter, rule 6, must exclude already-landed constituents).
    marker_bodies = ([om["body"] for om in merged_omnibus]
                     + [p["body"] for p in open_prs if is_trusted_omnibus(p)])
    run_git(["fetch", "origin", "main"])
    empty_vs_main: dict = {}
    open_by_num = {p["number"]: p for p in open_prs}
    for body in marker_bodies:
        for num in constituents_of_marker(body):
            pr = open_by_num.get(num)
            if pr is None or num in empty_vs_main:
                continue
            branch = pr["head_ref"]
            if run_git(["fetch", "origin", branch], check=False) != 0:
                empty_vs_main[num] = False  # unfetchable head: unknown never closes
                continue
            try:
                merged_tree = run_git_out(
                    ["merge-tree", "--write-tree", "origin/main", f"origin/{branch}"],
                    check=False).strip().splitlines()
                main_tree = run_git_out(["rev-parse", "origin/main^{tree}"]).strip()
                empty_vs_main[num] = bool(merged_tree) and merged_tree[0] == main_tree
            except (subprocess.CalledProcessError, IndexError):
                empty_vs_main[num] = False

    ls = run_git_out(["ls-remote", "origin", f"refs/heads/{OMNIBUS_PREFIX}*"])
    remote_omnibus_branches = [line.split("refs/heads/", 1)[1]
                               for line in ls.strip().splitlines() if "refs/heads/" in line]

    return {"repo": repo, "now": now, "open_prs": open_prs,
            "merged_omnibus": merged_omnibus, "empty_vs_main": empty_vs_main,
            "remote_omnibus_branches": remote_omnibus_branches,
            "batch_enabled": batch_enabled,
            "open_list_truncated": open_list_truncated,
            "recent_omnibus_closures": recent_omnibus_closures}


def execute(actions: list, state: dict, dry_run: bool) -> int:
    """Run the plan. Creation is SEGMENTED: each create-branch starts a new omnibus
    segment (the root, or one bisection child); within a segment the merge sequence is
    stateful — conflicting constituents are skipped and the tail (push / open-pr / arm)
    is re-templated over that segment's survivors. A segment with fewer than its min_ok
    clean merges is aborted (local branch deleted, nothing pushed) WITHOUT affecting the
    segments after it — one all-conflict child must not sink its sibling."""
    open_by_num = {p["number"]: p for p in state.get("open_prs", [])}
    seg_branch = None          # branch of the creation segment being executed
    seg_survivors, seg_skipped = [], []
    seg_aborted = False
    for act in actions:
        if act.kind == "create-branch":
            seg_branch, seg_survivors, seg_skipped = act.note, [], []
            seg_aborted = False
        if seg_aborted and act.kind in ("merge", "push-branch", "open-pr", "arm"):
            continue
        argv = list(act.argv)
        if act.kind == "push-branch" and seg_branch and not dry_run \
                and len(seg_survivors) < act.min_ok:
            # Too few clean merges: delete the local branch, never push/open/arm — but
            # only for THIS segment; any following sibling segment still runs.
            log(f"only {len(seg_survivors)} constituent(s) merged cleanly on "
                f"{seg_branch} (< {act.min_ok}) — aborting this omnibus")
            run_git(["checkout", "--detach", "origin/main"])
            run_git(["branch", "-D", seg_branch], check=False)
            seg_aborted = True
            continue
        if act.kind == "open-pr" and seg_branch:
            live = [open_by_num[n] for n in sorted(seg_survivors)]
            argv = ["pr", "create", "--repo", state["repo"], "--base", "main",
                    "--head", seg_branch,
                    "--title", omnibus_title(len(live), act.depth),
                    "--body", omnibus_body(live, seg_skipped,
                                           depth=act.depth, parent=act.parent)]
        log(("DRY-RUN " if dry_run else "") + f"{act.kind}: {act.tool} "
            + " ".join(argv[:6]) + (" …" if len(argv) > 6 else "")
            + (f"  [{act.note}]" if act.note else ""))
        if dry_run:
            if act.kind == "merge":
                seg_survivors.append(act.constituent)
            continue
        if act.tool == "git":
            if act.may_conflict:
                if run_git(argv, check=False) == 0:
                    seg_survivors.append(act.constituent)
                else:
                    run_git(["merge", "--abort"], check=False)
                    seg_skipped.append(act.constituent)
                    log(f"constituent #{act.constituent} conflicts — skipped (stays armed)")
            elif act.kind == "delete-branch":
                # Best-effort, like arm/disarm: an already-deleted branch (remote
                # ref not found) IS the target state, and a transient delete
                # failure must never wedge the run — the child / replacement-root
                # creation segments planned AFTER this action still have to
                # execute, and §8 stale hygiene retries the delete next run.
                if run_git(argv, check=False) != 0:
                    log("WARNING: delete-branch failed"
                        + (f" [{act.note}]" if act.note else "")
                        + " — non-fatal (already deleted, or retried by the next "
                          "run's stale hygiene); continuing")
            else:
                run_git(argv)
        else:
            try:
                run_gh(argv)
            except subprocess.CalledProcessError as e:
                if act.kind == "arm":
                    # Arm failures are loud but non-fatal: the omnibus PR exists and the
                    # NEXT run's §7 re-arm leg restores the arm idempotently (a
                    # persistently unarmed omnibus is closed at the §6 age bound).
                    log(f"WARNING: arming failed ({e}); omnibus left open UNARMED")
                elif act.kind == "disarm":
                    # Non-fatal: the culprit's arm may already have been dropped by its
                    # own failed merge group — the target state (unarmed) already holds.
                    log(f"WARNING: disarming failed ({e}); arm was likely already dropped")
                else:
                    raise
    return 0


# --------------------------------------------------------------------------------------
# Self-test: hermetic fixtures, gh + git stubbed (plan() is pure — nothing to stub away —
# and the asserts pin exact argv so any policy drift flips the suite red).
# --------------------------------------------------------------------------------------
def _pr(num, head, labels=(), armed_by=ORCHESTRATOR_APP, draft=False, author="worker[bot]",
        title="feat: x", files=(), body="", checks=(), mergeable="MERGEABLE",
        cross=False):
    return {"number": num, "head_ref": head, "title": title, "author_login": author,
            "labels": list(labels), "is_draft": draft,
            "auto_merge": ({"enabled_by": f"app/{armed_by}"} if armed_by else None),
            "files": list(files), "body": body, "checks": list(checks),
            "mergeable": mergeable, "is_cross_repository": cross}


def _worker(num, issue, labels=("review:pass",), **kw):
    return _pr(num, f"sparq-agent/issue-{issue}-123-1", labels=labels, **kw)


def _state(open_prs=(), merged=(), empty=None, branches=(), batch_enabled=True,
           truncated=False, recent_closures=()):
    return {"repo": "sparq-org/sparq",
            "now": datetime(2026, 7, 17, 12, 0, 0, tzinfo=timezone.utc),
            "open_prs": list(open_prs), "merged_omnibus": list(merged),
            "empty_vs_main": dict(empty or {}),
            "remote_omnibus_branches": list(branches),
            "batch_enabled": batch_enabled,
            "open_list_truncated": truncated,
            "recent_omnibus_closures": list(recent_closures)}


def self_test() -> int:
    failures = []

    def check(name, got, want):
        if got != want:
            failures.append(f"{name}:\n  got : {got!r}\n  want: {want!r}")

    stamp_branch = "sparq-omnibus/20260717T120000Z"
    gate_fail = [{"name": "gate", "status": "COMPLETED", "conclusion": "FAILURE"}]

    def _body_of(act):
        return act.argv[act.argv.index("--body") + 1]

    def _comment_of(act):
        return act.argv[act.argv.index("--comment") + 1]

    def creation_kinds(n):
        # The action-kind shape of one omnibus creation segment with n constituents.
        return ["fetch", "create-branch"] + ["merge"] * n \
            + ["push-branch", "open-pr", "arm"]

    # 1. DO-NOTHING: empty repo state -> zero actions.
    check("empty state is a no-op", plan(_state()), [])

    # 2. ELIGIBILITY + LONE PR (g): a single eligible armed worker PR is NEVER batched
    #    (the queue lands it alone), and every flavour of inert companion (draft /
    #    unarmed / wrong label / excluded label / foreign branch / release-plz) stays
    #    excluded, so the pair still produces no omnibus.
    lone = _worker(101, 901)
    check("lone eligible PR is a no-op", plan(_state(open_prs=[lone])), [])
    for bad in (
        _worker(109, 909, draft=True),
        _worker(109, 909, armed_by=None),
        _worker(109, 909, labels=("review:changes",)),
        _worker(109, 909, labels=("review:pass", "needs:user")),
        _worker(109, 909, labels=("review:pass", "trust:pending")),
        _pr(109, "dependabot/cargo/serde-1.0", labels=("review:pass",)),
        _pr(109, "release-plz-2026", labels=("review:pass",),
            author="app/github-actions", title="chore: release v0.2"),
        _worker(109, 909, armed_by="someother-app"),
    ):
        check(f"inert companion ({bad['head_ref']}/{bad['labels']}/draft={bad['is_draft']}"
              f"/arm={bad['auto_merge']}) leaves the lone PR unbatched",
              plan(_state(open_prs=[lone, bad])), [])

    # 3. ROOT CREATION (g): batches ALL eligible ascending — no window slice. Two
    #    eligible -> the exact plan shape + v2 depth-0 marker; NINE eligible (more than
    #    the old queue window of 8) -> ONE nine-constituent omnibus.
    two = [_worker(109, 909), _worker(110, 910, files=("crates/sparq-core/src/lib.rs",))]
    got = plan(_state(open_prs=two))
    check("two-eligible plan shape", [a.kind for a in got], creation_kinds(2))
    check("branch off origin/main", got[1].argv, ["checkout", "-B", stamp_branch, "origin/main"])
    check("first merge is lowest number", got[2].argv,
          ["merge", "--no-ff", "--no-edit", "-m",
           "omnibus: merge PR #109 (sparq-agent/issue-909-123-1)",
           "origin/sparq-agent/issue-909-123-1"])
    check("merges flagged conflict-skippable", [a.may_conflict for a in got[2:4]],
          [True, True])
    check("push argv", got[4].argv, ["push", "origin", stamp_branch])
    check("root push requires MIN_CONSTITUENTS", got[4].min_ok, MIN_CONSTITUENTS)
    check("PR title counts constituents", got[5].argv[got[5].argv.index("--title") + 1],
          "omnibus: 2 reviewed worker PRs")
    body = _body_of(got[5])
    check("body carries the v2 marker at depth 0",
          "<!-- sparq-omnibus:v2 constituents=109,110 depth=0 -->" in body, True)
    check("body self-IDs as SPARQ agent", body.startswith(SELF_ID), True)
    check("body closes both issues",
          ("Closes #909" in body, "Closes #910" in body), (True, True))
    check("body crate column from changed files", "| #110 | #910 | sparq-core |" in body, True)
    check("arm has no merge-method flag (queue chooses)", got[6].argv,
          ["pr", "merge", stamp_branch, "--repo", "sparq-org/sparq", "--auto"])
    check("no review:* label is ever added to the omnibus",
          any("label" in " ".join(a.argv) for a in got if a.tool == "gh"), False)
    nine = [_worker(100 + i, 900 + i) for i in range(9)]
    got = plan(_state(open_prs=nine))
    check("nine eligible -> ONE nine-constituent omnibus (no window slice)",
          [a.kind for a in got], creation_kinds(9))
    check("nine-batch title carries the count",
          got[-2].argv[got[-2].argv.index("--title") + 1],
          "omnibus: 9 reviewed worker PRs")
    check("nine-batch marker lists all nine ascending",
          "<!-- sparq-omnibus:v2 constituents=100,101,102,103,104,105,106,107,108 "
          "depth=0 -->" in _body_of(got[-2]), True)

    # 4. An OPEN (fresh, armed, mergeable) omnibus PR suppresses a new ROOT.
    open_om = _pr(200, stamp_branch, author="app/sparq-orchestrator")  # armed by orchestrator
    check("open omnibus suppresses a new root", plan(_state(open_prs=two + [open_om])), [])

    # 4b. --hygiene-only NEVER creates an omnibus (no App token: a GITHUB_TOKEN omnibus
    #     cannot enter the merge queue), but the closure legs still run.
    check("hygiene-only skips batching", plan(_state(open_prs=two, batch_enabled=False)), [])
    hygiene_merged = {"number": 310, "body": "<!-- sparq-omnibus:v1 constituents=101 -->"}
    got = plan(_state(open_prs=[_worker(101, 901)], merged=[hygiene_merged],
                      empty={101: True}, batch_enabled=False))
    check("hygiene-only still closes constituents", [a.kind for a in got],
          ["close-constituent"])

    # 5. CLOSURE: merged omnibus (v1 marker MUST still parse) + open constituent with an
    #    empty diff -> close w/ comment; the non-empty constituent is left armed — and a
    #    closing constituent is never re-batched, so the one leftover batchable PR is
    #    below MIN_CONSTITUENTS and no root opens.
    merged_om = {"number": 300, "body": "x\n<!-- sparq-omnibus:v1 constituents=101,102 -->\ny"}
    st = _state(open_prs=[_worker(101, 901), _worker(102, 902)],
                merged=[merged_om], empty={101: True, 102: False})
    got = plan(st)
    check("closure closes only the empty constituent (no re-batch of it)",
          [(a.kind, a.argv[:3]) for a in got],
          [("close-constituent", ["pr", "close", "101"])])
    check("closure comment links the omnibus", "#300" in _comment_of(got[0]), True)
    check("closure is idempotent (constituent already closed)",
          plan(_state(merged=[merged_om], empty={101: True})), [])

    # 6. FAILURE BASICS: an open MARKERLESS omnibus with a CONCLUDED gate failure has 0
    #    survivors -> plain close + delete branch; an in-progress gate (armed) is left
    #    alone; mergeable UNKNOWN never acts. (Head-gate checks exist on an omnibus
    #    because it is pushed/created with the App token.)
    failed_om = _pr(400, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                    armed_by=None, checks=gate_fail)
    got = plan(_state(open_prs=[failed_om],
                      branches=["sparq-omnibus/20260717T110000Z"]))
    check("failed markerless omnibus: close+delete only", [(a.kind, a.tool) for a in got],
          [("close-omnibus", "gh"), ("delete-branch", "git")])
    check("failure comment carries the v2 breadcrumb",
          "<!-- sparq-omnibus-failure:v1 bisection=v2 -->" in _comment_of(got[0]), True)
    check("branch delete argv", got[1].argv,
          ["push", "origin", "--delete", "sparq-omnibus/20260717T110000Z"])
    running_om = _pr(401, "sparq-omnibus/20260717T113000Z", author="app/sparq-orchestrator",
                     checks=[{"name": "gate", "status": "IN_PROGRESS", "conclusion": ""}])
    check("in-progress gate omnibus untouched",
          plan(_state(open_prs=[running_om],
                      branches=["sparq-omnibus/20260717T113000Z"])), [])
    unknown_om = _pr(403, "sparq-omnibus/20260717T114500Z", author="app/sparq-orchestrator",
                     mergeable="UNKNOWN")
    check("mergeable UNKNOWN (fresh, armed) never acts",
          plan(_state(open_prs=[unknown_om])), [])

    # 6b. AGE BOUND (liveness backstop for head-invisible failures: merge-group gate
    #     failure / dropped arm / checks never reported): an armed, MERGEABLE omnibus
    #     with NO checks at all, older than MAX_OMNIBUS_AGE_HOURS -> close + delete;
    #     a young one (same shape) is untouched; an unparseable stamp = expired.
    aged_om = _pr(404, "sparq-omnibus/20260717T060000Z", author="app/sparq-orchestrator")
    got = plan(_state(open_prs=[aged_om]))
    check("over-age omnibus closed+deleted", [a.kind for a in got],
          ["close-omnibus", "delete-branch"])
    check("age-close reason names the bound",
          f"did not merge within {MAX_OMNIBUS_AGE_HOURS}h" in _comment_of(got[0]), True)
    young_om = _pr(405, "sparq-omnibus/20260717T090000Z", author="app/sparq-orchestrator")
    check("young armed mergeable omnibus untouched", plan(_state(open_prs=[young_om])), [])
    bad_stamp_om = _pr(406, "sparq-omnibus/not-a-stamp", author="app/sparq-orchestrator")
    check("unparseable stamp treated as expired",
          [a.kind for a in plan(_state(open_prs=[bad_stamp_om]))],
          ["close-omnibus", "delete-branch"])

    # 6c. RE-ARM: a young, MERGEABLE omnibus whose auto-merge arm was DROPPED (merge
    #     groups drop the arm on failure) is re-armed idempotently — and still
    #     suppresses a new root.
    dropped_om = _pr(407, "sparq-omnibus/20260717T090000Z", author="app/sparq-orchestrator",
                     armed_by=None)
    got = plan(_state(open_prs=[dropped_om]))
    check("dropped arm re-armed", [(a.kind, a.argv) for a in got],
          [("arm", ["pr", "merge", "407", "--repo", "sparq-org/sparq", "--auto"])])
    got = plan(_state(open_prs=two + [dropped_om]))
    check("re-armed omnibus still suppresses a new root", [a.kind for a in got], ["arm"])

    # 6d. INFRA CLOSE = NO SAME-RUN REBUILD: an age-expired omnibus closes for an
    #     infra-shaped reason, so the still-eligible pair does NOT get a fresh root in
    #     the same plan (the close-and-rebuild flood damper; the CONFLICT path B-d is
    #     the close-and-recreate that DOES rebuild same-run).
    got = plan(_state(open_prs=two + [aged_om]))
    check("infra (age-expiry) close suppresses the same-run root rebuild",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])

    # ---------------------------- BISECTION v2 ----------------------------
    # B-a. Failing SIX-constituent omnibus (v1 marker: depth=0 implied) -> close parent
    #      + delete branch + TWO 3-constituent children at depth=1, both armed; the
    #      (still-eligible) constituents do NOT also get a new root in the same run.
    six = [_worker(200 + i, 920 + i) for i in range(1, 7)]  # 201..206
    parent6 = _pr(500, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                  armed_by=None, checks=gate_fail,
                  body="<!-- sparq-omnibus:v1 constituents=201,202,203,204,205,206 -->")
    got = plan(_state(open_prs=six + [parent6]))
    check("(a) bisect plan shape: close + delete + two 3-child creations",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(3) + creation_kinds(3))
    child_a = "sparq-omnibus/20260717T120000Z-p500a"
    child_b = "sparq-omnibus/20260717T120000Z-p500b"
    close_comment = _comment_of(got[0])
    check("(i) parent-close comment names child branch A", child_a in close_comment, True)
    check("(i) parent-close comment names child branch B", child_b in close_comment, True)
    check("(a) parent branch deleted", got[1].argv,
          ["push", "origin", "--delete", "sparq-omnibus/20260717T110000Z"])
    opens = [a for a in got if a.kind == "open-pr"]
    body_a, body_b = _body_of(opens[0]), _body_of(opens[1])
    check("(a) child A marker: first half, ascending, depth=1",
          "<!-- sparq-omnibus:v2 constituents=201,202,203 depth=1 -->" in body_a, True)
    check("(a) child B marker: second half, ascending, depth=1",
          "<!-- sparq-omnibus:v2 constituents=204,205,206 depth=1 -->" in body_b, True)
    check("(a) child bodies reference the failed parent PR",
          ("#500" in body_a, "#500" in body_b), (True, True))
    check("(a) child titles carry count + depth",
          [a.argv[a.argv.index("--title") + 1] for a in opens],
          ["omnibus: 3 reviewed worker PRs (bisect depth 1)"] * 2)
    check("(a) children branch off origin/main",
          [a.argv for a in got if a.kind == "create-branch"],
          [["checkout", "-B", child_a, "origin/main"],
           ["checkout", "-B", child_b, "origin/main"]])
    check("(a) both children armed with --auto",
          [a.argv for a in got if a.kind == "arm"],
          [["pr", "merge", child_a, "--repo", "sparq-org/sparq", "--auto"],
           ["pr", "merge", child_b, "--repo", "sparq-org/sparq", "--auto"]])

    # B-a2. hygiene-only cannot create children (no App token) -> v1 close instead.
    got = plan(_state(open_prs=six + [parent6], batch_enabled=False))
    check("(a2) hygiene-only: bisectable parent v1-closes, no children",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])

    # B-a3. an OVER-AGE omnibus NEVER bisects — age-expiry is infra-shaped (outage /
    #       starvation / head-invisible modes), not a verdict against any constituent:
    #       v1 close only, survivors untouched, no children, no same-run rebuild.
    aged_parent = _pr(510, "sparq-omnibus/20260717T060000Z", author="app/sparq-orchestrator",
                      body="<!-- sparq-omnibus:v2 constituents=109,110 depth=0 -->")
    got = plan(_state(open_prs=two + [aged_parent]))
    check("(a3) over-age omnibus: v1 close only — no bisection, no rebuild",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])

    # B-b. Failing TWO-constituent omnibus -> two SINGLETON children (allowed: a
    #      singleton CHILD is how the culprit gets isolated — MIN_CONSTITUENTS applies
    #      to ROOT creation only), each pushable at min_ok=1.
    pair = [_worker(211, 931), _worker(212, 932)]
    parent2 = _pr(520, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                  armed_by=None, checks=gate_fail,
                  body="<!-- sparq-omnibus:v2 constituents=211,212 depth=1 -->")
    got = plan(_state(open_prs=pair + [parent2]))
    check("(b) 2-constituent bisect -> singleton children",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(1) + creation_kinds(1))
    bodies = [_body_of(a) for a in got if a.kind == "open-pr"]
    check("(b) singleton child markers at depth=2",
          ("<!-- sparq-omnibus:v2 constituents=211 depth=2 -->" in bodies[0],
           "<!-- sparq-omnibus:v2 constituents=212 depth=2 -->" in bodies[1]),
          (True, True))
    check("(b) child pushes allow a singleton (min_ok=1)",
          [a.min_ok for a in got if a.kind == "push-branch"], [1, 1])

    # B-c. Failing SINGLETON child -> THE CULPRIT: comment (with machine marker), then
    #      convert to draft, then disarm — in THAT order (crash safety: the arm is only
    #      stripped after the durable breadcrumb exists, and the draft conversion drops
    #      the arm platform-side anyway); parent closed; the culprit PR itself is NOT
    #      closed and gains no labels.
    culprit = _worker(221, 941)
    parent1 = _pr(530, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                  armed_by=None, checks=gate_fail,
                  body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    got = plan(_state(open_prs=[culprit, parent1]))
    check("(c) culprit quarantine plan (comment BEFORE draft+disarm)",
          [a.kind for a in got],
          ["comment", "draft", "disarm", "close-omnibus", "delete-branch"])
    qc = _body_of(got[0])
    check("(c) quarantine comment self-IDs", qc.startswith(SELF_ID), True)
    check("(c) quarantine comment names the failed omnibus", "#530" in qc, True)
    check("(c) quarantine comment carries the machine marker",
          "<!-- sparq-omnibus-culprit:v1 parent=530 reason=gate-failure -->" in qc, True)
    check("(c) quarantine comment hands off to the registry stranded escalation",
          "stranded" in qc, True)
    check("(c) culprit converted to draft (stranded posture)", got[1].argv,
          ["pr", "ready", "221", "--repo", "sparq-org/sparq", "--undo"])
    check("(c) culprit disarm argv", got[2].argv,
          ["pr", "merge", "221", "--repo", "sparq-org/sparq", "--disable-auto"])
    check("(c) culprit PR is NOT closed",
          any(a.argv[:3] == ["pr", "close", "221"] for a in got), False)
    check("(c) no labels are added anywhere",
          any(a.tool == "gh" and "label" in " ".join(a.argv) for a in got), False)
    # B-c2. the quarantined culprit is excluded from the same-run fresh root that the
    #       REMAINING eligible PRs still get (parent died -> suppression lifted).
    others = [_worker(222, 942), _worker(223, 943)]
    got = plan(_state(open_prs=[culprit, parent1] + others))
    check("(c2) quarantine + fresh root over the remaining eligible",
          [a.kind for a in got],
          ["comment", "draft", "disarm", "close-omnibus", "delete-branch"]
          + creation_kinds(2))
    root_body = _body_of([a for a in got if a.kind == "open-pr"][0])
    check("(c2) the culprit is NOT re-batched into the fresh root",
          "<!-- sparq-omnibus:v2 constituents=222,223 depth=0 -->" in root_body, True)

    # B-c3. CONVICTION GATING: only a strict `gate` FAILURE conclusion convicts a
    #       singleton. cancelled / timed_out (a human-cancelled run, runner starvation)
    #       and age-expiry (queue congestion, checks that never reported — outage
    #       modes) v1-close the parent and leave the sole survivor individually armed:
    #       under a sustained outage every singleton leaf would otherwise serially
    #       disarm the entire armed backlog. (These conclusions never bisect at ANY
    #       survivor count either — B-a4.)
    for concl in ("CANCELLED", "TIMED_OUT"):
        soft = _pr(530, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                   armed_by=None,
                   checks=[{"name": "gate", "status": "COMPLETED", "conclusion": concl}],
                   body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
        got = plan(_state(open_prs=[culprit, soft]))
        check(f"(c3) {concl.lower()} singleton: v1 close only — survivor stays armed",
              [a.kind for a in got], ["close-omnibus", "delete-branch"])
    aged_child = _pr(531, "sparq-omnibus/20260717T060000Z-p530a",
                     author="app/sparq-orchestrator",
                     body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    got = plan(_state(open_prs=[culprit, aged_child]))
    check("(c3) age-expired singleton: v1 close only — survivor stays armed",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    check("(c3) age-expired singleton: no comment/draft/disarm on the survivor",
          any(a.kind in ("comment", "draft", "disarm") for a in got), False)

    # B-a4. BISECTION GATING + the 16-PR INFRA-FAILURE SIM: ONLY a strict `gate`
    #       FAILURE conclusion bisects. cancelled / timed_out (and age-expiry, a3)
    #       are infra-shaped: a 16-constituent root under a sustained outage must
    #       produce ZERO new omnibus nodes — no 2->4->8->16 split cascade, no
    #       quarantine, and no same-run depth-0 rebuild (the queue-flood mode) —
    #       just a v1 close; all 16 constituents stay individually armed. It also
    #       pins that cancelled / timed_out still CLOSE the omnibus at all (a
    #       mutant narrowing GATE_FAILING_CONCLUSIONS to failure-only would leave
    #       it open until the age bound).
    sixteen = [_worker(700 + i, 9700 + i) for i in range(16)]
    nums16 = ",".join(str(700 + i) for i in range(16))
    for concl in ("CANCELLED", "TIMED_OUT"):
        p = _pr(505, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                armed_by=None,
                checks=[{"name": "gate", "status": "COMPLETED", "conclusion": concl}],
                body=f"<!-- sparq-omnibus:v2 constituents={nums16} depth=0 -->")
        got = plan(_state(open_prs=sixteen + [p]))
        check(f"(a4) 16-PR sim, gate {concl.lower()}: v1 close only — zero new nodes",
              [a.kind for a in got], ["close-omnibus", "delete-branch"])
        check(f"(a4) 16-PR sim, gate {concl.lower()}: no constituent is implicated",
              any(a.kind in ("comment", "draft", "disarm", "create-branch")
                  for a in got), False)
    aged16 = _pr(506, "sparq-omnibus/20260717T060000Z", author="app/sparq-orchestrator",
                 body=f"<!-- sparq-omnibus:v2 constituents={nums16} depth=0 -->")
    got = plan(_state(open_prs=sixteen + [aged16]))
    check("(a4) 16-PR sim, age-expiry: v1 close only — zero new nodes",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    hard16 = _pr(507, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                 armed_by=None, checks=gate_fail,
                 body=f"<!-- sparq-omnibus:v2 constituents={nums16} depth=0 -->")
    got = plan(_state(open_prs=sixteen + [hard16]))
    check("(a4) 16-PR strict gate FAILURE still bisects into two 8-child halves",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(8) + creation_kinds(8))

    # B-j. ROOT COOLDOWN (durable, cross-run): any sparq-omnibus PR closed UNMERGED
    #      within ROOT_COOLDOWN_HOURS suppresses a NEW ROOT — the infra-flood
    #      backstop survives the batcher's statelessness via GitHub's closed-PR
    #      record. An older closure does not suppress, and bisection children are
    #      exempt (the cooldown damps re-rooting, never culprit isolation).
    fresh_close = [datetime(2026, 7, 17, 11, 30, 0, tzinfo=timezone.utc)]  # 0.5h ago
    stale_close = [datetime(2026, 7, 17, 8, 0, 0, tzinfo=timezone.utc)]    # 4h ago
    check("(j) recent failure-close: cooldown suppresses the new root",
          plan(_state(open_prs=two, recent_closures=fresh_close)), [])
    check("(j) cooldown elapsed: root creation resumes",
          [a.kind for a in plan(_state(open_prs=two, recent_closures=stale_close))],
          creation_kinds(2))
    got = plan(_state(open_prs=six + [parent6], recent_closures=fresh_close))
    check("(j) cooldown never blocks bisection children",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(3) + creation_kinds(3))

    # B-g. OPEN-PR LIST TRUNCATION: when gather_state's listing saturated its limit,
    #      constituent liveness is unreliable — absence from the list must NOT convict
    #      anything, and an omitted LIVE omnibus breaks the one-tree check. A failing
    #      parent v1-closes (no bisection, no quarantine) — closing/deleting a VISIBLE
    #      dying omnibus is still safe — but no new root opens on incomplete data.
    got = plan(_state(open_prs=[culprit, parent1], truncated=True))
    check("(g) truncated listing: singleton gate-failure does NOT quarantine",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    got = plan(_state(open_prs=six + [parent6], truncated=True))
    check("(g) truncated listing: bisectable parent v1-closes; NO re-root either",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    # B-g2. TRUNCATION IS FAIL-SAFE for the one-tree guarantee: a live omnibus OMITTED
    #       from the truncated list leaves its remote branch looking stale — that
    #       branch must NOT be deleted, and no new root may open over the invisible
    #       tree. Nothing destructive happens on incomplete data.
    got = plan(_state(open_prs=two, branches=["sparq-omnibus/20260717T100000Z"],
                      truncated=True))
    check("(g2) truncated: no stale-branch delete, no new root — nothing destructive",
          got, [])

    # B-d. CONFLICTING omnibus: NOT a constituent fault -> v1 close, NO children, no
    #      disarm; close-and-recreate then rebuilds a fresh ROOT (depth 0) off current
    #      main from the still-armed constituents.
    conflict_parent = _pr(540, "sparq-omnibus/20260717T110000Z",
                          author="app/sparq-orchestrator", armed_by=None,
                          mergeable="CONFLICTING",
                          body="<!-- sparq-omnibus:v2 constituents=109,110 depth=0 -->")
    got = plan(_state(open_prs=two + [conflict_parent]))
    check("(d) conflicting omnibus: close + fresh root, NO children",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(2))
    check("(d) no disarm on a conflict", any(a.kind == "disarm" for a in got), False)
    root_body = _body_of([a for a in got if a.kind == "open-pr"][0])
    check("(d) rebuild is a ROOT (depth 0), not a child",
          "<!-- sparq-omnibus:v2 constituents=109,110 depth=0 -->" in root_body, True)

    # B-d2. COMBINED STATE (conflict + STALE failed gate): mergeability is checked
    #       FIRST. A CONFLICTING omnibus whose head ALSO carries an OLD concluded gate
    #       FAILURE (evidence against a stale base — main moved under it) takes the
    #       conflict path: v1 close only, NO bisection, NO quarantine — its singleton
    #       marker constituent is untouched and stays individually armed. Conflicts
    #       must NEVER implicate constituents.
    conflict_stale = _pr(545, "sparq-omnibus/20260717T110000Z",
                         author="app/sparq-orchestrator", armed_by=None,
                         mergeable="CONFLICTING", checks=gate_fail,
                         body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    got = plan(_state(open_prs=[culprit, conflict_stale]))
    check("(d2) conflicting + stale failed gate: close only — constituent untouched",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    check("(d2) the close reason is the CONFLICT, not the stale gate verdict",
          "conflicts with `main`" in _comment_of(got[0]), True)
    check("(d2) no comment/draft/disarm ever reaches the constituent",
          any(a.kind in ("comment", "draft", "disarm") for a in got), False)

    # B-e. DEPTH CAP: a failing omnibus at depth DEPTH_CAP with >=2 survivors -> v1
    #      fallback: close only, NO children, NO same-run rebuild, constituents
    #      untouched (still individually armed).
    capped = _pr(550, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                 armed_by=None, checks=gate_fail,
                 body=f"<!-- sparq-omnibus:v2 constituents=109,110 depth={DEPTH_CAP} -->")
    got = plan(_state(open_prs=two + [capped]))
    check("(e) depth-cap: close only — no children, no rebuild, constituents untouched",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])

    # B-f. SURVIVOR FILTERING: merged-meanwhile (601, absent from the open set),
    #      no-longer-armed (602) and already-empty-vs-main (603) constituents are
    #      excluded; the two real survivors (604, 605) become the children.
    parent_f = _pr(560, "sparq-omnibus/20260717T110000Z", author="app/sparq-orchestrator",
                   armed_by=None, checks=gate_fail,
                   body="<!-- sparq-omnibus:v2 constituents=601,602,603,604,605 depth=0 -->")
    got = plan(_state(
        open_prs=[_worker(602, 962, armed_by=None), _worker(603, 963),
                  _worker(604, 964), _worker(605, 965), parent_f],
        empty={603: True}))
    check("(f) filtered-survivor bisect shape",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(1) + creation_kinds(1))
    bodies = [_body_of(a) for a in got if a.kind == "open-pr"]
    check("(f) children carry only the true survivors",
          ("<!-- sparq-omnibus:v2 constituents=604 depth=1 -->" in bodies[0],
           "<!-- sparq-omnibus:v2 constituents=605 depth=1 -->" in bodies[1]),
          (True, True))

    # B-h. RE-RUN IDEMPOTENCE: children already open (parent closed, so it is absent
    #      from the open set) -> nothing new is planned: no duplicate children, no new
    #      root, live child branches kept.
    kids = [_pr(570, "sparq-omnibus/20260717T115000Z-p560a", author="app/sparq-orchestrator",
                body="<!-- sparq-omnibus:v2 constituents=604 depth=1 -->"),
            _pr(571, "sparq-omnibus/20260717T115000Z-p560b", author="app/sparq-orchestrator",
                body="<!-- sparq-omnibus:v2 constituents=605 depth=1 -->")]
    check("(h) existing children (parent closed): re-run plans no duplicates",
          plan(_state(open_prs=[_worker(604, 964), _worker(605, 965)] + kids,
                      branches=["sparq-omnibus/20260717T115000Z-p560a",
                                "sparq-omnibus/20260717T115000Z-p560b"])), [])

    # B-s. PROVENANCE / IMPERSONATION: a PR spoofing the sparq-omnibus/ prefix from a
    #      FORK head (cross-repo) or from a NON-App author is NEVER classified as an
    #      omnibus. Even carrying a concluded gate FAILURE + a singleton marker it
    #      must not quarantine (or otherwise touch) the named constituent, must not
    #      be closed/bisected itself, and must not suppress the root over the real
    #      eligible PRs.
    spoof_fork = _pr(600, "sparq-omnibus/20260717T113000Z",
                     author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                     cross=True,
                     body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    spoof_author = _pr(600, "sparq-omnibus/20260717T113000Z", author="mallory",
                       armed_by=None, checks=gate_fail,
                       body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    for spoof, tag in ((spoof_fork, "fork head"), (spoof_author, "non-App author")):
        got = plan(_state(open_prs=[culprit, spoof] + others))
        check(f"(s) spoofed omnibus ({tag}) never quarantines/closes/bisects",
              any(a.kind in ("comment", "draft", "disarm", "close-omnibus")
                  for a in got), False)
        check(f"(s) spoofed omnibus ({tag}) does not suppress the root",
              [a.kind for a in got], creation_kinds(3))
        check(f"(s) spoofed omnibus ({tag}): the constituent it names is batched, "
              f"not drafted",
              "<!-- sparq-omnibus:v2 constituents=221,222,223 depth=0 -->"
              in _body_of([a for a in got if a.kind == "open-pr"][0]), True)
    got = plan(_state(open_prs=[spoof_author],
                      branches=["sparq-omnibus/20260717T113000Z"]))
    check("(s) branch under an open (even spoofed) PR is still protected from "
          "stale deletion", got, [])

    # B-x. ELIGIBILITY IS NOT CAUSALITY: marker {221, 225}; 225 became DRAFT
    #      (ineligible) WITHOUT landing — its code is still in the failed head, so
    #      the lone eligible survivor 221 must NOT be convicted on the parent's
    #      verdict. Instead: close the parent and create ONE singleton RE-TEST child
    #      carrying only 221 (fresh evidence off current main).
    lingering = _worker(225, 945, draft=True)
    parent_amb = _pr(535, "sparq-omnibus/20260717T110000Z",
                     author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                     body="<!-- sparq-omnibus:v2 constituents=221,225 depth=2 -->")
    got = plan(_state(open_prs=[culprit, lingering, parent_amb]))
    check("(x) ambiguous causality: close + ONE singleton re-test child, NO conviction",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(1))
    check("(x) no comment/draft/disarm ever reaches a constituent",
          any(a.kind in ("comment", "draft", "disarm") for a in got), False)
    retest_open = [a for a in got if a.kind == "open-pr"][0]
    check("(x) re-test child carries ONLY the eligible survivor, depth+1",
          "<!-- sparq-omnibus:v2 constituents=221 depth=3 -->"
          in _body_of(retest_open), True)
    check("(x) re-test child branch is a -p<parent>a child of the failed omnibus",
          [a.argv for a in got if a.kind == "create-branch"],
          [["checkout", "-B", "sparq-omnibus/20260717T120000Z-p535a", "origin/main"]])
    check("(x) close comment names the code-bearing blocker sibling",
          "#225" in _comment_of(got[0]), True)
    check("(x) close comment names the re-test child branch",
          "sparq-omnibus/20260717T120000Z-p535a" in _comment_of(got[0]), True)
    # B-x2. Same when the sibling is ABSENT from the open set (closed unmerged — no
    #       emptiness proof exists): its code may still be the culprit.
    got = plan(_state(open_prs=[culprit, parent_amb]))
    check("(x2) closed-unmerged sibling still blocks conviction (re-test instead)",
          [a.kind for a in got],
          ["close-omnibus", "delete-branch"] + creation_kinds(1))
    # B-x3. A sibling PROVEN landed (empty vs main) IS cleanly excluded: the causal
    #       set collapses to the survivor and the sound conviction path still runs.
    got = plan(_state(open_prs=[culprit, lingering, parent_amb], empty={225: True}))
    check("(x3) landed sibling cleanly excluded: conviction still runs",
          [a.kind for a in got],
          ["comment", "draft", "disarm", "close-omnibus", "delete-branch"])
    # B-x4. Ambiguity with no way to create a re-test child (hygiene-only) or at the
    #       depth cap: the NON-culpable close — never a quarantine, and at the cap
    #       no same-run root rebuild either.
    got = plan(_state(open_prs=[culprit, lingering, parent_amb], batch_enabled=False))
    check("(x4) hygiene-only ambiguity: v1 close, survivor stays armed",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    amb_capped = _pr(536, "sparq-omnibus/20260717T110000Z",
                     author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                     body=f"<!-- sparq-omnibus:v2 constituents=221,225 "
                          f"depth={DEPTH_CAP} -->")
    got = plan(_state(open_prs=[culprit, lingering, amb_capped] + others))
    check("(x5) depth-cap ambiguity: v1 close, no re-test, no same-run rebuild",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])

    # B-y. INDETERMINATE MERGEABILITY: after main advances GitHub reports UNKNOWN
    #      before resolving (possibly to CONFLICTING). A gate FAILURE without a
    #      DEFINITIVE MERGEABLE verdict must implicate NOBODY and do NOTHING this
    #      run — no quarantine, no bisection, no close — while the still-open
    #      omnibus keeps suppressing a new root.
    for mstate in ("UNKNOWN", ""):
        limbo = _pr(537, "sparq-omnibus/20260717T110000Z",
                    author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                    mergeable=mstate,
                    body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
        check(f"(y) gate failure + mergeable {mstate or 'unreported'!r}: no action",
              plan(_state(open_prs=[culprit, limbo])), [])
        check(f"(y) gate failure + mergeable {mstate or 'unreported'!r} still "
              f"suppresses a new root",
              plan(_state(open_prs=[culprit, limbo] + others)), [])
    limbo6 = _pr(538, "sparq-omnibus/20260717T110000Z",
                 author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                 mergeable="UNKNOWN",
                 body="<!-- sparq-omnibus:v2 constituents=201,202,203,204,205,206 -->")
    check("(y) gate failure + UNKNOWN never bisects either",
          plan(_state(open_prs=six + [limbo6])), [])
    # B-y2. Only the age-expiry NON-culpable close may act on a persistently-
    #       unresolved omnibus — close + delete, nobody implicated.
    limbo_aged = _pr(539, "sparq-omnibus/20260717T060000Z",
                     author="app/sparq-orchestrator", armed_by=None, checks=gate_fail,
                     mergeable="UNKNOWN",
                     body="<!-- sparq-omnibus:v2 constituents=221 depth=3 -->")
    got = plan(_state(open_prs=[culprit, limbo_aged]))
    check("(y2) persistently-unresolved + expired: non-culpable close only",
          [a.kind for a in got], ["close-omnibus", "delete-branch"])
    check("(y2) the close reason records the unresolved mergeability",
          "mergeability never became definitive" in _comment_of(got[0]), True)
    check("(y2) no constituent implicated, no children",
          any(a.kind in ("comment", "draft", "disarm", "create-branch")
              for a in got), False)

    # 9. STALE HYGIENE: remote omnibus branch w/o an open PR is deleted; a branch with an
    #    open PR survives.
    got = plan(_state(open_prs=[open_om],
                      branches=[stamp_branch, "sparq-omnibus/20260701T000000Z"]))
    check("stale branch deleted, live branch kept", [(a.kind, a.argv) for a in got],
          [("delete-branch", ["push", "origin", "--delete", "sparq-omnibus/20260701T000000Z"])])

    # 10. EXECUTE with STUBBED gh+git: conflict on one constituent re-templates the tail
    #     over that segment's survivors; the arm argv never gains a method flag. (No
    #     subprocess escapes the stubs.)
    three = two + [_worker(111, 911)]
    st = _state(open_prs=three)
    acts = plan(st)
    calls = []

    conflict_target = "origin/sparq-agent/issue-910-123-1"

    def stub_git(argv, check=True):
        calls.append(("git", list(argv)))
        return 1 if (argv[:1] == ["merge"] and conflict_target in argv) else 0

    def stub_gh(argv, capture=True):
        calls.append(("gh", list(argv)))
        return ""

    global run_git, run_gh, run_git_out
    real_git, real_gh, real_git_out = run_git, run_gh, run_git_out
    run_git, run_gh = stub_git, stub_gh
    try:
        execute(acts, st, dry_run=False)
    finally:
        run_git, run_gh = real_git, real_gh
    merges = [c for c in calls if c[0] == "git" and c[1][:1] == ["merge"] and "--abort" not in c[1]]
    check("three merges attempted", len(merges), 3)
    check("conflict aborted", ("git", ["merge", "--abort"]) in calls, True)
    opened = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "create"]]
    check("one PR opened after conflict", len(opened), 1)
    tbody = opened[0][1][opened[0][1].index("--body") + 1]
    check("re-templated marker drops the conflicted constituent",
          "<!-- sparq-omnibus:v2 constituents=109,111 depth=0 -->" in tbody, True)
    check("re-templated body records the skip",
          "Skipped (merge conflict — still individually armed): #110" in tbody, True)
    check("re-templated title", opened[0][1][opened[0][1].index("--title") + 1],
          "omnibus: 2 reviewed worker PRs")
    armed = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "merge"]]
    check("armed exactly once, --auto, no method", armed,
          [("gh", ["pr", "merge", stamp_branch, "--repo", "sparq-org/sparq", "--auto"])])

    # 11. EXECUTE abort: a 2-constituent ROOT where BOTH conflict is below its
    #     min_ok=MIN_CONSTITUENTS -> local branch deleted, nothing pushed/opened/armed.
    acts = plan(_state(open_prs=two))
    calls.clear()

    def stub_git_all_conflict(argv, check=True):
        calls.append(("git", list(argv)))
        return 1 if argv[:1] == ["merge"] and "--abort" not in argv else 0

    run_git, run_gh = stub_git_all_conflict, stub_gh
    try:
        execute(acts, _state(open_prs=two), dry_run=False)
    finally:
        run_git, run_gh = real_git, real_gh
    check("all-conflict: no push, no PR, no arm",
          [c for c in calls if c[0] == "gh"
           or c[1][:1] == ["push"]], [])
    check("all-conflict: local branch deleted",
          ("git", ["branch", "-D", stamp_branch]) in calls, True)

    # 12. EXECUTE segmented bisection children: child A's sole merge conflicts -> A is
    #     aborted (its min_ok=1 unmet) WITHOUT sinking child B, which is still pushed,
    #     opened with ITS OWN re-templated body, and armed.
    st = _state(open_prs=pair + [parent2])
    acts = plan(st)
    calls.clear()

    def stub_git_conflict_211(argv, check=True):
        calls.append(("git", list(argv)))
        return 1 if (argv[:1] == ["merge"]
                     and "origin/sparq-agent/issue-931-123-1" in argv) else 0

    run_git, run_gh = stub_git_conflict_211, stub_gh
    try:
        execute(acts, st, dry_run=False)
    finally:
        run_git, run_gh = real_git, real_gh
    child_a2 = "sparq-omnibus/20260717T120000Z-p520a"
    child_b2 = "sparq-omnibus/20260717T120000Z-p520b"
    opened = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "create"]]
    check("segmented: only the clean child opens a PR", len(opened), 1)
    check("segmented: the clean child keeps its own marker",
          "<!-- sparq-omnibus:v2 constituents=212 depth=2 -->"
          in opened[0][1][opened[0][1].index("--body") + 1], True)
    check("segmented: aborted child branch deleted locally",
          ("git", ["branch", "-D", child_a2]) in calls, True)
    pushes = [c for c in calls if c[0] == "git" and c[1][:1] == ["push"]
              and "--delete" not in c[1]]
    check("segmented: only child B pushed", pushes,
          [("git", ["push", "origin", child_b2])])
    armed = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "merge"]
             and "--auto" in c[1]]
    check("segmented: only child B armed", armed,
          [("gh", ["pr", "merge", child_b2, "--repo", "sparq-org/sparq", "--auto"])])

    # 13. EXECUTE: remote branch deletion is BEST-EFFORT — a failing
    #     `git push --delete` (perms flake, ref already gone) must NOT abort the run:
    #     the child segments planned AFTER the delete still execute (push / open /
    #     arm), so a persistently failing delete can never wedge the state machine at
    #     the same action run after run. The stub RAISES when the delete is invoked
    #     with check=True, so reverting the best-effort handling crashes this test.
    st = _state(open_prs=pair + [parent2])
    acts = plan(st)
    calls.clear()

    def stub_git_delete_fails(argv, check=True):
        calls.append(("git", list(argv)))
        if argv[:3] == ["push", "origin", "--delete"]:
            if check:
                raise subprocess.CalledProcessError(1, ["git"] + argv)
            return 1
        return 0

    run_git, run_gh = stub_git_delete_fails, stub_gh
    try:
        execute(acts, st, dry_run=False)
    finally:
        run_git, run_gh = real_git, real_gh
    check("delete-failure: the parent-branch delete was attempted",
          any(c[0] == "git" and c[1][:3] == ["push", "origin", "--delete"]
              for c in calls), True)
    opened = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "create"]]
    check("delete-failure: BOTH child segments still open their PRs", len(opened), 2)
    armed = [c for c in calls if c[0] == "gh" and c[1][:2] == ["pr", "merge"]
             and "--auto" in c[1]]
    check("delete-failure: both children still armed", len(armed), 2)

    # 14. GATHER_STATE boundary (wiring, not just plan()): stub gh + git at the
    #     run_gh / run_git / run_git_out seams and assert gather_state itself
    #     (a) SETS open_list_truncated when the open listing saturates its limit,
    #     (b) CAPTURES unmerged-close cooldown timestamps from the closed-PR record,
    #     (c) drops spoofed (fork / non-App) omnibus records from the merged +
    #     cooldown sets and never heavy-fetches them. Mutating gather_state to skip
    #     saturation detection or discard closure timestamps reds these directly.
    def fake_open(n):
        return [{"number": 10_000 + i,
                 "headRefName": f"sparq-agent/issue-{10_000 + i}-1-1",
                 "title": "feat: x", "author": {"login": "worker[bot]"},
                 "labels": [], "isDraft": False, "autoMergeRequest": None,
                 "isCrossRepository": False} for i in range(n)]

    gh_payloads = {}

    def stub_gh_gather(argv, capture=True):
        calls.append(("gh", list(argv)))
        if argv[:2] == ["pr", "list"]:
            return json.dumps(gh_payloads[argv[argv.index("--state") + 1]])
        if argv[:2] == ["pr", "view"]:
            return json.dumps({"body": "", "mergeable": "MERGEABLE",
                               "statusCheckRollup": [], "files": []})
        raise AssertionError(f"unexpected gh argv in gather_state: {argv}")

    def stub_git_gather(argv, check=True):
        calls.append(("git", list(argv)))
        return 0

    def stub_git_out_gather(argv, check=True):
        calls.append(("git", list(argv)))
        return ""  # ls-remote: no remote omnibus branches

    now_fixed = datetime(2026, 7, 17, 12, 0, 0, tzinfo=timezone.utc)
    run_git, run_gh, run_git_out = stub_git_gather, stub_gh_gather, stub_git_out_gather
    try:
        calls.clear()
        gh_payloads = {"open": fake_open(OPEN_PR_LIST_LIMIT), "merged": [],
                       "closed": []}
        st = gather_state("sparq-org/sparq", now_fixed)
        check("(14a) saturated open listing SETS open_list_truncated",
              st["open_list_truncated"], True)

        calls.clear()
        spoof_open = {"number": 901, "headRefName": "sparq-omnibus/20260717T110000Z",
                      "title": "omnibus: totally real", "author": {"login": "mallory"},
                      "labels": [], "isDraft": False, "autoMergeRequest": None,
                      "isCrossRepository": True}
        gh_payloads = {
            "open": fake_open(3) + [spoof_open],
            "merged": [
                {"number": 800, "headRefName": "sparq-omnibus/20260717T010000Z",
                 "body": "<!-- sparq-omnibus:v2 constituents=1 depth=0 -->",
                 "author": {"login": "app/sparq-orchestrator"},
                 "isCrossRepository": False},
                {"number": 801, "headRefName": "sparq-omnibus/20260717T010500Z",
                 "body": "<!-- sparq-omnibus:v2 constituents=2 depth=0 -->",
                 "author": {"login": "mallory"}, "isCrossRepository": True}],
            "closed": [
                {"number": 810, "headRefName": "sparq-omnibus/20260717T090000Z",
                 "closedAt": "2026-07-17T11:00:00Z", "mergedAt": None,
                 "author": {"login": "sparq-orchestrator[bot]"},
                 "isCrossRepository": False},
                {"number": 811, "headRefName": "sparq-omnibus/20260717T080000Z",
                 "closedAt": "2026-07-17T10:00:00Z",
                 "mergedAt": "2026-07-17T10:05:00Z",
                 "author": {"login": "app/sparq-orchestrator"},
                 "isCrossRepository": False},
                {"number": 812, "headRefName": "sparq-omnibus/20260717T070000Z",
                 "closedAt": "2026-07-17T11:30:00Z", "mergedAt": None,
                 "author": {"login": "mallory"}, "isCrossRepository": False}]}
        st = gather_state("sparq-org/sparq", now_fixed)
        check("(14b) unsaturated open listing: not truncated",
              st["open_list_truncated"], False)
        check("(14c) cooldown CAPTURES the unmerged App close; drops merged + spoofed",
              st["recent_omnibus_closures"],
              [datetime(2026, 7, 17, 11, 0, 0, tzinfo=timezone.utc)])
        check("(14d) merged-omnibus set drops the fork/non-App spoof",
              [m["number"] for m in st["merged_omnibus"]], [800])
        check("(14e) spoofed open omnibus is never heavy-fetched",
              any(c[0] == "gh" and c[1][:3] == ["pr", "view", "901"] for c in calls),
              False)
        check("(14f) isCrossRepository is wired into the snapshot",
              [p["is_cross_repository"] for p in st["open_prs"]
               if p["number"] == 901], [True])
    finally:
        run_git, run_gh, run_git_out = real_git, real_gh, real_git_out

    if failures:
        for f in failures:
            print(f"[self-test] FAIL {f}", file=sys.stderr)
        print(f"[self-test] {len(failures)} failure(s)", file=sys.stderr)
        return 1
    print("[self-test] all checks passed (gh + git stubbed; no live calls)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(prog=PROG, description=__doc__)
    ap.add_argument("--repo", help="owner/name")
    ap.add_argument("--dry-run", action="store_true", help="print the plan, mutate nothing")
    ap.add_argument("--hygiene-only", action="store_true",
                    help="closure/failure/re-arm/stale legs only; never create a new "
                         "omnibus (used when no App token is available — a GITHUB_TOKEN "
                         "omnibus cannot enter the merge queue)")
    ap.add_argument("--self-test", action="store_true", help="hermetic fixtures; gh+git stubbed")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not args.repo:
        ap.error("--repo is required outside --self-test")
    now = datetime.now(timezone.utc)
    state = gather_state(args.repo, now, batch_enabled=not args.hygiene_only)
    actions = plan(state)
    if not actions:
        log("nothing to do")
        return 0
    return execute(actions, state, args.dry_run)


if __name__ == "__main__":
    sys.exit(main())
