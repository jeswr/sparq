#!/usr/bin/env python3
# [FABLE-5] Auto-file a nightly metamorphic-lane finding (bead sq-3dyje.9).
#
# WHAT: the deterministic filing core of .github/workflows/metamorph.yml's failure
# path — the metamorphic sibling of scripts/ci-file-differential-failure.py (same
# structure, same safety properties, adapted to the single-engine TLP/NoREC
# oracles of crates/sparq-metamorph). When a metamorph-driver window reports a
# non-Pass verdict, this script:
#   1. PARSES the captured driver log — the `VIOLATION seed=N` /
#      `ENGINE-FAILURE seed=N` lines plus the "FIRST FAILING CASE:" block (seed +
#      replay command + all oracle queries + graph), a complete deterministic
#      repro (the generator is seeded SplitMix64 — the seed IS the corpus);
#   2. WRITES the parsed repro to --out-dir (uploaded as a CI artifact);
#   3. FILES a GitHub issue with the repro INLINE (via `gh`), deduped per shard:
#      if an open `[metamorph]` issue for the same shard already exists it
#      comments the fresh window/run instead of opening a duplicate;
#   4. APPENDS a P1 bug bead to .beads/issues.jsonl (append-only, deduped against
#      existing open metamorph beads for the shard). The workflow commits + pushes
#      this BEST-EFFORT (fail-soft on the GH013 protected-main ruleset rejection);
#      the GitHub issue is the reliable channel the orchestrator's issue sweep
#      beads from.
#
# SAFETY / HONESTY PROPERTIES (same as the differential filing script)
#   * Never edits or reorders existing JSONL lines (append-only).
#   * Bead ids are derived (sq-mm + hash of date+shard) and collision-checked.
#   * Idempotent per shard: a re-run neither opens a second issue nor appends a
#     second bead.
#   * `gh` failures degrade to warnings — filing must never mask the red lane.
#
# USAGE
#   scripts/ci-file-metamorph-failure.py --log metamorph-log.txt --shard nightly \
#       --seed-start N --count M --out-dir repro/ --jsonl .beads/issues.jsonl \
#       --run-url URL
#   scripts/ci-file-metamorph-failure.py --self-test   # hermetic; no gh, no writes
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

PROG = "ci-file-metamorph-failure"
MARKER = "[metamorph]"
# The GitHub label EVERY issue this filer opens carries. It is what SCOPES the dedupe
# listing in find_open_issue() to this filer's own issues, which is what lets that
# listing drop `--search` (#5804) and scan the titles in Python instead. Keep it equal
# to the bead label in build_bead_record() so the two records stay greppable together.
ISSUE_LABEL = "metamorph"

_FAIL_RE = re.compile(r"^(?:VIOLATION|ENGINE-FAILURE) seed=(\d+) ", re.MULTILINE)
_ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789"


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def parse_driver_log(text: str) -> dict:
    """Extract the failing-seed list, the FIRST FAILING CASE block, the summary."""
    # De-duplicate while preserving order (a seed may fail both oracles).
    seeds: list[int] = []
    for m in _FAIL_RE.findall(text):
        s = int(m)
        if s not in seeds:
            seeds.append(s)
    first_case = ""
    marker = "FIRST FAILING CASE:"
    if marker in text:
        first_case = text.split(marker, 1)[1].strip()
    summary = ""
    for line in text.splitlines():
        if line.startswith("metamorph seeds"):
            summary = line.strip()
    return {"seeds": seeds, "first_case": first_case, "summary": summary}


def derive_bead_id(shard: str, date: str, existing_ids: set, n: int = 5) -> str:
    """Deterministic, collision-checked bead id: sq-mm + hash(date+shard)."""
    digest = hashlib.sha256(f"metamorph:{date}:{shard}".encode()).hexdigest()
    chars = "".join(_ALPHABET[int(c, 16) % len(_ALPHABET)] for c in digest)
    while True:
        bead_id = f"sq-mm{chars[:n]}"
        if bead_id not in existing_ids:
            return bead_id
        n += 1


