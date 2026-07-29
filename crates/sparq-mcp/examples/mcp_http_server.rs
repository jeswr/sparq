//! [SONNET-4.6] (sq-2c0f0, gh #3221) Serve a dataset as MCP agent tools over the
//! **Streamable HTTP** transport (feature `http`) — the remote/multiplexed counterpart
//! to the stdio binary.
//!
//! ```sh
//! cargo run -p sparq-mcp --features http --example mcp_http_server -- [DATA.ttl] [ADDR]
//! ```
//!
//! `ADDR` defaults to `127.0.0.1:7332` — **loopback on purpose**. This transport has no
//! authentication: `Origin` validation (on by default, and it refuses every request that
//! carries an `Origin` at all) is a DNS-rebinding defence, not an access control. Anyone
//! who can open a TCP connection to the listener has exactly the access the `McpServer`
//! was configured with, which here is read-only.
//!
//! Drive it with any MCP client, or by hand:
//!
//! ```sh
//! curl -sD- http://127.0.0.1:7332/mcp -H 'content-type: application/json' \
//!   -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
//! # then reuse the Mcp-Session-Id header it returns:
//! curl -s http://127.0.0.1:7332/mcp -H 'content-type: application/json' \
//!   -H "mcp-session-id: $SESSION" \
//!   -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
//! ```

use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use sparq_core::Graph;
use sparq_mcp::http::{serve_http, HttpConfig, HttpTransport};
use sparq_mcp::McpServer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let data_file = args.next();
    let address = args.next().unwrap_or_else(|| "127.0.0.1:7332".to_string());

    let graph = match &data_file {
        Some(path) => Graph::load_str(&std::fs::read_to_string(path)?, "turtle")?,
        None => Graph::load_str("", "turtle")?,
    };
    eprintln!("serving {} triples", graph.len());

    // Read-only: `ServerConfig::allow_update` stays false, so the `update` tool is
    // neither advertised nor callable.
    let server = Arc::new(Mutex::new(McpServer::new(graph)));
    let transport = Arc::new(HttpTransport::new(server, HttpConfig::default()));

    let listener = TcpListener::bind(&address)?;
    eprintln!(
        "sparq-mcp streamable HTTP on http://{}{} (read-only, no authentication)",
        listener.local_addr()?,
        transport.config().endpoint
    );
    serve_http(listener, transport)?;
    Ok(())
}
