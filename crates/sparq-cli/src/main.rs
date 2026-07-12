#![doc = include_str!("../README.md")]
// [OPUS-4.8] MS-G2 (sq-8wbn): make `// SAFETY:` mandatory on every first-party unsafe block.
#![warn(clippy::undocumented_unsafe_blocks)]

use std::io::Read;
use std::time::Instant;

// [FABLE-5] (sq-lsp7k.8) Materializing tabular→RDF import (CSV direct mapping + the R2RML
// materializing subset over CSV logical tables). The whole module exists only under the
// opt-in `tabular` feature; the default CLI build carries none of it.
#[cfg(feature = "tabular")]
mod tabular;

// T1.0 scaling lever: replace the system allocator (whose per-thread arena locks contend under
// rayon's many-worker per-row allocation) with mimalloc (sharded, lock-light). Compile-time;
// `--no-default-features --features mmap` builds with the system allocator for A/B.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("query") => cmd_query(&args),
        Some("reason") => cmd_reason(&args),
        Some("bench") => cmd_bench(&args),
        Some("memstat") => cmd_memstat(&args),
        Some("bench-mmap") => cmd_bench_mmap(&args),
        Some("ingest") => cmd_ingest(&args),
        Some("save") => cmd_save(&args),
        Some("recompress") => cmd_recompress(&args),
        Some("compact") => cmd_compact(&args),
        Some("build") => cmd_build(&args),
        Some("query-mmap") => cmd_query_mmap(&args),
        Some("probe-compress") => cmd_probe_compress(&args),
        Some("compare-compress") => cmd_compare_compress(&args),
        Some("bench-remap") => cmd_bench_remap(&args),
        Some("scaling") => cmd_scaling(&args),
        // [OPUS-4.8] (sq-678h) `dump` re-serializes a loaded RDF document into the writer
        // matrix (turtle / trig / nquads). Gated behind the opt-in `serialize-rdf` feature:
        // the default CLI build carries no serializer code, so the subcommand is only present
        // when built with `--features serialize-rdf`.
        #[cfg(feature = "serialize-rdf")]
        Some("dump") => cmd_dump(&args),
        // [OPUS-4.8] (sq-vczh2) `terse` transpiles a terse query (the `K:<name>` keyword layer
        // over canonical SPARQL) into the canonical SPARQL it expands to, printing the verifiable
        // JSON contract. Gated behind the opt-in `terse` feature: the default CLI build carries no
        // terse code, so the subcommand is only present when built with `--features terse`.
        #[cfg(feature = "terse")]
        Some("terse") => cmd_terse(&args),
        // [FABLE-5] (sq-8ju74) `to-hdt` EXPORTS a loaded RDF document as a standard-layout
        // HDT v1.0 archive via sparq-hdt's direct in-memory encoder. Gated behind the opt-in
        // `hdt-write` cargo feature (which implies `hdt`, the loader): the default CLI build
        // carries no HDT code, so the subcommand is only present when built with
        // `--features hdt-write`.
        #[cfg(feature = "hdt-write")]
        Some("to-hdt") => cmd_to_hdt(&args),
        // [FABLE-5] (sq-lsp7k.8) `tabular` materializes RDF from CSV — direct mapping by
        // default, R2RML (CSV logical tables) with `--mapping` — then loads the graph (and
        // optionally queries it) or streams N-Triples to `--out`. Gated behind the opt-in
        // `tabular` feature: the default CLI build carries no CSV/R2RML code and the
        // subcommand is absent.
        #[cfg(feature = "tabular")]
        Some("tabular") => tabular::cmd_tabular(&args),
        _ => {
            eprintln!("usage:\n  sparq-cli query <data-file> <format> <sparql> [--format <table|tsv|csv|xml|json|ntriples>] [--count]\n  sparq-cli bench <data-file> <format> <queries-dir> [iters]\n  sparq-cli memstat <data-file> <format> [compressed]  # deterministic memory-composition breakdown (B/triple) + RSS\n  sparq-cli ingest <file[.gz|.bz2]> [parse|intern|full] [max_millions]\n  sparq-cli save <data-file> <format> <dir> [compressed]  # build + persist indexes to disk\n  sparq-cli recompress <src-dir> <dst-dir>          # re-persist with block-compressed indexes\n  sparq-cli compact <persist-dir>                   # WAL compact/vacuum: physically purge erased data (offline)\n  sparq-cli query-mmap <dir> <sparql> [--format <table|tsv|csv|xml|json|ntriples>] [--count]  # query with indexes MEMORY-MAPPED (out-of-core)\n\n  env SPARQ_STORE_PROFILE=raw|compressed selects the in-RAM store profile on the shared load path (query/bench/reason/scaling); unset=raw; unknown value is a hard error");
            std::process::exit(2);
        }
    }
}

/// Isolated micro-benchmark of the latency-bound dict-remap gather, to measure the per-ISA
/// software prefetch in isolation (undiluted by parsing) on each hardware target.
///   sparq-cli bench-remap n_triples dict_size iters
/// Run twice — once normally, once with SPARQ_NO_PREFETCH=1 — to get the prefetch delta.
fn cmd_bench_remap(args: &[String]) {
    let n: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000_000);
    let dict: usize = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000_000);
    let iters: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(5);
    let pf = std::env::var("SPARQ_NO_PREFETCH").as_deref() != Ok("1");
    let ms = sparq_core::bench_remap(n, dict, iters);
    let mtps = (n as f64) / (ms / 1e3) / 1e6;
    println!("remap\tn={n}\tdict={dict}\tprefetch={pf}\tbest_ms={ms:.2}\tMtriples_s={mtps:.2}");
}

/// Per-subsystem parallel SCALING harness (roadmap T8). Builds fixed-size rayon thread pools
/// (1,2,4,8,…) and runs each subsystem inside `pool.install()`, so the engine's `par_iter`/
/// `par_chunks` (which size off `rayon::current_num_threads()`) use exactly that many threads —
/// sweeping the thread count in ONE process. Reports best time, speedup vs the smallest pool, and
/// parallel EFFICIENCY (speedup ÷ thread-ratio; 1.0 = perfectly linear) per subsystem, so you can
/// see precisely where each part plateaus. On a many-core box pass e.g. `1,2,4,8,16,32,64,128,192`.
///   sparq-cli scaling `<data-file>` `<format>` `<queries-dir>` [threads=auto] [iters=3]
fn cmd_scaling(args: &[String]) {
    let (path, format, qdir) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(d)) => (p.clone(), f.clone(), d.clone()),
        _ => {
            eprintln!("usage: sparq-cli scaling <data-file> <format> <queries-dir> [threads=1,2,4,8,…] [iters=3]");
            std::process::exit(2);
        }
    };
    let threads: Vec<usize> = match args.get(5) {
        Some(s) => {
            let mut v: Vec<usize> = s
                .split(',')
                .filter_map(|t| t.trim().parse().ok())
                .filter(|&n| n >= 1)
                .collect();
            v.dedup();
            v
        }
        None => {
            let max = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(8);
            let mut v = vec![1usize];
            let mut n = 2;
            while n < max {
                v.push(n);
                n *= 2;
            }
            if *v.last().unwrap() != max {
                v.push(max);
            }
            v
        }
    };
    let iters: usize = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(3);
    if threads.is_empty() {
        eprintln!("no valid thread counts");
        std::process::exit(2);
    }
    let base = threads[0];
    eprintln!(
        "scaling sweep: threads={threads:?} iters={iters}  (efficiency = speedup ÷ (threads/{base}); 1.0 = linear)"
    );
    let pool = |n: usize| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .expect("rayon pool")
    };

    println!("subsystem\tthreads\tbest_ms\tspeedup\tefficiency");

    // LOAD subsystem — re-load the file inside each pool size (parse + dict-merge + 6 perms).
    {
        let mut t_base = 0.0f64;
        for (i, &n) in threads.iter().enumerate() {
            let p = pool(n);
            let mut best = f64::INFINITY;
            for _ in 0..iters {
                let t = Instant::now();
                let g = p.install(|| load_quiet(&path, &format));
                best = best.min(t.elapsed().as_secs_f64() * 1e3);
                std::hint::black_box(&g);
            }
            if i == 0 {
                t_base = best;
            }
            let sp = t_base / best;
            let eff = sp / (n as f64 / base as f64);
            println!("load\t{n}\t{best:.1}\t{sp:.2}\t{eff:.2}");
        }
    }

    // QUERY subsystems — load once, run each query at each pool size in `materialize` form (compute
    // all bindings, no serialization): that is the parallel compute we want to scale.
    let g = load(&path, &format);
    let mut queries: Vec<(String, String)> = std::fs::read_dir(&qdir)
        .unwrap_or_else(|e| {
            eprintln!("read {qdir}: {e}");
            std::process::exit(1);
        })
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "rq").unwrap_or(false))
        .filter_map(|p| {
            let name = p.file_stem()?.to_string_lossy().into_owned();
            Some((name, std::fs::read_to_string(&p).ok()?))
        })
        .collect();
    queries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, sparql) in &queries {
        let mut t_base = 0.0f64;
        let mut ok = true;
        for (i, &n) in threads.iter().enumerate() {
            let p = pool(n);
            let mut best = f64::INFINITY;
            for _ in 0..iters {
                let t = Instant::now();
                match p.install(|| sparq_engine::query(&g, sparql).map(|r| r.len())) {
                    Ok(rows) => {
                        best = best.min(t.elapsed().as_secs_f64() * 1e3);
                        std::hint::black_box(rows);
                    }
                    Err(e) => {
                        eprintln!("{name}: query error: {e}");
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                break;
            }
            if i == 0 {
                t_base = best;
            }
            let sp = t_base / best;
            let eff = sp / (n as f64 / base as f64);
            println!("{name}\t{n}\t{best:.1}\t{sp:.2}\t{eff:.2}");
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
            eprintln!(
                "usage: sparq-cli ingest <file[.gz|.bz2]> [parse|intern|full] [max_millions]"
            );
            std::process::exit(2);
        }
    };
    let mode = args.get(3).map(String::as_str).unwrap_or("parse");
    let cap: u64 = args
        .get(4)
        .and_then(|s| s.parse::<u64>().ok())
        .map(|m| m * 1_000_000)
        .unwrap_or(u64::MAX);

    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        eprintln!("open {path}: {e}");
        std::process::exit(1);
    });
    // `+ Send` so the build can run decompression on its own (overlapping) thread.
    let decoded: Box<dyn Read + Send> = if path.ends_with(".bz2") {
        Box::new(bzip2::read::MultiBzDecoder::new(file))
    } else if path.ends_with(".gz") {
        Box::new(flate2::read::MultiGzDecoder::new(file))
    } else if path.ends_with(".zst") || path.ends_with(".zstd") {
        // zstd decompresses ~12x faster than bzip2, so it stops being the ingestion
        // bottleneck; recompress a .bz2 source once with `zstd -9 -T0` to use it.
        Box::new(zstd::stream::read::Decoder::new(file).unwrap_or_else(|e| {
            eprintln!("zstd decode {path}: {e}");
            std::process::exit(1);
        }))
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

    eprintln!(
        "ingest mode={mode} cap={} from {path}",
        if cap == u64::MAX {
            "none".into()
        } else {
            format!("{}M", cap / 1_000_000)
        }
    );

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
    println!(
        "extrapolated full Wikidata truthy (~8.0B triples): {:.0} min ({:.1} h) at this rate",
        wikidata / rate / 60.0,
        wikidata / rate / 3600.0
    );
}

