//! [SONNET-4.6] sq-hmd7l.13 — LDBC Graphalytics runner for the `sparq-algos` surface.
//!
//! Reads a dataset in the **LDBC Graphalytics on-disk format** (`<name>.properties` +
//! a vertex file + an edge file), materialises it as RDF, loads it into a
//! [`sparq_core::Graph`] (the dict-encoded triple store), projects a [`NodeGraph`], and
//! runs ONE algorithm with **Graphalytics-conformant parameters** — writing the result in
//! the Graphalytics reference-output format (`<vertex-id> <value>` per line, ascending by
//! vertex id) so `bench/graphalytics/gx.py validate` can gate on correctness BEFORE any
//! timing is believed. That ordering — validate, then time — is the whole point of the
//! harness; see `bench/graphalytics/README.md`.
//!
//! # Which algorithms this can run
//!
//! Graphalytics specifies six: BFS, PR, WCC, CDLP, LCC, SSSP. `sparq-algos` implements
//! the third and (a variant of) the fourth, plus PageRank:
//!
//! | Graphalytics | sparq-algos | status |
//! |---|---|---|
//! | `pr`   | [`pagerank`]                     | **conformant** (see below) |
//! | `wcc`  | [`weakly_connected_components`]  | **conformant** |
//! | `cdlp` | [`label_propagation`]            | **non-conformant variant** (see below) |
//! | `bfs`  | —                                | not implemented — feature gap |
//! | `lcc`  | —                                | not implemented — feature gap |
//! | `sssp` | —                                | not implemented — feature gap (needs edge weights) |
//!
//! A not-implemented algorithm **exits non-zero with an explicit FEATURE GAP message**. It
//! is never silently skipped and never emits a fabricated row.
//!
//! ## Why `pr` is conformant
//!
//! The Graphalytics PageRank recurrence is
//!
//! ```text
//! PR_0(v) = 1/n
//! PR_i(v) = (1-d)/n  +  d * ( SUM_{u in Nin(v)} PR_{i-1}(u)/outdeg(u)
//!                           + SUM_{w : outdeg(w)=0} PR_{i-1}(w)/n )
//! ```
//!
//! run for a **fixed** `pr.num-iterations`. That is exactly the body of
//! [`sparq_algos::pagerank`] (uniform init, teleport term, dangling mass redistributed
//! uniformly, contributions split by out-degree). The only behavioural difference is that
//! the library additionally stops early once the L1 delta drops below `tolerance` — so
//! setting `tolerance = 0.0` (a delta is never `< 0.0`) makes it run exactly
//! `max_iterations` sweeps, i.e. the Graphalytics fixed-iteration semantics. We therefore
//! validate the **library** function, not a re-implementation.
//!
//! ## Why `cdlp` is NOT conformant
//!
//! Graphalytics CDLP updates every vertex **synchronously** from the previous iteration's
//! labels for a fixed `cdlp.max-iterations`. [`sparq_algos::label_propagation`] updates
//! **asynchronously** (in place, within a sweep) and stops at a fixed point. Both count a
//! reciprocal edge twice and both break ties toward the smallest label, but the
//! synchronous/asynchronous difference is a genuine semantic divergence: the two agree on
//! some graphs and disagree on others. The harness still runs the validation and reports
//! the honest verdict — it does not pretend the gate passed.
//!
//! # Usage
//!
//! ```sh
//! cargo run -p sparq-algos --release --example graphalytics -- \
//!     --dataset bench/graphalytics/data/smoke-directed --algo pr --out /tmp/pr.out
//! ```
//!
//! Flags: `--dataset DIR` (required), `--algo pr|wcc|cdlp|bfs|lcc|sssp` (required),
//! `--name NAME` (defaults to the single `*.properties` stem in DIR), `--out FILE`
//! (defaults to stdout), `--iters N` (timed repetitions, default 3),
//! `--export-edgelist FILE` (also write the plain `src dst` edge list a library such as
//! igraph would need).
//!
//! # Timing lines
//!
//! Emitted on stdout as `metric_us <key>=<microseconds>`:
//!
//! * `materialize` — Graphalytics files → an N-Triples file on disk.
//! * `load` — N-Triples → the dict-encoded [`sparq_core::Graph`].
//! * `project` — the store → a [`NodeGraph`] adjacency view (**no export**; this is
//!   sparq's differentiator — the analytics run over the store's own dictionary ids).
//! * `algo` — best-of-`--iters` for the algorithm itself.
//! * `export_edgelist` — what it costs to serialise that same graph OUT to the flat
//!   `src dst` text an embedded graph library consumes. Reported so the RDF-store path can
//!   be compared against igraph/NetworKit **honestly**, export cost included.
//!
//! Timings taken on a shared/CI box are NOT canonical — see `bench/CATALOG.md`.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use sparq_algos::{
    label_propagation, pagerank, weakly_connected_components, LabelPropConfig, NodeGraph,
    PageRankConfig,
};
use sparq_core::Graph;

