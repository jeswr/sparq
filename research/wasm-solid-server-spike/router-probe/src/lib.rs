// [GPT-5.6] sq-6xasp.1: isolated compatibility proof; this is not a shipped crate.
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
//! Runtime-free compatibility proof for the wasm Solid-server request seam.
//!
//! This experimental crate intentionally mirrors only the portable shape needed to
//! settle sq-6xasp.1: an async `Store`, an in-memory implementation, an axum
//! `build_router`, and a request driven through `tower::ServiceExt`. It does not
//! claim that the current `sparq-lws-core` dependency graph compiles unchanged.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use bytes::Bytes;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// The wasm-portable subset of the LWS asynchronous storage seam used by this probe.
#[async_trait]
pub trait Store: Send + Sync {
    /// Read a resource body by its router path.
    async fn read(&self, path: &str) -> Option<Bytes>;

    /// Seed a resource body before the request is routed.
    async fn write(&self, path: &str, body: Bytes);
}

/// An in-memory `Store` whose futures need no reactor, socket, timer, or filesystem.
#[derive(Clone, Default)]
pub struct InMemoryStore {
    resources: Arc<Mutex<HashMap<String, Bytes>>>,
}

#[async_trait]
impl Store for InMemoryStore {
    async fn read(&self, path: &str) -> Option<Bytes> {
        self.resources
            .lock()
            .expect("in-memory store mutex poisoned")
            .get(path)
            .cloned()
    }

    async fn write(&self, path: &str, body: Bytes) {
        self.resources
            .lock()
            .expect("in-memory store mutex poisoned")
            .insert(path.to_owned(), body);
    }
}

struct AppState<S: Store> {
    store: Arc<S>,
}

impl<S: Store> Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

/// Build the reduced LDP router without a listener or an ambient runtime.
pub fn build_router<S>(store: S) -> Router
where
    S: Store + 'static,
{
    Router::new()
        .route("/{*path}", get(get_handler::<S>))
        .with_state(AppState {
            store: Arc::new(store),
        })
}

async fn get_handler<S>(
    State(state): State<AppState<S>>,
    Path(path): Path<String>,
) -> Result<Response, StatusCode>
where
    S: Store + 'static,
{
    let body = state.store.read(&path).await.ok_or(StatusCode::NOT_FOUND)?;
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/turtle")
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Instantiate the in-memory store, build the router, and route one LDP GET.
///
/// The exact body assertion makes the proof mutation-witnessed: deleting the seed,
/// bypassing the store, changing the route, or returning a placeholder body fails.
pub async fn route_one_ldp_request() -> Result<String, String> {
    const RESOURCE: &str =
        "<https://example.test/card#me> <http://xmlns.com/foaf/0.1/name> \"Ada\" .\n";

    let store = InMemoryStore::default();
    store
        .write("profile/card", Bytes::from_static(RESOURCE.as_bytes()))
        .await;

    let response = build_router(store)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/profile/card")
                .body(Body::empty())
                .map_err(|error| error.to_string())?,
        )
        .await
        .map_err(|error| error.to_string())?;

    if response.status() != StatusCode::OK {
        return Err(format!("unexpected response status: {}", response.status()));
    }
    if response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        != Some("text/turtle")
    {
        return Err("missing text/turtle response type".to_owned());
    }

    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes();
    if body.as_ref() != RESOURCE.as_bytes() {
        return Err("router did not return the seeded Store body".to_owned());
    }

    String::from_utf8(body.to_vec()).map_err(|error| error.to_string())
}

/// Promise-returning wasm export proving `wasm-bindgen-futures` drives the request future.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = routeOneLdpRequest)]
pub async fn route_one_ldp_request_wasm() -> Result<String, wasm_bindgen::JsValue> {
    route_one_ldp_request()
        .await
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error))
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn routes_one_ldp_request_without_tokio() {
        let body = futures_executor::block_on(super::route_one_ldp_request())
            .expect("the in-memory LDP request must complete");
        assert!(body.contains("foaf/0.1/name"));
        assert!(body.contains("\"Ada\""));
    }
}
