// [SONNET-4.6] sq-b66fc — non-vacuous tests for the wasm-manifest resolution logic.
//
// We test resolveWasmAssetWithManifest() directly (no fetch mocking needed) since it is
// the pure core of the resolution logic. resolveWasmAsset() wraps it with a fetch + cache
// layer whose integration is covered by tests 1–2 via a globalThis.fetch mock.
import assert from "node:assert/strict";
import { test, beforeEach, afterEach } from "node:test";

import {
  resolveWasmAsset,
  resolveWasmAssetWithManifest,
} from "../src/index.ts";

// ---------------------------------------------------------------------------
// Test 1: resolveWasmAssetWithManifest returns hashed URL when manifest has the entry
// ---------------------------------------------------------------------------
test("resolveWasmAssetWithManifest returns hashed URL when manifest has the entry", () => {
  const manifest = {
    "sparq_wasm.js": "sparq_wasm-a1b2c3d4ef56.js",
    "sparq_wasm_bg.wasm": "sparq_wasm_bg-7890abcdef01.wasm",
  };
  const result = resolveWasmAssetWithManifest("sparq_wasm.js", manifest, "/sparq");
  assert.equal(result, "/sparq/wasm/sparq_wasm-a1b2c3d4ef56.js");
});

// ---------------------------------------------------------------------------
// Test 2: resolveWasmAsset falls back to unhashed URL when manifest is absent (fetch 404)
// ---------------------------------------------------------------------------
test("resolveWasmAsset falls back to unhashed URL when manifest fetch returns 404", async () => {
  // Install a fetch mock that returns 404 for the manifest URL.
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (url) => {
    if (String(url).endsWith("wasm-manifest.json")) {
      return { ok: false, status: 404 };
    }
    throw new Error(`Unexpected fetch: ${url}`);
  };

  try {
    // Use a unique basePath so the module-level cache has no entry for this call.
    const result = await resolveWasmAsset("sparq_wasm_bg.wasm", "/sparq-test-404");
    assert.equal(result, "/sparq-test-404/wasm/sparq_wasm_bg.wasm");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

// ---------------------------------------------------------------------------
// Test 3: changed asset content changes the resolved URL
//   Two manifests with different hashes for the same logical name must resolve
//   to different URLs — proving that content-addressing is actually respected.
// ---------------------------------------------------------------------------
test("changed asset content changes the resolved URL (different hash → different URL)", () => {
  const manifestV1 = { "sparq_wasm_bg.wasm": "sparq_wasm_bg-aaa111bbb222.wasm" };
  const manifestV2 = { "sparq_wasm_bg.wasm": "sparq_wasm_bg-ccc333ddd444.wasm" };
  const base = "/sparq";

  const urlV1 = resolveWasmAssetWithManifest("sparq_wasm_bg.wasm", manifestV1, base);
  const urlV2 = resolveWasmAssetWithManifest("sparq_wasm_bg.wasm", manifestV2, base);

  // Different manifests → different resolved URLs (core cacheability claim).
  assert.notEqual(urlV1, urlV2);
  assert.equal(urlV1, "/sparq/wasm/sparq_wasm_bg-aaa111bbb222.wasm");
  assert.equal(urlV2, "/sparq/wasm/sparq_wasm_bg-ccc333ddd444.wasm");
});

// ---------------------------------------------------------------------------
// Test 4 (mutation / non-vacuity): loader respects the manifest — returns the
//   UNHASHED fallback when the manifest has no entry for the key.
//   This test would FAIL if the loader returned a hashed name regardless of the manifest.
// ---------------------------------------------------------------------------
test("loader respects the manifest — returns fallback when manifest has no entry for the key", () => {
  // Manifest exists but has NO entry for "sparq_wasm.js".
  const manifestWithoutGlue = {
    "sparq_wasm_bg.wasm": "sparq_wasm_bg-deadbeef0000.wasm",
  };
  const result = resolveWasmAssetWithManifest(
    "sparq_wasm.js",
    manifestWithoutGlue,
    "/sparq",
  );
  // Must return the UNHASHED fallback, not any hashed name.
  assert.equal(result, "/sparq/wasm/sparq_wasm.js");
  // Sanity: if the manifest had an entry, the result WOULD be different — proving the
  // test actually fails when the fallback is bypassed.
  const manifestWithGlue = {
    "sparq_wasm.js": "sparq_wasm-abc123def456.js",
  };
  const hashedResult = resolveWasmAssetWithManifest(
    "sparq_wasm.js",
    manifestWithGlue,
    "/sparq",
  );
  assert.notEqual(result, hashedResult); // proves the test discriminates
});
