#!/usr/bin/env python3
# [OPUS-5] Regression guard for the 2026-07-26 model deprecation.
"""test_no_deprecated_models.py — a deprecated model may not occupy ANY routing position.

MAINTAINER DIRECTIVE 2026-07-26: "deprecate the use of fable and opus entirely in favour of opus5"
and "deprecate sonnet and haiku for docs writing in favor of gpt 5.6 sol".

Deleting the aliases from orchestration/routing.toml once is an EDIT; it decays the moment someone
copies an old chain back in. This suite is what makes it an INVARIANT, and it is deliberately
CROSS-SURFACE, because the routing table is not the only place a model gets selected:

  1. orchestration/routing.toml      — the issue-native routing table (chains + catalog)
  2. .claude/settings.json           — PreToolUse `type: "agent"` hooks carry their own `model`
  3. .claude/workflows/*.js          — `agent({model: ...})` dispatch sites + the TIER table
  4. .claude/agents/*.md             — role-agent frontmatter `model:`

Surface 2 is why this file exists rather than a routing-table-only assertion: the sparq-perf-reviewer
arm gate sat at `"model": "opus"` — a BARE ALIAS — long after the routing table had moved to opus5.
A routing-table-scoped guard would have stayed green while the live PR-arming gate ran on the
deprecated model.

BARE ALIASES ARE THEMSELVES A FINDING. `opus` resolved to claude-opus-4-8 for days after Opus 5
shipped, so "it points at the right model today" is not a property you can assert about an alias.
Every routing position must name a FULL model id.
"""
import json
import re
import sys
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml"

# Retired 2026-07-26. Both halves matter: the ALIAS (what a chain writes) and the concrete PROVIDER
# ID (what the alias resolved to) — banning only the alias lets the model return under a new name.
DEPRECATED_ALIASES = {"fable", "opus"}
DEPRECATED_PROVIDER_MODELS = {"claude-fable-5", "claude-opus-4-8"}

# Aliases whose target has moved before and can move again. A routing position must not use them.
BARE_ALIASES = {"opus", "fable", "sonnet", "haiku", "opus5", "sol", "luna", "terra"}

# The surviving top tier, probe-verified 2026-07-26 (`claude --model claude-opus-5 -p` -> OK,
# modelUsage canonicalModel "claude-opus-5").
OPUS5_ID = "claude-opus-5"


def _routing_doc():
    try:
        import tomllib
    except ModuleNotFoundError:  # pragma: no cover
        import tomli as tomllib
    with open(REPO_ROOT / "orchestration" / "routing.toml", "rb") as fh:
        return tomllib.load(fh)


