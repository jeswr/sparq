#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: the readiness engine (replaces `bd ready` + push-frontier).
"""ready-issues.py — compute the dispatchable frontier from GitHub issues, FAIL-CLOSED.

Per the GPT-5.6 review (C1/S2), readiness requires POSITIVE, bot-attested state — never mere absence
of a quarantine label. An issue is READY iff, in priority order, ALL hold:
  * OPEN, and
  * carries `status:ready` (positive attestation the triage/trust pipeline set), and
  * carries exactly ONE valid `priority:P0..P4` (ambiguous/invalid priority → excluded), and
  * carries a `role:*` label, and
  * carries NO gate label (`needs:*`, `trust:untrusted`) and is NOT busy
    (`status:in-progress|blocked|deferred|untriaged`), and
  * has zero open blockers, and
  * none of its PACKAGES (`area:<crate>`) is already taken by an active open PR, an in-progress
    issue, or an earlier-selected ready issue. Human-parked artifacts (`needs:user`,
    `review:needs-user`, `status:blocked`) reserve nothing. A no-package / cross-cutting issue
    reserves a **global partition** that serializes it against ALL other work
    (shared lockfiles/CI/workspace configs).

The snapshot uses real cursor pagination (`gh api --paginate`) with an explicit fail-closed
ceiling; native GitHub dependencies remain a follow-up — today blockers come from validated
`Blocked-by: #NN` markers. Pure `compute_ready()` is unit-tested; the CLI wraps it over the
paginated fetch.
"""
import argparse
import json
import re
import subprocess
import sys

GATE_LABELS = ("needs:", "trust:untrusted")
# [OPUS-5] `status:in-progress-review` was MISSING here while the registry's dispatch.yml keeps
# such rows in `ready_input` precisely because it believes compute_ready() will (a) never select
# them and (b) still let them RESERVE their package. Neither held: the label is not a gate label,
# so an in-progress-review issue that also carried `status:ready` was SELECTABLE, and the reserve
# branch below only fired on `status:in-progress`, so its crate was left free for a second
# dispatch. Both halves fail OPEN into a double-dispatch; see IN_FLIGHT_STATUS below.
BUSY_STATUS = {"status:in-progress", "status:in-progress-review", "status:blocked",
               "status:deferred", "status:untriaged"}
# The statuses that make an OPEN ISSUE in-flight work occupying its area (not merely excluded).
IN_FLIGHT_STATUS = {"status:in-progress", "status:in-progress-review"}
# [GPT-5.6] Parked is not in-flight: these terminal, human-owned snapshot artifacts cannot
# advance autonomously, so they must not reserve an area indefinitely. Removing the label in a
# later snapshot restores occupancy immediately; there is no remembered park state.
PARKED_AREA_LABELS = {"needs:user", "review:needs-user", "status:blocked"}
# [OPUS-4.8] an epic is a tracking umbrella (its children are the work) — never dispatchable, even
# with a full ready label-set + zero blockers. Excluded here so a worker never "implements" an epic.
NON_DISPATCHABLE = "kind:epic"
GLOBAL = "__global__"  # the cross-cutting partition (serializes against everything)
_PRIO = re.compile(r"^priority:P([0-4])$")   # only P0..P4 are valid
_PKG = re.compile(r"^area:(.+)$")
_ROLE = re.compile(r"^role:.+$")


def labels_of(issue):
    return {lb["name"] if isinstance(lb, dict) else lb for lb in issue.get("labels", [])}


def valid_priority(labels):
    """Exactly one valid priority:P0..P4 → its int; zero or multiple or out-of-range → None."""
    ps = {int(m.group(1)) for lb in labels for m in [_PRIO.match(lb)] if m}
    return next(iter(ps)) if len(ps) == 1 else None


def declared_packages(labels):
    """The SET of area:<crate> packages ACTUALLY declared — empty when none are, no global fallback.

    [OPUS-5] Separated from `packages_of` because the empty→GLOBAL fallback is only sound for a
    CANDIDATE (a no-area issue is cross-cutting work that must serialize against everything).
    Applying it to an in-flight artifact inverts the meaning: it turns "we cannot attribute this
    to a crate" into "this seizes every crate". Nothing labels PRs with `area:` — all 60 open
    sparq PRs carried none — so the old rule let a single unlabelled PR hold __global__ and
    reduce the whole frontier to zero. See `_reserving_packages`.
    """
    return {m.group(1) for lb in labels for m in [_PKG.match(lb)] if m}


