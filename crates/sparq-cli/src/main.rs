//! sparq command-line interface.
//!
//!   sparq-cli query <data-file> <format> <sparql>
//!       load a file and run one query, printing the solution count.
//!
//!   sparq-cli bench <data-file> <format> <queries-dir> [iters] [mode]
//!       load the file once, then run every `*.rq` file in <queries-dir>
//!       (sorted by name) `iters` times each, printing one TSV line per query:
//!           <name>\t<rows>\t<min_micros>
//!       Load time + triple count go to stderr. Used by the QLever comparison.
//!       mode: `count`       — compute only (no term materialisation)
//!             `materialize` — full QueryResult of terms (default)
//!             `json`        — materialise + serialise to SPARQL JSON (timed)
//!
//! `format`: turtle | ntriples | nquads | trig.

use std::io::Read;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("query") => cmd_query(&args),
        Some("bench") => cmd_bench(&args),
        Some("ingest") => cmd_ingest(&args),
        _ => {
            eprintln!("usage:\n  sparq-cli query <data-file> <format> <sparql>\n  sparq-cli bench <data-file> <format> <queries-dir> [iters]\n  sparq-cli ingest <file[.gz|.bz2]> [parse|intern|full] [max_millions]");
            std::process::exit(2);
        }
    }
}

/// Streaming-ingest throughput experiment (for the Wikidata-vs-RDFox comparison).
/// Decompresses (.gz/.bz2) and parses N-Triples from a stream, never holding the
/// whole document on disk or in memory:
///   parse  — decompress + parse + count only (constant memory, unbounded): the
///            raw front-end ceiling.
///   intern — + dictionary interning (memory grows with distinct terms); capped.
///   full   — + collect triples and build the six permutation indexes; capped.
/// Reports triples/s and extrapolates to full Wikidata truthy (~8.0B triples).
fn cmd_ingest(args: &[String]) {
    let path = match args.get(2) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: sparq-cli ingest <file[.gz|.bz2]> [parse|intern|full] [max_millions]");
            std::process::exit(2);
        }
    };
    let mode = args.get(3).map(String::as_str).unwrap_or("parse");
    let cap: u64 = args.get(4).and_then(|s| s.parse::<u64>().ok()).map(|m| m * 1_000_000).unwrap_or(u64::MAX);

    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        eprintln!("open {path}: {e}");
        std::process::exit(1);
    });
    let decoded: Box<dyn Read> = if path.ends_with(".bz2") {
        Box::new(bzip2::read::MultiBzDecoder::new(file))
    } else if path.ends_with(".gz") {
        Box::new(flate2::read::MultiGzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let reader = std::io::BufReader::with_capacity(1 << 22, decoded);

    let mut dict = sparq_core::dict::Dict::new();
    let mut triples: Vec<[u32; 3]> = Vec::new();
    let mut n: u64 = 0u64;
    let t0 = Instant::now();
    let mut last = 0u64;
    let mut last_t = Instant::now();

    eprintln!("ingest mode={mode} cap={} from {path}", if cap == u64::MAX { "none".into() } else { format!("{}M", cap / 1_000_000) });

    for triple in oxttl::NTriplesParser::new().for_reader(reader) {
        let t = match triple {
            Ok(t) => t,
            Err(e) => {
                // A truncated prefix (we only downloaded part of the dump) ends in
                // a decode/parse error — expected; stop cleanly.
                eprintln!("(stream ended after {n} triples: {e})");
                break;
            }
        };
        if mode != "parse" {
            let s = dict.intern(&subject_to_term(&t.subject));
            let p = dict.intern(&oxrdf::Term::NamedNode(t.predicate.clone()));
            let o = dict.intern(&t.object);
            if mode == "full" {
                triples.push([s, p, o]);
            }
        }
        n += 1;
        if n - last >= 5_000_000 {
            let dt = last_t.elapsed().as_secs_f64();
            eprintln!(
                "  {n:>12} triples  |  {:.2} M/s (window)  |  {:.2} M/s (avg)  |  {} distinct terms",
                (n - last) as f64 / 1e6 / dt,
                n as f64 / 1e6 / t0.elapsed().as_secs_f64(),
                dict.len()
            );
            last = n;
            last_t = Instant::now();
        }
        if n >= cap {
            eprintln!("(reached cap of {} triples)", cap);
            break;
        }
    }

    let parse_secs = t0.elapsed().as_secs_f64();
    let mut build_secs = 0.0;
    if mode == "full" && !triples.is_empty() {
        let tb = Instant::now();
        let store = sparq_core::store::TripleStore::from_triples(triples);
        build_secs = tb.elapsed().as_secs_f64();
        std::hint::black_box(&store);
    }

    let total = parse_secs + build_secs;
    let rate = n as f64 / total.max(1e-9);
    println!("\n=== ingest summary ({mode}) ===");
    println!("triples ingested : {n}");
    if mode != "parse" {
        println!("distinct terms   : {}", dict.len());
    }
    println!("decompress+parse : {parse_secs:.1}s");
    if mode == "full" {
        println!("index build      : {build_secs:.1}s (6 permutations)");
    }
    println!("total            : {total:.1}s");
    println!("throughput       : {:.2} M triples/s", rate / 1e6);
    // Extrapolate to full Wikidata truthy (~8.0B triples).
    let wikidata = 8.0e9;
    println!("extrapolated full Wikidata truthy (~8.0B triples): {:.0} min ({:.1} h) at this rate", wikidata / rate / 60.0, wikidata / rate / 3600.0);
}

fn subject_to_term(s: &oxrdf::NamedOrBlankNode) -> oxrdf::Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => oxrdf::Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => oxrdf::Term::BlankNode(b.clone()),
    }
}

