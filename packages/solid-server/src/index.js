// [GPT-5.6] sq-6xasp.9/.10: loopback Node listener with optional verified identity into wasm.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';

import initWasm, { SolidServer } from '../wasm/sparq_lws_wasm.js';
import { createOidcAuthenticator } from './auth.js';
import {
  RequestBodyTooLargeError,
  copyWasmResponse,
  flattenRequestHeaders,
  readRequestBody,
  writeNodeResponse,
} from './http.js';

const DEFAULT_PORT = 3000;
const DEFAULT_OWNER_WEBID = 'https://example.invalid/profile/card#me';
let wasmReady;

async function init() {
  if (!wasmReady) {
    wasmReady = (async () => {
      const bytes = await readFile(new URL('../wasm/sparq_lws_wasm_bg.wasm', import.meta.url));
      await initWasm({ module_or_path: bytes });
    })();
    wasmReady.catch(() => {
      wasmReady = undefined;
    });
  }
  return wasmReady;
}

function normalizePort(value) {
  const port = value ?? DEFAULT_PORT;
  if (!Number.isInteger(port) || port < 0 || port > 65_535) {
    throw new TypeError('port must be an integer from 0 through 65535');
  }
  return port;
}

function normalizeBaseUrl(value, port) {
  if (value === undefined && port === 0) {
    throw new TypeError('baseUrl is required when port is 0');
  }
  const url = new URL(value ?? `http://127.0.0.1:${port}`);
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new TypeError('baseUrl must use http or https');
  }
  if (url.username || url.password || url.search || url.hash || url.pathname !== '/') {
    throw new TypeError('baseUrl must be an origin without credentials, path, query, or fragment');
  }
  return url.origin;
}

function normalizeOwnerWebid(value) {
  const ownerWebid = value ?? DEFAULT_OWNER_WEBID;
  const url = new URL(ownerWebid);
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) {
    throw new TypeError('ownerWebid must be an http(s) WebID without credentials');
  }
  return ownerWebid;
}

function normalizeOidc(value) {
  if (value === undefined || value === false) return false;
  if (value === true) return true;
  throw new TypeError('oidc must be a boolean');
}

async function closeServer(server) {
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

/**
 * Start a loopback-only Solid/LDP development server.
 *
 * The default fixed-owner mode is deliberately not authentication. With `oidc: true`, the Node
 * host verifies each credential and anonymous requests remain anonymous.
 * The returned Node server owns one in-memory wasm pod until `server.close()` completes.
 */
export async function startSolidServer(options = {}) {
  const port = normalizePort(options.port);
  const baseUrl = normalizeBaseUrl(options.baseUrl, port);
  const ownerWebid = normalizeOwnerWebid(options.ownerWebid);
  const oidc = normalizeOidc(options.oidc);

  await init();
  const pod = new SolidServer(baseUrl, ownerWebid);
  const authenticate = oidc ? createOidcAuthenticator(baseUrl) : async () => ownerWebid;
  let podFreed = false;
  const freePod = () => {
    if (!podFreed) {
      podFreed = true;
      pod.free?.();
    }
  };

  const server = createServer((request, response) => {
    void (async () => {
      try {
        const wasmResponse = await pod.handleRequest(
          request.method ?? 'GET',
          request.url ?? '/',
          flattenRequestHeaders(request.rawHeaders),
          await readRequestBody(request),
          await authenticate(request),
        );
        writeNodeResponse(response, copyWasmResponse(wasmResponse));
      } catch (error) {
        if (response.headersSent) {
          response.destroy();
          return;
        }
        response.statusCode = error instanceof RequestBodyTooLargeError ? 413 : 500;
        response.setHeader('content-type', 'text/plain; charset=utf-8');
        response.end(error instanceof RequestBodyTooLargeError ? 'request body too large\n' : 'internal server error\n');
      }
    })();
  });

  server.once('close', freePod);
  server.closeAsync = () => closeServer(server);

  try {
    await new Promise((resolve, reject) => {
      const onError = (error) => {
        server.off('listening', onListening);
        reject(error);
      };
      const onListening = () => {
        server.off('error', onError);
        resolve();
      };
      server.once('error', onError);
      server.once('listening', onListening);
      server.listen(port, '127.0.0.1');
    });
  } catch (error) {
    freePod();
    throw error;
  }

  return server;
}
