#!/usr/bin/env python3
# [OPUS-4.8] Trust promotion: decide if a quarantined (trust:untrusted) item may be promoted.
"""promote.py — revision-bound 👍 promotion of third-party content.

GitHub Actions has NO reaction event, so a cron polls quarantined issues and calls this. An item is
promoted iff a MAINTAINER (write/maintain/admin) added a 👍 reaction **after** the item's last edit —
so editing the body after approval invalidates it (the review's S3 TOCTOU fix). Reactions before the
last edit, or from non-maintainers, do not promote.

Pure `should_promote()` is unit-tested; the cron supplies reactions + updated_at + the maintainer set.
`fetch_quarantined()` is the queue read the cron feeds it from — tested here for the same reason, see
its docstring.
"""
import datetime
import json
import sys
import urllib.parse


def should_promote(reactions, updated_at, maintainers):
    """`reactions`: list of {content, user, created_at (int)}. `updated_at` (int): the item's last
    edit time. `maintainers`: set of trusted logins. Promote iff some 👍 from a maintainer was created
    at/after the last edit."""
    mset = {m.lower() for m in maintainers}
    for r in reactions:
        if (r.get("content") in ("+1", "\U0001F44D")  # '+1' is the API name for 👍
                and str(r.get("user", "")).lower() in mset
                and int(r.get("created_at", 0)) >= int(updated_at)):
            return True
    return False


QUARANTINE_LABEL = "trust:untrusted"
FETCH_CEILING = 10000


