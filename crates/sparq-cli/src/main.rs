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
//! `format`: turtle | ntriples | nquads | trig — plus hdt when built with the
//! opt-in `hdt` cargo feature (`--features hdt`), which also auto-detects the
//! `.hdt` / `.hdt.gz` file extensions regardless of the format argument.

use std::io::Read;
use std::time::Instant;

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
        Some("bench-mmap") => cmd_bench_mmap(&args),
        Some("ingest") => cmd_ingest(&args),
        Some("save") => cmd_save(&args),
        Some("recompress") => cmd_recompress(&args),
        Some("build") => cmd_build(&args),
        Some("query-mmap") => cmd_query_mmap(&args),
        Some("probe-compress") => cmd_probe_compress(&args),
        Some("compare-compress") => cmd_compare_compress(&args),
        Some("bench-remap") => cmd_bench_remap(&args),
        Some("scaling") => cmd_scaling(&args),
        _ => {
            eprintln!("usage:\n  sparq-cli query <data-file> <format> <sparql>\n  sparq-cli bench <data-file> <format> <queries-dir> [iters]\n  sparq-cli ingest <file[.gz|.bz2]> [parse|intern|full] [max_millions]\n  sparq-cli save <data-file> <format> <dir> [compressed]  # build + persist indexes to disk\n  sparq-cli recompress <src-dir> <dst-dir>          # re-persist with block-compressed indexes\n  sparq-cli query-mmap <dir> <sparql>               # query with indexes MEMORY-MAPPED (out-of-core)");
            std::process::exit(2);
        }
    }
}

