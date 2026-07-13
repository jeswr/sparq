// [GPT-5.6] sq-6xasp.10: mutation witness for verify-before-trust and fail-anonymous behavior.
import assert from 'node:assert/strict';
import test from 'node:test';

import { createOidcAuthenticator } from '../src/auth.js';

function request(headers = {}) {
  return {
    headers,
    method: 'GET',
    url: '/private?view=full',
  };
}

test('missing credentials stay anonymous without invoking the verifier', async () => {
  let calls = 0;
  const authenticate = createOidcAuthenticator('https://pod.example', async () => {
    calls += 1;
    return { webid: 'https://id.example/alice#me' };
  });

  assert.equal(await authenticate(request()), undefined);
  assert.equal(calls, 0);
});

test('an unbound bearer token stays anonymous without invoking the verifier', async () => {
  let calls = 0;
  const authenticate = createOidcAuthenticator('https://pod.example', async () => {
    calls += 1;
    return { webid: 'https://id.example/alice#me' };
  });

  assert.equal(
    await authenticate(request({ authorization: 'Bearer signed-token' })),
    undefined,
  );
  assert.equal(calls, 0);
});

test('an invalid credential is verified and then stays anonymous', async () => {
  const calls = [];
  const authenticate = createOidcAuthenticator('https://pod.example', async (...args) => {
    calls.push(args);
    throw new Error('invalid signature');
  });

  assert.equal(await authenticate(request({
    authorization: 'DPoP invalid-token',
    dpop: 'invalid-proof',
  })), undefined);
  assert.deepEqual(calls, [[
    'DPoP invalid-token',
    {
      header: 'invalid-proof',
      method: 'GET',
      url: 'https://pod.example/private?view=full',
    },
  ]]);
});

test('only a WebID returned by the verifier becomes authenticated', async () => {
  const authenticate = createOidcAuthenticator(
    'https://pod.example',
    async () => ({ webid: 'https://id.example/alice#me' }),
  );
  assert.equal(
    await authenticate(request({
      authorization: 'DPoP signed-token',
      dpop: 'signed-proof',
    })),
    'https://id.example/alice#me',
  );
});
