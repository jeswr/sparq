// [FABLE-5] #2323: transport-agnostic pod dispatcher — the request↔wasm logic from
// attachTrapRecoveryHandler with the node:http transport factored out, so framework hosts
// (the first-party Fastify plugin in fastify.js, or a service-worker fetch handler) can
// mount the wasm pod without a Node socket attached.
import { Buffer } from 'node:buffer';

import {
  MAX_BODY_BYTES,
  RequestBodyTooLargeError,
  copyWasmResponse,
  flattenRequestHeaders,
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

function textResponse(status, text) {
  return {
    status,
    headers: ['content-type', 'text/plain; charset=utf-8'],
    body: Buffer.from(text),
  };
}

/** Enforce the same explicit body ceiling on pre-buffered dispatch bodies as the Node reader. */
function normalizeDispatchBody(body) {
  if (body === undefined || body === null) return new Uint8Array(0);
  const bytes = typeof body === 'string' ? new Uint8Array(Buffer.from(body)) : body;
  if (!(bytes instanceof Uint8Array)) {
    throw new TypeError('body must be a string, Buffer, or Uint8Array');
  }
  if (bytes.length > MAX_BODY_BYTES) {
    throw new RequestBodyTooLargeError();
  }
  return bytes;
}

/**
 * Build the request view handed to `authenticate` — the subset of a Node `IncomingMessage` the
 * Solid-OIDC authenticator reads (`method`, origin-form `url`, `headers`, `headersDistinct`),
 * derived from the flat name/value header pairs so no Node socket is required.
 */
function authRequestView(method, url, rawHeaders) {
  const headers = {};
  const headersDistinct = {};
  for (let index = 0; index < rawHeaders.length; index += 2) {
    const name = rawHeaders[index].toLowerCase();
    const value = rawHeaders[index + 1];
    if (headersDistinct[name] === undefined) {
      headersDistinct[name] = [value];
      headers[name] = value;
    } else {
      headersDistinct[name].push(value);
    }
  }
  return { headers, headersDistinct, method, rawHeaders, url };
}

/**
 * Wrap a pod factory in a transport-agnostic request dispatcher with trap recovery.
 *
 * `dispatch({ method, url, rawHeaders, body })` resolves to `{ status, headers, body }`:
 * `url` is the origin-form request target including any query string, `rawHeaders` are flat
 * name/value pairs (repeated fields stay repeated, both directions), `body` is a pre-buffered
 * `string | Buffer | Uint8Array` (or absent), and the returned `body` is a `Buffer`.
 *
 * The dispatcher owns the whole host contract:
 * - the {@link MAX_BODY_BYTES} request-body ceiling → the 413 response shape;
 * - trap recycle ([SONNET-4.6] sq-250si): a `WebAssembly.RuntimeError` frees the poisoned
 *   instance, constructs a fresh pod via `makePod()`, and answers 503 — a later request is
 *   never served by a poisoned instance. If the replacement `makePod()` throws, the dispatcher
 *   fails closed: the poisoned instance is discarded anyway, later dispatches retry the
 *   factory and answer 503 until it succeeds. Dispatches may overlap (node:http and Fastify
 *   both process requests concurrently), so each request is pinned to the exact instance
 *   generation it started on: a trap only installs a replacement when its own generation is
 *   still current — a stale trap never frees or displaces a healthy replacement — and a
 *   retired instance is freed only after its last in-flight request has returned;
 * - copying + freeing the wasm `LwsResponse` before it is handed back;
 * - any other pod/authenticator failure → the 500 response shape.
 * A malformed dispatch call (odd `rawHeaders`, a non-buffer body) throws instead — that is a
 * caller contract violation, not a request outcome.
 *
 * @param {() => { handleRequest: Function, free?: Function }} makePod - Factory that constructs a
 *   fresh pod. Called once up front and again after each wasm trap.
 * @param {{ authenticate: Function }} opts - `authenticate(view)` resolves to a WebID string or
 *   `undefined`; `view` carries `method`, `url`, `rawHeaders`, `headers`, `headersDistinct`.
 * @returns {{ dispatch: Function, free: () => void }}
 */
export function createPodDispatcher(makePod, { authenticate }) {
  // `state.generation` wraps the live instance together with the bookkeeping overlapping
  // dispatches need: `inFlight` counts requests currently executing on the instance, and
  // `retired` marks it poisoned/replaced. On a trap the CURRENT generation is replaced with a
  // fresh one so subsequent requests are not served by a poisoned module; the retired
  // instance is freed only once its last in-flight request has returned, and a trap raised by
  // a request pinned to an already-retired generation never touches the replacement. `null`
  // means the post-trap replacement failed: no instance is live, and dispatch retries the
  // factory.
  const makeGeneration = () => ({ pod: makePod(), inFlight: 0, retired: false });
  const state = { generation: makeGeneration(), freed: false };

  const freeGenerationPod = (generation) => {
    try {
      generation.pod.free?.();
    } catch (_freeError) {
      // ignore errors freeing a poisoned instance
    }
  };

  // Mark a generation as done-for; free its pod now if idle, else the last in-flight
  // request's finally block frees it. Idempotent — the `retired` flag guards double-free.
  const retireGeneration = (generation) => {
    if (generation.retired) return;
    generation.retired = true;
    if (generation.inFlight === 0) {
      freeGenerationPod(generation);
    }
  };

  const free = () => {
    if (!state.freed) {
      state.freed = true;
      if (state.generation !== null) {
        retireGeneration(state.generation);
        state.generation = null;
      }
    }
  };

  async function dispatch(request = {}) {
    if (state.freed) {
      throw new Error('dispatch() called after free()');
    }
    const method = request.method === undefined ? 'GET' : String(request.method);
    const url = request.url === undefined ? '/' : String(request.url);
    const rawHeaders = flattenRequestHeaders(request.rawHeaders ?? []);

    let body;
    try {
      body = normalizeDispatchBody(request.body);
    } catch (error) {
      if (error instanceof RequestBodyTooLargeError) {
        return textResponse(413, 'request body too large\n');
      }
      throw error;
    }

    if (state.generation === null) {
      try {
        state.generation = makeGeneration();
      } catch (recreateError) {
        console.error('[sparq-lws] failed to recreate SolidServer after trap:', recreateError);
        return textResponse(503, 'server temporarily unavailable (wasm trap)\n');
      }
    }

    // Pin this request to the generation that is current NOW. Everything from here to the
    // `inFlight` increment is synchronous, so no concurrent dispatch can retire it in between.
    const generation = state.generation;
    generation.inFlight += 1;
    try {
      const wasmResponse = await generation.pod.handleRequest(
        method,
        url,
        rawHeaders,
        body,
        await authenticate(authRequestView(method, url, rawHeaders)),
      );
      return copyWasmResponse(wasmResponse);
    } catch (error) {
      if (isWasmTrap(error)) {
        console.error('[sparq-lws] wasm trap — recycling SolidServer instance:', error);
        // Only replace the live instance if this request's generation is still the current
        // one — a stale trap (another request already trapped and installed a replacement,
        // or free() already ran) must not displace or free the healthy replacement.
        if (state.generation === generation) {
          // Detach the poisoned generation BEFORE calling the factory: if `makePod()` throws,
          // the trapped instance must not stay reachable as the live generation.
          state.generation = null;
          try {
            state.generation = makeGeneration();
          } catch (recreateError) {
            console.error('[sparq-lws] failed to recreate SolidServer after trap:', recreateError);
          }
        }
        retireGeneration(generation);
        // Respond with 503 rather than 500 to distinguish a transient trap from a logic error.
        return textResponse(503, 'server temporarily unavailable (wasm trap)\n');
      }
      return textResponse(500, 'internal server error\n');
    } finally {
      generation.inFlight -= 1;
      // The pod of a retired generation is freed by whichever in-flight request returns last.
      if (generation.retired && generation.inFlight === 0) {
        freeGenerationPod(generation);
      }
    }
  }

  return { dispatch, free };
}
