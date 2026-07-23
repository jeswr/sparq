// [SONNET-4.6] sq-250si: trap-recovery utilities extracted so tests can import them without a
// real wasm binary. `index.js` re-exports everything; callers should import from index.js.
// [FABLE-5] #2323: the request↔wasm logic now lives in dispatch.js (createPodDispatcher);
// this module is the thin node:http shell over it — behaviour unchanged.
import { createServer } from 'node:http';

import { createPodDispatcher, isWasmTrap } from './dispatch.js';
import {
  RequestBodyTooLargeError,
  readRequestBody,
  writeNodeResponse,
} from './http.js';

export { isWasmTrap };

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
 * @param {{ authenticate: Function }} opts - `authenticate(view)` resolves to a WebID string or
 *   `undefined`; `view` carries the request's `method`, `url`, `headers`, `headersDistinct`, and
 *   `rawHeaders` (the fields the Solid-OIDC authenticator reads), not the raw Node socket.
 * @returns {{ server: import('node:http').Server, freePod: () => void }}
 */
export function attachTrapRecoveryHandler(makePod, { authenticate }) {
  const { dispatch, free: freePod } = createPodDispatcher(makePod, { authenticate });

  const server = createServer((request, response) => {
    void (async () => {
      try {
        const body = await readRequestBody(request);
        const result = await dispatch({
          method: request.method ?? 'GET',
          url: request.url ?? '/',
          rawHeaders: request.rawHeaders,
          body,
        });
        writeNodeResponse(response, result);
      } catch (error) {
        // dispatch() answers pod failures itself (413/503/500); only transport-level errors
        // reach here — the streaming body-ceiling reader and response-write failures.
        if (response.headersSent) {
          response.destroy();
          return;
        }
        if (error instanceof RequestBodyTooLargeError) {
          response.statusCode = 413;
          response.setHeader('content-type', 'text/plain; charset=utf-8');
          response.end('request body too large\n');
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
