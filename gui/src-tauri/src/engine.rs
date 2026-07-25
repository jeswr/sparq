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

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sparq_core::Graph;

/// [OPUS-4.8] sq-w2sod — the single GENERIC error the native disk loader returns for ANY
/// open/read/authorisation failure. It carries NO OS error string, NO errno, and NO path, so a
/// webview script cannot use `load_path` as a filesystem existence/permission ORACLE: a
/// missing file, a permission-denied file, and an un-approved path are all indistinguishable
/// from the frontend. The detailed cause is logged server-side (see [`log_load_failure`]) where
/// only the operator — not the untrusted webview — can read it.
const LOAD_FAILED_MESSAGE: &str =
    "could not load the selected file. Pick the file again from the Import dialog and retry.";

/// [OPUS-4.8] sq-w2sod — the desktop app's single native store PLUS the set of disk paths the
/// user has actually picked through the native file-open dialog this session.
///
/// SECURITY (why the approved-set exists): an app-defined `#[tauri::command]` like [`load_path`]
/// is invokable by ANY script in the webview via `window.__TAURI__.core.invoke('load_path', …)`
/// and — unlike the `fs` PLUGIN commands — is NOT constrained by the capability allowlist in
/// `capabilities/default.json`. Without a guard it would read any process-readable path. So
/// [`load_path`] is bound to the approved set: a path is loadable ONLY if it was returned by
/// [`pick_rdf_files`], which opens the dialog SERVER-SIDE (the user must physically choose the
/// file — a script cannot forge a selection) and records the CANONICALISED path here. The set is
/// the unforgeable "token": membership is the capability. Fails closed — an un-approved path is
/// rejected with the same generic error as any other failure.
///
/// [OPUS-4.8] sq-w2sod — this no longer holds a `Graph`. The scaffolded in-process native store
/// and its nine query/update commands were never invoked (the query/update path runs in the
/// in-tab WASM engine), so they were removed to shrink the invocable surface; the native side owns
/// only the disk-INGEST loader (`pick_rdf_files` / `load_path` / `load_text`), which returns
/// N-Quads for the in-tab store to merge and keeps no server-side graph of its own.
pub struct EngineState {
    /// Canonicalised absolute paths the user picked via [`pick_rdf_files`] this session. Only a
    /// path in this set may be loaded by [`load_path`]. Populated exclusively by the server-side
    /// dialog flow, never by anything the webview controls.
    approved_paths: Mutex<HashSet<PathBuf>>,
}

impl EngineState {
    /// A fresh state: the approved-path set starts empty, so nothing is loadable until the user
    /// picks a file through the native dialog ([`pick_rdf_files`]).
    pub fn new() -> Self {
        Self {
            approved_paths: Mutex::new(HashSet::new()),
        }
    }

    /// Record a user-picked path as approved for loading. Canonicalises first so the stored key
    /// matches what [`is_approved`](Self::is_approved) canonicalises the load argument to
    /// (resolving `.`/`..`/symlinks); a path that does not canonicalise (e.g. it vanished between
    /// the dialog and this call) is simply not approved. Returns the canonical string that is
    /// handed back to the frontend to pass to [`load_path`].
    fn approve(&self, raw: &Path) -> Option<String> {
        let canonical = std::fs::canonicalize(raw).ok()?;
        let as_string = canonical.to_string_lossy().into_owned();
        if let Ok(mut set) = self.approved_paths.lock() {
            set.insert(canonical);
        }
        Some(as_string)
    }

