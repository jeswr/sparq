// AUTHORED-BY Claude Fable 5
//! `TCP_NODELAY` (Nagle-off) on the accepted sockets of BOTH serve paths — beyond-50k P1.5.
//!
//! ## Why
//! Nagle's algorithm (RFC 896) coalesces small outgoing TCP segments, waiting up to a round-trip
//! (or an ACK) before sending a sub-MSS write. For a request/response server emitting small
//! responses (a few hundred bytes of RDF — the measured hot path), that adds latency for no
//! benefit: the response is one logical write that should leave immediately. Disabling Nagle
//! (`TCP_NODELAY`) is the standard production server default. It is purely a TRANSPORT latency
//! knob — it changes WHEN bytes leave the socket, never the LDP/auth/WAC semantics of the request,
//! so conformance is unaffected. The effect on wall-clock latency is ADVISORY per the perf-gate
//! rule; the DETERMINISTIC assertion is the socket-option state (`TcpStream::nodelay()` reads back
//! `true`), pinned by `tests/tcp_nodelay.rs` for each path.
//!
//! ## Neither serve path sets it by default (the gap this closes)
//! - **Plain `axum::serve`** does NOT set `TCP_NODELAY` on accepted connections — an operator must
//!   opt in (axum ships the [`axum::serve::ListenerExt::tap_io`] adapter exactly for this). We wrap
//!   the plain listener with [`tap_nodelay`] so every accepted stream gets the option.
//! - **`axum-server`** (the in-process TLS path) also leaves it off by default (its `DefaultAcceptor`
//!   is a pass-through); axum-server ships [`NoDelayAcceptor`] for this. We compose it as the INNER
//!   acceptor of the `RustlsAcceptor`, so the option is set on the raw `TcpStream` BEFORE the TLS
//!   handshake — see `src/main.rs`. Re-exported here so `main` and the test reference the one type.
//!
//! Both mechanisms are the vendored, upstream-tested primitives; this module only WIRES them and
//! makes the option-set an owned, tested invariant of this crate.

use axum::serve::{ListenerExt, TapIo};
use tokio::net::{TcpListener, TcpStream};

/// axum-server's `TCP_NODELAY`-setting acceptor — composed as the INNER acceptor of the TLS path's
/// `RustlsAcceptor` (`acceptor.acceptor(NoDelayAcceptor::new())`), so it sets the option on the raw
/// accepted `TcpStream` before the rustls handshake runs. Re-exported so the serve path and the test
/// name the same type. Upstream-tested; this crate only wires it.
pub use axum_server::accept::NoDelayAcceptor;

/// Set `TCP_NODELAY` on one accepted plain-TCP stream. Best-effort: a failure to set the option is
/// LOGGED (matching `main`'s `eprintln!` startup diagnostics) and the connection is still served —
/// Nagle-on is a latency regression, never a correctness/security one, so it must never drop a
/// connection. This is the closure [`tap_nodelay`] taps onto every accepted stream.
pub fn set_nodelay(stream: &mut TcpStream) {
    if let Err(err) = stream.set_nodelay(true) {
        eprintln!("  WARNING: failed to set TCP_NODELAY on an accepted connection: {err}");
    }
}

/// Wrap a plain-TCP [`TcpListener`] so every accepted stream gets `TCP_NODELAY` (the plain
/// `axum::serve` path). Returns a [`TapIo`] listener that runs [`set_nodelay`] on each accepted
/// `TcpStream`; it still implements [`axum::serve::Listener`] with `Io = TcpStream` / `Addr =
/// SocketAddr`, so it drops straight into `axum::serve(..)` with
/// `into_make_service_with_connect_info::<SocketAddr>()` (the pre-crypto rate limiter's peer IP is
/// preserved). A `fn` pointer (not a closure) is used so the return type is nameable and `main` +
/// the test share the EXACT wrapper.
pub fn tap_nodelay(listener: TcpListener) -> TapIo<TcpListener, fn(&mut TcpStream)> {
    listener.tap_io(set_nodelay as fn(&mut TcpStream))
}