def packages_of(labels):
    """The SET of all area:<crate> packages; empty → the serializing global partition.

    CANDIDATE-side rule (fail-closed): an unlabelled issue is treated as cross-cutting.
    """
    return declared_packages(labels) or {GLOBAL}


def _reserving_packages(labels):
    """OCCUPANCY-side rule: an in-flight artifact reserves ONLY the areas it declares.

    [OPUS-5] The deliberate asymmetry with `packages_of`. Fail-closed on a candidate costs one
    dispatch; fail-closed on an occupant costs the ENTIRE fleet, because an unattributable
    occupant would serialize every package at once with no way for the pipeline to clear it
    (nothing in the pipeline applies `area:` labels to PRs). An occupant we cannot attribute
    therefore reserves nothing and is instead handled by the linked-issue suppression the
    registry planner applies (`Closes #N` / `sparq-agent/issue-N-*` heads).
    """
    return declared_packages(labels)


def has_role(labels):
    return any(_ROLE.match(lb) for lb in labels)


def is_gated(labels):
    return any(lb == g or lb.startswith(g) for lb in labels for g in GATE_LABELS)


def is_busy(labels):
    return bool(labels & BUSY_STATUS)


def is_parked(labels):
    return bool(labels & PARKED_AREA_LABELS)


def occupies_area(artifact):
    """Whether an otherwise in-flight PR/issue occupies its areas in this snapshot."""
    return not is_parked(labels_of(artifact))


def _artifact_name(artifact):
    if artifact is None:
        return "preseeded occupancy"
    kind = "pr" if "pull_request" in artifact else "issue"
    return f"{kind}#{artifact.get('number', '?')}"


def exclusion_reason(labels, open_blockers=0):
    """Why this label-set is NOT an enumerable ready candidate, or None when it is.

    Label/state gates ONLY — package serialization is a separate, capacity-shaped question
    answered by compute_ready(). Shared by ready_candidates() and the --diagnose taxonomy so the
    two can never drift apart.
    """
    if "status:ready" not in labels:
        return "no status:ready attestation"
    if NON_DISPATCHABLE in labels:
        return f"{NON_DISPATCHABLE} tracking umbrella"
    gates = sorted(lb for lb in labels for g in GATE_LABELS if lb == g or lb.startswith(g))
    if gates:
        return f"gated by {gates[0]}"
    busy = sorted(labels & BUSY_STATUS)
    if busy:
        return f"busy: {busy[0]}"
    parked = sorted(labels & PARKED_AREA_LABELS)
    if parked:
        return f"parked: {parked[0]}"
    if valid_priority(labels) is None:
        return "no single valid priority:P0..P4"
    if not has_role(labels):
        return "no role:* label"
    if int(open_blockers) > 0:
        return f"{int(open_blockers)} open blocker(s)"
    return None


def ready_candidates(issues, log=None):
    """The DRAINABLE backlog: every open issue whose LABELS make it dispatchable.

    [OPUS-5] Distinct from compute_ready(), which additionally serializes to one issue per
    package and therefore answers a CONCURRENCY question, not a "how much work is available"
    question. Conflating the two makes a healthy 200-item backlog behind a 4-wide frontier
    indistinguishable from an empty one. Returns [(priority, number, issue, packages)].

    `log`, when supplied, receives one attributable line per issue that carries the
    `status:ready` attestation and was nonetheless dropped — a silent `continue` here is what
    lets a label-regressed issue leave the frontier forever with zero signal. Issues without
    the attestation are NOT logged (they are not candidates and would flood the log).
    """
    out = []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN":
            continue
        if "pull_request" in it:             # PRs reserve work; they are never issue candidates
            continue
        L = labels_of(it)
        reason = exclusion_reason(L, it.get("open_blockers", 0))
        if reason is not None:
            if log is not None and "status:ready" in L:
                log(f"defer #{it.get('number', '?')}: {reason}")
            continue
        out.append((valid_priority(L), it.get("number", 0), it, packages_of(L)))
    return out


def roleless_ready(issues):
    """The SILENT-INVISIBILITY class: open, attested, un-gated, un-blocked — and yet NO `role:*`.

    [OPUS-5] Ported from the registry planner (its issue #225, where 117 accumulated unnoticed).
    `ready_candidates` drops these BEFORE any plan row exists, so they appear in no plan and in
    no diagnostic and drain never. The fail-closed drop is CORRECT — a role is never guessed —
    but its silence was not, so callers report this count loudly. Pure; returns sorted numbers.
    """
    numbers = []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN" or "pull_request" in it:
            continue
        L = labels_of(it)
        if "status:ready" not in L or NON_DISPATCHABLE in L:
            continue
        if is_gated(L) or is_busy(L) or is_parked(L):
            continue
        if int(it.get("open_blockers", 0)) > 0:
            continue
        if not has_role(L):
            numbers.append(it.get("number", 0))
    return sorted(numbers)


