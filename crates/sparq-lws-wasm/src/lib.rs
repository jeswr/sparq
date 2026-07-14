// [GPT-5.6] sq-6xasp.3: wasm host adapter over the real LWS router.
// [SONNET-4.6] sq-250si: wasm panic hook + host trap-recovery boundary.
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
//! WebAssembly request entry for the in-memory Solid/LDP server core.
//!
//! Building this dedicated crate is the opt-in boundary. On wasm32, `SolidServer` owns the real axum
//! router over the in-memory SPARQ metadata and blob stores. JavaScript remains responsible for the
//! HTTP listener and OIDC verification; each request supplies only an already-authenticated WebID.
//!
//! # Panic and trap behaviour
//!
//! The release profile uses `panic=abort`, so a Rust panic lowers to a `wasm32` `unreachable`
//! trap rather than unwinding. The `lws_init` function registered via `#[wasm_bindgen(start)]`
//! installs a `console_error_panic_hook` that forwards the panic message to `console.error`
//! before the trap fires — giving the host a readable diagnostic in environments where the
//! hook can run (debug builds, `wasm-bindgen-test`). In `panic=abort` release builds the hook
//! fires before the abort and the message appears in the browser or Node console.
//!
//! The Node host (`@jeswr/solid-server`) catches the resulting `WebAssembly.RuntimeError` and
//! recreates the `SolidServer` before the next request, so a single trap does not permanently
//! brick the process. See `packages/solid-server/src/index.js`.