fn subject_to_term(s: &oxrdf::NamedOrBlankNode) -> oxrdf::Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => oxrdf::Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => oxrdf::Term::BlankNode(b.clone()),
    }
}

/// `save <data> <format> <dir> [compressed]` — build the store and persist its indexes
/// to disk; `compressed` writes the block-compressed permutation format (auto-detected
/// by `query-mmap`/`bench-mmap`).
fn cmd_save(args: &[String]) {
    let (path, format, dir) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(d)) => (p.as_str(), f.as_str(), d.as_str()),
        _ => {
            eprintln!("usage: sparq-cli save <data-file> <format> <dir> [compressed]");
            std::process::exit(2);
        }
    };
    let compressed = args.get(5).map(String::as_str) == Some("compressed");
    let g = load(path, format);
    let t = Instant::now();
    let res = if compressed {
        g.save_compressed(std::path::Path::new(dir))
    } else {
        g.save(std::path::Path::new(dir))
    };
    res.unwrap_or_else(|e| {
        eprintln!("save error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "saved {} triples to {dir}{} in {:.3}s",
        g.len(),
        if compressed {
            " (compressed perms)"
        } else {
            ""
        },
        t.elapsed().as_secs_f64()
    );
}

/// `recompress <src-dir> <dst-dir>` — re-persist a saved dataset with block-compressed
/// permutation indexes (the dictionary/numerics are rewritten unchanged). Lets a raw
/// (e.g. external-memory `build`) directory be compacted without re-parsing the source.
fn cmd_recompress(args: &[String]) {
    let (src, dst) = match (args.get(2), args.get(3)) {
        (Some(s), Some(d)) if s != d => (s.as_str(), d.as_str()),
        _ => {
            eprintln!("usage: sparq-cli recompress <src-dir> <dst-dir>   (dirs must differ)");
            std::process::exit(2);
        }
    };
    let g = sparq_core::Graph::open(std::path::Path::new(src)).unwrap_or_else(|e| {
        eprintln!("open error: {e}");
        std::process::exit(1);
    });
    let t = Instant::now();
    g.save_compressed(std::path::Path::new(dst))
        .unwrap_or_else(|e| {
            eprintln!("save error: {e}");
            std::process::exit(1);
        });
    eprintln!(
        "recompressed {} triples {src} -> {dst} in {:.3}s",
        g.len(),
        t.elapsed().as_secs_f64()
    );
}

/// [OPUS-4.8] (sq-x32t) `compact <persist-dir>` — WAL COMPACTION / VACUUM for ERASURE-
/// COMPLETENESS (epic sq-toze.33). An OFFLINE operator command: stop the server, run this on its
/// `--persist <dir>`, restart. A logical SPARQL `DELETE` / `DROP GRAPH` retracts data from the
/// live view but leaves the superseded bytes in earlier WAL segments until a compaction folds the
/// live state into a fresh base; this command does exactly that — physically rewriting the store
/// to contain ONLY the current live triples, so erased data is gone from the on-disk history.
///
/// `Graph::open` replays the WAL into the live overlay, `Graph::vacuum` re-interns the live
/// triples into a fresh dictionary (so orphaned term VALUES are purged too) and ATOMICALLY
/// (rollback-safe two-rename swap, parent dir fsync'd between renames, WAL truncated) replaces the
/// on-disk store; an interrupted swap is healed deterministically on the next open. The live
/// triple set is preserved exactly (round-trip).
///
/// Online equivalent: `POST /admin/compact` on a running `--persist` server (gated by the write
/// auth token). The complement on the server side runs on the writer thread between batches.
///
/// PHYSICAL-ERASURE CAVEAT (honest scope): this scrubs the engine's own on-disk segments; it
/// cannot reach bytes already copied off-box — filesystem snapshots, block-level COW history, or
/// external backups — which the storage/backup tier must handle per the retention-erasure runbook.
fn cmd_compact(args: &[String]) {
    let dir = match args.get(2) {
        Some(d) => d.as_str(),
        None => {
            eprintln!("usage: sparq-cli compact <persist-dir>   # offline WAL compact/vacuum for erasure-completeness");
            std::process::exit(2);
        }
    };
    let path = std::path::Path::new(dir);
    // `open` replays the WAL into the live overlay (and heals any interrupted prior compaction).
    let mut g = sparq_core::Graph::open(path).unwrap_or_else(|e| {
        eprintln!("open error ({dir}): {e}");
        std::process::exit(1);
    });
    let before = g.len();
    let t = Instant::now();
    // [OPUS-4.8] (sq-x32t) ERASURE-GRADE vacuum: atomic, crash-safe rewrite of the on-disk store
    // to only the live triples + a fresh dictionary (so orphaned term VALUES are purged too) +
    // WAL truncate. `vacuum` re-interns; the lighter `Graph::compact` keeps the dict and would
    // leave a deleted literal's bytes on disk, which is not erasure-complete.
    g.vacuum().unwrap_or_else(|e| {
        eprintln!("compact error ({dir}): {e}");
        std::process::exit(1);
    });
    let after = g.len();
    // `len` is invariant across compaction (the live set is preserved exactly); print it as a
    // round-trip sanity signal, not as a deletion count (the purged data was already retracted).
    eprintln!(
        "compacted {dir}: {} live triples (was {before}) in {:.3}s — superseded/erased data physically removed from the on-disk WAL history",
        after,
        t.elapsed().as_secs_f64()
    );
}

/// `build <file[.gz|.bz2]> <format> <dir> [chunk_millions]` — EXTERNAL-MEMORY build:
/// stream the (optionally compressed) document straight to on-disk, memory-mapped indexes
/// via disk-backed sort/merge, so a dataset whose indexes exceed RAM can be constructed on
/// a small machine. `chunk_millions` (default 16) sets the in-memory run size (16M triples
/// ≈ 192 MB of ids); lower it to cap memory further. Query the result with `query-mmap`.
///
/// SPARQ_DICT_SPILL=1 additionally SPILLS the term dictionary (N-Triples only): peak RSS
/// is bounded by SPARQ_DICT_SPILL_BUDGET_MB (default: 1/4 of RAM) instead of growing with
/// distinct terms; SPARQ_DICT_SPILL_DISK_FLOOR_MB (default 1024) aborts before filling the
/// disk. Output is byte-identical. See research/external-dictionary.md.
fn cmd_build(args: &[String]) {
    let (path, format, dir) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(d)) => (p.as_str(), f.as_str(), d.as_str()),
        _ => {
            // [OPUS-4.8] sq-vkz7: `SPARQ_BUILD_COMPRESSED=1` makes the external build emit
            // block-compressed (SPQCPRM1) perms in ONE pass — skipping a later `recompress`.
            eprintln!(
                "usage: sparq-cli build <file[.gz|.bz2]> <format> <dir> [chunk_millions]\n  \
                 (set SPARQ_BUILD_COMPRESSED=1 to write block-compressed perms directly, no recompress pass)"
            );
            std::process::exit(2);
        }
    };
    // [OPUS-4.8] `build` calls Graph::build_external directly (not load_quiet), which shares
    // the same Turtle catch-all — so reject an unknown format here too (bug sq-q50l). HDT is
    // not a build target (build streams text formats), so it is excluded from the accepted set.
    if !is_known_format(format) || format == "hdt" {
        die_unknown_format(format);
    }
    let chunk = args
        .get(5)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(16)
        * 1_000_000;

    let file = std::fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("open {path}: {e}");
        std::process::exit(1);
    });
    // `+ Send` so the build can run decompression on its own (overlapping) thread.
    let decoded: Box<dyn Read + Send> = if path.ends_with(".bz2") {
        Box::new(bzip2::read::MultiBzDecoder::new(file))
    } else if path.ends_with(".gz") {
        Box::new(flate2::read::MultiGzDecoder::new(file))
    } else if path.ends_with(".zst") || path.ends_with(".zstd") {
        // zstd decompresses ~12x faster than bzip2, so it stops being the ingestion
        // bottleneck; recompress a .bz2 source once with `zstd -9 -T0` to use it.
        Box::new(zstd::stream::read::Decoder::new(file).unwrap_or_else(|e| {
            eprintln!("zstd decode {path}: {e}");
            std::process::exit(1);
        }))
    } else {
        Box::new(file)
    };
    let reader = std::io::BufReader::with_capacity(1 << 22, decoded);

    let t = Instant::now();
    // [OPUS-4.8] (sq-5atq) Quad formats (N-Quads / TriG) build OUT-OF-CORE *with* their named
    // graphs via `build_external_quads` (partition-by-graph + per-graph external sort, same
    // on-disk layout `save_named` emits) — a default-graph-only `build_external` would silently
    // FLATTEN every quad into the default graph, losing the dataset's named graphs (the PSS
    // shape). Triple formats keep the existing single-graph external path unchanged.
    let build_result = match format {
        "nquads" | "n-quads" | "trig" | "application/trig" => {
            sparq_core::Graph::build_external_quads(
                reader,
                format,
                std::path::Path::new(dir),
                chunk,
            )
        }
        _ => sparq_core::Graph::build_external(reader, format, std::path::Path::new(dir), chunk),
    };
    build_result.unwrap_or_else(|e| {
        eprintln!("build error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "built on-disk indexes in {dir} in {:.1}s (external-memory, {}M-triple runs)",
        t.elapsed().as_secs_f64(),
        chunk / 1_000_000,
    );
}

/// `probe-compress <perm-file>` — MEASURE-FIRST probe (no engine change): how small can a
/// sorted permutation index get? Reports raw vs a lexicographic delta + LEB128-varint
/// encoding (the natural columnar scheme for sorted triples) vs gzip, in bytes/triple, so
/// we can decide whether a block-compressed backend is worth building before building it.
fn cmd_probe_compress(args: &[String]) {
    let path = match args.get(2) {
        Some(p) => p.as_str(),
        None => {
            eprintln!("usage: sparq-cli probe-compress <perm-file>");
            std::process::exit(2);
        }
    };
    let file = std::fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("open {path}: {e}");
        std::process::exit(1);
    });
    // SAFETY: read-only mmap of a file held open for the call.
    let map = unsafe { memmap2::Mmap::map(&file) }.unwrap();
    let n = map.len() / 12;
    // SAFETY: a permutation file is a whole number of [u32;3] rows.
    let rows: &[[u32; 3]] =
        unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[u32; 3]>(), n) };
    if n == 0 {
        eprintln!("empty permutation");
        return;
    }

    // LEB128 byte length of a u64.
    let vlen = |mut x: u64| -> usize {
        let mut b = 1;
        while x >= 0x80 {
            x >>= 7;
            b += 1;
        }
        b
    };

    // Lexicographic delta: per row emit 3 varints (d0, x1, x2). When the higher column
    // changes the lower one is absolute; otherwise it is a (non-negative) delta. Exactly
    // decodable from the previous row — the standard sorted-triple columnar encoding.
    let t = Instant::now();
    let mut delta_bytes = 0usize;
    let mut prev = [0u32; 3];
    for &r in rows {
        if r[0] != prev[0] {
            delta_bytes += vlen((r[0] - prev[0]) as u64) + vlen(r[1] as u64) + vlen(r[2] as u64);
        } else if r[1] != prev[1] {
            delta_bytes += 1 + vlen((r[1] - prev[1]) as u64) + vlen(r[2] as u64);
        } else {
            delta_bytes += 1 + 1 + vlen(r[2].wrapping_sub(prev[2]) as u64);
        }
        prev = r;
    }
    let dscan = t.elapsed().as_secs_f64();

    // Block-directory overhead for random access: one (12-byte key + 8-byte offset) entry
    // per block of `B` triples.
    let block_dir = |b: usize| -> f64 { (n.div_ceil(b) * 20) as f64 / n as f64 };

    println!("permutation: {path}");
    println!("  triples           : {n}");
    println!(
        "  raw               : {:>6.2} B/triple ({:.2} GB)",
        12.0,
        map.len() as f64 / 1e9
    );
    println!(
        "  delta+varint      : {:>6.2} B/triple ({:.2} GB, {:.0}% of raw)  [{:.2} M/s decode-cost scan]",
        delta_bytes as f64 / n as f64,
        delta_bytes as f64 / 1e9,
        100.0 * delta_bytes as f64 / map.len() as f64,
        n as f64 / 1e6 / dscan,
    );
    println!(
        "  + block dir (B=128): +{:.2} B/triple  => {:.2} B/triple usable random-access",
        block_dir(128),
        delta_bytes as f64 / n as f64 + block_dir(128),
    );

    // gzip the raw bytes as a general-purpose ceiling — only for smaller files (it is slow).
    if map.len() <= 200_000_000 {
        use std::io::Write;
        let tg = Instant::now();
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(&map).unwrap();
        let gz = enc.finish().unwrap().len();
        println!(
            "  gzip(raw)         : {:>6.2} B/triple ({:.0}% of raw)  [{:.2} M/s]",
            gz as f64 / n as f64,
            100.0 * gz as f64 / map.len() as f64,
            n as f64 / 1e6 / tg.elapsed().as_secs_f64(),
        );
    } else {
        println!("  gzip(raw)         : skipped (>200 MB; delta+varint is the fast, random-accessible scheme anyway)");
    }
}

