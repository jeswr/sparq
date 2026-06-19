// [OPUS-4.8] sq-9vw5 — unit tests for the site's base-path resolver, the single switch that
// lets the SAME static export serve BOTH GitHub Pages (`/sparq` prefix) and the Tauri 2
// webview (root-relative `''`). `basePath()` / `withBasePath()` read `NEXT_PUBLIC_BASE_PATH`
// at call time, so each case sets/clears the env var and re-imports nothing (the read is
// function-scoped, not module-scoped). Mirrors the build-time switch in `next.config.ts` and
// the runtime wasm-loader switch in `@sparq/client`. Run via `npm run test:unit`.
import { test, afterEach } from "node:test";
import assert from "node:assert/strict";

import { basePath, withBasePath } from "../src/lib/base-path.ts";

const KEY = "NEXT_PUBLIC_BASE_PATH";

// Restore a pristine env after every case so order can't leak.
const originalHadKey = Object.prototype.hasOwnProperty.call(process.env, KEY);
const originalValue = process.env[KEY];
afterEach(() => {
  if (originalHadKey) process.env[KEY] = originalValue;
  else delete process.env[KEY];
});

test("unset env -> the GitHub Pages default '/sparq' (historical, no caller change)", () => {
  delete process.env[KEY];
  assert.equal(basePath(), "/sparq");
  assert.equal(withBasePath("/logo.svg"), "/sparq/logo.svg");
});

test("empty string -> root-relative '' (the Tauri webview export)", () => {
  process.env[KEY] = "";
  assert.equal(basePath(), "");
  // Root-relative: the asset path is returned unchanged, with no double slash.
  assert.equal(withBasePath("/logo.svg"), "/logo.svg");
  assert.equal(withBasePath("/papers/x.pdf"), "/papers/x.pdf");
});

test("an explicit '/'-leading prefix is honoured (a custom deploy root)", () => {
  process.env[KEY] = "/demo";
  assert.equal(basePath(), "/demo");
  assert.equal(withBasePath("/coi-serviceworker.js"), "/demo/coi-serviceworker.js");
});

test("a trailing slash on the prefix is stripped (no double slash on compose)", () => {
  process.env[KEY] = "/demo/";
  assert.equal(basePath(), "/demo");
  assert.equal(withBasePath("/logo.svg"), "/demo/logo.svg");
});

test("a malformed (non-'/'-leading) value falls back to the Pages default", () => {
  process.env[KEY] = "sparq"; // missing leading slash
  assert.equal(basePath(), "/sparq");
  assert.equal(withBasePath("/logo.svg"), "/sparq/logo.svg");
});
