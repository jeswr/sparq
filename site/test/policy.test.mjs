// [OPUS-4.8] sq-vw3ax.14 — unit tests for the ODRL usage-control walkthrough data.
//
// The policy surface is captured-output tier: sparq-policy is an opt-in native crate
// (evaluate() is pure Rust, no I/O) the wasm bundles never carry, and the static site
// has no backend, so src/lib/policy.ts REPLAYS evaluate() for each (policy, request)
// pair. This suite sends those exact pairs through the real native evaluator and
// compares its complete Decision with the captured object verbatim.
//
// The native comparison proves evaluator equivalence. The remaining tests also pin the
// pasted decisions' INTERNAL CONSISTENCY and make failures more diagnostic:
//   * the fail-closed invariant — a PERMIT has exactly a matched rule AND no unmet
//     caveat; a DENY always carries an explanation (never a silent no);
//   * every matched rule id actually appears in that scenario's ODRL Turtle (no
//     dangling/typo'd rule reference), and a prohibition-override deny names its
//     prohibition in the caveat;
//   * every verdict cites an odrl_eval.rs provenance handle AND every caveat matches one
//     of eval.rs's documented output shapes (so a fabricated free-text caveat fails);
//   * every constraint caveat's (rule, left, operator, right) anchors to the scenario's
//     OWN Turtle constraint — eval.rs prints the POLICY's operands, never the request's
//     supplied value — so an operand substitution fails, not just a malformed shape;
//   * requestLine renders a faithful, compilable-shape Request builder line.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

import { SCENARIOS, ODRL_NS, requestLine } from "../src/lib/policy.ts";

const valueKind = (left) =>
  left === "dateTime"
    ? "datetime"
    : left === "purpose"
      ? "iri"
      : left === "count"
        ? "number"
        : "string";

