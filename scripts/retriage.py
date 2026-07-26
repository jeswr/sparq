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
import urllib.parse

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


FETCH_CEILING = 10000


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output, dropping PRs (the issues endpoint returns
    both) and any non-list/non-dict junk."""
    return [i for page in pages if isinstance(page, list) for i in page
            if isinstance(i, dict) and "pull_request" not in i]


def _fetch_label(repo, label, ceiling=FETCH_CEILING):
    """Every OPEN issue carrying `label`, via REAL cursor pagination.

    [OPUS-5] This was `gh issue list --limit 500`, which SILENTLY TRUNCATES: the CLI stops at
    the limit and reports nothing. MEASURED live on sparq-org/sparq 2026-07-26 — 719 open
    `status:untriaged` issues, `--limit 500` returned exactly 500 and dropped 219 (#2360..#2750),
    newest-first, so the oldest parked issues were the permanent casualties. Retriage is a WORK
    QUEUE sweep, so a truncated fetch is not a slow drain: those issues are never candidates on
    any tick, at any point in the future, with no signal. `gh api --paginate` follows Link
    headers to exhaustion; the explicit ceiling still fails closed on a runaway snapshot (same
    shape as scripts/ready-issues.py `_fetch`).

    Field note: the REST payload names the author `user.login`, NOT the `author.login` that
    `gh issue list --json author` emits. Getting that wrong makes every author read as "" and
    `_trusted_factory` then rejects EVERY issue — a silent total stall, not a visible error.
    `_self_test` pins it.
    """
    r = _gh(["api", "--paginate", "--slurp",
             f"repos/{repo}/issues?state=open&labels={urllib.parse.quote(label)}&per_page=100"])
    if r.returncode != 0:
        raise SystemExit(f"retriage: could not list {label} issues")
    rows = _flatten_pages(json.loads(r.stdout or "[]"))
    if len(rows) >= ceiling:
        raise SystemExit(f"retriage: fetched {len(rows)} '{label}' issues >= ceiling {ceiling} — "
                         "snapshot looks runaway (fail-closed). Raise the ceiling deliberately.")
    return rows


def _fetch_candidates(repo):
    # Two label queries: parked status:untriaged issues, plus (#2474) flow-on issues —
    # plan_retriage's _needs_triage then keeps only the label-less-status flow-on subset,
    # so an already-routed flow-on issue is never churned. Deduped by issue number.
    issues, seen = [], set()
    for label in ("status:untriaged", FLOW_ON_LABEL):
        for it in _fetch_label(repo, label):
            num = it.get("number")
            if num in seen:
                continue
            seen.add(num)
            issues.append({"number": num, "labels": it.get("labels") or [],
                           "author": ((it.get("user") or {}).get("login") or "")})
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
        # [OPUS-4.8] (#2474 CI-fix) area:sparq-core so triage can reach status:ready — a no-area
        # issue fail-closes to needs:area (it would reserve the serializing __global__ partition),
        # so promotion REQUIRES an area, exactly as a real minted follow-on now carries one.
        iss(9, ["status:untriaged", "priority:P2", "kind:test", "area:sparq-core"], author=TRUSTED_BOT),
        # 10 (#2474): flow-on follow-up minted by github-actions[bot] with NO status:* label
        # (the GITHUB_TOKEN event-suppression class) -> default P3 -> promotes.
        # [OPUS-4.8] (#2474 CI-fix) carries area:docs, matching how flow-on-rules.toml now mints
        # every follow-on with an area:<crate> (a no-area issue can never reach status:ready —
        # triage fail-closes it to needs:area to keep it out of the __global__ partition).
        iss(10, ["flow-on", "auto", "docs", "area:docs"], author=FLOW_ON_MINTER),
        # 11 (#2474): github-actions[bot] WITHOUT the flow-on label -> no blanket bot trust
        iss(11, ["status:untriaged", "priority:P1", "kind:ci"], author=FLOW_ON_MINTER),
        # 12 (#2474): flow-on issue already routed (has a status:*) -> out of scope, no churn
        iss(12, ["flow-on", "auto", "status:ready", "priority:P3", "role:impl"],
            author=FLOW_ON_MINTER),
    ]
    actions = {n: (a, r) for n, a, r in plan_retriage(fixture, trusted)}
    # [FABLE-5] (#3419) .get defaults: a fixture dropped by a triage.py behavior change (how
    # the #2898 no-area parking drift surfaced) must print a clean FAIL line, not a KeyError.
    a1, a2, a9, a10 = (actions.get(n, ([], [])) for n in (1, 2, 9, 10))
    chk("promoted set", sorted(actions), [1, 2, 9, 10])
    chk("label-added priority promotes", a1, (["status:ready"], ["status:untriaged"]))
    chk("default P3 applied", "priority:P3" in a2[0], True)
    chk("default promotion readies", ("status:ready" in a2[0],
                                      "status:untriaged" in a2[1]), (True, True))
    chk("role derived for default case", any(x.startswith("role:") for x in a2[0]), True)
    chk("bot author promoted", bool(a9[0]) and a9[0][0].startswith(("priority", "role", "status")), True)
    chk("flow-on statusless backlog promoted (#2474)",
        ("status:ready" in a10[0], "priority:P3" in a10[0],
         any(x.startswith("role:") for x in a10[0])), (True, True, True))
    chk("github-actions[bot] not blanket-trusted", 11 in actions, False)
    chk("already-routed flow-on untouched", 12 in actions, False)
    chk("ambiguous priority untouched", 3 in actions, False)
    chk("epic skipped", 4 in actions, False)
    chk("quarantine skipped", 5 in actions, False)
    chk("untrusted author skipped", 6 in actions, False)
    chk("needs:user untouched", 7 in actions, False)

    # --- candidate FETCH must not silently truncate the work queue (review of #3823) ------------
    # `gh issue list --limit 500` stops at the limit and reports nothing. MEASURED live
    # 2026-07-26: 719 open status:untriaged issues, 500 returned, 219 dropped (#2360..#2750)
    # newest-first. Retriage is a work-queue sweep, so those never become candidates on ANY
    # future tick. The stub below emulates BOTH CLI shapes faithfully — `gh api --paginate`
    # exhausts the Link chain, `gh issue list --limit N` truncates newest-first — so reverting
    # to the limited call is EXECUTABLE here and fails on the missing rows rather than on an
    # unrecognised command. That is what makes this a behavioural guard and not a spelling test.
    class _Resp:
        def __init__(self, stdout, returncode=0):
            self.stdout, self.returncode = stdout, returncode

    total = 620                     # > 500, so a --limit 500 fetch must lose the oldest 120
    def _row(n, name):
        return {"number": n, "labels": [{"name": name}, {"name": "priority:P2"},
                                        {"name": "role:impl"}, {"name": "area:sparq-core"}],
                "user": {"login": "jeswr"}, "author": {"login": "jeswr"}}

    corpus = {"status:untriaged": [_row(n, "status:untriaged") for n in range(1, total + 1)],
              # #7 and #8 carry BOTH labels, so the second query re-returns them: the dedupe
              # is the only thing keeping them out of the candidate list twice.
              FLOW_ON_LABEL: [_row(n, FLOW_ON_LABEL) for n in (7, 8)]}
    seen_cmds = []

    def _stub_gh(args):
        """Faithful emulator of BOTH gh shapes, so either implementation is EXECUTABLE here.

        `gh api --paginate` follows the Link chain to exhaustion; WITHOUT --paginate gh returns
        only the first page. `gh issue list --limit N` truncates newest-first. Modelling the
        truncation both ways is what lets a reverted fetch fail on missing ROWS rather than on
        an unrecognised command line.
        """
        seen_cmds.append(list(args))
        if args[0] == "api":
            label = urllib.parse.unquote(args[-1].split("labels=")[1].split("&")[0])
            rows = corpus.get(label, [])
            pages = [rows[i:i + 100] for i in range(0, len(rows), 100)] or [[]]
            return _Resp(json.dumps(pages if "--paginate" in args else pages[:1]))
        if args[0:2] == ["issue", "list"]:
            label = args[args.index("--label") + 1]
            rows = corpus.get(label, [])
            limit = int(args[args.index("--limit") + 1]) if "--limit" in args else len(rows)
            return _Resp(json.dumps(sorted(rows, key=lambda r: -r["number"])[:limit]))
        raise AssertionError(f"unexpected gh invocation: {args}")

    real_gh = globals()["_gh"]
    globals()["_gh"] = _stub_gh
    try:
        cands = _fetch_candidates("o/r")
        nums = sorted(c["number"] for c in cands)
        chk("every labelled issue is fetched, not just the first page", len(nums), total)
        chk("the OLDEST issues survive the fetch (the truncation casualties)",
            nums[:3], [1, 2, 3])
        chk("no issue is dropped or duplicated", (nums[0], nums[-1], len(set(nums))),
            (1, total, total))
        # #7/#8 carry both query labels; without the dedupe they enter the queue twice and
        # plan_retriage emits two label-edit actions for the same issue.
        chk("an issue matching BOTH label queries appears exactly once",
            [nums.count(n) for n in (7, 8)], [1, 1])
        # REST names the author `user.login`; reading `author.login` yields "" for every issue
        # and _trusted_factory then rejects the entire queue — a silent total stall.
        chk("author login is read from the REST `user` field",
            {c["author"] for c in cands}, {"jeswr"})
        chk("the fetch uses cursor pagination, not a fixed page limit",
            all(c[0] == "api" and "--paginate" in c for c in seen_cmds), True)
        # fail-closed ceiling: a runaway snapshot must raise, never silently half-report
        try:
            _fetch_label("o/r", "status:untriaged", ceiling=10)
            chk("runaway snapshot fails closed", "no raise", "SystemExit")
        except SystemExit:
            chk("runaway snapshot fails closed", "SystemExit", "SystemExit")
        # PR rows come back from the issues endpoint and must not enter the triage queue
        corpus["status:untriaged"].append(
            {"number": 9001, "labels": [{"name": "status:untriaged"}],
             "user": {"login": "jeswr"}, "pull_request": {"url": "x"}})
        chk("pull requests are not retriage candidates",
            9001 in {c["number"] for c in _fetch_candidates("o/r")}, False)
    finally:
        globals()["_gh"] = real_gh

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
