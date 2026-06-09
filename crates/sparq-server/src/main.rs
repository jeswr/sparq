//! `sparq-server` binary: load an RDF file into the engine and expose it over HTTP per the
//! W3C SPARQL 1.1 Protocol + Graph Store HTTP Protocol (read side).
//!
//! Usage:
//!   sparq-server [--addr 127.0.0.1:3030] [--format turtle] [DATA_FILE]
//!
//! With no DATA_FILE the server starts with an empty default graph (still answers queries —
//! they just return no rows). The format defaults to `turtle`; pass `--format ntriples |
//! nquads | trig` to match the file.

use std::net::SocketAddr;

use sparq_core::Graph;
use sparq_server::{router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut addr = "127.0.0.1:3030".to_string();
    let mut format = "turtle".to_string();
    let mut data_file: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--addr" => addr = args.next().ok_or("--addr requires a value")?,
            "--format" => format = args.next().ok_or("--format requires a value")?,
            "-h" | "--help" => {
                eprintln!("usage: sparq-server [--addr HOST:PORT] [--format FMT] [DATA_FILE]");
                return Ok(());
            }
            other => data_file = Some(other.to_string()),
        }
    }

    let graph = match &data_file {
        Some(path) => {
            let text = std::fs::read_to_string(path)?;
            eprintln!("loading {path} ({format}) ...");
            Graph::load_str(&text, &format).map_err(|e| format!("failed to load {path}: {e}"))?
        }
        None => {
            eprintln!("no data file given — starting with an empty default graph");
            Graph::load_str("", "turtle").map_err(|e| format!("init: {e}"))?
        }
    };
    eprintln!("loaded {} triples", graph.len());

    let state = AppState::new(graph);
    let app = router(state);

    let addr: SocketAddr = addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("sparq-server listening on http://{addr}  (SPARQL endpoint: /sparql)");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    eprintln!("shutting down");
}
