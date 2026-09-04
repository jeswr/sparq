#!/usr/bin/env python3
"""Wiring pins for the orphan-thread sweep (issue #4542).

This sweep RESOLVES A SECURITY REVIEW THREAD. Everything that makes that safe lives in
two places — the eligibility rules in `scripts/orphan-thread-sweeper.py` (pinned by its own
`--self-test`) and the YAML seam that decides whether the policy runs at all, on what
token, and under what bound. Neither the Python nor a substring assertion can see the seam,
so it is read STRUCTURALLY here, from the parsed document.

The shapes this file exists to red are the ones measured to survive every other kind of
check in this estate: `continue-on-error` on the job or the step, `if: false`, a `|| true`
suffix, a `--dry-run` left in the scheduled step, a missing or unbounded per-tick cap, a
trigger that would put this sweep on a pull-request head, a widened permission grant, and
an unpinned third-party action. Every one of them is applied to a MUTATED copy of the real
workflow in `TestTheGuardRedsOnEachShape`, because a guard nobody has watched go red is the
thing a guard exists to prevent. That is why both guards below — `seam_findings` and
`policy_findings` — are PURE functions of the parsed document and never read the file
themselves: a check that can only be run against the one document that satisfies it cannot
be shown to reject anything.
"""

from __future__ import annotations

import copy
import importlib.util
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
SWEEP_YML = WORKFLOWS / "orphan-thread-sweeper.yml"
SWEEP_PY = REPO_ROOT / "scripts" / "orphan-thread-sweeper.py"
DOCS_QUALITY_YML = WORKFLOWS / "docs-quality.yml"

SWEEP_SCRIPT = "scripts/orphan-thread-sweeper.py"
CAP_FLAG = "--max-resolutions"
SELF_TEST_FLAG = "--self-test"

# The bound this repository is willing to have a cron apply to security review threads in
# one tick, without a person looking. Deliberately small.
MAX_PERMITTED_CAP = 10

# A line invoking the sweep must BE the whole command. `||`, `;`, `|` and `&` each decide
# the exit status the runner sees, so none may follow it (`… --self-test || true` still
# "contains" `… --self-test`, which is why a substring match cannot enforce this).
BARE_SWEEP_LINE = re.compile(r"^[ \t]*python3 +" + re.escape(SWEEP_SCRIPT) + r"[^|;&]*$")

SHA_PIN = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def load_module(path: Path, name: str) -> ModuleType:
    """Import a hyphenated script by path (the scripts are not importable by name)."""
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None, path
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def run_of(step: dict) -> str:
    return str(step.get("run") or "")


def checkout_cone(document: dict) -> list[str]:
    """The paths the sweep's checkout materialises — its `sparse-checkout` cone.

    Whitespace-separated, matching how actions/checkout reads the input, so a one-line
    scalar and a block list are the same thing here.
    """
    for job in (document.get("jobs") or {}).values():
        for step in job.get("steps") or []:
            raw = (step.get("with") or {}).get("sparse-checkout")
            if raw:
                return [entry for entry in str(raw).split() if entry]
    return []


def sweep_steps(document: dict) -> list[tuple[str, dict, dict]]:
    """(job_id, job, step) for every step whose `run` invokes the sweep script."""
    return [
        (job_id, job, step)
        for job_id, job in (document.get("jobs") or {}).items()
        for step in (job.get("steps") or [])
        if any(
            line.strip().startswith("python3") and SWEEP_SCRIPT in line
            for line in run_of(step).splitlines()
        )
    ]


# Every `if:` permitted on a step that runs the sweep, keyed by step name. FAIL-CLOSED in
# both directions: a step that runs the sweep must appear here, and its `if:` must match
# EXACTLY — so `if: false`, a plausible-looking `github.event_name` guard, and a brand-new
# unreviewed step all red rather than inheriting silence from a permissive predicate.
# Both are None: neither step may carry a condition. The self-test step running
# unconditionally is what makes it a precondition; the sweep step running unconditionally
# is what stops a failed self-test from being the only reason nothing was swept.
SEAM_STEP_IFS: dict[str, str | None] = {
    "Self-test policy": None,
    "Resolve orphaned review threads": None,
}


