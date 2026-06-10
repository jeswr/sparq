//! `sparq-server` binary: load an RDF file into the engine and expose it over HTTP per the
//! W3C SPARQL 1.1 Protocol + Graph Store HTTP Protocol (read side).
//!
//! Usage:
//!   sparq-server [--addr 127.0.0.1:3030] [--format turtle]
//!                [--query-timeout SECS] [--max-body-bytes N] [--max-concurrent N]
//!                [--max-results N] [--verbose] [DATA_FILE]
//!
//! With no DATA_FILE the server starts with an empty default graph (still answers queries —
//! they just return no rows). The format defaults to `turtle`; pass `--format ntriples |
//! nquads | trig` to match the file.
//!
//! Hardening flags (T15) — defaults in brackets; each flag overrides the matching
//! `SPARQ_*` environment variable (see crates/sparq-server/README.md):
//!   --query-timeout SECS   per-request query timeout, 0 disables   [30, env SPARQ_QUERY_TIMEOUT]
//!   --max-body-bytes N     maximum request body in bytes           [1048576, env SPARQ_MAX_BODY_BYTES]
//!   --max-concurrent N     maximum in-flight requests (429 beyond) [32, env SPARQ_MAX_CONCURRENT]
//!   --max-results N        maximum SELECT rows (413 beyond), 0 off [unlimited, env SPARQ_MAX_RESULTS]
//!   --verbose              per-request logging (TraceLayer)

use std::net::SocketAddr;
use std::time::Duration;

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};

// Same allocator as the CLI (T1.0a, measured ~1.29x on the parallel join): the system allocator's
// arena locks contend under rayon's per-row allocation, and the server is the long-running,
// concurrent-query process where that matters most.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut addr = "127.0.0.1:3030".to_string();
    let mut format = "turtle".to_string();
    let mut data_file: Option<String> = None;
    // Env first, flags override.
    let mut config = ServerConfig::from_env();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--addr" => addr = args.next().ok_or("--addr requires a value")?,
            "--format" => format = args.next().ok_or("--format requires a value")?,
            "--query-timeout" => {
                let secs: u64 = parse_flag(&mut args, "--query-timeout")?;
                config.query_timeout = (secs > 0).then(|| Duration::from_secs(secs));
            }
            "--max-body-bytes" => config.max_body_bytes = parse_flag(&mut args, "--max-body-bytes")?,
            "--max-concurrent" => {
                let n: usize = parse_flag(&mut args, "--max-concurrent")?;
                config.max_concurrent = n.max(1);
            }
            "--max-results" => {
                let n: usize = parse_flag(&mut args, "--max-results")?;
                config.max_results = (n > 0).then_some(n);
            }
            "--verbose" => config.verbose = true,
            "-h" | "--help" => {
                eprintln!(
                    "usage: sparq-server [--addr HOST:PORT] [--format FMT] [--query-timeout SECS] \
                     [--max-body-bytes N] [--max-concurrent N] [--max-results N] [--verbose] [DATA_FILE]"
                );
                return Ok(());
            }
            other => data_file = Some(other.to_string()),
        }
    }

    if config.verbose {
        // Default to request-level tracing; RUST_LOG still wins if set.
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "tower_http=debug,sparq_server=debug".into()),
            )
            .init();
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
    eprintln!(
        "guards: query-timeout={} max-body-bytes={} max-concurrent={} max-results={}",
        config.query_timeout.map_or("off".into(), |t| format!("{}s", t.as_secs())),
        config.max_body_bytes,
        config.max_concurrent,
        config.max_results.map_or("off".into(), |n| n.to_string()),
    );

    let state = AppState::with_config(graph, config);
    let app = router(state);

    let addr: SocketAddr = addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("sparq-server listening on http://{addr}  (SPARQL endpoint: /sparql)");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn parse_flag<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    let v = args.next().ok_or_else(|| format!("{flag} requires a value"))?;
    v.parse().map_err(|_| format!("{flag}: invalid value '{v}'"))
}

/// Resolves on SIGINT (Ctrl-C) or SIGTERM (the signal init systems / container runtimes
/// send), letting `axum::serve` drain in-flight requests before the process exits.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
    eprintln!("shutting down");
}
