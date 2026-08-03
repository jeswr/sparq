#!/usr/bin/env python3
"""Per-leg CI wall-time INVENTORY — constant overhead vs compile vs test-run. 🤖 SPARQ agent.

Bead sq-6vshe.7 (a), design record research/ci-structural-speedup.md §8. This is the
STEERING INSTRUMENT for the non-coverage test-execution topology program: it turns the
GitHub Actions jobs API into a per-leg table decomposed into

    wait  |  constant overhead  |  compile  |  test-run  |  other

so the remaining questions in that bead — which nextest partitions are unbalanced (b),
which legs are worth extending the build-once archive to (c), what the workspace doctest
bill actually is (d), and which tiny legs are pure per-job constant tax and should be
folded together (e) — get answered from MEASUREMENT rather than from the leg names.

NOT A GATE, and deliberately not wired into any workflow. It reports; it decides nothing
about a commit. Exit 0 = a report was produced, exit 2 = the instrument itself is broken
(no data, unusable API payload, or a step corpus it can no longer classify). There is no
exit 1: collapsing "broken instrument" into "bad finding" is exactly how a measurement
tool starts lying.

=============================================================================
WHAT THE DECOMPOSITION CAN AND CANNOT SEE — read before trusting a column
=============================================================================
The jobs API returns, per step, only a NAME and a start/end timestamp. It does not return
the command. So every classification here is a function of the step NAME, and that has
three consequences that bound what the table means:

  1. THE CONSTANT-OVERHEAD COLUMN IS THE RELIABLE ONE. Its members are mechanical and
     mostly not authored by us at all — GitHub's own implicit `Set up job` / `Post <x>` /
     `Complete job` steps, plus checkout, toolchain, cache restore/save, tool install,
     artifact up/download, disk reclaim, pinned fixture fetches. These names do not drift
     with a workflow edit, and they are precisely the per-job tax that (e) is about.

  2. COMPILE vs TEST-RUN IS STEP-GRANULAR, NOT PHASE-GRANULAR. A bare `cargo test` step
     compiles AND runs inside one step; no timestamp separates the two halves, so such a
     step is attributed whole to `test-run` and its build cost is INVISIBLE as compile.
     This is not a defect to be papered over — it is the reason ci.yml's build-once
     nextest-archive topology exists, and it is why the archive legs (`build + archive
     test binaries` = pure compile, the `test (...)` shards = pure run) are the legs whose
     split this table can be trusted on. Read a mixed leg's `test-run` as "compile+run",
     and treat a large one as a CANDIDATE for the archive split (c), not as a measurement
     of its run time.

  3. NAME-BASED CLASSIFICATION CAN GO STALE. A renamed step silently falls to
     `unclassified`, which would quietly move time out of a real column. So unclassified
     time is a VISIBLE column, never folded into `other`, and exceeding
     --max-unclassified (see DEFAULT_MAX_UNCLASSIFIED_PCT) exits 2. A drifted classifier
     fails loudly instead of producing a plausible wrong table.

     Coverage is NOT uniform across the repo, and the difference is deliberate. Every step
     name in ci.yml — the default --workflow — classifies, and a test asserts that over
     the live file so a renamed step reds in CI rather than in a report. A LARGE MINORITY
     of the step names in the OTHER workflows do not classify — no figure is quoted here
     because it moves with every workflow edit and a stale one would itself be a lie; the
     rules below cover the compile/test-run distinction this bead needs and stop short of
     guessing at deploy/attest/publish steps. So pointing this tool at another workflow
     will usually trip --max-unclassified and want an --overrides file first; that is
     working as intended, not a gap to paper over by loosening a pattern.

`wait` is `job.started_at - run.run_started_at`: queue time AND time spent blocked on an
upstream `needs:`. Those are not separable from this payload either, and for topology
work they are usually wanted together — a shard that idles because `build-archive` has
not finished yet is a topology fact, not a runner-availability fact. (Runner availability
proper is scripts/ci_execution_latency_alarm.py's job, and its measured finding is that
pickup lag is near zero, so a large `wait` here is almost always the `needs:` edge.)

=============================================================================
THE TRAP THE LEG NAMES SET — do not skip
=============================================================================
ci.yml's test matrix legs are named `heavy-diskann`, `heavy-hnsw`, `bulk 1/3`..`bulk 3/3`.
The bulk legs do NOT run "one third of the suite each": each runs
`--partition count:K/3` over the filterset `not ( $HEAVY )`, i.e. over the suite MINUS
the named heavy tests. Rebalancing (b) therefore means re-deriving the HEAVY filterset
env and the shard count TOGETHER from this table — moving a test out of HEAVY silently
adds it to all three bulk legs' input set. `--families` prints each matrix family's
spread so the imbalance is visible before anyone edits the filterset.

Usage:
  ci_leg_inventory.py --repo o/r                        # last N runs of ci.yml
  ci_leg_inventory.py --repo o/r --workflow feature-matrix.yml --event pull_request
  ci_leg_inventory.py --fixture dump.json               # hermetic; no gh, no network
  ci_leg_inventory.py --repo o/r --json > inventory.json

Stdlib + the `gh` CLI (only when --fixture is absent).
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import statistics
import subprocess
import sys

# --- Tunables. Each is a REPORTING threshold, not a gate: nothing here can fail a commit.
DEFAULT_RUNS = 20
RUNS_HARD_CAP = 100
# A classifier that no longer recognises this share of measured step time is producing a
# plausible-looking wrong table, which is worse than no table. Fail loud (exit 2).
DEFAULT_MAX_UNCLASSIFIED_PCT = 15.0
# (e) fold candidates: a leg whose wall-time is small AND mostly constant per-job tax.
# Both halves are required — a long leg with high overhead wants the overhead fixed, not
# to be merged into a neighbour.
FOLD_MAX_TOTAL_SECONDS = 180.0
FOLD_MIN_OVERHEAD_SHARE = 0.50
# (b) a matrix family whose slowest leg exceeds its fastest by this factor is unbalanced:
# the fast legs' runners are idle while the slow one is the family's wall-clock.
IMBALANCE_RATIO = 1.5


class InventoryError(RuntimeError):
    """The instrument is broken. Always exit 2 — never emit a table we cannot stand behind."""


def _require_list(value, what: str) -> list:
    """A collection the payload promised, checked BEFORE it is sliced or iterated.

    The no-exit-1 contract in the header covers malformed SHAPES as much as missing keys:
    an API or fixture that answers `{"jobs": "none"}` must exit 2 with a diagnostic, not
    raise an AttributeError one frame later and exit 1 with a traceback.
    """
    if not isinstance(value, list):
        raise InventoryError(f"{what} is not a list (got {type(value).__name__})")
    return value


def _require_object(value, what: str) -> dict:
    """One element of such a collection, checked BEFORE any `.get` on it."""
    if not isinstance(value, dict):
        raise InventoryError(f"{what} is not an object (got {type(value).__name__})")
    return value


# ---------------------------------------------------------------------------------
# Step classification
# ---------------------------------------------------------------------------------
# Buckets roll up into CLASSES. The class is what the headline table shows; the bucket is
# what --steps shows, and is what you edit when a name drifts.
CLASS_OF_BUCKET = {
    "setup": "constant",        # GitHub's implicit Set up job / Post <x> / Complete job
    "checkout": "constant",
    "toolchain": "constant",
    "cache": "constant",
    "tool-install": "constant",
    "artifact": "constant",
    "disk": "constant",
    "fixture-fetch": "constant",
    "compile": "compile",
    "test-run": "test-run",
    "script": "other",          # cheap bookkeeping: ratchets, gates, summaries, uploads of text
    "unclassified": "unclassified",
}
CLASSES = ("constant", "compile", "test-run", "other", "unclassified")

# ORDERED rules — first match wins, so a more specific pattern must precede a broader one.
# Patterns are matched case-insensitively against the step name.
_RULES: tuple[tuple[str, str], ...] = (
    # GitHub's own implicit steps. These are the floor of the per-job constant tax and
    # exist on EVERY job whether or not the workflow author wrote a step.
    (r"^set up job$", "setup"),
    (r"^complete job$", "setup"),
    (r"^post ", "setup"),
    (r"^set up (python|node|job container)", "toolchain"),
    # Explicit per-job constant tax.
    (r"checkout", "checkout"),
    (r"free (runner )?disk|reclaim disk", "disk"),
    (r"^rust \(|rustup|toolchain|llvm-tools", "toolchain"),
    (r"^cache |rust-cache|restore .*cache|save .*cache", "cache"),
    (r"^install |^ensure .*installed", "tool-install"),
    (r"^fetch ", "fixture-fetch"),
    # A pinned external corpus that arrives via actions/cache rather than a fetch is the
    # SAME tax, so it books to the same bucket — `Restore/Save the pinned OWL 2 / RIF
    # corpora` in ci.yml carries no "cache" token and would otherwise fall unclassified.
    (r"^(save|restore) the pinned ", "fixture-fetch"),
    (r"^(upload|download) ", "artifact"),
    (r"^restore .*\.profraw|^download all partition", "artifact"),
    # Compile-bearing steps that do NOT run tests. `nextest archive` is the canonical
    # build-once step: pure compile, zero test execution — the split (c) generalises.
    (r"archive test binaries|nextest archive", "compile"),
    (r"^clippy|^cargo check|^rustdoc|^rustfmt|^build |^build\+|instrumented .*objects",
     "compile"),
    (r"^check |^compile ", "compile"),
    # Test execution. NOTE (header point 2): a `cargo test` step compiles first; this
    # bucket therefore means "run, possibly including its own compile".
    (r"doctest", "test-run"),
    (r"^test |^test\(|^test$|nextest, shard|wasm tests|smoke test", "test-run"),
    (r"^run .*(suite|lane|test|oracle|smoke|mutants|\barm\b|conformance)", "test-run"),
    (r"^measure \+ enforce per-crate coverage|^run cargo-mutants", "test-run"),
    (r"conformance suites|^headless |selftest|self-test", "test-run"),
    # Test-execution-shaped names outside ci.yml (sanitizer / model-checker / fuzz /
    # browser lanes). These are expensive and MUST NOT be missing from the test-run
    # column when this tool is pointed at asan.yml / miri.yml / kani.yml / fuzz.yml / js.yml.
    (r"^asan\b|^miri\b|^kani\b|fuzz|\be2e\b|differential|unit tests|stress test|journeys",
     "test-run"),
    # Cheap bookkeeping around the real work.
    (r"ratchet|^enforce |gate$|^.*-presence gate|guard", "script"),
    (r"^summari[sz]e|^aggregate |^report |^emit |^resolve |^decide |^assert |^detect |"
     r"^file |^commit |scoreboard|shadow .*report|coverage report", "script"),
    (r"^compute |^merge |^measure .*size|^introspect |^lint|^validate |^verify ", "script"),
)
_COMPILED = tuple((re.compile(pat, re.IGNORECASE), bucket) for pat, bucket in _RULES)


def classify_step(name: str, overrides: dict[str, str] | None = None) -> str:
    """Bucket a step by NAME. Returns a key of CLASS_OF_BUCKET.

    `overrides` maps an EXACT step name to a bucket and wins over every rule, so a leg
    whose name genuinely misleads (a `Run …` step that only shells a python check, say)
    can be corrected without loosening a pattern for everyone.
    """
    text = (name or "").strip()
    if overrides:
        pinned = overrides.get(text)
        if pinned is not None:
            if pinned not in CLASS_OF_BUCKET:
                raise InventoryError(
                    f"override for {text!r} names unknown bucket {pinned!r}; "
                    f"known: {sorted(CLASS_OF_BUCKET)}"
                )
            return pinned
    if not text:
        return "unclassified"
    for pattern, bucket in _COMPILED:
        if pattern.search(text):
            return bucket
    return "unclassified"


# ---------------------------------------------------------------------------------
# Decomposition
# ---------------------------------------------------------------------------------
def _ts(value):
    if value in (None, ""):
        return None
    try:
        return dt.datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    except ValueError as exc:
        raise InventoryError(f"unparseable timestamp {value!r}: {exc}") from exc


def _span(start, end) -> float:
    """Seconds between two API timestamps. A missing endpoint (a SKIPPED step carries
    nulls) or a negative span contributes ZERO rather than raising — a skipped step is a
    real, legitimate zero, and clamping keeps one clock skew from poisoning a whole run."""
    a, b = _ts(start), _ts(end)
    if a is None or b is None:
        return 0.0
    return max(0.0, (b - a).total_seconds())


def leg_family(job_name: str) -> str:
    """The matrix FAMILY a leg belongs to: everything before the first ` (`.

    GitHub names a matrix leg `<job name> (<matrix values>)`, so this groups
    `test (load-aware shard bulk 1/3)` with its siblings while leaving a non-matrix job
    as its own single-member family.
    """
    name = (job_name or "").strip()
    head = name.split(" (", 1)[0]
    return head or name


def decompose_job(job: dict, run_started_at=None, overrides=None) -> dict:
    """One job -> its wall-time decomposition. Pure; no I/O."""
    if not isinstance(job, dict):
        raise InventoryError("job payload is not an object")
    name = job.get("name")
    if not name:
        raise InventoryError("job payload carries no name")
    total = _span(job.get("started_at"), job.get("completed_at"))
    # `wait` is queue + blocked-on-`needs:`; see the header. Measured from the RUN's start
    # so it is comparable across legs of the same run; falls back to the job's own
    # created_at (pure queue) when the run timestamp is unavailable.
    wait_from = run_started_at or job.get("created_at")
    wait = _span(wait_from, job.get("started_at"))

    buckets: dict[str, float] = {}
    steps: list[dict] = []
    for step in job.get("steps") or []:
        if not isinstance(step, dict):
            raise InventoryError(f"{name}: a step entry is not an object")
        seconds = _span(step.get("started_at"), step.get("completed_at"))
        bucket = classify_step(step.get("name", ""), overrides)
        buckets[bucket] = buckets.get(bucket, 0.0) + seconds
        steps.append({"name": step.get("name", ""), "seconds": seconds, "bucket": bucket})

    classes = dict.fromkeys(CLASSES, 0.0)
    for bucket, seconds in buckets.items():
        classes[CLASS_OF_BUCKET[bucket]] += seconds
    accounted = sum(classes.values())
    return {
        "leg": name,
        "family": leg_family(name),
        "conclusion": job.get("conclusion"),
        "wait_s": wait,
        "total_s": total,
        "classes": classes,
        "buckets": buckets,
        # Steps do not tile the job: runner start-up before step 1 and teardown after the
        # last one land nowhere. Surfaced rather than hidden, so the columns can be
        # checked against the total instead of being taken on faith.
        "ungrouped_s": max(0.0, total - accounted),
        "steps": steps,
    }


def _median(values: list[float]) -> float:
    return statistics.median(values) if values else 0.0


def aggregate_legs(decomposed: list[dict]) -> list[dict]:
    """Median each column PER LEG across runs.

    Median, not mean: a single runner hiccup or a cold cache should not move a leg's
    steering number, and the distribution across runs of one leg is small-N and skewed.
    """
    by_leg: dict[str, list[dict]] = {}
    for entry in decomposed:
        by_leg.setdefault(entry["leg"], []).append(entry)
    rows = []
    for leg, entries in sorted(by_leg.items()):
        row = {
            "leg": leg,
            "family": entries[0]["family"],
            "n": len(entries),
            "wait_s": _median([e["wait_s"] for e in entries]),
            "total_s": _median([e["total_s"] for e in entries]),
            "ungrouped_s": _median([e["ungrouped_s"] for e in entries]),
        }
        for cls in CLASSES:
            row[cls] = _median([e["classes"][cls] for e in entries])
        measured = row["total_s"] or 1.0
        row["overhead_share"] = row["constant"] / measured
        rows.append(row)
    rows.sort(key=lambda r: r["total_s"], reverse=True)
    return rows


def unclassified_share(decomposed: list[dict]) -> float:
    """Unclassified seconds as a percentage of all step time. The instrument's own
    health check — see header point 3."""
    unknown = sum(e["classes"]["unclassified"] for e in decomposed)
    known = sum(sum(e["classes"].values()) for e in decomposed)
    if known <= 0:
        return 0.0
    return 100.0 * unknown / known


# ---------------------------------------------------------------------------------
# Findings — the actionable half, one function per bead sub-part
# ---------------------------------------------------------------------------------
def fold_candidates(rows: list[dict],
                    max_total: float = FOLD_MAX_TOTAL_SECONDS,
                    min_share: float = FOLD_MIN_OVERHEAD_SHARE) -> list[dict]:
    """(e) Legs that are mostly per-job constant tax and short enough that folding them
    into a neighbour would pay. BOTH conditions are required: a long leg with a high
    overhead share needs its overhead fixed (cache, toolchain), not a merge."""
    out = []
    for row in rows:
        if row["total_s"] <= 0:
            continue
        if row["total_s"] <= max_total and row["overhead_share"] >= min_share:
            out.append({
                "leg": row["leg"],
                "total_s": row["total_s"],
                "constant_s": row["constant"],
                "overhead_share": row["overhead_share"],
            })
    return sorted(out, key=lambda r: r["constant_s"], reverse=True)


def family_imbalance(rows: list[dict], ratio: float = IMBALANCE_RATIO) -> list[dict]:
    """(b) Matrix families whose legs do not finish together. `spread` is
    slowest/fastest; anything above `ratio` means the fast legs' runners idle while the
    family's wall-clock is set by the slow one — the partition wants rebalancing."""
    by_family: dict[str, list[dict]] = {}
    for row in rows:
        by_family.setdefault(row["family"], []).append(row)
    out = []
    for family, legs in sorted(by_family.items()):
        # ONE guard, not two: a single-member family has no spread to report, and a leg
        # with no measured duration cannot be one end of a ratio. Folding both into this
        # single condition keeps it individually killable by a mutation — two redundant
        # early-outs would each be masked by the other.
        totals = [leg["total_s"] for leg in legs if leg["total_s"] > 0]
        if len(totals) < 2:
            continue
        slowest, fastest = max(totals), min(totals)
        spread = slowest / fastest
        if spread < ratio:
            continue
        out.append({
            "family": family,
            "legs": len(legs),
            "slowest_s": slowest,
            "fastest_s": fastest,
            "spread": spread,
            "slowest_leg": max(legs, key=lambda leg: leg["total_s"])["leg"],
            "idle_s": sum(slowest - t for t in totals),
        })
    return sorted(out, key=lambda r: r["idle_s"], reverse=True)


