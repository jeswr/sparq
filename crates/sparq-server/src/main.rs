//! `sparq-server` binary: load an RDF file into the engine and expose it over HTTP per the
//! W3C SPARQL 1.1 Protocol + Graph Store HTTP Protocol (read side).
//!
//! Usage:
//!   sparq-server [--addr 127.0.0.1:3030] [--allow-remote] [--format turtle]
//!                [--auth-token TOKEN] [--auth-token-read]
//!                [--query-timeout SECS] [--max-body-bytes N] [--max-concurrent N]
//!                [--max-results N] [--max-subscriptions N]
//!                [--max-subscriptions-per-conn N]
//!                [--service-allow HOST|*.SUFFIX]... [--service-allow-file PATH]
//!                [--verbose] [DATA_FILE]
//!
//! SERVICE federation (`service` build feature) is DENY-ALL by default: a `SERVICE <iri>`
//! clause reaches NOTHING unless its host is allowlisted via `--service-allow` (repeatable;
//! exact host or `*.suffix` wildcard), `--service-allow-file` (one entry per line) or the
//! `SPARQ_SERVICE_ALLOW` env var (comma/whitespace-separated). This is an SSRF guard: a
//! `SERVICE` clause turns attacker-controlled query text into an outbound request from this
//! host. See crates/sparq-server/README.md -> "SERVICE federation (egress allowlist)". [OPUS-4.8]
//!
//! SECURITY (auth): by default the server has NO authentication on any endpoint (query, the
//! `application/sparql-update` mutation path, the `/subscriptions` WebSocket). [OPUS-4.8]
//! sq-zcby (PSS gh-46): set `--auth-token TOKEN` (env `SPARQ_AUTH_TOKEN`) to require
//! `Authorization: Bearer TOKEN` on every WRITE (a SPARQL Update on `/sparql` — by
//! `application/sparql-update` Content-Type OR a `query`/`update` body that parses as an
//! update; classification keys on whether the request MUTATES, not the route — plus the GSP
//! `PUT`/`POST`/`DELETE` methods); otherwise `401` with `WWW-Authenticate: Bearer`
//! (constant-time compared, scheme-casing tolerant; mirrors QLever's `-a <token>`). Add
//! `--auth-token-read` (env `SPARQ_AUTH_TOKEN_READ=1`) to ALSO gate reads. The
//! `/subscriptions` WebSocket (a read surface) is NOT gated by this token.
//!
//! SECURITY (bind): `--addr` defaults to loopback (127.0.0.1:3030), reachable only from this
//! host. A NON-loopback bind (e.g. `0.0.0.0`) is REFUSED unless `--allow-remote` (env
//! `SPARQ_ALLOW_REMOTE=1`) is set OR the whole surface is authenticated (`--auth-token` AND
//! `--auth-token-read`) — a write-token alone still leaves reads open, so it is not
//! sufficient by itself. Even an allowed remote bind logs a warning. Deliver the token over
//! TLS (terminate at a proxy); for per-user authz front it with a reverse proxy / gateway
//! (or sparq-solid). See crates/sparq-server/README.md → "Security posture".
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
//!
//! Subscription limits (T23, the /subscriptions WebSocket — see SUBSCRIPTIONS.md):
//!   --max-subscriptions N           server-wide active subscriptions [256, env SPARQ_MAX_SUBSCRIPTIONS]
//!   --max-subscriptions-per-conn N  active subscriptions per socket  [16, env SPARQ_MAX_SUBSCRIPTIONS_PER_CONN]
//!
//! Time-travel retention (opt-in `time-travel` cargo feature — see the README's
//! "Time-travel queries" section; each retained generation is a FULL graph today):
//!   --time-travel-generations N  queryable generations older than current [16, env SPARQ_TIME_TRAVEL_GENERATIONS]
//!   --time-travel-max-age SECS   age generations out after SECS, 0 off    [off, env SPARQ_TIME_TRAVEL_MAX_AGE]
//!
//! Update path (Wave A wiring — see crates/sparq-server/README.md "Update concurrency
//! model"): sparq-serve's sequenced group-commit writer over the generation ring. The
//! old `--compact-every` flag is gone — every batch commit forks a freshly folded base,
//! so overlays never accumulate across batches and there is nothing left to compact.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use std::net::SocketAddr;
use std::time::Duration;

