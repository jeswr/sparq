/**
 * Environment-aware initialisation of the wasm-pack (`--target web`) module:
 * in Node the wasm bytes are read from disk (Node's `fetch` cannot load
 * `file:` URLs); in the browser/Deno the default `fetch`-relative-to-module
 * path is used. Initialisation is idempotent and memoised.
 */
import initWasm, { Store as WasmStore } from '../wasm/sparq_wasm.js';

let ready: Promise<void> | undefined;

declare const process: { versions?: { node?: string } } | undefined;

export function init(): Promise<void> {
  if (!ready) {
    ready = (async () => {
      if (typeof process !== 'undefined' && process?.versions?.node) {
        const { readFile } = await import('node:fs/promises');
        const bytes = await readFile(new URL('../wasm/sparq_wasm_bg.wasm', import.meta.url));
        await initWasm({ module_or_path: bytes });
      } else {
        await initWasm();
      }
    })();
    // Allow a retry if initialisation failed (e.g. transient fetch error).
    ready.catch(() => {
      ready = undefined;
    });
  }
  return ready;
}

export { WasmStore };
