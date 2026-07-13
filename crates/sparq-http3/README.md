<!-- [GPT-5.6] sq-oprna.1: internal-stub README for a publish=false crate. -->
# sparq-http3

Internal, unstable HTTP/3-over-QUIC transport bridge shared by sparq's axum servers.
Its default-off `server` feature contains the pre-1.0 `h3`/`h3-quinn` stack and Quinn,
so a default workspace build does not pull those dependencies.

Enable `server` to convert an aws-lc-rs-backed rustls server configuration and dispatch
HTTP/3 requests into a cloned `axum::Router`. The bridge injects the QUIC peer address as
`ConnectInfo<SocketAddr>` and streams request and response bodies. Its compatibility entry point is
safe by default: `serve_h3` applies global and per-IP concurrent-connection caps, while
`serve_h3_with_limits` accepts custom `H3ConnectionLimits`. Internal IPs are exempt from the per-IP
default but remain covered by the global cap. The generated Quinn configuration also uses a bounded
connection receive window.

WebSocket-over-HTTP/3 is outside this helper's scope; clients use the existing HTTP/1.1 or
HTTP/2 listener for WebSocket upgrades.

See `skills/http3-server/SKILL.md` for the embedding API and
`research/http3-quic-servers-design.md` for the transport architecture.

## License

MIT.
