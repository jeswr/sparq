// [GPT-5.6] sq-6xasp.10: fail-closed Solid-OIDC verification at the Node/wasm trust boundary.
import { createSolidTokenVerifier } from '@solid/access-token-verifier';

function singleHeader(request, name) {
  const distinctValues = request.headersDistinct?.[name];
  if (distinctValues !== undefined) {
    return distinctValues.length === 1 ? distinctValues[0] : undefined;
  }
  const value = request.headers[name];
  return typeof value === 'string' ? value : undefined;
}

function verifiedWebid(payload) {
  if (typeof payload?.webid !== 'string') return undefined;
  const url = new URL(payload.webid);
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) {
    return undefined;
  }
  return payload.webid;
}

/**
 * Build the opt-in request authenticator.
 *
 * `verifier` is an internal test seam. Production callers use the verifier package's cached
 * WebID, issuer, JWKS, access-token, DPoP-proof, and replay checks.
 */
export function createOidcAuthenticator(baseUrl, verifier = createSolidTokenVerifier()) {
  return async function authenticate(request) {
    const authorization = singleHeader(request, 'authorization');
    if (authorization === undefined || !/^DPoP +/i.test(authorization)) return undefined;

    const dpopHeader = singleHeader(request, 'dpop');
    if (dpopHeader === undefined) return undefined;
    const dpop = {
      header: dpopHeader,
      method: request.method ?? 'GET',
      url: new URL(request.url ?? '/', `${baseUrl}/`).href,
    };

    try {
      return verifiedWebid(await verifier(authorization, dpop));
    } catch {
      return undefined;
    }
  };
}
