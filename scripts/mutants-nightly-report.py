#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4820 — NIGHTLY MUTATION-LANE LEG REPORT. 🤖 SPARQ agent.
#
# THE DEFECT THIS EXISTS FOR. ci.yml's `mutants-nightly-advisory` matrix is, per #4820's
# measurement, the largest single consumer of nightly CI capacity in this repo — and its
# outcome is unreadable. Three layers compose:
#   1. The lane is ADVISORY (declared in .github/advisory-registry.json), so nothing it
#      finds can ever appear red on a PR. Correct by design.
#   2. The workflow RUN conclusion is a roll-up over every sibling job in ci.yml, and
#      #4820 measured a night whose 62 successful and 10 FAILING jobs reported
#      `cancelled` — the same status as a run killed at startup, because six siblings
#      were cancelled and that outranks the failures in the roll-up.
#   3. The legs are inside a 51-entry matrix (24 whole-crate + 3 sparq-reason shards +
#      24 sparq-engine shards), so per-leg conclusions are invisible without expanding it.
#
# AND A FOURTH, READABLE STRAIGHT OFF ci.yml — the one that decides what this script
# must report. The `Run cargo-mutants … + ratchet against the baseline` STEP carries
# `continue-on-error: true`, and so does the JOB. `scripts/mutants-gate.py --check` runs
# INSIDE that step. So a ratchet regression — MORE surviving mutants than the committed
# ceiling, i.e. the actual test-QUALITY finding this lane exists to produce — CANNOT
# change a leg's conclusion. A red leg therefore never means "mutants survived"; it means
# the leg did not get to the end (job timeout, runner shutdown, or a setup step failing).
# The budget signal and the quality signal are not merely hard to see, they are NOT
# DISTINGUISHABLE from the job conclusions at all.
#
# WHAT THIS SCRIPT DOES. Given (a) this run's own jobs, as the Actions API returns them,
# and (b) the `mutants-outcomes-nightly-*` artifacts the legs uploaded, it renders ONE
# markdown report that states the two signals SEPARATELY:
#   * COMPLETION — per-leg conclusion + wall-clock, and per-leg EVIDENCE state derived
#     from the artifact (complete / truncated / unverifiable / absent). Truncation matters
#     because it is silent: cargo-mutants writes outcomes.json incrementally, so a leg
#     that ran out of budget leaves a well-formed artifact indistinguishable from a
#     complete run of a smaller crate (the reasoning is scripts/mutants-gate.py's, and
#     this script calls THAT module's completeness() rather than restating it).
#   * COST — summed leg wall-clock, i.e. the runner-hours the lane actually billed
#     tonight, measured rather than asserted. Every sizing comment in ci.yml's matrix
#     says to RE-TUNE from "the first run that completes, using that run's own timings as
#     the evidence of record". This is where that evidence lands.
# The ratchet VERDICT is deliberately NOT computed here: the report job runs
# `scripts/mutants-gate.py --check` over every downloaded artifact at once, which is the
# only place a sub-sharded crate's shards are summed into a whole-crate figure.
#
# WHAT IT DOES NOT DO. It cannot change the workflow run's conclusion — that roll-up is
# GitHub's and a cancelled sibling wins it. What it changes is that the lane's own state
# is now carried by a SEPARATE, NAMED check-run whose conclusion means one specific
# thing, instead of being inferred from a run-level status that reads "ignore me".
#
# NON-GATING. This runs in an advisory-declared job, so it is safe to RED — and it is
# BUILT to red while the lane is broken, because a report that stays green while ten legs
# time out would reproduce the exact failure it was written for.
#
# Usage:
#   mutants-nightly-report.py --jobs jobs.ndjson --artifacts DIR   # render + verdict
#   mutants-nightly-report.py --self-test                          # hermetic; no network
# Exit 0 = every leg completed and produced complete evidence; 1 = otherwise.
# stdlib-only (plus scripts/mutants-gate.py, loaded from disk).
from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import sys
from datetime import datetime

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

