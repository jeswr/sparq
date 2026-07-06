// [FABLE-5] sq-vw3ax.9 — unit tests for the wasm-free-main-bundle guard's pure analyser
// (scripts/check-bundle.mjs `analyzeBundle`). These feed SYNTHETIC manifests — a healthy build, a
// chunk that LEAKED into a route entry, a demo that regressed to a STATIC import, and a broken
// config — and assert the analyser flags exactly the right failure. They need no `next build`, so
// they run in the fast `npm run test:unit` lane (site-e2e); the real `npm run check:bundle` /
// `postbuild` run the SAME analyser against the actual .next/out artifacts.
import { test } from "node:test";
import assert from "node:assert/strict";

import {
  analyzeBundle,
  HEAVY_MODULE_SUFFIXES,
  GUARDED_ROUTES,
} from "../scripts/check-bundle.mjs";

// A minimal healthy fixture: every heavy module is a distinct dynamic split point with its own
// chunk, NONE of which appear in either guarded route's first-load set.
function healthyInputs() {
  const loadable = {};
  let n = 0;
  for (const suffix of HEAVY_MODULE_SUFFIXES) {
    loadable[`src ${suffix}`] = { files: [`static/chunks/heavy-${n++}.js`] };
  }
  const first = ["static/chunks/webpack.js", "static/chunks/main-app.js"];
  const app = {
    pages: {
      "/page": [...first, "static/chunks/app/page.js"],
      "/capabilities/page": [...first, "static/chunks/app/capabilities/page.js"],
    },
  };
  const routes = { basePath: "/sparq" };
  const images = { images: { unoptimized: true } };
  const htmlByLabel = {
    "/": '<script src="/sparq/_next/static/chunks/webpack.js"></script>',
    "/capabilities": '<script src="/sparq/_next/static/chunks/app/capabilities/page.js"></script>',
  };
  return { loadable, app, routes, images, htmlByLabel };
}

test("healthy build passes with zero errors", () => {
  const { errors, summary } = analyzeBundle(healthyInputs());
  assert.deepEqual(errors, []);
  assert.equal(summary.unoptimized, true);
  assert.equal(summary.basePath, "/sparq");
  assert.equal(summary.heavyReport.length, HEAVY_MODULE_SUFFIXES.length);
});

test("ISOLATION: a heavy chunk leaked into a route first-load is flagged", () => {
  const inputs = healthyInputs();
  // Force the zk-car-hire chunk into the /capabilities first-load set (the regression we guard).
  const zkChunk = inputs.loadable["src -> @/components/zk-car-hire"].files[0];
  inputs.app.pages["/capabilities/page"].push(zkChunk);
  const { errors } = analyzeBundle(inputs);
  assert.equal(errors.length, 1);
  assert.match(errors[0], /^ISOLATION: \/capabilities/);
  assert.match(errors[0], new RegExp(zkChunk.replace(/[.]/g, "\\.")));
});

test("PRESENCE: a demo switched to a static import (no split point) is flagged", () => {
  const inputs = healthyInputs();
  // Simulate zk-car-hire becoming a static import: its dynamic entry disappears from the manifest.
  delete inputs.loadable["src -> @/components/zk-car-hire"];
  const { errors } = analyzeBundle(inputs);
  assert.equal(errors.length, 1);
  assert.match(errors[0], /^PRESENCE:/);
  assert.match(errors[0], /@\/components\/zk-car-hire/);
});

test("PRESENCE: the ZK prover deps must stay dynamic", () => {
  const inputs = healthyInputs();
  delete inputs.loadable["src -> @aztec/bb.js"];
  delete inputs.loadable["src -> @noir-lang/noir_js"];
  const { errors } = analyzeBundle(inputs);
  assert.equal(errors.length, 2);
  assert.ok(errors.every((e) => e.startsWith("PRESENCE:")));
});

test("CONFIG: images.unoptimized=false is flagged", () => {
  const inputs = healthyInputs();
  inputs.images.images.unoptimized = false;
  const { errors } = analyzeBundle(inputs);
  assert.equal(errors.length, 1);
  assert.match(errors[0], /images\.unoptimized/);
});

test("CONFIG: adapts to the root-relative (basePath '') build mode", () => {
  const inputs = healthyInputs();
  inputs.routes.basePath = "";
  inputs.htmlByLabel = {
    "/": '<script src="/_next/static/chunks/webpack.js"></script>',
    "/capabilities": '<script src="/_next/static/chunks/app/capabilities/page.js"></script>',
  };
  const { errors, summary } = analyzeBundle(inputs);
  assert.deepEqual(errors, []);
  assert.equal(summary.basePath, "");
});

test("CONFIG: HTML missing the resolved basePath prefix is flagged", () => {
  const inputs = healthyInputs();
  // basePath is /sparq but the HTML emits root-relative assets → prefixing broke.
  inputs.htmlByLabel["/"] = '<script src="/_next/static/chunks/webpack.js"></script>';
  const { errors } = analyzeBundle(inputs);
  assert.ok(errors.some((e) => /basePath prefixing is broken/.test(e)));
});

test("CONFIG: an image-optimiser reference in exported HTML is flagged", () => {
  const inputs = healthyInputs();
  inputs.htmlByLabel["/capabilities"] += '<img src="/sparq/_next/image?url=%2Fx.png&w=64&q=75">';
  const { errors } = analyzeBundle(inputs);
  assert.ok(errors.some((e) => /image optimiser/.test(e)));
});

test("GUARDED_ROUTES covers exactly the Home REPL and /capabilities", () => {
  assert.deepEqual(
    GUARDED_ROUTES.map((r) => r.label).sort(),
    ["/", "/capabilities"],
  );
});