use sparq_core::Graph;
// [OPUS-4.8] sq-o4qf: bind_posture / BindPosture gate the non-loopback bind; sq-zcby:
// AuthPosture folds the --auth-token Bearer gate into that bind decision.
use sparq_server::{bind_posture, router, AppState, AuthPosture, BindPosture, ServerConfig};

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
    // Env first, flags override. [OPUS-4.8] sq-4w18: a malformed SPARQ_SERVICE_ALLOW
    // surfaces here as a clean config error (the `?` turns the String into the boxed
    // error main returns -> a one-line message to stderr + non-zero exit), not a panic.
    let mut config = ServerConfig::from_env()?;
    // [OPUS-4.8] sq-4w18: collect SERVICE egress allowlist entries from the CLI; they
    // are UNIONed with the SPARQ_SERVICE_ALLOW env baseline (already in `config`) + an
    // optional --service-allow-file, after the arg loop. An allowlist is additive, so
    // the CLI only ever widens what env granted (never silently narrows it).
    let mut service_allow_cli: Vec<String> = Vec::new();
    let mut service_allow_file: Option<String> = None;

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
            "--max-subscriptions" => {
                config.max_subscriptions = parse_flag(&mut args, "--max-subscriptions")?;
            }
            "--max-subscriptions-per-conn" => {
                config.max_subscriptions_per_conn = parse_flag(&mut args, "--max-subscriptions-per-conn")?;
            }
            #[cfg(feature = "time-travel")]
            "--time-travel-generations" => {
                config.time_travel_generations = parse_flag(&mut args, "--time-travel-generations")?;
            }
            #[cfg(feature = "time-travel")]
            "--time-travel-max-age" => {
                let secs: u64 = parse_flag(&mut args, "--time-travel-max-age")?;
                config.time_travel_max_age = (secs > 0).then(|| Duration::from_secs(secs));
            }
            "--verbose" => config.verbose = true,
            // [OPUS-4.8] sq-o4qf: opt in to binding a non-loopback address. Without this (and
            // without SPARQ_ALLOW_REMOTE), a non-loopback --addr is refused unless the whole
            // surface is authenticated (--auth-token AND --auth-token-read) — see bind_posture.
            "--allow-remote" => config.allow_remote = true,
            // [OPUS-4.8] sq-zcby (PSS gh-46): require a Bearer token on the WRITE surface.
            // An empty token is rejected (an empty shared secret is a footgun, never valid).
            "--auth-token" => {
                let tok = args.next().ok_or("--auth-token requires a token value")?;
                if tok.is_empty() {
                    return Err("--auth-token must not be empty".into());
                }
                config.auth_token = Some(tok);
            }
            // [OPUS-4.8] sq-zcby: ALSO gate reads with the same token (QLever-style; only
            // meaningful alongside --auth-token).
            "--auth-token-read" => config.auth_token_read = true,
            // [OPUS-4.8] sq-4w18: SERVICE egress allowlist. Repeatable: each value adds one
            // host (`sparql.example.org`) or suffix wildcard (`*.example.org`). With NO
            // allowlist (the default) every SERVICE clause is refused (default-DENY-all).
            "--service-allow" => {
                service_allow_cli.push(args.next().ok_or("--service-allow requires a host/pattern")?);
            }
            "--service-allow-file" => {
                service_allow_file = Some(args.next().ok_or("--service-allow-file requires a path")?);
            }
            "-h" | "--help" => {
                let time_travel = if cfg!(feature = "time-travel") {
                    " [--time-travel-generations N] [--time-travel-max-age SECS]"
                } else {
                    ""
                };
                let service = if cfg!(feature = "service") {
                    " [--service-allow HOST|*.SUFFIX]... [--service-allow-file PATH]"
                } else {
                    ""
                };
                eprintln!(
                    "usage: sparq-server [--addr HOST:PORT] [--allow-remote] \
                     [--auth-token TOKEN] [--auth-token-read] [--format FMT] \
                     [--query-timeout SECS] [--max-body-bytes N] [--max-concurrent N] \
                     [--max-results N] [--max-subscriptions N] \
                     [--max-subscriptions-per-conn N]{time_travel}{service} [--verbose] [DATA_FILE]\n\n  \
                     AUTH: --auth-token TOKEN (env SPARQ_AUTH_TOKEN) requires \
                     'Authorization: Bearer TOKEN' on every WRITE (SPARQL Update + GSP \
                     PUT/POST/DELETE) -> 401 + 'WWW-Authenticate: Bearer' otherwise \
                     (constant-time compared; QLever's -a). Add --auth-token-read (env \
                     SPARQ_AUTH_TOKEN_READ=1) to ALSO gate reads. Unset = no auth.\n  \
                     SECURITY (bind): --addr defaults to loopback (127.0.0.1); a non-loopback \
                     bind (e.g. 0.0.0.0) is REFUSED unless --allow-remote (or \
                     SPARQ_ALLOW_REMOTE=1) is set OR the whole surface is authenticated \
                     (--auth-token AND --auth-token-read) — a write-token alone still leaves \
                     reads open. Put it behind a reverse proxy / gateway that enforces auth.\n  \
                     SERVICE federation (the `service` build feature) is DENY-ALL by default: \
                     SERVICE <iri> reaches NOTHING unless the host is allowlisted via \
                     --service-allow / --service-allow-file / SPARQ_SERVICE_ALLOW \
                     (exact host or *.suffix wildcard) — an SSRF guard."
                );
                return Ok(());
            }
            other => data_file = Some(other.to_string()),
        }
    }

    // [OPUS-4.8] sq-4w18: merge the --service-allow-file + --service-allow CLI entries
    // INTO the env baseline already in config.service_allow (union — additive). A
    // malformed entry or unreadable file is a hard startup error: better to refuse to
    // boot than to silently run with an allowlist narrower than the operator wrote.
    if let Some(path) = &service_allow_file {
        let text = std::fs::read_to_string(path).map_err(|e| format!("--service-allow-file {path}: {e}"))?;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            config.service_allow.add(line)?;
        }
    }
    for entry in &service_allow_cli {
        config.service_allow.add(entry)?;
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
        "guards: query-timeout={} max-body-bytes={} max-concurrent={} max-results={} \
         max-subscriptions={} max-subscriptions-per-conn={}",
        config.query_timeout.map_or("off".into(), |t| format!("{}s", t.as_secs())),
        config.max_body_bytes,
        config.max_concurrent,
        config.max_results.map_or("off".into(), |n| n.to_string()),
        config.max_subscriptions,
        config.max_subscriptions_per_conn,
    );
    // [OPUS-4.8] sq-4w18: surface the SERVICE egress posture at startup so an operator
    // sees whether (and where) federation can reach. Only meaningful with the `service`
    // build feature; without it a SERVICE clause errors at execution regardless.
    #[cfg(feature = "service")]
    eprintln!("SERVICE egress allowlist: {}", config.service_allow.display());
    #[cfg(feature = "time-travel")]
    eprintln!(
        "time travel: ?generation=N enabled — generations={} max-age={} (each retained generation is a full graph)",
        config.time_travel_generations,
        config.time_travel_max_age.map_or("off".into(), |t| format!("{}s", t.as_secs())),
    );

    // [OPUS-4.8] sq-zcby: surface the auth posture at startup. A write-only token (no
    // --auth-token-read) leaves reads OPEN — warn so the operator is not surprised; a fully
    // gated surface is reported too. No token = silent (the bind posture already warns on a
    // non-loopback bind without auth).
    let auth = AuthPosture::from_config(&config);
    match auth {
        AuthPosture::None => {}
        AuthPosture::WriteOnly => eprintln!(
            "auth: --auth-token set — WRITES require 'Authorization: Bearer <token>'; READS \
             remain OPEN (add --auth-token-read to gate reads too). The /subscriptions \
             WebSocket is a read surface and is NOT gated by this token."
        ),
        AuthPosture::ReadAndWrite => eprintln!(
            "auth: --auth-token + --auth-token-read set — the whole surface (reads AND writes) \
             requires 'Authorization: Bearer <token>'. Note: the /subscriptions WebSocket is \
             NOT gated by this token; deliver the token over TLS."
        ),
    }

    let addr: SocketAddr = addr.parse()?;
    // [OPUS-4.8] sq-o4qf / sq-zcby: enforce the bind posture BEFORE binding. A non-loopback
    // address is refused unless explicitly opted into (--allow-remote / SPARQ_ALLOW_REMOTE)
    // OR the whole surface is authenticated (--auth-token AND --auth-token-read). A
    // write-token alone still leaves reads open, so it still requires --allow-remote.
    match bind_posture(&addr, config.allow_remote, auth) {
        BindPosture::Loopback => {}
        BindPosture::RemoteAllowed { warning } => eprintln!("{warning}"),
        BindPosture::RemoteRefused { message } => return Err(message.into()),
    }

    let state = AppState::with_config(graph, config);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("sparq-server listening on http://{addr}  (SPARQL endpoint: /sparql, subscriptions: ws://{addr}/subscriptions)");
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