/// `compare-compress <data-file> <format> [<sparql>]` — load a dataset both raw and
/// block-compressed, report the in-RAM index footprint of each, and (if a query is given)
/// the query latency of each — the memory-vs-decode tradeoff that decides the browser
/// storage mode.
fn cmd_compare_compress(args: &[String]) {
    let (path, format) = match (args.get(2), args.get(3)) {
        (Some(p), Some(f)) => (p.as_str(), f.as_str()),
        _ => {
            eprintln!("usage: sparq-cli compare-compress <data-file> <format> [<sparql>]");
            std::process::exit(2);
        }
    };
    let sparql = args.get(4).map(String::as_str);

    let raw = load(path, format);
    let raw_heap = raw.heap_bytes();
    let raw_store = raw.store.heap_bytes();
    // Re-encode the same store compressed (keeps the dict + numeric cache).
    let t = Instant::now();
    let cmp = raw.into_compressed();
    let enc_s = t.elapsed().as_secs_f64();
    let cmp_heap = cmp.heap_bytes();
    let cmp_store = cmp.store.heap_bytes();

    println!("=== index footprint (in-RAM) ===");
    println!("  triples            : {}", cmp.len());
    println!(
        "  raw   perms        : {:>7.2} MB ({:.1} B/triple)",
        raw_store as f64 / 1e6,
        raw_store as f64 / cmp.len().max(1) as f64
    );
    println!(
        "  compressed perms   : {:>7.2} MB ({:.1} B/triple, {:.0}% of raw)  [encoded in {:.2}s]",
        cmp_store as f64 / 1e6,
        cmp_store as f64 / cmp.len().max(1) as f64,
        100.0 * cmp_store as f64 / raw_store.max(1) as f64,
        enc_s,
    );
    println!(
        "  total graph (perms+dict+numerics): raw {:.2} GB -> compressed {:.2} GB",
        raw_heap as f64 / 1e9,
        cmp_heap as f64 / 1e9,
    );

    if let Some(q) = sparql {
        // Materialise to SPARQL JSON — the heaviest path, which actually scans + (for the
        // compressed store) decodes the blocks the query touches.
        println!("\n=== query latency, MATERIALISED to JSON (min of 5) ===");
        let bench = |g: &sparq_core::Graph| -> (usize, f64) {
            let mut best = f64::MAX;
            let mut len = 0;
            for _ in 0..5 {
                let t = Instant::now();
                len = sparq_engine::query_json(g, q).map(|s| s.len()).unwrap_or(0);
                best = best.min(t.elapsed().as_secs_f64() * 1e3);
            }
            (len, best)
        };
        let (rn, rt) = bench(&raw_reload(path, format));
        let (cn, ct) = bench(&cmp);
        println!("  raw        : {rn} bytes of JSON in {rt:.3} ms");
        println!(
            "  compressed : {cn} bytes of JSON in {ct:.3} ms  ({:.2}x raw)",
            ct / rt
        );
        assert_eq!(rn, cn, "compressed returned different JSON length!");
    }
}

/// Reloads a fresh raw graph (the original was consumed by `into_compressed`).
fn raw_reload(path: &str, format: &str) -> sparq_core::Graph {
    let text = std::fs::read_to_string(path).unwrap();
    sparq_core::Graph::load_str(&text, format).unwrap()
}

/// Parses `path`, optionally materializes the entailed closure for `profile` (opt-in
/// reasoning), then builds the graph. The reasoning step runs between parse and index-build
/// (the `parse_to_triples` → `from_parts` seam), so the default no-`--reason` path is
/// untouched. Reports the closure expansion.
fn load_reasoned(path: &str, format: &str, profile: sparq_reason::Profile) -> sparq_core::Graph {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        std::process::exit(1);
    });
    let (mut dict, mut triples) = sparq_core::Graph::parse_to_triples(&text, format)
        .unwrap_or_else(|e| {
            eprintln!("parse error: {e}");
            std::process::exit(1);
        });
    let base = triples.len();
    let t = Instant::now();
    let added = sparq_reason::materialize(profile, &mut dict, &mut triples);
    eprintln!(
        "reasoned [{:?}]: {base} -> {} triples (+{added} entailed) in {:.3}s",
        profile,
        triples.len(),
        t.elapsed().as_secs_f64()
    );
    // OWL: report any detected inconsistency (cax-dw/cls-com/eq-diff/cls-nothing clashes).
    if profile == sparq_reason::Profile::OwlRl {
        let clashes = sparq_reason::inconsistencies(&dict, &triples);
        if !clashes.is_empty() {
            eprintln!("INCONSISTENT — {} clash(es) detected:", clashes.len());
            for c in &clashes {
                eprintln!("  - {c}");
            }
        }
    }
    // [FABLE-5] (sq-7d3dj.32.2.1) The reasoned load path also honours `SPARQ_STORE_PROFILE`, so
    // `reason` / `query --reason` inherit the store profile uniformly with the non-reasoned path.
    apply_store_profile(
        sparq_core::Graph::from_parts(dict, triples),
        store_profile_from_env(),
    )
}

/// Pull an optional `--reason <profile>` flag (rdfs | owl | n3) out of the argument list.
fn reason_flag(args: &[String]) -> Option<String> {
    let i = args.iter().position(|a| a == "--reason")?;
    Some(
        args.get(i + 1)
            .unwrap_or_else(|| {
                eprintln!("--reason needs a profile (rdfs | owl | n3)");
                std::process::exit(2);
            })
            .clone(),
    )
}

/// Load a graph applying the named reasoning profile. `rdfs`/`owl` materialize over the
/// parsed triples; `n3` parses the file as Notation3 (rules + facts) and forward-chains.
fn load_with_reasoning(path: &str, format: &str, profile: &str) -> sparq_core::Graph {
    if profile.eq_ignore_ascii_case("n3") {
        return load_n3(path);
    }
    let prof = sparq_reason::Profile::parse(profile).unwrap_or_else(|| {
        eprintln!("unknown reasoning profile '{profile}' (known: rdfs | owl | n3)");
        std::process::exit(2);
    });
    load_reasoned(path, format, prof)
}

