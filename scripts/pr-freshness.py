#!/usr/bin/env python3
# [FABLE-5] PR-freshness refresher (scheduled + on-main-push; NON-gating). Issue #3424.
#
# WHAT THIS IS
# ------------
# The deterministic policy core of .github/workflows/pr-freshness.yml. When a fix to CI/gate
# code merges to `main`, every open PR keeps its stale-red required `gate` — the gate was
# computed with the OLD gate code on an OLD merge ref. `gh run rerun` does NOT help: a rerun
# re-executes the PR's old scripts on the old merge commit. The ONLY correct refresh is the
# update-branch API (PUT /repos/{o}/{r}/pulls/{n}/update-branch), which pushes a merge commit
# with current `main` to the PR branch and thereby triggers a fresh pull_request CI run on a
# rebuilt merge ref. On 2026-07-17 ~17 armed review:pass PRs sat red for an hour because
# nothing did this automatically (issue #3424) — this script is the automation.
#
# POLICY (issue #3424)
# --------------------
# A PR is refreshed iff ALL of:
#   (a) it is OPEN, NON-draft, targets `main` (update-branch merges the PR's BASE branch,
#       so for a stacked PR the behind-main compare is meaningless and the call could
#       only 422 or merge the wrong branch), and (auto-merge is armed OR it carries
#       label `review:pass`);
#   (b) its branch is BEHIND main (compare(head...main).ahead_by > 0);
#   (c) the latest `gate` check-run on its head CONCLUDED red (failure/cancelled/timed_out/
#       action_required/startup_failure/stale) OR the head has NO `gate` check-run at all
#       (the merge ref was never gated);
#   (d) it does NOT carry label `review:needs-user`, and it is NOT DIRTY (merge conflict —
#       update-branch would 422; DIRTY PRs are REPORTED for the merge-fixer instead).
# and NONE of:
#   - fresh CI already queued/in-progress on its CURRENT head (a non-terminal `gate`
#     check-run, or any queued/in-progress workflow run on the head SHA) — never
#     cancel-churn a run that is already recomputing the verdict on a current merge ref;
#   - COOLDOWN: the head commit is younger than COOLDOWN_MIN minutes (see below);
#   - the per-tick CAP (MAX_UPDATES_PER_TICK) is already spent — the remainder is skipped
#     WITH a log line ("truncated"), never silently.
#
# COOLDOWN MECHANISM (the simplest robust one — documented per #3424)
# -------------------------------------------------------------------
# update-branch pushes a fresh merge commit to the PR branch, so the head commit's
# COMMITTER timestamp *is* the branch's last-update time. We read it from the same
# compare() call that measures behind-ness (compare base_commit == the head commit) and
# skip any PR whose head commit is < COOLDOWN_MIN old. No marker comments, no external
# state, nothing to garbage-collect; an author's own recent push also (correctly) counts
# as "recently refreshed" — its CI is at most COOLDOWN_MIN stale.
#
# LOOP-SAFETY / CONGESTION SAFETY
# -------------------------------
# update-branch pushes a merge commit to the PR BRANCH, which triggers that PR's
# pull_request CI but NOT another push to main — so the workflow's on-main-push trigger
# cannot recurse. The per-tick cap + per-PR cooldown bound the CI fan-out per tick
# (ci-summary.yml's congestion-collapse doctrine, bead sq-90cv4: uncapped mass triggering
# once melted the runner pool; never re-create that).
#
# DESIGN: the policy is a PURE FUNCTION plan(prs, now) -> [Action] over pre-enriched PR
# dicts. An Action carries the exact `gh` argv it maps to. `live` mode enriches candidates
# with 1 compare + 1 check-runs + 1 runs-list API call each, then runs the argv through a
# real gh; `--self-test` runs plan() over fixtures with gh STUBBED — no live mutation is
# possible from a test, and every guard has a fixture that turns red if the guard is
# deleted (non-vacuity).
#
# USAGE
#   scripts/pr-freshness.py --repo owner/repo                # live refresh
#   scripts/pr-freshness.py --repo owner/repo --report-only  # print plan, no gh mutations
#   scripts/pr-freshness.py --self-test                      # hermetic; gh stubbed
#
# Exit 0 on success; non-zero on a real error, a failed self-test, or any unexpected
# (non-head-race) update-branch failure in live mode — a broken/underscoped token must
# turn the workflow red, never report "applied" while updating nothing.
import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone

PROG = "pr-freshness"

# Congestion-safety knobs (see LOOP-SAFETY above). Cap chosen with the maintainer in
# issue #3424; cooldown > the typical gate wall-clock so one PR is never churned twice
# while its fresh run is still converging.
MAX_UPDATES_PER_TICK = 6
COOLDOWN_MIN = 40
# Runs-per-head listing page. Raised from 30 so the common case is a COMPLETE listing;
# correctness does not depend on the value — a truncated page is detected via
# total_count and fails closed (see enrich).
_RUNS_PAGE = 100

ARMED_LABEL = "review:pass"
BLOCK_LABEL = "review:needs-user"

