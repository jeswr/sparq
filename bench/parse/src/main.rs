//! Baseline measurements for the custom-parsers design (one bin, subcommands).
//! Numbers land in research/custom-parsers-baseline.md.
//!
//! Subcommands:
//!   gen <entities> <out.nt> <out.ttl>   deterministic synthetic dataset (same
//!                                       generator as crates/sparq-bench/src/dataset.rs)
//!   to-ttl <in.nt> <out.ttl>            convert N-Triples to Turtle via the oxttl
//!                                       serializer (predicate/object grouping, a few
//!                                       Wikidata prefixes registered)
//!   compress <file>                     write <file>.gz (flate2 -6) and <file>.zst (level 3)
//!   bench-nt <file.nt>                  parser/ingest measurements on N-Triples
//!   bench-ttl <file.ttl>                parser/ingest measurements on Turtle
//!   bench-zip <file.nt>                 decode-only + two-stage vs streaming ingest
//!                                       (expects <file.nt>.gz and <file.nt>.zst)

// Match the production CLI: sparq-cli ingest runs under mimalloc.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use sparq_core::Graph;
use std::hint::black_box;
use std::io::{Read, Write};
use std::time::Instant;

const ITERS: usize = 3;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("gen") => gen(args[2].parse().unwrap(), &args[3], &args[4]),
        Some("to-ttl") => to_ttl(&args[2], &args[3]),
        Some("compress") => compress(&args[2]),
        Some("bench-nt") => bench_nt(&args[2]),
        Some("bench-ttl") => bench_ttl(&args[2]),
        Some("bench-zip") => bench_zip(&args[2]),
        Some("probe-read") => probe_read(&args[2]),
        _ => {
            eprintln!("usage: parse-baseline gen|to-ttl|compress|bench-nt|bench-ttl|bench-zip ...");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------------------
// timing
// ---------------------------------------------------------------------------

/// Runs `f` ITERS times, returns the median wall seconds.
fn median<F: FnMut()>(mut f: F) -> f64 {
    let mut times: Vec<f64> = (0..ITERS)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64()
        })
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

/// Prints one markdown row: task, threads, seconds, input MB/s, Mtriples/s.
fn row(dataset: &str, task: &str, threads: usize, secs: f64, bytes: usize, triples: usize) {
    let mbs = bytes as f64 / 1e6 / secs;
    let mts = triples as f64 / 1e6 / secs;
    println!("| {dataset} | {task} | {threads} | {secs:.3} | {mbs:.0} | {mts:.2} |");
}

fn pool(n: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new().num_threads(n).build().unwrap()
}

fn dataset_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn read_to_string(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

// ---------------------------------------------------------------------------
// bench-nt
// ---------------------------------------------------------------------------

fn bench_nt(path: &str) {
    let text = read_to_string(path);
    let bytes = text.as_bytes();
    let name = dataset_name(path);
    let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);

    // Triple count (for triples/s) via oxttl, also validates the slice parses.
    let triples = oxttl::NTriplesParser::new()
        .for_slice(bytes)
        .map(|r| r.expect("dataset must parse"))
        .count();
    eprintln!("{name}: {} bytes, {triples} triples", bytes.len());
    println!("| dataset | task | threads | s | MB/s | Mtriples/s |");
    println!("|---|---|---|---|---|---|");

    // Memory-traversal ceiling: sum the buffer as u64 words (what "the parser is
    // memory-bound" would look like).
    let secs = median(|| {
        let mut acc = 0u64;
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            acc = acc.wrapping_add(u64::from_le_bytes(c.try_into().unwrap()));
        }
        for &b in chunks.remainder() {
            acc = acc.wrapping_add(b as u64);
        }
        black_box(acc);
    });
    row(name, "memscan (sum bytes, 1 core)", 1, secs, bytes.len(), triples);

    // oxttl, parse only (discard triples): pure parser cost, no interning/index.
    let secs = median(|| {
        let n = oxttl::NTriplesParser::new().for_slice(bytes).map(|r| r.unwrap()).count();
        black_box(n);
    });
    row(name, "oxttl NT parse-only", 1, secs, bytes.len(), triples);

    // oxttl + intern into Dict (no index build): what Graph::load_reader does
    // before Graph::build.
    let secs = median(|| {
        let mut dict = sparq_core::dict::Dict::new();
        let mut out: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
        for t in oxttl::NTriplesParser::new().for_slice(bytes) {
            let t = t.unwrap();
            let s = dict.intern(&subject_to_term(&t.subject));
            let p = dict.intern(&oxrdf::Term::NamedNode(t.predicate.clone()));
            let o = dict.intern(&t.object);
            out.push([s, p, o]);
        }
        black_box(out.len());
    });
    row(name, "oxttl NT parse+intern", 1, secs, bytes.len(), triples);

    // oxttl end-to-end into Graph (serial streaming loader = the non-NT real path).
    let secs = median(|| {
        let g = Graph::load_reader(std::io::Cursor::new(bytes), "ntriples").unwrap();
        black_box(g.len());
    });
    row(name, "oxttl NT -> Graph (load_reader)", 1, secs, bytes.len(), triples);

    // Incumbent custom byte-level parser (sparq_core::nt via parse_to_triples),
    // single-threaded and parallel; then the full Graph build on top.
    for &n in &[1usize, ncpu] {
        let p = pool(n);
        let secs = median(|| {
            let (d, t) = p.install(|| Graph::parse_to_triples(&text, "ntriples")).unwrap();
            black_box((d.len(), t.len()));
        });
        row(name, "custom NT parse+intern (incumbent)", n, secs, bytes.len(), triples);

        let secs = median(|| {
            let g = p.install(|| Graph::load_str(&text, "ntriples")).unwrap();
            black_box(g.len());
        });
        row(name, "custom NT -> Graph (load_str, incumbent)", n, secs, bytes.len(), triples);
    }
}

