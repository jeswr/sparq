<!-- [GPT-5.6] sq-6xasp.3: full-template README for the opt-in wasm Solid-server adapter. -->
# sparq-lws-wasm

An opt-in WebAssembly adapter for SPARQ's experimental Solid/LDP server core. It
owns a single-threaded in-memory pod and drives the real axum router as a Tower
service. The JavaScript host supplies the HTTP listener and authenticated WebID.

> This local-development adapter is not a production Solid server. Storage is
> process-local and ephemeral; TLS and OIDC verification stay in the host; PoP,
> notifications, networking, and persistent backends are excluded from wasm.

## 🚀 Quickstart

```sh
wasm-pack build crates/sparq-lws-wasm --target web
```

The `@jeswr/solid-server` workspace package stages the generated JavaScript,
TypeScript declarations, and wasm binary in its own `wasm/` directory:

```sh
npm --workspace @jeswr/solid-server run build:lws-wasm
npm --workspace @jeswr/solid-server run build:lws-wasm-core
```

Both commands intentionally emit the same core-Solid artifact until `sq-r1ei8` adds the
`sparq-lws-core/sparql-endpoint` implementation. The default-off `sparql-endpoint` feature
reserves that future full-tier selector without changing the feature-off wasm build.

```js
import init, { SolidServer } from "./pkg/sparq_lws_wasm.js";
await init();

const owner = "https://id.example/alice#me";
const pod = new SolidServer("https://pod.example", owner);
const response = await pod.handleRequest(
  "PUT",
  "/card",
  ["content-type", "text/turtle"],
  new TextEncoder().encode("<card> <name> \"Ada\" .\n"),
  owner,
);
console.log(response.status);
```

## ✨ Features

- `CompositeStore<InMemorySparqClient, InMemoryBlobStore>` keeps metadata and
  bytes inside wasm linear memory.
- `handleRequest(method, path, headers, body, authenticatedWebid)` constructs an
  HTTP request and invokes the existing LDP + WAC router. Headers are flat
  name/value arrays so repeated fields survive the wasm boundary.
- The constructor provisions one owner ACL. Each request is anonymous when
  `authenticatedWebid` is absent; otherwise the host must have verified that
  WebID before passing it to wasm.
- No Tokio reactor, native listener, filesystem, TLS, OIDC verifier, PoP,
  notifications, or network backend is linked into the wasm artifact.

## 📚 Learn more

- Usage and host-authentication contract:
  [`skills/javascript-wasm/SKILL.md`](../../skills/javascript-wasm/SKILL.md).
- Portable request core: [`sparq-lws-core`](../sparq-lws-core/README.md), built
  with its default-off `wasm` feature.
- [GPT-5.6] The [`@jeswr/solid-server`](../../packages/solid-server/) package owns
  the loopback Node listener and `npx` entry. Its fixed-owner mode is for local
  development only; OIDC verification remains outside this wasm crate.

## License

[MIT](../../LICENSE).