def epoch(ts):
    """ISO-8601 (`...Z`) -> unix seconds. Raises on a missing/unparseable stamp rather than
    returning 0: `should_promote` compares the 👍 against this value, so a silent 0 would make
    EVERY historical maintainer 👍 count as post-edit and disarm the revision-binding entirely."""
    if not ts:
        raise ValueError("promote: missing timestamp — refusing to treat it as epoch 0, which "
                         "would promote on any stale 👍")
    return int(datetime.datetime.fromisoformat(ts.replace("Z", "+00:00")).timestamp())


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output, dropping PRs (the issues endpoint returns
    both) and any non-list/non-dict junk."""
    return [i for page in pages if isinstance(page, list) for i in page
            if isinstance(i, dict) and "pull_request" not in i]


def fetch_quarantined(repo, gh, label=QUARANTINE_LABEL, ceiling=FETCH_CEILING):
    """Every OPEN issue carrying `label`, via REAL cursor pagination.

    [OPUS-5] #5425, the remainder of the #4985 `--limit` sweep. This lived inline in
    `.github/workflows/promote-on-approval.yml` as `gh issue list --limit 200`, which SILENTLY
    TRUNCATES: the CLI stops at the limit newest-first and reports nothing. Quarantine is a WORK
    QUEUE the cron polls, so truncation is not a slow drain — once the open `trust:untrusted` set
    passes the limit, the OLDEST quarantined issues fall off the end of every tick and are never
    polled for approval again, at any point in the future, with no line in the run log. A
    maintainer's 👍 on one of them does nothing, forever.

    The defect is DEPTH-CONDITIONAL: below the limit the truncated and paginated fetches return
    identical rows, and it arms the moment the queue crosses the limit. That is why it is fixed
    rather than tuned. No live queue depth was measured here, so no gain is claimed — the guard
    below demonstrates the shape of the loss on a fixture, not on production.

    `gh api --paginate` follows Link headers to exhaustion; the explicit ceiling still fails
    closed on a runaway snapshot (same shape as scripts/retriage.py `_fetch_label`). It was moved
    out of the workflow because inline heredoc python has no test harness — the guards below are
    only executable from a script.

    Rows are normalised to `{number, updated_at, labels}` with DIRECT indexing, because the REST
    payload does not name its fields the way `gh issue list --json` does: REST says `updated_at`,
    the CLI says `updatedAt`. Reading the CLI name off a REST row through `.get()` would yield
    None -> `epoch()` -> 0, and a 0 last-edit time promotes on any stale 👍. Direct indexing makes
    that mapping error a KeyError instead; `_self_test` pins it behaviourally through
    `should_promote`, since asserting the row SHAPE alone would not catch it.
    """
    r = gh("api", "--paginate", "--slurp",
           f"repos/{repo}/issues?state=open&labels={urllib.parse.quote(label)}&per_page=100")
    if r.returncode != 0:
        raise SystemExit(f"promote: could not list open '{label}' issues")
    rows = _flatten_pages(json.loads(r.stdout or "[]"))
    if len(rows) >= ceiling:
        raise SystemExit(f"promote: fetched {len(rows)} open '{label}' issues >= ceiling {ceiling} "
                         "— snapshot looks runaway (fail-closed). Raise the ceiling deliberately.")
    return [{"number": i["number"], "updated_at": i["updated_at"], "labels": i["labels"]}
            for i in rows]


def _self_test():
    ok = True

    def chk(n, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {n}: {got} (want {want})")

    M = {"jeswr"}
    chk("maintainer 👍 after edit", should_promote([{"content": "+1", "user": "jeswr", "created_at": 100}], 90, M), True)
    chk("👍 BEFORE edit (revoked)", should_promote([{"content": "+1", "user": "jeswr", "created_at": 80}], 90, M), False)
    chk("non-maintainer 👍", should_promote([{"content": "+1", "user": "rando", "created_at": 100}], 90, M), False)
    chk("wrong reaction", should_promote([{"content": "heart", "user": "jeswr", "created_at": 100}], 90, M), False)
    chk("no reactions", should_promote([], 90, M), False)
    chk("exactly at edit time", should_promote([{"content": "+1", "user": "jeswr", "created_at": 90}], 90, M), True)

    # --- the quarantine QUEUE READ must not silently truncate (#5425 / the #4985 sweep) --------
    # `gh issue list --limit 200` stops at the limit newest-first and reports nothing, so the
    # OLDEST quarantined issues are never polled for approval again. The stub below emulates BOTH
    # CLI shapes faithfully — `gh api --paginate` exhausts the Link chain, `gh issue list --limit
    # N` truncates newest-first and emits camelCase `updatedAt` — so reverting to the limited call
    # is EXECUTABLE here and fails on the missing ROWS rather than on an unrecognised command.
    # That is what makes this a behavioural guard and not a spelling test.
    class _Resp:
        def __init__(self, stdout, returncode=0):
            self.stdout, self.returncode = stdout, returncode

    EDITED_AT = "2026-01-02T00:00:00Z"          # every quarantined row's last edit
    QUEUE = 250                                 # > the 200 the workflow used to ask for
    corpus = [{"number": n, "updated_at": EDITED_AT,
               "labels": [{"name": "trust:untrusted"}, {"name": "kind:bug"}]}
              for n in range(1, QUEUE + 1)]
    # the issues endpoint also returns PRs, and rows that do not carry the label
    corpus.append({"number": 900, "updated_at": EDITED_AT, "pull_request": {"url": "x"},
                   "labels": [{"name": "trust:untrusted"}]})
    corpus.append({"number": 901, "updated_at": EDITED_AT, "labels": [{"name": "kind:bug"}]})
    seen_cmds = []

    def _labels_of(row):
        return {lb["name"] for lb in row["labels"]}

    def _stub_gh(*args):
        seen_cmds.append(list(args))
        if args[0] == "api":
            query = args[-1]
            rows = list(corpus)
            if "labels=" in query:
                want = urllib.parse.unquote(query.split("labels=")[1].split("&")[0])
                rows = [r for r in rows if want in _labels_of(r)]
            pages = [rows[i:i + 100] for i in range(0, len(rows), 100)] or [[]]
            return _Resp(json.dumps(pages if "--paginate" in args else pages[:1]))
        if list(args[0:2]) == ["issue", "list"]:
            want = args[args.index("--label") + 1]
            rows = [r for r in corpus if want in _labels_of(r) and "pull_request" not in r]
            limit = int(args[args.index("--limit") + 1]) if "--limit" in args else len(rows)
            return _Resp(json.dumps(  # the CLI spells it `updatedAt`, the REST payload `updated_at`
                [{"number": r["number"], "updatedAt": r["updated_at"], "labels": r["labels"]}
                 for r in sorted(rows, key=lambda r: -r["number"])[:limit]]))
        raise AssertionError(f"unexpected gh invocation: {args}")

    got = fetch_quarantined("o/r", _stub_gh)
    chk("the whole quarantine queue is polled, past any page limit",
        len(got), QUEUE)
    chk("the OLDEST quarantined issue is reached (what --limit dropped first)",
        min(r["number"] for r in got), 1)
    chk("PRs and unlabelled issues are dropped",
        sorted({900, 901} & {r["number"] for r in got}), [])
    chk("the fetch uses cursor pagination, not a fixed page limit",
        all(c[0] == "api" and "--paginate" in c for c in seen_cmds), True)
    # The field mapping is pinned END-TO-END, not by shape: a row whose last-edit time was lost
    # (REST `updated_at` misread as the CLI's `updatedAt` -> None -> epoch 0) promotes on a 👍
    # that predates the edit, silently disarming the revision-binding this script exists for.
    stale = [{"content": "+1", "user": "jeswr", "created_at": epoch(EDITED_AT) - 3600}]
    chk("a 👍 predating the fetched row's last edit does NOT promote",
        should_promote(stale, epoch(got[0]["updated_at"]), M), False)
    chk("...and a 👍 after it does", should_promote(
        [{"content": "+1", "user": "jeswr", "created_at": epoch(EDITED_AT) + 1}],
        epoch(got[0]["updated_at"]), M), True)
    chk("labels survive the fetch (the triage delta reads them)",
        sorted(lb["name"] for lb in got[0]["labels"]), ["kind:bug", "trust:untrusted"])
    try:
        epoch(None)
        chk("a missing timestamp raises rather than reading as epoch 0", "no raise", "ValueError")
    except ValueError:
        chk("a missing timestamp raises rather than reading as epoch 0", "ValueError", "ValueError")
    # fail-closed ceiling: a runaway snapshot must raise, never silently half-report
    try:
        fetch_quarantined("o/r", _stub_gh, ceiling=10)
        chk("runaway snapshot fails closed", "no raise", "SystemExit")
    except SystemExit:
        chk("runaway snapshot fails closed", "SystemExit", "SystemExit")
    # a failed read must raise too, not be flattened into an empty (== "nothing to promote") queue
    try:
        fetch_quarantined("o/r", lambda *a: _Resp("", returncode=1))
        chk("a failed queue read fails closed", "no raise", "SystemExit")
    except SystemExit:
        chk("a failed queue read fails closed", "SystemExit", "SystemExit")

    print("promote self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(_self_test() if "--self-test" in sys.argv else 0)