def existing_open_metamorph_bead(jsonl_path: Path, shard: str) -> str | None:
    """The id of an existing open metamorph bead for this shard, if any (dedupe:
    one standing bug bead per shard until it is closed)."""
    if not jsonl_path.exists():
        return None
    with open(jsonl_path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (
                rec.get("_type") == "issue"
                and rec.get("status") in ("open", "in_progress", "blocked")
                and MARKER in rec.get("title", "")
                and f"shard={shard}" in rec.get("title", "")
            ):
                return rec.get("id")
    return None


def all_bead_ids(jsonl_path: Path) -> set:
    ids = set()
    if not jsonl_path.exists():
        return ids
    with open(jsonl_path, encoding="utf-8") as fh:
        for line in fh:
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(rec, dict) and rec.get("id"):
                ids.add(rec["id"])
    return ids


def replay_command(parsed: dict, args) -> str:
    first = parsed["seeds"][0] if parsed["seeds"] else None
    if first is not None:
        return f"cargo run -p sparq-metamorph --bin metamorph-driver -- {first} 1"
    return (
        f"cargo run -p sparq-metamorph --bin metamorph-driver -- "
        f"{args.seed_start} {args.count}"
    )


def build_bead_record(bead_id: str, shard: str, parsed: dict, args, now: str) -> dict:
    """A minimal, schema-shaped bead line (mirrors .beads/issues.jsonl records)."""
    n = len(parsed["seeds"])
    first = parsed["seeds"][0] if parsed["seeds"] else "?"
    title = (
        f"[BUG] {MARKER} shard={shard}: {n} TLP/NoREC oracle failure(s), "
        f"first seed={first} (window {args.seed_start}+{args.count})"
    )
    description = (
        f"Auto-filed by the nightly metamorphic lane (sq-3dyje.9). Run: {args.run_url}\n"
        f"Failing seeds: {parsed['seeds'][:50]}{' …' if n > 50 else ''}\n"
        f"Summary: {parsed['summary']}\n"
        f"Replay: {replay_command(parsed, args)}\n"
        f"Repro (seed + queries + graph) is inline in the linked GitHub issue and in "
        f"the metamorph-repro artifact of the run. TLP/NoREC are SINGLE-ENGINE "
        f"metamorphic oracles: a VIOLATION is an internal-consistency wrong-result "
        f"signal in sparq itself (no cross-engine adjudication applies); an "
        f"ENGINE-FAILURE is a generated valid query that failed to evaluate. "
        f"🤖 SPARQ agent [FABLE-5]"
    )
    return {
        "_type": "issue",
        "id": bead_id,
        "title": title,
        "description": description,
        "status": "open",
        "priority": 1,
        "issue_type": "bug",
        "created_at": now,
        "created_by": "metamorph CI",
        "updated_at": now,
        "labels": ["metamorph", "ci"],
        "dependency_count": 0,
        "dependent_count": 0,
        "comment_count": 0,
    }


def append_bead(jsonl_path: Path, record: dict) -> None:
    """Append-only, newline-safe: never touches existing lines."""
    with open(jsonl_path, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, ensure_ascii=False) + "\n")


def build_issue_body(bead_id: str, shard: str, parsed: dict, args) -> str:
    n = len(parsed["seeds"])
    seeds_line = ", ".join(str(s) for s in parsed["seeds"][:50]) + (" …" if n > 50 else "")
    case = parsed["first_case"] or "(no FIRST FAILING CASE block captured — see the artifact log)"
    return f"""> 🤖 **SPARQ agent** — auto-filed by the nightly metamorphic lane (bead sq-3dyje.9). [FABLE-5]

The nightly TLP/NoREC metamorphic driver found **{n} failing seed(s)** in shard `{shard}` (seed window {args.seed_start}+{args.count}).

- **Failing seeds:** {seeds_line}
- **Deterministic replay:** `{replay_command(parsed, args)}`
- **Run:** {args.run_url}
- **Bead:** `{bead_id}` (P1, auto-appended; push best-effort)
- **Artifact:** `metamorph-repro-{shard}` on the run above (full log)

TLP/NoREC are **single-engine** metamorphic oracles (SQLancer's TLP/NoREC re-derived for SPARQL's three-valued EBV semantics): a `VIOLATION` means sparq's own evaluation paths are internally inconsistent — a wrong-result logic-bug signal that needs no cross-engine adjudication. An `ENGINE-FAILURE` means a generated, precondition-respecting query failed to evaluate (an engine-error bug or a harness bug). Both fail the lane; neither is ever skipped.

### First failing case (seed + replay + queries + graph)

```
{case}
```
"""


def gh(*argv: str) -> str:
    """Run gh, returning stdout; raises CalledProcessError on failure."""
    return subprocess.run(
        ["gh", *argv], check=True, capture_output=True, text=True
    ).stdout.strip()


