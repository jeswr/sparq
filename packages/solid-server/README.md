<!-- [GPT-5.6] sq-6xasp.9: honest public-package README for the Node wasm host. -->
# @jeswr/solid-server

A local-development [Solid](https://solidproject.org/) LDP + WAC server backed by
sparq WebAssembly. Node owns the loopback HTTP listener; one long-lived wasm
`SolidServer` owns the in-memory pod.

> This v1 package is not a production server. Every request is treated as the
> configured owner WebID without authentication. Do not expose the listener or
> place sensitive data in it.

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
- Authentication is fixed-owner local mode. There is no OIDC verification.
- TLS, persistent storage, notifications, PoP, and native networking remain
  outside the wasm module.
- The current full/core builds contain the same Solid-only surface; the SPARQL
  endpoint is tracked separately in `sq-r1ei8`.

## 📚 API

`startSolidServer({ port, baseUrl, ownerWebid })` resolves to a Node
`http.Server` after it is listening. The package adds
`server.closeAsync(): Promise<void>` for clean shutdown. One server instance
reuses one wasm pod, so writes remain visible until shutdown.

See the [JavaScript/Wasm usage skill](../../skills/javascript-wasm/SKILL.md)
for the raw request adapter and host contract.

## License

[MIT](../../LICENSE).