/// Parse a Notation3 document (facts + `{…}=>{…}` rules), run the rule closure, build a graph.
fn load_n3(path: &str) -> sparq_core::Graph {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        std::process::exit(1);
    });
    let mut dict = sparq_core::dict::Dict::new();
    let t = Instant::now();
    let triples = sparq_reason::reason_n3(&mut dict, &text).unwrap_or_else(|e| {
        eprintln!("n3 reasoning error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "reasoned [N3]: {} ground triples in closure in {:.3}s",
        triples.len(),
        t.elapsed().as_secs_f64()
    );
    // [FABLE-5] (sq-7d3dj.32.2.1) Honour `SPARQ_STORE_PROFILE` on the N3 path too (see `load_reasoned`).
    apply_store_profile(
        sparq_core::Graph::from_parts(dict, triples),
        store_profile_from_env(),
    )
}

/// `reason <data-file> <format> <profile> [out.nt]` — materialize the entailed closure and
/// report the expansion; with `out.nt`, write the full closure as N-Triples.
fn cmd_reason(args: &[String]) {
    let (path, format, profile) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(pr)) => (p.as_str(), f.as_str(), pr.as_str()),
        _ => {
            eprintln!("usage: sparq-cli reason <data-file> <format> <rdfs|owl|n3> [out.nt]");
            std::process::exit(2);
        }
    };
    // N3 proof output (EYE --proof analogue): print each derivation step.
    if profile.eq_ignore_ascii_case("n3") && args.iter().any(|a| a == "--proof") {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error reading {path}: {e}");
            std::process::exit(1);
        });
        let mut dict = sparq_core::dict::Dict::new();
        let (closure, proof) =
            sparq_reason::reason_n3_proof(&mut dict, &text).unwrap_or_else(|e| {
                eprintln!("n3 reasoning error: {e}");
                std::process::exit(1);
            });
        let t = |id| dict.term(id).to_string();
        println!(
            "{} triples in closure; {} derivation step(s):",
            closure.len(),
            proof.len()
        );
        for (i, step) in proof.iter().enumerate() {
            let c = step.conclusion;
            println!("  [{}] {} {} {} .", i + 1, t(c[0]), t(c[1]), t(c[2]));
            println!("      ⊢ by rule #{} from:", step.rule);
            for p in &step.premises {
                println!("        {} {} {} .", t(p[0]), t(p[1]), t(p[2]));
            }
        }
        return;
    }
    let g = load_with_reasoning(path, format, profile);
    println!("{} triples after {profile} reasoning", g.len());
    if let Some(out) = args.get(5) {
        use std::io::Write;
        let mut w = std::io::BufWriter::new(std::fs::File::create(out).unwrap_or_else(|e| {
            eprintln!("create {out}: {e}");
            std::process::exit(1);
        }));
        let scan = g.store.scan(&[None, None, None]);
        for r in scan.rows.iter() {
            let spo = scan.to_spo(r);
            writeln!(
                w,
                "{} {} {} .",
                g.dict.term(spo[0]),
                g.dict.term(spo[1]),
                g.dict.term(spo[2])
            )
            .unwrap();
        }
        w.flush().unwrap();
        eprintln!("wrote closure to {out}");
    }
}