class TestRoutingTable(unittest.TestCase):
    """Surface 1 — orchestration/routing.toml."""

    @classmethod
    def setUpClass(cls):
        cls.doc = _routing_doc()

    def _chains(self):
        yield "defaults", list(self.doc.get("defaults", {}).get("model_chain", []))
        for r in self.doc.get("route", []):
            where = r.get("role") or ",".join(r.get("match_labels", [])) or "<unnamed>"
            yield where, list(r.get("model_chain", []))

    def test_no_chain_names_a_deprecated_alias(self):
        """MUTANT: put `fable` or `opus` back in any model_chain => RED."""
        for where, chain in self._chains():
            self.assertEqual(
                sorted(set(chain) & DEPRECATED_ALIASES), [],
                f"{where}: model_chain names a model retired on 2026-07-26; use 'opus5'")

    def test_catalog_defines_no_deprecated_alias_or_provider_id(self):
        """MUTANT: re-add [models.fable], or pin claude-opus-4-8 under any alias => RED."""
        for name, spec in self.doc.get("models", {}).items():
            self.assertNotIn(name, DEPRECATED_ALIASES,
                             f"[models.{name}] is a retired alias")
            self.assertNotIn(
                spec.get("provider_model"), DEPRECATED_PROVIDER_MODELS,
                f"[models.{name}] pins a retired provider id under a non-retired alias name")

    def test_opus5_is_the_sole_anthropic_routing_tier(self):
        """Every anthropic model REACHABLE from a chain must be opus5. This is the positive form
        of the rule: the negative tests above ban two known names, this one bans an unknown third."""
        reachable = {m for _, chain in self._chains() for m in chain}
        anthropic = {m for m in reachable
                     if self.doc["models"].get(m, {}).get("provider") == "anthropic"}
        self.assertEqual(anthropic, {"opus5"},
                         "opus5 must be the only anthropic model reachable from a routing chain")

    def test_opus5_pins_the_full_probe_verified_id(self):
        self.assertEqual(self.doc["models"]["opus5"]["provider_model"], OPUS5_ID)

    def test_docs_chain_leads_with_sol_and_names_no_cheap_anthropic_tier(self):
        """MUTANT: put haiku/sonnet back at the head of role=docs => RED.

        This is the SECOND half of the directive ("deprecate sonnet and haiku for docs writing in
        favor of gpt 5.6 sol"). Scoped to role=docs on purpose: haiku and sonnet are NOT banned
        from the repo — they still serve non-docs-writing roles in .claude/agents (mechanical
        retrieval, mechanical verification, bulk implementation, triage, context monitoring),
        which the maintainer did not deprecate.
        """
        docs = [r for r in self.doc["route"] if r.get("role") == "docs"]
        self.assertEqual(len(docs), 1, "exactly one role=docs route expected")
        chain = docs[0]["model_chain"]
        self.assertEqual(chain[0], "sol", "docs writing must lead with sol (gpt-5.6)")
        self.assertEqual(sorted(set(chain) & {"haiku", "sonnet"}), [],
                         "docs writing was moved off the cheap anthropic tiers on 2026-07-26")

    def test_no_chain_is_empty(self):
        """An escalation chain that cannot escalate is worse than a deprecated rung: it dead-ends
        silently. Every chain must retain at least one reachable model."""
        for where, chain in self._chains():
            self.assertTrue(chain, f"{where}: model_chain is empty — it can never dispatch")

    def test_single_model_chains_terminate_explicitly(self):
        """Collapsing ["opus5", "opus"] onto ["opus5"] removed a rung. A one-rung chain is only
        safe if exhaustion has a DEFINED exit — `escalate = true` (park to a human) — or if the
        chain is cross-provider (another provider's outage cannot starve it). Otherwise a single
        capacity outage stalls the role with no escape."""
        for r in self.doc.get("route", []):
            chain = r.get("model_chain", [])
            if len(chain) > 1:
                continue
            where = r.get("role") or ",".join(r.get("match_labels", []))
            providers = {self.doc["models"][m]["provider"] for m in chain}
            self.assertTrue(
                r.get("escalate") or len(providers) > 1,
                f"{where}: single-model chain {chain} without `escalate = true` — on chain "
                f"exhaustion it can only defer forever with no human exit")