def compute_ready(issues, in_progress_packages=None, conflict_log=None):
    """Conflict-free, priority-ordered, FAIL-CLOSED CONCURRENCY FRONTIER.

    This is the one-per-package concurrency WIDTH, not the size of the drainable backlog — use
    `ready_candidates()` for the latter. compute_ready() ⊆ ready_candidates() always.

    `conflict_log`, when supplied, receives one attribution line per conflict-excluded candidate;
    the live default writes those diagnostics to stderr without polluting the frontier rows.
    """
    blockers = {}

    def reserve(pkgs, artifact):
        for pkg in sorted(pkgs):
            blockers.setdefault(pkg, []).append(artifact)

    def conflict(pkgs):
        if GLOBAL in blockers:
            area = GLOBAL
        elif GLOBAL in pkgs and blockers:
            area = sorted(blockers)[0]
        else:
            overlap = pkgs & blockers.keys()
            if not overlap:
                return None
            area = sorted(overlap)[0]
        return area, blockers[area][0]

    for pkg in sorted(set(in_progress_packages or ())):
        reserve({pkg}, None)
    # [GPT-5.6] Every active open PR is in flight (drafts included); open issues occupy only while
    # status:in-progress. The shared parked predicate is applied before either can reserve areas.
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN" or not occupies_area(it):
            continue
        L = labels_of(it)
        if "pull_request" in it or L & IN_FLIGHT_STATUS:
            reserve(_reserving_packages(L), it)
    cands = ready_candidates(issues)
    cands.sort(key=lambda c: (c[0], c[1]))   # priority then number (deterministic)
    ready = []
    for _p, _n, it, pkgs in cands:
        held = conflict(pkgs)
        if held is not None:
            area, blocker = held
            message = (f"conflict #{it.get('number', '?')}: area {area} held by "
                       f"{_artifact_name(blocker)}")
            if conflict_log is None:
                print(message, file=sys.stderr)
            else:
                conflict_log(message)
            continue
        reserve(pkgs, it)
        ready.append(it)
    return ready