# Terminal-red gate conclusions (same set ci-summary's aggregator treats as failing).
_GATE_RED = {"failure", "cancelled", "timed_out", "action_required", "startup_failure", "stale"}
_GATE_GREEN = {"success", "neutral", "skipped"}


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def _parse_iso(s: str) -> datetime:
    s = (s or "").strip()
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    dt = datetime.fromisoformat(s)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt


# --------------------------------------------------------------------------------------
# The action model (same shape as scripts/pr-backlog.py). `update` rows carry the
# update-branch argv; `skip` / `dirty` rows are report-only for the step summary.
# --------------------------------------------------------------------------------------
@dataclass
class Action:
    kind: str                       # update | skip | dirty
    pr: int
    reason: str
    argv: list = field(default_factory=list)

    def run(self, gh) -> None:
        if self.argv:
            gh(self.argv)


def update_argv(repo: str, number: int, head_sha: str) -> list:
    # expected_head_sha: if the head moved between observation and this call, GitHub
    # answers 422 instead of updating a branch we never assessed. apply() classifies
    # that specific race as a warning ("caught next tick"); any other failure — auth,
    # permissions, validation, rate limits — fails the run (see apply()).
    return ["api", "-X", "PUT", f"repos/{repo}/pulls/{number}/update-branch",
            "-f", f"expected_head_sha={head_sha}"]


# --------------------------------------------------------------------------------------
# THE POLICY (pure). Each pr dict carries:
#   number:int  is_draft:bool  labels:[str]  armed:bool  merge_state:str  head_sha:str
#   base_ref:str
# and, for PRs that survive the cheap guards (enriched live only for those):
#   behind_by:int  gate:'failure'|'success'|'pending'|'missing'
#   active_runs:int  head_committed_at:iso-str
# Guard order matters: cheap label/state guards fire before enrichment fields are read,
# so live mode may leave enrichment fields absent on cheap-skipped PRs.
# --------------------------------------------------------------------------------------
def plan(prs, repo, now, cap=MAX_UPDATES_PER_TICK):
    actions = []
    selected = 0
    for pr in prs:
        num = pr["number"]
        labels = [l.lower() for l in pr.get("labels", [])]

        # (a)/(d) cheap guards — no enrichment needed.
        if pr.get("is_draft"):
            actions.append(Action("skip", num, "draft"))
            continue
        if BLOCK_LABEL in labels:
            actions.append(Action("skip", num, f"label {BLOCK_LABEL}"))
            continue
        if not (pr.get("armed") or ARMED_LABEL in labels):
            actions.append(Action("skip", num, f"not armed and no {ARMED_LABEL} label"))
            continue
        if (pr.get("merge_state") or "").upper() == "DIRTY":
            actions.append(Action("dirty", num,
                                  "merge conflict — needs sparq-merge-fixer, not update-branch"))
            continue
        # update-branch merges the PR's BASE branch, not `main` — a stacked PR (base !=
        # main) must never qualify: its behind-MAIN measurement is meaningless and the
        # call would only burn the per-tick cap on 422s / merge the wrong branch.
        base = pr.get("base_ref") or ""
        if base != "main":
            actions.append(Action("skip", num, f"base {base or '?'} is not main"))
            continue
        # update-branch needs write access to the HEAD repository, which the App token has only
        # for this repo. A fork PR would 403 every tick while consuming the cap.
        head_repo = pr.get("head_repo") or ""
        if head_repo != repo:
            actions.append(Action("skip", num,
                                  f"head repo {head_repo or '?'} is not {repo} (fork)"))
            continue

        # A guard we could not EVALUATE is not a guard that passed. Every enrichment default
        # used to fail open (see enrich) — this is the single place that decision now lands.
        if pr.get("observation_error"):
            actions.append(Action("skip", num, f"unobservable: {pr['observation_error']}"))
            continue

        # (b) behind main.
        if int(pr.get("behind_by") or 0) <= 0:
            actions.append(Action("skip", num, "not behind main"))
            continue

        # (c) gate red or absent — a green/converging gate is left alone.
        gate = pr.get("gate") or "missing"
        if gate == "success":
            actions.append(Action("skip", num, "gate is green"))
            continue
        if gate == "pending":
            actions.append(Action("skip", num, "fresh CI in progress (gate not terminal)"))
            continue

        # Never churn a head whose fresh CI is already queued/running (a just-created
        # run may not have registered its `gate` check-run yet — the runs list catches it).
        if int(pr.get("active_runs") or 0) > 0:
            actions.append(Action("skip", num, "fresh CI in progress (runs on head)"))
            continue

        # COOLDOWN (see header): head commit younger than COOLDOWN_MIN => recently
        # updated/pushed; let that CI cycle land before touching the branch again.
        committed = pr.get("head_committed_at")
        if committed and now - _parse_iso(committed) < timedelta(minutes=COOLDOWN_MIN):
            actions.append(Action("skip", num, f"cooldown (<{COOLDOWN_MIN}m since last update)"))
            continue

        # CAP — skip WITH a log line, never silently (congestion-collapse doctrine).
        if selected >= cap:
            actions.append(Action("skip", num, f"truncated (cap {cap}/tick)"))
            continue

        selected += 1
        actions.append(Action("update", num, f"behind + gate {gate} -> update-branch",
                              update_argv(repo, num, pr.get("head_sha", ""))))

    truncated = sum(1 for a in actions if a.reason.startswith("truncated"))
    if truncated:
        log(f"cap {cap}/tick reached — {truncated} eligible PR(s) deferred to the next tick")
    return actions