class TestProviderPreference(unittest.TestCase):
    """[OPUS-5] Opus 5 is preferred over sol EXCEPT on `area:gui` (maintainer 2026-07-26).

    WHY THIS IS A GUARD AND NOT JUST AN EDIT: sol did not win ~every implementation route through
    any adaptive heuristic. The registry's allocator walks `model_chain` IN ORDER and claims the
    first available account, so the sol-first literal — set during a 2026-07-18 high-availability
    window — simply won every route on every tick, and kept winning long after availability
    shifted. Measured 2026-07-26: 11 of the 12 most recent orchestrator PRs were sol-implemented.
    A preference expressed purely as chain ORDER decays exactly this way, so the order is pinned.
    """

    # The routes where opus5 and sol are BOTH viable implementors. role:docs is excluded: the
    # separate 2026-07-26 docs-writing directive puts sol first there deliberately.
    OPUS5_FIRST_ROLES = ("impl", "site", "ci", "perf")
    GUI_ROLE = "gui"

    @classmethod
    def setUpClass(cls):
        cls.doc = _routing_doc()
        cls.routes = {r["role"]: r for r in cls.doc["route"] if r.get("role")}

    def test_default_prefers_opus5_over_sol(self):
        """MUTANT: reorder any of these chains back to sol-first => RED."""
        for role in self.OPUS5_FIRST_ROLES:
            chain = self.routes[role]["model_chain"]
            self.assertEqual(chain[0], "opus5",
                             f"role:{role} must prefer opus5 over sol (maintainer 2026-07-26)")
            self.assertLess(chain.index("opus5"), chain.index("sol"),
                            f"role:{role}: opus5 must outrank sol")

    def test_defaults_chain_prefers_opus5(self):
        self.assertEqual(self.doc["defaults"]["model_chain"][0], "opus5")

    def test_preference_is_not_exclusion(self):
        """sol must stay REACHABLE on non-GUI work. MUTANT: drop sol from a chain => RED."""
        for role in self.OPUS5_FIRST_ROLES:
            self.assertIn("sol", self.routes[role]["model_chain"],
                          f"role:{role}: sol must remain a fallback, not be excluded")

    def test_gui_carve_out_keeps_sol_first(self):
        """MUTANT: delete the role:gui route, or flip it to opus5-first => RED."""
        self.assertIn(self.GUI_ROLE, self.routes,
                      "the area:gui carve-out route is missing entirely")
        chain = self.routes[self.GUI_ROLE]["model_chain"]
        self.assertEqual(chain[0], "sol",
                         "GUI work keeps sol first (original-builder steer, task #331)")

    def test_gui_carve_out_is_preference_not_exclusion(self):
        """GUI must stay dispatchable during a sol/OpenAI outage."""
        self.assertIn("opus5", self.routes[self.GUI_ROLE]["model_chain"])

    def test_every_preference_chain_terminates_cross_provider(self):
        """Both directions must terminate: a chain naming only ONE provider can be starved by that
        provider's outage with no other rung to fall to."""
        for role in self.OPUS5_FIRST_ROLES + (self.GUI_ROLE,):
            chain = self.routes[role]["model_chain"]
            providers = {self.doc["models"][m]["provider"] for m in chain}
            self.assertEqual(providers, {"anthropic", "openai"},
                             f"role:{role} chain {chain} is single-provider — it can be starved")

    def test_carve_out_selector_is_exactly_area_gui(self):
        """THE ONE THAT MATTERS MOST. "GUI" informally reads as covering the site surfaces, so the
        likely future mistake is widening the sol carve-out back over `site*`. The carve-out is
        `area:gui` and nothing else (maintainer: "Let's just go with area:gui work").

        MUTANT: add "area:site" (or any site* label) to triage's GUI_SURFACE_LABELS => RED.
        """
        triage_src = (REPO_ROOT / "scripts" / "triage.py").read_text(encoding="utf-8")
        m = re.search(r"(?m)^GUI_SURFACE_LABELS = \(([^)]*)\)", triage_src)
        self.assertIsNotNone(m, "GUI_SURFACE_LABELS not found or reshaped")
        labels = [x.strip().strip("\"'") for x in m.group(1).split(",") if x.strip()]
        self.assertEqual(labels, ["area:gui"],
                         "the sol carve-out selector must be EXACTLY area:gui — no site* label, "
                         "no surface:frontend, no dashboard")

    def test_site_surfaces_are_not_in_the_carve_out(self):
        """The same property from the other side: the generic UI label set (which routes to the
        opus5-first role:site) must not contain area:gui, and the gui set must not contain any
        site* label. MUTANT: move area:gui back into UI_SURFACE_LABELS => RED."""
        triage_src = (REPO_ROOT / "scripts" / "triage.py").read_text(encoding="utf-8")
        ui = re.search(r"(?m)^UI_SURFACE_LABELS = \(([^)]*)\)", triage_src)
        self.assertIsNotNone(ui, "UI_SURFACE_LABELS not found or reshaped")
        ui_labels = [x.strip().strip("\"'") for x in ui.group(1).split(",") if x.strip()]
        self.assertNotIn("area:gui", ui_labels,
                         "area:gui must derive role:gui, not role:site — otherwise the carve-out "
                         "is inexpressible and GUI silently takes the opus5-first default")
        self.assertIn("area:site", ui_labels, "role:site must still cover area:site")