DOCTEST_RE = re.compile(r"doctest", re.IGNORECASE)


def doctest_bill(decomposed: list[dict]) -> dict:
    """(d) What the workspace doctest lane actually costs.

    Every doctest compiles and links as its OWN crate, so this is the one test population
    whose cost is dominated by linking rather than running — and nextest neither archives
    nor runs doctests, which is why ci.yml executes them in a single `cargo test --doc`
    step off the already-warm archive build. That step is what this measures; any
    consolidation (`no_run`, merging examples) must keep a `cargo test --doc` lane alive
    or the doc examples stop being compiled at all.
    """
    per_leg: dict[str, list[float]] = {}
    for entry in decomposed:
        seconds = sum(s["seconds"] for s in entry["steps"] if DOCTEST_RE.search(s["name"]))
        if seconds > 0:
            per_leg.setdefault(entry["leg"], []).append(seconds)
    legs = [{"leg": leg, "seconds": _median(vals), "n": len(vals)}
            for leg, vals in sorted(per_leg.items())]
    return {
        "legs": sorted(legs, key=lambda leg: leg["seconds"], reverse=True),
        "total_s": sum(leg["seconds"] for leg in legs),
        "found": bool(legs),
    }


def slowest_steps(decomposed: list[dict], limit: int = 15) -> list[dict]:
    """The individual steps that dominate, medianed across runs. Feeds (c): a long
    compile step on more than one leg is a build-once-archive candidate."""
    per_step: dict[tuple[str, str], list[float]] = {}
    buckets: dict[tuple[str, str], str] = {}
    for entry in decomposed:
        for step in entry["steps"]:
            key = (entry["leg"], step["name"])
            per_step.setdefault(key, []).append(step["seconds"])
            buckets[key] = step["bucket"]
    rows = [{"leg": leg, "step": step, "bucket": buckets[(leg, step)],
             "seconds": _median(vals), "n": len(vals)}
            for (leg, step), vals in per_step.items()]
    rows.sort(key=lambda r: r["seconds"], reverse=True)
    return rows[:limit]