def seam_findings(document: dict) -> list[tuple[str, str]]:
    """(kind, message) for every way this document could run the sweep and hide a failure."""
    hosts = sweep_steps(document)
    if not hosts:
        return [("absent", f"no step runs {SWEEP_SCRIPT} at all")]
    findings: list[tuple[str, str]] = []
    for job_id, job, step in hosts:
        name = str(step.get("name") or "<unnamed>")
        if job.get("continue-on-error") not in (None, False):
            findings.append((
                "job-continue-on-error",
                f"job {job_id!r} hosting {name!r} is continue-on-error, so a failed sweep "
                "reports the job green",
            ))
        if step.get("continue-on-error") not in (None, False):
            findings.append((
                "step-continue-on-error",
                f"step {name!r} is continue-on-error, so it cannot red its own failure",
            ))
        if job.get("if") is not None:
            findings.append((
                "if", f"job {job_id!r} carries `if: {job.get('if')!r}`"
            ))
        if name not in SEAM_STEP_IFS:
            findings.append((
                "undeclared",
                f"step {name!r} runs the sweep but is not declared in SEAM_STEP_IFS",
            ))
        elif step.get("if") != SEAM_STEP_IFS[name]:
            findings.append((
                "if",
                f"step {name!r} carries `if: {step.get('if')!r}`, not the vetted "
                f"{SEAM_STEP_IFS[name]!r}",
            ))
        run = run_of(step)
        for line in run.splitlines():
            if line.strip().startswith("python3") and SWEEP_SCRIPT in line:
                if not BARE_SWEEP_LINE.match(line):
                    findings.append((
                        "discard",
                        f"step {name!r} does not invoke the sweep as a bare command: "
                        f"{line.strip()!r}",
                    ))
        if "set +e" in run:
            findings.append(("discard", f"step {name!r} disables errexit with `set +e`"))
    return findings


def policy_findings(document: dict) -> list[tuple[str, str]]:
    """(kind, message) for every way this document would run the sweep with the wrong
    trigger, the wrong privileges, an unpinned action, or without its per-tick bound.

    A pure function of the parsed document, for the same reason `seam_findings` is: the
    mutation class below feeds it MUTATED copies of the real workflow and requires the
    corresponding finding, which is impossible for a check that reads the file itself.
    """
    findings: list[tuple[str, str]] = []

    # PyYAML parses the unquoted key `on:` as the boolean True.
    triggers = set(document.get(True) or document.get("on") or {})
    if triggers != {"schedule", "workflow_dispatch"}:
        findings.append((
            "trigger",
            f"triggers are {sorted(map(str, triggers))}, not schedule + workflow_dispatch; "
            "a per-ref trigger would run pull-request-authored policy against the "
            "repository, expose this workflow to the #3776 checkout ref-skew class, and "
            "publish a check-run that `ci-summary / gate` would then wait on",
        ))

    if document.get("permissions") != {"contents": "read", "pull-requests": "write"}:
        findings.append((
            "permissions",
            f"top-level permissions are {document.get('permissions')!r}; "
            "`pull-requests: write` is exactly what resolveReviewThread and the review "
            "comment reply consume, and anything beyond it is authority this sweep never "
            "exercises — `contents: write` in particular would let a mistake here reach "
            "the repository itself",
        ))

    for job_id, job in (document.get("jobs") or {}).items():
        if job.get("permissions") is not None:
            findings.append((
                "job-permissions",
                f"job {job_id!r} re-declares permissions; the top-level grant is the pin",
            ))
        for step in job.get("steps") or []:
            uses = step.get("uses")
            if uses and not SHA_PIN.match(str(uses)):
                findings.append((
                    "unpinned",
                    f"{uses!r} is not pinned to a full commit SHA — a floating tag is a "
                    "mutable dependency on a job that can resolve review threads",
                ))

    hosts = sweep_steps(document)
    names = [str(step.get("name")) for _j, _job, step in hosts]
    if names != ["Self-test policy", "Resolve orphaned review threads"]:
        findings.append((
            "order",
            f"the sweep steps are {names}, not the self-test followed by the sweep; the "
            "eligibility rules must be validated before a single thread is touched",
        ))

    live = [
        step for _j, _job, step in hosts
        if SELF_TEST_FLAG not in " ".join(run_of(step).split())
    ]
    if len(live) != 1:
        findings.append((
            "invocations",
            f"{len(live)} live sweep invocation(s); two double-act and zero sweep nothing",
        ))
        return findings

    run = " ".join(run_of(live[0]).split())
    if "--dry-run" in run:
        findings.append((
            "dry-run",
            "the scheduled step carries --dry-run; a sweep permanently in dry-run reports "
            "beautifully and removes no dead end, which is the state this sweep was built "
            "to leave",
        ))
    if CAP_FLAG not in run:
        findings.append((
            "bound-missing",
            f"{CAP_FLAG} is not on the command line; a bound that is only a default is not "
            "a bound",
        ))
    else:
        value = run.split(CAP_FLAG, 1)[1].split()
        value = value[0] if value else ""
        if not value.isdigit() or not 1 <= int(value) <= MAX_PERMITTED_CAP:
            findings.append((
                "bound-unbounded",
                f"{CAP_FLAG} is {value!r}; an unbounded tick would resolve a whole backlog "
                "of security review threads in one unreviewable burst",
            ))
    return findings