fn load(path: &str, format: &str) -> sparq_core::Graph {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        std::process::exit(1);
    });
    let t = Instant::now();
    let g = sparq_core::Graph::load_str(&text, format).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        std::process::exit(1);
    });
    let secs = t.elapsed().as_secs_f64();
    let heap = g.heap_bytes();
    let dict = g.dict.heap_bytes();
    eprintln!(
        "loaded {} triples in {:.3}s ({:.2} M/s) | store ~{:.2} GB ({:.0} B/triple), dict ~{:.2} GB ({} terms, {:.0} B/term)",
        g.len(),
        secs,
        g.len() as f64 / 1e6 / secs,
        heap as f64 / 1e9,
        heap as f64 / g.len().max(1) as f64,
        dict as f64 / 1e9,
        g.dict.len(),
        dict as f64 / g.dict.len().max(1) as f64,
    );
    g
}

fn cmd_query(args: &[String]) {
    let (path, format, sparql) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(q)) => (p, f, q),
        _ => {
            eprintln!("usage: sparq-cli query <data-file> <format> <sparql>");
            std::process::exit(2);
        }
    };
    let g = load(path, format);
    let t = Instant::now();
    match sparq_engine::query(&g, sparql) {
        Ok(r) => println!("{} solutions in {:.3}ms", r.len(), t.elapsed().as_secs_f64() * 1e3),
        Err(e) => {
            eprintln!("query error: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_bench(args: &[String]) {
    let (path, format, dir) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(d)) => (p, f, d),
        _ => {
            eprintln!("usage: sparq-cli bench <data-file> <format> <queries-dir> [iters]");
            std::process::exit(2);
        }
    };
    let iters: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(5);
    let mode = args.get(6).map(String::as_str).unwrap_or("materialize");
    if !matches!(mode, "count" | "materialize" | "json") {
        eprintln!("unknown mode `{mode}`; expected count | materialize | json");
        std::process::exit(2);
    }
    let g = load(path, format);

    // Collect *.rq files, sorted by name.
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| {
            eprintln!("error reading {dir}: {e}");
            std::process::exit(1);
        })
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "rq").unwrap_or(false))
        .collect();
    entries.sort();

    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let sparql = std::fs::read_to_string(&path).unwrap();
        // Warm + measure min over `iters` runs in the requested mode.
        let mut best = f64::INFINITY;
        let mut rows = 0;
        let mut err = None;
        for _ in 0..iters {
            let t = Instant::now();
            let r: Result<usize, String> = match mode {
                "count" => sparq_engine::count(&g, &sparql),
                "json" => sparq_engine::query(&g, &sparql).map(|r| {
                    let s = sparq_engine::json::to_sparql_json(&r);
                    std::hint::black_box(&s);
                    r.len()
                }),
                _ => sparq_engine::query(&g, &sparql).map(|r| r.len()),
            };
            match r {
                Ok(n) => {
                    rows = n;
                    best = best.min(t.elapsed().as_secs_f64() * 1e6);
                }
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }
        match err {
            None => println!("{name}\t{rows}\t{best:.1}"),
            Some(e) => println!("{name}\tERROR\t{e}"),
        }
    }
}
