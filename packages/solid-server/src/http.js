// [GPT-5.6] sq-6xasp.9: byte-faithful Node HTTP adaptation for the wasm request boundary.
import { Buffer } from 'node:buffer';

export const MAX_BODY_BYTES = 2 * 1024 * 1024;

export class RequestBodyTooLargeError extends Error {
  constructor() {
    super(`request body exceeds ${MAX_BODY_BYTES} bytes`);
    this.name = 'RequestBodyTooLargeError';
  }
}

/** Clone Node's raw name/value sequence without coalescing repeated fields. */
export function flattenRequestHeaders(rawHeaders) {
  if (!Array.isArray(rawHeaders) || rawHeaders.length % 2 !== 0) {
    throw new TypeError('rawHeaders must contain name/value pairs');
  }
  return rawHeaders.map((value) => String(value));
}

/** Buffer a request up to the same explicit ceiling enforced by the wasm router. */
export async function readRequestBody(request) {
  const chunks = [];
  let length = 0;
  for await (const chunk of request) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    length += bytes.length;
    if (length > MAX_BODY_BYTES) {
      request.resume();
      throw new RequestBodyTooLargeError();
    }
    chunks.push(bytes);
  }
  return new Uint8Array(Buffer.concat(chunks, length));
}

/** Copy a wasm response before freeing it, preserving repeated response fields. */
export function copyWasmResponse(response) {
  try {
    const headers = Array.from(response.headers, String);
    if (headers.length % 2 !== 0) {
      throw new TypeError('wasm response headers must contain name/value pairs');
    }
    return {
      status: Number(response.status),
      headers,
      body: Buffer.from(response.body),
    };
  } finally {
    response.free?.();
  }
}

/** Reconstruct a Node response without collapsing duplicate header names. */
export function writeNodeResponse(response, result) {
  if (!Number.isInteger(result.status) || result.status < 100 || result.status > 999) {
    throw new TypeError(`invalid response status ${result.status}`);
  }
  if (result.headers.length % 2 !== 0) {
    throw new TypeError('response headers must contain name/value pairs');
  }
  response.statusCode = result.status;
  for (let index = 0; index < result.headers.length; index += 2) {
    response.appendHeader(result.headers[index], result.headers[index + 1]);
  }
  response.end(result.body);
}
