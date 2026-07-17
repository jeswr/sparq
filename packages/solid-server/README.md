<!-- [GPT-5.6] sq-6xasp.9/.10: honest public-package README for the Node wasm host. -->
# @jeswr/solid-server

A local-development [Solid](https://solidproject.org/) LDP + WAC server backed by
sparq WebAssembly. Node owns the loopback HTTP listener; one long-lived wasm
`SolidServer` owns the in-memory pod.

> This v1 package is not a production server. Its default mode treats every
> request as the configured owner without authentication. Solid-OIDC verification
> is opt-in and does not add production transport or persistence hardening.

## 🚀 Quickstart

The published npm package includes its wasm artifact:

```sh
npx @jeswr/solid-server \
  --port 3000 \
  --base-url http://127.0.0.1:3000 \
  --owner-webid https://id.example/alice#me
```

From a source checkout, build the artifact first:

```sh
npm --workspace @jeswr/solid-server run build:lws-wasm
npx --yes --package ./packages/solid-server solid-server --port 3000
```

Use it from Node:

```js
import { startSolidServer } from '@jeswr/solid-server';

const server = await startSolidServer({
  port: 3000,
  baseUrl: 'http://127.0.0.1:3000',
  ownerWebid: 'https://id.example/alice#me',
  oidc: true,
});

// Later: await server.closeAsync();
```

Options may also come from `SPARQ_SOLID_PORT` (or `PORT`),
`SPARQ_SOLID_BASE_URL`, and `SPARQ_SOLID_OWNER_WEBID`.

## ✨ Scope

- PUT/GET/POST/PATCH/DELETE and WAC run through the real `sparq-lws-core`
  router over in-wasm metadata and blob stores.
- The listener binds only `127.0.0.1`; `baseUrl` is the public pod origin and
  may differ when a local proxy terminates TLS.
- Storage is process-local and disappears on shutdown.
- Authentication defaults to fixed-owner local mode. Set `oidc: true` in the
  Node API to require and verify Solid-OIDC access tokens plus request-bound DPoP
  proofs with `@solid/access-token-verifier`; missing, invalid, expired, replayed,
  or bearer-only credentials stay anonymous. The `npx` command remains fixed-owner.
- OIDC verification runs in Node, not wasm. `baseUrl` must match the public URL
  used in DPoP proofs; the verifier dereferences the WebID and issuer JWKS.
- TLS, persistent storage, notifications, PoP, and native networking remain
  outside the wasm module.
- [GPT-5.6] The shipped full build includes query-only GET/POST `/sparql` for
  SELECT, ASK, and CONSTRUCT. Its dataset contains one named graph per resource
  the authenticated WebID may read; the default graph is empty. SPARQL Update
  is not exposed.
- `npm run build:lws-wasm-core` produces the pure Solid tier with `/sparql` and
  the embedded query engine compiled out.

## 📚 API

`startSolidServer({ port, baseUrl, ownerWebid, oidc })` resolves to a Node
`http.Server` after it is listening. The package adds
`server.closeAsync(): Promise<void>` for clean shutdown. One server instance
reuses one wasm pod, so writes remain visible until shutdown. `ownerWebid`
provisions the root WAC policy; in OIDC mode it is not an authentication claim.

See the [JavaScript/Wasm usage skill](../../skills/javascript-wasm/SKILL.md)
for the raw request adapter and host contract.

## License

[MIT](../../LICENSE).