class TestThePolicyPinsHold(unittest.TestCase):
    """The real workflow satisfies every pin above."""

    def test_the_real_workflow_has_no_policy_findings(self) -> None:
        self.assertEqual(policy_findings(load(SWEEP_YML)), [])


class TestTheSweepIsReachedAndBounded(unittest.TestCase):
    def setUp(self) -> None:
        self.document = load(SWEEP_YML)
        self.hosts = sweep_steps(self.document)

    def test_the_harness_is_comment_blind(self) -> None:
        """The tripwire for the measured false-green: a COMMENT must not satisfy a pin."""
        commented = yaml.safe_load(
            "jobs:\n  j:\n    steps:\n"
            f"      # python3 {SWEEP_SCRIPT} {CAP_FLAG} 10\n"
            "      - name: decoy\n        run: echo nothing-here\n"
        )
        self.assertEqual(sweep_steps(commented), [])
        live = yaml.safe_load(
            "jobs:\n  j:\n    steps:\n"
            f"      - name: real\n        run: python3 {SWEEP_SCRIPT} {CAP_FLAG} 10\n"
        )
        self.assertEqual(len(sweep_steps(live)), 1)


class TestTheSelfTestCanRunInTheCheckoutTheCronGets(unittest.TestCase):
    """The precondition step must be RUNNABLE, not merely present.

    Every other pin in this file reads the workflow. None of them can see the one thing
    that decides whether the self-test passes at 03:00: the checkout is SPARSE, and the
    self-test resolves `.github/workflows/<file>` on disk for each registry entry. A cone
    that omits that directory turns "runs FIRST so a regression reds the cron before a
    single thread is touched" into "reds the cron every tick, forever" — the job fails at
    its first step and the sweep this workflow exists for never runs at all. So this class
    does not assert about the cone; it BUILDS one and runs the real step inside it.
    """

    def _run_in(self, cone: list[str]) -> subprocess.CompletedProcess:
        """`--self-test`, executed in a checkout materialising exactly `cone`.

        Cone mode also materialises the root-level files, so they are linked in too —
        leaving them out would make this guard stricter than the runner and red a change
        that in fact works.
        """
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for entry in cone + [
                p.name for p in REPO_ROOT.iterdir() if p.is_file()
            ]:
                source = REPO_ROOT / entry
                if not source.exists():
                    continue
                link = root / entry
                link.parent.mkdir(parents=True, exist_ok=True)
                if not link.exists():
                    link.symlink_to(source)
            script = root / SWEEP_SCRIPT
            if not script.exists():
                return subprocess.CompletedProcess(
                    [], 127, "", f"{SWEEP_SCRIPT} is not in the cone {cone}"
                )
            return subprocess.run(
                [sys.executable, str(script), SELF_TEST_FLAG],
                capture_output=True,
                text=True,
                cwd=root,
                check=False,
            )

    def test_the_declared_cone_is_enough_to_run_the_self_test(self) -> None:
        cone = checkout_cone(load(SWEEP_YML))
        self.assertTrue(cone, "the sweep's checkout declares no sparse-checkout cone")
        done = self._run_in(cone)
        self.assertEqual(
            0,
            done.returncode,
            f"`{SELF_TEST_FLAG}` cannot run in the cron's own checkout (cone {cone}):\n"
            f"{done.stderr}",
        )

    def test_a_cone_missing_the_workflows_directory_reds(self) -> None:
        """MUTANT: the cone as first written. The guard must see this, or it sees nothing."""
        done = self._run_in(["scripts"])
        self.assertNotEqual(
            0,
            done.returncode,
            "a checkout without .github/workflows passed the self-test, so this guard "
            "cannot distinguish a runnable cone from an unrunnable one",
        )
        self.assertIn("codeql.yml", done.stderr)