fn subject_to_term(s: &oxrdf::NamedOrBlankNode) -> oxrdf::Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => oxrdf::Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => oxrdf::Term::BlankNode(b.clone()),
    }
}

// ---------------------------------------------------------------------------
// bench-ttl
// ---------------------------------------------------------------------------

fn bench_ttl(path: &str) {
    let text = read_to_string(path);
    let bytes = text.as_bytes();
    let name = dataset_name(path);
    let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);

    let triples = oxttl::TurtleParser::new()
        .for_slice(bytes)
        .map(|r| r.expect("dataset must parse"))
        .count();
    eprintln!("{name}: {} bytes, {triples} triples", bytes.len());
    println!("| dataset | task | threads | s | MB/s | Mtriples/s |");
    println!("|---|---|---|---|---|---|");

    // oxttl, parse only.
    let secs = median(|| {
        let n = oxttl::TurtleParser::new().for_slice(bytes).map(|r| r.unwrap()).count();
        black_box(n);
    });
    row(name, "oxttl Turtle parse-only", 1, secs, bytes.len(), triples);

    // oxttl + intern (no index).
    let secs = median(|| {
        let mut dict = sparq_core::dict::Dict::new();
        let mut out: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
        for t in oxttl::TurtleParser::new().for_slice(bytes) {
            let t = t.unwrap();
            let s = dict.intern(&subject_to_term(&t.subject));
            let p = dict.intern(&oxrdf::Term::NamedNode(t.predicate.clone()));
            let o = dict.intern(&t.object);
            out.push([s, p, o]);
        }
        black_box(out.len());
    });
    row(name, "oxttl Turtle parse+intern", 1, secs, bytes.len(), triples);

    // Incumbent path (parse_turtle_parallel chunks the doc, oxttl parses chunks).
    for &n in &[1usize, ncpu] {
        let p = pool(n);
        let secs = median(|| {
            let (d, t) = p.install(|| Graph::parse_to_triples(&text, "turtle")).unwrap();
            black_box((d.len(), t.len()));
        });
        row(name, "Turtle parse+intern (incumbent chunked)", n, secs, bytes.len(), triples);

        let secs = median(|| {
            let g = p.install(|| Graph::load_str(&text, "turtle")).unwrap();
            black_box(g.len());
        });
        row(name, "Turtle -> Graph (load_str)", n, secs, bytes.len(), triples);
    }
}

// ---------------------------------------------------------------------------
// bench-zip
// ---------------------------------------------------------------------------

