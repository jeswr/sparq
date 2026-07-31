import assert from "node:assert/strict";
import test from "node:test";

import { simulateThreshold, workspaceIntegers } from "./mpc-sim.ts";

test("reveals the threshold verdict while retaining a share matrix", () => {
  const result = simulateThreshold(
    [{ name: "A", value: 40 }, { name: "B", value: 70 }],
    100,
    () => 0.25,
  );
  assert.equal(result.verdict, true);
  assert.equal(result.shares.length, 2);
  assert.equal(result.localSums.length, 2);
});

test("extracts unique integer literals from a live N-Quads snapshot", () => {
  const datatype = "http://www.w3.org/2001/XMLSchema#integer";
  const snapshot = [
    `<urn:a> <urn:value> "24"^^<${datatype}> .`,
    `<urn:b> <urn:value> "30"^^<${datatype}> .`,
    `<urn:c> <urn:value> "24"^^<${datatype}> .`,
  ].join("\n");
  assert.deepEqual(workspaceIntegers(snapshot), [24, 30]);
});
