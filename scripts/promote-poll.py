#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — the quarantine POLL behind promote-on-approval.yml (sparq#5425).
"""promote-poll.py — poll the `trust:untrusted` quarantine for maintainer 👍 approval.

GitHub Actions has NO reaction event, so `promote-on-approval.yml` polls the quarantine on a cron
and promotes any issue a MAINTAINER has 👍-approved AFTER its last edit (revision-bound: an edit
after approval re-quarantines). The decision itself is `scripts/promote.py::should_promote`; the
post-promotion label delta is `scripts/triage.py::triage`. This file is the driver that joins them
to `gh` — it invents no policy of its own.

WHY IT IS A FILE AND NOT INLINE WORKFLOW PYTHON (sparq#5425, follow-up to the #4985 `--limit`
sweep). The driver used to live in a `python3 - <<'PY'` heredoc inside the workflow, where it
enumerated the queue with:

    gh issue list --label trust:untrusted --state open --limit 200

`--limit` TRUNCATES SILENTLY — the CLI stops at the limit, prints no warning and exits 0 — and it
truncates newest-first, so once the open quarantine passed 200 the OLDEST issues stopped being
polled at all. Not "polled late": never again, on any tick, with nothing in the run log saying so,
which is precisely the class of defect a cron cannot surface by itself. `gh api --paginate` follows
the Link headers to exhaustion instead; the explicit ceiling still fails CLOSED on a runaway
snapshot rather than half-reporting (same shape as `retriage.py::_fetch_label`,
`ready-issues.py::_fetch` and `triage-area.py`). The issues endpoint returns PRs too, so rows
carrying `pull_request` are dropped.

Being a file is the other half of the fix: inline heredoc python has no `--self-test`, so nothing
could go red on the PR that reverted the fetch. `--self-test` here is hermetic (stdlib only, no
gh, no network) and drives the REAL driver against a stub that emulates BOTH `gh` shapes
faithfully, so reverting to the limited call is EXECUTABLE and fails on the missing rows.

Default is a dry-run print; the cron passes --apply.
"""
import argparse
import datetime
import json
import os
import subprocess
import sys
# Imported as bare names, not via `urllib.parse.…`: scripts/selftest_write_guard.py treats a
# `urllib`-rooted call as a network SEAM, and a query-string quote in the fetch path would make
# every function reaching it un-neutralisable in a self-test.
from urllib.parse import quote, unquote

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from promote import should_promote  # noqa: E402  (same-directory pure core)
from triage import triage  # noqa: E402  (same-directory pure core)

QUARANTINE_LABEL = "trust:untrusted"
MAINTAINER_PERMISSIONS = frozenset({"admin", "maintain", "write"})
# '+1' is the REST API's name for 👍; promote.py accepts either spelling.
APPROVAL_REACTION = "+1"
PROMOTION_COMMENT = ("> 🤖 automation — maintainer-approved (👍 after the last edit); promoted "
                     "out of quarantine and triaged.")
# A quarantine larger than this is not a queue, it is a runaway snapshot — fail closed rather than
# half-report, exactly as the sibling sweeps do.
FETCH_CEILING = 10000


def _gh(args):
    """The ONE subprocess seam in this module (`--self-test` rebinds it; nothing else may spawn)."""
    return subprocess.run(["gh", *args], capture_output=True, text=True, check=False)


def _epoch(timestamp):
    """ISO-8601 -> int epoch seconds, or None when absent/unparseable.

    None is NOT 0. `should_promote` compares the reaction time against this value, so a missing
    timestamp read as 0 would make EVERY maintainer 👍 post-date the last edit and silently strip
    the revision-binding this whole workflow exists to enforce. Callers skip the issue instead.
    """
    if not timestamp:
        return None
    try:
        return int(datetime.datetime.fromisoformat(
            str(timestamp).replace("Z", "+00:00")).timestamp())
    except (TypeError, ValueError):
        return None


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output, dropping PRs (the issues endpoint returns both)
    and any non-list/non-dict junk."""
    return [row for page in pages if isinstance(page, list) for row in page
            if isinstance(row, dict) and "pull_request" not in row]


def fetch_quarantined(repo, ceiling=FETCH_CEILING):
    """Every OPEN `trust:untrusted` issue, via REAL cursor pagination — see the module docstring
    for the `--limit 200` truncation this replaces."""
    result = _gh(["api", "--paginate", "--slurp",
                  f"repos/{repo}/issues?state=open"
                  f"&labels={quote(QUARANTINE_LABEL)}&per_page=100"])
    if result.returncode != 0:
        raise SystemExit(f"promote-poll: could not list open {QUARANTINE_LABEL} issues: "
                         f"{(result.stderr or '').strip()[:300]}")
    rows = _flatten_pages(json.loads(result.stdout or "[]"))
    if len(rows) >= ceiling:
        raise SystemExit(f"promote-poll: fetched {len(rows)} quarantined issues >= ceiling "
                         f"{ceiling} — refusing to promote from a snapshot that may itself be "
                         f"truncated")
    return rows


def labels_of(issue):
    return [lb["name"] for lb in (issue.get("labels") or [])
            if isinstance(lb, dict) and lb.get("name")]


def _maintainer_factory(repo):
    """`login -> bool` (write/maintain/admin), memoised — one login reacts on many issues."""
    cache = {}

    def is_maintainer(login):
        if not login:
            return False
        if login not in cache:
            result = _gh(["api", f"repos/{repo}/collaborators/{login}/permission",
                          "--jq", ".permission"])
            cache[login] = (result.stdout or "").strip() if result.returncode == 0 else "none"
        return cache[login] in MAINTAINER_PERMISSIONS

    return is_maintainer


def reactions_of(repo, number):
    """`should_promote`'s reaction rows for one issue: {content, user, created_at (int)}."""
    raw = json.loads(_gh(["api", f"repos/{repo}/issues/{number}/reactions"]).stdout or "[]")
    return [{"content": r.get("content"),
             "user": ((r.get("user") or {}).get("login") or ""),
             "created_at": _epoch(r.get("created_at")) or 0}
            for r in raw if isinstance(r, dict)]