def _self_test():
    def iss(n, labels, blk=0, state="OPEN"):
        return {"number": n, "state": state, "labels": labels, "open_blockers": blk}

    R = ["status:ready", "role:impl"]

    def quiet(_message):
        pass

    F = [
        iss(1, R + ["priority:P2", "area:sparq-core"]),
        iss(2, R + ["priority:P0", "area:sparq-core"]),
        iss(3, R + ["priority:P1", "area:sparq-engine"]),
        iss(4, R + ["priority:P1", "area:sparq-engine", "needs:user"]),         # gated
        iss(5, R + ["priority:P1", "area:sparq-zk"], blk=2),                     # blocked
        iss(6, R + ["priority:P0", "area:sparq-hdt"], state="CLOSED"),           # closed
        iss(7, R + ["priority:P1", "trust:untrusted", "area:sparq-geo"]),        # untrusted
        iss(8, ["priority:P3", "role:impl", "area:sparq-text"]),                 # not status:ready
        iss(9, R + ["priority:P1", "priority:P2", "area:sparq-sim"]),            # ambiguous priority
        iss(10, R + ["priority:P1", "area:sparq-fedplan", "status:in-progress"]),# in-progress fedplan
        iss(11, R + ["priority:P4"]),                                            # no package -> global
        iss(12, R + ["priority:P1", "area:sparq-hdt"]),                          # hdt (free)
        iss(13, R + ["priority:P0", "area:sparq-text", "kind:epic"]),            # epic -> excluded
    ]
    ok = True

    def check(name, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {name}: {got} (want {want})")

    ready = compute_ready(F, conflict_log=quiet)
    # eligible: 1,2,3,12 (+11 global). 4 gated, 5 blocked, 6 closed, 7 untrusted, 8 no-ready,
    # 9 ambiguous-prio, 10 in-progress, 13 epic (kind:epic → excluded despite a P0 ready label-set).
    # Order by prio: #2(P0 core) -> #3(P1 engine) -> #12(P1 hdt) -> #11(P4 global). core taken by #2
    # so #1(P2 core) excluded. #11 global: only selectable if nothing taken -> excluded.
    check("ready order", [i["number"] for i in ready], [2, 3, 12])
    check("existing readiness fixtures unchanged-green", [i["number"] for i in ready], [2, 3, 12])
    # a P0 epic with an otherwise-perfect ready label-set must NOT dispatch (tracking umbrella):
    check("epic excluded", 13 in [i["number"] for i in ready], False)
    # a lone global issue with an empty board is selectable:
    check("lone global", [i["number"] for i in compute_ready(
        [iss(11, R + ["priority:P4"])], conflict_log=quiet)], [11])
    # global blocks everything else:
    g = compute_ready(
        [iss(11, R + ["priority:P0"]), iss(12, R + ["priority:P1", "area:sparq-hdt"])],
        conflict_log=quiet)
    check("global serializes", [i["number"] for i in g], [11])

    # [GPT-5.6] Parked-occupancy tripwires. These are end-to-end through compute_ready so deleting
    # the predicate, broadening it to every draft, or removing attribution makes --self-test red.
    def pr(n, labels, draft=True):
        return {"number": n, "state": "OPEN", "labels": labels,
                "pull_request": {}, "draft": draft}

    waiting = iss(20, R + ["priority:P1", "area:sparq-store"])
    parked = pr(70, ["area:sparq-store", "needs:user"])
    check("needs:user-parked draft PR does not block ready issue",
          [i["number"] for i in compute_ready([parked, waiting], conflict_log=quiet)], [20])
    unparked = {**parked, "labels": ["area:sparq-store"]}
    check("park-label removal restores snapshot occupancy",
          compute_ready([unparked, waiting], conflict_log=quiet), [])

    active = pr(71, ["area:sparq-store", "review:changes"])
    active_logs = []
    check("non-parked draft PR still blocks",
          compute_ready([active, waiting], conflict_log=active_logs.append), [])
    check("conflict log names the blocking artifact",
          active_logs, ["conflict #20: area sparq-store held by pr#71"])
    per_exclusion_logs = []
    global_then_two = compute_ready([
        iss(30, R + ["priority:P0"]),
        iss(31, R + ["priority:P1", "area:sparq-core"]),
        iss(32, R + ["priority:P2", "area:sparq-engine"]),
    ], conflict_log=per_exclusion_logs.append)
    check("one conflict log per excluded candidate",
          ([it["number"] for it in global_then_two], per_exclusion_logs),
          ([30], ["conflict #31: area __global__ held by issue#30",
                  "conflict #32: area __global__ held by issue#30"]))

    in_progress = iss(72, ["status:in-progress", "area:sparq-store"])
    check("status:in-progress issue still blocks",
          compute_ready([in_progress, waiting], conflict_log=quiet), [])
    check("all terminal park labels remove occupancy",
          [[it["number"] for it in compute_ready(
              [pr(73 + i, ["area:sparq-store", label]), waiting], conflict_log=quiet)]
           for i, label in enumerate(sorted(PARKED_AREA_LABELS))], [[20], [20], [20]])
    check("valid_priority single", valid_priority({"priority:P0"}), 0)
    check("valid_priority ambiguous", valid_priority({"priority:P1", "priority:P2"}), None)
    check("valid_priority out-of-range", valid_priority({"priority:P7"}), None)
    check("packages multi", packages_of({"area:a", "area:b"}), {"a", "b"})
    check("packages none->global", packages_of({"role:impl"}), {GLOBAL})
    check("untriaged is busy", is_busy({"status:untriaged"}), True)
    # paginated-snapshot flattening: multi-page merge, PR rows retained for occupancy, junk tolerated
    check("flatten pages retains PRs", _flatten_pages(
        [[{"number": 1}, {"number": 2, "pull_request": {}}], [{"number": 3}], "junk", [None]]),
        [{"number": 1}, {"number": 2, "pull_request": {}}, {"number": 3}])
    print("ready-issues self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output, retaining PRs as occupancy artifacts."""
    return [i for page in pages for i in (page if isinstance(page, list) else [])
            if isinstance(i, dict)]


_LINK_HEAD = re.compile(r"^sparq-agent/issue-([1-9][0-9]*)-")
_CLOSES = re.compile(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#([1-9][0-9]*)\b")
TRUSTED_ASSOCIATIONS = {"OWNER", "MEMBER", "COLLABORATOR"}


def linked_issue_numbers(pulls, repo):
    """Issues already covered by an open PR — the suppression the REGISTRY planner applies.

    [OPUS-5] Mirrors dispatch.yml's `linked_issue_numbers` so the local CLI previews the same
    frontier the orchestrator dispatches. A fork PR must never suppress an issue: only a
    same-repo `sparq-agent/issue-N-*` head is pipeline-owned provenance (a fork's head branch
    text is attacker-controlled), and a closing keyword in a body counts only from a trusted
    author association.
    """
    linked = set()
    for pull in pulls:
        head = pull.get("head") or {}
        ref = head.get("ref") or ""
        body = pull.get("body") or ""
        same_repo = ((head.get("repo") or {}).get("full_name") == repo)
        app_pr = same_repo and _LINK_HEAD.match(ref) is not None
        if app_pr:
            linked.update(int(n) for n in _LINK_HEAD.findall(ref))
        if app_pr or str(pull.get("author_association", "")).upper() in TRUSTED_ASSOCIATIONS:
            linked.update(int(n) for n in _CLOSES.findall(body))
    return linked


def _fetch_pulls(repo):
    out = subprocess.run(
        ["gh", "api", "--paginate", "--slurp", f"repos/{repo}/pulls?state=open&per_page=100"],
        capture_output=True, text=True, check=True).stdout
    return [p for page in json.loads(out or "[]") if isinstance(page, list)
            for p in page if isinstance(p, dict)]


def _fetch(repo, ceiling=10000):
    """Open-issue snapshot via REAL cursor pagination (`gh api --paginate` follows Link headers),
    replacing the old single-page `--limit 1000` fetch that FAILED CLOSED at exactly 1000 open
    issues — the full bd migration (~900 beads on top of organic issues) crosses that. The explicit
    ceiling still fails closed on a runaway snapshot."""
    out = subprocess.run(
        ["gh", "api", "--paginate", "--slurp",
         f"repos/{repo}/issues?state=open&per_page=100"],
        capture_output=True, text=True, check=True).stdout
    pages = json.loads(out or "[]")
    raw = _flatten_pages(pages)
    if len(raw) >= ceiling:
        raise SystemExit(f"refusing: fetched {len(raw)} >= ceiling {ceiling} — snapshot looks "
                         "runaway (fail-closed). Raise the ceiling deliberately if the backlog "
                         "is really that large.")
    open_numbers = {i["number"] for i in raw if "pull_request" not in i}
    issues = []
    for i in raw:
        row = {"number": i["number"], "state": i["state"], "labels": i["labels"],
               "open_blockers": 0}
        if "pull_request" in i:
            row["pull_request"] = i["pull_request"]
            row["draft"] = i.get("draft")
        else:
            blockers = re.findall(r"[Bb]locked-by:\s*#(\d+)", i.get("body") or "")
            row["open_blockers"] = sum(1 for b in blockers if int(b) in open_numbers)
        issues.append(row)
    return issues


def diagnose(issues, linked=()):
    """Re-runnable VISIBILITY taxonomy: why the open backlog is not on the frontier.

    Returns (counts, roleless, candidates, frontier). Every open issue lands in exactly one
    bucket, so the buckets sum to the open-issue count and no class can hide.
    """
    linked = set(linked)
    counts, open_issues = {}, []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN" or "pull_request" in it:
            continue
        open_issues.append(it)
        if it.get("number") in linked:
            reason = "covered by an open linked PR"
        else:
            reason = exclusion_reason(labels_of(it), it.get("open_blockers", 0)) or "ENUMERABLE"
        counts[reason] = counts.get(reason, 0) + 1
    visible = [it for it in issues if it.get("number") not in linked]
    return (counts, roleless_ready(open_issues),
            ready_candidates(visible), compute_ready(visible, conflict_log=lambda _m: None))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--diagnose", action="store_true",
                    help="print the full visibility taxonomy instead of just the frontier")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    issues = _fetch(args.repo)
    linked = linked_issue_numbers(_fetch_pulls(args.repo), args.repo)
    visible = [it for it in issues if it.get("number") not in linked]
    if args.diagnose:
        counts, roleless, cands, frontier = diagnose(issues, linked)
        total = sum(counts.values())
        print(f"open issues: {total}")
        for reason, n in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"  {n:5d}  {100 * n / total:5.1f}%  {reason}")
        print(f"\ndrainable backlog (ready_candidates): {len(cands)}")
        print(f"concurrency frontier (compute_ready): {len(frontier)}")
        if roleless:
            print(f"\n::warning:: {len(roleless)} attested issue(s) carry NO role:* label and are "
                  f"INVISIBLE to dispatch: {', '.join('#%d' % n for n in roleless[:20])}")
        return 0
    for it in compute_ready(visible):
        L = labels_of(it)
        print(f"P{valid_priority(L)}  #{it['number']:5}  {sorted(packages_of(L))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
