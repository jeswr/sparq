#!/usr/bin/env python3
# [FABLE-5] CI-economy grouping (maintainer directive 2026-07-18): run ONE bin-packed
# GROUP of opt-in feature-matrix legs inside a single runner job, and emit the
# GATE-CRITICAL per-leg check-run for each leg — across a TRUSTED BOUNDARY.
#
# WHY: feature-matrix.yml used to spawn one runner per leg (159 legs = 159 runners,
# each paying checkout + toolchain + cache-restore + dependency compile). The legs are
# now bin-packed by scripts/assemble-feature-matrix.py --grouped; this script is the
# group job's engine. Consecutive legs share the job's warm target dir, so same-crate
# legs recompile only the crate itself, never the dependency stack.
#
# TRUSTED-BOUNDARY SPLIT (PR #3511 review — do NOT recombine the two modes into one
# privileged job). The group job EXECUTES PR-controlled code: cargo build scripts,
# proc macros, tests and clippy lints all run arbitrary code from the PR head (and
# from the entire third-party dependency graph). A `checks: write` token in that
# job's environment lets malicious build-time code forge arbitrary check runs —
# including duplicate gate-critical sibling names, or an attempt at the sole
# required `ci-summary / gate` context. Therefore:
#   * EXECUTION MODE (default; runs in the UNPRIVILEGED group job — contents: read,
#     persist-credentials: false, NO GH_TOKEN): runs the legs and writes the per-leg
#     outcomes to a results JSON file (FMG_RESULTS), which the workflow uploads as
#     an artifact. This mode performs NO GitHub API call and needs NO token.
#   * REPORT MODE (--report; runs in the separate `report-legs` job, which executes
#     NO PR-controlled build code and holds `checks: write`): downloads the group
#     artifacts, validates them HOSTILE-INPUT-STYLE (the artifact crossed the trust
#     boundary — every field is attacker-controlled by construction), and posts the
#     per-leg check-runs. Leg names must resolve against the assembled leg set
#     (FMG_VALID_LEGS_FILE, produced in the reporter job by running the committed
#     assembler) — never PR-supplied free text — and the emitted check-run name is
#     built HERE as `opt-in <name>`, so no artifact can ever name a non-`opt-in`
#     context. crate/features in the check-run summary are taken from the assembled
#     set, not the artifact. Any schema violation FAILS the reporter job (which is
#     itself a required, non-advisory check).
#
# NAME CONTRACT (do NOT relax — the same contract the assembler's header pins): the
# check-run NAME for every leg is `opt-in <name>`, BYTE-IDENTICAL to the name the
# per-leg matrix job used to emit (`name: opt-in ${{ matrix.name }}`). The single
# required `ci-summary / gate` aggregator discovers check-runs on the head commit BY
# NAME (scripts/ci_summary_gate.py polls repos/<repo>/commits/<sha>/check-runs), and
# the assemble self-test (scripts/tests/test_feature_matrix_assemble.py) asserts the
# emitted name set against the golden snapshot. Report mode creates each leg's
# check-run via the Checks API — ALWAYS `status: completed` (a terminal check-run can
# never hold the polling gate pending; sq-ipkku semantics make late terminal arrivals
# safe), with conclusion success/failure per leg.
#
# FAILURE SEMANTICS: a failed leg does NOT stop the group (the old matrix ran
# fail-fast: false — every leg reports, so one broken feature shows the full
# picture). At the end the group job FAILS LOUDLY, naming exactly which legs failed
# (::error + job summary), and each failed leg also carries its own red `opt-in
# <name>` check-run (posted by the reporter) for precise attribution.
#
# REPORTER FAILURE SEMANTICS — FAIL CLOSED (PR #3511 review finding 3): a Checks API
# POST failure is only tolerable when it is the EXPECTED fork-PR degradation (a
# read-only GITHUB_TOKEN cannot create check-runs; GitHub answers "Resource not
# accessible by integration"). That case degrades to a ::warning and the group jobs'
# own conclusions remain the (coarser) gating signal — requiredness flows through
# `ci-summary / gate`, never through any individual leg name (sq-fmx4u.4). EVERY
# OTHER failure (auth regression, outage, rate limit, malformed request) gets a
# bounded retry and then FAILS the reporter job: the per-leg check contract this PR
# claims to preserve must never silently evaporate into warnings.
#
# Env (EXECUTION mode):
#   GROUP_LEGS   (required) JSON array of {name, crate, features, test}
#   GROUP_NAME   (required) the group's display id (diagnostics + results file)
#   FMG_RESULTS  (required) path to write the per-leg results JSON for the reporter
#   GITHUB_STEP_SUMMARY  (optional) job-summary sink
#   FMG_CARGO    (tests only) stub binary for cargo
#
# Env (REPORT mode, --report):
#   FMG_RESULTS_DIR      (required) dir scanned recursively for the group *.json files
#   FMG_VALID_LEGS_FILE  (required) assembler default-output JSON ({"include": [...]})
#   REPO / HEAD_SHA      (required) owner/repo + commit for the Checks API POSTs
#   DETAILS_URL          (optional) workflow-run URL embedded in each check-run
#   GROUP_JOBS_RESULT    (optional) needs.opt-in-features.result — 'success' with zero
#                        result files is an inconsistency and fails the reporter
#   FMG_GH               (tests only) stub binary for gh
#   FMG_RETRY_SLEEP      (tests only) seconds between unexpected-failure retries