fn bench_zip(path: &str) {
    let raw = std::fs::read(path).unwrap();
    let gz = std::fs::read(format!("{path}.gz")).unwrap();
    let zst = std::fs::read(format!("{path}.zst")).unwrap();
    let name = dataset_name(path);
    let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    let p = pool(ncpu);

    let triples = oxttl::NTriplesParser::new()
        .for_slice(&raw)
        .map(|r| r.expect("dataset must parse"))
        .count();
    eprintln!(
        "{name}: raw {} B, gz {} B ({:.2}x), zst {} B ({:.2}x), {triples} triples",
        raw.len(),
        gz.len(),
        raw.len() as f64 / gz.len() as f64,
        zst.len(),
        raw.len() as f64 / zst.len() as f64
    );
    println!("| dataset | task | threads | s | MB/s (decompressed) | Mtriples/s |");
    println!("|---|---|---|---|---|---|");

    // Decode only, discard output: bounds what any fused unzip+parse could save.
    let mut sink = vec![0u8; 1 << 20];
    let secs = median(|| {
        let mut d = flate2::read::MultiGzDecoder::new(&gz[..]);
        let mut n = 0usize;
        loop {
            let k = d.read(&mut sink).unwrap();
            if k == 0 {
                break;
            }
            n += k;
        }
        assert_eq!(n, raw.len());
        black_box(n);
    });
    row(name, "gzip decode-only (flate2)", 1, secs, raw.len(), triples);

    let secs = median(|| {
        let mut d = zstd::stream::read::Decoder::new(&zst[..]).unwrap();
        let mut n = 0usize;
        loop {
            let k = d.read(&mut sink).unwrap();
            if k == 0 {
                break;
            }
            n += k;
        }
        assert_eq!(n, raw.len());
        black_box(n);
    });
    row(name, "zstd decode-only", 1, secs, raw.len(), triples);

    // Two-stage: decompress fully to a String, then parallel load_str.
    for (codec, comp) in [("gzip", &gz), ("zstd", &zst)] {
        let secs = median(|| {
            let mut buf = Vec::with_capacity(raw.len());
            match codec {
                "gzip" => {
                    flate2::read::MultiGzDecoder::new(&comp[..]).read_to_end(&mut buf).unwrap()
                }
                _ => zstd::stream::read::Decoder::new(&comp[..])
                    .unwrap()
                    .read_to_end(&mut buf)
                    .unwrap(),
            };
            let text = String::from_utf8(buf).unwrap();
            let g = p.install(|| Graph::load_str(&text, "ntriples")).unwrap();
            black_box(g.len());
        });
        row(name, &format!("{codec} two-stage decode->load_str"), ncpu, secs, raw.len(), triples);

        // Streaming: decompressor feeds load_reader_parallel's 32 MiB block loop
        // (= what `sparq-cli ingest` does today).
        let secs = median(|| {
            let g = p
                .install(|| match codec {
                    "gzip" => Graph::load_reader_parallel(
                        flate2::read::MultiGzDecoder::new(&comp[..]),
                        "ntriples",
                    ),
                    _ => Graph::load_reader_parallel(
                        zstd::stream::read::Decoder::new(&comp[..]).unwrap(),
                        "ntriples",
                    ),
                })
                .unwrap();
            black_box(g.len());
        });
        row(name, &format!("{codec} streaming decode->load_reader_parallel"), ncpu, secs, raw.len(), triples);
    }
}

/// Diagnostic: what does one `read()` into a 32 MiB buffer actually return for each
/// decompressor? Graph::load_reader_parallel flushes a parse block per read() call, so
/// small reads mean tiny parse blocks (the streaming-slowdown hypothesis).
fn probe_read(path: &str) {
    let gz = std::fs::read(format!("{path}.gz")).unwrap();
    let zst = std::fs::read(format!("{path}.zst")).unwrap();
    let mut buf = vec![0u8; 32 << 20];

    let mut stats = |name: &str, mut r: Box<dyn Read>| {
        let (mut reads, mut total, mut min, mut max) = (0usize, 0usize, usize::MAX, 0usize);
        loop {
            let n = r.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            reads += 1;
            total += n;
            min = min.min(n);
            max = max.max(n);
        }
        eprintln!(
            "{name}: {reads} reads, avg {} B, min {min} B, max {max} B (into a 32 MiB buffer)",
            total / reads.max(1)
        );
    };
    stats("gzip (MultiGzDecoder)", Box::new(flate2::read::MultiGzDecoder::new(std::io::Cursor::new(gz))));
    stats("zstd (Decoder)", Box::new(zstd::stream::read::Decoder::new(std::io::Cursor::new(zst)).unwrap()));
}

// ---------------------------------------------------------------------------
// dataset prep
// ---------------------------------------------------------------------------