/// IRI prefix for a Graphalytics vertex id. The id is recoverable from the term, which is
/// how results are mapped back out of dense [`NodeGraph`] indices.
const VERTEX_PREFIX: &str = "http://ldbc.eu/graphalytics/v/";
/// The single predicate every Graphalytics edge is asserted with.
const EDGE_PRED: &str = "http://ldbc.eu/graphalytics/edge";
/// Data predicate used to pin an ISOLATED vertex into the store: `NodeGraph`'s default
/// entity projection keeps a subject as a node even when the object is a literal, so this
/// makes a vertex with no incident edges a real (degree-0) node rather than a missing one.
const ID_PRED: &str = "http://ldbc.eu/graphalytics/id";

/// Exit code for "sparq-algos does not implement this Graphalytics algorithm". Distinct
/// from a usage error (2) so the harness can report it as a FEATURE GAP row rather than a
/// failure of the run.
const EXIT_FEATURE_GAP: u8 = 3;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("[graphalytics] ERROR: {e}");
            ExitCode::from(2)
        }
    }
}

/// Parsed command line.
struct Args {
    dataset: PathBuf,
    algo: String,
    name: Option<String>,
    out: Option<PathBuf>,
    iters: usize,
    export_edgelist: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut dataset = None;
    let mut algo = None;
    let mut name = None;
    let mut out = None;
    let mut iters = 3usize;
    let mut export_edgelist = None;

    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} requires a value"));
        match flag.as_str() {
            "--dataset" => dataset = Some(PathBuf::from(value()?)),
            "--algo" => algo = Some(value()?.to_ascii_lowercase()),
            "--name" => name = Some(value()?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--export-edgelist" => export_edgelist = Some(PathBuf::from(value()?)),
            "--iters" => {
                let v = value()?;
                iters = v.parse().map_err(|_| format!("--iters: not a number: {v}"))?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Args {
        dataset: dataset.ok_or("--dataset DIR is required")?,
        algo: algo.ok_or("--algo pr|wcc|cdlp|bfs|lcc|sssp is required")?,
        name,
        out,
        iters: iters.max(1),
        export_edgelist,
    })
}

fn run() -> Result<ExitCode, String> {
    let args = parse_args()?;
    let name = match &args.name {
        Some(n) => n.clone(),
        None => discover_name(&args.dataset)?,
    };
    let props = Props::load(&args.dataset, &name)?;

    // Fail on an unimplemented algorithm BEFORE doing any work, so the harness gets a
    // clean, unambiguous FEATURE GAP signal.
    match args.algo.as_str() {
        "pr" | "wcc" | "cdlp" => {}
        gap @ ("bfs" | "lcc" | "sssp") => {
            eprintln!(
                "[graphalytics] FEATURE GAP: sparq-algos does not implement Graphalytics \
                 '{}' ({}). No row is emitted — see research/gap-graphalytics-2026-07.md.",
                gap.to_ascii_uppercase(),
                gap_description(gap)
            );
            return Ok(ExitCode::from(EXIT_FEATURE_GAP));
        }
        other => return Err(format!("unknown --algo: {other}")),
    }

    // ---- 1. materialise the Graphalytics files as RDF -----------------------------------
    let vertex_file = args
        .dataset
        .join(props.get_or("vertex-file", &format!("{name}.v")));
    let edge_file = args
        .dataset
        .join(props.get_or("edge-file", &format!("{name}.e")));
    let directed = props.bool_or("directed", true)?;

    let nt_path = std::env::temp_dir().join(format!("sparq-graphalytics-{name}.nt"));
    let t = Instant::now();
    let (n_vertices, n_edges) = materialize_nt(&vertex_file, &edge_file, directed, &nt_path)?;
    println!("metric_us materialize={}", t.elapsed().as_micros());
    // Best-effort cleanup: the file is scratch, and a stale one would just be overwritten.
    let _cleanup = ScratchFile(nt_path.clone());

    // ---- 2. load into the dict-encoded store, then project --------------------------------
    let t = Instant::now();
    let reader = BufReader::new(File::open(&nt_path).map_err(|e| format!("{nt_path:?}: {e}"))?);
    let graph = Graph::load_reader(reader, "nt").map_err(|e| format!("load {nt_path:?}: {e}"))?;
    println!("metric_us load={}", t.elapsed().as_micros());

    let t = Instant::now();
    let ng = NodeGraph::build(&graph);
    println!("metric_us project={}", t.elapsed().as_micros());
    println!(
        "graph name={name} directed={directed} file_vertices={n_vertices} file_edges={n_edges} \
         nodes={} edges={}",
        ng.len(),
        ng.edge_count()
    );

    // Dense node index -> Graphalytics vertex id, recovered from the term text.
    let vertex_ids = vertex_ids(&graph, &ng)?;

    // ---- 3. the export-to-igraph cost, measured on the SAME projected graph ---------------
    let t = Instant::now();
    let edgelist = export_edgelist(&ng, &vertex_ids);
    let export_us = t.elapsed().as_micros();
    println!("metric_us export_edgelist={export_us}");
    if let Some(path) = &args.export_edgelist {
        std::fs::write(path, &edgelist).map_err(|e| format!("{path:?}: {e}"))?;
    }

    // ---- 4. run the algorithm (best-of-iters), then emit the Graphalytics output ----------
    let mut best_us = u128::MAX;
    let mut lines: Vec<(i64, String)> = Vec::new();
    for _ in 0..args.iters {
        let t = Instant::now();
        let produced = match args.algo.as_str() {
            "pr" => {
                let cfg = PageRankConfig {
                    damping: props.f64_req("pr.damping-factor")?,
                    // Never satisfied, so the power method runs exactly `max_iterations`
                    // sweeps — the Graphalytics fixed-iteration semantics. See the module
                    // docs for why this makes the library function conformant.
                    tolerance: 0.0,
                    max_iterations: props.usize_req("pr.num-iterations")?,
                };
                let r = pagerank(&ng, cfg);
                Produced::Float(r)
            }
            "wcc" => Produced::Label(weakly_connected_components(&ng)),
            "cdlp" => {
                let cfg = LabelPropConfig {
                    max_iterations: props.usize_req("cdlp.max-iterations")?,
                };
                Produced::Label(label_propagation(&ng, cfg))
            }
            other => return Err(format!("unreachable algo {other}")),
        };
        best_us = best_us.min(t.elapsed().as_micros());
        lines = produced.render(&vertex_ids);
    }
    println!("metric_us algo={best_us}");

    if args.algo == "cdlp" {
        eprintln!(
            "[graphalytics] NOTE: sparq's label_propagation is an ASYNCHRONOUS, \
             run-to-fixed-point variant; Graphalytics CDLP is SYNCHRONOUS over a fixed \
             iteration count. The validation verdict for cdlp is reported, never assumed."
        );
    }

    lines.sort_by_key(|(v, _)| *v);
    let mut sink: Box<dyn Write> = match &args.out {
        Some(p) => Box::new(BufWriter::new(
            File::create(p).map_err(|e| format!("{p:?}: {e}"))?,
        )),
        None => Box::new(BufWriter::new(std::io::stdout())),
    };
    for (v, value) in &lines {
        writeln!(sink, "{v} {value}").map_err(|e| e.to_string())?;
    }
    sink.flush().map_err(|e| e.to_string())?;
    Ok(ExitCode::SUCCESS)
}

/// One-line description of a Graphalytics algorithm sparq-algos does not implement, used in
/// the FEATURE GAP message so the gap is self-explanatory in a log.
fn gap_description(algo: &str) -> &'static str {
    match algo {
        "bfs" => "breadth-first-search hop distance from a source vertex",
        "lcc" => "local clustering coefficient per vertex",
        "sssp" => "single-source shortest paths over WEIGHTED edges; NodeGraph is unweighted",
        _ => "",
    }
}

