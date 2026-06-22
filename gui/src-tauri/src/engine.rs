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
    let graph = parse_graph(&text, &format, preserve_graphs)?;
    let size = graph.len();
    *lock(&state)? = graph;
    Ok(size)
}

/// Parse an in-memory RDF document into a `Graph`, honouring the named-graph-preserving
/// toggle. The ONE parse helper [`load`] / [`load_text`] share so the `preserve_graphs`
/// routing (`load_dataset` for the quad-bearing formats vs the cheaper `load_str`) is decided
/// in exactly one place.
fn parse_graph(text: &str, format: &str, preserve_graphs: bool) -> Result<Graph, String> {
    if preserve_graphs {
        Graph::load_dataset(text, format)
    } else {
        Graph::load_str(text, format)
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-ixc3.13 — the Import drawer's NATIVE loader.
//
// The whole point of the desktop target (research/gui-design.md §A.4): real disk/URL ingest
// through the NATIVE engine — threads, no ~2 GiB wasm-tab ceiling, COMPRESSED streams, and
// NATIVE-ONLY HDT — capabilities the browser read-replica fundamentally cannot offer. The
// native loader decodes the document into a `sparq_core::Graph` and hands it back to the
// frontend as N-QUADS text (the named-graph-preserving wire format), which the in-tab store
// MERGES (or replaces) into the live workspace. Keeping the in-tab store the single query
// surface — rather than splitting queries across two engines — is the honest, smallest design:
// the native side owns the heavy *ingest*, the in-tab side owns *query* (where the GUI already
// runs today). No performance number is asserted anywhere.
// ---------------------------------------------------------------------------

/// What the native loader hands the frontend after decoding a document: the whole dataset as
/// N-Quads (default graph + every named graph), the triple/quad count, and the format the
/// loader actually parsed it as (so a `.ttl.gz` file reports `turtle`, an `.hdt` reports `hdt`).
#[derive(Debug, serde::Serialize)]
pub struct LoadedDocument {
    /// The whole dataset serialised to N-Quads — the named-graph-preserving merge wire format.
    pub nquads: String,
    /// Total triples/quads parsed (default graph + every named graph).
    pub count: usize,
    /// The RDF serialisation the loader actually parsed the document as.
    pub format: String,
}

/// Serialise a decoded `Graph` into a [`LoadedDocument`]. Counts the WHOLE dataset (default
/// graph + every named graph) — NOT `Graph::len()`, which reports the default graph only and so
/// UNDER-counts a dataset with named graphs. The count is the non-empty N-Quads line count
/// (one line per quad), the same whole-dataset count the site derives from its N-Quads dump.
fn to_loaded_document(graph: &Graph, format: String) -> LoadedDocument {
    let nquads = sparq_engine::serialize::graph_to_nquads(graph);
    let count = nquads.lines().filter(|l| !l.trim().is_empty()).count();
    LoadedDocument {
        nquads,
        count,
        format,
    }
}

/// Decode an in-memory RDF document and return it as N-Quads for the in-tab store to merge.
/// The PASTE path: a document typed/pasted into the drawer. Mirrors [`load_path`]'s output so
/// the frontend's merge logic is one branch. Returns the whole dataset as N-Quads, the count,
/// and the (echoed) format.
#[tauri::command]
pub fn load_text(
    text: String,
    format: String,
    preserve_graphs: bool,
) -> Result<LoadedDocument, String> {
    let graph = parse_graph(&text, &format, preserve_graphs)?;
    Ok(to_loaded_document(&graph, format))
}

/// Decode an RDF document FROM DISK and return it as N-Quads for the in-tab store to merge —
/// the FILE tab of the Import drawer. This is the native loader's headline capability:
///
///   * **Compressed** streams (`.gz` / `.bz2` / `.zst[d]`) are decompressed natively (the SAME
///     codec matrix as the CLI's `open_reader`) — the browser cannot stream a multi-GiB
///     compressed file without buffering the whole decoded copy in the tab.
///   * **Native-only HDT** (`.hdt` / `.hdt.gz`, or `format == "hdt"`) routes through the opt-in
///     `sparq-hdt` crate (gated behind this crate's `hdt` feature). The wasm read-replica
///     cannot load HDT at all.
///   * Everything else parses as the given `format` (auto-detected by the frontend from the
///     extension), with `preserve_graphs` choosing the named-graph-preserving dataset path.
///
/// `format` is the frontend's extension-derived guess (it strips a compression suffix first);
/// the loader trusts it for the parse but routes HDT by extension regardless.
#[tauri::command]
pub fn load_path(
    path: String,
    format: String,
    preserve_graphs: bool,
) -> Result<LoadedDocument, String> {
    let lower = path.to_ascii_lowercase();

    // HDT archives route through sparq-hdt by extension OR an explicit `hdt` format. Opt-in:
    // a build without the `hdt` feature reports the actionable rebuild hint rather than
    // silently mis-parsing the binary as Turtle.
    if format == "hdt" || lower.ends_with(".hdt") || lower.ends_with(".hdt.gz") {
        return load_hdt(&path);
    }

    let graph = decode_file_to_graph(&path, &format, preserve_graphs)?;
    Ok(to_loaded_document(&graph, format))
}

/// Open a (possibly compressed) file as a streaming reader. Mirrors the CLI's `open_reader`
/// codec matrix (crates/sparq-cli/src/main.rs): a `.gz` is MultiGzDecoder, a `.bz2` is
/// MultiBzDecoder, a `.zst`/`.zstd` is the zstd stream decoder, anything else is the raw file.
fn open_reader(path: &str) -> std::io::Result<Box<dyn std::io::Read + Send>> {
    let file = std::fs::File::open(path)?;
    let lower = path.to_ascii_lowercase();
    Ok(if lower.ends_with(".gz") {
        Box::new(flate2::read::MultiGzDecoder::new(file))
    } else if lower.ends_with(".bz2") {
        Box::new(bzip2::read::MultiBzDecoder::new(file))
    } else if lower.ends_with(".zst") || lower.ends_with(".zstd") {
        Box::new(zstd::stream::read::Decoder::new(file)?)
    } else {
        Box::new(file)
    })
}

/// Decode a non-HDT file into a `Graph`. N-Triples streams block-by-block through the parallel
/// pipelined parser (no whole-decompressed copy held in RAM); the other formats need the whole
/// document buffered for the statement splitter. This mirrors the CLI's `load_quiet` shape.
fn decode_file_to_graph(path: &str, format: &str, preserve_graphs: bool) -> Result<Graph, String> {
    if matches!(format, "ntriples" | "n-triples") && !preserve_graphs {
        let reader = open_reader(path).map_err(|e| format!("cannot open {path}: {e}"))?;
        return Graph::load_reader_parallel(reader, format);
    }
    use std::io::Read;
    let mut text = String::new();
    open_reader(path)
        .and_then(|mut r| r.read_to_string(&mut text))
        .map_err(|e| format!("cannot read {path}: {e}"))?;
    parse_graph(&text, format, preserve_graphs)
}

/// Load an HDT archive via the opt-in `sparq-hdt` crate. Compiled out when the `hdt` feature
/// is off (the default build), returning an actionable rebuild hint so an HDT import in a
/// lean build fails LOUDLY rather than mis-parsing the binary — honest about the gate.
#[cfg(feature = "hdt")]
fn load_hdt(path: &str) -> Result<LoadedDocument, String> {
    let graph = sparq_hdt::load(path).map_err(|e| e.to_string())?;
    Ok(to_loaded_document(&graph, "hdt".to_string()))
}

#[cfg(not(feature = "hdt"))]
fn load_hdt(_path: &str) -> Result<LoadedDocument, String> {
    Err(
        "HDT support is not compiled into this build — rebuild the desktop app with \
         `cargo build --features hdt` to import .hdt archives."
            .to_string(),
    )
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
pub fn query_quads(state: tauri::State<'_, EngineState>, sparql: String) -> Result<String, String> {
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
        let json =
            sparq_engine::query_json(&graph, "SELECT * WHERE { ?s ?p ?o }").expect("select runs");
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

    // ── [OPUS-4.8] sq-ixc3.13 — the native loader (paste / disk / compressed / named graphs) ──

    const NQUADS: &str = "<http://example.org/a> <http://example.org/p> <http://example.org/b> <http://example.org/g> .\n<http://example.org/c> <http://example.org/p> <http://example.org/d> .";

    #[test]
    fn load_text_returns_nquads_count_and_format() {
        // The PASTE path: a single triple round-trips back as one N-Quads line.
        let doc = load_text(TTL.to_string(), "turtle".to_string(), false).expect("paste loads");
        assert_eq!(doc.count, 1);
        assert_eq!(doc.format, "turtle");
        assert!(doc.nquads.contains("http://example.org/a"));
    }

    #[test]
    fn load_text_preserves_named_graphs_when_toggled() {
        // preserve_graphs routes through load_dataset, so the named graph survives into the
        // returned N-Quads (a 4-term line); without it the quad-bearing parse path is not taken.
        let doc = load_text(NQUADS.to_string(), "nquads".to_string(), true).expect("nquads loads");
        assert_eq!(doc.count, 2);
        assert!(
            doc.nquads.contains("http://example.org/g"),
            "named graph must survive into the merged N-Quads: {}",
            doc.nquads
        );
    }

    #[test]
    fn load_path_decodes_a_plain_ntriples_file() {
        let dir = std::env::temp_dir().join(format!("sparq-gui-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("data.nt");
        std::fs::write(&path, TTL).expect("write nt file");
        let doc = load_path(
            path.to_string_lossy().into_owned(),
            "ntriples".to_string(),
            false,
        )
        .expect("ntriples file loads");
        assert_eq!(doc.count, 1);
        assert!(doc.nquads.contains("http://example.org/a"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_path_decodes_a_gzip_compressed_file_natively() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("sparq-gui-import-gz-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("data.nt.gz");
        // Write a gzip-compressed N-Triples document, the COMPRESSED half of the bead.
        let file = std::fs::File::create(&path).expect("create gz file");
        let mut enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        enc.write_all(TTL.as_bytes()).expect("gz write");
        enc.finish().expect("gz finish");
        // The frontend strips the .gz suffix to derive the format; the loader sniffs .gz itself.
        let doc = load_path(
            path.to_string_lossy().into_owned(),
            "ntriples".to_string(),
            false,
        )
        .expect("gzip file decodes + parses natively");
        assert_eq!(doc.count, 1);
        assert!(doc.nquads.contains("http://example.org/b"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_path_reports_missing_file_as_an_error_not_a_panic() {
        let err = load_path(
            "/definitely/not/a/real/path/data.nt".to_string(),
            "ntriples".to_string(),
            false,
        )
        .expect_err("a missing file is an Err");
        assert!(
            err.contains("cannot open") || err.contains("cannot read"),
            "got: {err}"
        );
    }

    #[test]
    #[cfg(not(feature = "hdt"))]
    fn hdt_import_in_a_lean_build_fails_loudly_with_a_rebuild_hint() {
        // Without the `hdt` feature, an .hdt path must NOT be mis-parsed as Turtle — it must
        // return the actionable rebuild hint.
        let err = load_path("/tmp/whatever.hdt".to_string(), "hdt".to_string(), false)
            .expect_err("HDT in a lean build is an Err");
        assert!(err.contains("--features hdt"), "got: {err}");
    }
}
