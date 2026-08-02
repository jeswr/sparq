#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4966 — a MISSING CREDENTIAL must fail LOUD, not suppress
# silently into a green run. 🤖 SPARQ agent.
#
# THE DEFECT THIS PINS. Both cross-repo doorbells end in one privileged call gated on
# `secrets.REGISTRY_RING_TOKEN`. That secret does not exist — MEASURED from a runtime
# env dump of run 30411348407, where `GH_TOKEN: ***` renders masked-and-present while
# `REGISTRY_RING_TOKEN:` renders empty (and that log carries ZERO `ESC[36;1m` lines, so
# these are runtime values, not echoed script source). Both guards therefore took the
# fail-soft branch on every single invocation:
#
#     fast-fix-ring   1969 runs, 355 executed, ~130 rings suppressed, 0 sent
#     batch-merge ring  12/12 sampled push runs reached the guard,       0 sent
#
# …and every one of those runs reported SUCCESS. The difference between "the ring
# fired" and "the ring CANNOT fire and never could" was not visible in any field a
# monitor queries. A control that is disabled by missing configuration and green about
# it is indistinguishable from one that works.
#
# HOW THIS SUITE IS BUILT TO FAIL.
#   * It does not re-implement the guard, and it does not grep for it. It EXTRACTS the
#     real `run:` script out of the real workflow YAML (structurally, via PyYAML — so a
#     comment mentioning the guard cannot satisfy it) and EXECUTES it under bash with a
#     stubbed `gh` on PATH. The assertions are on the process exit code, the emitted
#     workflow commands, and whether the privileged dispatch was actually attempted.
#     Reverting either guard to `::notice::` + `exit 0` REDs this file by name.
#   * Both credential states are pinned. Without the credential-PRESENT half, "always
#     exit 1" would satisfy the missing-credential half — the fix could be a brick.
#   * TestTheNewFailureCannotGate drives the REAL ci_summary_gate.render_verdict() to
#     prove the new hard failure cannot red `ci-summary / gate`, and carries an
#     anti-vacuity CONTROL (an UNDECLARED lane failing MUST still red it). Without the
#     control, a render_verdict() that returned 0 unconditionally would satisfy it.
#
# WHY LOUD IS SAFE HERE, given the repo's own stop-the-line doctrine. Neither job's
# check-run can land on a PR or merge_group head: `fast-fix-ring` is a `workflow_run`
# workflow (GitHub runs those from the DEFAULT BRANCH) and `batch-merge`'s ring is
# `push: branches: [main]`. Both land on the MAIN head SHA — MEASURED, both names are
# present in the check-run set of main head 2cbf37359 — which the push-triggered
# `ci-summary / gate` polls. Both are therefore DECLARED in
# .github/advisory-registry.json, the only thing that makes a check-run non-gating
# since #3773. That declaration is load-bearing, so it is asserted below rather than
# assumed.
#
# Run:  python3 scripts/tests/test_ring_credential_loud.py
# Requires `jq` on PATH — see _require_jq() below for why that is a hard precondition
# rather than a skip.

from __future__ import annotations

import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
REGISTRY_PATH = REPO_ROOT / ".github" / "advisory-registry.json"

sys.path.insert(0, str(REPO_ROOT / "scripts"))
import ci_summary_gate as gate  # noqa: E402

SECRET = "REGISTRY_RING_TOKEN"

# job name -> (workflow file, job id). Both are the doorbell shape #4966 covers.
RING_JOBS = {
    "fast-fix ring (advisory)": ("fast-fix-ring.yml", "ring"),
    "ring registry dispatcher": ("batch-merge.yml", "ring"),
}

# A name deliberately NOT in the registry, for the anti-vacuity control.
UNDECLARED_LANE = "some undeclared lane that must gate"


def _ring_step(workflow_file: str, job_id: str) -> dict:
    """The single `run:` step of a ring job, read STRUCTURALLY from the live YAML."""
    doc = yaml.safe_load((WORKFLOWS / workflow_file).read_text())
    steps = [s for s in doc["jobs"][job_id]["steps"] if "run" in s]
    assert len(steps) == 1, f"{workflow_file}:{job_id} expected one run-step, got {len(steps)}"
    return steps[0]


