#!/usr/bin/env python3
# [OPUS-4.8] Trust promotion: decide if a quarantined (trust:untrusted) item may be promoted.
"""promote.py — revision-bound 👍 promotion of third-party content.

GitHub Actions has NO reaction event, so a cron polls quarantined issues and calls this. An item is
promoted iff a MAINTAINER (write/maintain/admin) added a 👍 reaction **after** the item's last edit —
so editing the body after approval invalidates it (the review's S3 TOCTOU fix). Reactions before the
last edit, or from non-maintainers, do not promote.

Pure `should_promote()` is unit-tested; `fetch_reactions()` (injected `gh`) does the paginated read
the cron feeds it, and the cron supplies updated_at + the maintainer set.
"""
import datetime
import json
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


def epoch(ts):
    """ISO-8601 (`...Z`) timestamp -> int epoch seconds; 0 for a missing/empty one."""
    return int(datetime.datetime.fromisoformat(ts.replace("Z", "+00:00")).timestamp()) if ts else 0


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output (a LIST OF PAGES) into one row list, dropping
    non-list/non-dict junk. Same shape as scripts/retriage.py `_flatten_pages`, minus the PR filter
    (the reactions endpoint returns reactions only)."""
    return [r for page in pages if isinstance(page, list) for r in page if isinstance(r, dict)]


def fetch_reactions(repo, number, gh):
    """Every reaction on issue `number`, normalized to the rows `should_promote` reads.

    [OPUS-5] #5802. This was `gh api repos/{repo}/issues/{n}/reactions`, which returns ONLY THE
    FIRST PAGE (30 rows by default) and reports nothing about the rest. `should_promote` scans
    these rows for a maintainer 👍 that post-dates the last edit, so once an issue accumulates
    more than a page of reactions the approving 👍 can sit on a later page and the issue stays
    quarantined on every future tick, with no error — the same fail-quiet shape #5425 reports for
    this cron's queue read, one endpoint over. `--paginate` follows Link headers to exhaustion;
    `--slurp` makes the output a list of pages, hence `_flatten_pages`.

    DEPTH-CONDITIONAL, like the truncation fixes it mirrors: below one page the old and new reads
    return the same rows. It is fixed rather than tuned because it is silent when it does bite.

    `gh` is injected (the cron's `subprocess.run` wrapper) so `--self-test` can drive this end to
    end over a fixture whose only maintainer 👍 lands on page 2. A failed read returns no rows —
    the safe direction (no promotion) — but says so on stderr rather than passing for "no 👍".
    """
    r = gh("api", "--paginate", "--slurp", f"repos/{repo}/issues/{number}/reactions?per_page=100")
    if r.returncode != 0:
        print(f"promote: could not read reactions on #{number}; not promoting this tick", file=sys.stderr)
        return []
    return [{"content": x.get("content"),
             "user": (x.get("user") or {}).get("login", ""),
             "created_at": epoch(x.get("created_at"))}
            for x in _flatten_pages(json.loads(r.stdout or "[]"))]


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

    # [OPUS-5] #5802 — the reaction READ must paginate. `fake_gh` mimics real `gh api`: it serves
    # every page only when the call passes `--paginate --slurp`, and page 1 alone otherwise, so
    # reverting fetch_reactions to the unpaginated read reds the four checks below (measured).
    class _R:  # CompletedProcess-shaped stub
        def __init__(self, returncode, stdout):
            self.returncode, self.stdout = returncode, stdout

    def rx(login, ts, content="+1"):
        # ONLY the keys the REST payload carries — a fixture with a flat `user` string would
        # satisfy the wrong read equally and let that mutant survive (retriage.py's field note).
        return {"content": content, "user": {"login": login}, "created_at": ts}

    pages = [[rx(f"rando{i}", "2026-01-02T00:00:00Z") for i in range(30)],  # a full first page…
             [rx("jeswr", "2026-01-03T00:00:00Z")]]                        # …the only 👍 that counts
    edit = epoch("2026-01-01T00:00:00Z")
    calls = []

    def fake_gh(*a):
        calls.append(a)
        return _R(0, json.dumps(pages if ("--paginate" in a and "--slurp" in a) else pages[0]))

    rows = fetch_reactions("o/r", 7, fake_gh)
    chk("paginated read returns every page", len(rows), 31)
    chk("reaction user flattened to login", rows[-1]["user"] if rows else None, "jeswr")
    chk("👍 on page 2 promotes", should_promote(rows, edit, M), True)
    chk("read asks for per_page=100", any("per_page=100" in x for x in calls[-1]), True)
    # the fixture is non-vacuous only if page 1 alone carries no maintainer 👍 — pin that, else the
    # check above would pass on the single-page read it exists to reject.
    chk("page 1 alone does NOT promote (#5802 defect)",
        should_promote(fetch_reactions("o/r", 7, lambda *a: _R(0, json.dumps(pages[0]))), edit, M), False)
    chk("failed read yields no rows", fetch_reactions("o/r", 7, lambda *a: _R(1, "")), [])

    print("promote self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(_self_test() if "--self-test" in sys.argv else 0)