# --------------------------------------------------------------------------------------
# LIVE data collection (real gh). Not exercised by the self-test (gh stubbed there).
# --------------------------------------------------------------------------------------
def _gh_json(argv):
    out = subprocess.check_output(["gh"] + argv, text=True)
    return json.loads(out)


def collect_prs(repo, gh_json=_gh_json):
    prs = gh_json([
        "pr", "list", "--repo", repo, "--state", "open", "--limit", "200",
        "--json", "number,isDraft,labels,autoMergeRequest,mergeStateStatus,headRefOid,"
                  "baseRefName,headRepository,headRepositoryOwner",
    ])
    norm = []
    for p in prs:
        # A fork PR's head lives in another repository. `update-branch` requires write access
        # to the HEAD repo, which the App token does not have, so such a PR 403s on every tick
        # while consuming the per-tick cap and starving PRs we can actually refresh. Build the
        # full name and let the cheap guards drop it; an unreadable owner/name yields "" and is
        # therefore also dropped, which is the safe direction.
        owner = ((p.get("headRepositoryOwner") or {}).get("login") or "")
        name = ((p.get("headRepository") or {}).get("name") or "")
        norm.append({
            "number": p["number"],
            "is_draft": bool(p.get("isDraft")),
            "labels": [l.get("name", "") for l in (p.get("labels") or [])],
            "armed": p.get("autoMergeRequest") is not None,
            "merge_state": p.get("mergeStateStatus", ""),
            "head_sha": p.get("headRefOid", ""),
            "base_ref": p.get("baseRefName", ""),
            "head_repo": f"{owner}/{name}" if owner and name else "",
        })
    return norm


def _cheaply_eligible(pr, repo) -> bool:
    """Mirror of plan()'s cheap guards — decides which PRs are worth 3 enrichment calls."""
    labels = [l.lower() for l in pr.get("labels", [])]
    return (not pr.get("is_draft")
            and BLOCK_LABEL not in labels
            and (pr.get("armed") or ARMED_LABEL in labels)
            and (pr.get("merge_state") or "").upper() != "DIRTY"
            and (pr.get("base_ref") or "") == "main"
            and (pr.get("head_repo") or "") == repo)


def enrich(repo, pr, gh_json=_gh_json):
    """3 API calls: behind-ness + head-commit time (one compare), gate check, active runs."""
    sha = pr["head_sha"]

    # compare(head...main): ahead_by == commits main has that head lacks (= behind-ness),
    # and base_commit == the head commit itself, carrying the committer timestamp the
    # cooldown reads. One call, two signals.
    # OBSERVATION vs ABSENCE. Every default below used to collapse "the API did not tell us"
    # into the value that PERMITS the mutation: a missing commit date disabled the cooldown
    # entirely, a payload without `check_runs` became gate="missing" (an ELIGIBLE state), and a
    # payload without `workflow_runs` became zero active runs. All three fail OPEN. So each
    # unreadable payload now records an observation_error and plan() skips the PR — a guard we
    # could not evaluate is not a guard that passed.
    pr["observation_error"] = ""

    def unobservable(what):
        if not pr["observation_error"]:
            pr["observation_error"] = what

    cmp_ = gh_json(["api", f"repos/{repo}/compare/{sha}...main"])
    if not isinstance(cmp_, dict) or "ahead_by" not in cmp_:
        unobservable("compare payload unreadable")
        cmp_ = {}
    pr["behind_by"] = int(cmp_.get("ahead_by") or 0)
    pr["head_committed_at"] = (((cmp_.get("base_commit") or {}).get("commit") or {})
                               .get("committer") or {}).get("date")
    if not pr["head_committed_at"]:
        # Without it the cooldown cannot be evaluated at all, and the old code simply skipped
        # the check — refreshing a head that may have been pushed seconds ago.
        unobservable("head commit date missing — cooldown unevaluable")

    # Latest check-run named exactly `gate` on the head (draft-tier runs render a
    # different name, `gate, draft-tier`, and drafts are excluded anyway).
    checks = gh_json(["api", f"repos/{repo}/commits/{sha}/check-runs?per_page=100"])
    if not isinstance(checks, dict) or not isinstance(checks.get("check_runs"), list):
        unobservable("check-runs payload unreadable")
        checks = {"check_runs": []}
    gates = sorted((c for c in checks.get("check_runs", [])
                    if (c.get("name") or "").strip().lower() == "gate"),
                   key=lambda c: c.get("started_at") or "")
    if not gates:
        pr["gate"] = "missing"
    else:
        g = gates[-1]
        if (g.get("status") or "").lower() != "completed":
            pr["gate"] = "pending"
        else:
            concl = (g.get("conclusion") or "").lower()
            pr["gate"] = ("failure" if concl in _GATE_RED
                          else "success" if concl in _GATE_GREEN
                          else "pending")

    # ANY status other than `completed` counts as active — the API also surfaces
    # `requested` and `pending` before a run is queued, and a head in any pre-completion
    # state may already be recomputing the verdict. Over-counting merely defers the PR
    # one tick (fail-safe); under-counting churns a converging head — the exact race
    # this guard exists to prevent.
    #
    # QUANTIFIER: the obligation is "NO active run exists" — universal. Observing zero
    # non-completed runs in ONE page does not establish it, and the old code scanned a single
    # unpaginated page of 30, so an older active run on a re-run head sorted past the page
    # boundary read as "no active runs" and the head got churned. Presence is easy to prove and
    # absence is not, so: any non-completed run found => active (skip); zero found on a
    # TRUNCATED listing => unobservable (skip); zero found on a complete listing => genuinely
    # idle. Only the last of those permits the refresh.
    runs = gh_json(["api",
                    f"repos/{repo}/actions/runs?head_sha={sha}&per_page={_RUNS_PAGE}"])
    if not isinstance(runs, dict) or not isinstance(runs.get("workflow_runs"), list):
        unobservable("workflow-runs payload unreadable")
        pr["active_runs"] = 0
        return pr
    listed = runs["workflow_runs"]
    pr["active_runs"] = sum(1 for r in listed
                            if (r.get("status") or "").lower() != "completed")
    total = runs.get("total_count")
    if pr["active_runs"] == 0 and isinstance(total, int) and total > len(listed):
        unobservable(f"workflow-run listing truncated ({len(listed)} of {total}) — "
                     f"cannot prove no active run")
    return pr


