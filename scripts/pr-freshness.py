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
#   (a) it is OPEN, NON-draft, and (auto-merge is armed OR it carries label `review:pass`);
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
# Exit 0 on success; non-zero only on a real error or a failed self-test.
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
    # answers 422 instead of updating a branch we never assessed. The runner treats a
    # per-PR gh failure as a warning, so a lost race degrades to "caught next tick".
    return ["api", "-X", "PUT", f"repos/{repo}/pulls/{number}/update-branch",
            "-f", f"expected_head_sha={head_sha}"]


# --------------------------------------------------------------------------------------
# THE POLICY (pure). Each pr dict carries:
#   number:int  is_draft:bool  labels:[str]  armed:bool  merge_state:str  head_sha:str
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


def collect_prs(repo):
    prs = _gh_json([
        "pr", "list", "--repo", repo, "--state", "open", "--limit", "200",
        "--json", "number,isDraft,labels,autoMergeRequest,mergeStateStatus,headRefOid",
    ])
    norm = []
    for p in prs:
        norm.append({
            "number": p["number"],
            "is_draft": bool(p.get("isDraft")),
            "labels": [l.get("name", "") for l in (p.get("labels") or [])],
            "armed": p.get("autoMergeRequest") is not None,
            "merge_state": p.get("mergeStateStatus", ""),
            "head_sha": p.get("headRefOid", ""),
        })
    return norm


def _cheaply_eligible(pr) -> bool:
    """Mirror of plan()'s cheap guards — decides which PRs are worth 3 enrichment calls."""
    labels = [l.lower() for l in pr.get("labels", [])]
    return (not pr.get("is_draft")
            and BLOCK_LABEL not in labels
            and (pr.get("armed") or ARMED_LABEL in labels)
            and (pr.get("merge_state") or "").upper() != "DIRTY")


def enrich(repo, pr):
    """3 API calls: behind-ness + head-commit time (one compare), gate check, active runs."""
    sha = pr["head_sha"]

    # compare(head...main): ahead_by == commits main has that head lacks (= behind-ness),
    # and base_commit == the head commit itself, carrying the committer timestamp the
    # cooldown reads. One call, two signals.
    cmp_ = _gh_json(["api", f"repos/{repo}/compare/{sha}...main"])
    pr["behind_by"] = int(cmp_.get("ahead_by") or 0)
    pr["head_committed_at"] = (((cmp_.get("base_commit") or {}).get("commit") or {})
                               .get("committer") or {}).get("date")

    # Latest check-run named exactly `gate` on the head (draft-tier runs render a
    # different name, `gate, draft-tier`, and drafts are excluded anyway).
    checks = _gh_json(["api", f"repos/{repo}/commits/{sha}/check-runs?per_page=100"])
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

    runs = _gh_json(["api", f"repos/{repo}/actions/runs?head_sha={sha}&per_page=30"])
    pr["active_runs"] = sum(1 for r in runs.get("workflow_runs", [])
                            if r.get("status") in ("queued", "in_progress", "waiting"))
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
    subprocess.run(["gh"] + argv, check=True)


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
        if _cheaply_eligible(pr):
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

    for a in actions:
        try:
            a.run(real_gh)
        except subprocess.CalledProcessError as e:
            # A 422 here is a lost expected_head_sha race or an already-updated branch —
            # both self-heal on the next tick. Never fail the whole run for one PR.
            log(f"WARNING: update-branch failed for PR #{a.pr}: {e} — continuing.")
    log(f"applied {sum(1 for a in actions if a.argv)} update-branch call(s).")
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
                "merge_state": "BLOCKED", "head_sha": f"sha{number}",
                "behind_by": 5, "gate": "failure", "active_runs": 0,
                "head_committed_at": ago(120)}
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

    print()
    if fails == 0:
        log("self-test PASSED")
        return 0
    log(f"self-test FAILED ({fails} check(s))")
    return 1


if __name__ == "__main__":
    sys.exit(main())