def ensure_label() -> None:
    """Upsert ISSUE_LABEL (idempotent via --force). Needed because the dedupe listing and
    `issue create` below are both keyed on it: a label the repo has never seen makes
    `issue create --label` FAIL, and an issue that never carries the label is invisible
    to every future find_open_issue(). Fail-soft — a labelling problem must not sink a
    filing run; `issue create` below surfaces a genuine auth failure."""
    try:
        gh("label", "create", ISSUE_LABEL, "--color", "b60205",
           "--description", "TLP/NoREC metamorphic oracle failure (auto-filed)", "--force")
    except subprocess.CalledProcessError as e:
        log(f"warning: label upsert failed (non-fatal): {e.stderr.strip() if e.stderr else e}")


def select_open_issue(items: list[dict], shard: str) -> str | None:
    """PURE: the number of the first issue in `items` whose TITLE carries this filer's
    marker immediately followed by this shard's key AND the key's `:` delimiter, else
    None. Split out from the gh call so the dedupe predicate is exercised hermetically
    by --self-test.

    [SONNET-4.6] review round 2 of #5860: the match is the STRUCTURED PREFIX
    `"<MARKER> shard=<shard>:"`, not two independent substrings. Shard names are
    prefix-dense (`smoke` vs a future `smoke-extended`), so an unanchored
    `f"shard={shard}" in title` would comment this shard's oracle failures onto the
    longer shard's open issue AND skip creating the issue for the real failure,
    breaking one-rolling-issue-per-shard in both directions. Every title this filer
    writes is `f"{MARKER} shard={shard}: ..."`, so requiring the delimiter pins the
    key's right boundary and the space pins its left."""
    for item in items:
        title = item.get("title", "")
        if f"{MARKER} shard={shard}:" in title:
            return str(item["number"])
    return None


def open_issues_with_label(label: str) -> list[dict]:
    """EVERY open issue carrying `label`, via REAL cursor pagination.

    [OPUS-5] review of #5804: a single `gh issue list --limit N` page SILENTLY TRUNCATES
    — gh stops at the limit and reports nothing, no warning, no non-zero exit. "At most
    one open issue per shard" bounds the issues per SHARD; it does not bound the number
    of distinct shards under the label, so once the label carries more than a page of
    open issues the newest-first page drops the OLDEST ones and a real match becomes
    invisible — re-filing an issue we already filed, which is the very dedupe miss this
    listing exists to prevent. `gh api --paginate` follows the Link chain to exhaustion
    instead (same idiom as ci_execution_latency_alarm.py::file_issue). `{owner}`/`{repo}`
    resolve from the same repo context `gh issue list` used, so nothing about WHICH repo
    is listed changes.

    The REST issues endpoint returns PRs as well; they are dropped, so the rows handed
    back are the same shape `gh issue list --json number,title` produced."""
    out = gh("api", "--paginate", "--slurp",
             "repos/{owner}/{repo}/issues?state=open"
             f"&labels={label}&per_page=100")
    pages = json.loads(out or "[]")
    return [i for page in pages if isinstance(page, list) for i in page
            if isinstance(i, dict) and "pull_request" not in i]


def find_open_issue(shard: str) -> str | None:
    """Number of an existing open metamorph issue for this shard, if any.

    [OPUS-5] #5804: we deliberately do NOT pass `--search`. Two mechanisms in it both
    fail the same way — gh's search TOKENISER handles the query this used to build
    unreliably (a bracketed marker plus a `shard=<name>` key, quoted as ONE multi-token
    phrase), and the search INDEX LAGS, so an issue this
    lane filed minutes ago can be invisible on the next tick. Either one MISSES an
    existing issue, and a missed dedupe re-files an issue we already filed: the non-spam
    invariant failing open, quietly. Instead we enumerate ISSUE_LABEL EXHAUSTIVELY
    (open_issues_with_label — no page limit for an older match to fall off) and match the
    title in Python, where no tokeniser is involved. Same shape as
    ci_execution_latency_alarm.py::file_issue.

    TRANSITION: issues filed BEFORE ISSUE_LABEL was introduced carry no label and are
    invisible to this listing, so the first failure per shard after this change may mint
    one duplicate. That cost is one-time and self-healing (the replacement carries the
    label)."""
    try:
        return select_open_issue(open_issues_with_label(ISSUE_LABEL), shard)
    except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
        log(f"warning: issue dedupe listing failed ({e}) — will attempt creation")
    return None


