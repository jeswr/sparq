#!/usr/bin/env python3
# [FABLE-5] Issue-native orchestration: the designed RETRIAGE cron pass (audit-2026-07-17).
"""retriage.py — re-run static triage over parked `status:untriaged` issues, fail-closed.

`triage-issue.yml` only fires on opened/edited/reopened, and GitHub's `edited` activity does NOT
cover label changes — so an issue that lands `status:untriaged` (e.g. opened without a priority)
was TERMINAL: even a human later adding `priority:P2` never re-ran triage. This sweep closes the
loop the triage.py docstring already promised ("status:untriaged for the retriage cron"):

  * OPEN issues carrying `status:untriaged` are considered, PLUS (#2474) OPEN `flow-on` issues
    with NO `status:*` label at all — the flow-on engine used to mint follow-ups under
    `secrets.GITHUB_TOKEN` with no routing labels, and GitHub suppresses workflow events for
    GITHUB_TOKEN-created objects, so triage-issue.yml never saw them (flow-on.py now labels at
    creation; this sweep converges the pre-fix backlog);
  * `trust:untrusted` is excluded (owned by promote-on-approval) and `kind:epic` is excluded
    (untriaged-by-design tracking umbrellas);
  * the author trust-gate is re-verified exactly like triage-issue.yml (the orchestrator App bot,
    or admin/maintain/write collaborators) — untrusted authors are never triaged here;
  * scripts/triage.py (the same static pass) recomputes the label delta; if the issue has NO
    priority label at all, a default `priority:P3` is applied so the loop CONVERGES instead of
    parking forever (an AMBIGUOUS priority still parks — that needs a human);
  * retriage only PROMOTES (untriaged -> ready). It never re-parks or edits gated issues, so a
    needs:* / ambiguous issue is simply left untouched.

Pure `plan_retriage()` is unit-tested (--self-test); the CLI wraps it over `gh`. Default is a
dry-run print; the cron passes --apply.
"""
import argparse
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import triage  # noqa: E402  (same-directory static-triage module)

DEFAULT_PRIORITY = "priority:P3"
TRUSTED_BOT = "sparq-orchestrator[bot]"
SKIP_LABELS = {"trust:untrusted", "kind:epic"}
# [FABLE-5] (#2474) flow-on.yml mints follow-ups under secrets.GITHUB_TOKEN, i.e. as
# github-actions[bot]. That login is trusted HERE only for `flow-on`-labelled issues:
# their content comes from the checked-in rule table (scripts/flow-on-rules.toml), not
# third-party input — this is NOT blanket [bot] trust (an arbitrary github-actions[bot]
# issue without the flow-on label still fails the collaborator probe and is skipped).
FLOW_ON_MINTER = "github-actions[bot]"
FLOW_ON_LABEL = "flow-on"


def _needs_triage(labels):
    """status:untriaged, or (#2474) a flow-on-minted issue with NO status:* label at all —
    the class the event triager can never see (GITHUB_TOKEN event suppression) and that the
    old status:untriaged-only sweep missed."""
    if "status:untriaged" in labels:
        return True
    return FLOW_ON_LABEL in labels and not any(lb.startswith("status:") for lb in labels)


def plan_retriage(issues, trusted):
    """[(number, add:sortedlist, remove:sortedlist)] for open issues needing triage (see
    `_needs_triage`) whose author passes `trusted(login)`. Only PROMOTING deltas are returned
    (ready==True); everything else is left untouched for a human / the future LLM pass."""
    actions = []
    for it in issues:
        labels = {lb["name"] if isinstance(lb, dict) else lb for lb in it.get("labels", [])}
        if not _needs_triage(labels) or labels & SKIP_LABELS:
            continue
        author = str(it.get("author", ""))
        flow_on_minted = author == FLOW_ON_MINTER and FLOW_ON_LABEL in labels
        if not (trusted(author) or flow_on_minted):
            continue
        add_default = not any(triage._PRIO.match(lb) for lb in labels)
        effective = labels | ({DEFAULT_PRIORITY} if add_default else set())
        result = triage.triage(effective, "task")
        if not result["ready"]:
            continue  # ambiguous priority / needs:* / no role — parked for a human, no churn
        add = set(result["add"]) | ({DEFAULT_PRIORITY} if add_default else set())
        remove = set(result["remove"])
        actions.append((it.get("number"), sorted(add - labels), sorted(remove & labels)))
    return actions


def _gh(args):
    return subprocess.run(["gh", *args], capture_output=True, text=True, check=False)


def _trusted_factory(repo):
    cache = {}

    def trusted(login):
        if not login:
            return False
        if login == TRUSTED_BOT:
            return True
        if login not in cache:
            r = _gh(["api", f"repos/{repo}/collaborators/{login}/permission",
                     "--jq", ".permission"])
            cache[login] = (r.stdout or "").strip() if r.returncode == 0 else "none"
        return cache[login] in {"admin", "maintain", "write"}

    return trusted


