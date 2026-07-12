# Wasm Solid-server feasibility spike (`sq-6xasp.1`) [GPT-5.6]

Status: complete feasibility evidence, not production implementation. These are
standalone research crates and are intentionally outside the SPARQ workspace. No
shipped crate, public API, Cargo feature, or default dependency graph changes.

## Verdict

The reduced in-memory demo is a **go**, subject to the cfg/dependency work in
`sq-6xasp.2`. The portable shape was executed as real wasm in Node: an
`async-trait` `Store`, an in-memory implementation, axum `build_router`, and one
LDP GET driven through `tower::ServiceExt` all complete under
`wasm-bindgen-futures` without Tokio or a native reactor. The test asserts the
status, media type, and exact Turtle body fetched through the store, so replacing
the handler with a constant or disconnecting the route/store makes it fail.

This proves the **lws-core seam and dependency shape**, not that today's
`sparq-lws-core` compiles unchanged. Its current manifest enables Tokio net and
native transport/crypto dependencies before Rust reaches the router source.

Real Solid-OIDC in wasm is a **no-go with the currently pinned verifier revision**.
The `network` feature can be disabled, and the resulting graph contains none of
Tokio, reqwest, or hickory-resolver. However, revision
`89c896249a726398b78302fd2f65eef0a82af681` unconditionally selects
`jsonwebtoken/aws_lc_rs`. After this probe supplies the separate getrandom 0.2
`js` shim, the wasm build reaches `aws-lc-sys`, reports target
`wasm32-unknown-unknown` with no target-specific source, and its native build
step fails. Real auth therefore needs upstream verifier work: a supported
wasm crypto backend selected behind cfg/feature boundaries, followed by JS-fetch
JWKS/WebID adapters. The v1 demo must use an explicitly documented anonymous or
static-auth stub.

The in-memory SPARQ engine path is independently wasm-proven by the existing
`sparq-wasm` crate and its Node wasm tests, which construct a graph, load RDF,
and execute SPARQL. No mmap or blocking adapter is needed in wasm. For the first
LWS demo, `InMemorySparqClient` remains the smaller path; an in-memory `Graph`
backend is feasible after removing the current `mmap` and `spawn_blocking`
wiring.

## Exact v1 exclusions

The wasm feature implemented by `sq-6xasp.2` must leave these out of the wasm
graph or replace them with the noted seam:

- native listener and transport: binary `main`, TLS, `axum-server`, rustls,
  tokio-rustls, native Hyper client/server utilities, socket tuning, filesystem,
  signals, and Tokio net/multi-thread runtime features;
- crypto-coupled server surfaces: `aws-lc-rs`, TLS, notifications, and PoP-SK;
- live auth networking: verifier `network`, reqwest, hickory-resolver, and the
  current verifier core until its unconditional AWS-LC backend is replaced;
- non-memory backends and background tasks: `object_store`, HTTP SPARQ, Redis
  replay, mimalloc, periodic reconciler spawning, `spawn_blocking`, and mmap;
- the wasm v1 substitutes: in-memory SPARQ/blob stores, no listener, no TLS,
  no notifications, no PoP, and an honest anonymous/static auth stub.

`body_limit`, LDP handlers, WAC logic, axum routing, and the synchronous portions
of the store remain candidates for the portable core. Phase 2 must confirm the
final module cfg boundary against the real crate; this spike does not pre-empt
that work.

## Reproduce

Prerequisites are the repository Rust toolchain, the
`wasm32-unknown-unknown` target, Node, and `wasm-pack`.

```sh
python3 research/wasm-solid-server-spike/verify.py
```

The script requires all positive router gates to pass. It also requires the
isolated no-network verifier build to fail after reaching `aws-lc-sys`, and
checks that the verifier graph has no network feature dependencies. The
negative build failure is expected evidence, not a green compilation gate.

The positive proof can also be inspected separately:

```sh
cargo test \
  --manifest-path research/wasm-solid-server-spike/router-probe/Cargo.toml
wasm-pack test --node \
  research/wasm-solid-server-spike/router-probe
```