/// Same deterministic generator as crates/sparq-bench/src/dataset.rs (write_nt /
/// generate), copied so this standalone project doesn't pull the bench crate
/// (which links oxigraph).
fn gen(n: u32, nt_path: &str, ttl_path: &str) {
    let n = n.max(1);
    let follows_per = 4u32;
    let n_cities = (n / 10).max(1);

    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut rng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };
    let mut nt = std::io::BufWriter::new(std::fs::File::create(nt_path).unwrap());
    let mut triples = 0usize;
    for i in 0..n {
        writeln!(nt, "<http://ex/n{i}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Person> .").unwrap();
        writeln!(nt, "<http://ex/n{i}> <http://ex/name> \"name{i}\" .").unwrap();
        writeln!(
            nt,
            "<http://ex/n{i}> <http://ex/age> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .",
            20 + i % 80
        )
        .unwrap();
        writeln!(nt, "<http://ex/n{i}> <http://ex/city> <http://ex/c{}> .", i % n_cities).unwrap();
        triples += 4;
        for _ in 0..follows_per {
            writeln!(nt, "<http://ex/n{i}> <http://ex/follows> <http://ex/n{}> .", rng() % n).unwrap();
            triples += 1;
        }
    }
    nt.flush().unwrap();

    // Turtle variant: same data, prefixed + predicate-grouped (the realistic
    // human-authored shape, same as sparq-bench's generate()).
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut rng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as u32
    };
    let mut ttl = std::io::BufWriter::new(std::fs::File::create(ttl_path).unwrap());
    writeln!(ttl, "@prefix ex: <http://ex/> .").unwrap();
    for i in 0..n {
        write!(
            ttl,
            "ex:n{i} a ex:Person ;\n  ex:name \"name{i}\" ;\n  ex:age {} ;\n  ex:city ex:c{}",
            20 + i % 80,
            i % n_cities
        )
        .unwrap();
        for _ in 0..follows_per {
            write!(ttl, " ;\n  ex:follows ex:n{}", rng() % n).unwrap();
        }
        writeln!(ttl, " .").unwrap();
    }
    ttl.flush().unwrap();
    eprintln!("gen: {n} entities, {triples} triples -> {nt_path}, {ttl_path}");
}

fn to_ttl(nt_path: &str, ttl_path: &str) {
    let bytes = std::fs::read(nt_path).unwrap();
    let mut ser = oxttl::TurtleSerializer::new()
        .with_prefix("wd", "http://www.wikidata.org/entity/")
        .unwrap()
        .with_prefix("wdt", "http://www.wikidata.org/prop/direct/")
        .unwrap()
        .with_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#")
        .unwrap()
        .with_prefix("schema", "http://schema.org/")
        .unwrap()
        .with_prefix("skos", "http://www.w3.org/2004/02/skos/core#")
        .unwrap()
        .for_writer(std::io::BufWriter::new(std::fs::File::create(ttl_path).unwrap()));
    let mut n = 0usize;
    for t in oxttl::NTriplesParser::new().for_slice(&bytes) {
        ser.serialize_triple(&t.unwrap()).unwrap();
        n += 1;
    }
    ser.finish().unwrap().flush().unwrap();
    eprintln!("to-ttl: {n} triples -> {ttl_path}");
}

fn compress(path: &str) {
    let raw = std::fs::read(path).unwrap();

    let f = std::fs::File::create(format!("{path}.gz")).unwrap();
    let mut enc = flate2::write::GzEncoder::new(std::io::BufWriter::new(f), flate2::Compression::new(6));
    enc.write_all(&raw).unwrap();
    enc.finish().unwrap().flush().unwrap();

    let f = std::fs::File::create(format!("{path}.zst")).unwrap();
    let mut enc = zstd::stream::write::Encoder::new(std::io::BufWriter::new(f), 3).unwrap();
    enc.write_all(&raw).unwrap();
    enc.finish().unwrap().flush().unwrap();

    let gz = std::fs::metadata(format!("{path}.gz")).unwrap().len();
    let zst = std::fs::metadata(format!("{path}.zst")).unwrap().len();
    eprintln!(
        "compress: {} B -> gz {} B ({:.2}x), zst {} B ({:.2}x)",
        raw.len(),
        gz,
        raw.len() as f64 / gz as f64,
        zst,
        raw.len() as f64 / zst as f64
    );
}
