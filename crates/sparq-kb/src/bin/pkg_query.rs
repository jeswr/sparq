//! `pkg-query` — one command to **introspect → ground → ask** over the ingested PKG.
//!
//! [OPUS-4.8] sq-2m6zm.3 (epic sq-2m6zm); design record
//! `research/dogfooding-sparq-knowledge-graph.md` §6 (PR #1063). 🤖 SPARQ agent —
//! dogfooding sparq as a project knowledge graph. Written while Fable unavailable; flag
//! for re-review when Fable returns.
//!
//! Loads the PKG ontology (`pkg.ttl`) + the ingested instances (`pkg-instances.ttl`)
//! into a sparq store and runs either a **named canned query** (`--query <name>`) or a
//! **raw SPARQL string** (`--sparql '<select …>'`), printing the engine's exact result
//! rows. The point of the tool is to let an agent pay tokens proportional to the
//! *answer*, not to the source document. Whether that actually saves tokens is
//! **measured by sq-2m6zm.4**, not claimed here.
//!
//! ## Usage
//!
//! ```text
//! # list the canned introspect/ground queries:
//! cargo run -p sparq-kb --features query --bin pkg-query -- --list
//!
//! # INTROSPECT — what classes/properties can I ask about?
//! cargo run -p sparq-kb --features query --bin pkg-query -- --query schema-classes
//!
//! # GROUND+ASK — a canned query (some take an argument):
//! cargo run -p sparq-kb --features query --bin pkg-query -- \
//!     --query findings-about --arg https://sparq.dev/ns/pkg/kb#topic-merge-discipline
//!
//! # ASK — a raw SPARQL SELECT:
//! cargo run -p sparq-kb --features query --bin pkg-query -- \
//!     --sparql 'PREFIX pkg: <https://sparq.dev/ns/pkg#> SELECT (COUNT(*) AS ?n) WHERE { ?s a pkg:Task }'
//!
//! # CITE — every Finding with its [source] citation + the measured citation
//! # metrics (resolution rate / fabricated count; sq-2489d.11):
//! cargo run -p sparq-kb --features query --bin pkg-query -- --citations
//!
//! # optionally materialise the RDFS/OWL-RL closure first (needs --features close):
//! cargo run -p sparq-kb --features close --bin pkg-query -- --close owl-rl --query task-blocks --arg sq-8thu
//!
//! # load extra graph(s) — e.g. a foundational-ontology overlay — alongside the PKG
//! # before (optionally) closing + querying (repeatable; the FO-KM benchmark seam,
//! # epic sq-mztg8 Metric 1):
//! cargo run -p sparq-kb --features close --bin pkg-query -- \
//!     --extra-graph bench/fo-km/overlays/gufo.ttl --close owl-rl \
//!     --sparql 'PREFIX gufo: <http://purl.org/nemo/gufo#> SELECT (COUNT(*) AS ?n) WHERE { ?x a gufo:Event }'
//! ```

use oxrdf::Term;
use sparq_core::Graph;
use sparq_engine::QueryResult;
use sparq_kb::query::citations::render_citations;
use sparq_kb::query::{ask_pkg, canned, load_pkg_with_extra, nl_tool};

fn usage() -> &'static str {
    "\
pkg-query — introspect → ground → ask over the ingested PKG (sq-2m6zm.3, sq-ve5dy)

USAGE:
  pkg-query --list
  pkg-query --query <name> [--arg <value>] [--extra-graph <path>]… [--close rdfs|owl-rl] [--sparql-only] [--json]
  pkg-query --sparql '<SPARQL SELECT/ASK>' [--extra-graph <path>]… [--close rdfs|owl-rl] [--json]
  pkg-query --citations [--extra-graph <path>]… [--close rdfs|owl-rl] [--sparql-only]

