// [FABLE-5] #2323: unit tests for the transport-agnostic pod dispatcher (createPodDispatcher).
//
// These exercise the real dispatch logic in src/dispatch.js with stubbed pods — no wasm binary
// and no listener socket. The node:http shell over the same dispatcher is covered by
// trap-recovery.test.mjs; the Fastify plugin over it by fastify.test.mjs.
//
// Non-vacuity: the trap-recycle assertion fails when the recycle assignment is removed from
// createPodDispatcher (both requests then answer 503, not 503 then 200), and the 413 assertion
// fails when the pre-buffered ceiling check is removed from normalizeDispatchBody (the stub pod
// then receives the oversized body and answers 200).
import assert from 'node:assert/strict';
import test from 'node:test';

import { createPodDispatcher } from '../src/dispatch.js';
import { MAX_BODY_BYTES } from '../src/http.js';

function fakePod(handleFn) {
  return {
    freed: false,
    handleRequest: handleFn,
    free() { this.freed = true; },
  };
}

function okResponse(bodyText = 'ok\n', headers = ['content-type', 'text/plain; charset=utf-8']) {
  return {
    status: 200,
    headers,
    body: new Uint8Array(Buffer.from(bodyText)),
    freed: false,
    free() { this.freed = true; },
  };
}

const anonymous = { authenticate: async () => undefined };

test('dispatch forwards method/url/flat headers/body and copies + frees the wasm response', async () => {
  const seen = {};
  const wasmResponse = okResponse('ok\n', ['link', '<a>; rel="type"', 'link', '<b>; rel="type"']);
  const pod = fakePod(async (method, url, headers, body, webid) => {
    Object.assign(seen, { body, headers, method, url, webid });
    return wasmResponse;
  });
  const { dispatch, free } = createPodDispatcher(() => pod, {
    authenticate: async () => 'https://id.example/alice#me',
  });

  const result = await dispatch({
    method: 'PUT',
    url: '/card?keep=query',
    rawHeaders: ['Accept', 'text/turtle', 'Accept', 'text/html'],
    body: 'turtle bytes',
  });

  assert.equal(seen.method, 'PUT');
  assert.equal(seen.url, '/card?keep=query', 'origin-form target incl. query reaches the pod');
  assert.deepEqual(
    seen.headers,
    ['Accept', 'text/turtle', 'Accept', 'text/html'],
    'repeated request headers stay repeated flat pairs',
  );
  assert.equal(Buffer.from(seen.body).toString(), 'turtle bytes');
  assert.equal(seen.webid, 'https://id.example/alice#me');

  assert.equal(result.status, 200);
  assert.deepEqual(
    result.headers,
    ['link', '<a>; rel="type"', 'link', '<b>; rel="type"'],
    'repeated response headers stay repeated flat pairs',
  );
  assert.ok(Buffer.isBuffer(result.body));
  assert.equal(result.body.toString(), 'ok\n');
  assert.equal(wasmResponse.freed, true, 'the wasm response is freed after the copy');
  free();
});

test('authenticate receives a request view with method/url/headers/headersDistinct', async () => {
  let view;
  const pod = fakePod(async () => okResponse());
  const { dispatch, free } = createPodDispatcher(() => pod, {
    authenticate: async (request) => { view = request; return undefined; },
  });

  await dispatch({
    method: 'GET',
    url: '/private?x=1',
    rawHeaders: ['Authorization', 'DPoP token', 'DPoP', 'proof', 'Accept', 'a', 'Accept', 'b'],
  });

  assert.equal(view.method, 'GET');
  assert.equal(view.url, '/private?x=1');
  assert.deepEqual(view.headersDistinct.authorization, ['DPoP token']);
  assert.deepEqual(view.headersDistinct.accept, ['a', 'b'], 'repeats preserved distinctly');
  assert.equal(view.headers.dpop, 'proof');
  free();
});

test('an oversized pre-buffered body answers the host 413 shape without reaching the pod', async () => {
  let handled = 0;
  const pod = fakePod(async () => { handled += 1; return okResponse(); });
  const { dispatch, free } = createPodDispatcher(() => pod, anonymous);

  const result = await dispatch({
    method: 'PUT',
    url: '/big',
    body: Buffer.alloc(MAX_BODY_BYTES + 1),
  });

  assert.equal(result.status, 413);
  assert.deepEqual(result.headers, ['content-type', 'text/plain; charset=utf-8']);
  assert.equal(result.body.toString(), 'request body too large\n');
  assert.equal(handled, 0, 'the pod never sees an oversized body');

  const exact = await dispatch({ method: 'PUT', url: '/fits', body: Buffer.alloc(MAX_BODY_BYTES) });
  assert.equal(exact.status, 200, 'a body of exactly MAX_BODY_BYTES is admitted');
  free();
});

test('a wasm trap answers 503, frees the poisoned pod, and recycles for the next dispatch', async () => {
  const pod1 = fakePod(async () => { throw new WebAssembly.RuntimeError('unreachable'); });
  const pod2 = fakePod(async () => okResponse());
  const pods = [pod1, pod2];
  let makeCount = 0;
  const { dispatch, free } = createPodDispatcher(() => { makeCount += 1; return pods.shift(); }, anonymous);

  const trapped = await dispatch({ url: '/trigger-trap' });
  assert.equal(trapped.status, 503, 'a wasm trap answers 503');
  assert.equal(trapped.body.toString(), 'server temporarily unavailable (wasm trap)\n');
  assert.equal(pod1.freed, true, 'the poisoned pod is freed on trap');
  assert.equal(makeCount, 2, 'the factory supplies the replacement instance');

  const ok = await dispatch({ url: '/any-path' });
  assert.equal(ok.status, 200, 'the next dispatch is served by the fresh instance');
  assert.equal(pod2.freed, false);
  free();
  assert.equal(pod2.freed, true, 'free() releases the current (recycled) pod');
});

test('a non-trap pod error answers 500 and does NOT recycle the instance', async () => {
  let makeCount = 0;
  const pod = fakePod(async () => { throw new Error('internal logic error'); });
  const { dispatch, free } = createPodDispatcher(() => { makeCount += 1; return pod; }, anonymous);

  const result = await dispatch({ url: '/any' });
  assert.equal(result.status, 500);
  assert.equal(result.body.toString(), 'internal server error\n');
  assert.equal(makeCount, 1, 'the factory is not called again on a non-trap error');
  assert.equal(pod.freed, false, 'the pod is not freed on a non-trap error');
  free();
});

test('free() is idempotent and dispatch afterwards throws a caller error', async () => {
  const pod = fakePod(async () => okResponse());
  const { dispatch, free } = createPodDispatcher(() => pod, anonymous);
  free();
  free();
  assert.equal(pod.freed, true);
  await assert.rejects(() => dispatch({ url: '/late' }), /after free\(\)/);
});

test('malformed dispatch calls throw to the caller instead of masking as 500', async () => {
  const pod = fakePod(async () => okResponse());
  const { dispatch, free } = createPodDispatcher(() => pod, anonymous);
  await assert.rejects(() => dispatch({ rawHeaders: ['odd'] }), TypeError);
  await assert.rejects(() => dispatch({ body: 42 }), TypeError);
  free();
});
