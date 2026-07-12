#!/usr/bin/env node
// [FABLE-5] sq-3ul2n.1 — Node baseline for the cross-engine wasm harness.
//
// Runs the SAME shipped web-target artifact + the SAME shared workload as the
// browser pages, directly under Node — the non-browser reference point that
// separates "browsers are slow at X" from "the wasm is slow at X".
//
// Init decomposition mirrors the browser `mode=main` page: read bytes
// (Node's analogue of fetch — no HTTP so the number is a floor, recorded as
// such) / WebAssembly.compile / instantiate. `init_streaming_total` cannot be
// measured here (Node fetch will not stream file: URLs → no
// instantiateStreaming path) and is skipped WITH NOTICE, never estimated.
//
// Usage: node node-baseline.mjs --out <file.json> [--quick]
// All timings ADVISORY / NON-CANONICAL.

import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { runWorkload } from "./workload.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..", "..");
const WASM_PATH = path.join(REPO, "js", "wasm", "sparq_wasm_bg.wasm");

function arg(name) {
  const i = process.argv.indexOf(name);
  return i === -1 ? undefined : process.argv[i + 1];
}
const OUT = arg("--out");
const QUICK = process.argv.includes("--quick");
if (!OUT) {
  console.error("usage: node node-baseline.mjs --out <file.json> [--quick]");
  process.exit(2);
}

const now = () => performance.now();
const rows = [];
const skipped = [
  {
    phase: "init_streaming_total",
    reason: "Node cannot fetch file: URLs, so the instantiateStreaming shipped path is browser-only; not estimated",
  },
];

let t = now();
const bytes = await readFile(WASM_PATH);
rows.push({ phase: "fetch_wasm", kind: "total", ms: now() - t, bytes: bytes.byteLength, iters: 1, note: "disk read, not HTTP — a floor for the browsers' fetch phase" });

t = now();
const module = await WebAssembly.compile(bytes);
rows.push({ phase: "compile_wasm", kind: "total", ms: now() - t, iters: 1 });

// Both imports resolve to the same glue module file → one shared instance,
// exactly as in the browser page.
const glue = await import(path.join(REPO, "js", "wasm", "sparq_wasm.js"));
const { SparqStore } = await import(path.join(REPO, "js", "dist", "index.js"));
t = now();
await glue.default({ module_or_path: module });
rows.push({ phase: "instantiate_wasm", kind: "total", ms: now() - t, iters: 1 });

const wl = runWorkload({
  Store: glue.Store,
  SparqStore,
  quick: QUICK,
  log: (m) => console.error(`[node-baseline] ${m}`),
});
rows.push(...wl.rows);
skipped.push(...wl.skipped);

await writeFile(OUT, JSON.stringify({ ok: true, rows, skipped, mode: "node" }, null, 2));
console.error(`[node-baseline] wrote ${rows.length} rows to ${OUT}`);
