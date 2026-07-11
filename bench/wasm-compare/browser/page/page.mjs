// [FABLE-5] sq-3ul2n.1 — browser side of the cross-engine wasm harness.
//
// Two modes, selected by `?mode=`:
//   streaming — times the SHIPPED instantiation path exactly as
//               js/packages/sparq-client does it: bare `initWasm()` →
//               fetch + WebAssembly.instantiateStreaming with compile
//               overlapped with download. One total number; nothing else runs
//               (a wasm module instantiates once per page, so the decomposed
//               phases need their own page load).
//   main      — DECOMPOSED init (fetch+arrayBuffer / WebAssembly.compile /
//               instantiate) so a regression attributes to download vs
//               compile vs instantiation, then the full shared workload
//               (store load/parse, query shapes cold+warm, serialization out,
//               boundary-marshalling probes) from ../workload.mjs.
//
// Results land on `window.__WASM_COMPARE_RESULT__` (+ `__WASM_COMPARE_DONE__`)
// for run.mjs to collect. All timings ADVISORY / NON-CANONICAL.

import initWasm, { Store } from "/js/wasm/sparq_wasm.js";
// The wrapper resolves its `../wasm/sparq_wasm.js` import to the SAME URL as
// the raw import above → same module instance, so the decomposed init below
// initialises both. Only sync wrapper entry points are used (fromStringSync /
// queryBindings) — awaiting the wrapper's own init() would re-instantiate the
// module and orphan every live Store.
import { SparqStore } from "/js/dist/index.js";
import { runWorkload } from "../workload.mjs";

const WASM_URL = "/js/wasm/sparq_wasm_bg.wasm";
const logEl = document.getElementById("log");
const log = (msg) => {
  logEl.textContent += `\n${msg}`;
  console.log(`[harness] ${msg}`);
};
const now = () => performance.now();

async function main() {
  const params = new URLSearchParams(location.search);
  const mode = params.get("mode") ?? "main";
  const quick = params.get("quick") === "1";
  const rows = [];
  const skipped = [];

  if (mode === "streaming") {
    // Shipped path: bare init → fetch + instantiateStreaming (overlapped).
    const t0 = now();
    await initWasm();
    rows.push({ phase: "init_streaming_total", kind: "total", ms: now() - t0, iters: 1 });
    log(`init_streaming_total ${rows[0].ms.toFixed(1)}ms`);
    return { rows, skipped, mode };
  }

  // Decomposed init: download / compile / instantiate, separately attributable.
  let t = now();
  const resp = await fetch(WASM_URL);
  if (!resp.ok) throw new Error(`fetch ${WASM_URL}: ${resp.status}`);
  const bytes = await resp.arrayBuffer();
  rows.push({ phase: "fetch_wasm", kind: "total", ms: now() - t, bytes: bytes.byteLength, iters: 1 });

  t = now();
  const module = await WebAssembly.compile(bytes);
  rows.push({ phase: "compile_wasm", kind: "total", ms: now() - t, iters: 1 });

  t = now();
  await initWasm({ module_or_path: module });
  rows.push({ phase: "instantiate_wasm", kind: "total", ms: now() - t, iters: 1 });
  log(rows.map((r) => `${r.phase} ${r.ms.toFixed(1)}ms`).join(", "));

  const wl = runWorkload({ Store, SparqStore, quick, log });
  rows.push(...wl.rows);
  skipped.push(...wl.skipped);
  return { rows, skipped, mode };
}

main()
  .then((result) => {
    window.__WASM_COMPARE_RESULT__ = { ok: true, ...result };
    window.__WASM_COMPARE_DONE__ = true;
    log("done ✓");
  })
  .catch((err) => {
    window.__WASM_COMPARE_RESULT__ = { ok: false, error: String(err?.stack ?? err) };
    window.__WASM_COMPARE_DONE__ = true;
    log(`FAILED: ${err}`);
  });
