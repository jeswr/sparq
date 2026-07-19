// [OPUS-4.8] sq-vw3ax.14 — unit tests for the ODRL usage-control walkthrough data.
//
// The policy surface is captured-output tier: sparq-policy is an opt-in native crate
// (evaluate() is pure Rust, no I/O) the wasm bundles never carry, and the static site
// has no backend, so src/lib/policy.ts REPLAYS the real evaluate() decision for each
// (policy, request) pair — every verdict pinned by a named crate test in
// crates/sparq-policy/tests/odrl_eval.rs.
//
// These tests pin the pasted decisions' INTERNAL CONSISTENCY so a silent hand-edit (the
// fabrication failure mode) flips the unit gate red:
//   * the fail-closed invariant — a PERMIT has exactly a matched rule AND no unmet
//     caveat; a DENY always carries an explanation (never a silent no);
//   * every matched rule id actually appears in that scenario's ODRL Turtle (no
//     dangling/typo'd rule reference), and a prohibition-override deny names its
//     prohibition in the caveat;
//   * every verdict carries a real odrl_eval.rs provenance handle;
//   * requestLine expands the compact Request builder faithfully.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import { SCENARIOS, ODRL_NS, requestLine } from "../src/lib/policy.ts";

test("scenarios are well-formed and uniquely identified", () => {
  assert.ok(SCENARIOS.length >= 3, "at least a few scenarios");
  const ids = new Set();
  for (const s of SCENARIOS) {
    assert.ok(s.id && !ids.has(s.id), `unique scenario id: ${s.id}`);
    ids.add(s.id);
    assert.ok(s.title && s.summary && s.feature, `${s.id} has title/summary/feature`);
    assert.ok(s.turtle.includes("odrl:"), `${s.id} turtle uses the odrl prefix`);
    assert.ok(s.variants.length >= 2, `${s.id} offers at least two requests`);

    const vids = new Set();
    for (const v of s.variants) {
      assert.ok(v.id && !vids.has(v.id), `${s.id}: unique variant id ${v.id}`);
      vids.add(v.id);
    }
  }
});

test("the fail-closed invariant holds for every decision", () => {
  let permits = 0;
  let denies = 0;
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      const d = v.decision;
      if (d.allow) {
        permits++;
        // A PERMIT is justified by exactly its granting rule, with NO unmet caveat.
        assert.equal(d.matched.length, 1, `${s.id}/${v.id}: one granting rule`);
        assert.equal(d.unmet.length, 0, `${s.id}/${v.id}: a clean ALLOW has no caveat`);
      } else {
        denies++;
        // A DENY is NEVER silent — it always explains why it did not grant.
        assert.ok(d.unmet.length >= 1, `${s.id}/${v.id}: a DENY must explain itself`);
      }
    }
  }
  assert.ok(permits >= 1 && denies >= 1, "covers both PERMIT and DENY outcomes");
});

test("every matched rule id is grounded in the scenario's Turtle", () => {
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      for (const m of v.decision.matched) {
        assert.ok(
          s.turtle.includes(m),
          `${s.id}/${v.id}: matched ${m} must appear in the policy`,
        );
        // A matched rule on a DENY is a deny-overrides prohibition — the caveat must name it.
        if (!v.decision.allow) {
          assert.ok(
            v.decision.unmet.some((u) => u.includes(m)),
            `${s.id}/${v.id}: prohibition ${m} named in the caveat`,
          );
        }
      }
    }
  }
});

test("every verdict carries real crate-test provenance", () => {
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      assert.ok(
        v.test.startsWith("odrl_eval.rs::"),
        `${s.id}/${v.id}: provenance points at odrl_eval.rs (${v.test})`,
      );
    }
  }
});

test("requestLine expands the compact Request builder faithfully", () => {
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      const line = requestLine(v);
      assert.ok(line.startsWith("Request::new("), `${s.id}/${v.id}: starts with the builder`);
      // The action expands to a full IRI (bare local names get the ODRL namespace).
      const expanded = v.action.includes(":") ? v.action : `${ODRL_NS}${v.action}`;
      assert.ok(line.includes(expanded), `${s.id}/${v.id}: action IRI expanded`);
      if (v.target) assert.ok(line.includes(`.on("${v.target}")`), `${s.id}/${v.id}: target`);
      if (v.party) assert.ok(line.includes(`.by("${v.party}")`), `${s.id}/${v.id}: party`);
      for (const d of v.discharged ?? []) {
        assert.ok(line.includes(`.discharge(${d})`), `${s.id}/${v.id}: duty ${d}`);
      }
    }
  }
});