# --------------------------------------------------------------------------------------
# Step-summary table (GitHub Actions job summary, or stdout locally).
# --------------------------------------------------------------------------------------
def render_summary(actions) -> str:
    lines = [
        "## pr-freshness refresher",
        "",
        "| PR | action | reason |",
        "| --- | --- | --- |",
    ]
    for a in actions:
        lines.append(f"| #{a.pr} | {a.kind} | {a.reason} |")
    dirty = [a.pr for a in actions if a.kind == "dirty"]
    if dirty:
        lines += ["", "**DIRTY (conflict) PRs for sparq-merge-fixer:** "
                  + ", ".join(f"#{n}" for n in dirty)]
    lines.append("")
    return "\n".join(lines)


def real_gh(argv) -> None:
    # capture_output so a failure carries the HTTP status/message for apply()'s
    # benign-race vs real-failure classification.
    subprocess.run(["gh"] + argv, check=True, capture_output=True, text=True)


def _is_head_race(err: subprocess.CalledProcessError) -> bool:
    """True iff the failure is the benign expected_head_sha race (the head moved between
    observation and the PUT) — the ONLY failure that may stay a warning. A 422 is NOT
    proof of that race (it also covers validation and secondary-rate-limit failures), so
    this matches GitHub's specific race message; anything else is an unexpected failure."""
    text = f"{getattr(err, 'stdout', '') or ''}\n{getattr(err, 'stderr', '') or ''}".lower()
    return "http 422" in text and "expected head sha" in text


def apply(actions, gh):
    """Run the update argvs through gh. A per-PR failure never aborts the remaining PRs,
    but only the expected_head_sha race is benign — every other failure (auth, token
    permissions, validation, rate limits) is counted so main() can fail the run: a broken
    token must not leave the workflow green while updating nothing.
    Returns (applied, races, failures)."""
    applied = races = failures = 0
    for a in actions:
        if not a.argv:
            continue
        try:
            a.run(gh)
            applied += 1
        except subprocess.CalledProcessError as e:
            if _is_head_race(e):
                races += 1
                log(f"WARNING: PR #{a.pr} lost the expected_head_sha race — "
                    "self-heals next tick; continuing.")
            else:
                failures += 1
                detail = (getattr(e, "stderr", "") or getattr(e, "stdout", "") or str(e)).strip()
                log(f"ERROR: update-branch failed for PR #{a.pr}: {detail} — continuing.")
    return applied, races, failures


def main() -> int:
    ap = argparse.ArgumentParser(description="PR-freshness refresher (pure policy).")
    ap.add_argument("--repo", help="owner/repo (required unless --self-test).")
    ap.add_argument("--report-only", action="store_true",
                    help="Fetch state + print the plan/summary, run NO gh mutations. "
                         "(The workflow forces this when no App token is available: an "
                         "update-branch push made with GITHUB_TOKEN fires NO events, so "
                         "the fresh CI it exists to trigger would never run.)")
    ap.add_argument("--self-test", action="store_true",
                    help="Run the hermetic self-test (gh stubbed) and exit.")
    ap.add_argument("--summary-file", default=None,
                    help="Write the step-summary table here (e.g. $GITHUB_STEP_SUMMARY).")
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if not args.repo:
        raise SystemExit(f"[{PROG}] ERROR: --repo is required (unless --self-test).")

    prs = collect_prs(args.repo)
    for pr in prs:
        if _cheaply_eligible(pr, args.repo):
            enrich(args.repo, pr)
    actions = plan(prs, args.repo, datetime.now(timezone.utc))

    summary = render_summary(actions)
    if args.summary_file:
        with open(args.summary_file, "a", encoding="utf-8") as f:
            f.write(summary + "\n")
    print(summary)

    if args.report_only:
        log("--report-only: no gh mutations performed.")
        return 0

    applied, races, failures = apply(actions, real_gh)
    log(f"update-branch: {applied} applied, {races} lost head race(s), "
        f"{failures} unexpected failure(s).")
    if failures:
        log("ERROR: unexpected update-branch failure(s) above — failing the run so a "
            "broken token/permission set cannot stay green while updating nothing.")
        return 1
    return 0


