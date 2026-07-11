// [FABLE-5] sq-vw3ax.13 — unit tests for the federation capability walkthrough.
// The federation surface is captured-output tier (the three federation crates are
// opt-in native code the wasm bundles never carry, and the static site has no
// backend), so it replays REAL sparq-fedplan planner output from the committed
// capture harness (crates/sparq-fedplan/examples/capture_surface_federation.rs, run
// with `--features fedplan`) over the declared fixture reproduced in the module.
//
// These tests pin the pasted capture's INTERNAL CONSISTENCY against the fixture and
// against the planner's documented cost model, so a silent hand-edit (the sq-rnwc
// fabrication failure mode) flips the unit gate red:
//   * pruning is justified by the declared property partitions (HiBISCuS bound-
//     predicate rule) and partitions the source set exactly;
//   * every leaf estimate equals the declared partition's triple count (CostFed,
//     var-predicate-var shape);
//   * every Bind step cost satisfies L·request_cost + out, every Hash/Streaming step
//     cost satisfies L + R, and totals are the sums — the documented cost model;
//   * the star-join root cardinality equals the mined characteristic set's
//     subjects × Π avg_multiplicity (the CS estimate, not independence).
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  FED_QUERY,
  PATTERNS,
  SCENARIOS,
  SELECTION,
  SOURCES,
  patternPredicate,
  pruneReason,
  scenarioById,
} from "../src/lib/federation.ts";

// The real request_cost knob behind each captured scenario (mirrors the harness).
const REQUEST_COST = {
  default: 1.0,
  "pricier-round-trips": 2.0,
  "bounded-memory": 2.0,
};

test("fixture shape: 3 sources, 3 patterns, all patterns var-predicate-var", () => {
  assert.equal(SOURCES.length, 3);
  assert.equal(PATTERNS.length, 3);
  for (const p of PATTERNS) {
    const [s, pred, o] = p.split(/\s+/);
    assert.ok(s.startsWith("?"), `${p}: subject must be a variable`);
    assert.ok(!pred.startsWith("?"), `${p}: predicate must be bound`);
    assert.ok(o.startsWith("?"), `${p}: object must be a variable`);
  }
  // The display query block carries every pattern's predicate.
  for (let i = 0; i < PATTERNS.length; i++) {
    assert.ok(FED_QUERY.includes(patternPredicate(i)), `query shows ${patternPredicate(i)}`);
  }
});

test("selection covers every pattern; retained ∪ pruned partitions the sources", () => {
  assert.equal(SELECTION.length, PATTERNS.length);
  for (const sel of SELECTION) {
    const retained = sel.retained.map((c) => c.source);
    const all = [...retained, ...sel.pruned].sort();
    assert.deepEqual(all, [0, 1, 2], `pattern ${sel.pattern}: sources partitioned`);
    assert.equal(new Set(all).size, SOURCES.length, "no source both retained and pruned");
  }
});

test("HiBISCuS pruning is justified by the declared partitions (recall-safe)", () => {
  for (const sel of SELECTION) {
    const pred = patternPredicate(sel.pattern);
    for (const c of sel.retained) {
      assert.ok(
        SOURCES[c.source].partitions.some((p) => p.predicate === pred),
        `pattern ${sel.pattern}: retained source ${c.source} declares ${pred}`,
      );
    }
    for (const s of sel.pruned) {
      assert.ok(
        !SOURCES[s].partitions.some((p) => p.predicate === pred),
        `pattern ${sel.pattern}: pruned source ${s} must provably lack ${pred}`,
      );
      // The rendered reason names the pruned source and the predicate.
      const reason = pruneReason(sel.pattern, s);
      assert.ok(reason.includes(SOURCES[s].label) && reason.includes(pred));
    }
  }
});