/// Deletes a scratch file on drop. Failure is ignored — it lives in the temp dir.
struct ScratchFile(PathBuf);

impl Drop for ScratchFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The per-vertex result of an algorithm, before it is rendered to the Graphalytics output
/// format.
enum Produced {
    /// A floating-point score per node index (PageRank).
    Float(Vec<f64>),
    /// A community/component label per node index, in the crate's dense `0..k` id space.
    Label(Vec<usize>),
}

impl Produced {
    /// Renders `(vertex_id, value)` pairs. Labels are **canonicalised to the smallest
    /// vertex id in the group**, which is the labelling convention the Graphalytics
    /// reference outputs use. This is purely a relabelling — the partition is unchanged,
    /// and the validator compares partitions by equivalence regardless.
    fn render(&self, vertex_ids: &[i64]) -> Vec<(i64, String)> {
        match self {
            Produced::Float(scores) => scores
                .iter()
                .enumerate()
                .map(|(i, s)| (vertex_ids[i], format!("{s:.12e}")))
                .collect(),
            Produced::Label(labels) => {
                let mut min_vertex: HashMap<usize, i64> = HashMap::new();
                for (i, &l) in labels.iter().enumerate() {
                    let v = vertex_ids[i];
                    min_vertex
                        .entry(l)
                        .and_modify(|m| *m = (*m).min(v))
                        .or_insert(v);
                }
                labels
                    .iter()
                    .enumerate()
                    .map(|(i, l)| (vertex_ids[i], min_vertex[l].to_string()))
                    .collect()
            }
        }
    }
}