# ---------------------------------------------------------------------------------
# gh I/O
# ---------------------------------------------------------------------------------
def _gh(args: list[str]):
    try:
        out = subprocess.run(["gh", *args], check=True, capture_output=True,
                             text=True, timeout=180)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise InventoryError(f"gh {' '.join(args[:2])} failed: {detail}") from exc
    try:
        return json.loads(out.stdout or "null")
    except json.JSONDecodeError as exc:
        raise InventoryError(f"gh {' '.join(args[:2])}: unparseable JSON: {exc}") from exc


def fetch_runs(repo: str, workflow: str, event: str | None, branch: str | None,
               limit: int) -> list[dict]:
    query = f"status=completed&per_page={min(limit, RUNS_HARD_CAP)}"
    if event:
        query += f"&event={event}"
    if branch:
        query += f"&branch={branch}"
    payload = _gh(["api", f"repos/{repo}/actions/workflows/{workflow}/runs?{query}"])
    if not isinstance(payload, dict) or payload.get("workflow_runs") is None:
        raise InventoryError(f"{workflow}: run listing carries no workflow_runs")
    return _require_list(payload["workflow_runs"], f"{workflow}: workflow_runs")[:limit]


JOBS_PAGE_CAP = 20


def fetch_jobs(repo: str, run_id) -> list[dict]:
    """Every job of one run, paged EXPLICITLY.

    Not `gh api --paginate`: on an OBJECT-valued endpoint (this one returns
    `{total_count, jobs}`) `--paginate` concatenates one JSON object per page into a
    stream that `json.loads` cannot parse — so a run wide enough to need a second page
    would fail, and only a run wide enough to need one. That is precisely the population
    this inventory is for. Terminates on a SHORT page rather than on `total_count`,
    mirroring _paged_runs.
    """
    jobs: list[dict] = []
    page = 1
    while page <= JOBS_PAGE_CAP:
        payload = _gh(["api",
                       f"repos/{repo}/actions/runs/{run_id}/jobs?per_page=100&page={page}"])
        if not isinstance(payload, dict) or payload.get("jobs") is None:
            raise InventoryError(f"run {run_id}: jobs response carries no jobs")
        batch = _require_list(payload["jobs"], f"run {run_id}: jobs")
        jobs.extend(batch)
        if len(batch) < 100:
            return jobs
        page += 1
    raise InventoryError(f"run {run_id}: job listing exceeded {JOBS_PAGE_CAP} pages")