#[cfg(target_arch = "wasm32")]
mod wasm {
    use axum::body::{to_bytes, Body, Bytes};
    use http::header::{HeaderName, HeaderValue};
    use http::{Method, Request, Uri};
    use oxrdf::{NamedNode, Triple};
    use sparq_lws_core::auth::VerifiedToken;
    use sparq_lws_core::ldp::content::{serialize_triples, RdfFormat};
    use sparq_lws_core::ldp::handler::LdpState;
    use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient, Store};
    use sparq_lws_core::{build_router, AppState};
    use tower::ServiceExt;
    use wasm_bindgen::prelude::*;

    type MemoryStore = CompositeStore<InMemorySparqClient, InMemoryBlobStore>;

    const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    const ACL: &str = "http://www.w3.org/ns/auth/acl#";

    /// Called once by the wasm-bindgen runtime when the module is first loaded.
    ///
    /// Installs a panic hook that writes the panic message to `console.error` before the
    /// `unreachable` trap propagates to the host as a `WebAssembly.RuntimeError`. The hook
    /// is idempotent — safe to call from any context, and a no-op if already installed.
    // [SONNET-4.6] sq-250si: set_once is idempotent; safe for concurrent (future) init paths.
    #[wasm_bindgen(start)]
    pub fn lws_init() {
        console_error_panic_hook::set_once();
    }

    /// A response returned across the wasm boundary.
    ///
    /// Headers are a flat `[name, value, name, value, ...]` array. Repeated header names remain
    /// repeated, which lets a Node host reconstruct the original HTTP response without joining them.
    #[wasm_bindgen]
    pub struct LwsResponse {
        status: u16,
        headers: Vec<String>,
        body: Vec<u8>,
    }

    #[wasm_bindgen]
    impl LwsResponse {
        /// Numeric HTTP status code.
        #[wasm_bindgen(getter)]
        pub fn status(&self) -> u16 {
            self.status
        }

        /// Flat name/value response-header pairs.
        #[wasm_bindgen(getter)]
        pub fn headers(&self) -> Vec<String> {
            self.headers.clone()
        }

        /// Complete response bytes.
        #[wasm_bindgen(getter)]
        pub fn body(&self) -> Vec<u8> {
            self.body.clone()
        }
    }

    /// A single-threaded, in-memory Solid/LDP request handler.
    ///
    /// The owner WebID passed to the constructor receives Read, Write, and Control on the storage
    /// root and descendants. That is initial provisioning, not OIDC verification. The JavaScript
    /// host verifies every real credential and passes the resulting WebID to `handleRequest`.
    #[wasm_bindgen]
    pub struct SolidServer {
        router: axum::Router,
    }

    #[wasm_bindgen]
    impl SolidServer {
        /// Create an isolated in-memory pod and provision its owner ACL.
        #[wasm_bindgen(constructor)]
        pub fn new(base_url: String, owner_webid: String) -> Result<SolidServer, JsError> {
            let base_url = base_url.trim_end_matches('/').to_owned();
            if base_url.is_empty() {
                return Err(JsError::new("base_url must not be empty"));
            }

            let store = MemoryStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
            futures_executor::block_on(seed_owner_acl(&store, &base_url, &owner_webid))
                .map_err(|error| JsError::new(&error))?;
            let ldp = LdpState::new(store, base_url);
            Ok(Self {
                router: build_router(AppState::new(ldp)),
            })
        }

        /// Drive one request through the real axum router and its LDP, WAC, CORS, and body-limit
        /// layers.
        ///
        /// `headers` is a flat `[name, value, ...]` array and must contain an even number of strings.
        /// `authenticated_webid` is trusted only because the JavaScript host is responsible for OIDC;
        /// omit it for an anonymous request. The returned Promise needs no Tokio reactor.
        #[wasm_bindgen(js_name = handleRequest)]
        pub async fn handle_request(
            &self,
            method: String,
            path: String,
            headers: Vec<String>,
            body: Vec<u8>,
            authenticated_webid: Option<String>,
        ) -> Result<LwsResponse, JsError> {
            let request = build_request(method, path, headers, body, authenticated_webid)
                .map_err(|error| JsError::new(&error))?;
            let response = self
                .router
                .clone()
                .oneshot(request)
                .await
                .map_err(|error| JsError::new(&error.to_string()))?;

            let status = response.status().as_u16();
            let headers = flatten_headers(response.headers())?;
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .map_err(|error| JsError::new(&error.to_string()))?
                .to_vec();
            Ok(LwsResponse {
                status,
                headers,
                body,
            })
        }
    }

    fn build_request(
        method: String,
        path: String,
        headers: Vec<String>,
        body: Vec<u8>,
        authenticated_webid: Option<String>,
    ) -> Result<Request<Body>, String> {
        if !headers.len().is_multiple_of(2) {
            return Err("headers must contain name/value pairs".to_owned());
        }

        let method = Method::from_bytes(method.as_bytes())
            .map_err(|error| format!("invalid HTTP method: {}", error))?;
        let uri: Uri = path
            .parse()
            .map_err(|error| format!("invalid request path: {}", error))?;
        if uri.scheme().is_some() || uri.authority().is_some() || !uri.path().starts_with('/') {
            return Err("path must be an origin-form URI beginning with '/'".to_owned());
        }

        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::from(body))
            .map_err(|error| format!("could not build request: {}", error))?;
        for pair in headers.chunks_exact(2) {
            let name = HeaderName::from_bytes(pair[0].as_bytes())
                .map_err(|error| format!("invalid header name {:?}: {}", pair[0], error))?;
            let value = HeaderValue::from_str(&pair[1])
                .map_err(|error| format!("invalid value for header {}: {}", pair[0], error))?;
            request.headers_mut().append(name, value);
        }
        if let Some(webid) = authenticated_webid {
            request
                .extensions_mut()
                .insert(VerifiedToken::from_authenticated_webid(webid));
        }
        Ok(request)
    }

    fn flatten_headers(headers: &http::HeaderMap) -> Result<Vec<String>, JsError> {
        let mut flat = Vec::with_capacity(headers.len() * 2);
        for (name, value) in headers {
            flat.push(name.as_str().to_owned());
            flat.push(
                value
                    .to_str()
                    .map_err(|_| JsError::new("response contains a non-text header value"))?
                    .to_owned(),
            );
        }
        Ok(flat)
    }

    async fn seed_owner_acl(
        store: &MemoryStore,
        base_url: &str,
        owner_webid: &str,
    ) -> Result<(), String> {
        let root = format!("{}/", base_url);
        let acl_doc = format!("{}.acl", root);
        let auth = named_node(&format!("{}#owner", acl_doc))?;
        let root_node = named_node(&root)?;
        let mode = |local: &str| named_node(&format!("{}{}", ACL, local));
        let triples = vec![
            Triple::new(auth.clone(), named_node(RDF_TYPE)?, mode("Authorization")?),
            Triple::new(auth.clone(), mode("agent")?, named_node(owner_webid)?),
            Triple::new(auth.clone(), mode("accessTo")?, root_node.clone()),
            Triple::new(auth.clone(), mode("default")?, root_node),
            Triple::new(auth.clone(), mode("mode")?, mode("Read")?),
            Triple::new(auth.clone(), mode("mode")?, mode("Write")?),
            Triple::new(auth, mode("mode")?, mode("Control")?),
        ];
        let bytes = serialize_triples(RdfFormat::Turtle, &triples)
            .map_err(|error| format!("could not serialize owner ACL: {}", error))?;
        store
            .write(&acl_doc, Bytes::from(bytes), RdfFormat::Turtle.media_type())
            .await
            .map_err(|error| format!("could not provision owner ACL: {}", error))?;
        Ok(())
    }

    fn named_node(iri: &str) -> Result<NamedNode, String> {
        NamedNode::new(iri).map_err(|error| format!("invalid IRI {:?}: {}", iri, error))
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{LwsResponse, SolidServer};