/// `query-mmap <dir> <sparql> [--format <out>] [--count]` — open a saved dataset with
/// memory-mapped indexes and run a query, printing its RESULTS. Reports load time + the
/// store self-estimate to stderr (≈0 GB heap for the mmap'd permutations — they live in the
/// OS page cache, not the process heap).
///
/// [OPUS-4.8] (sq-iwyy) Output is at PARITY with `query`: it shares the exact same
/// `emit_query_results` emission core, so SELECT prints bindings (default a readable table),
/// ASK prints a boolean, CONSTRUCT/DESCRIBE print N-Triples, `--format <table|tsv|csv|xml|
/// json|ntriples>` selects the SELECT/ASK serialisation, and `--count` restores the old
/// count-only line (`<n> solutions/triples in <ms>ms`). The only difference from `query` is
/// the data source: an mmap-backed `Graph::open` instead of an in-RAM `load`.
fn cmd_query_mmap(args: &[String]) {
    let (dir, sparql) = match (args.get(2), args.get(3)) {
        (Some(d), Some(q)) => (d.as_str(), q.as_str()),
        _ => {
            eprintln!("usage: sparq-cli query-mmap <dir> <sparql> [--format <table|tsv|csv|xml|json|ntriples>] [--count]");
            std::process::exit(2);
        }
    };
    // [OPUS-4.8] (sq-iwyy) Same flag surface as `query`: `--count` for the legacy count-only
    // line, `--format` for the SELECT/ASK serialisation (default table).
    let count_only = args.iter().any(|a| a == "--count");
    let out_fmt = out_format_flag(args);

    let t = Instant::now();
    let g = sparq_core::Graph::open(std::path::Path::new(dir)).unwrap_or_else(|e| {
        eprintln!("open error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "opened {} triples (indexes MEMORY-MAPPED) in {:.3}s | store-heap ~{:.2} GB (mmap'd perms not counted), dict ~{:.2} GB",
        g.len(),
        t.elapsed().as_secs_f64(),
        g.heap_bytes() as f64 / 1e9,
        g.dict.heap_bytes() as f64 / 1e9,
    );
    // The mmap-backed Graph borrows its on-disk permutations for the whole of `g`'s scope;
    // `emit_query_results` takes `&g` and finishes before `g` drops, so the borrow is sound.
    emit_query_results(&g, sparql, count_only, out_fmt);
}

/// [OPUS-4.8] Whether `format` is an RDF serialization this CLI accepts. Kept in lock-step
/// with the format arms in `load_quiet` / `cmd_build` and with `sparq_core::parse_to_triples`.
/// `parse_to_triples` falls back to Turtle for ANY unrecognised string, so the CLI must
/// gate on this set itself to honour the "unsupported format → non-zero exit" contract.
fn is_known_format(format: &str) -> bool {
    // [OPUS-4.8] (sq-oy1f.4) JSON-LD input is recognised in the DEFAULT build (the `jsonld`
    // feature is in the CLI default set — a maintainer-directed exception). Without the feature
    // (`--no-default-features`) the `oxjsonld` parser is not linked, so the JSON-LD tokens are
    // NOT "known" and a `jsonld` input format errors (exit 2) rather than mis-parsing as Turtle.
    #[cfg(feature = "jsonld")]
    if matches!(format, "jsonld" | "json-ld" | "application/ld+json") {
        return true;
    }
    matches!(
        format,
        "hdt"
            | "ntriples"
            | "n-triples"
            | "nt"
            | "application/n-triples"
            | "nquads"
            | "n-quads"
            | "nq"
            | "application/n-quads"
            | "trig"
            | "application/trig"
            | "turtle"
            | "ttl"
            | "text/turtle"
            | "application/turtle"
    )
}

/// [OPUS-4.8] Report an unknown `--format` value and exit 2 (usage error).
fn die_unknown_format(format: &str) -> ! {
    // [OPUS-4.8] (sq-oy1f.4) `jsonld` is named in the default build (the `jsonld` feature is in
    // the CLI default set); a `--no-default-features` build omits it from the list.
    eprintln!(
        "unknown format '{}' (known: turtle | ntriples | nquads | trig{}{})",
        format,
        if cfg!(feature = "jsonld") {
            " | jsonld"
        } else {
            ""
        },
        if cfg!(feature = "hdt") { " | hdt" } else { "" }
    );
    std::process::exit(2);
}

/// Opens a (possibly compressed) RDF document as a streaming reader. `.gz`/`.bz2`/`.zst[d]` are
/// decompressed transparently on the fly — the decompressed bytes are never all held at once.
fn open_reader(path: &str) -> std::io::Result<Box<dyn std::io::Read + Send>> {
    let file = std::fs::File::open(path)?;
    Ok(if path.ends_with(".gz") {
        Box::new(flate2::read::MultiGzDecoder::new(file))
    } else if path.ends_with(".bz2") {
        Box::new(bzip2::read::MultiBzDecoder::new(file))
    } else if path.ends_with(".zst") || path.ends_with(".zstd") {
        Box::new(zstd::stream::read::Decoder::new(file)?)
    } else {
        Box::new(file)
    })
}

/// [FABLE-5] (sq-7d3dj.32.2.1) The in-RAM store profile selected by `SPARQ_STORE_PROFILE`.
/// `Raw` is the default six-permutation layout; `Compressed` re-encodes into the
/// block-compressed in-RAM mode (`Graph::into_compressed`). Read once per load from the env.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StoreProfile {
    Raw,
    Compressed,
}

/// [FABLE-5] (sq-7d3dj.32.2.1) Resolve the `SPARQ_STORE_PROFILE` env var into a [`StoreProfile`].
/// Contract: unset or `raw` → [`StoreProfile::Raw`] (byte-identical current behaviour);
/// `compressed` → [`StoreProfile::Compressed`]; ANY other value → hard error, exit 2 —
/// fail-closed, no silent typo fall-through (a mistyped profile must never quietly select raw).
/// The value is matched case-insensitively and after trimming surrounding whitespace.
fn store_profile_from_env() -> StoreProfile {
    match std::env::var("SPARQ_STORE_PROFILE") {
        Err(_) => StoreProfile::Raw,
        Ok(v) => {
            let norm = v.trim().to_ascii_lowercase();
            match norm.as_str() {
                "" | "raw" => StoreProfile::Raw,
                "compressed" => StoreProfile::Compressed,
                _ => {
                    eprintln!(
                        "error: SPARQ_STORE_PROFILE={:?} is not a valid store profile (known: raw | compressed)",
                        v
                    );
                    std::process::exit(2);
                }
            }
        }
    }
}

/// [FABLE-5] (sq-7d3dj.32.2.1) Apply a resolved store [`StoreProfile`] to a freshly-loaded graph.
/// `Raw` returns `g` untouched (the default path is byte-identical to before this hook existed);
/// `Compressed` re-encodes it via `Graph::into_compressed()`. Honest scope: this reduces
/// steady-state resident footprint only — the load-time peak RSS still includes the raw perm
/// build `into_compressed` encodes from (see `research/compressed-memory-profile.md` §2).
fn apply_store_profile(g: sparq_core::Graph, profile: StoreProfile) -> sparq_core::Graph {
    match profile {
        StoreProfile::Raw => g,
        StoreProfile::Compressed => g.into_compressed(),
    }
}

/// Load without the timing/size summary line — used by the scaling harness, which loads many
/// times and prints its own table. Honours `SPARQ_STORE_PROFILE` (see [`store_profile_from_env`])
/// so `query` / `bench` / `scaling` inherit the profile uniformly. The raw-format routing lives
/// in `load_quiet_raw`; this wrapper adds only the post-load profile encoding.
fn load_quiet(path: &str, format: &str) -> sparq_core::Graph {
    apply_store_profile(load_quiet_raw(path, format), store_profile_from_env())
}

/// [FABLE-5] (sq-7d3dj.32.2.1) The unconditional raw load — format routing + parse + index build,
/// with NO store-profile applied. Byte-identical to the pre-profile `load_quiet` body, so callers
/// that manage their own profile decision (e.g. `memstat`, which has an explicit positional) use
/// this to avoid a double application.
fn load_quiet_raw(path: &str, format: &str) -> sparq_core::Graph {
    let die = |e: String| -> ! {
        eprintln!("error loading {path}: {e}");
        std::process::exit(1);
    };
    // [OPUS-4.8] Reject an unknown format argument up-front (exit 2 = usage error).
    // sparq-core's `parse_to_triples` treats ANY unrecognised format string as Turtle
    // (a catch-all `_ => Turtle` arm), so without this guard a typo'd / unsupported
    // format value would SILENTLY parse the input as Turtle and exit 0 — the opposite
    // of the "unsupported format → non-zero exit" CLI contract (bug sq-q50l).
    let ext_routed = path.ends_with(".hdt") || path.ends_with(".hdt.gz");
    if !is_known_format(format) && !ext_routed {
        die_unknown_format(format);
    }
    // HDT archives — `hdt` as the format argument, or a `.hdt`/`.hdt.gz` file
    // extension — route through sparq-hdt (gzip is sniffed by magic bytes
    // there, so a mislabelled extension still loads). Opt-in cargo feature:
    // the decode stack stays out of the default build (see Cargo.toml).
    if format == "hdt" || path.ends_with(".hdt") || path.ends_with(".hdt.gz") {
        #[cfg(feature = "hdt")]
        return sparq_hdt::load(path).unwrap_or_else(|e| die(e.to_string()));
        #[cfg(not(feature = "hdt"))]
        die("HDT support is not compiled into this binary: rebuild with `cargo build -p sparq-cli --features hdt`".to_string());
    }
    // N-Triples streams block-by-block (parallel parse, no full decompressed copy in RAM); other
    // formats need the whole document buffered for the parallel statement-splitter.
    if matches!(format, "ntriples" | "n-triples") {
        let reader = open_reader(path).unwrap_or_else(|e| die(e.to_string()));
        sparq_core::Graph::load_reader_parallel(reader, format).unwrap_or_else(|e| die(e))
    } else {
        use std::io::Read;
        let mut text = String::new();
        open_reader(path)
            .and_then(|mut r| r.read_to_string(&mut text))
            .unwrap_or_else(|e| die(e.to_string()));
        // N-Quads / TriG (and — [OPUS-4.8] sq-oy1f.4 — JSON-LD, whose `@graph` carries named
        // graphs) load as a DATASET so GRAPH queries and full-dataset re-serialisation (`dump …
        // jsonld`) see the named graphs instead of folding them into the default graph.
        #[cfg(feature = "jsonld")]
        let dataset = matches!(
            format,
            "nquads"
                | "n-quads"
                | "trig"
                | "application/trig"
                | "jsonld"
                | "json-ld"
                | "application/ld+json"
        );
        #[cfg(not(feature = "jsonld"))]
        let dataset = matches!(format, "nquads" | "n-quads" | "trig" | "application/trig");
        if dataset {
            sparq_core::Graph::load_dataset(&text, format).unwrap_or_else(|e| die(e))
        } else {
            sparq_core::Graph::load_str(&text, format).unwrap_or_else(|e| die(e))
        }
    }
}

fn load(path: &str, format: &str) -> sparq_core::Graph {
    let t = Instant::now();
    let g = load_quiet(path, format);
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

/// [FABLE-5] (sq-7d3dj.32) `memstat <data-file> <format> [compressed]` — load a document and print a
/// DETERMINISTIC memory-composition breakdown as `name<TAB>value` lines on stdout, plus the
/// process RSS counters. This is the at-scale extension of the CI `store_bytes_per_triple`
/// metric (scripts/ci-bench.sh greps the same `heap_bytes()` self-accounting off the load
/// summary line): here the total is DECOMPOSED into its components — dictionary vs the six
/// permutation indexes vs the numeric/temporal caches — so a bytes-per-triple figure comes
/// with its explanation, and the kernel's `VmRSS`/`VmHWM` (post-load resident + peak during
/// load, Linux `/proc/self/status`; 0 elsewhere) sit next to the self-accounted heap so
/// allocator/parse-transient overhead is visible rather than assumed. Consumed by
/// `scripts/bench/bytes-per-triple.sh` (bench id `bytes-per-triple`).
///
/// The trailing literal `compressed` (or `SPARQ_STORE_PROFILE=compressed`) re-encodes into the
/// memory-bound in-RAM mode (block-compressed permutations + blob dictionary,
/// `Graph::into_compressed`) before reporting — the same graph, the other end of the in-memory
/// footprint/latency trade, so the two framings come from one instrument. The reported `mode`
/// line says which. Because `memstat` reads the raw load (`load_quiet_raw`) and applies the
/// compression decision itself, the positional flag and the env are OR'd and applied exactly
/// once (no double `into_compressed`).
fn cmd_memstat(args: &[String]) {
    let (path, format) = match (args.get(2), args.get(3)) {
        (Some(p), Some(f)) => (p.as_str(), f.as_str()),
        _ => {
            eprintln!("usage: sparq-cli memstat <data-file> <format> [compressed]");
            std::process::exit(2);
        }
    };
    // The explicit positional `compressed` OR `SPARQ_STORE_PROFILE=compressed` (an unknown
    // profile value still fails closed via `store_profile_from_env`). Applied exactly once
    // over the raw load — `load_quiet_raw` never applies a profile itself.
    let compressed = args.get(4).map(String::as_str) == Some("compressed")
        || store_profile_from_env() == StoreProfile::Compressed;
    let t = Instant::now();
    let mut g = load_quiet_raw(path, format);
    if compressed {
        g = g.into_compressed();
    }
    let load_s = t.elapsed().as_secs_f64();

    let triples = g.len().max(1);
    let terms = g.dict.len().max(1);
    let heap_total = g.heap_bytes();
    let heap_dict = g.dict.heap_bytes();
    let heap_store = g.store.heap_bytes();
    // The remainder is the numerics + temporals literal-value caches (their accessors are
    // crate-private; the subtraction is exact because Graph::heap_bytes is the plain sum).
    let heap_caches = heap_total.saturating_sub(heap_dict + heap_store);
    let (vm_rss, vm_hwm) = proc_vm_bytes();

    println!("memstat_version\t1");
    println!("mode\t{}", if compressed { "compressed" } else { "raw" });
    println!("triples\t{}", g.len());
    println!("dict_terms\t{}", g.dict.len());
    println!("load_s\t{:.3}", load_s);
    println!("heap_total_bytes\t{}", heap_total);
    println!("heap_dict_bytes\t{}", heap_dict);
    println!("heap_store_bytes\t{}", heap_store);
    println!("heap_caches_bytes\t{}", heap_caches);
    println!(
        "heap_b_per_triple\t{:.2}",
        heap_total as f64 / triples as f64
    );
    println!(
        "store_b_per_triple\t{:.2}",
        heap_store as f64 / triples as f64
    );
    println!(
        "dict_b_per_triple\t{:.2}",
        heap_dict as f64 / triples as f64
    );
    println!(
        "caches_b_per_triple\t{:.2}",
        heap_caches as f64 / triples as f64
    );
    println!("dict_b_per_term\t{:.2}", heap_dict as f64 / terms as f64);
    println!("vm_rss_bytes\t{}", vm_rss);
    println!("vm_hwm_bytes\t{}", vm_hwm);
    println!("rss_b_per_triple\t{:.2}", vm_rss as f64 / triples as f64);
    println!("hwm_b_per_triple\t{:.2}", vm_hwm as f64 / triples as f64);
}

/// [FABLE-5] (sq-7d3dj.32) Read the process `VmRSS` / `VmHWM` (resident set + high-water mark) in
/// bytes from Linux `/proc/self/status`. Returns `(0, 0)` where the file is unavailable
/// (non-Linux), so `memstat`'s deterministic heap lines still work there.
fn proc_vm_bytes() -> (u64, u64) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let field = |key: &str| -> u64 {
        status
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
            .map_or(0, |kib| kib * 1024)
    };
    (field("VmRSS:"), field("VmHWM:"))
}

/// [OPUS-4.8] (sq-678h, sq-e3pj) `dump <file[.gz|.bz2|.zst]> <in-format> <out-format>` — load an
/// RDF document and re-serialize the whole graph (default + named graphs) into one of the writer
/// matrix formats and print it to stdout. `out-format` ∈ {turtle, trig, nquads, ntriples,
/// jsonld[-expanded|-flattened|-compacted]}. Turtle emits only the default graph (named graphs
/// need a dataset format); trig/nquads/jsonld emit the full dataset. JSON-LD defaults to the
/// expanded form; `jsonld-flattened` / `jsonld-compacted` select the other 1.1 document forms.
/// [OPUS-4.8] (sq-ixc3.3) `jsonld-pretty[-expanded|-flattened|-compacted]` emit the same
/// JSON-LD documents in an indented, multi-line shape (whitespace-only over the minified writer).
/// [OPUS-4.8] (sq-oy1f.5) `jsonld-compact` (+ `jsonld-compact-pretty`) emit the FULL W3C
/// JSON-LD 1.1 Compaction Algorithm against a caller `@context` supplied via `--context <file>`
/// (term definitions / `@vocab` / type-language-`@container` coercion / `@reverse` / aliasing),
/// not the prefix-only `jsonld-compacted` form. Behind the opt-in `serialize-rdf` cargo feature.
#[cfg(feature = "serialize-rdf")]
fn cmd_dump(args: &[String]) {
    let (path, in_fmt, out_fmt) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(i), Some(o)) => (p.as_str(), i.as_str(), o.as_str()),
        _ => {
            eprintln!("usage: sparq-cli dump <file[.gz|.bz2|.zst]> <in-format> <out-format> [--context <ctx.jsonld>]\n  out-format: turtle | turtle-pretty | trig | trig-pretty | nquads | ntriples | jsonld[-expanded|-flattened|-compacted] | jsonld-pretty[-expanded|-flattened|-compacted] | jsonld-compact[-pretty] (needs --context)");
            std::process::exit(2);
        }
    };
    // [OPUS-4.8] (sq-oy1f.5) `--context <file>`: the JSON-LD `@context` for full 1.1 Compaction.
    // Scanned from the args (the CLI is positional, not clap); only the `jsonld-compact` out-
    // formats consult it. A present `--context` with no value is a usage error.
    let context_path: Option<&str> = match args.iter().position(|a| a == "--context") {
        None => None,
        Some(i) => Some(args.get(i + 1).map(String::as_str).unwrap_or_else(|| {
            eprintln!("--context needs a value (a JSON-LD @context file)");
            std::process::exit(2);
        })),
    };
    let g = load_quiet(path, in_fmt);
    use sparq_engine::serialize::JsonLdForm;
    let serialized = match out_fmt {
        // [OPUS-4.8] (sq-oy1f.5) FULL W3C JSON-LD 1.1 Compaction against the `--context` file.
        "jsonld-compact"
        | "json-ld-compact"
        | "jsonld-compact-pretty"
        | "json-ld-compact-pretty" => {
            let ctx_path = context_path.unwrap_or_else(|| {
                eprintln!(
                    "out-format '{out_fmt}' needs a `@context`: pass --context <file.jsonld>"
                );
                std::process::exit(2);
            });
            let ctx_text = std::fs::read_to_string(ctx_path).unwrap_or_else(|e| {
                eprintln!("error reading context file {ctx_path}: {e}");
                std::process::exit(1);
            });
            let ctx = sparq_engine::serialize::parse_context_json(&ctx_text).unwrap_or_else(|| {
                eprintln!("context file {ctx_path} is not a JSON object (a JSON-LD @context)");
                std::process::exit(1);
            });
            if out_fmt.ends_with("-pretty") {
                let opts = sparq_engine::serialize::JsonLdPrettyOptions::default();
                sparq_engine::serialize::graph_to_jsonld_compact_pretty(&g, &ctx, &opts)
            } else {
                sparq_engine::serialize::graph_to_jsonld_compact(&g, &ctx)
            }
        }
        "turtle" | "ttl" => sparq_engine::serialize::graph_to_turtle(&g),
        // [OPUS-4.8] (sq-ixc3.2) idiomatic, deterministic pretty Turtle / TriG.
        "turtle-pretty" | "ttl-pretty" => sparq_engine::serialize::graph_to_turtle_pretty(&g),
        "trig-pretty" => sparq_engine::serialize::graph_to_trig_pretty(&g),
        "trig" => sparq_engine::serialize::graph_to_trig(&g),
        "nquads" | "n-quads" => sparq_engine::serialize::graph_to_nquads(&g),
        "jsonld" | "json-ld" | "jsonld-expanded" => {
            sparq_engine::serialize::graph_to_jsonld(&g, JsonLdForm::Expanded)
        }
        "jsonld-flattened" => sparq_engine::serialize::graph_to_jsonld(&g, JsonLdForm::Flattened),
        "jsonld-compacted" => sparq_engine::serialize::graph_to_jsonld(&g, JsonLdForm::Compacted),
        // [OPUS-4.8] (sq-ixc3.3) pretty (indented) JSON-LD — whitespace-only over the minified
        // forms above. `jsonld-pretty` defaults to expanded, matching `jsonld`.
        "jsonld-pretty" | "json-ld-pretty" | "jsonld-pretty-expanded" => {
            sparq_engine::serialize::graph_to_jsonld_pretty(&g, JsonLdForm::Expanded)
        }
        "jsonld-pretty-flattened" => {
            sparq_engine::serialize::graph_to_jsonld_pretty(&g, JsonLdForm::Flattened)
        }
        "jsonld-pretty-compacted" => {
            sparq_engine::serialize::graph_to_jsonld_pretty(&g, JsonLdForm::Compacted)
        }
        // N-Triples stays on the always-on writer (the default graph as `s p o .` lines).
        "ntriples" | "n-triples" => {
            let triples: Vec<oxrdf::Triple> = g
                .iter_ids()
                .map(|[s, p, o]| {
                    let subject = match g.dict.term(s) {
                        oxrdf::Term::NamedNode(n) => oxrdf::NamedOrBlankNode::NamedNode(n),
                        oxrdf::Term::BlankNode(b) => oxrdf::NamedOrBlankNode::BlankNode(b),
                        other => {
                            eprintln!("corrupt store: non-IRI/blank subject {other}");
                            std::process::exit(1);
                        }
                    };
                    let predicate = match g.dict.term(p) {
                        oxrdf::Term::NamedNode(n) => n,
                        other => {
                            eprintln!("corrupt store: non-IRI predicate {other}");
                            std::process::exit(1);
                        }
                    };
                    oxrdf::Triple {
                        subject,
                        predicate,
                        object: g.dict.term(o),
                    }
                })
                .collect();
            sparq_engine::triples_to_ntriples(&triples)
        }
        other => {
            eprintln!("unknown out-format '{other}' (known: turtle | turtle-pretty | trig | trig-pretty | nquads | ntriples | jsonld[-expanded|-flattened|-compacted] | jsonld-pretty[-expanded|-flattened|-compacted] | jsonld-compact[-pretty] (needs --context))");
            std::process::exit(2);
        }
    };
    print!("{serialized}");
}