# --------------------------------------------------------------------------------------
# Hermetic self-test. gh is a recording stub — NO live mutation is possible. Every guard
# has a fixture that is selectable EXCEPT for that one guard, so deleting any guard turns
# at least one check red (non-vacuity).
# --------------------------------------------------------------------------------------
def self_test() -> int:
    fails = 0

    def check(label, cond):
        nonlocal fails
        if cond:
            print(f"  ok   {label}")
        else:
            print(f"  FAIL {label}")
            fails += 1

    log("running --self-test (hermetic; gh STUBBED — no live mutation possible)")

    now = datetime(2026, 7, 17, 23, 30, 0, tzinfo=timezone.utc)
    repo = "sparq-org/sparq"

    def ago(minutes):
        return (now - timedelta(minutes=minutes)).strftime("%Y-%m-%dT%H:%M:%SZ")

    def fx(number, **over):
        """Baseline: fully eligible for update. Each fixture flips exactly one field."""
        base = {"number": number, "is_draft": False, "labels": [], "armed": True,
                "merge_state": "BLOCKED", "head_sha": f"sha{number}", "base_ref": "main",
                "behind_by": 5, "gate": "failure", "active_runs": 0,
                "head_committed_at": ago(120), "head_repo": repo,
                "observation_error": ""}
        base.update(over)
        return base

    fixtures = [
        fx(1),                                                # armed+behind+red -> UPDATE
        fx(2, labels=["review:needs-user"]),                  # (d) label -> skip
        fx(3, gate="pending"),                                # in-progress gate -> skip
        fx(4, gate="missing", active_runs=2),                 # in-progress runs -> skip
        fx(5, head_committed_at=ago(10)),                     # cooldown -> skip
        fx(6, behind_by=0),                                   # (b) not behind -> skip
        fx(7, gate="success"),                                # (c) green -> skip
        fx(8, armed=False),                                   # (a) unarmed, no label -> skip
        fx(9, armed=False, labels=["review:pass"]),           # (a) label arm -> UPDATE
        fx(10, merge_state="DIRTY"),                          # (d) conflict -> dirty report
        fx(11, is_draft=True),                                # (a) draft -> skip
        fx(12, gate="missing"),                               # (c) no run on merge ref -> UPDATE
        fx(13, base_ref="feature/stack-base"),                # (a) stacked PR -> skip
    ]
    # Cap fixtures: 9 more fully-eligible PRs; with the 3 updates above, only 6 total
    # may update and the rest must be explicit "truncated" skips.
    fixtures += [fx(100 + i) for i in range(9)]

    actions = plan(fixtures, repo, now)
    by_pr = {a.pr: a for a in actions}

    def kind(n):
        return by_pr[n].kind

    def reason(n):
        return by_pr[n].reason

    # Selection + argv shape.
    check("armed+behind+red #1 -> update", kind(1) == "update")
    a1 = by_pr[1].argv
    check("#1 argv is PUT pulls/1/update-branch",
          a1[:3] == ["api", "-X", "PUT"] and f"repos/{repo}/pulls/1/update-branch" in a1)
    check("#1 argv pins expected_head_sha", "expected_head_sha=sha1" in a1)
    check("review:pass label alone #9 -> update", kind(9) == "update")
    check("missing gate (no run on merge ref) #12 -> update", kind(12) == "update")

    # Guards — each of these is red if its guard is deleted (the fixture is otherwise
    # fully eligible, so it would fall through to "update").
    check("review:needs-user #2 -> skip", kind(2) == "skip" and "review:needs-user" in reason(2))
    check("pending gate #3 -> skip (in progress)", kind(3) == "skip" and "in progress" in reason(3))
    check("active runs #4 -> skip (in progress)", kind(4) == "skip" and "runs on head" in reason(4))
    check("cooldown #5 -> skip", kind(5) == "skip" and "cooldown" in reason(5))
    check("not behind #6 -> skip", kind(6) == "skip" and "not behind" in reason(6))
    check("green gate #7 -> skip", kind(7) == "skip" and "green" in reason(7))
    check("unarmed #8 -> skip", kind(8) == "skip" and "not armed" in reason(8))
    check("DIRTY #10 -> dirty report, no argv",
          kind(10) == "dirty" and not by_pr[10].argv and "merge-fixer" in reason(10))
    check("draft #11 -> skip", kind(11) == "skip" and reason(11) == "draft")
    check("stacked (base != main) #13 -> skip",
          kind(13) == "skip" and "base feature/stack-base is not main" in reason(13))

    # Cap: 12 eligible in total (1, 9, 12, 100..108) but only 6 update; the other 6 are
    # explicit truncated skips (never silent).
    updates = [a for a in actions if a.kind == "update"]
    truncs = [a for a in actions if a.reason.startswith("truncated")]
    check(f"cap: exactly {MAX_UPDATES_PER_TICK} updates emitted", len(updates) == MAX_UPDATES_PER_TICK)
    check("cap: 6 eligible PRs explicitly truncated", len(truncs) == 6)
    check("cap: truncated rows carry the cap in the reason",
          all(f"cap {MAX_UPDATES_PER_TICK}/tick" in a.reason for a in truncs))

    # Only update rows mutate; skip/dirty rows carry no argv.
    check("only update rows carry argv",
          all(bool(a.argv) == (a.kind == "update") for a in actions))

    # gh stub records exactly the update argv — and nothing on a re-run inside cooldown.
    recorded = []
    for a in actions:
        a.run(recorded.append)
    check("gh stub recorded exactly the update calls", len(recorded) == len(updates))

    # Immediately-after re-run: every just-updated PR now has a fresh head commit
    # (cooldown) — the plan must emit ZERO updates for them (loop-safety in miniature).
    rerun_fixtures = [fx(a.pr, head_committed_at=ago(1), gate="missing") for a in updates]
    rerun = plan(rerun_fixtures, repo, now)
    check("re-run within cooldown emits zero updates",
          all(a.kind == "skip" and "cooldown" in a.reason for a in rerun))

    # Summary renders a row per action + the merge-fixer hand-off line.
    summary = render_summary(actions)
    check("summary is a markdown table", "| PR | action | reason |" in summary)
    check("summary lists DIRTY PRs for the merge-fixer",
          "sparq-merge-fixer" in summary and "#10" in summary)

    # ----------------------------------------------------------------------------------
    # apply(): a per-PR failure never aborts the batch, the expected_head_sha race is
    # the ONLY benign failure, and anything else is counted so main() exits non-zero
    # (a broken token must not stay green while updating nothing).
    # ----------------------------------------------------------------------------------
    apply_acts = [Action("update", 1, "t", update_argv(repo, 1, "s1")),
                  Action("update", 2, "t", update_argv(repo, 2, "s2")),
                  Action("update", 3, "t", update_argv(repo, 3, "s3")),
                  Action("skip", 4, "no argv — must not be attempted")]
    attempted = []

    def gh_mixed(argv):
        attempted.append(argv)
        if "/pulls/2/" in argv[3]:
            raise subprocess.CalledProcessError(
                1, argv, output="",
                stderr="gh: expected head sha didn't match current head ref. (HTTP 422)")
        if "/pulls/3/" in argv[3]:
            raise subprocess.CalledProcessError(
                1, argv, output="",
                stderr="gh: Resource not accessible by integration (HTTP 403)")

    applied_n, races_n, failures_n = apply(apply_acts, gh_mixed)
    check("apply: ok/race/failure classified as (1, 1, 1)",
          (applied_n, races_n, failures_n) == (1, 1, 1))
    check("apply: continues past failures (all 3 update argvs attempted)",
          len(attempted) == 3)
    ok_applied, ok_races, ok_failures = apply(apply_acts[:1], lambda argv: None)
    check("apply: clean run counts only real successes",
          (ok_applied, ok_races, ok_failures) == (1, 0, 0))
    check("head-race classifier: 422 alone is NOT benign (validation/rate-limit 422s "
          "must fail the run)",
          not _is_head_race(subprocess.CalledProcessError(
              1, ["gh"], output="", stderr="gh: Validation Failed (HTTP 422)")))

    # ----------------------------------------------------------------------------------
    # Raw-API-boundary tests: drive collect_prs() -> _cheaply_eligible() -> enrich()
    # -> plan() from RAW gh JSON (gh stubbed), so mutations to the normalization, base
    # selection, ahead_by wiring, gate classification, or run-status counting go red —
    # not just mutations to plan() over pre-normalized fixtures.
    # ----------------------------------------------------------------------------------
    raw_pr_list = [
        # armed via label (case-insensitive), red gate, behind -> must reach `update`.
        {"number": 21, "isDraft": False, "labels": [{"name": "Review:Pass"}],
         "autoMergeRequest": None, "mergeStateStatus": "BLOCKED",
         "headRefOid": "sha21", "baseRefName": "main"},
        # armed via autoMergeRequest, gate missing, but nonterminal runs on the head.
        {"number": 22, "isDraft": False, "labels": [], "autoMergeRequest": {"enabledAt": "x"},
         "mergeStateStatus": "BEHIND", "headRefOid": "sha22", "baseRefName": "main"},
        # stacked PR: base != main — must be rejected before spending enrichment calls.
        {"number": 23, "isDraft": False, "labels": [], "autoMergeRequest": {"enabledAt": "x"},
         "mergeStateStatus": "BLOCKED", "headRefOid": "sha23", "baseRefName": "release/stack"},
        # FORK head: otherwise fully eligible. update-branch needs write on the HEAD repo, so
        # this 403s every tick while consuming the cap. Must be rejected by the cheap guards,
        # BEFORE any enrichment call is spent on it.
        {"number": 26, "isDraft": False, "labels": [{"name": "review:pass"}],
         "autoMergeRequest": None, "mergeStateStatus": "BLOCKED",
         "headRefOid": "sha26", "baseRefName": "main",
         "headRepository": {"name": "sparq"},
         "headRepositoryOwner": {"login": "some-contributor"}},
    ]
    # Same-repo head for every fixture that is meant to be eligible. Injected rather than
    # written inline so a future fixture cannot silently omit it and read as a fork.
    _owner, _name = repo.split("/", 1)
    for _raw in raw_pr_list:
        _raw.setdefault("headRepository", {"name": _name})
        _raw.setdefault("headRepositoryOwner", {"login": _owner})
    compare_by_sha = {
        "sha21": {"ahead_by": 4,
                  "base_commit": {"commit": {"committer": {"date": ago(120)}}}},
        "sha22": {"ahead_by": 3,
                  "base_commit": {"commit": {"committer": {"date": ago(120)}}}},
        "sha24": {"ahead_by": 1,
                  "base_commit": {"commit": {"committer": {"date": ago(120)}}}},
        "sha25": {"ahead_by": 1,
                  "base_commit": {"commit": {"committer": {"date": ago(120)}}}},
    }
    checkruns_by_sha = {
        # Older gate green, newer gate red, plus a non-gate run: latest-by-started_at
        # must win and non-gate names must be ignored.
        "sha21": {"check_runs": [
            {"name": "clippy", "status": "completed", "conclusion": "success",
             "started_at": ago(90)},
            {"name": "gate", "status": "completed", "conclusion": "success",
             "started_at": ago(200)},
            {"name": "gate", "status": "completed", "conclusion": "failure",
             "started_at": ago(100)},
        ]},
        "sha22": {"check_runs": [
            {"name": "clippy", "status": "completed", "conclusion": "success",
             "started_at": ago(90)},
        ]},
        # Name matching is strip()+case-insensitive; `cancelled` is a red conclusion.
        "sha25": {"check_runs": [
            {"name": "GATE ", "status": "completed", "conclusion": "cancelled",
             "started_at": ago(100)},
        ]},
        # A gate that has not completed is `pending`, whatever its conclusion field says.
        "sha24": {"check_runs": [
            {"name": "gate", "status": "in_progress", "conclusion": None,
             "started_at": ago(5)},
        ]},
    }
    runs_by_sha = {
        "sha21": {"workflow_runs": [{"status": "completed"}]},
        # `requested` and `pending` are nonterminal — both MUST count as active
        # (reverting to a queued/in_progress/waiting allowlist turns this red).
        "sha22": {"workflow_runs": [{"status": "requested"}, {"status": "pending"},
                                    {"status": "completed"}]},
        "sha24": {"workflow_runs": []},
        "sha25": {"workflow_runs": []},
    }
    enrich_calls = []

    def stub_gh_json(argv):
        if argv[:2] == ["pr", "list"]:
            return raw_pr_list
        path = argv[1]
        if "/compare/" in path:
            sha = path.split("/compare/")[1].split("...")[0]
            enrich_calls.append(sha)
            return compare_by_sha[sha]
        if "check-runs" in path:
            return checkruns_by_sha[path.split("/commits/")[1].split("/")[0]]
        if "actions/runs" in path:
            return runs_by_sha[path.split("head_sha=")[1].split("&")[0]]
        raise AssertionError(f"unexpected gh argv in stub: {argv}")

    norm = collect_prs(repo, gh_json=stub_gh_json)
    by_num = {p["number"]: p for p in norm}
    check("collect_prs: armed derived from autoMergeRequest",
          not by_num[21]["armed"] and by_num[22]["armed"])
    check("collect_prs: label names flattened", by_num[21]["labels"] == ["Review:Pass"])
    check("collect_prs: baseRefName normalized to base_ref",
          by_num[22]["base_ref"] == "main" and by_num[23]["base_ref"] == "release/stack")
    check("cheap-eligible: label-armed and automerge-armed main PRs pass",
          _cheaply_eligible(by_num[21], repo) and _cheaply_eligible(by_num[22], repo))
    check("cheap-eligible: stacked (base != main) PR rejected before enrichment",
          not _cheaply_eligible(by_num[23], repo))

    enriched = [enrich(repo, p, gh_json=stub_gh_json) if _cheaply_eligible(p, repo) else p
                for p in norm]
    check("enrich: stacked PR spent zero API calls", "sha23" not in enrich_calls)
    check("enrich: behind_by wired from compare.ahead_by", by_num[21]["behind_by"] == 4)
    check("enrich: cooldown timestamp from compare.base_commit committer date",
          by_num[21]["head_committed_at"] == ago(120))
    check("enrich: latest gate check-run wins, non-gate names ignored",
          by_num[21]["gate"] == "failure")
    check("enrich: no gate check-run -> missing", by_num[22]["gate"] == "missing")
    check("enrich: requested/pending runs count as active",
          by_num[22]["active_runs"] == 2)
    p24 = enrich(repo, {"head_sha": "sha24"}, gh_json=stub_gh_json)
    check("enrich: non-completed gate -> pending", p24["gate"] == "pending")
    p25 = enrich(repo, {"head_sha": "sha25"}, gh_json=stub_gh_json)
    check("enrich: 'GATE ' matches (strip+casefold) and cancelled is red",
          p25["gate"] == "failure")

    raw_actions = {a.pr: a for a in plan(enriched, repo, now)}
    check("raw path: #21 (behind + red gate) -> update with pinned head sha",
          raw_actions[21].kind == "update"
          and "expected_head_sha=sha21" in raw_actions[21].argv)
    check("raw path: #22 skipped for nonterminal runs on head",
          raw_actions[22].kind == "skip" and "runs on head" in raw_actions[22].reason)
    check("raw path: #23 skipped for non-main base",
          raw_actions[23].kind == "skip" and "is not main" in raw_actions[23].reason)
    check("raw path: FORK head #26 is skipped, and NOT at the cost of enrichment calls "
          "(update-branch has no write access to another repo — it would 403 every tick "
          "while consuming the per-tick cap)",
          raw_actions[26].kind == "skip"
          and "fork" in raw_actions[26].reason
          and "sha26" not in enrich_calls)

    # ---- FAIL-CLOSED OBSERVATION (cross-provider review of this PR, finding 2) -------------
    # Each default below used to collapse "the API did not tell us" into the value that PERMITS
    # the refresh. Asserted against the SAME baseline fixture that DOES update, so these cannot
    # pass merely because the fixture was ineligible for some other reason.
    check("baseline for the unobservable checks below really does update",
          plan([fx(70)], repo, now)[0].kind == "update")
    for label, over in [
        ("compare payload unreadable", {"observation_error": "compare payload unreadable"}),
        ("head commit date missing (cooldown UNEVALUABLE, previously skipped outright)",
         {"observation_error": "head commit date missing — cooldown unevaluable"}),
        ("check-runs payload unreadable (previously became gate=missing, an ELIGIBLE state)",
         {"observation_error": "check-runs payload unreadable"}),
        ("workflow-runs payload unreadable (previously became 0 active runs)",
         {"observation_error": "workflow-runs payload unreadable"}),
    ]:
        act = plan([fx(71, **over)], repo, now)[0]
        check(f"unobservable -> skip: {label}",
              act.kind == "skip" and "unobservable" in act.reason)

    # enrich() must SET those errors from real payload shapes, not just honour them in plan().
    for label, payload_key, payload in [
        ("compare without ahead_by", "compare", {}),
        ("compare without a committer date", "compare", {"ahead_by": 2}),
        ("check-runs without the check_runs list", "checks", {"total_count": 0}),
        ("workflow-runs without the workflow_runs list", "runs", {"total_count": 0}),
    ]:
        def _stub(argv, _k=payload_key, _p=payload):
            path = argv[1]
            if "compare/" in path:
                return _p if _k == "compare" else {
                    "ahead_by": 2,
                    "base_commit": {"commit": {"committer": {"date": ago(120)}}}}
            if "check-runs" in path:
                return _p if _k == "checks" else {"check_runs": []}
            return _p if _k == "runs" else {"total_count": 0, "workflow_runs": []}
        got = enrich(repo, {"head_sha": "shaX"}, gh_json=_stub)
        check(f"enrich records an observation error: {label}",
              bool(got.get("observation_error")))

    # QUANTIFIER (finding 6): "no active run exists" is UNIVERSAL. Zero found in a TRUNCATED
    # page does not establish it — the old code scanned one unpaginated page of 30.
    def _trunc_stub(argv):
        path = argv[1]
        if "compare/" in path:
            return {"ahead_by": 2,
                    "base_commit": {"commit": {"committer": {"date": ago(120)}}}}
        if "check-runs" in path:
            return {"check_runs": []}
        return {"total_count": 250, "workflow_runs": [
            {"status": "completed"} for _ in range(100)]}
    trunc = enrich(repo, {"head_sha": "shaT"}, gh_json=_trunc_stub)
    check("truncated run listing with zero active runs is UNOBSERVABLE, not idle "
          "(cannot prove absence from one page)",
          "truncated" in (trunc.get("observation_error") or ""))
    check("...and plan() therefore skips it rather than churning the head",
          plan([fx(72, **{k: trunc[k] for k in ('observation_error',)})],
               repo, now)[0].kind == "skip")

    def _complete_stub(argv):
        path = argv[1]
        if "compare/" in path:
            return {"ahead_by": 2,
                    "base_commit": {"commit": {"committer": {"date": ago(120)}}}}
        if "check-runs" in path:
            return {"check_runs": []}
        return {"total_count": 3, "workflow_runs": [{"status": "completed"}] * 3}
    complete = enrich(repo, {"head_sha": "shaC"}, gh_json=_complete_stub)
    check("a COMPLETE listing with zero active runs is genuinely idle (the guard must not "
          "become a blanket refusal — otherwise the refresher never refreshes anything)",
          not complete.get("observation_error") and complete["active_runs"] == 0)

    print()
    if fails == 0:
        log("self-test PASSED")
        return 0
    log(f"self-test FAILED ({fails} check(s))")
    return 1


if __name__ == "__main__":
    sys.exit(main())