def _require_jq() -> None:
    """Fail on the REAL cause when `jq` is missing (sparq-org/sparq#5342).

    `fast-fix-ring.yml:ring` parses its PR state with `jq -r`/`jq -e`. With no `jq` on
    PATH every one of those parses yields the empty string, the script takes its
    `head repo is not ...` cross-repo branch and exits 0 — and the two execution tests
    below then report that as "the control is a no-op in BOTH states" and "~130 rings
    vanished into green runs", accusing the production ring of the exact regression
    this file exists to prevent. Naming the missing tool costs a reader minutes
    instead of an investigation.

    Applied at harness construction, so it covers EVERY extracted body rather than
    only the one that happens to call `jq` today.

    Deliberately a FAILURE and not a `skipUnless`: this file's whole thesis is that a
    control which goes quiet because a dependency is absent is indistinguishable from
    one that works. Skipping would be that same shape, one level up. It only ever
    fires locally — the ring workflow calls `jq` with no install step of its own, so a
    CI runner without `jq` would already have broken the workflow this file pins.
    """
    if shutil.which("jq") is None:
        raise AssertionError(
            "jq is required to exercise the ring step body, and it is not on PATH. "
            "The fast-fix-ring body parses its PR state with jq; without it the "
            "eligibility checks all compare against the empty string, the script "
            "skips to its cross-repo branch and exits 0, and the assertions in this "
            "file would blame the workflow for a missing local tool. Install jq and "
            "re-run."
        )


class _Harness:
    """Executes an extracted `run:` script with a stubbed `gh` on PATH."""

    def __init__(self, tmp: Path):
        _require_jq()
        self.tmp = tmp
        self.dispatch_log = tmp / "dispatch.log"
        self.summary = tmp / "step-summary.md"
        self.summary.touch()
        bindir = tmp / "bin"
        bindir.mkdir()
        # The stub answers the two `gh` shapes these scripts use: an `api` read of the
        # PR's live state, and the privileged `workflow run` dispatch (recorded, so the
        # tests can assert the doorbell was — or was not — actually pressed).
        pr_state = json.dumps({
            "draft": False, "armed": True,
            "head_repo": "o/r", "head_sha": "f" * 40,
            "labels": ["review:pass"],
        })
        (bindir / "gh").write_text(
            "#!/usr/bin/env bash\n"
            'if [ "$1" = "api" ]; then\n'
            f"  echo '{pr_state}'\n"
            "  exit 0\n"
            "fi\n"
            'if [ "$1" = "workflow" ]; then\n'
            f'  echo "$@" >> "{self.dispatch_log}"\n'
            "  exit 0\n"
            "fi\n"
            'echo "unstubbed gh invocation: $*" >&2\n'
            "exit 97\n"
        )
        (bindir / "gh").chmod(
            (bindir / "gh").stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH
        )
        self.bindir = bindir

    def run(self, script: str, token: str) -> subprocess.CompletedProcess:
        env = dict(os.environ)
        env.update({
            "PATH": f"{self.bindir}:{env['PATH']}",
            SECRET: token,
            "GH_TOKEN": "stub-workflow-token",
            "REPO": "o/r",
            "HEAD_SHA": "f" * 40,
            "RUN_PR_NUMBER": "123",
            "GITHUB_STEP_SUMMARY": str(self.summary),
        })
        return subprocess.run(
            ["bash", "-c", script], env=env, capture_output=True, text=True, timeout=60
        )

    @property
    def rang(self) -> bool:
        return self.dispatch_log.exists() and "dispatch.yml" in self.dispatch_log.read_text()