class TestTheYamlSeamIsIntact(unittest.TestCase):
    def setUp(self) -> None:
        self.document = load(SWEEP_YML)

    def _kinds(self, *kinds: str) -> list[str]:
        return [m for kind, m in seam_findings(self.document) if kind in kinds]

    def test_the_real_workflow_has_no_findings_at_all(self) -> None:
        self.assertEqual(seam_findings(self.document), [])

    def test_every_declared_step_still_exists(self) -> None:
        """A rename must not leave a dead allowance behind, silently vetting nothing."""
        live = {str(step.get("name")) for _j, _job, step in sweep_steps(self.document)}
        self.assertEqual(sorted(set(SEAM_STEP_IFS) - live), [])


class TestTheGuardRedsOnEachShape(unittest.TestCase):
    """Watch the guards go red. Each shape is applied to the REAL document."""

    def setUp(self) -> None:
        self.document = load(SWEEP_YML)

    def _mutate(self, fn) -> list[tuple[str, str]]:
        document = copy.deepcopy(self.document)
        fn(document)
        return seam_findings(document)

    def _policy(self, fn) -> set[str]:
        document = copy.deepcopy(self.document)
        fn(document)
        return {kind for kind, _m in policy_findings(document)}

    def _kinds(self, findings: list[tuple[str, str]]) -> set[str]:
        return {kind for kind, _m in findings}

    # ---- the policy pins ---------------------------------------------------------------

    def test_a_pull_request_trigger(self) -> None:
        def mutate(doc):
            key = True if True in doc else "on"
            doc[key]["pull_request"] = {"branches": ["main"]}

        self.assertIn("trigger", self._policy(mutate))

    def test_a_widened_permission_grant(self) -> None:
        for widened in (
            {"contents": "write", "pull-requests": "write"},
            {"contents": "read", "pull-requests": "write", "issues": "write"},
            {"contents": "read"},
        ):
            def mutate(doc, widened=widened):
                doc["permissions"] = widened

            self.assertIn("permissions", self._policy(mutate), widened)

    def test_a_job_level_permission_redeclaration(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["permissions"] = {"contents": "write"}

        self.assertIn("job-permissions", self._policy(mutate))

    def test_an_action_unpinned_to_a_floating_tag(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"][0]["uses"] = "actions/checkout@v7"

        self.assertIn("unpinned", self._policy(mutate))

    def test_a_dry_run_left_in_the_scheduled_step(self) -> None:
        def mutate(doc):
            step = doc["jobs"]["sweep"]["steps"][-1]
            step["run"] = run_of(step).rstrip() + " --dry-run"

        self.assertIn("dry-run", self._policy(mutate))

    def test_the_per_tick_bound_removed(self) -> None:
        def mutate(doc):
            step = doc["jobs"]["sweep"]["steps"][-1]
            step["run"] = run_of(step).replace(f"{CAP_FLAG} 10", "")

        self.assertIn("bound-missing", self._policy(mutate))

    def test_the_per_tick_bound_raised_past_the_ceiling(self) -> None:
        def mutate(doc):
            step = doc["jobs"]["sweep"]["steps"][-1]
            step["run"] = run_of(step).replace(f"{CAP_FLAG} 10", f"{CAP_FLAG} 1000")

        self.assertIn("bound-unbounded", self._policy(mutate))

    def test_the_self_test_removed_from_in_front_of_the_sweep(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"] = [
                s for s in doc["jobs"]["sweep"]["steps"]
                if SELF_TEST_FLAG not in run_of(s)
            ]

        self.assertIn("order", self._policy(mutate))

    # ---- the seam --------------------------------------------------------------------

    def test_job_continue_on_error(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["continue-on-error"] = True

        self.assertIn("job-continue-on-error", self._kinds(self._mutate(mutate)))

    def test_step_continue_on_error(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"][-1]["continue-on-error"] = True

        self.assertIn("step-continue-on-error", self._kinds(self._mutate(mutate)))

    def test_if_false_on_the_step(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"][-1]["if"] = False

        self.assertIn("if", self._kinds(self._mutate(mutate)))

    def test_if_false_on_the_job(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["if"] = False

        self.assertIn("if", self._kinds(self._mutate(mutate)))

    def test_or_true_appended_to_the_run_line(self) -> None:
        def mutate(doc):
            step = doc["jobs"]["sweep"]["steps"][-1]
            step["run"] = run_of(step).rstrip() + " || true"

        self.assertIn("discard", self._kinds(self._mutate(mutate)))

    def test_a_brand_new_undeclared_sweep_step(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"].append(
                {"name": "sneaky", "run": f"python3 {SWEEP_SCRIPT} --repo x/y"}
            )

        self.assertIn("undeclared", self._kinds(self._mutate(mutate)))

    def test_deleting_the_sweep_entirely(self) -> None:
        def mutate(doc):
            doc["jobs"]["sweep"]["steps"] = [
                s for s in doc["jobs"]["sweep"]["steps"] if not run_of(s)
            ]

        self.assertIn("absent", self._kinds(self._mutate(mutate)))


class TestTheRegistryCannotNameAnUnfalsifiableAnalysis(unittest.TestCase):
    """The registry grants authority to resolve an author's threads. Bound it.

    An analysis with no workflow behind it can never be PROVEN to have stopped running, so
    an entry for it could only ever be a way to resolve LIVE review threads. Copilot code
    review is the live example — it is a repository setting, not a workflow — and it files
    review threads on this repository today.
    """

    def setUp(self) -> None:
        self.module = load_module(SWEEP_PY, "orphan_thread_sweeper_under_test")

    def test_every_registered_analysis_maps_to_a_workflow_file_that_exists(self) -> None:
        self.assertTrue(self.module.ORPHANABLE_ANALYSES, "an empty registry sweeps nothing")
        for login, analysis in self.module.ORPHANABLE_ANALYSES.items():
            path = WORKFLOWS / analysis.workflow
            self.assertTrue(
                path.is_file(),
                f"{login!r} maps to {analysis.workflow!r}, which is not a workflow in this "
                "repository — its state could never be read, so eligibility would be "
                "decided on an unanswerable question",
            )
            self.assertTrue(
                analysis.marker,
                f"{login!r} has no body marker, so one disabled workflow would orphan "
                "every other analysis that shares this author",
            )

    def test_no_reviewer_that_is_not_a_workflow_is_registered(self) -> None:
        for login in ("copilot-pull-request-reviewer", "copilot", "github-actions", "jeswr"):
            self.assertNotIn(
                login,
                self.module.ORPHANABLE_ANALYSES,
                f"{login!r} is not a workflow-backed analysis; nothing could prove it "
                "stopped running, so registering it would authorise resolving live threads",
            )

    def test_only_a_disabled_state_can_ever_be_eligible(self) -> None:
        """Fail-closed on `active` and on every state the script does not recognise."""
        self.assertEqual(
            self.module.DISABLED_STATES, frozenset({"disabled_manually", "disabled_inactivity"})
        )
        self.assertNotIn("active", self.module.DISABLED_STATES)

    def test_exactly_one_class_mutates(self) -> None:
        actions = self.module.CLASS_ACTIONS
        resolving = [k for k, v in actions.items() if v == self.module.ACTION_RESOLVE]
        self.assertEqual(
            resolving,
            ["orphaned"],
            "the class enum is the routing table; a second acting class is a second, "
            "unreviewed authority to resolve a review thread",
        )


class TestSelfTestsRunInCi(unittest.TestCase):
    """A guard nothing runs is decoration. Both must gate on the PR that changes them."""

    def test_docs_quality_runs_the_self_test_and_this_suite(self) -> None:
        blob = "\n".join(
            run_of(step)
            for job in (load(DOCS_QUALITY_YML).get("jobs") or {}).values()
            for step in (job.get("steps") or [])
        )
        self.assertIn(f"python3 {SWEEP_SCRIPT} {SELF_TEST_FLAG}", blob)
        self.assertIn("python3 scripts/tests/test_orphan_thread_sweeper_wiring.py", blob)


if __name__ == "__main__":
    unittest.main(verbosity=2)