import glob
import json
import os
import re
import subprocess
import sys
import time

CARGO = os.environ.get("FMG_CARGO", "cargo")
GH = os.environ.get("FMG_GH", "gh")
BUILD_ATTEMPTS = 3  # same bounded transient-registry retry the per-leg matrix used

RESULTS_SCHEMA = "feature-matrix-group-results/v1"
VALID_STEPS = ("build", "test", "clippy")
# Conservative shape for the (hostile) artifact-supplied group display id — it is
# embedded in check-run summary text. The assembler emits `gNN <crate>`.
GROUP_NAME_RE = re.compile(r"^[A-Za-z0-9 ._()-]{1,64}$")
POST_ATTEMPTS = 3
# The ONE expected, tolerable Checks API failure: a fork PR's read-only
# GITHUB_TOKEN (GitHub's fixed wording for an integration-token permission miss).
FORK_DENIAL_MARKER = "Resource not accessible by integration"


def run(cmd):
    print(f"+ {' '.join(cmd)}", flush=True)
    return subprocess.run(cmd).returncode


def build_with_retry(crate, features):
    # [OPUS-4.8] sq-hhxc heritage: bounded retry absorbs transient crates.io
    # hiccups during any residual resolution the build performs.
    for attempt in range(1, BUILD_ATTEMPTS + 1):
        rc = run([CARGO, "build", "-p", crate, "--features", features])
        if rc == 0:
            return 0
        if attempt < BUILD_ATTEMPTS:
            print(
                f"cargo build attempt {attempt} failed; retrying...",
                file=sys.stderr,
                flush=True,
            )
    return rc


def run_leg(leg):
    """Build -> (test) -> clippy, exactly the per-leg matrix step set.
    Returns None on success, else the name of the failing step."""
    crate, features = leg["crate"], leg["features"]
    if build_with_retry(crate, features) != 0:
        return "build"
    if leg["test"]:
        if run([CARGO, "test", "-p", crate, "--features", features]) != 0:
            return "test"
    if (
        run(
            [
                CARGO,
                "clippy",
                "-p",
                crate,
                "--features",
                features,
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]
        )
        != 0
    ):
        return "clippy"
    return None


def write_results(path, group, results):
    """Persist the per-leg outcomes for the trusted reporter job. Rewritten after
    every leg so an infra death mid-group still ships the completed legs."""
    payload = {
        "schema": RESULTS_SCHEMA,
        "group": group,
        "results": [
            {"name": leg["name"], "failed_step": failed_step}
            for leg, failed_step in results
        ],
    }
    tmp = f"{path}.tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, ensure_ascii=False)
    os.replace(tmp, path)


def main():
    try:
        legs = json.loads(os.environ["GROUP_LEGS"])
    except (KeyError, json.JSONDecodeError) as exc:
        print(f"::error::GROUP_LEGS missing/malformed: {exc}", file=sys.stderr)
        return 2
    if not isinstance(legs, list) or not legs:
        print("::error::GROUP_LEGS must be a non-empty JSON array", file=sys.stderr)
        return 2
    results_path = os.environ.get("FMG_RESULTS", "")
    if not results_path:
        # The reporter job's per-leg check-runs depend on this file — a group job
        # running without it would silently drop every leg name from the gate's
        # discovery set, so refuse to run at all.
        print("::error::FMG_RESULTS unset — nowhere to write the per-leg results the reporter job posts from", file=sys.stderr)
        return 2
    os.makedirs(os.path.dirname(results_path) or ".", exist_ok=True)
    group = os.environ.get("GROUP_NAME", "?")
    print(f"feature-matrix group {group}: {len(legs)} leg(s)", flush=True)
    results = []
    for leg in legs:
        print(f"::group::opt-in {leg['name']}", flush=True)
        failed_step = run_leg(leg)
        print("::endgroup::", flush=True)
        results.append((leg, failed_step))
        write_results(results_path, group, results)
        if failed_step is not None:
            # Keep going: every leg reports (the old matrix was fail-fast: false).
            print(
                f"::error::opt-in {leg['name']}: FAILED at {failed_step} "
                "(continuing with the group's remaining legs)",
                flush=True,
            )
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY", "")
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as fh:
            fh.write(f"## feature-matrix group `{group}`\n\n")
            fh.write("| leg | result |\n|---|---|\n")
            for leg, failed_step in results:
                verdict = "✅ pass" if failed_step is None else f"❌ **{failed_step}**"
                fh.write(f"| `opt-in {leg['name']}` | {verdict} |\n")
    failed = [leg["name"] for leg, failed_step in results if failed_step is not None]
    if failed:
        print(
            f"::error title=feature-matrix group {group} FAILED::"
            f"{len(failed)} of {len(results)} leg(s) failed: "
            + "; ".join(f"opt-in {n}" for n in failed),
            flush=True,
        )
        return 1
    print(f"feature-matrix group {group}: all {len(results)} leg(s) green", flush=True)
    return 0