def decompose_run(jobs, run_started_at, overrides) -> list[dict]:
    """The one place a run's jobs become inventory rows — shared by the gh path and the
    fixture path so the exclusion rule below has a single, testable home.

    Only COMPLETED work has a wall-time. A cancelled, skipped or still-running leg has no
    duration to contribute and must never be medianed in as a fast one: that would
    understate exactly the legs this table exists to steer.
    """
    rows = []
    for index, job in enumerate(_require_list(jobs if jobs is not None else [], "`jobs`")):
        _require_object(job, f"job #{index}")
        if job.get("conclusion") in (None, "cancelled", "skipped"):
            continue
        rows.append(decompose_job(job, run_started_at, overrides))
    return rows


def collect(repo: str, workflow: str, event: str | None, branch: str | None,
            limit: int, overrides) -> list[dict]:
    decomposed = []
    for index, run in enumerate(fetch_runs(repo, workflow, event, branch, limit)):
        _require_object(run, f"{workflow}: run listing entry #{index}")
        run_id = run.get("id")
        if run_id is None:
            raise InventoryError(f"{workflow}: run listing entry #{index} carries no id")
        started = run.get("run_started_at") or run.get("created_at")
        decomposed.extend(decompose_run(fetch_jobs(repo, run_id), started, overrides))
    return decomposed


