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

use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("query") => cmd_query(&args),
        Some("bench") => cmd_bench(&args),
        _ => {
            eprintln!("usage:\n  sparq-cli query <data-file> <format> <sparql>\n  sparq-cli bench <data-file> <format> <queries-dir> [iters]");
            std::process::exit(2);
        }
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
    eprintln!("loaded {} triples in {:.3}s", g.len(), t.elapsed().as_secs_f64());
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