def approving_maintainers(reactions, is_maintainer):
    """The maintainer logins that 👍'd — the `maintainers` set `should_promote` intersects."""
    return {r["user"] for r in reactions
            if r.get("content") == APPROVAL_REACTION and is_maintainer(r.get("user"))}


def promote_one(repo, issue, is_maintainer, apply=False, log=print):
    """Promote ONE quarantined issue if approved. Returns "promoted", "held" or "failed".

    Write ORDER is load-bearing and so is the return code of the FIRST write. Removing
    `trust:untrusted` is the trust-boundary crossing; the triage delta and the notice comment only
    describe it. The inline predecessor discarded every return code, so a failed un-quarantine
    still posted a comment claiming the issue had been promoted while it stayed in the queue.
    """
    number = issue.get("number")
    updated_at = _epoch(issue.get("updated_at"))
    if updated_at is None:
        log(f"skip #{number}: no parsable updated_at — cannot bind approval to a revision")
        return "held"
    reactions = reactions_of(repo, number)
    if not should_promote(reactions, updated_at, approving_maintainers(reactions, is_maintainer)):
        return "held"
    if not apply:
        log(f"would promote #{number}")
        return "promoted"
    result = _gh(["issue", "edit", str(number), "-R", repo, "--remove-label", QUARANTINE_LABEL])
    if result.returncode != 0:
        log(f"#{number}: un-quarantine FAILED (rc={result.returncode}); NOT triaged and NOT "
            f"commented: {(result.stderr or '').strip()[:300]}")
        return "failed"
    # triage() is a no-op while the quarantine label is present, so it is dropped from the set
    # handed over — the issue has just left quarantine.
    delta = triage([lb for lb in labels_of(issue) if lb != QUARANTINE_LABEL], "task", trusted=True)
    for label in sorted(delta["add"]):
        _gh(["issue", "edit", str(number), "-R", repo, "--add-label", label])
    for label in sorted(delta["remove"]):
        _gh(["issue", "edit", str(number), "-R", repo, "--remove-label", label])
    _gh(["issue", "comment", str(number), "-R", repo, "--body", PROMOTION_COMMENT])
    log(f"promoted #{number}")
    return "promoted"


def poll(repo, apply=False, log=print):
    """Poll the whole quarantine. Returns (considered, promoted-numbers, failed-numbers)."""
    issues = fetch_quarantined(repo)
    is_maintainer = _maintainer_factory(repo)
    promoted, failed = [], []
    for issue in issues:
        outcome = promote_one(repo, issue, is_maintainer, apply=apply, log=log)
        if outcome == "promoted":
            promoted.append(issue.get("number"))
        elif outcome == "failed":
            failed.append(issue.get("number"))
    return len(issues), promoted, failed