/// The `<name>.properties` sidecar, flattened to the part of each key after
/// `graph.<name>.`.
struct Props {
    map: HashMap<String, String>,
}

impl Props {
    fn load(dir: &Path, name: &str) -> Result<Props, String> {
        let path = dir.join(format!("{name}.properties"));
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{path:?}: {e}"))?;
        let prefix = format!("graph.{name}.");
        let mut map = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            if let Some(sub) = k.trim().strip_prefix(&prefix) {
                map.insert(sub.to_string(), v.trim().to_string());
            }
        }
        Ok(Props { map })
    }

    fn get_or(&self, key: &str, default: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| default.into())
    }

    fn bool_or(&self, key: &str, default: bool) -> Result<bool, String> {
        match self.map.get(key) {
            None => Ok(default),
            Some(v) => v
                .parse()
                .map_err(|_| format!("property {key}: not a boolean: {v}")),
        }
    }

    /// A REQUIRED conformance parameter. Deliberately has no default: silently guessing
    /// `pr.num-iterations` would produce a number that looks valid and is not.
    fn f64_req(&self, key: &str) -> Result<f64, String> {
        let v = self
            .map
            .get(key)
            .ok_or_else(|| format!("required property graph.<name>.{key} is missing"))?;
        v.parse()
            .map_err(|_| format!("property {key}: not a number: {v}"))
    }

    fn usize_req(&self, key: &str) -> Result<usize, String> {
        let v = self
            .map
            .get(key)
            .ok_or_else(|| format!("required property graph.<name>.{key} is missing"))?;
        v.parse()
            .map_err(|_| format!("property {key}: not an integer: {v}"))
    }
}