/// [FABLE-5] (sq-8ju74) `to-hdt <data-file[.gz|.bz2|.zst]> <format> <out.hdt[.gz|.zst|.bz2]>` —
/// load an RDF document through the shared load path (any ingestible format, `.hdt` itself
/// included) and EXPORT it as a standard-layout HDT v1.0 archive (FourSectionDictionary +
/// BitmapTriples, SPO) via `sparq_hdt::save` — the direct in-memory encoder, no temporary
/// N-Triples round-trip. The output container (`.hdt.gz` / `.hdt.zst` / `.hdt.bz2`, or a bare
/// `.hdt`) is chosen by the OUTPUT path's extension (the write side cannot sniff content that
/// does not exist yet). HDT carries a SINGLE default graph: when the loaded input has named
/// graphs (TriG / N-Quads / JSON-LD `@graph`) they are DROPPED from the archive, and a loud
/// warning with the dropped graph/triple counts goes to stderr — never silently. An RDF 1.2
/// quoted-triple term cannot be represented in standard HDT; `save` fails with a term error
/// (exit 1) rather than emitting a lossy archive. Behind the opt-in `hdt-write` cargo feature.
#[cfg(feature = "hdt-write")]
fn cmd_to_hdt(args: &[String]) {
    let (path, format, out) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(o)) => (p.as_str(), f.as_str(), o.as_str()),
        _ => {
            eprintln!(
                "usage: sparq-cli to-hdt <data-file[.gz|.bz2|.zst]> <format> <out.hdt[.gz|.zst|.bz2]>\n  \
                 exports the loaded document as a standard HDT v1.0 archive (default graph only; \
                 output compression chosen by the output extension)"
            );
            std::process::exit(2);
        }
    };
    let g = load_quiet(path, format);
    // Honesty over silence: HDT has no place for named graphs, so a dataset input (TriG /
    // N-Quads / JSON-LD `@graph`) loses them in the archive. Count what is dropped and say so.
    let (mut named_graphs, mut named_triples) = (0usize, 0usize);
    g.for_named_graphs_with_prefix("", |_, sub| {
        named_graphs += 1;
        named_triples += sub.len();
    });
    if named_graphs > 0 {
        eprintln!(
            "warning: HDT carries a single default graph — dropping {named_graphs} named graph(s) \
             ({named_triples} triple(s)); only the {} default-graph triple(s) are written",
            g.len()
        );
    }
    let t = Instant::now();
    sparq_hdt::save(&g, out).unwrap_or_else(|e| {
        eprintln!("error writing {out}: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "wrote {} triples to {out} in {:.3}s",
        g.len(),
        t.elapsed().as_secs_f64()
    );
}

/// [OPUS-4.8] (sq-vczh2, epic sq-2m6zm) `terse <terse-query>` — transpile a *terse* query into
/// the canonical, conformant SPARQL it expands to, printing the verifiable JSON contract
/// (`{ canonical_sparql, keywords, resolutions, warnings, legendVersion }`) — the SAME shape the
/// server's `POST /terse/transpile` endpoint returns.
///
/// The terse query is read from the positional argument, or — when that argument is `-` — from
/// stdin (so a query with shell-hostile characters can be piped). It is the LEAN `sparq-terse`
/// build: the `K:<name>` keyword layer is fully on; a `V("phrase")` construct loud-FAILS (exit 2)
/// rather than being guessed (concept resolution is a future `vectors`-gated extension; the V()
/// ambiguity caveat is tracked by sq-26fdp). It NEVER executes the query — pipe the printed
/// `canonical_sparql` into `query`/`query-mmap` to run it. Behind the opt-in `terse` cargo feature.
#[cfg(feature = "terse")]
fn cmd_terse(args: &[String]) {
    let src: String = match args.get(2).map(String::as_str) {
        Some("-") => {
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("error reading terse query from stdin: {e}");
                std::process::exit(1);
            }
            buf
        }
        Some(q) => q.to_string(),
        None => {
            eprintln!(
                "usage: sparq-cli terse <terse-query | ->\n  \
                 Transpiles a terse query (the `K:<name>` keyword layer over canonical SPARQL) and\n  \
                 prints the canonical SPARQL it expands to, as JSON. Pass `-` to read the query from\n  \
                 stdin. It does NOT execute the query — pipe the canonical_sparql into `query`."
            );
            std::process::exit(2);
        }
    };
    match sparq_terse::terse_to_sparql(&src) {
        Ok(expansion) => println!("{}", terse_expansion_to_json(&expansion)),
        // Every terse failure is a loud, deterministic input error (unknown keyword, `PREFIX K:`
        // collision, an un-resolvable `V(...)` in this lean build, or the conformance canary): it
        // goes to stderr with the transpiler's own message and a non-zero exit, never a guess.
        Err(e) => {
            eprintln!("terse: {e}");
            std::process::exit(2);
        }
    }
}