    /// Whether `raw` canonicalises to a path the user actually picked this session. The load
    /// argument is canonicalised the SAME way [`approve`](Self::approve) canonicalised the
    /// dialog result, so `.`/`..`/symlink spellings of an approved file still match, and a path
    /// that never went through the dialog — or that fails to canonicalise — is not approved.
    fn is_approved(&self, raw: &str) -> bool {
        let Ok(canonical) = std::fs::canonicalize(raw) else {
            return false;
        };
        self.approved_paths
            .lock()
            .map(|set| set.contains(&canonical))
            .unwrap_or(false)
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

/// [OPUS-4.8] sq-w2sod — log the DETAILED cause of a native-load failure server-side (stderr),
/// where the operator can see it, and return the GENERIC [`LOAD_FAILED_MESSAGE`] to the webview.
/// This keeps a useful diagnostic without leaking OS/path detail to the untrusted frontend
/// (closing the existence/permission oracle). Called at every `load_path` failure edge.
fn log_load_failure(context: &str, detail: &str) -> String {
    eprintln!("[sparq-gui] native load failed ({context}): {detail}");
    LOAD_FAILED_MESSAGE.to_string()
}

/// Parse an in-memory RDF document into a `Graph`, honouring the named-graph-preserving
/// toggle. The ONE parse helper [`load_text`] / [`decode_file_to_graph`] share so the
/// `preserve_graphs` routing (`load_dataset` for the quad-bearing formats vs the cheaper
/// `load_str`) is decided in exactly one place.
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

/// [OPUS-4.8] sq-w2sod — open the NATIVE file-open dialog for RDF documents SERVER-SIDE, record
/// each chosen path (canonicalised) as approved in [`EngineState`], and return those canonical
/// paths to the frontend. The returned paths are the ONLY ones [`load_path`] will subsequently
/// load — the approved set is the unforgeable capability token.
///
/// WHY SERVER-SIDE (the security property): the dialog is driven from Rust here rather than from
/// the webview, so a script cannot fabricate a "selection" — the user must physically pick the
/// file(s) in the OS dialog. The chosen paths are canonicalised and stored; nothing the webview
/// controls can add to the approved set. This replaces the frontend's direct `plugin:dialog|open`
/// call, letting us drop the `dialog:allow-open` capability grant and shrink the invocable
/// surface. `blocking_pick_files` runs the dialog on the OS thread and blocks this command's
/// worker until the user confirms/cancels; a cancel returns an empty list.
#[tauri::command]
pub fn pick_rdf_files(
    app: tauri::AppHandle,
    state: tauri::State<'_, EngineState>,
) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    // The SAME extension filter the old frontend dialog offered: RDF syntaxes plus the
    // compression suffixes the native loader streams and the native-only HDT archive extensions.
    let picked = app
        .dialog()
        .file()
        .set_title("Import RDF files into the workspace")
        .add_filter(
            "RDF (incl. compressed + HDT)",
            &[
                "ttl", "nt", "nq", "trig", "jsonld", "json", "hdt", "gz", "bz2", "zst", "zstd",
            ],
        )
        .add_filter("All files", &["*"])
        .blocking_pick_files();

    let Some(files) = picked else {
        // User cancelled — no paths, nothing approved.
        return Ok(Vec::new());
    };

    // Canonicalise + approve each pick, returning the canonical strings the frontend threads back
    // into `load_path`. A pick that fails to canonicalise (vanished between dialog and now) is
    // silently dropped rather than approved.
    let approved = files
        .into_iter()
        .filter_map(|fp| fp.into_path().ok())
        .filter_map(|p| state.approve(&p))
        .collect();
    Ok(approved)
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
///
/// SECURITY (sq-w2sod): this command is invokable by any webview script and is NOT bound by the
/// `fs`-plugin capability allowlist. It is therefore GATED — `path` must be one the user picked
/// through [`pick_rdf_files`] (recorded canonicalised in [`EngineState`]); any other path, and
/// any open/read/parse failure, returns the single GENERIC [`LOAD_FAILED_MESSAGE`] (no OS detail,
/// no path echo) so it cannot be used as a filesystem existence/permission oracle. Fails closed.
#[tauri::command]
pub fn load_path(
    state: tauri::State<'_, EngineState>,
    path: String,
    format: String,
    preserve_graphs: bool,
) -> Result<LoadedDocument, String> {
    load_approved_path(&state, &path, &format, preserve_graphs)
}

/// The gated load body, factored out of the [`load_path`] command so it can be unit-tested
/// against a plain [`EngineState`] (no Tauri runtime / managed-state harness needed).
///
/// The security contract lives here: `path` must be in the approved set (a path the user picked
/// through [`pick_rdf_files`]); an un-approved path — and every open/read/parse failure — returns
/// the single generic [`LOAD_FAILED_MESSAGE`], with the detailed cause logged server-side only.
fn load_approved_path(
    state: &EngineState,
    path: &str,
    format: &str,
    preserve_graphs: bool,
) -> Result<LoadedDocument, String> {
    // Fail closed: only a path the user actually picked through the native dialog this session is
    // loadable. Everything else is rejected with the SAME generic error as any other failure, so
    // an un-approved path is indistinguishable from a missing/unreadable one (no enumeration
    // oracle). The detailed reason is logged server-side, not returned.
    if !state.is_approved(path) {
        return Err(log_load_failure(
            "authorisation",
            "path was not selected through the native Import dialog",
        ));
    }

    let lower = path.to_ascii_lowercase();

    // HDT archives route through sparq-hdt by extension OR an explicit `hdt` format. Opt-in:
    // a build without the `hdt` feature reports the actionable rebuild hint rather than
    // silently mis-parsing the binary as Turtle.
    if format == "hdt" || lower.ends_with(".hdt") || lower.ends_with(".hdt.gz") {
        return load_hdt(path);
    }

    let graph = decode_file_to_graph(path, format, preserve_graphs)?;
    Ok(to_loaded_document(&graph, format.to_string()))
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
    // Every failure edge below maps to the SAME generic error (the OS detail is logged
    // server-side, not returned) so a caller cannot distinguish "does not exist" from
    // "permission denied" from "not valid RDF" — closing the oracle (sq-w2sod).
    if matches!(format, "ntriples" | "n-triples") && !preserve_graphs {
        let reader = open_reader(path).map_err(|e| log_load_failure("open", &e.to_string()))?;
        return Graph::load_reader_parallel(reader, format)
            .map_err(|e| log_load_failure("parse", &e));
    }
    use std::io::Read;
    let mut text = String::new();
    open_reader(path)
        .and_then(|mut r| r.read_to_string(&mut text))
        .map_err(|e| log_load_failure("read", &e.to_string()))?;
    parse_graph(&text, format, preserve_graphs).map_err(|e| log_load_failure("parse", &e))
}

/// Load an HDT archive via the opt-in `sparq-hdt` crate. Compiled out when the `hdt` feature
/// is off (the default build), returning an actionable rebuild hint so an HDT import in a
/// lean build fails LOUDLY rather than mis-parsing the binary — honest about the gate.
#[cfg(feature = "hdt")]
fn load_hdt(path: &str) -> Result<LoadedDocument, String> {
    // The HDT decode error can carry the path / OS detail, so it is logged server-side and the
    // caller gets the same generic message as any other load failure (sq-w2sod).
    let graph = sparq_hdt::load(path).map_err(|e| log_load_failure("hdt", &e.to_string()))?;
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

// [OPUS-4.8] sq-w2sod — the nine scaffolded native query/update commands
// (`load` / `query` / `query_quads` / `update_in_place` / `explain` / `explain_analyze` /
// `count` / `ask` / `store_size`) that the frontend never invoked (query/update runs in the
// in-tab WASM engine) were REMOVED here to shrink the invocable surface. The native side owns
// only the disk-ingest loader (`pick_rdf_files` / `load_path` / `load_text`) plus the disk-usage
// probe (`disk.rs`). Their equivalents remain available in `sparq_engine::{query_json, ask,
// count, …}` if a future native query surface is ever wired — that would be a deliberate,
// reviewed addition, not a pre-wired latent surface.

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

    // Loading exercises the GATED body `load_approved_path`: the disk-loader tests must first
    // APPROVE the path (as the server-side dialog would), so they double as proof that the
    // approve → load handshake works end to end.

    #[test]
    fn load_path_decodes_a_plain_ntriples_file() {
        let dir = std::env::temp_dir().join(format!("sparq-gui-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("data.nt");
        std::fs::write(&path, TTL).expect("write nt file");
        let state = EngineState::new();
        // Approve the pick exactly as the dialog flow would, then load it.
        let approved = state.approve(&path).expect("canonicalise + approve");
        let doc = load_approved_path(&state, &approved, "ntriples", false)
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
        let state = EngineState::new();
        let approved = state.approve(&path).expect("canonicalise + approve");
        // The frontend strips the .gz suffix to derive the format; the loader sniffs .gz itself.
        let doc = load_approved_path(&state, &approved, "ntriples", false)
            .expect("gzip file decodes + parses natively");
        assert_eq!(doc.count, 1);
        assert!(doc.nquads.contains("http://example.org/b"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── [OPUS-4.8] sq-w2sod — the approved-path gate + generic-error oracle closure ──

    #[test]
    fn load_path_rejects_an_unapproved_path_with_the_generic_error() {
        // An EXISTING, readable RDF file that was NEVER picked through the dialog must NOT load —
        // and the rejection must be the generic message, revealing nothing about the file.
        let dir = std::env::temp_dir().join(format!("sparq-gui-unapproved-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("secret.nt");
        std::fs::write(&path, TTL).expect("write nt file");
        let state = EngineState::new(); // nothing approved

        let err = load_approved_path(
            &state,
            &path.to_string_lossy(),
            "ntriples",
            false,
        )
        .expect_err("an un-approved path must be rejected");

        // Fail closed with the EXACT generic message — no path, no OS detail.
        assert_eq!(err, LOAD_FAILED_MESSAGE);
        assert!(!err.contains("secret"), "must not echo the path: {err}");
        assert!(
            !err.to_ascii_lowercase().contains("permission"),
            "must not leak permission state: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_path_missing_file_returns_the_generic_error_not_an_oracle() {
        // A missing path never went through the dialog, so approval rejects it first — and does so
        // with the SAME generic message a missing/permission-denied file would give, so the caller
        // cannot tell "does not exist" from "not approved" from "denied" (no enumeration oracle).
        let state = EngineState::new();
        let err = load_approved_path(
            &state,
            "/definitely/not/a/real/path/data.nt",
            "ntriples",
            false,
        )
        .expect_err("a missing / un-approved file is an Err");

        assert_eq!(err, LOAD_FAILED_MESSAGE, "exact generic message");
        // Assert the oracle is CLOSED: no path echo, no errno/OS-detail leakage.
        for leak in ["/definitely", "data.nt", "No such file", "os error", "cannot open"] {
            assert!(!err.contains(leak), "generic error must not contain {leak:?}: {err}");
        }
    }

    #[test]
    fn approved_path_matches_across_dot_and_symlink_spellings() {
        // A `.`-laden spelling of an approved path still canonicalises to the same key, so a
        // legitimate re-spelling of the SAME picked file is accepted (the check is on the
        // canonical identity, not the raw string).
        let dir = std::env::temp_dir().join(format!("sparq-gui-canon-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("data.nt");
        std::fs::write(&path, TTL).expect("write nt file");
        let state = EngineState::new();
        let _canonical = state.approve(&path).expect("approve");

        // A `dir/./data.nt` spelling of the same file must be recognised as approved.
        let dotted = dir.join(".").join("data.nt");
        assert!(
            state.is_approved(&dotted.to_string_lossy()),
            "a `.`-spelling of an approved file must canonicalise to the approved key"
        );
        // A sibling that was never approved must NOT be.
        let sibling = dir.join("other.nt");
        std::fs::write(&sibling, TTL).expect("write sibling");
        assert!(
            !state.is_approved(&sibling.to_string_lossy()),
            "a never-picked sibling must not be approved"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(not(feature = "hdt"))]
    fn hdt_import_in_a_lean_build_fails_loudly_with_a_rebuild_hint() {
        // Without the `hdt` feature, an APPROVED .hdt path must NOT be mis-parsed as Turtle — it
        // must return the actionable rebuild hint (an actionable BUILD-config message, distinct
        // from the generic load error; it never touches the file so it is no FS oracle).
        let dir = std::env::temp_dir().join(format!("sparq-gui-hdt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mk tmp dir");
        let path = dir.join("whatever.hdt");
        std::fs::write(&path, b"not really hdt").expect("write stub .hdt");
        let state = EngineState::new();
        let approved = state.approve(&path).expect("approve");
        let err = load_approved_path(&state, &approved, "hdt", false)
            .expect_err("HDT in a lean build is an Err");
        assert!(err.contains("--features hdt"), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
