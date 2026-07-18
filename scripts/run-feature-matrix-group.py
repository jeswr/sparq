#!/usr/bin/env python3
# [FABLE-5] CI-economy grouping (maintainer directive 2026-07-18): run ONE bin-packed
# GROUP of opt-in feature-matrix legs inside a single runner job, and emit the
# GATE-CRITICAL per-leg check-run for each leg as it finishes.
#
# WHY: feature-matrix.yml used to spawn one runner per leg (159 legs = 159 runners,
# each paying checkout + toolchain + cache-restore + dependency compile). The legs are
# now bin-packed by scripts/assemble-feature-matrix.py --grouped; this script is the
# group job's engine. Consecutive legs share the job's warm target dir, so same-crate
# legs recompile only the crate itself, never the dependency stack.
#
# NAME CONTRACT (do NOT relax — the same contract the assembler's header pins): the
# check-run NAME for every leg is `opt-in <name>`, BYTE-IDENTICAL to the name the
# per-leg matrix job used to emit (`name: opt-in ${{ matrix.name }}`). The single
# required `ci-summary / gate` aggregator discovers check-runs on the head commit BY
# NAME (scripts/ci_summary_gate.py polls repos/<repo>/commits/<sha>/check-runs), and
# the assemble self-test (scripts/tests/test_feature_matrix_assemble.py) asserts the
# emitted name set against the golden snapshot. This script creates each leg's
# check-run via the Checks API — ALWAYS `status: completed` (a terminal check-run can
# never hold the polling gate pending; sq-ipkku semantics make late terminal arrivals
# safe), with conclusion success/failure per leg.
#
# FAILURE SEMANTICS: a failed leg does NOT stop the group (the old matrix ran
# fail-fast: false — every leg reports, so one broken feature shows the full
# picture). At the end the group job FAILS LOUDLY, naming exactly which legs failed
# (::error + job summary), and each failed leg also carries its own red `opt-in
# <name>` check-run for precise attribution.
#
# TOKEN DEGRADATION (fork PRs): a read-only GITHUB_TOKEN cannot create check-runs.
# The per-leg POST then degrades to a ::warning and the group job's OWN
# success/failure remains the (coarser) gating signal — requiredness flows through
# `ci-summary / gate`, never through any individual leg name (sq-fmx4u.4), so a
# missing per-leg check-run can neither hang nor un-gate the merge.
#
# Env:
#   GROUP_LEGS   (required) JSON array of {name, crate, features, test}
#   GROUP_NAME   (required) the group's display id (diagnostics only)
#   REPO         owner/repo — for the Checks API POST
#   HEAD_SHA     commit to attach check-runs to (PR head sha / github.sha)
#   DETAILS_URL  workflow-run URL embedded in each check-run
#   GITHUB_STEP_SUMMARY  (optional) job-summary sink
#   FMG_CARGO / FMG_GH   (tests only) stub binaries for cargo / gh

import json
import os
import subprocess
import sys

CARGO = os.environ.get("FMG_CARGO", "cargo")
GH = os.environ.get("FMG_GH", "gh")
BUILD_ATTEMPTS = 3  # same bounded transient-registry retry the per-leg matrix used


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


def post_check_run(leg, failed_step):
    """Create the terminal `opt-in <name>` check-run for this leg (see NAME
    CONTRACT above). Degrades to a warning on API failure (read-only token)."""
    repo = os.environ.get("REPO", "")
    head_sha = os.environ.get("HEAD_SHA", "")
    if not repo or not head_sha:
        print(
            "::warning::REPO/HEAD_SHA unset — skipping per-leg check-run emission",
            flush=True,
        )
        return
    name = f"opt-in {leg['name']}"
    group = os.environ.get("GROUP_NAME", "?")
    if failed_step is None:
        conclusion, title = "success", "leg passed"
        summary = (
            f"build/test/clippy green for `-p {leg['crate']} --features "
            f"{leg['features']}` (grouped leg — ran inside feature-matrix group "
            f"`{group}`)."
        )
    else:
        conclusion, title = "failure", f"leg FAILED at {failed_step}"
        summary = (
            f"`cargo {failed_step}` failed for `-p {leg['crate']} --features "
            f"{leg['features']}`. See the feature-matrix group job `{group}` log "
            f"for the full output."
        )
    payload = {
        "name": name,
        "head_sha": head_sha,
        "status": "completed",
        "conclusion": conclusion,
        "output": {"title": title, "summary": summary},
    }
    details_url = os.environ.get("DETAILS_URL", "")
    if details_url:
        payload["details_url"] = details_url
    proc = subprocess.run(
        [GH, "api", f"repos/{repo}/check-runs", "--method", "POST", "--input", "-"],
        input=json.dumps(payload).encode("utf-8"),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        # Read-only token (fork PR) or transient API failure: the group job's own
        # conclusion still gates via ci-summary — degrade loudly, never fatally.
        err = proc.stderr.decode("utf-8", "replace").strip().splitlines()
        print(
            f"::warning::could not create check-run {name!r} "
            f"({err[-1] if err else 'unknown error'}) — the group job's own "
            "conclusion remains the gating signal",
            flush=True,
        )


def main():
    try:
        legs = json.loads(os.environ["GROUP_LEGS"])
    except (KeyError, json.JSONDecodeError) as exc:
        print(f"::error::GROUP_LEGS missing/malformed: {exc}", file=sys.stderr)
        return 2
    if not isinstance(legs, list) or not legs:
        print("::error::GROUP_LEGS must be a non-empty JSON array", file=sys.stderr)
        return 2
    group = os.environ.get("GROUP_NAME", "?")
    print(f"feature-matrix group {group}: {len(legs)} leg(s)", flush=True)
    results = []
    for leg in legs:
        print(f"::group::opt-in {leg['name']}", flush=True)
        failed_step = run_leg(leg)
        print("::endgroup::", flush=True)
        results.append((leg, failed_step))
        post_check_run(leg, failed_step)
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


if __name__ == "__main__":
    sys.exit(main())