def _fetch_candidates(repo):
    # Two label queries: parked status:untriaged issues, plus (#2474) flow-on issues —
    # plan_retriage's _needs_triage then keeps only the label-less-status flow-on subset,
    # so an already-routed flow-on issue is never churned. Deduped by issue number.
    issues, seen = [], set()
    for label in ("status:untriaged", FLOW_ON_LABEL):
        r = _gh(["issue", "list", "-R", repo, "--state", "open", "--label", label,
                 "--limit", "500", "--json", "number,labels,author"])
        if r.returncode != 0:
            raise SystemExit(f"retriage: could not list {label} issues")
        for it in json.loads(r.stdout or "[]"):
            num = it.get("number")
            if num in seen:
                continue
            seen.add(num)
            issues.append({"number": num, "labels": it.get("labels") or [],
                           "author": ((it.get("author") or {}).get("login") or "")})
    return issues


def _self_test():
    ok = True

    def chk(n, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {n}: {got} (want {want})")

    def iss(n, labels, author="jeswr"):
        return {"number": n, "labels": labels, "author": author}

    trusted = lambda login: login in {"jeswr", TRUSTED_BOT}  # noqa: E731
    fixture = [
        # 1: human later added a priority by LABEL (the exact terminal case) -> promotes
        iss(1, ["status:untriaged", "role:impl", "priority:P2", "area:sparq-core"]),
        # 2: no priority at all -> default P3 -> promotes
        iss(2, ["status:untriaged", "kind:docs", "area:site"]),
        # 3: AMBIGUOUS priority -> needs a human, untouched
        iss(3, ["status:untriaged", "role:impl", "priority:P1", "priority:P2"]),
        # 4: epic umbrella -> untriaged-by-design, skipped
        iss(4, ["status:untriaged", "kind:epic", "priority:P1", "role:impl"]),
        # 5: quarantined -> owned by promote-on-approval, skipped
        iss(5, ["status:untriaged", "trust:untrusted", "priority:P1"]),
        # 6: untrusted author -> never triaged here
        iss(6, ["status:untriaged", "priority:P1", "role:impl"], author="rando"),
        # 7: needs:user gate -> triage refuses ready, untouched
        iss(7, ["status:untriaged", "priority:P1", "role:impl", "needs:user"]),
        # 8: not untriaged at all -> out of scope
        iss(8, ["status:ready", "priority:P1", "role:impl"]),
        # 9: orchestrator-bot author is trusted
        iss(9, ["status:untriaged", "priority:P2", "kind:test"], author=TRUSTED_BOT),
        # 10 (#2474): flow-on follow-up minted by github-actions[bot] with NO status:* label
        # (the GITHUB_TOKEN event-suppression class) -> default P3 -> promotes
        iss(10, ["flow-on", "auto", "docs"], author=FLOW_ON_MINTER),
        # 11 (#2474): github-actions[bot] WITHOUT the flow-on label -> no blanket bot trust
        iss(11, ["status:untriaged", "priority:P1", "kind:ci"], author=FLOW_ON_MINTER),
        # 12 (#2474): flow-on issue already routed (has a status:*) -> out of scope, no churn
        iss(12, ["flow-on", "auto", "status:ready", "priority:P3", "role:impl"],
            author=FLOW_ON_MINTER),
    ]
    actions = {n: (a, r) for n, a, r in plan_retriage(fixture, trusted)}
    chk("promoted set", sorted(actions), [1, 2, 9, 10])
    chk("label-added priority promotes", actions[1],
        (["status:ready"], ["status:untriaged"]))
    chk("default P3 applied", "priority:P3" in actions[2][0], True)
    chk("default promotion readies", ("status:ready" in actions[2][0],
                                      "status:untriaged" in actions[2][1]), (True, True))
    chk("role derived for default case", any(x.startswith("role:") for x in actions[2][0]), True)
    chk("bot author promoted", actions[9][0][0].startswith(("priority", "role", "status")), True)
    chk("flow-on statusless backlog promoted (#2474)",
        ("status:ready" in actions[10][0], "priority:P3" in actions[10][0],
         any(x.startswith("role:") for x in actions[10][0])), (True, True, True))
    chk("github-actions[bot] not blanket-trusted", 11 in actions, False)
    chk("already-routed flow-on untouched", 12 in actions, False)
    chk("ambiguous priority untouched", 3 in actions, False)
    chk("epic skipped", 4 in actions, False)
    chk("quarantine skipped", 5 in actions, False)
    chk("untrusted author skipped", 6 in actions, False)
    chk("needs:user untouched", 7 in actions, False)
    print("retriage self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--apply", action="store_true", help="apply the label deltas (cron mode)")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    actions = plan_retriage(_fetch_candidates(args.repo), _trusted_factory(args.repo))
    for number, add, remove in actions:
        print(f"#{number}: +{','.join(add) or '-'} -{','.join(remove) or '-'}")
        if args.apply:
            for lb in add:
                _gh(["issue", "edit", str(number), "-R", args.repo, "--add-label", lb])
            for lb in remove:
                _gh(["issue", "edit", str(number), "-R", args.repo, "--remove-label", lb])
    print(f"retriage: {len(actions)} issue(s) {'promoted' if args.apply else 'promotable (dry-run)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
