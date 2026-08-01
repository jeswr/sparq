#!/usr/bin/env python3
# [OPUS-4.8] Hermetic unit/dry-run tests for the flow-on engine (bead sq-ncvq.2,
# epic sq-ncvq). Authored by Opus 4.8 (Fable unavailable; flag for re-review when
# Fable returns).
#
# Fully hermetic: imports scripts/flow-on.py and exercises rule evaluation +
# idempotency-key generation against fixture diffs. NO live `gh` calls — we only
# drive `evaluate()` and `main(--dry-run)`, neither of which touches the network.
#
# Run:  python3 scripts/tests/test_flow_on.py
# (stdlib only; no pytest required.)

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
FLOW_ON = REPO_ROOT / "scripts" / "flow-on.py"
RULES = REPO_ROOT / "scripts" / "flow-on-rules.toml"


def _load_module():
    spec = importlib.util.spec_from_file_location("flow_on", FLOW_ON)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    # Register before exec so dataclasses can resolve forward annotations
    # (cls.__module__ lookup in _is_type) for the module-level @dataclass types.
    sys.modules["flow_on"] = mod
    spec.loader.exec_module(mod)
    return mod


flow_on = _load_module()


@contextlib.contextmanager
def checkout_with(files: dict[str, str] | None = None):
    """[OPUS-5] (#2547) Run the body with flow-on reading shared artifacts from a
    fixture checkout instead of this repo.

    `unless_paths` suppression now ATTRIBUTES a shared artifact to a crate by
    reading its content from the merged checkout. Fixturing that content keeps
    these tests hermetic — asserting against the live `site/src/data/surfaces.ts`
    would make them pass or fail on which crates happen to be advertised today.
    Crate publish status is unaffected: it still reads the real manifests."""
    original = flow_on.CHECKOUT_ROOT
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        for rel, text in (files or {}).items():
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        flow_on.CHECKOUT_ROOT = root
        try:
            yield root
        finally:
            flow_on.CHECKOUT_ROOT = original


class RuleLoadingTest(unittest.TestCase):
    def test_rules_load_and_are_well_formed(self):
        rules = flow_on.load_rules(RULES)
        self.assertTrue(rules, "expected at least one rule")
        ids = {r.id for r in rules}
        # The taxonomy rules the design requires must all be present.
        for required in (
            "changed-public-feature-docs",
            "new-bench-dashboard-row",
            "competitor-feature-gather",
            # [OPUS-4.8] ZK circuit gate-count follow-on (a PR-description
            # deliverable, present in the rules file) — enforce the contract.
            "new-zk-circuit-gatecount",
            # [OPUS-5] (#2547) The un-gateable remainder a new PUBLIC crate owes,
            # split one rule per artifact so a satisfied artifact suppresses only
            # its own follow-on.
            "new-crate-site-advert",
            "new-crate-guide-docs",
            "new-crate-test-methods",
        ):
            self.assertIn(required, ids)
        # [OPUS-4.8] (bead sq-l0a0) The new-crate bench/SKILL follow-on rule was
        # REMOVED — merge gate G1 (scripts/gate-new-crate.py) already enforces a
        # registered bench + README + SKILL in the SAME PR, so a post-merge
        # follow-on for them was pure redundant noise (minted #245/#246/#264/#265).
        self.assertNotIn("new-crate-bench-and-skill", ids)
        # Every rule has a trigger + at least one create template.
        for r in rules:
            self.assertTrue(
                r.when_paths or r.when_new_paths or r.when_label or r.when_title,
                f"{r.id} has no trigger",
            )
            self.assertTrue(r.creates, f"{r.id} has no create templates")