# --------------------------------------------------------------------------------
# REPORT MODE — runs in the trusted reporter job (checks: write, NO PR build code).
# --------------------------------------------------------------------------------


def _report_error(msg):
    print(f"::error::{msg}", file=sys.stderr, flush=True)


def load_valid_legs(path):
    """The assembled leg set (committed fragments via the committed assembler) —
    the ONLY names/metadata the reporter will ever put in a check-run. Returns
    {leg_name: leg_dict} or None on any malformation (reporter-side config bug)."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            obj = json.load(fh)
    except (OSError, json.JSONDecodeError) as exc:
        _report_error(f"cannot read valid-legs file {path!r}: {exc}")
        return None
    include = obj.get("include") if isinstance(obj, dict) else None
    if not isinstance(include, list) or not include:
        _report_error(f"valid-legs file {path!r} is not an assembler include-object")
        return None
    valid = {}
    for leg in include:
        if (
            not isinstance(leg, dict)
            or not isinstance(leg.get("name"), str)
            or not isinstance(leg.get("crate"), str)
            or not isinstance(leg.get("features"), str)
        ):
            _report_error(f"valid-legs file {path!r} carries a malformed leg: {leg!r}")
            return None
        valid[leg["name"]] = leg
    return valid


def validate_results_file(path, valid_legs, seen_names):
    """HOSTILE-INPUT validation of one group-results artifact file. The file was
    produced inside a job that ran arbitrary PR-controlled build code, so every
    byte is attacker-controlled: only a name that resolves in the assembled leg
    set, a failed_step from the fixed enum, and a conservatively-shaped group id
    are accepted. Returns [(leg_name, group, failed_step)] or None on ANY
    violation (the reporter then fails closed)."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            obj = json.load(fh)
    except (OSError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        _report_error(f"results file {path!r}: unreadable/not JSON ({exc})")
        return None
    if not isinstance(obj, dict) or set(obj.keys()) != {"schema", "group", "results"}:
        _report_error(f"results file {path!r}: top level must be exactly {{schema, group, results}}")
        return None
    if obj["schema"] != RESULTS_SCHEMA:
        _report_error(f"results file {path!r}: unknown schema {obj['schema']!r} (want {RESULTS_SCHEMA!r})")
        return None
    group = obj["group"]
    if not isinstance(group, str) or not GROUP_NAME_RE.match(group):
        _report_error(f"results file {path!r}: group id fails the conservative shape check")
        return None
    if not isinstance(obj["results"], list) or not obj["results"]:
        _report_error(f"results file {path!r}: results must be a non-empty list")
        return None
    out = []
    for entry in obj["results"]:
        if not isinstance(entry, dict) or set(entry.keys()) != {"name", "failed_step"}:
            _report_error(f"results file {path!r}: entry must be exactly {{name, failed_step}}: {entry!r}")
            return None
        name, failed_step = entry["name"], entry["failed_step"]
        if not isinstance(name, str) or name not in valid_legs:
            # PR-supplied free text stops HERE: an unassembled leg name is never
            # allowed to become a check-run.
            _report_error(f"results file {path!r}: leg name {name!r} is not in the assembled leg set")
            return None
        if failed_step is not None and failed_step not in VALID_STEPS:
            _report_error(f"results file {path!r}: failed_step {failed_step!r} not in {VALID_STEPS}")
            return None
        if name in seen_names:
            # Duplicate sibling check-runs are exactly the latest-run-confusion
            # attack the trusted split exists to prevent — refuse them from the
            # honest side too.
            _report_error(f"leg {name!r} appears in multiple results files ({seen_names[name]} and {path!r})")
            return None
        seen_names[name] = path
        out.append((name, group, failed_step))
    return out


def post_check_run(repo, head_sha, name, group, leg, failed_step, details_url):
    """POST one terminal `opt-in <name>` check-run. Returns 'ok', 'fork-denied'
    (the expected read-only-token degradation) or 'failed' (unexpected — the
    caller fails the reporter job: fail CLOSED)."""
    check_name = f"opt-in {name}"
    if failed_step is None:
        conclusion, title = "success", "leg passed"
        summary = (
            f"build/test/clippy green for `-p {leg['crate']} --features "
            f"{leg['features']}` (grouped leg — ran inside feature-matrix group "
            f"`{group}`; posted by the trusted reporter job)."
        )
    else:
        conclusion, title = "failure", f"leg FAILED at {failed_step}"
        summary = (
            f"`cargo {failed_step}` failed for `-p {leg['crate']} --features "
            f"{leg['features']}`. See the feature-matrix group job `{group}` log "
            f"for the full output."
        )
    payload = {
        "name": check_name,
        "head_sha": head_sha,
        "status": "completed",
        "conclusion": conclusion,
        "output": {"title": title, "summary": summary},
    }
    if details_url:
        payload["details_url"] = details_url
    retry_sleep = float(os.environ.get("FMG_RETRY_SLEEP", "5"))
    err = ""
    for attempt in range(1, POST_ATTEMPTS + 1):
        proc = subprocess.run(
            [GH, "api", f"repos/{repo}/check-runs", "--method", "POST", "--input", "-"],
            input=json.dumps(payload).encode("utf-8"),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        if proc.returncode == 0:
            return "ok"
        err = proc.stderr.decode("utf-8", "replace").strip()
        if FORK_DENIAL_MARKER in err:
            # Deterministic, EXPECTED: a fork PR's read-only token. The group
            # job's own conclusion remains the (coarser) gating signal.
            print(
                f"::warning::could not create check-run {check_name!r}: read-only "
                "token (fork PR) — the group job's own conclusion remains the "
                "gating signal",
                flush=True,
            )
            return "fork-denied"
        if attempt < POST_ATTEMPTS:
            print(
                f"check-run POST for {check_name!r} failed (attempt {attempt}/"
                f"{POST_ATTEMPTS}): {err.splitlines()[-1] if err else 'unknown error'}; retrying...",
                file=sys.stderr,
                flush=True,
            )
            time.sleep(retry_sleep)
    _report_error(
        f"check-run POST for {check_name!r} failed after {POST_ATTEMPTS} attempts "
        f"with an UNEXPECTED error ({err.splitlines()[-1] if err else 'unknown error'}). "
        "This is not the fork-PR read-only degradation — failing CLOSED so the "
        "preserved per-leg check contract never silently evaporates."
    )
    return "failed"


def report_main():
    results_dir = os.environ.get("FMG_RESULTS_DIR", "")
    valid_legs_file = os.environ.get("FMG_VALID_LEGS_FILE", "")
    repo = os.environ.get("REPO", "")
    head_sha = os.environ.get("HEAD_SHA", "")
    if not results_dir or not valid_legs_file or not repo or not head_sha:
        _report_error("--report needs FMG_RESULTS_DIR, FMG_VALID_LEGS_FILE, REPO and HEAD_SHA")
        return 2
    details_url = os.environ.get("DETAILS_URL", "")
    valid_legs = load_valid_legs(valid_legs_file)
    if valid_legs is None:
        return 2
    files = sorted(glob.glob(os.path.join(results_dir, "**", "*.json"), recursive=True))
    if not files:
        if os.environ.get("GROUP_JOBS_RESULT", "") == "success":
            _report_error(
                "the group jobs report success but produced ZERO results artifacts — "
                "an inconsistency that would silently drop every per-leg check-run"
            )
            return 1
        print(
            "::warning::no group results artifacts found (group jobs did not "
            "succeed) — nothing to report; the failed/skipped group jobs gate via "
            "ci-summary",
            flush=True,
        )
        return 0
    seen_names = {}
    all_results = []
    for path in files:
        validated = validate_results_file(path, valid_legs, seen_names)
        if validated is None:
            return 1
        all_results.extend(validated)
    fork_denied = failed = 0
    for name, group, failed_step in all_results:
        outcome = post_check_run(
            repo, head_sha, name, group, valid_legs[name], failed_step, details_url
        )
        if outcome == "fork-denied":
            fork_denied += 1
        elif outcome == "failed":
            failed += 1
    posted = len(all_results) - fork_denied - failed
    print(
        f"reporter: {posted} check-run(s) posted, {fork_denied} fork-denied, "
        f"{failed} failed, across {len(files)} group artifact(s)",
        flush=True,
    )
    if failed:
        _report_error(f"{failed} per-leg check-run POST(s) failed unexpectedly — failing closed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(report_main() if "--report" in sys.argv[1:] else main())
