// [SONNET-4.6] sq-250si: trap-recovery utilities extracted so tests can import them without a
// real wasm binary. `index.js` re-exports everything; callers should import from index.js.
import { createServer } from 'node:http';

import {
  RequestBodyTooLargeError,
  copyWasmResponse,
  flattenRequestHeaders,
  readRequestBody,
  writeNodeResponse,
} from './http.js';

/**
 * Returns true when an error is a wasm trap — a `WebAssembly.RuntimeError` thrown by the engine
 * after `panic=abort` lowers to an `unreachable` instruction or after an allocation failure at the
 * wasm32 linear-memory ceiling. A trapped instance is permanently poisoned and must be replaced.
 *
 * [SONNET-4.6] sq-250si: the check is intentionally conservative — only exact
 * `WebAssembly.RuntimeError` instances are treated as traps, not generic `Error`s.
 */
export function isWasmTrap(error) {
  return typeof WebAssembly !== 'undefined' &&
    typeof WebAssembly.RuntimeError === 'function' &&
    error instanceof WebAssembly.RuntimeError;
}

async function closeServer(server) {
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

/**
 * Attach the trap-recovery HTTP request dispatcher to a Node `http.Server` using the provided
 * pod factory. The `makePod()` factory is called once at startup and again after each trap to
 * recycle the wasm instance.
 *
 * Exported for unit testing without a real wasm binary; production callers use `startSolidServer`.
 *
 * [SONNET-4.6] sq-250si: trap-recovery core, separated from wasm init for testability.
 *
 * @param {() => { handleRequest: Function, free?: Function }} makePod - Factory that constructs a
 *   fresh pod. Called once at startup and again after each wasm trap.
 * @param {{ authenticate: Function }} opts - `authenticate(request)` resolves to a WebID string
 *   or `undefined`.
 * @returns {{ server: import('node:http').Server, freePod: () => void }}
 */
export function attachTrapRecoveryHandler(makePod, { authenticate }) {
  // [SONNET-4.6] sq-250si: `state.pod` is the live instance. On a trap it is replaced with a
  // fresh instance so subsequent requests are not permanently served by a poisoned module.
  const state = { pod: makePod(), freed: false };

  const freePod = () => {
    if (!state.freed) {
      state.freed = true;
      state.pod.free?.();
    }
  };

  const server = createServer((request, response) => {
    void (async () => {
      try {
        const wasmResponse = await state.pod.handleRequest(
          request.method ?? 'GET',
          request.url ?? '/',
          flattenRequestHeaders(request.rawHeaders),
          await readRequestBody(request),
          await authenticate(request),
        );
        writeNodeResponse(response, copyWasmResponse(wasmResponse));
      } catch (error) {
        // [SONNET-4.6] sq-250si: recycle the wasm instance on a trap so the next request is
        // served by a fresh pod. State loss is acceptable — this is a local-dev-only server.
        if (isWasmTrap(error)) {
          console.error('[sparq-lws] wasm trap — recycling SolidServer instance:', error);
          try {
            state.pod.free?.();
          } catch (_freeError) {
            // ignore errors freeing a poisoned instance
          }
          try {
            state.pod = makePod();
          } catch (recreateError) {
            console.error('[sparq-lws] failed to recreate SolidServer after trap:', recreateError);
          }
        }

        if (response.headersSent) {
          response.destroy();
          return;
        }
        if (error instanceof RequestBodyTooLargeError) {
          response.statusCode = 413;
          response.setHeader('content-type', 'text/plain; charset=utf-8');
          response.end('request body too large\n');
        } else if (isWasmTrap(error)) {
          // Respond with 503 rather than 500 to distinguish a transient trap from a logic error.
          response.statusCode = 503;
          response.setHeader('content-type', 'text/plain; charset=utf-8');
          response.end('server temporarily unavailable (wasm trap)\n');
        } else {
          response.statusCode = 500;
          response.setHeader('content-type', 'text/plain; charset=utf-8');
          response.end('internal server error\n');
        }
      }
    })();
  });

  server.once('close', freePod);
  server.closeAsync = () => closeServer(server);
  return { server, freePod };
}