class NewCrateTest(unittest.TestCase):
    """[OPUS-4.8] (bead sq-l0a0) A new-crate PR must yield NO bench/SKILL
    follow-on: merge gate G1 already guarantees a registered bench + README +
    SKILL in the SAME PR, so the old reactive follow-on was pure redundant noise
    (minted #245/#246/#264/#265). The rule was removed."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def _eval(self, changed, added, labels=None, title="add sparq-foo crate"):
        return flow_on.evaluate(
            self.rules, 42, title, changed, added, labels or []
        )

    def test_new_crate_with_bench_and_skill_yields_no_redundant_follow_on(self):
        # A new crate that ships (as G1 requires) its bench + README + SKILL must
        # NOT mint a bench/SKILL follow-on — G1 owns that, in-PR.
        added = [
            "crates/sparq-foo/Cargo.toml",
            "crates/sparq-foo/src/lib.rs",
            "crates/sparq-foo/README.md",
        ]
        changed = added + ["bench/benchmarks.toml", "skills/sparq-foo/SKILL.md"]
        fos = self._eval(changed=changed, added=added)
        keys = {fo.dedup_key for fo in fos}
        self.assertNotIn("new-crate-bench-sparq-foo", keys)
        self.assertNotIn("new-crate-skill-sparq-foo", keys)
        self.assertNotIn("new-crate-bench-and-skill", {fo.rule_id for fo in fos})

    def test_new_crate_alone_yields_no_redundant_follow_on(self):
        # Even a bare new crate (which G1 would BLOCK at merge) mints no flow-on
        # follow-on — the redundant rule is gone.
        added = ["crates/sparq-foo/Cargo.toml", "crates/sparq-foo/src/lib.rs"]
        fos = self._eval(changed=added, added=added)
        self.assertNotIn("new-crate-bench-and-skill", {fo.rule_id for fo in fos})

    def test_modifying_existing_crate_file_does_not_fire_new_crate_rule(self):
        # Cargo.toml is CHANGED but not ADDED -> when_new_paths must not match.
        changed = ["crates/sparq-core/Cargo.toml", "crates/sparq-core/src/store/mod.rs"]
        added: list[str] = []
        fos = self._eval(changed=changed, added=added)
        self.assertNotIn("new-crate-bench-and-skill", {fo.rule_id for fo in fos})


class NewPublicCrateFollowOnTest(unittest.TestCase):
    """[OPUS-5] (#2547) The un-gateable half of new-crate completeness.

    G1 blocks the merge until a new crate ships a registered bench + README +
    SKILL. The rest of what a new PUBLIC surface owes — a website card, an mdBook
    guide chapter, and a decision about which test methods apply — is produced
    out-of-band and is a judgment call, so it is minted as follow-on issues here.

    The public/stub distinction is read from the REAL manifests on disk via merge
    gate G1's own predicate, so these tests use two real crates rather than mocks:
    a publishable one and a `publish = false` stub."""

    PUBLIC_CRATE = "sparq-core"
    OTHER_PUBLIC_CRATE = "sparq-engine"
    STUB_CRATE = "sparq-canon"
    RULE_IDS = {"new-crate-site-advert", "new-crate-guide-docs", "new-crate-test-methods"}

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)
        # Guard the fixtures: if either crate's publish status ever flips, fail
        # HERE with a clear message instead of silently voiding the assertions.
        for crate in (self.PUBLIC_CRATE, self.OTHER_PUBLIC_CRATE):
            self.assertTrue(
                flow_on.crate_is_public(crate),
                f"fixture crate {crate} is no longer publishable",
            )
        self.assertFalse(
            flow_on.crate_is_public(self.STUB_CRATE),
            f"fixture crate {self.STUB_CRATE} is no longer a publish=false stub",
        )

    def _eval(self, changed, added, title="feat: new crate"):
        fos = self._eval_list(changed, added, title)
        return {fo.rule_id: fo for fo in fos}

    def _eval_list(self, changed, added, title="feat: new crate"):
        """Every new-crate follow-on, un-collapsed — a multi-crate PR produces
        more than one follow-on per rule, which the dict view above would lose."""
        fos = flow_on.evaluate(self.rules, 900, title, changed, added, [])
        return [fo for fo in fos if fo.rule_id in self.RULE_IDS]

    def test_new_public_crate_mints_site_guide_and_test_method_follow_ons(self):
        added = [
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
            f"crates/{self.PUBLIC_CRATE}/src/lib.rs",
            f"crates/{self.PUBLIC_CRATE}/README.md",
        ]
        changed = added + ["bench/benchmarks.toml", "skills/sparql/SKILL.md"]
        by_rule = self._eval(changed=changed, added=added)
        self.assertEqual(set(by_rule), self.RULE_IDS)
        self.assertEqual(
            {fo.dedup_key for fo in by_rule.values()},
            {
                f"site-advert-{self.PUBLIC_CRATE}",
                f"guide-doc-{self.PUBLIC_CRATE}",
                f"test-methods-{self.PUBLIC_CRATE}",
            },
        )

    def test_internal_stub_crate_mints_nothing(self):
        # A `publish = false` stub is exempt — the same escape hatch G1 gives it
        # for bench + SKILL. 47 of the workspace's 67 crates are stubs today, so
        # without this the rules would be a noise firehose.
        added = [f"crates/{self.STUB_CRATE}/Cargo.toml"]
        self.assertEqual(self._eval(changed=added, added=added), {})

    def test_site_entry_in_the_same_pr_suppresses_only_the_site_follow_on(self):
        added = [f"crates/{self.PUBLIC_CRATE}/Cargo.toml"]
        changed = added + ["site/src/data/surfaces.ts"]
        # The site record names the crate it advertises (its README link), so the
        # artifact is attributable to this crate and suppresses its follow-on.
        with checkout_with(
            {
                "site/src/data/surfaces.ts": (
                    '{ slug: "core", readme: '
                    f'"https://github.com/jeswr/sparq/blob/main/crates/{self.PUBLIC_CRATE}/README.md" }}'
                )
            }
        ):
            by_rule = self._eval(changed=changed, added=added)
        self.assertNotIn("new-crate-site-advert", by_rule)
        self.assertIn("new-crate-guide-docs", by_rule)
        self.assertIn("new-crate-test-methods", by_rule)

    def test_guide_page_in_the_same_pr_suppresses_only_the_guide_follow_on(self):
        added = [f"crates/{self.PUBLIC_CRATE}/Cargo.toml"]
        changed = added + ["book/src/SUMMARY.md", "book/src/getting-started/new.md"]
        with checkout_with(
            {
                "book/src/SUMMARY.md": "- [Core](./getting-started/new.md)",
                "book/src/getting-started/new.md": (
                    f"{{{{#include ../../../crates/{self.PUBLIC_CRATE}/README.md}}}}"
                ),
            }
        ):
            by_rule = self._eval(changed=changed, added=added)
        self.assertNotIn("new-crate-guide-docs", by_rule)
        self.assertIn("new-crate-site-advert", by_rule)
        self.assertIn("new-crate-test-methods", by_rule)

    def test_an_artifact_that_never_names_the_crate_does_not_suppress(self):
        # The conservative half of attribution: touching the shared site file for
        # some OTHER reason establishes no coverage for this crate, so its
        # follow-on survives. Silently dropping it is the failure mode that a bare
        # "the path matched" suppression had.
        added = [f"crates/{self.PUBLIC_CRATE}/Cargo.toml"]
        changed = added + ["site/src/data/surfaces.ts"]
        with checkout_with(
            {"site/src/data/surfaces.ts": '{ slug: "sparql", title: "SPARQL" }'}
        ):
            by_rule = self._eval(changed=changed, added=added)
        self.assertIn("new-crate-site-advert", by_rule)
        self.assertEqual(
            by_rule["new-crate-site-advert"].dedup_key,
            f"site-advert-{self.PUBLIC_CRATE}",
        )

    def test_an_unreadable_artifact_does_not_suppress(self):
        # Coverage must be POSITIVELY established: a file that is not in the
        # checkout (or is unreadable) proves nothing, so the follow-on survives.
        added = [f"crates/{self.PUBLIC_CRATE}/Cargo.toml"]
        changed = added + ["site/src/data/surfaces.ts"]
        with checkout_with():  # empty fixture checkout — the artifact is absent
            by_rule = self._eval(changed=changed, added=added)
        self.assertIn("new-crate-site-advert", by_rule)

    def test_a_longer_crate_name_does_not_claim_coverage(self):
        # Token match, not substring: an entry for `sparq-core-ng` must NOT count
        # as coverage for `sparq-core` (a bare `in` test would say it does).
        self.assertFalse(
            flow_on._mentions_crate(
                f"crates/{self.PUBLIC_CRATE}-ng/README.md", self.PUBLIC_CRATE
            )
        )
        self.assertTrue(
            flow_on._mentions_crate(
                f"crates/{self.PUBLIC_CRATE}/README.md", self.PUBLIC_CRATE
            )
        )

    def test_modifying_an_existing_crate_mints_nothing(self):
        # The manifest must be ADDED: an edit to an existing crate is not a new
        # surface and must not re-mint the whole set on every touch.
        changed = [
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
            f"crates/{self.PUBLIC_CRATE}/src/lib.rs",
        ]
        self.assertEqual(self._eval(changed=changed, added=[]), {})

    def test_test_method_follow_on_is_routed_to_the_new_crate_area(self):
        # The per-crate `area:{crate}` label is placeholder-expanded, so the
        # follow-on routes to the crate its work lands in (an unexpanded
        # `area:{crate}` literal would be a garbage label on a real issue).
        added = [f"crates/{self.PUBLIC_CRATE}/Cargo.toml"]
        fo = self._eval(changed=added, added=added)["new-crate-test-methods"]
        self.assertIn(f"area:{self.PUBLIC_CRATE}", fo.labels)
        self.assertNotIn("area:{crate}", fo.labels)

    def test_placeholder_names_the_added_crate_not_another_touched_one(self):
        # A PR that adds a file to an EXISTING crate as well as the new crate's
        # manifest must name the crate whose MANIFEST was added — picking the
        # first `crates/<x>/` path in the diff would name the wrong one here.
        added = [
            "crates/sparq-engine/src/exec/new_join.rs",
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
        ]
        changed = list(added)
        by_rule = self._eval(changed=changed, added=added)
        for fo in by_rule.values():
            self.assertIn(self.PUBLIC_CRATE, fo.dedup_key)
            self.assertNotIn("sparq-engine", fo.dedup_key)


class MultipleNewPublicCratesTest(unittest.TestCase):
    """[OPUS-5] (#2547) A PR may add MORE THAN ONE public crate.

    Each of them owes its own website card, guide chapter and test-method plan, so
    the `when_new_crate_public` rules are evaluated once per added public crate
    (matching gate G1's `added_crates`, which likewise returns every new crate).
    Evaluating them once with a single global `{crate}` dropped everything the
    second crate needed."""

    PUBLIC_CRATE = NewPublicCrateFollowOnTest.PUBLIC_CRATE
    OTHER_PUBLIC_CRATE = NewPublicCrateFollowOnTest.OTHER_PUBLIC_CRATE
    STUB_CRATE = NewPublicCrateFollowOnTest.STUB_CRATE
    RULE_IDS = NewPublicCrateFollowOnTest.RULE_IDS

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def _eval_list(self, changed, added, title="feat: new crates"):
        fos = flow_on.evaluate(self.rules, 900, title, changed, added, [])
        return [fo for fo in fos if fo.rule_id in self.RULE_IDS]

    def _both(self, artifact=()):
        added = [
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
            f"crates/{self.OTHER_PUBLIC_CRATE}/Cargo.toml",
        ]
        return added, added + list(artifact)

    def test_each_new_public_crate_gets_its_own_full_follow_on_set(self):
        added, changed = self._both()
        fos = self._eval_list(changed=changed, added=added)
        self.assertEqual(
            {(fo.rule_id, fo.dedup_key) for fo in fos},
            {
                ("new-crate-site-advert", f"site-advert-{self.PUBLIC_CRATE}"),
                ("new-crate-guide-docs", f"guide-doc-{self.PUBLIC_CRATE}"),
                ("new-crate-test-methods", f"test-methods-{self.PUBLIC_CRATE}"),
                ("new-crate-site-advert", f"site-advert-{self.OTHER_PUBLIC_CRATE}"),
                ("new-crate-guide-docs", f"guide-doc-{self.OTHER_PUBLIC_CRATE}"),
                ("new-crate-test-methods", f"test-methods-{self.OTHER_PUBLIC_CRATE}"),
            },
        )
        # Distinct dedup keys — a collision would make the run-level de-dup drop
        # the second crate's issue exactly as the single-{crate} bug did.
        self.assertEqual(len({fo.dedup_key for fo in fos}), 6)
        # And each follow-on names ITS crate in the title/body and routes there.
        for fo in fos:
            crate = (
                self.PUBLIC_CRATE
                if self.PUBLIC_CRATE in fo.dedup_key
                else self.OTHER_PUBLIC_CRATE
            )
            self.assertIn(crate, fo.title)
            self.assertIn(f"crates/{crate}", fo.body)
            if fo.rule_id == "new-crate-test-methods":
                self.assertIn(f"area:{crate}", fo.labels)

    def test_a_stub_alongside_a_public_crate_mints_only_the_public_set(self):
        added = [
            f"crates/{self.STUB_CRATE}/Cargo.toml",
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
        ]
        fos = self._eval_list(changed=added, added=added)
        self.assertEqual(len(fos), 3)
        for fo in fos:
            self.assertIn(self.PUBLIC_CRATE, fo.dedup_key)

    def test_a_shared_artifact_suppresses_only_the_crate_it_covers(self):
        # `site/src/data/surfaces.ts` is a SHARED file: a PR adding two crates
        # touches it once, so "the path matched" cannot mean both crates are
        # advertised. Attribution reads the merged checkout — the record here
        # names only PUBLIC_CRATE, so ONLY its site follow-on is suppressed and
        # OTHER_PUBLIC_CRATE keeps its own (with its own dedup key). The rules
        # whose artifact is absent entirely still mint for BOTH.
        added, changed = self._both(["site/src/data/surfaces.ts"])
        with checkout_with(
            {
                "site/src/data/surfaces.ts": (
                    f'{{ slug: "core", readme: "crates/{self.PUBLIC_CRATE}/README.md" }}'
                )
            }
        ):
            fos = self._eval_list(changed=changed, added=added)
        self.assertEqual(
            {fo.dedup_key for fo in fos if fo.rule_id == "new-crate-site-advert"},
            {f"site-advert-{self.OTHER_PUBLIC_CRATE}"},
        )
        for rule_id in ("new-crate-guide-docs", "new-crate-test-methods"):
            self.assertEqual(
                {fo.dedup_key for fo in fos if fo.rule_id == rule_id},
                {
                    f"{'guide-doc' if rule_id.endswith('docs') else 'test-methods'}-{c}"
                    for c in (self.PUBLIC_CRATE, self.OTHER_PUBLIC_CRATE)
                },
            )

    def test_a_guide_page_for_one_crate_leaves_the_other_guide_follow_on(self):
        # Same attribution for `book/src/**`: a guide page that `{{#include}}`s
        # one crate's README covers that crate only. Round-1's PR-wide reading
        # dropped the second crate's chapter with no surviving issue to close.
        added, changed = self._both(["book/src/SUMMARY.md", "book/src/core.md"])
        with checkout_with(
            {
                "book/src/SUMMARY.md": "- [Core](./core.md)",
                "book/src/core.md": (
                    f"{{{{#include ../../crates/{self.PUBLIC_CRATE}/README.md}}}}"
                ),
            }
        ):
            fos = self._eval_list(changed=changed, added=added)
        self.assertEqual(
            {fo.dedup_key for fo in fos if fo.rule_id == "new-crate-guide-docs"},
            {f"guide-doc-{self.OTHER_PUBLIC_CRATE}"},
        )

    def test_a_crate_named_artifact_suppresses_only_that_crate(self):
        # When a rule's `unless_paths` glob carries `{crate}` the artifact CAN be
        # attributed, so suppression is per crate. No shipped rule has a
        # crate-named artifact path today; this pins the mechanism that lets one
        # be written (and that the shipped placeholder-free globs opt out of).
        rules_toml = """
[[rule]]
id = "per-crate-artifact"
when_new_paths = ["crates/*/Cargo.toml"]
when_new_crate_public = true
unless_paths = ["crates/{crate}/GUIDE.md"]

[[rule.create]]
dedup_key = "per-crate-{crate}"
title = "guide for {crate}"
body = "guide for crates/{crate}"
labels = ["area:{crate}"]
"""
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "rules.toml"
            path.write_text(rules_toml)
            rules = flow_on.load_rules(path)
        added = [
            f"crates/{self.PUBLIC_CRATE}/Cargo.toml",
            f"crates/{self.OTHER_PUBLIC_CRATE}/Cargo.toml",
        ]
        changed = added + [f"crates/{self.PUBLIC_CRATE}/GUIDE.md"]
        fos = flow_on.evaluate(rules, 900, "feat: two crates", changed, added, [])
        self.assertEqual(
            [fo.dedup_key for fo in fos], [f"per-crate-{self.OTHER_PUBLIC_CRATE}"]
        )


class ChangedSurfaceTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_cli_pub_change_without_skill_fires_docs_rule(self):
        # [OPUS-4.8] (bead sq-l0a0) A NET public-API change (the path is in
        # pub_changed) without a SKILL.md DOES fire — the genuinely-useful case.
        changed = ["crates/sparq-cli/src/main.rs"]
        fos = flow_on.evaluate(
            self.rules, 7, "add --explain flag", changed, [], [],
            pub_changed={"crates/sparq-cli/src/main.rs"},
        )
        self.assertIn("changed-public-feature-docs", {fo.rule_id for fo in fos})

    def test_cli_comment_only_change_does_not_fire_docs_rule(self):
        # [OPUS-4.8] (bead sq-l0a0) REGRESSION (mis-fired #250): a comment-only /
        # lint-attribute-only edit to a public surface is NOT a net public-API
        # change (the path is absent from pub_changed), so it mints NO skill-sync
        # follow-on — even without a SKILL.md in the diff.
        changed = ["crates/sparq-cli/src/main.rs"]
        fos = flow_on.evaluate(
            self.rules, 250, "add SAFETY comments + #![warn] attrs", changed, [], [],
            pub_changed=set(),
        )
        self.assertNotIn("changed-public-feature-docs", {fo.rule_id for fo in fos})

    def test_cli_change_with_skill_update_suppresses_docs_rule(self):
        # A SKILL.md in the same diff means the docs were synced → suppress, even
        # on a real pub change.
        changed = ["crates/sparq-cli/src/main.rs", "skills/cli/SKILL.md"]
        fos = flow_on.evaluate(
            self.rules, 8, "add --explain flag", changed, [], [],
            pub_changed={"crates/sparq-cli/src/main.rs"},
        )
        self.assertNotIn("changed-public-feature-docs", {fo.rule_id for fo in fos})


class PubApiDiffPlumbingTest(unittest.TestCase):
    """[OPUS-4.8] (bead sq-l0a0) The docs rule reuses gate G2's shared pub-item
    multiset diff (scripts/pub_api_diff.py). Verify the patch→pub_changed plumbing
    distinguishes a genuine new `pub fn` from a comment/attr/relocation diff."""

    def test_pub_changed_from_patch_detects_new_pub_fn(self):
        patch = (
            "diff --git a/crates/sparq-cli/src/main.rs b/crates/sparq-cli/src/main.rs\n"
            "--- a/crates/sparq-cli/src/main.rs\n"
            "+++ b/crates/sparq-cli/src/main.rs\n"
            "@@ -1,0 +1,1 @@\n"
            "+pub fn brand_new_flag() {}\n"
        )
        self.assertEqual(
            flow_on.pub_changed_from_patch(patch),
            {"crates/sparq-cli/src/main.rs"},
        )

    def test_pub_changed_from_patch_ignores_comment_and_attr_only(self):
        patch = (
            "diff --git a/crates/sparq-cli/src/main.rs b/crates/sparq-cli/src/main.rs\n"
            "--- a/crates/sparq-cli/src/main.rs\n"
            "+++ b/crates/sparq-cli/src/main.rs\n"
            "@@ -1,0 +1,2 @@\n"
            "+// SAFETY: this is sound because ...\n"
            "+#![warn(clippy::all)]\n"
        )
        self.assertEqual(flow_on.pub_changed_from_patch(patch), set())

    def test_pub_changed_from_patch_ignores_pure_relocation(self):
        patch = (
            "diff --git a/crates/sparq-cli/src/main.rs b/crates/sparq-cli/src/main.rs\n"
            "--- a/crates/sparq-cli/src/main.rs\n"
            "+++ b/crates/sparq-cli/src/main.rs\n"
            "@@ -10,1 +10,0 @@\n"
            "-pub fn moved() {}\n"
            "@@ -40,0 +50,1 @@\n"
            "+pub fn moved() {}\n"
        )
        self.assertEqual(flow_on.pub_changed_from_patch(patch), set())


class CompetitorLabelTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_engine_change_requires_label(self):
        changed = ["crates/sparq-engine/src/exec/join.rs"]
        # Without the label, the gather rule must NOT fire.
        fos = flow_on.evaluate(self.rules, 9, "faster joins", changed, [], [])
        self.assertNotIn("competitor-feature-gather", {fo.rule_id for fo in fos})
        # With the label, it fires.
        fos = flow_on.evaluate(
            self.rules, 9, "faster joins", changed, [], ["competitor-relevant"]
        )
        self.assertIn("competitor-feature-gather", {fo.rule_id for fo in fos})


class NewBenchTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_new_bench_suite_triggers_dashboard_row(self):
        added = ["bench/watdiv/benchmarks.toml", "bench/watdiv/run.sh"]
        fos = flow_on.evaluate(self.rules, 11, "add watdiv bench", added, added, [])
        dash = [fo for fo in fos if fo.rule_id == "new-bench-dashboard-row"]
        self.assertTrue(dash)
        self.assertEqual(dash[0].dedup_key, "dashboard-row-watdiv")
        self.assertIn("watdiv", dash[0].title)

    def test_unfeatured_agent_token_harnesses_do_not_trigger_dashboard_row(self):
        # [GPT-5.6] These registry entries explicitly set featured=false: their
        # NON-CANONICAL token A/B verdicts never feed the performance dashboard.
        for suite in ("pkg-dogfood", "terse"):
            with self.subTest(suite=suite):
                added = [f"bench/{suite}/run.sh"]
                fos = flow_on.evaluate(
                    self.rules, 11, f"add {suite} research harness", added, added, []
                )
                self.assertNotIn(
                    "new-bench-dashboard-row", {fo.rule_id for fo in fos}
                )

    def test_unregistered_harness_does_not_trigger_dashboard_row(self):
        added = ["bench/token-research-spike/run.sh"]
        fos = flow_on.evaluate(self.rules, 11, "add token spike", added, added, [])
        self.assertNotIn("new-bench-dashboard-row", {fo.rule_id for fo in fos})


class ZkCircuitGatecountTest(unittest.TestCase):
    """[OPUS-4.8] sq-fyzq7 (#1655): the `new-zk-circuit-gatecount` rule must fire
    ONLY for a genuinely-new top-level `zk/<family>/` circuit — never for a new
    member inside the existing `zk/compose/` family (those are already covered by
    the hard `snapshot_covers_every_member` gate) — and must name the circuit from
    `zk/`, never from a `bench/` output dir."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def _zk_follow_ons(self, fos):
        return [fo for fo in fos if fo.rule_id == "new-zk-circuit-gatecount"]

    def test_new_compose_member_does_not_fire(self):
        # REGRESSION (mis-fired #1655): merged #1608 added `zk/compose/<member>/`
        # path_reach members (already baselined + covered by
        # snapshot_covers_every_member) and modified the bench gate-count JSON.
        # `fnmatch`'s `*` crosses `/`, so `zk/*/Nargo.toml` matched the two-level
        # member manifest; `exclude_new_paths = ["zk/compose/**"]` now drops it.
        added = [
            "zk/compose/path_reach_d2_k1_n16/Nargo.toml",
            "zk/compose/path_reach_d2_k1_n16/src/main.nr",
        ]
        changed = added + [
            "bench/zk-compose/gate_counts_latest.json",  # MODIFIED, not added
            "crates/sparq-zk-compose/tests/gate_count_snapshot.json",
        ]
        fos = flow_on.evaluate(
            self.rules, 1608, "feat(zk/compose): path_reach family", changed, added, []
        )
        self.assertEqual(self._zk_follow_ons(fos), [])
        # And no phantom `zk-compose` key leaks from any rule.
        self.assertNotIn("zk-gatecount-zk-compose", {fo.dedup_key for fo in fos})

    def test_new_top_level_family_fires_with_zk_derived_name(self):
        # A genuinely-new top-level family under zk/ DOES fire, and the circuit is
        # named from `zk/` (here `arith`), NOT from a co-changed `bench/` dir.
        added = ["zk/arith/Nargo.toml", "zk/arith/src/main.nr"]
        changed = added + ["bench/zk-compose/gate_counts_latest.json"]
        fos = flow_on.evaluate(
            self.rules, 2000, "feat(zk): arith circuit", changed, added, []
        )
        zk = self._zk_follow_ons(fos)
        self.assertEqual(len(zk), 1)
        self.assertEqual(zk[0].dedup_key, "zk-gatecount-arith")
        self.assertIn("zk/arith", zk[0].body)
        # The bench dir name must NOT contaminate the zk circuit name.
        self.assertNotIn("zk-compose", zk[0].dedup_key)
        self.assertNotIn("zk/zk-compose", zk[0].body)

    def test_new_family_with_member_subdir_fires_once(self):
        # A new WORKSPACE family (root + a member circuit) still yields exactly one
        # follow-on, named for the family root.
        added = [
            "zk/lattice/Nargo.toml",
            "zk/lattice/scan_a/Nargo.toml",
            "zk/lattice/scan_a/src/main.nr",
        ]
        fos = flow_on.evaluate(
            self.rules, 2100, "feat(zk): lattice family", added, added, []
        )
        zk = self._zk_follow_ons(fos)
        self.assertEqual(len(zk), 1)
        self.assertEqual(zk[0].dedup_key, "zk-gatecount-lattice")

    def test_modified_only_zk_family_does_not_fire(self):
        # Editing an EXISTING top-level circuit (changed, not added) must not fire
        # the new-family rule.
        changed = ["zk/arith/src/main.nr"]
        fos = flow_on.evaluate(
            self.rules, 2200, "tweak arith circuit", changed, [], []
        )
        self.assertEqual(self._zk_follow_ons(fos), [])


class RoutingLabelsTest(unittest.TestCase):
    """[FABLE-5] (#2474) GITHUB_TOKEN-created issues fire no `issues: opened`
    event, so triage-issue.yml can never label a flow-on follow-up post-hoc.
    Every minted follow-on must therefore be dispatch-ready AT CREATION: exactly
    one `role:*`, exactly one `priority:*`, and `status:ready` — computed by the
    shared static triage pass (scripts/triage.py) with a default priority:P3."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def _all_follow_ons(self):
        # One evaluate() per rule family so every template in the table is minted.
        fixtures = [
            # changed-public-feature-docs
            dict(changed=["crates/sparq-cli/src/main.rs"], added=[], labels=[],
                 title="add --explain flag",
                 pub_changed={"crates/sparq-cli/src/main.rs"}),
            # new-bench-dashboard-row
            dict(changed=["bench/watdiv/benchmarks.toml"],
                 added=["bench/watdiv/benchmarks.toml"], labels=[], title="bench",
                 pub_changed=None),
            # competitor-feature-gather
            dict(changed=["crates/sparq-engine/src/exec/join.rs"], added=[],
                 labels=["competitor-relevant"], title="joins", pub_changed=None),
            # new-zk-circuit-gatecount
            dict(changed=["zk/arith/Nargo.toml"], added=["zk/arith/Nargo.toml"],
                 labels=[], title="zk arith", pub_changed=None),
            # [OPUS-5] (#2547) the three new-PUBLIC-crate rules (site / guide /
            # test-methods) — sparq-core is a real publishable crate on disk.
            dict(changed=["crates/sparq-core/Cargo.toml"],
                 added=["crates/sparq-core/Cargo.toml"], labels=[],
                 title="feat: new crate", pub_changed=None),
        ]
        out = []
        for fx in fixtures:
            out += flow_on.evaluate(
                self.rules, 99, fx["title"], fx["changed"], fx["added"],
                fx["labels"], fx["pub_changed"],
            )
        return out

    def test_every_follow_on_is_dispatch_ready_at_creation(self):
        fos = self._all_follow_ons()
        self.assertTrue(fos, "fixture should mint at least one follow-on per rule")
        for fo in fos:
            roles = [lb for lb in fo.labels if lb.startswith("role:")]
            prios = [lb for lb in fo.labels if lb.startswith("priority:")]
            self.assertEqual(len(roles), 1, f"{fo.rule_id}: {fo.labels}")
            self.assertEqual(len(prios), 1, f"{fo.rule_id}: {fo.labels}")
            self.assertIn("status:ready", fo.labels, fo.rule_id)
            self.assertNotIn("status:untriaged", fo.labels, fo.rule_id)

    def test_roles_derive_from_rule_kind_labels(self):
        by_rule = {fo.rule_id: fo for fo in self._all_follow_ons()}
        # kind:docs → role:docs; kind:perf → role:perf; the zk security keyword
        # forces role:soundness; default priority:P3 everywhere (no rule sets one).
        self.assertIn("role:docs", by_rule["changed-public-feature-docs"].labels)
        self.assertIn("role:docs", by_rule["new-bench-dashboard-row"].labels)
        self.assertIn("role:perf", by_rule["competitor-feature-gather"].labels)
        self.assertIn("role:soundness", by_rule["new-zk-circuit-gatecount"].labels)
        # [OPUS-5] (#2547) kind:site → role:site (the website card is site work);
        # kind:test → role:impl (wiring a fuzz target / proptest is impl work).
        self.assertIn("role:site", by_rule["new-crate-site-advert"].labels)
        self.assertIn("role:docs", by_rule["new-crate-guide-docs"].labels)
        self.assertIn("role:impl", by_rule["new-crate-test-methods"].labels)
        for fo in by_rule.values():
            self.assertIn("priority:P3", fo.labels, fo.rule_id)

    def test_dry_run_prints_routing_labels(self):
        # The dry-run plan is how a human previews a mint — it must show that the
        # issue will be born routed (labels line includes status:ready).
        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("bench/watdiv/benchmarks.toml\nbench/watdiv/run.sh\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    ["--pr", "42", "--dry-run", "--changed-files", str(changed),
                     "--added-files", str(changed), "--title", "add widgets bench"]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("status:ready", out)
        self.assertIn("priority:P3", out)


class IdempotencyTest(unittest.TestCase):
    """The dedup-key marker is what open_issue_exists() searches for; verify the
    key is stable + embedded so a re-run is a no-op once the issue is open."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_dedup_key_stable_across_runs(self):
        # [OPUS-4.8] (bead sq-l0a0) Repointed off the removed new-crate rule onto
        # the new-bench-dashboard-row follow-on, which still fires and produces a
        # stable, non-empty dedup key.
        added = ["bench/watdiv/benchmarks.toml"]
        a = flow_on.evaluate(self.rules, 42, "t", added, added, [])
        b = flow_on.evaluate(self.rules, 42, "t", added, added, [])
        keys_a = sorted(fo.dedup_key for fo in a)
        self.assertIn("dashboard-row-watdiv", keys_a)
        self.assertEqual(keys_a, sorted(fo.dedup_key for fo in b))

    def test_marker_round_trips(self):
        key = "dashboard-row-widgets"
        body = "some body\n\n" + flow_on.key_marker(key)
        # open_issue_exists greps for exactly this marker substring.
        self.assertIn(flow_on.key_marker(key), body)


class DryRunTest(unittest.TestCase):
    """main(--dry-run) must print the would-create plan and make NO gh calls.

    We provide all inputs hermetically (changed-files/title) so fetch_pr_inputs
    is never reached, and --dry-run skips open_issue_exists/create_issue."""

    def test_dry_run_on_new_bench_suite_fixture(self):
        # [OPUS-4.8] (bead sq-l0a0) Repointed off the removed new-crate rule onto
        # the surviving new-bench-suite → dashboard-row follow-on.
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("bench/watdiv/benchmarks.toml\nbench/watdiv/run.sh\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    [
                        "--pr",
                        "42",
                        "--dry-run",
                        "--changed-files",
                        str(changed),
                        "--added-files",
                        str(changed),
                        "--title",
                        "add watdiv bench suite",
                    ]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("[dry-run] WOULD create", out)
        self.assertIn("dashboard-row-watdiv", out)

    def test_dry_run_on_new_crate_with_g1_artifacts_is_clean(self):
        # [OPUS-4.8] (bead sq-l0a0) A new-crate PR that ships its G1 artifacts
        # (bench + README + SKILL) triggers NO redundant flow-on follow-on.
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text(
                "crates/sparq-foo/Cargo.toml\ncrates/sparq-foo/src/lib.rs\n"
                "crates/sparq-foo/README.md\nbench/benchmarks.toml\n"
                "skills/sparq-foo/SKILL.md\n"
            )
            added = Path(td) / "added.txt"
            added.write_text("crates/sparq-foo/Cargo.toml\ncrates/sparq-foo/src/lib.rs\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    [
                        "--pr",
                        "245",
                        "--dry-run",
                        "--changed-files",
                        str(changed),
                        "--added-files",
                        str(added),
                        "--title",
                        "add sparq-foo crate (with bench+SKILL, as G1 requires)",
                    ]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertNotIn("new-crate-bench", out)
        self.assertNotIn("new-crate-skill", out)

    def test_dry_run_no_match_is_clean(self):
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("README.md\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    ["--pr", "1", "--dry-run", "--changed-files", str(changed), "--title", "docs"]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("no rules triggered", out)

    def test_hermetic_without_added_files_does_not_fire_new_path_rules(self):
        # [OPUS-4.8] Regression: when --added-files is omitted, `added` must
        # default to EMPTY, not to all changed files — otherwise a MODIFIED
        # benchmarks.toml would falsely fire the new-bench (when_new_paths)
        # rule. Here it is changed-but-not-added.
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("bench/widgets/benchmarks.toml\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    [
                        "--pr",
                        "7",
                        "--dry-run",
                        "--changed-files",
                        str(changed),
                        "--title",
                        "tweak widgets bench",
                    ]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertNotIn("dashboard-row-widgets", out)
        self.assertIn("no rules triggered", out)

    def test_hermetic_comment_only_public_surface_is_clean(self):
        # [OPUS-4.8] (bead sq-l0a0) End-to-end via main(): a public-surface src
        # change WITHOUT --pub-diff-file (so pub_changed defaults to empty) mints
        # NO skill-sync follow-on — the #250 mis-fire is fixed at the CLI layer.
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("crates/sparq-cli/src/main.rs\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    ["--pr", "250", "--dry-run", "--changed-files", str(changed),
                     "--title", "add SAFETY comments"]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("no rules triggered", out)

    def test_hermetic_pub_change_fires_skill_sync(self):
        # [OPUS-4.8] (bead sq-l0a0) With --pub-diff-file marking a net pub change,
        # the skill-sync follow-on DOES fire (the useful case is preserved).
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("crates/sparq-cli/src/main.rs\n")
            pub = Path(td) / "pub.txt"
            pub.write_text("crates/sparq-cli/src/main.rs\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    ["--pr", "251", "--dry-run", "--changed-files", str(changed),
                     "--pub-diff-file", str(pub), "--title", "add --explain flag"]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("skill-sync-pr-251", out)


# --------------------------------------------------------------------------- #
# CI guard (bead sq-z0se) — refuse to MINT issues outside CI
# --------------------------------------------------------------------------- #
class CiGuardTest(unittest.TestCase):
    """The issue-MINTING path must refuse to run outside CI; an accidental
    dev-box run of the sibling drift scanner minted 20 spurious issues
    (#397-416). --dry-run is NEVER guarded (it makes zero gh calls)."""

    def _clear_ci_env(self) -> None:
        backup = {k: os.environ.get(k) for k in ("GITHUB_ACTIONS", "CI",
                                                 "FLOW_ON_ALLOW_LOCAL")}

        def restore() -> None:
            for k, v in backup.items():
                if v is None:
                    os.environ.pop(k, None)
                else:
                    os.environ[k] = v

        self.addCleanup(restore)
        for k in backup:
            os.environ.pop(k, None)

    def _firing_inputs(self, td: str) -> list[str]:
        # A net pub-API change to a watched surface fires the skill-sync rule, so
        # `follow_ons` is non-empty and the guard is reached on the write path.
        changed = Path(td) / "changed.txt"
        changed.write_text("crates/sparq-cli/src/main.rs\n")
        pub = Path(td) / "pub.txt"
        pub.write_text("crates/sparq-cli/src/main.rs\n")
        return ["--pr", "999", "--changed-files", str(changed),
                "--pub-diff-file", str(pub), "--title", "add --explain flag"]

    def test_running_in_ci_detects_markers(self):
        self.assertTrue(flow_on.running_in_ci({"GITHUB_ACTIONS": "true"}))
        self.assertTrue(flow_on.running_in_ci({"CI": "1"}))

    def test_running_in_ci_false_when_unset_or_falsey(self):
        self.assertFalse(flow_on.running_in_ci({}))
        self.assertFalse(flow_on.running_in_ci({"CI": ""}))

    def test_require_ci_passes_in_ci(self):
        flow_on.require_ci({"GITHUB_ACTIONS": "true"})  # no raise

    def test_require_ci_passes_with_escape_hatch(self):
        flow_on.require_ci({"FLOW_ON_ALLOW_LOCAL": "1"})  # no raise

    def test_require_ci_refuses_outside_ci(self):
        err = io.StringIO()
        with redirect_stderr(err), self.assertRaises(SystemExit) as cm:
            flow_on.require_ci({})
        self.assertEqual(cm.exception.code, 2)
        self.assertIn("refusing to file GitHub issues outside CI", err.getvalue())

    def test_main_mint_path_refuses_outside_ci_before_any_gh_call(self):
        # A real (non-dry-run) run that triggers a rule must SystemExit on the
        # guard rather than shelling out to `gh` to mint issues.
        self._clear_ci_env()
        with tempfile.TemporaryDirectory() as td:
            buf, err = io.StringIO(), io.StringIO()
            with redirect_stdout(buf), redirect_stderr(err), self.assertRaises(
                SystemExit
            ) as cm:
                flow_on.main(self._firing_inputs(td))  # NO --dry-run
        self.assertEqual(cm.exception.code, 2)
        self.assertIn("refusing to file GitHub issues outside CI", err.getvalue())

    def test_main_dry_run_never_guarded_outside_ci(self):
        # --dry-run must succeed with no CI markers (local previews / tests).
        self._clear_ci_env()
        with tempfile.TemporaryDirectory() as td:
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(self._firing_inputs(td) + ["--dry-run"])
        self.assertEqual(rc, 0)
        self.assertIn("skill-sync-pr-999", buf.getvalue())


if __name__ == "__main__":
    unittest.main(verbosity=2)
