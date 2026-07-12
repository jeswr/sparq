---
name: http3-server
description: Embed sparq's internal opt-in HTTP/3-over-QUIC bridge into an axum server with rustls 0.23/aws-lc-rs, Quinn, peer ConnectInfo injection, and graceful shutdown.
license: MIT
metadata:
  version: "0.1.0"
  homepage: https://github.com/jeswr/sparq
---

# sparq HTTP/3 server bridge

Use `sparq-http3` when a sparq HTTP server needs a second, encrypted UDP listener that
dispatches into the same `axum::Router` as its existing HTTP/1.1 or HTTP/2 listener. The
crate is internal and unstable (`publish = false`), and its `server` feature is off by
default so Quinn and the pre-1.0 h3 stack do not enter ordinary workspace builds.

## Add the opt-in dependency

```toml
[features]
default = []
http3 = ["dep:sparq-http3"]

[dependencies]
sparq-http3 = { path = "../sparq-http3", optional = true, features = ["server"] }
```

The downstream crate owns certificate/key loading and runtime configuration. Build a
`rustls::ServerConfig` with the aws-lc-rs provider, TLS 1.3, and the exact `h3` ALPN token:

```rust
use std::sync::Arc;

let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
let mut tls = rustls::ServerConfig::builder_with_provider(provider)
    .with_protocol_versions(&[&rustls::version::TLS13])?
    .with_no_client_auth()
    .with_single_cert(certificates, private_key)?;
tls.alpn_protocols = vec![b"h3".to_vec()];

let quic = sparq_http3::quic_server_config(tls)?;
let endpoint = quinn::Endpoint::server(quic, udp_addr)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`quic_server_config` fails closed when `h3` is missing from ALPN or the provider lacks
QUIC's required initial cipher suite.

## Serve the shared router

```rust
sparq_http3::serve_h3(endpoint, router.clone(), async move {
    shutdown_signal.await;
})
.await?;
# Ok::<(), std::io::Error>(())
```

The bridge clones the router per connection and request, streams request and response
bodies, and inserts the Quinn peer address as `axum::extract::ConnectInfo<SocketAddr>`.
That extension is load-bearing for request policies keyed by the remote socket address.
Protocol failures are isolated to their connection or stream; resolving the shutdown
future closes the endpoint and waits for its QUIC connections to drain.

## Boundaries

- The caller owns rustls policy, certificates, client authentication, Quinn transport
  tuning, listener binding, and process-wide provider installation.
- HTTP/3 is a separate UDP listener. Keep the existing TCP listener running for HTTP/1.1
  and HTTP/2 clients.
- `h3` and `h3-quinn` are pre-1.0 dependencies; keep all direct use inside this helper.
- WebSocket-over-HTTP/3 extended CONNECT is not implemented. WebSocket clients fall back
  to the existing TCP listener.
- Do not advertise `Alt-Svc` until the UDP listener has successfully bound.

## Related material

- `crates/sparq-http3/README.md` — crate scope and opt-in posture.
- `research/http3-quic-servers-design.md` — shared-listener architecture and maturity
  boundary.
- `skills/http-server/SKILL.md` — the SPARQL Protocol server surface.