def file_github_issue(bead_id: str, shard: str, parsed: dict, args) -> None:
    body = build_issue_body(bead_id, shard, parsed, args)
    n = len(parsed["seeds"])
    first = parsed["seeds"][0] if parsed["seeds"] else "?"
    title = (
        f"{MARKER} shard={shard}: {n} TLP/NoREC oracle failure(s) "
        f"(first seed={first})"
    )
    ensure_label()
    existing = find_open_issue(shard)
    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as tf:
        tf.write(body)
        body_file = tf.name
    try:
        if existing:
            gh("issue", "comment", existing, "--body-file", body_file)
            log(f"commented the fresh window on existing open issue #{existing} (deduped)")
        else:
            url = gh("issue", "create", "--title", title, "--body-file", body_file,
                     "--label", ISSUE_LABEL)
            log(f"filed GitHub issue: {url}")
    except subprocess.CalledProcessError as e:
        log(f"warning: gh issue filing failed (exit {e.returncode}): {e.stderr.strip() if e.stderr else e}")
        log("the lane is already red and the artifact carries the repro — not fatal.")
    finally:
        os.unlink(body_file)


def write_repro_artifact(out_dir: Path, parsed: dict, log_text: str, args) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "repro.md").write_text(
        f"# metamorph repro — shard={args.shard}\n\n"
        f"window: seeds {args.seed_start}+{args.count}\n"
        f"failing seeds: {parsed['seeds']}\n"
        f"summary: {parsed['summary']}\n\n"
        f"## first failing case\n\n```\n{parsed['first_case']}\n```\n",
        encoding="utf-8",
    )
    (out_dir / "metamorph-log.txt").write_text(log_text, encoding="utf-8")