test("CostFed leaf estimates equal the declared partition triple counts", () => {
  // Every fixture pattern is ?s <p> ?o, so the per-source estimate is exactly the
  // source's declared void:triples for that predicate — and totals are the sums.
  for (const sel of SELECTION) {
    const pred = patternPredicate(sel.pattern);
    let total = 0;
    for (const c of sel.retained) {
      const part = SOURCES[c.source].partitions.find((p) => p.predicate === pred);
      assert.equal(
        c.estimatedCardinality,
        part.triples,
        `pattern ${sel.pattern} source ${c.source}: estimate = declared triples`,
      );
      total += c.estimatedCardinality;
    }
    assert.equal(sel.total, total, `pattern ${sel.pattern}: total is the union sum`);
  }
});

test("every scenario shares the same greedy order, starting at the most selective leaf", () => {
  const minPattern = SELECTION.reduce((a, b) => (b.total < a.total ? b : a)).pattern;
  for (const sc of SCENARIOS) {
    assert.deepEqual([...sc.joinOrder].sort(), [0, 1, 2], `${sc.id}: order is a permutation`);
    assert.equal(sc.joinOrder[0], minPattern, `${sc.id}: starts with the smallest leaf`);
    assert.deepEqual(
      sc.steps.map((st) => st.right),
      sc.joinOrder.slice(1),
      `${sc.id}: steps follow the join order`,
    );
  }
});

test("captured step costs satisfy the documented cost model; totals are the sums", () => {
  for (const sc of SCENARIOS) {
    const req = REQUEST_COST[sc.id];
    assert.ok(req !== undefined, `${sc.id}: known request_cost`);
    let left = SELECTION.find((s) => s.pattern === sc.joinOrder[0]).total;
    let total = 0;
    for (const st of sc.steps) {
      const right = SELECTION.find((s) => s.pattern === st.right).total;
      if (st.algo === "Bind") {
        // Bind: one bound request per left row + the matched rows (L·req + out).
        assert.equal(st.cost, left * req + st.cardinality, `${sc.id} p${st.right}: bind cost`);
      } else {
        // Hash & Streaming: one full retrieval of each side (L + R).
        assert.equal(st.cost, left + right, `${sc.id} p${st.right}: hash-class cost`);
      }
      total += st.cost;
      left = st.cardinality;
    }
    assert.equal(sc.totalCost, total, `${sc.id}: total cost is the sum of steps`);
    assert.equal(
      sc.rootCardinality,
      sc.steps[sc.steps.length - 1].cardinality,
      `${sc.id}: root cardinality is the last step's output`,
    );
  }
});

test("the star-join estimate comes from the characteristic set, not independence", () => {
  // The final join extends the ?author star {foaf:name} with foaf:knows. The mined CS
  // says 800 subjects share {foaf:name, foaf:knows} at multiplicities 1.0 × 10.0 —
  // so the captured output cardinality must be 800 · 1 · 10 = 8000.
  const cs = SOURCES[0].charSets.find((c) => c.predicates.includes("foaf:knows"));
  const expected = cs.subjects * cs.avgMultiplicity.reduce((a, b) => a * b, 1);
  for (const sc of SCENARIOS) {
    const last = sc.steps[sc.steps.length - 1];
    assert.equal(last.cardinality, expected, `${sc.id}: CS star estimate`);
  }
});

test("the three scenarios demonstrate all three join algorithms, honestly labelled", () => {
  assert.deepEqual(
    scenarioById("default").steps.map((s) => s.algo),
    ["Bind", "Bind"],
  );
  assert.deepEqual(
    scenarioById("pricier-round-trips").steps.map((s) => s.algo),
    ["Bind", "Hash"],
  );
  assert.deepEqual(
    scenarioById("bounded-memory").steps.map((s) => s.algo),
    ["Bind", "Streaming"],
  );
  // Each non-default scenario says which real PlanOptions knob it turned.
  assert.ok(scenarioById("pricier-round-trips").options.includes("request_cost: 2.0"));
  assert.ok(scenarioById("bounded-memory").options.includes("stream_threshold: 10_000.0"));
  assert.equal(scenarioById("nope"), undefined);
});