class TestMissingCredentialIsLoud(unittest.TestCase):
    """CRITERION 3. An empty credential must produce a VISIBLE NON-SUCCESS."""

    def test_empty_credential_exits_non_zero_and_annotates(self):
        for job_name, (wf, job_id) in RING_JOBS.items():
            with self.subTest(job=job_name):
                step = _ring_step(wf, job_id)
                with tempfile.TemporaryDirectory() as d:
                    h = _Harness(Path(d))
                    proc = h.run(step["run"], token="")
                    self.assertNotEqual(
                        proc.returncode, 0,
                        f"{wf}:{job_id} exited 0 with {SECRET} EMPTY — the ring was "
                        f"suppressed and the run still reported success. That is the "
                        f"#4966 defect: ~130 rings vanished into green runs.\n"
                        f"stdout:\n{proc.stdout}",
                    )
                    self.assertIn(
                        "::error", proc.stdout,
                        f"{wf}:{job_id} produced no ::error:: annotation — a "
                        f"`::notice::` is not a visible non-success",
                    )
                    self.assertIn(
                        SECRET, proc.stdout,
                        "the annotation must NAME the missing secret, or the reader "
                        "cannot act on it",
                    )
                    self.assertFalse(
                        h.rang,
                        "no dispatch may be attempted without the credential",
                    )
                    self.assertIn(
                        "SUPPRESSED", h.summary.read_text(),
                        f"{wf}:{job_id} wrote no step-summary row — the suppression "
                        f"must be countable on the run page, not only in the log",
                    )

    def test_the_annotation_is_not_merely_a_notice(self):
        """`::notice::` is what made this invisible for ~130 rings; its return must red
        this test even if an `::error::` were added alongside it."""
        for job_name, (wf, job_id) in RING_JOBS.items():
            with self.subTest(job=job_name):
                script = _ring_step(wf, job_id)["run"]
                guard = script.split(f'if [ -z "${{{SECRET}}}" ]; then', 1)
                self.assertEqual(len(guard), 2, f"{wf}: the {SECRET} guard moved")
                body = guard[1].split("fi", 1)[0]
                self.assertNotIn(
                    "::notice::", body,
                    f"{wf}:{job_id} still reports the missing credential as a NOTICE — "
                    f"a notice on a green run is precisely the silent suppression "
                    f"#4966 exists to remove",
                )


class TestCredentialPresentStillWorks(unittest.TestCase):
    """CRITERION 4. The fix must not be 'always fail' — with the credential present the
    doorbell must still be pressed and the job must still succeed."""

    def test_present_credential_rings_and_exits_zero(self):
        for job_name, (wf, job_id) in RING_JOBS.items():
            with self.subTest(job=job_name):
                step = _ring_step(wf, job_id)
                with tempfile.TemporaryDirectory() as d:
                    h = _Harness(Path(d))
                    proc = h.run(step["run"], token="ghp_a_real_looking_token")
                    self.assertEqual(
                        proc.returncode, 0,
                        f"{wf}:{job_id} failed WITH a credential present — the fix "
                        f"became an unconditional failure.\nstdout:\n{proc.stdout}\n"
                        f"stderr:\n{proc.stderr}",
                    )
                    self.assertTrue(
                        h.rang,
                        f"{wf}:{job_id} did not dispatch jeswr/agent-account-registry "
                        f"even with the credential present — the control is a no-op "
                        f"in BOTH states, which is worse than the bug",
                    )
                    self.assertNotIn("::error", proc.stdout)