# ── self-test (hermetic: no gh, no repo writes) ──────────────────────────────────
def self_test() -> int:
    sample = (
        "VIOLATION seed=42 oracle=tlp: tlp violation on [sparq]: partition union "
        "differs from base (base=9 true=3 false=3 error=2)\n"
        "VIOLATION seed=42 oracle=norec: norec violation on [sparq]: optimized "
        "cardinality differs from rewrite true-count (optimized=3 rewrite-rows=9 rewrite-true=4)\n"
        "ENGINE-FAILURE seed=57 oracle=tlp: engine failure [Evaluation] on sparq: parse error\n"
        "\nFIRST FAILING CASE:\nseed=42\n"
        "replay: cargo run -p sparq-metamorph --bin metamorph-driver -- 42 1\n"
        "--- queries ---\nSELECT * WHERE { ?s <http://example.org/v> ?v }\n"
        "--- graph ---\n<http://example.org/s0> <http://example.org/v> \"1\"^^<x> .\n"
        "metamorph seeds 0..2000: checked=2000 pass=1998 tlp-violation=1 "
        "norec-violation=1 engine-failure=1\n"
    )
    parsed = parse_driver_log(sample)
    assert parsed["seeds"] == [42, 57], parsed["seeds"]  # deduped, order-preserving
    assert "seed=42" in parsed["first_case"]
    assert "--- graph ---" in parsed["first_case"]
    assert parsed["summary"].startswith("metamorph seeds")
    # No-failure log parses to empty (a red lane with no VIOLATION lines — e.g. a
    # panic — still files, with the raw log as the payload).
    empty = parse_driver_log("metamorph seeds 0..10: checked=10 pass=10 ...")
    assert empty["seeds"] == [] and empty["first_case"] == ""

    with tempfile.TemporaryDirectory() as td:
        jsonl = Path(td) / "issues.jsonl"
        jsonl.write_text(
            json.dumps({"_type": "issue", "id": "sq-aaaaa", "title": "x", "status": "open"}) + "\n",
            encoding="utf-8",
        )
        ids = all_bead_ids(jsonl)
        assert ids == {"sq-aaaaa"}
        # Deterministic id derivation + collision extension.
        b1 = derive_bead_id("nightly", "2026-07-11", ids)
        b2 = derive_bead_id("nightly", "2026-07-11", ids)
        assert b1 == b2 and b1.startswith("sq-mm") and len(b1) == len("sq-mm") + 5
        b3 = derive_bead_id("nightly", "2026-07-11", ids | {b1})
        assert b3 != b1 and b3.startswith("sq-mm")

        class A:  # minimal args stand-in
            shard, seed_start, count = "nightly", "42", "2000"
            run_url = "https://example.invalid/run/1"

        assert replay_command(parsed, A).endswith("metamorph-driver -- 42 1")
        assert replay_command(empty, A).endswith("metamorph-driver -- 42 2000")
        rec = build_bead_record(b1, "nightly", parsed, A, "2026-07-11T00:00:00Z")
        assert rec["priority"] == 1 and rec["issue_type"] == "bug" and rec["status"] == "open"
        assert "sq-3dyje.9" in rec["description"] and "🤖" in rec["description"]
        append_bead(jsonl, rec)
        # Existing lines untouched; the new line parses; dedupe now finds it.
        lines = jsonl.read_text(encoding="utf-8").splitlines()
        assert len(lines) == 2 and json.loads(lines[0])["id"] == "sq-aaaaa"
        assert json.loads(lines[1])["id"] == b1
        assert existing_open_metamorph_bead(jsonl, "nightly") == b1
        assert existing_open_metamorph_bead(jsonl, "smoke") is None
        # Issue body carries the inline repro + the self-id.
        body = build_issue_body(b1, "nightly", parsed, A)
        assert "🤖" in body and "SPARQ agent" in body
        assert "--- graph ---" in body and "42" in body
        assert "single-engine" in body  # honest oracle framing, not "vs Oxigraph"
        # Artifact writer round-trip.
        out = Path(td) / "repro"
        write_repro_artifact(out, parsed, sample, argparse.Namespace(
            shard="nightly", seed_start="42", count="2000"))
        assert (out / "repro.md").exists() and (out / "metamorph-log.txt").exists()
    # [OPUS-5] #5804: the dedupe PREDICATE, over the shape `gh issue list --label` returns.
    listed = [
        {"number": 7, "title": "[metamorph] shard=bgp: 1 TLP/NoREC oracle failure(s)"},
        {"number": 42, "title": f"{MARKER} shard=nightly: 3 TLP/NoREC oracle failure(s) (first seed=42)"},
    ]
    assert select_open_issue(listed, "nightly") == "42"      # matched => COMMENT, no duplicate
    assert select_open_issue(listed, "bgp") == "7"
    assert select_open_issue(listed, "optional") is None     # unmatched => file a new one
    assert select_open_issue([], "nightly") is None
    # A same-shard title from a DIFFERENT filer must not be mistaken for ours.
    assert select_open_issue([{"number": 9, "title": "[differential-fuzz] shard=nightly: x"}],
                             "nightly") is None
    # A title missing the key is not a match even when the marker is present.
    assert select_open_issue([{"number": 9, "title": f"{MARKER} no shard here"}],
                             "nightly") is None
    # [SONNET-4.6] review round 2 of #5860: KEY BOUNDARY. Shard names are prefix-dense
    # (`smoke` vs a future `smoke-extended`), so a requested shard must NOT match a
    # LONGER shard that merely starts with it — that would comment this shard's oracle
    # failures onto the other shard's issue AND skip filing the issue for the real
    # failure. The `:` delimiter pins the right boundary.
    prefixy = [{"number": 11,
                "title": f"{MARKER} shard=smoke-extended: 1 TLP/NoREC oracle failure(s)"}]
    assert select_open_issue(prefixy, "smoke") is None
    assert select_open_issue(prefixy, "smoke-extended") == "11"   # exact still matches
    # [OPUS-5] review of #5804: the LISTING must not silently truncate. "One open issue
    # per shard" does not bound the number of distinct shards under the label, so the label
    # really can carry more than a page. The stub emulates BOTH gh shapes faithfully —
    # `api --paginate` exhausts the page chain, `issue list --limit N` keeps only the
    # NEWEST N — so reverting find_open_issue() to the single-page call is EXECUTABLE
    # here and fails on the MISSING ROW rather than on an unrecognised command line.
    real_gh = globals()["gh"]
    corpus = [{"number": n, "title": f"{MARKER} shard=filler{n}: older finding"}
              for n in range(1, 251)]
    corpus[2]["title"] = f"{MARKER} shard=oldest: older finding"   # the match, LAST page
    newest_first = sorted(corpus, key=lambda r: -r["number"])

    def stub_gh(*argv):
        if argv[0] == "api":
            assert "--paginate" in argv, argv   # one page is not an exhaustive listing
            return json.dumps([newest_first[i:i + 100]
                               for i in range(0, len(newest_first), 100)])
        if list(argv[:2]) == ["issue", "list"]:
            return json.dumps(newest_first[:int(argv[argv.index("--limit") + 1])])
        raise AssertionError(f"unexpected gh invocation: {argv}")

    globals()["gh"] = stub_gh
    try:
        assert find_open_issue("oldest") == "3", (
            "the dedupe listing must reach PAST the first page — a truncated listing "
            "reports 'no open issue' and re-files one that is already open")
        assert find_open_issue("absent") is None
    finally:
        globals()["gh"] = real_gh
    # [SONNET-4.6] review round 4 of #5860: ensure_label() is a GUARD and was pinned by
    # nothing. Both the dedupe listing and `issue create --label` are keyed on
    # ISSUE_LABEL, so if the upsert stops running, the create fails against a repo that
    # has never carried the label and the oracle failure is filed NOWHERE. Drive the
    # REAL filing path with a recording stub and pin the three properties that make it
    # a guard: it runs, it runs BEFORE the create, and it upserts idempotently.
    calls: list[tuple[str, ...]] = []

    def recording_gh(*argv):
        calls.append(argv)
        if argv[0] == "api":
            return "[]"                      # no open issue => take the CREATE path
        return "https://example.invalid/issues/1"

    globals()["gh"] = recording_gh
    try:
        file_github_issue(b1, "nightly", parsed, A)
    finally:
        globals()["gh"] = real_gh
    verbs = [tuple(c[:2]) for c in calls]
    assert ("label", "create") in verbs, (
        "ensure_label() must run on the filing path — without the upsert, "
        f"`issue create --label {ISSUE_LABEL}` fails on a repo that has never seen "
        f"the label and the oracle failure is filed nowhere: {verbs}")
    assert ("issue", "create") in verbs, verbs
    assert verbs.index(("label", "create")) < verbs.index(("issue", "create")), (
        f"the label must be upserted BEFORE the issue that carries it: {verbs}")
    upsert = calls[verbs.index(("label", "create"))]
    assert ISSUE_LABEL in upsert and "--force" in upsert, (
        "the upsert must name ISSUE_LABEL and be idempotent — without --force the "
        f"second filing run errors on the already-existing label: {upsert}")
    # ...and it must be FAIL-SOFT: a repo where labelling is refused must still get the
    # issue. (Letting the CalledProcessError escape ensure_label makes this go red.)
    calls.clear()

    def label_refused_gh(*argv):
        calls.append(argv)
        if argv[0] == "label":
            raise subprocess.CalledProcessError(1, "gh", stderr="label: forbidden")
        if argv[0] == "api":
            return "[]"
        return "https://example.invalid/issues/2"

    globals()["gh"] = label_refused_gh
    try:
        file_github_issue(b1, "nightly", parsed, A)
    finally:
        globals()["gh"] = real_gh
    assert ("issue", "create") in [tuple(c[:2]) for c in calls], (
        "a REFUSED label upsert must not sink the filing run — the oracle failure "
        f"still has to reach an issue: {calls}")
    log("self-test OK")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(prog=PROG)
    ap.add_argument("--log", help="captured driver stdout+stderr")
    ap.add_argument("--shard", help="workflow shard name (smoke | nightly)")
    ap.add_argument("--seed-start", default="unknown")
    ap.add_argument("--count", default="unknown")
    ap.add_argument("--out-dir", help="repro artifact directory")
    ap.add_argument("--jsonl", default=".beads/issues.jsonl")
    ap.add_argument("--run-url", default="")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if not args.log or not args.shard or not args.out_dir:
        ap.error("--log, --shard and --out-dir are required (or use --self-test)")

    log_text = Path(args.log).read_text(encoding="utf-8", errors="replace")
    parsed = parse_driver_log(log_text)
    log(f"parsed {len(parsed['seeds'])} failing seed(s) for shard {args.shard}")

    write_repro_artifact(Path(args.out_dir), parsed, log_text, args)

    jsonl = Path(args.jsonl)
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    existing = existing_open_metamorph_bead(jsonl, args.shard)
    if existing:
        log(f"open metamorph bead {existing} already tracks shard {args.shard} — no new bead")
        bead_id = existing
    else:
        bead_id = derive_bead_id(args.shard, now[:10], all_bead_ids(jsonl))
        append_bead(jsonl, build_bead_record(bead_id, args.shard, parsed, args, now))
        log(f"appended P1 bug bead {bead_id} to {jsonl}")

    file_github_issue(bead_id, args.shard, parsed, args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