class TestSettingsHooks(unittest.TestCase):
    """Surface 2 — .claude/settings.json agent hooks. The bare-alias regression lived HERE."""

    @classmethod
    def setUpClass(cls):
        cls.settings = json.loads(
            (REPO_ROOT / ".claude" / "settings.json").read_text(encoding="utf-8"))

    def _hook_models(self):
        found = []

        def walk(node, path):
            if isinstance(node, dict):
                if node.get("type") == "agent" and "model" in node:
                    found.append((path, node["model"]))
                for k, v in node.items():
                    walk(v, f"{path}.{k}")
            elif isinstance(node, list):
                for i, v in enumerate(node):
                    walk(v, f"{path}[{i}]")

        walk(self.settings, "settings")
        return found

    def test_at_least_one_agent_hook_is_scanned(self):
        """Anti-vacuity: if the settings shape changes so no hook is found, every assertion below
        passes over an EMPTY list. Fail instead."""
        self.assertTrue(self._hook_models(),
                        "no `type: agent` hook with a `model` found — the scan below is vacuous")

    def test_no_hook_uses_a_deprecated_model(self):
        """MUTANT: set the perf-reviewer arm gate back to `"model": "opus"` => RED."""
        for path, model in self._hook_models():
            self.assertNotIn(model, DEPRECATED_ALIASES, f"{path}: retired alias {model!r}")
            self.assertNotIn(model, DEPRECATED_PROVIDER_MODELS,
                             f"{path}: retired provider id {model!r}")

    def test_no_hook_uses_a_bare_alias(self):
        """MUTANT: swap any full id for its bare alias => RED. `opus` silently meant claude-opus-4-8
        for days after Opus 5 shipped; an alias is not a pin."""
        for path, model in self._hook_models():
            self.assertNotIn(
                model, BARE_ALIASES,
                f"{path}: {model!r} is a bare alias whose target can move — pin the full model id")