# The matrix job's `name:` in .github/workflows/ci.yml. GitHub renders a matrix leg as
# "<name> (<value>, <value>, …)", so the legs of that job — and ONLY those — are the
# check-runs whose name starts with this string followed by " (". The report job's own
# name ("mutation ratchet report (…)") deliberately does not match, so it never counts
# itself. scripts/tests/test_mutants_nightly_report.py pins this constant against the
# live YAML so a rename of the matrix job cannot silently empty this report.
DEFAULT_LEG_PREFIX = "mutation ratchet (cargo-mutants, advisory)"

# A matrix leg's parenthesised suffix is the matrix values joined with ", ". Two of those
# values identify the artifact the leg uploaded: the crate (a `sparq-*` package name) and,
# for a sub-sharded crate, the `k/n` shard. Anything else in the suffix (a feature list, a
# timeout override) is not identity and is ignored.
_CRATE_RE = re.compile(r"^sparq-[a-z0-9-]+$")
_SHARD_RE = re.compile(r"^(\d+)/(\d+)$")

# Conclusions that mean "this leg is not evidence of anything". `skipped` is excluded from
# the red set on purpose: a leg the `if:` guard skipped never claimed to measure a crate.
_NOT_RUN = {"skipped", None, ""}


def _load_mutants_gate():
    """Import scripts/mutants-gate.py (hyphenated => not importable by name). Reusing its
    completeness()/crate_of()/shard_of()/summarise() is deliberate: truncation detection is
    the subtle half of reading these artifacts and there must be exactly one copy of it."""
    path = os.path.join(REPO_ROOT, "scripts", "mutants-gate.py")
    spec = importlib.util.spec_from_file_location("mutants_gate", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def parse_jobs(text):
    """Jobs as the Actions API returns them, tolerant of every shape `gh api` emits.

    `gh api --paginate .../jobs` yields one `{"jobs": [...]}` object PER PAGE, and with
    `--jq '.jobs[]'` it yields bare job objects that may be pretty-printed across lines.
    Neither is valid as a single JSON document, and a strict json.load() on the wrong one
    is a crash that would take the report down for a formatting reason. Decode greedily
    instead: repeated raw_decode over the whole text handles a lone object, an array, an
    NDJSON stream and a concatenation of pretty-printed pages identically.
    """
    dec = json.JSONDecoder()
    out, i, n = [], 0, len(text)
    while i < n:
        while i < n and text[i].isspace():
            i += 1
        if i >= n:
            break
        val, i = dec.raw_decode(text, i)
        if isinstance(val, dict) and "jobs" in val:
            out.extend(val["jobs"])
        elif isinstance(val, list):
            out.extend(val)
        elif isinstance(val, dict):
            out.append(val)
    return out


def leg_key(name, prefix):
    """(crate, shard) identity for a matrix leg name, or None if it is not one of ours.

    shard is the "k/n" string when the leg is sub-sharded, else None — the same identity
    the artifact name carries as "-shard-k-n", which is what lets a leg be joined to its
    evidence. A leg of ours whose suffix names no crate returns (None, None): reported as
    unattributable rather than guessed at.
    """
    if not name.startswith(prefix + " (") or not name.endswith(")"):
        return None
    suffix = name[len(prefix) + 2:-1]
    crate = shard = None
    for tok in suffix.split(", "):
        tok = tok.strip()
        if _CRATE_RE.match(tok):
            crate = tok
        elif _SHARD_RE.match(tok):
            shard = tok
    return (crate, shard)


def leg_minutes(job):
    """Wall-clock of one leg, or None when the API gave no usable timestamps."""
    started, completed = job.get("started_at"), job.get("completed_at")
    if not started or not completed:
        return None
    try:
        a = datetime.fromisoformat(started.replace("Z", "+00:00"))
        b = datetime.fromisoformat(completed.replace("Z", "+00:00"))
    except ValueError:
        return None
    return max(0.0, (b - a).total_seconds() / 60.0)


def collect_legs(jobs, prefix=DEFAULT_LEG_PREFIX):
    """Every matrix leg of the mutation job in this run, with its identity + wall-clock."""
    legs = []
    for j in jobs:
        key = leg_key(j.get("name", ""), prefix)
        if key is None:
            continue
        crate, shard = key
        legs.append({
            "name": j.get("name", ""),
            "crate": crate,
            "shard": shard,
            "conclusion": j.get("conclusion"),
            "minutes": leg_minutes(j),
            "url": j.get("html_url", ""),
        })
    return legs


def collect_evidence(artifact_root, gate=None):
    """{(crate, shard): evidence} for every outcomes.json under artifact_root.

    The download lands each artifact in its own directory named
    `mutants-outcomes-nightly-<crate>[-shard-k-n]`, which is the ONLY place the shard
    identity exists (cargo-mutants does not record it in the JSON) — mutants-gate.shard_of
    reads it from the path, so the join key here and the one --seed enforces are the same.
    """
    gate = gate or _load_mutants_gate()
    rows = {}
    for dirpath, _dirs, files in os.walk(artifact_root):
        if "outcomes.json" not in files:
            continue
        path = os.path.join(dirpath, "outcomes.json")
        sid = gate.shard_of(path)
        shard = f"{sid[0]}/{sid[1]}" if sid else None
        try:
            doc = gate.load(path)
        except (ValueError, OSError) as exc:
            rows[(None, shard)] = {"state": "unreadable", "detail": str(exc),
                                   "done": 0, "planned": None}
            continue
        surviving, caught, unviable, timeout, _total = gate.summarise(doc)
        done = surviving + caught + timeout + unviable
        planned, done, why = gate.completeness(path, done)
        if why is not None:
            state, detail = "unverifiable", why
        elif done < planned:
            state, detail = "truncated", f"{done}/{planned} mutants finished"
        else:
            state, detail = "complete", f"{done}/{planned} mutants finished"
        rows[(gate.crate_of(path, doc), shard)] = {
            "state": state, "detail": detail, "done": done, "planned": planned,
            "surviving": surviving,
        }
    return rows


def _fmt_minutes(m):
    return "—" if m is None else f"{m:.0f}m"


def build_report(legs, evidence, prefix=DEFAULT_LEG_PREFIX):
    """(markdown_lines, annotations, verdict).

    Markdown and `::error::` annotations are returned SEPARATELY on purpose: the markdown
    is appended to $GITHUB_STEP_SUMMARY, while a workflow command is only honoured when it
    reaches the step's LOG. Teeing one stream into both renders the annotations as literal
    text in the summary and is the usual way this pairing gets broken.

    The verdict counts the two signals separately so neither can be read as the other;
    exit_code is 1 iff the lane did not deliver a complete measurement tonight.
    """
    by_conclusion = {}
    for leg in legs:
        by_conclusion[leg["conclusion"] or "in-progress"] = \
            by_conclusion.get(leg["conclusion"] or "in-progress", 0) + 1
    ran = [leg for leg in legs if leg["conclusion"] not in _NOT_RUN]
    not_completed = [leg for leg in ran if leg["conclusion"] != "success"]
    billed = sum(leg["minutes"] or 0.0 for leg in legs)

    # Join each leg that RAN to the artifact it should have uploaded.
    bad_evidence, no_evidence, unattributable = [], [], []
    for leg in ran:
        if leg["crate"] is None:
            unattributable.append(leg)
            continue
        ev = evidence.get((leg["crate"], leg["shard"]))
        if ev is None:
            no_evidence.append(leg)
        elif ev["state"] != "complete":
            bad_evidence.append((leg, ev))

    L = []
    L.append("## mutation ratchet nightly — leg report")
    L.append("")
    tally = " · ".join(f"{n} {c}" for c, n in sorted(by_conclusion.items()))
    L.append(f"**{len(legs)} legs — {tally or 'none found'}**")
    L.append("")
    L.append("> The workflow RUN conclusion is NOT this tally. A run's status is a "
             "roll-up over every sibling job in ci.yml, and one cancelled sibling makes "
             "the whole run report `cancelled` — which is why a night with failing "
             "mutation legs has historically been indistinguishable from a night that "
             "was killed at startup. The counts above are this lane's own state.")
    L.append("")
    L.append(f"**Billed wall-clock this run: {billed / 60:.1f} runner-hours** across "
             f"{len(legs)} legs. This is the measurement to re-tune the matrix from — "
             "every shard-count and timeout comment in ci.yml asks for exactly it.")
    L.append("")
    L.append("| leg | conclusion | wall-clock | evidence |")
    L.append("| --- | --- | --- | --- |")
    for leg in sorted(legs, key=lambda x: -(x["minutes"] or 0.0)):
        ident = leg["crate"] or "?"
        if leg["shard"]:
            ident += f" {leg['shard']}"
        ev = evidence.get((leg["crate"], leg["shard"])) if leg["crate"] else None
        if leg["conclusion"] in _NOT_RUN:
            eword = "not run"
        elif ev is None:
            eword = "**no artifact**"
        elif ev["state"] == "complete":
            eword = ev["detail"]
        else:
            eword = f"**{ev['state']}** ({ev['detail']})"
        L.append(f"| {ident} | {leg['conclusion'] or 'in-progress'} | "
                 f"{_fmt_minutes(leg['minutes'])} | {eword} |")
    L.append("")
    L.append("### what these numbers do and do not mean")
    L.append("")
    L.append("* A leg's conclusion CANNOT report a mutation finding. Both the "
             "cargo-mutants step and the job carry `continue-on-error: true` while the "
             "baseline seeds, so a ratchet regression leaves the leg green. A non-success "
             "leg means the leg did not finish — budget or infrastructure, never "
             "\"mutants survived\".")
    L.append("* The ratchet VERDICT for this run is the whole-run "
             "`scripts/mutants-gate.py --check` below, which sums a sub-sharded crate's "
             "shards into one whole-crate figure. A per-leg check cannot: a single shard "
             "is a lower bound on its crate.")
    L.append("* `truncated` evidence is the silent case: cargo-mutants writes "
             "outcomes.json incrementally, so a leg cut short still uploads a well-formed "
             "artifact. It must never seed a ceiling.")
    L.append("")
    L.append("### verdict")
    for label, items in (
        ("legs that did not complete (budget / infrastructure)", not_completed),
        ("legs that ran but uploaded no outcomes artifact", no_evidence),
        ("legs whose evidence is truncated or unverifiable", bad_evidence),
        ("legs whose name carried no crate identity", unattributable),
    ):
        L.append(f"* {label}: **{len(items)}**")
    ok = not (not_completed or no_evidence or bad_evidence or unattributable)
    L.append("")
    L.append("**lane delivered a complete measurement tonight: "
             f"{'yes' if ok else 'NO'}**")

    A = []
    if not legs:
        # Fail loud rather than green-on-empty: a report that finds no legs has either
        # lost the name join or been wired into a run that has none, and both look
        # exactly like a healthy quiet night if this returns 0.
        A.append(f"::error::mutation ratchet report found NO legs matching "
                 f"{prefix!r} — the lane did not run, or the job name drifted from "
                 f"DEFAULT_LEG_PREFIX in scripts/mutants-nightly-report.py")
        ok = False
    if not_completed:
        A.append(f"::error::mutation ratchet: {len(not_completed)} of {len(ran)} legs did "
                 f"not complete ({billed / 60:.1f} runner-hours billed). These are budget "
                 f"/ infrastructure failures, NOT mutation findings.")
    if no_evidence or bad_evidence:
        A.append(f"::error::mutation ratchet: {len(no_evidence)} leg(s) uploaded no "
                 f"outcomes and {len(bad_evidence)} uploaded incomplete outcomes — those "
                 f"crates have no seedable measurement from this run.")
    if unattributable:
        A.append(f"::warning::mutation ratchet: {len(unattributable)} leg name(s) carried "
                 f"no `sparq-*` crate token, so they could not be joined to an artifact.")

    return L, A, {
        "legs": len(legs),
        "ran": len(ran),
        "not_completed": len(not_completed),
        "no_evidence": len(no_evidence),
        "bad_evidence": len(bad_evidence),
        "unattributable": len(unattributable),
        "runner_hours": round(billed / 60, 2),
        "exit_code": 0 if ok else 1,
    }


def main():
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    ap = argparse.ArgumentParser(description="nightly mutation-lane leg report")
    ap.add_argument("--jobs", required=True,
                    help="jobs for THIS run, as `gh api .../jobs` emits them")
    ap.add_argument("--artifacts", required=True,
                    help="directory the mutants-outcomes-nightly-* artifacts were "
                         "downloaded into")
    ap.add_argument("--leg-prefix", default=DEFAULT_LEG_PREFIX,
                    help="the matrix job's `name:` in ci.yml")
    ap.add_argument("--summary", default="",
                    help="file to APPEND the markdown report to (normally "
                         "$GITHUB_STEP_SUMMARY); prints to stdout when omitted")
    a = ap.parse_args()
    with open(a.jobs) as f:
        jobs = parse_jobs(f.read())
    legs = collect_legs(jobs, a.leg_prefix)
    evidence = collect_evidence(a.artifacts) if os.path.isdir(a.artifacts) else {}
    lines, annotations, verdict = build_report(legs, evidence, a.leg_prefix)
    body = "\n".join(lines) + "\n"
    if a.summary:
        with open(a.summary, "a") as f:
            f.write(body)
    else:
        sys.stdout.write(body)
    # Annotations go to the LOG unconditionally — that is the only stream GitHub reads
    # workflow commands from, and the verdict must be visible without opening the summary.
    for line in annotations:
        print(line)
    print(f"report: {json.dumps(verdict, sort_keys=True)}")
    sys.exit(verdict["exit_code"])


def self_test():
    """Hermetic: synthetic jobs + synthetic artifacts on a temp dir. No gh, no network."""
    import tempfile

    P = DEFAULT_LEG_PREFIX

    from datetime import timedelta
    _t0 = datetime.fromisoformat("2026-09-03T00:00:00+00:00")

    def job(suffix, conclusion, mins=10):
        return {"name": f"{P} ({suffix})", "conclusion": conclusion,
                "status": "completed", "html_url": "u",
                "started_at": _t0.isoformat(),
                "completed_at": (_t0 + timedelta(minutes=mins)).isoformat()}

    # --- parse_jobs: every shape gh emits decodes to the same job list ---------------
    one_page = json.dumps({"jobs": [job("sparq-canon", "success")]})
    assert len(parse_jobs(one_page)) == 1
    assert len(parse_jobs(one_page + "\n" + one_page)) == 2, "concatenated pages"
    ndjson = "\n".join(json.dumps(job(c, "success")) for c in ("sparq-a", "sparq-b"))
    assert len(parse_jobs(ndjson)) == 2, "NDJSON stream"
    pretty = json.dumps([job("sparq-canon", "success")], indent=2)
    assert len(parse_jobs(pretty)) == 1, "pretty-printed array"

    # --- leg_key: identity join, and NOT matching the report job's own name ----------
    assert leg_key(f"{P} (sparq-canon)", P) == ("sparq-canon", None)
    assert leg_key(f"{P} (sparq-engine, 360, 9/24)", P) == ("sparq-engine", "9/24")
    assert leg_key(f"{P} (sparq-core, mmap,dict-spill, 360)", P) == ("sparq-core", None)
    assert leg_key("mutation ratchet report (cargo-mutants, advisory)", P) is None, \
        "the report job must never count itself as a leg"
    assert leg_key(f"{P} (360)", P) == (None, None), "no crate token => unattributable"
    assert leg_key("some other job (sparq-canon)", P) is None

    # --- leg_minutes ----------------------------------------------------------------
    assert leg_minutes(job("sparq-canon", "success", mins=42)) == 42.0
    assert leg_minutes({"started_at": None, "completed_at": None}) is None

    # --- evidence + verdict over real files on disk ----------------------------------
    gate = _load_mutants_gate()
    with tempfile.TemporaryDirectory() as tmp:
        def artifact(name, planned, finished):
            d = os.path.join(tmp, f"mutants-outcomes-nightly-{name}")
            os.makedirs(d, exist_ok=True)
            with open(os.path.join(d, "mutants.json"), "w") as f:
                json.dump([{"i": i} for i in range(planned)], f)
            with open(os.path.join(d, "outcomes.json"), "w") as f:
                json.dump({"outcomes": [
                    {"summary": "CaughtMutant",
                     "scenario": {"Mutant": {"package": name.split("-shard-")[0]}}}
                ] * finished}, f)

        artifact("sparq-canon", planned=5, finished=5)              # complete
        artifact("sparq-engine-shard-9-24", planned=100, finished=7)  # truncated
        ev = collect_evidence(tmp, gate)
        assert ev[("sparq-canon", None)]["state"] == "complete", ev
        assert ev[("sparq-engine", "9/24")]["state"] == "truncated", ev

        # ANTI-VACUITY CONTROL: a healthy night must be GREEN. Without this, a
        # build_report() that returned 1 unconditionally would pass every case below.
        healthy = collect_legs(parse_jobs(json.dumps({"jobs": [
            job("sparq-canon", "success")]})), P)
        _lines, _ann, v = build_report(healthy, {k: ev[k] for k in [("sparq-canon", None)]}, P)
        assert v["exit_code"] == 0, v
        assert v["not_completed"] == 0 and v["no_evidence"] == 0, v

        # The real shape of the nights this was written for: one leg times out (red, no
        # artifact), one finishes green but uploaded TRUNCATED evidence (silently
        # useless), one is clean. All three must be counted, and separately.
        legs = collect_legs(parse_jobs(json.dumps({"jobs": [
            job("sparq-canon", "success", mins=30),
            job("sparq-engine, 360, 9/24", "failure", mins=360),
            job("sparq-mpc, 360", "failure", mins=360),
        ]})), P)
        _lines, _ann, v = build_report(legs, ev, P)
        assert v["legs"] == 3 and v["ran"] == 3, v
        assert v["not_completed"] == 2, v
        assert v["no_evidence"] == 1, v          # sparq-mpc uploaded nothing
        assert v["bad_evidence"] == 1, v         # sparq-engine 9/24 truncated
        assert v["exit_code"] == 1, v
        assert v["runner_hours"] == 12.5, v      # (30 + 360 + 360) / 60

        # A truncated artifact from a GREEN leg must still red the report — this is the
        # case the leg conclusions cannot express at all.
        green_but_truncated = collect_legs(parse_jobs(json.dumps({"jobs": [
            job("sparq-engine, 360, 9/24", "success")]})), P)
        _lines, _ann, v = build_report(green_but_truncated, ev, P)
        assert v["not_completed"] == 0 and v["bad_evidence"] == 1 and v["exit_code"] == 1, v

    # --- empty leg set fails LOUD ----------------------------------------------------
    _lines, _ann, v = build_report([], {}, P)
    assert v["exit_code"] == 1, "no legs found must never report success"

    print("mutants-nightly-report self-test: OK")
    return 0


if __name__ == "__main__":
    main()