def collect_fixture(payload: dict, overrides) -> list[dict]:
    """Hermetic input: {"runs": [{"run_started_at": ..., "jobs": [<job>, ...]}, ...]}.

    Also accepts a bare {"jobs": [...]} for a single saved
    `gh api repos/o/r/actions/runs/<id>/jobs` dump.
    """
    if not isinstance(payload, dict):
        raise InventoryError("fixture is not an object")
    runs = payload.get("runs")
    if runs is None:
        if payload.get("jobs") is None:
            raise InventoryError("fixture carries neither `runs` nor `jobs`")
        runs = [payload]
    decomposed = []
    for index, run in enumerate(_require_list(runs, "fixture `runs`")):
        _require_object(run, f"fixture run #{index}")
        started = run.get("run_started_at") or run.get("created_at")
        decomposed.extend(decompose_run(run.get("jobs"), started, overrides))
    return decomposed


# ---------------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------------
def _fmt(seconds: float) -> str:
    return f"{seconds:.0f}s" if seconds < 600 else f"{seconds / 60:.1f}m"


def render_markdown(rows: list[dict], decomposed: list[dict], *, source: str,
                    show_steps: bool, show_families: bool) -> str:
    lines = [
        f"# CI per-leg wall-time inventory — {source}",
        "",
        f"{len(rows)} legs over {len(decomposed)} completed job observations; every cell "
        "is the MEDIAN across runs.",
        "",
        "`wait` = queue + time blocked on an upstream `needs:`. `constant` = per-job tax "
        "(set-up/checkout/toolchain/cache/tool-install/artifact/disk/fixture-fetch). "
        "`test-run` on a leg that has no separate compile step means compile+run "
        "together — see the module header before reading that column as run time.",
        "",
        "| leg | n | total | wait | constant | compile | test-run | other | unclassified | overhead |",
        "|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|",
    ]
    for row in rows:
        lines.append(
            f"| `{row['leg']}` | {row['n']} | {_fmt(row['total_s'])} | {_fmt(row['wait_s'])} "
            f"| {_fmt(row['constant'])} | {_fmt(row['compile'])} | {_fmt(row['test-run'])} "
            f"| {_fmt(row['other'])} | {_fmt(row['unclassified'])} "
            f"| {100 * row['overhead_share']:.0f}% |"
        )

    bill = doctest_bill(decomposed)
    lines += ["", "## Doctest bill (bead sub-part d)", ""]
    if bill["found"]:
        lines.append("Each doctest compiles and links as its own crate, and nextest does "
                     "not run doctests at all — any consolidation must keep a "
                     "`cargo test --doc` lane alive.")
        lines.append("")
        for leg in bill["legs"]:
            lines.append(f"- `{leg['leg']}`: {_fmt(leg['seconds'])}")
        lines.append(f"- **total: {_fmt(bill['total_s'])}**")
    else:
        lines.append("No step matching /doctest/ was observed in this sample — either the "
                     "sampled workflow runs none, or the doctest step was renamed.")

    folds = fold_candidates(rows)
    lines += ["", "## Fold candidates — overhead-dominated short legs (sub-part e)", ""]
    if folds:
        lines.append(f"Legs under {_fmt(FOLD_MAX_TOTAL_SECONDS)} whose constant tax is "
                     f"at least {100 * FOLD_MIN_OVERHEAD_SHARE:.0f}% of their wall-time. "
                     "Merging N of these into one job pays that tax once instead of N times.")
        lines.append("")
        for cand in folds:
            lines.append(f"- `{cand['leg']}`: {_fmt(cand['total_s'])} total, "
                         f"{_fmt(cand['constant_s'])} constant "
                         f"({100 * cand['overhead_share']:.0f}%)")
    else:
        lines.append("None — no sampled leg is both short and overhead-dominated.")

    if show_families:
        gaps = family_imbalance(rows)
        lines += ["", "## Matrix imbalance — legs that do not finish together (sub-part b)", ""]
        if gaps:
            lines.append("`idle` is runner-time spent waiting for the family's slowest leg. "
                         "For ci.yml's `test` family remember the names lie: a `bulk K/3` leg "
                         "runs `--partition count:K/3` over `not ( $HEAVY )`, so moving a test "
                         "out of HEAVY adds it to every bulk leg's input set.")
            lines.append("")
            for gap in gaps:
                lines.append(
                    f"- `{gap['family']}` ({gap['legs']} legs): "
                    f"{_fmt(gap['fastest_s'])} .. {_fmt(gap['slowest_s'])} "
                    f"({gap['spread']:.1f}x, slowest `{gap['slowest_leg']}`), "
                    f"idle {_fmt(gap['idle_s'])}")
        else:
            lines.append(f"No family exceeds a {IMBALANCE_RATIO:.1f}x spread.")

    if show_steps:
        lines += ["", "## Slowest individual steps (feeds the archive-extension question, c)", ""]
        for step in slowest_steps(decomposed):
            lines.append(f"- {_fmt(step['seconds'])} `{step['step']}` "
                         f"[{step['bucket']}] on `{step['leg']}`")

    unknown = unclassified_share(decomposed)
    lines += ["", f"_Unclassified step time: {unknown:.1f}% of measured step wall-time._", ""]
    return "\n".join(lines)