class TestWorkflowJsDispatch(unittest.TestCase):
    """Surface 3 — .claude/workflows/*.js `model:` dispatch values."""

    @classmethod
    def setUpClass(cls):
        cls.files = sorted((REPO_ROOT / ".claude" / "workflows").glob("*.js"))

    def _model_literals(self):
        """Every `model: '<literal>'` string in the workflow JS. `model: null` (the
        attribution-only TIER rows) is intentionally not a string and so not collected."""
        pat = re.compile(r"""\bmodel\s*:\s*(['"])([^'"]+)\1""")
        for f in self.files:
            for m in pat.finditer(f.read_text(encoding="utf-8")):
                yield f.name, m.group(2)

    def test_workflow_files_are_present(self):
        self.assertTrue(self.files, "no .claude/workflows/*.js found — scan would be vacuous")

    def test_at_least_one_model_literal_is_scanned(self):
        self.assertTrue(list(self._model_literals()),
                        "no `model:` literal found — the scan below is vacuous")

    def test_no_dispatchable_deprecated_model(self):
        """MUTANT: restore `model: 'claude-fable-5'` on the fable-5 TIER row => RED."""
        for fname, model in self._model_literals():
            self.assertNotIn(model, DEPRECATED_PROVIDER_MODELS,
                             f"{fname}: dispatches retired model {model!r}")
            self.assertNotIn(model, DEPRECATED_ALIASES,
                             f"{fname}: dispatches retired alias {model!r}")

    def test_top_tier_dispatch_uses_the_full_id(self):
        """The `fable`/`opus` TIER keys are retained as stable dispatch TOKENS, but both must
        resolve to the full claude-opus-5 id — never to the bare alias."""
        drain = (REPO_ROOT / ".claude" / "workflows" / "fable-architect-drain.js").read_text(
            encoding="utf-8")
        for key in ("fable", "opus"):
            m = re.search(rf"^\s*{key}:\s*\{{\s*model:\s*'([^']+)'", drain, re.M)
            self.assertIsNotNone(m, f"TIER row {key!r} not found or reshaped")
            self.assertEqual(m.group(1), OPUS5_ID,
                             f"TIER row {key!r} must dispatch the full {OPUS5_ID} id")

    def test_attribution_only_rows_are_not_dispatchable(self):
        """The downgrade rows keep their marker/trailer (attribution is not routing) but must
        carry no dispatchable model. MUTANT: give them a model id back => RED."""
        drain = (REPO_ROOT / ".claude" / "workflows" / "fable-architect-drain.js").read_text(
            encoding="utf-8")
        for key in ("fable-5", "opus-4-8"):
            m = re.search(rf"^\s*'{re.escape(key)}':\s*\{{\s*model:\s*([^,]+),", drain, re.M)
            self.assertIsNotNone(m, f"TIER row {key!r} not found or reshaped")
            self.assertEqual(m.group(1).strip(), "null",
                             f"TIER row {key!r} must not be dispatchable")

    def test_dispatch_model_fails_closed_on_a_non_dispatchable_tier(self):
        """Returning `undefined` for a null-model row would pass NO --model flag, i.e. silently
        serve the session default. The helper must THROW. MUTANT: delete the throw => RED."""
        drain = (REPO_ROOT / ".claude" / "workflows" / "fable-architect-drain.js").read_text(
            encoding="utf-8")
        body = drain[drain.index("const dispatchModel"):]
        body = body[:body.index("\n}\n") + 3]
        self.assertIn("throw new Error", body,
                      "dispatchModel must refuse a non-dispatchable tier, not return undefined")


class TestAgentFrontmatter(unittest.TestCase):
    """Surface 4 — .claude/agents/*.md `model:` frontmatter."""

    @classmethod
    def setUpClass(cls):
        cls.agents = sorted((REPO_ROOT / ".claude" / "agents").glob("*.md"))

    def _models(self):
        for f in self.agents:
            text = f.read_text(encoding="utf-8")
            if not text.startswith("---"):
                continue
            fm = text.split("---", 2)[1]
            m = re.search(r"(?m)^model:\s*(\S+)\s*$", fm)
            if m:
                yield f.name, m.group(1)

    def test_agents_are_present(self):
        self.assertTrue(list(self._models()), "no agent frontmatter models found — vacuous scan")

    def test_no_agent_targets_a_deprecated_model(self):
        """MUTANT: set any agent to `model: opus` or `model: claude-fable-5` => RED."""
        for name, model in self._models():
            self.assertNotIn(model, DEPRECATED_ALIASES, f"{name}: retired alias {model!r}")
            self.assertNotIn(model, DEPRECATED_PROVIDER_MODELS,
                             f"{name}: retired provider id {model!r}")


