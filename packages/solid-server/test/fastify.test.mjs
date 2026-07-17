// [FABLE-5] #2323: integration tests for the `@jeswr/solid-server/fastify` plugin — a REAL
// Fastify instance driving the REAL wasm pod (no stubs), via fastify's bundled inject()
// (light-my-request), so no listener socket is needed.
//
// SKIP CONDITIONS (both reported loudly, never silently green):
// - `fastify` is an OPTIONAL peer dependency and is not yet a workspace devDependency (that
//   lockfile refresh needs npm + network — tracked as a follow-up issue); until it lands this
//   suite reports SKIP in CI rather than failing module resolution.
// - the wasm artifact (`npm run build:lws-wasm`) is a build product; like server.test.mjs this
//   suite needs it, but unlike server.test.mjs it skips instead of failing at import time.
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import test from 'node:test';

let Fastify;
try {
  ({ default: Fastify } = await import('fastify'));
} catch {
  Fastify = undefined;
}

const wasmBuilt = existsSync(new URL('../wasm/sparq_lws_wasm.js', import.meta.url));
const skip = Fastify === undefined
  ? 'fastify is not installed (optional peer; devDependency + lockfile follow-up pending)'
  : (wasmBuilt ? false : 'wasm artifact missing — run `npm run build:lws-wasm` first');

const baseUrl = 'http://127.0.0.1';
const ownerWebid = 'https://id.example/alice#me';
const turtle = '<http://127.0.0.1/card> <http://xmlns.com/foaf/0.1/name> "Ada" .\n';

async function buildApp() {
  // Dynamic import: src/fastify.js statically imports the wasm glue via index.js, so it must
  // not be loaded when the suite is skipping.
  const { solidPod } = await import('../src/fastify.js');
  const app = Fastify();
  await app.register(solidPod, { baseUrl, ownerWebid });
  await app.ready();
  return app;
}

test('fastify plugin round-trips Turtle through the wasm pod', { skip, timeout: 60_000 }, async () => {
  const app = await buildApp();
  try {
    const put = await app.inject({
      method: 'PUT',
      url: '/card',
      headers: { 'content-type': 'text/turtle' },
      payload: turtle,
    });
    assert.equal(put.statusCode, 201);

    const get = await app.inject({
      method: 'GET',
      url: '/card',
      headers: { accept: 'text/turtle' },
    });
    assert.equal(get.statusCode, 200);
    assert.equal(get.body, turtle, 'bytes reach wasm unparsed and round-trip exactly');

    const missing = await app.inject({ method: 'GET', url: '/missing' });
    assert.equal(missing.statusCode, 404);
  } finally {
    await app.close();
  }
});

test('fastify plugin enforces WAC through the pod', { skip, timeout: 60_000 }, async () => {
  const app = await buildApp();
  try {
    const locked = '<http://127.0.0.1/locked> <http://xmlns.com/foaf/0.1/name> "Private" .\n';
    const putLocked = await app.inject({
      method: 'PUT',
      url: '/locked',
      headers: { 'content-type': 'text/turtle' },
      payload: locked,
    });
    assert.equal(putLocked.statusCode, 201);

    const acl = `@prefix acl: <http://www.w3.org/ns/auth/acl#> .
<#mallory> a acl:Authorization;
  acl:agent <https://id.example/mallory#me>;
  acl:accessTo <http://127.0.0.1/locked>;
  acl:mode acl:Read .
`;
    const putAcl = await app.inject({
      method: 'PUT',
      url: '/locked.acl',
      headers: { 'content-type': 'text/turtle' },
      payload: acl,
    });
    assert.equal(putAcl.statusCode, 201);

    const denied = await app.inject({
      method: 'GET',
      url: '/locked',
      headers: { accept: 'text/turtle' },
    });
    assert.equal(denied.statusCode, 403, 'the owner is denied by the mallory-only ACL');
  } finally {
    await app.close();
  }
});

test('fastify plugin maps the body-limit error to the host 413 shape', { skip, timeout: 60_000 }, async () => {
  const app = await buildApp();
  try {
    const big = await app.inject({
      method: 'PUT',
      url: '/big',
      headers: { 'content-type': 'text/turtle' },
      payload: Buffer.alloc(2 * 1024 * 1024 + 1),
    });
    assert.equal(big.statusCode, 413);
    assert.equal(big.headers['content-type'], 'text/plain; charset=utf-8');
    assert.equal(big.body, 'request body too large\n');
  } finally {
    await app.close();
  }
});

test('fastify plugin preserves the /sparql query string', { skip, timeout: 60_000 }, async () => {
  const app = await buildApp();
  try {
    const put = await app.inject({
      method: 'PUT',
      url: '/card',
      headers: { 'content-type': 'text/turtle' },
      payload: turtle,
    });
    assert.equal(put.statusCode, 201);

    const query = 'SELECT ?name FROM <http://www.w3.org/ns/solid/sparql#union-default-graph> ' +
      'WHERE { ?s <http://xmlns.com/foaf/0.1/name> ?name }';
    const res = await app.inject({
      method: 'GET',
      url: `/sparql?query=${encodeURIComponent(query)}`,
    });
    assert.equal(res.statusCode, 200, `sparql endpoint answered ${res.statusCode}: ${res.body}`);
    const json = JSON.parse(res.body);
    assert.deepEqual(
      json.results.bindings.map((b) => b.name.value),
      ['Ada'],
      'the query parameter survived request.raw.url intact',
    );
  } finally {
    await app.close();
  }
});