class TestTheNewFailureCannotGate(unittest.TestCase):
    """The declaration that makes the new hard failure SAFE, asserted rather than
    assumed. Both check-runs land on the MAIN head SHA that the push-triggered
    `ci-summary / gate` polls (sq-huwr8), so only the advisory registry keeps them from
    blocking merges."""

    def setUp(self) -> None:
        gate.load_advisory_registry(str(REGISTRY_PATH))

    def test_both_ring_jobs_are_declared_advisory(self):
        for job_name in RING_JOBS:
            with self.subTest(job=job_name):
                self.assertTrue(
                    gate.is_advisory(job_name),
                    f"{job_name!r} is NOT declared in .github/advisory-registry.json, "
                    f"so ci_summary_gate.py GATES on it (#3773). Since #4966 this job "
                    f"REDs whenever {SECRET} is unset, and its check-run lands on the "
                    f"main head SHA the push-triggered gate polls — undeclared, that "
                    f"reds `ci-summary / gate` on every push to main.",
                )

    def test_registry_key_matches_the_live_yaml_job_name(self):
        """A rename must not silently re-arm either doorbell as a merge blocker."""
        registry = json.loads(REGISTRY_PATH.read_text())["jobs"]
        for job_name, (wf, job_id) in RING_JOBS.items():
            with self.subTest(job=job_name):
                self.assertIn(job_name, registry)
                entry = registry[job_name]
                self.assertEqual((entry["workflow"], entry["job_id"]), (wf, job_id))
                doc = yaml.safe_load((WORKFLOWS / wf).read_text())
                self.assertEqual(doc["jobs"][job_id]["name"], job_name)

    def test_a_failing_ring_leaves_the_gate_green(self):
        """BEHAVIOUR, via the REAL render_verdict(): the exact new failure mode must
        not red the gate."""
        healthy = [
            {"name": "changed-file test selection", "status": "completed",
             "conclusion": "success", "id": 1, "_workflow_id": 1},
            {"name": "build", "status": "completed", "conclusion": "success",
             "id": 2, "_workflow_id": 1},
        ]
        for job_name in RING_JOBS:
            with self.subTest(job=job_name):
                runs = healthy + [{"name": job_name, "status": "completed",
                                   "conclusion": "failure", "id": 3, "_workflow_id": 2}]
                self.assertEqual(
                    gate.render_verdict(runs), 0,
                    f"a suppressed-ring failure from {job_name!r} red the gate",
                )

    def test_control_undeclared_lane_failure_does_red_the_gate(self):
        """ANTI-VACUITY CONTROL. Same fixture shape, undeclared lane. This MUST red — if
        it ever passes green, every assertion in this class is measuring nothing."""
        healthy = [
            {"name": "changed-file test selection", "status": "completed",
             "conclusion": "success", "id": 1, "_workflow_id": 1},
            {"name": "build", "status": "completed", "conclusion": "success",
             "id": 2, "_workflow_id": 1},
        ]
        self.assertFalse(gate.is_advisory(UNDECLARED_LANE))
        runs = healthy + [{"name": UNDECLARED_LANE, "status": "completed",
                           "conclusion": "failure", "id": 3, "_workflow_id": 2}]
        self.assertEqual(
            gate.render_verdict(runs), 1,
            "CONTROL FAILED: an UNDECLARED lane concluding `failure` did not red the "
            "gate, so the non-gating assertions above are vacuous.",
        )

    def test_declaring_the_rings_does_not_forgive_a_real_gating_failure(self):
        """QUANTIFIER DIRECTION. The declarations must make the DOORBELLS non-gating,
        not buy green by weakening the gate for the commit's own legs."""
        runs = [
            {"name": "changed-file test selection", "status": "completed",
             "conclusion": "success", "id": 1, "_workflow_id": 1},
            {"name": "ring registry dispatcher", "status": "completed",
             "conclusion": "failure", "id": 2, "_workflow_id": 2},
            {"name": "build", "status": "completed", "conclusion": "failure",
             "id": 3, "_workflow_id": 1},
        ]
        self.assertFalse(gate.is_advisory("build"))
        self.assertEqual(
            gate.render_verdict(runs), 1,
            "a real gating leg's failure was forgiven — the gate was weakened",
        )


class TestTheJqPreconditionBlamesTheTool(unittest.TestCase):
    """#5342. A missing `jq` must be reported as a missing `jq`, never as the #4966
    regression — the two execution tests above cannot tell the difference, because a
    jq-less body and a reverted guard both exit 0 without dispatching."""

    def test_harness_refuses_to_run_without_jq(self):
        with tempfile.TemporaryDirectory() as d:
            empty = Path(d) / "empty-path"
            empty.mkdir()
            with mock.patch.dict(os.environ, {"PATH": str(empty)}):
                self.assertIsNone(
                    shutil.which("jq"), "PATH was not actually emptied of jq"
                )
                with self.assertRaises(AssertionError) as caught:
                    _Harness(Path(d))

        message = str(caught.exception)
        self.assertIn("jq", message, "the failure must name the missing tool")
        for accusation in ("no-op", "vanished into green runs"):
            self.assertNotIn(
                accusation, message,
                f"the tool-missing failure reused {accusation!r} from the ring "
                f"assertions — that is the misleading diagnosis #5342 removes",
            )


class TestSuiteIsWiredIntoCI(unittest.TestCase):
    """Anti-vacuity anchor: without this the file could leave CI's reachable set."""

    def test_docs_quality_runs_this_suite(self):
        text = (WORKFLOWS / "docs-quality.yml").read_text()
        self.assertIn("scripts/tests/test_ring_credential_loud.py", text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