class TestWorkflowWiring(unittest.TestCase):
    """THE YAML SEAM. A guard no workflow RUNS is not a guard.

    Substring assertions over the workflow text do NOT catch `if: false` — the text still contains
    the call site while the step never executes. So this parses the YAML STRUCTURALLY and asserts
    on the resolved job/step objects.
    """

    @classmethod
    def setUpClass(cls):
        cls.raw = WORKFLOW.read_text(encoding="utf-8")
        cls.doc = yaml.safe_load(cls.raw)
        cls.job = cls.doc["jobs"]["validate"]
        cls.steps = cls.job["steps"]

    def _run_steps(self):
        return [s for s in self.steps if "run" in s]

    def test_job_exists_and_has_no_skip_condition(self):
        """MUTANT: add `if: false` (or any `if:`) to the validate job => RED."""
        self.assertNotIn("if", self.job,
                         "a job-level `if:` can skip the whole routing gate")

    def test_no_gating_step_has_a_skip_condition(self):
        """MUTANT: add `if: false` to the self-test step => RED. This is the assertion a substring
        or count(...) check over the workflow text cannot make."""
        for step in self._run_steps():
            self.assertNotIn(
                "if", step,
                f"step {step.get('name')!r} carries an `if:` — it can silently stop running")

    def test_this_suite_is_actually_invoked(self):
        """MUTANT: delete the invocation line, or the whole step => RED."""
        runs = "\n".join(s["run"] for s in self._run_steps())
        self.assertIn("python3 scripts/tests/test_no_deprecated_models.py", runs,
                      "this suite is never RUN — its assertions are dead in CI")

    def test_invocation_is_not_neutralised_by_a_trailing_true(self):
        """MUTANT: append `|| true` to the invocation => RED. Exit-zero swallowing has bitten this
        repo three times in one day; a guard whose failure is discarded is not a guard."""
        runs = "\n".join(s["run"] for s in self._run_steps())
        for line in runs.splitlines():
            if "test_no_deprecated_models.py" in line:
                self.assertNotRegex(
                    line.strip(), r"(\|\|\s*true|;\s*true|\|\|\s*:)\s*$",
                    "the guard's exit status is discarded")

    def test_run_block_uses_pipefail(self):
        """Without `set -euo pipefail` a failing early command does not fail the step."""
        for step in self._run_steps():
            if "test_no_deprecated_models.py" in step["run"]:
                self.assertIn("set -euo pipefail", step["run"])

    def test_this_file_is_a_path_trigger_on_both_triggers(self):
        """MUTANT: drop the paths entry => this suite stops running on its own PRs => RED.
        Scoped to the trigger section: the filename also appears in the run: block, so a
        whole-file search would pass for the wrong reason."""
        trigger_section = self.raw[:self.raw.index("permissions:")]
        self.assertEqual(
            trigger_section.count('"scripts/tests/test_no_deprecated_models.py"'), 2,
            "must be a path trigger on BOTH pull_request and push")

    def test_routing_surfaces_are_path_triggers(self):
        """The surfaces this suite guards must re-run it when THEY change — otherwise a
        deprecated model can be reintroduced by a PR that never runs this gate."""
        trigger_section = self.raw[:self.raw.index("permissions:")]
        for surface in ("orchestration/routing.toml", ".claude/settings.json",
                        ".claude/workflows/fable-architect-drain.js"):
            self.assertEqual(
                trigger_section.count(f'"{surface}"'), 2,
                f"{surface} must re-run this gate on BOTH pull_request and push")

    def test_merge_group_trigger_present(self):
        self.assertIn("merge_group", self.doc.get(True, self.doc.get("on", {})),
                      "the merge queue must evaluate this gate")

    def test_gate_is_not_declared_advisory(self):
        """MUTANT: declare routing-self-tests/validate advisory => ci-summary stops gating => RED."""
        registry = json.loads(
            (REPO_ROOT / ".github" / "advisory-registry.json").read_text(encoding="utf-8"))
        declared = {(e.get("workflow"), e.get("job_id"))
                    for e in registry.get("jobs", {}).values()}
        self.assertNotIn(("routing-self-tests.yml", "validate"), declared,
                         "declaring this job advisory stops ci-summary gating on it")


if __name__ == "__main__":
    sys.exit(not unittest.main(verbosity=2, exit=False).result.wasSuccessful())