OPTIONS:
  --list             list the canned introspect/ground queries and exit
  --query <name>     run a canned query by name (see --list)
  --arg <value>      argument for a parameterised canned query ({ARG} substitution)
  --sparql <query>   run a raw SPARQL SELECT/ASK string
  --citations        run the finding-provenance canned query and render each
                     Finding with its [source] citation + the citation metrics
                     (resolution rate, fabricated count) — sq-2489d.11
  --extra-graph <p>  load an extra Turtle file alongside the PKG before querying
                     (repeatable; e.g. an FO overlay — epic sq-mztg8 Metric 1).
                     Its triples participate in --close.
  --close <profile>  materialise RDFS or OWL-RL closure first (needs --features close)
  --sparql-only      print the executed SPARQL but do not run it (for verification)
  --json             emit the NL-tool envelope as JSON (answer + executed SPARQL +
                     resolved IRIs + grounding confidence) — the agent-flavor tool
                     output (sq-ve5dy)
  -h, --help         show this help
"
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("pkg-query: {msg}\n\n{}", usage());
            std::process::ExitCode::FAILURE
        }
    }
}

/// Consume the value following a flag at `args[*i]`, advancing `i` past it.
fn next<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    *i += 1;
    args.get(*i)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn run(args: &[String]) -> Result<(), String> {
    let mut query_name: Option<&str> = None;
    let mut raw_sparql: Option<&str> = None;
    let mut arg: Option<&str> = None;
    let mut close: Option<&str> = None;
    let mut sparql_only = false;
    let mut json = false;
    let mut citations = false;
    let mut extra_paths: Vec<&str> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print!("{}", usage());
                return Ok(());
            }
            "--list" => {
                list_canned();
                return Ok(());
            }
            "--query" => {
                query_name = Some(next(args, &mut i, "--query")?);
            }
            "--sparql" => {
                raw_sparql = Some(next(args, &mut i, "--sparql")?);
            }
            "--arg" => {
                arg = Some(next(args, &mut i, "--arg")?);
            }
            "--extra-graph" => {
                extra_paths.push(next(args, &mut i, "--extra-graph")?);
            }
            "--close" => {
                close = Some(next(args, &mut i, "--close")?);
            }
            "--sparql-only" => sparql_only = true,
            "--json" => json = true,
            "--citations" => citations = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
        i += 1;
    }

    // Resolve the SPARQL to run. --citations is its own mode: it always runs the
    // finding-provenance canned query, so it cannot be combined with --query/--sparql
    // (and --json stays the nl_tool envelope, which has no citation fields).
    let sparql: String = if citations {
        if query_name.is_some() || raw_sparql.is_some() {
            return Err("--citations runs the finding-provenance canned query; do not combine it with --query/--sparql".into());
        }
        if json {
            return Err("--citations has no --json form (the NL-tool envelope carries no citation fields)".into());
        }
        canned::FINDING_PROVENANCE.render(None)
    } else {
        match (query_name, raw_sparql) {
            (Some(_), Some(_)) => return Err("pass either --query or --sparql, not both".into()),
            (None, None) => {
                return Err(
                    "nothing to run — pass --query <name>, --sparql '<…>', --citations or --list"
                        .into(),
                )
            }
            (Some(name), None) => {
                let q = canned::by_name(name)
                    .ok_or_else(|| format!("no canned query named `{name}` (try --list)"))?;
                q.render(arg)
            }
            (None, Some(s)) => s.to_string(),
        }
    };

    // Surface the executed SPARQL (the design's "surface the executed SPARQL" rule).
    eprintln!(
        "--- executed SPARQL ---\n{}\n-----------------------",
        sparql.trim()
    );
    if sparql_only {
        return Ok(());
    }

    // Read any --extra-graph files (e.g. an FO overlay) into owned Turtle docs to load
    // alongside the PKG. Read once here so both the table and --json paths see the same
    // data; a missing/unreadable file is a hard error, not a silent skip.
    let extra_docs: Vec<String> = extra_paths
        .iter()
        .map(|p| {
            std::fs::read_to_string(p)
                .map_err(|e| format!("cannot read --extra-graph `{}`: {}", p, e))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if !extra_docs.is_empty() {
        eprintln!(
            "--- {} extra graph(s) loaded: {} ---",
            extra_docs.len(),
            extra_paths.join(", ")
        );
    }

    // Load the store (optionally closed), with any extra graphs merged in first.
    let graph = load_graph(close, &extra_docs)?;

    // --json: emit the NL-tool envelope (sq-ve5dy) — answer + executed SPARQL +
    // resolved IRIs + grounding confidence — the agent-flavor tool output. Canned
    // queries carry `Confidence::Canned`; a raw query's confidence is derived from its
    // dictionary grounding.
    if json {
        let envelope = match query_name {
            Some(name) => nl_tool::run_canned(&graph, name, arg)?,
            None => nl_tool::run_raw(&graph, &sparql)?,
        };
        println!("{}", envelope.to_json());
        return Ok(());
    }

    // --citations: render each Finding with its [source] citation, then the MEASURED
    // metrics (sq-2489d.11 / sq-pdvx7). Rendered text on stdout; metrics on stderr,
    // mirroring the "N row(s)." convention.
    if citations {
        let result = ask_pkg(&graph, &sparql)?;
        let report = render_citations(&graph, &result);
        print!("{}", report.rendered_text);
        eprintln!(
            "\n{} citation(s); {} fabricated; resolution rate {:.2}.",
            report.metrics.total_citations,
            report.metrics.fabricated_count,
            report.metrics.citation_resolution_rate()
        );
        return Ok(());
    }

    // Default human/table output.
    let result = ask_pkg(&graph, &sparql)?;
    print_table(&result);
    eprintln!("\n{} row(s).", result.rows.len());
    Ok(())
}

/// Load the PKG store (plus any `extra_docs`), optionally under an RDFS/OWL-RL closure.
/// The closure path is only available when the crate is built with `--features close`.
/// Shared by both the table output and the `--json` NL-tool envelope so the two cannot
/// diverge on what data they answer over.
fn load_graph(close: Option<&str>, extra_docs: &[String]) -> Result<Graph, String> {
    let extra: Vec<&str> = extra_docs.iter().map(String::as_str).collect();
    match close {
        None => load_pkg_with_extra(&extra),
        Some(profile) => load_closed(profile, &extra),
    }
}

#[cfg(feature = "close")]
fn load_closed(profile: &str, extra: &[&str]) -> Result<Graph, String> {
    use sparq_kb::query::close::{load_pkg_closed_with_extra, Profile};
    let profile = Profile::parse(profile)
        .ok_or_else(|| format!("unknown closure profile `{profile}` (use rdfs|owl-rl)"))?;
    let (g, entailed) = load_pkg_closed_with_extra(profile, extra)?;
    eprintln!("--- closure: {entailed} entailed triple(s) materialised ---");
    Ok(g)
}

#[cfg(not(feature = "close"))]
fn load_closed(_profile: &str, _extra: &[&str]) -> Result<Graph, String> {
    Err("--close needs the `close` feature: rebuild with --features close".into())
}

/// List every canned query (name + description + whether it takes an argument).
fn list_canned() {
    println!("Canned PKG queries (introspect → ground → ask):\n");
    for q in canned::ALL {
        let param = match q.param {
            Some((label, default)) => format!("  [--arg <{label}>; default: {default}]"),
            None => String::new(),
        };
        println!("  {:<24}{}", q.name, q.about);
        if !param.is_empty() {
            println!("  {:<24}{param}", "");
        }
    }
    println!("\nRun one with:  pkg-query --query <name> [--arg <value>]");
}

/// Pretty-print a result set as a header + rows (short-IRI local names; truncated long
/// literals) so the answer is human- and agent-readable.
fn print_table(r: &QueryResult) {
    let header: Vec<String> = r.vars.iter().map(|v| v.as_str().to_string()).collect();
    println!("{}", header.join("  |  "));
    println!("{}", "-".repeat(header.join("  |  ").len().max(3)));
    for row in &r.rows {
        let cells: Vec<String> = row.iter().map(cell).collect();
        println!("{}", cells.join("  |  "));
    }
}

/// Render one result cell: short local name for an IRI, truncated value for a literal.
fn cell(c: &Option<Term>) -> String {
    match c {
        Some(Term::NamedNode(n)) => n
            .as_str()
            .rsplit(['#', '/'])
            .next()
            .unwrap_or("")
            .to_string(),
        Some(Term::Literal(l)) => {
            let s = l.value();
            if s.chars().count() > 110 {
                let cut: String = s.chars().take(110).collect();
                format!("{cut}…")
            } else {
                s.to_string()
            }
        }
        Some(t) => t.to_string(),
        None => "(unbound)".into(),
    }
}