/// [OPUS-4.8] (sq-vczh2) Hand-build the verifiable terse-expansion JSON (the CLI runtime binary
/// carries no `serde_json` — it hand-builds JSON like `bench`'s `--json`). The shape mirrors the
/// server's `POST /terse/transpile` body so a caller gets the SAME contract from either surface.
#[cfg(feature = "terse")]
fn terse_expansion_to_json(expansion: &sparq_terse::Expansion) -> String {
    // Minimal JSON-string escaper (the same control-char + quote/backslash discipline the server's
    // hand-rolled JSON uses); positional `format!` args avoid the CodeQL rust/unused-variable
    // false positive on inline-captured identifiers.
    fn esc(out: &mut String, s: &str) {
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
    }

    let mut out = String::from("{\"canonical_sparql\":");
    esc(&mut out, &expansion.canonical_sparql);
    out.push_str(",\"keywords\":[");
    for (i, k) in expansion.keywords.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"keyword\":");
        esc(&mut out, &k.keyword);
        out.push_str(",\"iri\":");
        esc(&mut out, &k.iri);
        out.push_str(",\"legendVersion\":");
        esc(&mut out, &k.legend_version);
        out.push('}');
    }
    // `resolutions` is always empty in this lean (no-`vectors`) CLI build, but the field is in the
    // contract so a future `vectors`-enabled build can populate it without a shape change.
    out.push_str("],\"resolutions\":[");
    for (i, r) in expansion.resolutions.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"phrase\":");
        esc(&mut out, &r.phrase);
        out.push_str(",\"iri\":");
        esc(&mut out, &r.iri);
        out.push_str(&format!(",\"score\":{}", r.score));
        match &r.runner_up {
            Some(ru) => {
                out.push_str(",\"runnerUp\":");
                esc(&mut out, ru);
            }
            None => out.push_str(",\"runnerUp\":null"),
        }
        match r.runner_up_score {
            Some(s) => out.push_str(&format!(",\"runnerUpScore\":{}", s)),
            None => out.push_str(",\"runnerUpScore\":null"),
        }
        out.push_str(&format!(",\"confidence\":{}", r.confidence));
        out.push_str(",\"method\":");
        esc(&mut out, r.method.as_str());
        out.push('}');
    }
    out.push_str("],\"warnings\":[");
    for (i, w) in expansion.warnings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        esc(&mut out, w);
    }
    out.push_str("],\"legendVersion\":");
    esc(&mut out, sparq_terse::LEGEND_VERSION);
    out.push('}');
    out
}

/// [OPUS-4.8] (sq-l4ki) Output serialisation chosen by `query --format`. `Table` (the
/// human-readable default) and the four W3C SELECT result formats apply to SELECT (and,
/// where meaningful, ASK); CONSTRUCT/DESCRIBE always emit N-Triples regardless of this.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OutFormat {
    Table,
    Tsv,
    Csv,
    Xml,
    Json,
    NTriples,
}

impl OutFormat {
    /// Parses the `--format` value; `None` for an unknown name (caller reports + exits 2).
    fn parse(s: &str) -> Option<OutFormat> {
        Some(match s {
            "table" => OutFormat::Table,
            "tsv" => OutFormat::Tsv,
            "csv" => OutFormat::Csv,
            "xml" => OutFormat::Xml,
            "json" => OutFormat::Json,
            "ntriples" | "n-triples" | "nt" => OutFormat::NTriples,
            _ => return None,
        })
    }
}

/// [OPUS-4.8] (sq-l4ki) Pull an optional `--format <name>` flag out of the argument list,
/// defaulting to a readable `Table`. Exits 2 (usage error) on a missing/unknown value —
/// matching the rest of the CLI's flag-validation contract.
fn out_format_flag(args: &[String]) -> OutFormat {
    match args.iter().position(|a| a == "--format") {
        None => OutFormat::Table,
        Some(i) => {
            let val = args.get(i + 1).unwrap_or_else(|| {
                eprintln!("--format needs a value (table | tsv | csv | xml | json | ntriples)");
                std::process::exit(2);
            });
            OutFormat::parse(val).unwrap_or_else(|| {
                eprintln!(
                    "unknown --format '{val}' (known: table | tsv | csv | xml | json | ntriples)"
                );
                std::process::exit(2);
            })
        }
    }
}

/// [OPUS-4.8] (sq-l4ki) Renders a SELECT `QueryResult` as a fixed-width ASCII table —
/// the default human-readable `query` output. Unbound cells render empty; each term uses
/// its SPARQL/Turtle term syntax (oxrdf's `Display`). Column widths are sized to the
/// widest cell so columns line up; for a zero-variable result (which only ASK produces,
/// not SELECT) it prints the row count.
fn select_to_table(r: &sparq_engine::QueryResult) -> String {
    use std::fmt::Write;
    if r.vars.is_empty() {
        return format!("({} row(s))\n", r.rows.len());
    }
    let headers: Vec<String> = r.vars.iter().map(|v| format!("?{}", v.as_str())).collect();
    let cells: Vec<Vec<String>> = r
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| c.as_ref().map(|t| t.to_string()).unwrap_or_default())
                .collect()
        })
        .collect();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in &cells {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    let sep = |out: &mut String| {
        out.push('+');
        for w in &widths {
            for _ in 0..w + 2 {
                out.push('-');
            }
            out.push('+');
        }
        out.push('\n');
    };
    let row_line = |out: &mut String, fields: &[String]| {
        out.push('|');
        for (i, f) in fields.iter().enumerate() {
            let pad = widths[i] - f.chars().count();
            let _ = write!(out, " {f}{} |", " ".repeat(pad));
        }
        out.push('\n');
    };
    let mut out = String::new();
    sep(&mut out);
    row_line(&mut out, &headers);
    sep(&mut out);
    for row in &cells {
        row_line(&mut out, row);
    }
    sep(&mut out);
    let _ = writeln!(out, "({} row(s))", r.rows.len());
    out
}

fn cmd_query(args: &[String]) {
    let (path, format, sparql) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(q)) => (p, f, q),
        _ => {
            eprintln!(
                "usage: sparq-cli query <data-file> <format> <sparql> [--format <table|tsv|csv|xml|json|ntriples>] [--count]"
            );
            std::process::exit(2);
        }
    };
    // [OPUS-4.8] (sq-l4ki) `--count` preserves the historical count-only output; otherwise
    // we emit real results. `--format` selects the SELECT/ASK serialisation (default table).
    let count_only = args.iter().any(|a| a == "--count");
    let out_fmt = out_format_flag(args);

    let g = match reason_flag(args) {
        Some(profile) => load_with_reasoning(path, format, &profile),
        None => load(path, format),
    };

    emit_query_results(&g, sparql, count_only, out_fmt);
}

/// [OPUS-4.8] (sq-iwyy) The shared result-emission core used by BOTH `query` and
/// `query-mmap`, so the two stay at output PARITY by construction. Classifies the parsed
/// query FORM (SELECT / ASK / CONSTRUCT / DESCRIBE) once and routes each to its executor —
/// the engine's `query`/`query_json` run SELECT/ASK; the graph-valued forms go through
/// `construct_or_describe`/`construct_ntriples` (the same path the `bench` runner uses).
///
/// `count_only` restores the historical count-only line; `out_fmt` selects the SELECT/ASK
/// serialisation (CONSTRUCT/DESCRIBE always emit N-Triples). Takes `&Graph` by reference, so
/// it is agnostic to whether the graph is in-RAM (`query`) or mmap-backed (`query-mmap`).
fn emit_query_results(g: &sparq_core::Graph, sparql: &str, count_only: bool, out_fmt: OutFormat) {
    // Parse once so we can classify the query FORM (SELECT / ASK / CONSTRUCT / DESCRIBE)
    // and route it to the matching executor — the engine's `query`/`query_json` only run
    // SELECT/ASK; the graph-valued forms go through `construct_or_describe` (the same path
    // the `bench` runner already uses). [OPUS-4.8]
    let prepared = sparq_engine::PreparedQuery::parse(sparql).unwrap_or_else(|e| {
        eprintln!("query error: {e}");
        std::process::exit(1);
    });

    let t = Instant::now();

    // Backward-friendly count-only path (the pre-sq-l4ki behaviour).
    if count_only {
        if prepared.is_graph_form() {
            match sparq_engine::construct_or_describe(g, sparql) {
                Ok(ts) => println!(
                    "{} triples in {:.3}ms",
                    ts.len(),
                    t.elapsed().as_secs_f64() * 1e3
                ),
                Err(e) => die_query(e),
            }
        } else {
            match sparq_engine::query(g, sparql) {
                Ok(r) => println!(
                    "{} solutions in {:.3}ms",
                    r.len(),
                    t.elapsed().as_secs_f64() * 1e3
                ),
                Err(e) => die_query(e),
            }
        }
        return;
    }

    // CONSTRUCT / DESCRIBE -> the resulting triples as N-Triples (always; `--format` is a
    // SELECT/ASK results-format selector and does not apply to the graph forms).
    if prepared.is_graph_form() {
        match sparq_engine::construct_ntriples(g, sparql) {
            Ok(nt) => print!("{nt}"),
            Err(e) => die_query(e),
        }
        return;
    }

    // ASK -> a boolean. (`--format json`/`xml` emit the W3C boolean documents; the other
    // formats fall back to the bare `true`/`false` token.)
    if matches!(prepared.query(), spargebra::Query::Ask { .. }) {
        let value = match sparq_engine::ask(g, sparql) {
            Ok(b) => b,
            Err(e) => die_query(e),
        };
        match out_fmt {
            OutFormat::Json => println!("{}", sparq_server::results::ask_to_json(value)),
            OutFormat::Xml => print!("{}", sparq_server::results::ask_to_xml(value)),
            _ => println!("{value}"),
        }
        return;
    }

    // SELECT -> the solution bindings. JSON reuses the engine's fast direct serialiser; the
    // other formats reuse sparq-server's W3C SELECT serialisers over the QueryResult.
    if out_fmt == OutFormat::Json {
        match sparq_engine::query_json(g, sparql) {
            Ok(s) => println!("{s}"),
            Err(e) => die_query(e),
        }
        return;
    }
    let r = match sparq_engine::query(g, sparql) {
        Ok(r) => r,
        Err(e) => die_query(e),
    };
    match out_fmt {
        OutFormat::Table => print!("{}", select_to_table(&r)),
        OutFormat::Tsv => print!("{}", sparq_server::results::select_to_tsv(&r)),
        OutFormat::Csv => print!("{}", sparq_server::results::select_to_csv(&r)),
        OutFormat::Xml => print!("{}", sparq_server::results::select_to_xml(&r)),
        // A SELECT has no graph form, so N-Triples is meaningless for bindings — fall back
        // to TSV (the closest line-oriented bindings format) rather than error.
        OutFormat::NTriples => print!("{}", sparq_server::results::select_to_tsv(&r)),
        OutFormat::Json => unreachable!("json handled above"),
    }
}