/// Isolated micro-benchmark of the latency-bound dict-remap gather, to measure the per-ISA
/// software prefetch in isolation (undiluted by parsing) on each hardware target.
///   sparq-cli bench-remap [n_triples] [dict_size] [iters]
/// Run twice — once normally, once with SPARQ_NO_PREFETCH=1 — to get the prefetch delta.
fn cmd_bench_remap(args: &[String]) {
    let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20_000_000);
    let dict: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(50_000_000);
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
///   sparq-cli scaling <data-file> <format> <queries-dir> [threads=auto] [iters=3]
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
            let mut v: Vec<usize> = s.split(',').filter_map(|t| t.trim().parse().ok()).filter(|&n| n >= 1).collect();
            v.dedup();
            v
        }
        None => {
            let max = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
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
    let pool = |n: usize| rayon::ThreadPoolBuilder::new().num_threads(n).build().expect("rayon pool");

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
    let res = if compressed { g.save_compressed(std::path::Path::new(dir)) } else { g.save(std::path::Path::new(dir)) };
    res.unwrap_or_else(|e| {
        eprintln!("save error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "saved {} triples to {dir}{} in {:.3}s",
        g.len(),
        if compressed { " (compressed perms)" } else { "" },
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
    g.save_compressed(std::path::Path::new(dst)).unwrap_or_else(|e| {
        eprintln!("save error: {e}");
        std::process::exit(1);
    });
    eprintln!("recompressed {} triples {src} -> {dst} in {:.3}s", g.len(), t.elapsed().as_secs_f64());
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
            eprintln!("usage: sparq-cli build <file[.gz|.bz2]> <format> <dir> [chunk_millions]");
            std::process::exit(2);
        }
    };
    let chunk = args.get(5).and_then(|s| s.parse::<usize>().ok()).unwrap_or(16) * 1_000_000;

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
    sparq_core::Graph::build_external(reader, format, std::path::Path::new(dir), chunk)
        .unwrap_or_else(|e| {
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
    let rows: &[[u32; 3]] = unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[u32; 3]>(), n) };
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
    println!("  raw               : {:>6.2} B/triple ({:.2} GB)", 12.0, map.len() as f64 / 1e9);
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
    println!("  raw   perms        : {:>7.2} MB ({:.1} B/triple)", raw_store as f64 / 1e6, raw_store as f64 / cmp.len().max(1) as f64);
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
        println!("  compressed : {cn} bytes of JSON in {ct:.3} ms  ({:.2}x raw)", ct / rt);
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
    let (mut dict, mut triples) = sparq_core::Graph::parse_to_triples(&text, format).unwrap_or_else(|e| {
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
    sparq_core::Graph::from_parts(dict, triples)
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
    eprintln!("reasoned [N3]: {} ground triples in closure in {:.3}s", triples.len(), t.elapsed().as_secs_f64());
    sparq_core::Graph::from_parts(dict, triples)
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
        let (closure, proof) = sparq_reason::reason_n3_proof(&mut dict, &text).unwrap_or_else(|e| {
            eprintln!("n3 reasoning error: {e}");
            std::process::exit(1);
        });
        let t = |id| dict.term(id).to_string();
        println!("{} triples in closure; {} derivation step(s):", closure.len(), proof.len());
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
            writeln!(w, "{} {} {} .", g.dict.term(spo[0]), g.dict.term(spo[1]), g.dict.term(spo[2])).unwrap();
        }
        w.flush().unwrap();
        eprintln!("wrote closure to {out}");
    }
}

/// `query-mmap <dir> <sparql>` — open a saved dataset with memory-mapped indexes and
/// run a query. Reports load time + the store self-estimate (≈0 GB heap for the
/// mmap'd permutations — they live in the OS page cache, not the process heap).
fn cmd_query_mmap(args: &[String]) {
    let (dir, sparql) = match (args.get(2), args.get(3)) {
        (Some(d), Some(q)) => (d.as_str(), q.as_str()),
        _ => {
            eprintln!("usage: sparq-cli query-mmap <dir> <sparql>");
            std::process::exit(2);
        }
    };
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
    let t = Instant::now();
    match sparq_engine::count(&g, sparql) {
        Ok(n) => println!("{n} solutions in {:.3}ms", t.elapsed().as_secs_f64() * 1e3),
        Err(e) => {
            eprintln!("query error: {e}");
            std::process::exit(1);
        }
    }
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

/// Load without the timing/size summary line — used by the scaling harness, which loads many
/// times and prints its own table.
fn load_quiet(path: &str, format: &str) -> sparq_core::Graph {
    let die = |e: String| -> ! {
        eprintln!("error loading {path}: {e}");
        std::process::exit(1);
    };
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
        open_reader(path).and_then(|mut r| r.read_to_string(&mut text)).unwrap_or_else(|e| die(e.to_string()));
        // N-Quads / TriG carry named graphs — load them as a dataset so GRAPH queries work.
        if matches!(format, "nquads" | "n-quads" | "trig" | "application/trig") {
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

fn cmd_query(args: &[String]) {
    let (path, format, sparql) = match (args.get(2), args.get(3), args.get(4)) {
        (Some(p), Some(f), Some(q)) => (p, f, q),
        _ => {
            eprintln!("usage: sparq-cli query <data-file> <format> <sparql>");
            std::process::exit(2);
        }
    };
    let g = match reason_flag(args) {
        Some(profile) => load_with_reasoning(path, format, &profile),
        None => load(path, format),
    };
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
    run_query_suite(&g, dir, iters, mode);
}

/// `bench-mmap <dir> <queries-dir> [iters] [count|materialize|json]` — same as `bench`
/// but OPENS the dataset out-of-core (memory-mapped), so the compute of a >RAM index can
/// be measured without loading it into RAM. Used to compare sparq's compute against the
/// stored QLever baselines at 100M+ on a 16 GB machine.
fn cmd_bench_mmap(args: &[String]) {
    let (dir, qdir) = match (args.get(2), args.get(3)) {
        (Some(d), Some(q)) => (d.as_str(), q.as_str()),
        _ => {
            eprintln!("usage: sparq-cli bench-mmap <index-dir> <queries-dir> [iters] [count|materialize|json] [decompress]");
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
    eprintln!("opened {} triples (mmap) in {:.3}s | committed heap ~{:.3} GB", g.len(), t.elapsed().as_secs_f64(), g.heap_bytes() as f64 / 1e9);
    // Load-time decompression mode: decode compressed perms to raw RAM before querying.
    if args.get(6).map(String::as_str) == Some("decompress") {
        let t = Instant::now();
        g.decompress_indexes();
        eprintln!("decompressed indexes to RAM in {:.3}s | committed heap ~{:.3} GB", t.elapsed().as_secs_f64(), g.heap_bytes() as f64 / 1e9);
    }
    run_query_suite(&g, qdir, iters, mode);
}

/// Runs every `*.rq` in `dir` (sorted) `iters` times in `mode`, printing one TSV line
/// per query: `<name>\t<rows>\t<min_micros>`.
fn run_query_suite(g: &sparq_core::Graph, dir: &str, iters: usize, mode: &str) {
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
        // [OPUS-4.8] Route the graph-valued forms (CONSTRUCT / DESCRIBE) through the
        // construct/describe executor — count()/query()/query_json() only handle SELECT/ASK.
        // This lets the operator-coverage suite (bench/operators/queries) exercise every
        // SPARQL query form through the same `bench` runner; "rows" = produced triples.
        let graph_form = sparq_engine::PreparedQuery::parse(&sparql).map(|p| p.is_graph_form()).unwrap_or(false);
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
        match err {
            None => println!("{name}\t{rows}\t{best:.1}"),
            Some(e) => println!("{name}\tERROR\t{e}"),
        }
    }
}