/// Finds the dataset name from the single `*.properties` file in `dir`.
fn discover_name(dir: &Path) -> Result<String, String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{dir:?}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "properties") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                found.push(stem.to_string());
            }
        }
    }
    found.sort();
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!("no *.properties file in {dir:?} — pass --name")),
        _ => Err(format!(
            "{} *.properties files in {dir:?} ({}) — pass --name",
            found.len(),
            found.join(", ")
        )),
    }
}

/// Writes the dataset out as N-Triples. Returns `(vertices, edge-file lines)`.
///
/// Streamed line-by-line rather than built in memory: the larger Graphalytics graphs have
/// hundreds of millions of edges, and a single `String` of that N-Triples text would not
/// fit alongside the store.
fn materialize_nt(
    vertex_file: &Path,
    edge_file: &Path,
    directed: bool,
    out: &Path,
) -> Result<(usize, usize), String> {
    let mut w = BufWriter::new(File::create(out).map_err(|e| format!("{out:?}: {e}"))?);
    let mut n_vertices = 0usize;
    let mut n_edges = 0usize;

    let vf = BufReader::new(File::open(vertex_file).map_err(|e| format!("{vertex_file:?}: {e}"))?);
    for line in vf.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let Some(id) = line.split_whitespace().next() else {
            continue;
        };
        // A literal object keeps the subject a node without adding an edge, so an ISOLATED
        // vertex still shows up in the projection with degree 0.
        writeln!(w, "<{VERTEX_PREFIX}{id}> <{ID_PRED}> \"{id}\" .")
            .map_err(|e| e.to_string())?;
        n_vertices += 1;
    }

    let ef = BufReader::new(File::open(edge_file).map_err(|e| format!("{edge_file:?}: {e}"))?);
    for line in ef.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let mut f = line.split_whitespace();
        let (Some(s), Some(o)) = (f.next(), f.next()) else {
            continue; // blank line; a trailing weight column is simply ignored
        };
        writeln!(w, "<{VERTEX_PREFIX}{s}> <{EDGE_PRED}> <{VERTEX_PREFIX}{o}> .")
            .map_err(|e| e.to_string())?;
        if !directed {
            // Graphalytics undirected edge files list each edge ONCE; the algorithms see it
            // in both directions.
            writeln!(w, "<{VERTEX_PREFIX}{o}> <{EDGE_PRED}> <{VERTEX_PREFIX}{s}> .")
                .map_err(|e| e.to_string())?;
        }
        n_edges += 1;
    }
    w.flush().map_err(|e| e.to_string())?;
    Ok((n_vertices, n_edges))
}

/// Recovers the Graphalytics vertex id of every dense node index from its term text.
fn vertex_ids(graph: &Graph, ng: &NodeGraph) -> Result<Vec<i64>, String> {
    (0..ng.len())
        .map(|i| {
            let term = ng.term(graph, i).to_string();
            // `Term`'s Display for an IRI is `<...>`.
            let inner = term.trim_start_matches('<').trim_end_matches('>');
            let id = inner
                .strip_prefix(VERTEX_PREFIX)
                .ok_or_else(|| format!("node {i} is not a Graphalytics vertex: {term}"))?;
            id.parse::<i64>()
                .map_err(|_| format!("vertex id is not an integer: {id}"))
        })
        .collect()
}

/// Serialises the projected graph to the flat `src dst` edge-list text an embedded graph
/// library (igraph, NetworKit) reads. This is the **export cost** an RDF-store user pays to
/// hand their graph to such a library; sparq's own algorithms never do it.
fn export_edgelist(ng: &NodeGraph, vertex_ids: &[i64]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(ng.edge_count() * 16);
    for (i, src) in vertex_ids.iter().enumerate() {
        for &j in ng.out_neighbors(i) {
            let dst = vertex_ids[j as usize];
            let _ = writeln!(buf, "{src} {dst}");
        }
    }
    buf
}
