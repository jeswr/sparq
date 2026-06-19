// [OPUS-4.8] sq-2e93 — the native engine command layer for the sparq Tauri GUI.
//
// This is the concrete proof of the design's headline embedding decision
// (research/gui-design.md §1): on desktop the app links `sparq-engine` / `sparq-core`
// DIRECTLY and runs the FULL native store, instead of the WASM read-replica the browser is
// limited to. Each command below maps onto the SAME operation the wasm `Store` exposes
// (crates/sparq-wasm/src/lib.rs) — `load` / `query` / `queryQuads` / `updateInPlace` /
// `explain` / `count` / `ask` — but backed by the native `sparq_core::Graph` (rayon-built
// indexes, no ~2 GiB tab ceiling). The webview frontend (the reused Next.js REPL) calls
// these over Tauri IPC instead of the wasm-bindgen boundary.
//
// State: a single in-process `Graph` behind a `Mutex`, held in Tauri's managed state. This
// is the MVP's "real local store" — load a file from disk, run queries against it. The
// native `save`/`open` persistence path (crates/sparq-core) is a follow-up (see the bead),
// not wired here.
//
// HONESTY: no performance number is asserted. A future result panel may TIME a query with
// the engine's own measured latency and label it as such; it must never bake in a benchmark.

use std::sync::Mutex;

use sparq_core::Graph;

/// The desktop app's single native store: a `Graph` guarded by a `Mutex` (Tauri commands
/// can run concurrently, and the engine's query/update take `&Graph` / `&mut Graph`). Held
/// in Tauri managed state via [`tauri::Builder::manage`].
pub struct EngineState {
    graph: Mutex<Graph>,
}

impl EngineState {
    /// A fresh, empty native store. `sparq_core::Graph` has no `Default`/`new` (it is built
    /// by loading), so the empty store is an empty N-Triples parse — the same way the wasm
    /// `Store` constructs an empty receiver (`Store::load("", _)`).
    pub fn new() -> Self {
        let graph = Graph::load_str("", "ntriples").expect("empty N-Triples is a valid graph");
        Self {
            graph: Mutex::new(graph),
        }
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

/// A small helper so command bodies can lock the graph and map a poisoned lock to a
/// stringified error (Tauri commands return `Result<_, String>` to the webview).
fn lock(state: &EngineState) -> Result<std::sync::MutexGuard<'_, Graph>, String> {
    state
        .graph
        .lock()
        .map_err(|_| "engine state lock poisoned".to_string())
}

/// Load RDF into the store, REPLACING its current contents. `format` is one of the syntaxes
/// `Graph::load_str` / `load_dataset` accept (`turtle` / `ntriples` / `nquads` / `trig` /
/// `jsonld`). `preserve_graphs` routes the quad-bearing formats through `load_dataset` so
/// named graphs survive (mirrors the site's `loadIntoStore`). Returns the loaded triple
/// count.
#[tauri::command]
pub fn load(
    state: tauri::State<'_, EngineState>,
    text: String,
    format: String,
    preserve_graphs: bool,
) -> Result<usize, String> {
    let graph = if preserve_graphs {
        Graph::load_dataset(&text, &format)?
    } else {
        Graph::load_str(&text, &format)?
    };
    let size = graph.len();
    *lock(&state)? = graph;
    Ok(size)
}

/// Run a SELECT/ASK query, returning the SPARQL 1.1 JSON results document — the exact shape
/// the reused REPL frontend already renders (`@sparq/client`'s `SparqlResults`).
#[tauri::command]
pub fn query(state: tauri::State<'_, EngineState>, sparql: String) -> Result<String, String> {
    let graph = lock(&state)?;
    sparq_engine::query_json(&graph, &sparql)
}

/// Run a CONSTRUCT/DESCRIBE query, returning the constructed graph as an N-Triples document
/// (mirrors the wasm `queryQuads`).
#[tauri::command]
pub fn query_quads(
    state: tauri::State<'_, EngineState>,
    sparql: String,
) -> Result<String, String> {
    let graph = lock(&state)?;
    sparq_engine::construct_ntriples(&graph, &sparql)
}

/// Apply a SPARQL 1.1 Update IN PLACE through the engine's delta overlay (mirrors the wasm
/// `updateInPlace`). Returns the store's triple count after the update.
#[tauri::command]
pub fn update_in_place(
    state: tauri::State<'_, EngineState>,
    sparql: String,
) -> Result<usize, String> {
    let mut graph = lock(&state)?;
    sparq_engine::update_in_place(&mut graph, &sparql)?;
    Ok(graph.len())
}

/// The planning-only EXPLAIN plan text for any query form (mirrors the wasm `explain`).
#[tauri::command]
pub fn explain(state: tauri::State<'_, EngineState>, sparql: String) -> Result<String, String> {
    let graph = lock(&state)?;
    sparq_engine::explain(&graph, &sparql)
}

/// The plan + per-operator execution trace for SELECT/ASK (mirrors the wasm
/// `explainAnalyze`).
#[tauri::command]
pub fn explain_analyze(
    state: tauri::State<'_, EngineState>,
    sparql: String,
) -> Result<String, String> {
    let graph = lock(&state)?;
    sparq_engine::explain_analyze(&graph, &sparql)
}

/// A materialisation-free SELECT solution count (mirrors the wasm `count`).
#[tauri::command]
pub fn count(state: tauri::State<'_, EngineState>, sparql: String) -> Result<usize, String> {
    let graph = lock(&state)?;
    sparq_engine::count(&graph, &sparql)
}

/// The ASK fast path: a plain boolean, no SELECT materialised (mirrors the wasm `ask`).
#[tauri::command]
pub fn ask(state: tauri::State<'_, EngineState>, sparql: String) -> Result<bool, String> {
    let graph = lock(&state)?;
    sparq_engine::ask(&graph, &sparql)
}

/// The current store's total triple count (default graph + every named graph).
#[tauri::command]
pub fn store_size(state: tauri::State<'_, EngineState>) -> Result<usize, String> {
    Ok(lock(&state)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests exercise the native engine link directly (no Tauri runtime needed), so
    // they run on any box — including ones without the webview system libraries. They are
    // the locally-runnable proof that the direct-Rust-link command layer is wired to the
    // real engine, even when a full Tauri `cargo build` is CI-only.
    const TTL: &str = "<http://example.org/a> <http://example.org/p> <http://example.org/b> .";

    #[test]
    fn load_then_query_round_trips_through_the_native_engine() {
        let graph = Graph::load_str(TTL, "turtle").expect("turtle parses");
        assert_eq!(graph.len(), 1);
        let json = sparq_engine::query_json(&graph, "SELECT * WHERE { ?s ?p ?o }")
            .expect("select runs");
        assert!(json.contains("http://example.org/a"));
        assert!(json.contains("\"bindings\""));
    }

    #[test]
    fn ask_and_count_match_the_wasm_surface() {
        let graph = Graph::load_str(TTL, "turtle").expect("turtle parses");
        assert!(sparq_engine::ask(&graph, "ASK { ?s ?p ?o }").expect("ask runs"));
        assert_eq!(
            sparq_engine::count(&graph, "SELECT * WHERE { ?s ?p ?o }").expect("count runs"),
            1
        );
    }
}
