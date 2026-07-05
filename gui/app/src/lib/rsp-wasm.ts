// [SONNET-4.6] sq-kwb74 — the GUI-side loader for the RSP-QL wasm bundle
// (crates/sparq-rsp-wasm), mirroring the pattern of lib/reason-wasm.ts.
//
// The bundle provides a stateful windowed-query handle (Rsp) that accepts timestamped
// triple pushes and fires closed windows as JSON. Like the reason-wasm loader, the
// wasm-pack glue is imported at RUNTIME with a webpackIgnore dynamic import from /public,
// so the bundle never enters the page chunk; asset URLs are prefixed with the same
// NEXT_PUBLIC_BASE_PATH the rest of the GUI uses (@/lib/base-path). The bundle is
// OPTIONAL: if it is not present in this build, loadRspModule() rejects and the streaming
// tool degrades honestly to "unavailable" rather than crashing.

import { basePath } from "@/lib/base-path";

/**
 * The JS-facing `Rsp` query handle the RSP wasm bundle exports — one live stateful
 * query that accepts timestamped triple pushes and fires closed windows as JSON.
 *
 * Terms are Turtle-syntax strings: `<iri>`, `10` / `10.5` (numeric shorthand), `"str"`,
 * `"hi"@en`, `"v"^^<…#decimal>`, `_:b`. Timestamps are plain JS `Number`s (not BigInt).
 */
export interface WasmRspQuery {
  /** Push one timestamped triple (Turtle-syntax strings). Returns JSON array of closed windows. */
  push(s: string, p: string, o: string, ts: number): string;
  /** End-of-stream: close remaining windows. Returns same JSON array format as push(). */
  flush(): string;
  /** Count of arrivals dropped as too late (all covering windows already closed). */
  lateDropped(): number;
  /** The registered SPARQL text (echo, for the UI). */
  sparql(): string;
}

/**
 * The JS-facing `Rsp` static class the RSP wasm bundle exports.
 */
export interface WasmRsp {
  select(
    sparql: string,
    range: number,
    step: number,
    maxDelay: number,
    r2s: string,
  ): WasmRspQuery;
}

interface RspModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Rsp: WasmRsp;
}

let modulePromise: Promise<RspModule> | null = null;

/**
 * [SONNET-4.6] sq-kwb74 — load + initialise the RSP wasm bundle once; subsequent calls
 * reuse it. A failed load resets the cache so a later attempt retries (e.g. after a build
 * that syncs the bundle). Rejects when the bundle is not present in this build.
 */
export async function loadRspModule(): Promise<WasmRsp> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const base = basePath();
      const gluePath = `${base}/wasm/rsp/sparq_rsp_wasm.js`;
      const wasmPath = `${base}/wasm/rsp/sparq_rsp_wasm_bg.wasm`;
      const mod = (await import(/* webpackIgnore: true */ gluePath)) as RspModule;
      await mod.default({ module_or_path: new URL(wasmPath, window.location.origin) });
      return mod;
    })();
    modulePromise.catch(() => {
      modulePromise = null; // allow retry on transient failure / a later synced build
    });
  }
  const mod = await modulePromise;
  return mod.Rsp;
}