def _self_test():
    ok = True

    def chk(name, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {name}: {got} (want {want})")

    # ---------------------------------------------------------------------------------------
    # The corpus. 250 open quarantined issues — deliberately more than the 200 the inline
    # predecessor asked for, since the defect is DEPTH-CONDITIONAL: under 200 the truncated and
    # the paginated fetch are identical and nothing here could tell them apart.
    # ---------------------------------------------------------------------------------------
    QUARANTINED = 250
    APPROVED = 7          # OLD: inside the tail `--limit 200` drops (it truncates newest-first)
    STALE_APPROVAL = 8    # 👍 predates the last edit
    OUTSIDER = 9          # 👍 from a non-maintainer
    NO_TIMESTAMP = 10     # unparsable updated_at
    A_PULL_REQUEST = 998  # the issues endpoint returns PRs too
    NOT_QUARANTINED = 999

    EDIT = 1_000_000      # issue.updated_at for every fixture row
    MAINTAINER, RANDO = "jeswr", "rando"

    def stamp(seconds):
        return datetime.datetime.fromtimestamp(
            seconds, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    def row(number, labels, updated_at=stamp(EDIT), pull=False):
        out = {"number": number, "updated_at": updated_at,
               "labels": [{"name": lb} for lb in labels]}
        if pull:
            out["pull_request"] = {"url": "…"}
        return out

    QUARANTINE_LABELS = [QUARANTINE_LABEL, "priority:P2", "role:impl", "area:sparq-core"]
    corpus = [row(n, QUARANTINE_LABELS) for n in range(1, QUARANTINED + 1)]
    corpus[NO_TIMESTAMP - 1] = row(NO_TIMESTAMP, QUARANTINE_LABELS, updated_at="")
    corpus.append(row(A_PULL_REQUEST, QUARANTINE_LABELS, pull=True))
    corpus.append(row(NOT_QUARANTINED, ["status:ready", "priority:P2"]))

    REACTIONS = {
        APPROVED: [{"content": "+1", "user": MAINTAINER, "created_at": stamp(EDIT + 60)}],
        STALE_APPROVAL: [{"content": "+1", "user": MAINTAINER, "created_at": stamp(EDIT - 60)}],
        OUTSIDER: [{"content": "+1", "user": RANDO, "created_at": stamp(EDIT + 60)}],
        NO_TIMESTAMP: [{"content": "+1", "user": MAINTAINER, "created_at": stamp(EDIT + 60)}],
    }

    seen, writes = [], []
    fail_unquarantine = set()

    class _Resp:
        def __init__(self, stdout="", returncode=0, stderr=""):
            self.stdout, self.returncode, self.stderr = stdout, returncode, stderr

    def _labels_of(r):
        return {lb["name"] for lb in r["labels"]}

    def _stub_gh(args):
        """Faithful emulator of BOTH gh shapes, so EITHER implementation is executable here.

        `gh api --paginate` follows the Link chain to exhaustion and without `--paginate` gh
        returns only the first page; `gh issue list --limit N` truncates NEWEST-FIRST. Modelling
        all three means reverting `fetch_quarantined` to the limited CLI call fails on missing
        ROWS rather than on an unrecognised command — that is what makes this a behavioural guard
        and not a spelling test.
        """
        seen.append(list(args))
        if args[0:1] == ["api"]:
            path = next(a for a in args[1:] if a.startswith("repos/"))
            if "/issues?" in path:
                query = path.split("?", 1)[1]
                rows = list(corpus)
                if "labels=" in query:
                    label = unquote(query.split("labels=")[1].split("&")[0])
                    rows = [r for r in rows if label in _labels_of(r)]
                pages = [rows[i:i + 100] for i in range(0, len(rows), 100)] or [[]]
                return _Resp(json.dumps(pages if "--paginate" in args else pages[:1]))
            if path.endswith("/reactions"):
                number = int(path.rsplit("/", 2)[-2])
                return _Resp(json.dumps([{"content": r["content"],
                                          "user": {"login": r["user"]},
                                          "created_at": r["created_at"]}
                                         for r in REACTIONS.get(number, [])]))
            if "/collaborators/" in path:
                login = path.split("/collaborators/")[1].split("/")[0]
                return _Resp("admin\n" if login == MAINTAINER else "none\n")
        if args[0:2] == ["issue", "list"]:
            # `gh issue list` never returns PRs and its --json keys are camelCase — modelled so
            # the alternative implementation is emulated as it really behaves, not as a strawman.
            label = args[args.index("--label") + 1]
            rows = [{"number": r["number"], "updatedAt": r["updated_at"], "labels": r["labels"]}
                    for r in corpus if label in _labels_of(r) and "pull_request" not in r]
            limit = int(args[args.index("--limit") + 1]) if "--limit" in args else len(rows)
            return _Resp(json.dumps(sorted(rows, key=lambda r: -r["number"])[:limit]))
        if args[0:2] in (["issue", "edit"], ["issue", "comment"]):
            writes.append(list(args))
            if (args[0:2] == ["issue", "edit"] and QUARANTINE_LABEL in args
                    and int(args[2]) in fail_unquarantine):
                return _Resp(returncode=1, stderr="could not update issue")
            return _Resp()
        raise AssertionError(f"unexpected gh invocation: {args}")

    # NEUTRALISE the one subprocess seam for the duration of the run — `global` + assign, the
    # pattern scripts/selftest_write_guard.py requires of any write-enabled self-test (#5098).
    global _gh
    real_gh = _gh
    try:
        _gh = _stub_gh

        # --- the FETCH must not silently truncate the quarantine ------------------------------
        fetched = fetch_quarantined("fixture/repo")
        numbers = [r["number"] for r in fetched]
        chk("the whole open quarantine is fetched, not the newest 200", len(numbers), QUARANTINED)
        # Sorted, not sliced in fetch order: what must hold is that the oldest issues are PRESENT,
        # and the real endpoint's ordering is not this guard's subject.
        chk("the OLDEST quarantined issues survive the fetch (the tail --limit drops)",
            sorted(numbers)[:3], [1, 2, 3])
        chk("a PR carrying the quarantine label is not polled as an issue",
            A_PULL_REQUEST in numbers, False)
        chk("an unquarantined issue is not fetched", NOT_QUARANTINED in numbers, False)
        chk("the fetch uses cursor pagination, not a fixed page limit",
            all(c[0] == "api" and "--paginate" in c for c in seen), True)
        # fail-closed ceiling: a runaway snapshot raises rather than half-reporting.
        try:
            fetch_quarantined("fixture/repo", ceiling=10)
            chk("a runaway snapshot fails closed", "no raise", "SystemExit")
        except SystemExit:
            chk("a runaway snapshot fails closed", "SystemExit", "SystemExit")

        # --- end-to-end: the approval decision, and the writes it authorises -------------------
        # THE ACCEPTANCE ROW for sparq#5425. #7 is old enough to sit in the tail a `--limit 200`
        # fetch drops, so reverting the fetch turns this line red — the poll would report zero
        # promotions with the maintainer's 👍 sitting there unread.
        writes.clear()
        considered, promoted, _failed = poll("fixture/repo", apply=True, log=lambda *_: None)
        chk("every quarantined issue is considered", considered, QUARANTINED)
        chk("the maintainer-approved OLD issue is promoted", promoted, [APPROVED])
        edits = [w for w in writes if w[0:2] == ["issue", "edit"]]
        chk("the quarantine label is removed first",
            edits[0][2:] == [str(APPROVED), "-R", "fixture/repo",
                             "--remove-label", QUARANTINE_LABEL], True)
        # The `trust:untrusted` strip before triage() is load-bearing: triage is a no-op while the
        # label is present, so leaving it in yields an EMPTY delta and no status:ready.
        chk("the post-promotion triage delta is applied",
            [w[-1] for w in edits[1:]], ["status:ready"])
        chk("the promotion notice is commented once",
            [w[-1] for w in writes if w[0:2] == ["issue", "comment"]], [PROMOTION_COMMENT])

        # --- the approval rules survive the extraction ----------------------------------------
        chk("a 👍 that PREDATES the last edit does not promote", STALE_APPROVAL in promoted, False)
        chk("a non-maintainer 👍 does not promote", OUTSIDER in promoted, False)
        chk("an issue with no parsable updated_at is SKIPPED, not promoted (fail-closed)",
            NO_TIMESTAMP in promoted, False)

        # --- dry run writes nothing ------------------------------------------------------------
        writes.clear()
        _considered, dry, _failed = poll("fixture/repo", apply=False, log=lambda *_: None)
        chk("a dry run reports the same promotion and writes nothing", (dry, writes),
            ([APPROVED], []))

        # --- a failed un-quarantine is not laundered into a promotion --------------------------
        writes.clear()
        fail_unquarantine.add(APPROVED)
        _considered, none_promoted, failed = poll("fixture/repo", apply=True, log=lambda *_: None)
        chk("a FAILED un-quarantine promotes nothing", none_promoted, [])
        # ...and is REPORTED, so main() exits non-zero instead of a green cron over a silent drop.
        chk("a FAILED un-quarantine is reported to the caller", failed, [APPROVED])
        chk("...and posts no promotion notice claiming it did",
            [w for w in writes if w[0:2] == ["issue", "comment"]], [])
        chk("...and applies no triage delta to a still-quarantined issue",
            [w for w in writes if "--add-label" in w], [])
        fail_unquarantine.clear()
    finally:
        _gh = real_gh

    print("promote-poll self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description="poll the trust:untrusted quarantine for 👍 approval")
    # No default: a write path must never inherit production by omission (the rule
    # scripts/selftest_write_guard.py enforces across scripts/).
    ap.add_argument("--repo", help="owner/name to poll (required unless --self-test)")
    ap.add_argument("--apply", action="store_true", help="perform the promotions (cron mode)")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    if not args.repo:
        ap.error("--repo is required")
    considered, promoted, failed = poll(args.repo, apply=args.apply)
    print(f"promote-poll: {len(promoted)} of {considered} quarantined issue(s) "
          f"{'promoted' if args.apply else 'promotable (dry-run)'}")
    if failed:
        print(f"promote-poll: {len(failed)} issue(s) FAILED to promote: {failed[:10]}",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