test("captured decisions equal the real Rust evaluator output verbatim", () => {
  // [GPT-5] #3557 — pass the imported objects themselves to the native harness: no
  // second policy/request fixture can drift independently from the walkthrough.
  const fixtureDir = mkdtempSync(join(tmpdir(), "sparq-policy-site-"));
  try {
    for (const scenario of SCENARIOS) {
      const turtlePath = join(fixtureDir, `${scenario.id}.ttl`);
      writeFileSync(turtlePath, scenario.turtle);
      for (const variant of scenario.variants) {
        const args = [
          "run",
          "--quiet",
          "--manifest-path",
          "test/fixtures/policy-evaluator/Cargo.toml",
          "--",
          turtlePath,
          expandIri(variant.action),
          variant.target ?? "-",
          variant.party ?? "-",
        ];
        for (const context of variant.context ?? []) {
          args.push("context", valueKind(context.left), expandIri(context.left), context.value);
        }
        for (const duty of variant.discharged ?? []) args.push("discharge", expandIri(duty));

        const result = spawnSync("cargo", args, { cwd: process.cwd(), encoding: "utf8" });
        assert.equal(
          result.status,
          0,
          `${scenario.id}/${variant.id}: Rust fixture failed\n${result.stderr}`,
        );
        let decision;
        try {
          decision = JSON.parse(result.stdout);
        } catch (error) {
          assert.fail(
            `${scenario.id}/${variant.id}: Rust fixture emitted invalid JSON: ${error.message}\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
          );
        }
        assert.deepEqual(
          decision,
          variant.decision,
          `${scenario.id}/${variant.id}: captured decision drifted from sparq_policy::evaluate`,
        );
      }
    }
  } finally {
    rmSync(fixtureDir, { recursive: true, force: true });
  }
});

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

// The `matched`/`unmet` strings are reproduced from eval.rs's output FORMAT (see
// crates/sparq-policy/src/eval.rs): a prohibition carve-out, a constraint/logical caveat,
// an undischarged-duty caveat, a fail-closed "no permission" default, or a rule
// action/target/assignee non-match. A caveat that matches none of these is a fabricated
// free-text string, not evaluator output — so the diagnostic gate rejects it.
const UNMET_SHAPES = [
  /^prohibition .+ matches the request$/,
  /^rule .+ constraint \(.+\) unsatisfied$/,
  /^rule .+ logical constraint .+ \(.+\) unsatisfied$/,
  /^permission .+ requires undischarged duty .+$/,
  /^no permission matches the request$/,
  /^rule .+ action .+ != requested .+$/,
  /^rule .+ target .+ != requested .+$/,
  /^rule .+ assignee .+ != requester .+$/,
];

test("every verdict cites provenance and every caveat is an eval.rs output shape", () => {
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      assert.ok(
        v.test.startsWith("odrl_eval.rs::"),
        `${s.id}/${v.id}: provenance points at odrl_eval.rs (${v.test})`,
      );
      for (const u of v.decision.unmet) {
        assert.ok(
          UNMET_SHAPES.some((re) => re.test(u)),
          `${s.id}/${v.id}: caveat must match an eval.rs output shape (${u})`,
        );
      }
    }
  }
});

// eval.rs formats a constraint caveat from the POLICY's own parsed constraint —
// `rule {id} constraint ({c.left} {c.operator:?} {c.right}) unsatisfied` — so every
// operand in the caveat must be the policy's, NEVER the request's supplied value (a
// purpose-mismatch deny names the policy's bound, not the offending purpose). This
// test parses each constraint caveat and anchors all four parts to the scenario's own
// Turtle, so an operand substitution (the round-2 fabrication failure mode) fails the
// gate — shape alone cannot catch it.
const CONSTRAINT_CAVEAT = /^rule (\S+) constraint \((\S+) (\S+) (.+)\) unsatisfied$/;

test("every constraint caveat's operands anchor to the policy's own constraint", () => {
  let anchored = 0;
  for (const s of SCENARIOS) {
    // Collapse the Turtle's alignment whitespace so `predicate object` pairs match.
    const flat = s.turtle.replace(/\s+/g, " ");
    for (const v of s.variants) {
      for (const u of v.decision.unmet) {
        const m = CONSTRAINT_CAVEAT.exec(u);
        if (!m) continue; // prohibition/duty/logical caveats are anchored elsewhere
        anchored++;
        const [, ruleId, left, op, right] = m;
        assert.ok(flat.includes(ruleId), `${s.id}/${v.id}: caveat rule ${ruleId} in the policy`);
        // left operand: eval.rs prints the full IRI; the Turtle uses the odrl: prefix.
        const leftTtl = left.startsWith(ODRL_NS) ? `odrl:${left.slice(ODRL_NS.length)}` : left;
        assert.ok(
          flat.includes(`odrl:leftOperand ${leftTtl}`),
          `${s.id}/${v.id}: caveat left ${left} is the policy's leftOperand`,
        );
        // operator: the Debug variant name maps to the odrl term by lowercasing the
        // first letter (Eq→eq, Lteq→lteq, IsPartOf→isPartOf).
        assert.ok(
          flat.includes(`odrl:operator odrl:${op[0].toLowerCase()}${op.slice(1)}`),
          `${s.id}/${v.id}: caveat operator ${op} is the policy's operator`,
        );
        // right operand: eval.rs prints the POLICY's rightOperand (Value's Display —
        // an IRI as <iri>, a string/dateTime as its quoted lexical form), which must
        // appear verbatim as the Turtle's rightOperand. A request-supplied value
        // (e.g. <urn:purpose/marketing>) is not in the policy, so it fails here.
        assert.ok(
          flat.includes(`odrl:rightOperand ${right}`),
          `${s.id}/${v.id}: caveat right ${right} is the policy's rightOperand, not the request's value`,
        );
      }
    }
  }
  assert.ok(anchored >= 4, `the anchor gate actually ran (${anchored} constraint caveats)`);
});

// Expand a display token to the full IRI the Rust API uses (mirrors expandIri in
// src/lib/policy.ts): `odrl:` prefix and bare local names resolve against ODRL_NS.
const expandIri = (a) =>
  a.startsWith("odrl:") ? `${ODRL_NS}${a.slice(5)}` : a.includes(":") ? a : `${ODRL_NS}${a}`;

test("requestLine renders a faithful, compilable-shape Request builder line", () => {
  for (const s of SCENARIOS) {
    for (const v of s.variants) {
      const line = requestLine(v);
      assert.ok(line.startsWith("Request::new("), `${s.id}/${v.id}: starts with the builder`);
      // Action, target, party expand to full, QUOTED IRIs / values.
      assert.ok(
        line.includes(`Request::new("${expandIri(v.action)}")`),
        `${s.id}/${v.id}: action IRI quoted+expanded`,
      );
      if (v.target) assert.ok(line.includes(`.on("${v.target}")`), `${s.id}/${v.id}: target`);
      if (v.party) assert.ok(line.includes(`.by("${v.party}")`), `${s.id}/${v.id}: party`);
      for (const c of v.context ?? []) {
        // Left operand is a full quoted IRI; the value is wrapped in a Value constructor —
        // never a bare unquoted identifier or bare literal (the old, non-Rust form).
        assert.ok(
          line.includes(`.with("${expandIri(c.left)}", Value::`),
          `${s.id}/${v.id}: context ${c.left} is a quoted IRI + Value ctor`,
        );
        assert.ok(
          !line.includes(`.with(${c.left},`),
          `${s.id}/${v.id}: left operand is not a bare identifier`,
        );
      }
      for (const d of v.discharged ?? []) {
        // The duty action is a full, quoted IRI — never bare `odrl:anonymize` (invalid Rust).
        assert.ok(line.includes(`.discharge("${expandIri(d)}")`), `${s.id}/${v.id}: duty ${d}`);
      }
    }
  }
});

test("requestLine pins the exact line for a representative context and duty variant", () => {
  const inWindow = SCENARIOS.find((s) => s.id === "time-window").variants.find(
    (v) => v.id === "in-window",
  );
  assert.equal(
    requestLine(inWindow),
    'Request::new("http://www.w3.org/ns/odrl/2/read").on("urn:asset/x")' +
      '.with("http://www.w3.org/ns/odrl/2/dateTime", Value::DateTime("2026-06-16T09:00:00Z".into()))',
  );

  const discharged = SCENARIOS.find((s) => s.id === "duty").variants.find(
    (v) => v.id === "discharged",
  );
  assert.equal(
    requestLine(discharged),
    'Request::new("http://www.w3.org/ns/odrl/2/read").on("urn:asset/x")' +
      '.discharge("http://www.w3.org/ns/odrl/2/anonymize")',
  );
});
