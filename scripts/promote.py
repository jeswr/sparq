#!/usr/bin/env python3
# [OPUS-4.8] Trust promotion: decide if a quarantined (trust:untrusted) item may be promoted.
"""promote.py — revision-bound 👍 promotion of third-party content.

GitHub Actions has NO reaction event, so a cron polls quarantined issues and calls this. An item is
promoted iff a MAINTAINER (write/maintain/admin) added a 👍 reaction **after** the item's last edit —
so editing the body after approval invalidates it (the review's S3 TOCTOU fix). Reactions before the
last edit, or from non-maintainers, do not promote.

Pure `should_promote()` is unit-tested; the cron supplies updated_at + the maintainer set, and reads
the reactions through `fetch_reactions()` (also self-tested, over an injected `gh`).
"""
import datetime
import json
import subprocess
import sys


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


def _gh(*a):
    return subprocess.run(["gh", *a], capture_output=True, text=True)


def _epoch(ts):
    return int(datetime.datetime.fromisoformat(str(ts).replace("Z", "+00:00")).timestamp()) if ts else 0


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output: a LIST OF PAGES, each page a list of rows.

    A bare (unpaginated) `gh api` response is a flat list of rows instead, and every row is a dict,
    not a list — so it flattens to nothing here. That is deliberate: the shape mismatch is loud in
    the self-test rather than quietly degrading back to one page.
    """
    return [r for page in pages if isinstance(page, list) for r in page if isinstance(r, dict)]


def fetch_reactions(repo, number, gh=_gh):
    """Every reaction on issue `number`, normalised to the rows `should_promote` reads.

    [OPUS-5] #5802. This was `gh api repos/{repo}/issues/{n}/reactions` — no `--paginate`, so it
    returned only the FIRST page (30 rows) and silently reported nothing about the rest. Promotion
    scans those rows for a maintainer 👍 that post-dates the last edit, so on any issue carrying
    more than one page of reactions the approving 👍 could sit on page 2 and the issue would stay
    quarantined forever, with no error to notice. Same fail-quiet shape as the `gh issue list
    --limit` truncation on the queue read (#5425), one endpoint over.

    `--paginate` follows Link headers to exhaustion and `--slurp` collects the pages into one JSON
    array; `per_page=100` just makes that fewer round-trips. A non-zero exit is FAIL-CLOSED (raise,
    do not promote): with `--paginate` a mid-pagination failure leaves a truncated body, and
    treating that as the full reaction set is exactly the bug being fixed. The cron re-runs every
    ten minutes, so a transient failure costs one tick.
    """
    r = gh("api", "--paginate", "--slurp",
           f"repos/{repo}/issues/{number}/reactions?per_page=100")
    if r.returncode != 0:
        raise SystemExit(f"promote: could not read reactions on #{number} (fail-closed)")
    return [{"content": row.get("content"),
             "user": (row.get("user") or {}).get("login", ""),
             "created_at": _epoch(row.get("created_at"))}
            for row in _flatten_pages(json.loads(r.stdout or "[]"))]


class _R:
    """Minimal stand-in for `subprocess.CompletedProcess` (self-test only)."""

    def __init__(self, returncode, stdout):
        self.returncode, self.stdout = returncode, stdout


def _fake_gh(pages, returncode=0, seen=None):
    """A `gh` that models the REAL CLI's page behaviour: it hands back every page only when asked
    to paginate and slurp, and otherwise just the first page as a flat list of rows — so dropping
    either flag from `fetch_reactions` reds the multi-page fixture instead of passing vacuously."""
    def gh(*a):
        if seen is not None:
            seen.append(a)
        paginated = "--paginate" in a and "--slurp" in a
        return _R(returncode, json.dumps(pages if paginated else (pages[0] if pages else [])))
    return gh


def _rx(login, created_at, content="+1"):
    return {"content": content, "user": {"login": login}, "created_at": created_at}


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

    # [OPUS-5] #5802 — the reaction read must exhaust pagination. PAGE 1 is 30 non-approving
    # reactions; the ONLY maintainer 👍 is on PAGE 2. A single-page read sees page 1 only and
    # `should_promote` returns False, so this guard reds the moment `--paginate`/`--slurp` is
    # dropped or `_flatten_pages` stops flattening.
    p1 = [_rx(f"rando{i}", "2026-01-01T00:00:00Z", "heart") for i in range(30)]
    p2 = [_rx("jeswr", "2026-02-02T00:00:00Z")]
    seen = []
    rows = fetch_reactions("o/r", 7, _fake_gh([p1, p2], seen=seen))
    chk("reads BOTH pages", len(rows), 31)
    chk("page-2 👍 promotes", should_promote(rows, _epoch("2026-02-01T00:00:00Z"), M), True)
    last = rows[-1] if rows else {}  # tolerate the empty read so a single-page regression prints the WHOLE table
    chk("normalises user.login", last.get("user"), "jeswr")
    chk("normalises created_at to epoch", last.get("created_at"), _epoch("2026-02-02T00:00:00Z"))
    chk("asks for 100 per page", any("per_page=100" in str(x) for x in seen[0]), True)
    # A page-2 👍 that PRE-dates the last edit still must not promote: pagination widens the
    # read, it does not relax the revision binding.
    chk("page-2 👍 before edit (revoked)",
        should_promote(fetch_reactions("o/r", 7, _fake_gh([p1, p2])), _epoch("2026-03-03T00:00:00Z"), M), False)
    chk("empty pages", fetch_reactions("o/r", 7, _fake_gh([[]])), [])

    # Fail-CLOSED: a failed read must raise, never look like "no reactions" (which promotes nothing
    # but, worse, would hide a truncated body behind a silent False).
    try:
        fetch_reactions("o/r", 7, _fake_gh([p1, p2], returncode=1))
        chk("non-zero gh exit raises", "returned", "SystemExit")
    except SystemExit:
        chk("non-zero gh exit raises", "SystemExit", "SystemExit")

    print("promote self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(_self_test() if "--self-test" in sys.argv else 0)
