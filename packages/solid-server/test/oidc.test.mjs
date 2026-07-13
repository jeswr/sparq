// [GPT-5.6] sq-6xasp.10: real Solid-OIDC + DPoP verification through Node, wasm, and WAC.
import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { createServer } from 'node:http';
import test from 'node:test';

import {
  base64url,
  calculateJwkThumbprint,
  exportJWK,
  generateKeyPair,
  SignJWT,
} from 'jose';

import { startSolidServer } from '../src/index.js';

const podBaseUrl = 'http://127.0.0.1';
const turtle = '<http://127.0.0.1/private> <http://xmlns.com/foaf/0.1/name> "Private" .\n';

async function listen(server, host = 'localhost') {
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, host, resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address !== 'string');
  return address.port;
}

async function close(server) {
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

async function createIdentityFixture() {
  const issuerKeys = await generateKeyPair('ES256', { extractable: true });
  const issuerJwk = {
    ...await exportJWK(issuerKeys.publicKey),
    alg: 'ES256',
    kid: 'issuer-signing-key',
    use: 'sig',
  };
  let issuer;
  let webid;
  const identityServer = createServer((request, response) => {
    if (request.url === '/profile') {
      response.setHeader('content-type', 'text/turtle');
      response.end(`<${webid}> <http://www.w3.org/ns/solid/terms#oidcIssuer> <${issuer}> .\n`);
      return;
    }
    if (request.url === '/.well-known/openid-configuration') {
      response.setHeader('content-type', 'application/json');
      response.end(JSON.stringify({ issuer, jwks_uri: `${issuer}/jwks` }));
      return;
    }
    if (request.url === '/jwks') {
      response.setHeader('content-type', 'application/json');
      response.end(JSON.stringify({ keys: [issuerJwk] }));
      return;
    }
    response.statusCode = 404;
    response.end();
  });
  const identityPort = await listen(identityServer);
  issuer = `http://localhost:${identityPort}`;
  webid = `${issuer}/profile#me`;

  const dpopKeys = await generateKeyPair('ES256', { extractable: true });
  const dpopJwk = await exportJWK(dpopKeys.publicKey);
  const thumbprint = await calculateJwkThumbprint(dpopJwk);

  async function accessToken({ expired = false } = {}) {
    const now = Math.floor(Date.now() / 1000);
    return new SignJWT({
      client_id: 'https://app.example/client',
      cnf: { jkt: thumbprint },
      webid,
    })
      .setProtectedHeader({ alg: 'ES256', kid: issuerJwk.kid })
      .setIssuer(issuer)
      .setAudience('solid')
      .setIssuedAt(expired ? now - 600 : now)
      .setExpirationTime(expired ? now - 300 : now + 300)
      .sign(issuerKeys.privateKey);
  }

  async function headers(method, path, token) {
    const ath = base64url.encode(createHash('sha256').update(token, 'ascii').digest());
    const proof = await new SignJWT({
      ath,
      htm: method,
      htu: `${podBaseUrl}${path}`,
      jti: randomUUID(),
    })
      .setProtectedHeader({ alg: 'ES256', jwk: dpopJwk, typ: 'dpop+jwt' })
      .setIssuedAt()
      .sign(dpopKeys.privateKey);
    return { authorization: `DPoP ${token}`, dpop: proof };
  }

  return { accessToken, close: () => close(identityServer), headers, webid };
}

function listenerUrl(server) {
  const address = server.address();
  assert.ok(address && typeof address !== 'string');
  return `http://127.0.0.1:${address.port}`;
}

function corruptSignature(token) {
  const segments = token.split('.');
  const signature = segments[2];
  segments[2] = `${signature[0] === 'A' ? 'B' : 'A'}${signature.slice(1)}`;
  return segments.join('.');
}

function assertDenied(response) {
  assert.ok([401, 403].includes(response.status), `expected 401/403, received ${response.status}`);
}

test('OIDC mode verifies DPoP credentials and leaves failed credentials anonymous', { timeout: 30_000 }, async () => {
  const identity = await createIdentityFixture();
  let server;

  try {
    server = await startSolidServer({
      baseUrl: podBaseUrl,
      oidc: true,
      ownerWebid: identity.webid,
      port: 0,
    });
    const url = listenerUrl(server);
    const token = await identity.accessToken();
    const put = await fetch(`${url}/private`, {
      body: turtle,
      headers: {
        ...await identity.headers('PUT', '/private', token),
        'content-type': 'text/turtle',
      },
      method: 'PUT',
    });
    assert.equal(put.status, 201);

    const getHeaders = {
      ...await identity.headers('GET', '/private', token),
      accept: 'text/turtle',
    };
    const get = await fetch(`${url}/private`, { headers: getHeaders });
    assert.equal(get.status, 200);
    assert.equal(await get.text(), turtle);

    const replayed = await fetch(`${url}/private`, { headers: getHeaders });
    assertDenied(replayed);

    const missing = await fetch(`${url}/private`);
    assertDenied(missing);

    const invalidToken = corruptSignature(token);
    const invalid = await fetch(`${url}/private`, {
      headers: await identity.headers('GET', '/private', invalidToken),
    });
    assertDenied(invalid);

    const expiredToken = await identity.accessToken({ expired: true });
    const expired = await fetch(`${url}/private`, {
      headers: await identity.headers('GET', '/private', expiredToken),
    });
    assertDenied(expired);
  } finally {
    await server?.closeAsync();
    await identity.close();
  }
});