/// [OPUS-4.8] (sq-l4ki) Reports a query execution error to stderr and exits 1 (runtime
/// error) — the shared failure tail of every `query` dispatch arm.
fn die_query(e: String) -> ! {
    eprintln!("query error: {e}");
    std::process::exit(1);
}

/// [OPUS-4.8] (sq-d7d) Extract a `--json <path>` results-emit flag (and its value) from the
/// positional argument vector, returning `(positional_args_without_the_flag, Option<path>)`.
/// `bench` / `bench-mmap` index their other arguments positionally (`args.get(N)`), so the
/// flag+value pair is removed BEFORE positional parsing rather than handled inline — this keeps
/// the historical positional contract intact whether the flag is present or absent. A bare
/// `--json` with no following value is a usage error (exit 2), mirroring the rest of the CLI.
fn take_json_flag(args: &[String]) -> (Vec<String>, Option<String>) {
    let mut out = Vec::with_capacity(args.len());
    let mut json_path = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--json" {
            match args.get(i + 1) {
                Some(p) => {
                    json_path = Some(p.clone());
                    i += 2;
                    continue;
                }
                None => {
                    eprintln!("`--json` requires a path argument: --json <path>");
                    std::process::exit(2);
                }
            }
        }
        out.push(args[i].clone());
        i += 1;
    }
    (out, json_path)
}

fn cmd_bench(args: &[String]) {
    let (args, json_path) = take_json_flag(args);
    let (path, format, dir) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(d)) => (p, f, d),
        _ => {
            eprintln!("usage: sparq-cli bench <data-file> <format> <queries-dir> [iters] [count|materialize|json] [--json <results.json>]");
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
    run_query_suite(&g, dir, iters, mode, json_path.as_deref());
}

/// `bench-mmap <dir> <queries-dir> [iters] [count|materialize|json]` — same as `bench`
/// but OPENS the dataset out-of-core (memory-mapped), so the compute of a >RAM index can
/// be measured without loading it into RAM. Used to compare sparq's compute against the
/// stored QLever baselines at 100M+ on a 16 GB machine.
fn cmd_bench_mmap(args: &[String]) {
    let (args, json_path) = take_json_flag(args);
    let (dir, qdir) = match (args.get(2), args.get(3)) {
        (Some(d), Some(q)) => (d.as_str(), q.as_str()),
        _ => {
            eprintln!("usage: sparq-cli bench-mmap <index-dir> <queries-dir> [iters] [count|materialize|json] [decompress] [--json <results.json>]");
            std::process::exit(2);
        }
    };
    let iters: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(5);
    let mode = args.get(5).map(String::as_str).unwrap_or("count");
    let t = Instant::now();
    let mut g = sparq_core::Graph::open(std::path::Path::new(dir)).unwrap_or_else(|e| {
        eprintln!("open error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "opened {} triples (mmap) in {:.3}s | committed heap ~{:.3} GB",
        g.len(),
        t.elapsed().as_secs_f64(),
        g.heap_bytes() as f64 / 1e9
    );
    // Load-time decompression mode: decode compressed perms to raw RAM before querying.
    if args.get(6).map(String::as_str) == Some("decompress") {
        let t = Instant::now();
        g.decompress_indexes();
        eprintln!(
            "decompressed indexes to RAM in {:.3}s | committed heap ~{:.3} GB",
            t.elapsed().as_secs_f64(),
            g.heap_bytes() as f64 / 1e9
        );
    }
    run_query_suite(&g, qdir, iters, mode, json_path.as_deref());
}

/// One measured row of the query suite — the same fields the TSV reports
/// (`name`, `rows`, `min_micros`), captured so they can be serialised to JSON.
/// [OPUS-4.8] (sq-d7d)
struct SuiteRow {
    name: String,
    /// `Ok(min_micros)` for a successful query (with `rows`), or `Err(message)` if it failed.
    outcome: Result<(usize, f64), String>,
}

/// Runs every `*.rq` in `dir` (sorted) `iters` times in `mode`, printing one TSV line
/// per query: `<name>\t<rows>\t<min_micros>`.
///
/// [OPUS-4.8] (sq-d7d) When `json_path` is `Some`, the SAME measured fields are ALSO written
/// to that path as a machine-readable JSON document (the structured-benchmark-catalog pattern,
/// mirroring `bench/memtier` / `mpc_net_bench`'s dependency-free emit). STDOUT is byte-for-byte
/// unchanged whether or not the flag is present — JSON is strictly additive.
fn run_query_suite(
    g: &sparq_core::Graph,
    dir: &str,
    iters: usize,
    mode: &str,
    json_path: Option<&str>,
) {
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

    let mut results: Vec<SuiteRow> = Vec::with_capacity(entries.len());
    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let sparql = std::fs::read_to_string(&path).unwrap();
        // [OPUS-4.8] Route the graph-valued forms (CONSTRUCT / DESCRIBE) through the
        // construct/describe executor — count()/query()/query_json() only handle SELECT/ASK.
        // This lets the operator-coverage suite (bench/operators/queries) exercise every
        // SPARQL query form through the same `bench` runner; "rows" = produced triples.
        let graph_form = sparq_engine::PreparedQuery::parse(&sparql)
            .map(|p| p.is_graph_form())
            .unwrap_or(false);
        let mut best = f64::INFINITY;
        let mut rows = 0;
        let mut err = None;
        for _ in 0..iters {
            let t = Instant::now();
            let r: Result<usize, String> = if graph_form {
                sparq_engine::construct_or_describe(g, &sparql).map(|ts| ts.len())
            } else {
                match mode {
                    "count" => sparq_engine::count(g, &sparql),
                    "json" => sparq_engine::query_json(g, &sparql).map(|s| {
                        let n = s.len();
                        std::hint::black_box(s);
                        n
                    }),
                    _ => sparq_engine::query(g, &sparql).map(|r| r.len()),
                }
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
        match &err {
            None => println!("{name}\t{rows}\t{best:.1}"),
            Some(e) => println!("{name}\tERROR\t{e}"),
        }
        let outcome = match err {
            None => Ok((rows, best)),
            Some(e) => Err(e),
        };
        results.push(SuiteRow { name, outcome });
    }

    if let Some(p) = json_path {
        let doc = suite_results_json(mode, iters, &results);
        if let Err(e) = std::fs::write(p, doc) {
            eprintln!("error writing --json results to {p}: {e}");
            std::process::exit(1);
        }
        eprintln!("wrote {} query results to {p}", results.len());
    }
}

/// [OPUS-4.8] (sq-d7d) Serialise a query-suite run to stable, dependency-free JSON — the same
/// hand-built `format!` convention as `mpc_net_bench::cell_json` (no serde_json dep added to the
/// CLI). The shape mirrors the catalog pattern: a top-level object carrying the run parameters +
/// an honest `note` (these are the numbers THIS machine measured — non-canonical), and a
/// `queries` array of one object per `*.rq`, each with the SAME fields the TSV prints
/// (`name`, `rows`, `min_micros`) or an `error` string when the query failed.
fn suite_results_json(mode: &str, iters: usize, rows: &[SuiteRow]) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"harness\": \"sparq-cli bench\",\n");
    s.push_str(&format!("  \"mode\": {},\n", json_str(mode)));
    s.push_str(&format!("  \"iters\": {iters},\n"));
    s.push_str(
        "  \"note\": \"min-of-iters wall-clock MEASURED on the running host; \
         NON-CANONICAL (whatever this machine measured) — do not bake into committed files\",\n",
    );
    s.push_str("  \"queries\": [\n");
    for (i, r) in rows.iter().enumerate() {
        s.push_str("    {\n");
        s.push_str(&format!("      \"name\": {}", json_str(&r.name)));
        match &r.outcome {
            Ok((rows, micros)) => {
                s.push_str(&format!(",\n      \"rows\": {rows}"));
                s.push_str(&format!(",\n      \"min_micros\": {micros:.1}\n"));
            }
            Err(e) => {
                s.push_str(&format!(",\n      \"error\": {}\n", json_str(e)));
            }
        }
        s.push_str("    }");
        if i + 1 < rows.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}

/// [OPUS-4.8] (sq-d7d) Minimal JSON string escaper for the dependency-free emit — escapes the
/// characters JSON requires (`"`, `\`, and the C0 control set incl. the named whitespace
/// escapes). Query names are file stems and error strings are engine messages, so this covers
/// the realistic input; anything else still produces valid `\uXXXX` escapes.
fn json_str(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for c in raw.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