# ---------------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------------
def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Per-leg CI wall-time inventory (bead sq-6vshe.7). Reports; never gates.")
    parser.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY"),
                        help="owner/repo (defaults to $GITHUB_REPOSITORY)")
    parser.add_argument("--workflow", default="ci.yml", help="workflow file name")
    parser.add_argument("--event", default=None, help="filter to one trigger event")
    parser.add_argument("--branch", default=None, help="filter to one branch")
    parser.add_argument("--runs", type=int, default=DEFAULT_RUNS,
                        help=f"how many completed runs to sample (cap {RUNS_HARD_CAP})")
    parser.add_argument("--fixture", default=None,
                        help="read a saved jobs payload instead of calling gh (hermetic)")
    parser.add_argument("--overrides", default=None,
                        help="JSON file mapping an exact step name to a bucket")
    parser.add_argument("--max-unclassified", type=float,
                        default=DEFAULT_MAX_UNCLASSIFIED_PCT,
                        help="exit 2 if this share of step time cannot be classified")
    parser.add_argument("--json", action="store_true", help="emit the inventory as JSON")
    parser.add_argument("--steps", action="store_true", help="include the slowest-steps section")
    parser.add_argument("--families", action="store_true",
                        help="include the matrix-imbalance section")
    return parser


def run(args: argparse.Namespace) -> int:
    overrides = None
    if args.overrides:
        with open(args.overrides, encoding="utf-8") as handle:
            overrides = json.load(handle)
        if not isinstance(overrides, dict):
            raise InventoryError("--overrides must be a JSON object of name -> bucket")

    if args.fixture:
        with open(args.fixture, encoding="utf-8") as handle:
            decomposed = collect_fixture(json.load(handle), overrides)
        source = f"fixture {args.fixture}"
    else:
        if not args.repo:
            raise InventoryError("--repo is required (or set GITHUB_REPOSITORY)")
        if args.runs < 1:
            raise InventoryError("--runs must be at least 1")
        decomposed = collect(args.repo, args.workflow, args.event, args.branch,
                             min(args.runs, RUNS_HARD_CAP), overrides)
        source = f"{args.repo} {args.workflow}"

    # An empty population is the instrument failing, not a repo with no CI cost. Reporting
    # a clean empty table here is the exact failure mode ci_execution_latency_alarm.py's
    # empty-scan-set guard exists to prevent.
    if not decomposed:
        raise InventoryError(
            f"no completed jobs sampled for {source} — check --workflow/--event/--branch")

    unknown = unclassified_share(decomposed)
    if unknown > args.max_unclassified:
        worst = sorted(
            {s["name"] for e in decomposed for s in e["steps"] if s["bucket"] == "unclassified"}
        )[:10]
        raise InventoryError(
            f"{unknown:.1f}% of step wall-time is unclassified (limit "
            f"{args.max_unclassified:.1f}%) — the name-based classifier has drifted and "
            f"would mis-attribute the table. Add a rule or pass --overrides. "
            f"Unrecognised: {worst}")

    rows = aggregate_legs(decomposed)
    if args.json:
        print(json.dumps({
            "source": source,
            "observations": len(decomposed),
            "unclassified_pct": unknown,
            "legs": rows,
            "doctests": doctest_bill(decomposed),
            "fold_candidates": fold_candidates(rows),
            "family_imbalance": family_imbalance(rows),
            "slowest_steps": slowest_steps(decomposed),
        }, indent=2, sort_keys=True))
    else:
        print(render_markdown(rows, decomposed, source=source,
                              show_steps=args.steps, show_families=args.families))
    return 0


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return run(args)
    except InventoryError as exc:
        print(f"ci-leg-inventory: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
