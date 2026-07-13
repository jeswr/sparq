//! Opt-in HTTP/3 transport plumbing for sparq's axum servers.
//!
//! The crate is intentionally empty unless its default-off `server` feature is enabled.
//! That feature contains the pre-1.0 h3 stack in one internal crate and exposes the
//! rustls-to-Quinn configuration bridge plus the axum request-dispatch loop.
#![forbid(unsafe_code)]

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
pub use server::{
    alt_svc_layer, quic_server_config, serve_h3, serve_h3_with_limits, H3ConnectionLimits,
    Http3ConfigError,
};
