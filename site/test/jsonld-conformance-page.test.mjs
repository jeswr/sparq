// [GPT-5.6] sq-ztdez — mutation witness for all six ratios and the honesty caveat.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);

test("JSON-LD scoreboard renders every measured floor and the qualifying caveat", async () => {
  const [page, data] = await Promise.all([
    readFile(new URL("src/app/assurance/jsonld-conformance/page.tsx", root), "utf8"),
    import("../src/data/jsonld-conformance.ts"),
  ]);

  assert.deepEqual(
    data.jsonLdConformanceLanes.map(({ id, floor, total }) => [id, floor, total]),
    [
      ["toRdf", 413, 467],
      ["expand", 276, 385],
      ["flatten", 53, 58],
      ["compact", 228, 246],
      ["frame", 92, 92],
      ["fromRdf", 52, 53],
    ],
  );
  assert.match(page, /Measured floors, not conformance claims/);
  assert.match(page, /A lane below its total is not claimed conformant/);
  assert.match(page, /jsonLdConformanceLanes\.map/);
});
