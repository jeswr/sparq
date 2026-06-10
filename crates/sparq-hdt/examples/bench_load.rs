//! Load-throughput sketch: HDT archive vs gzipped N-Triples on ~1M synthetic triples.
//!
//! HDT's win is the compact on-disk size plus no-text-parse loading: the dictionary
//! is decompressed term-by-term (each distinct term once) and the triples arrive as
//! ids, versus tokenizing + interning ~100 MB of N-Triples text.
//!
//! Usage (generates `crates/sparq-hdt/bench-data/` on first run, gitignored):
//! ```sh
//! cargo run --release -p sparq-hdt --example bench_load
//! ```

use std::io::Write;
use std::path::Path;
use std::time::Instant;

const N_TRIPLES: usize = 1_000_000;
const RUNS: usize = 3;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench-data");
    std::fs::create_dir_all(&dir).unwrap();
    let nt_path = dir.join("bench.nt");
    let gz_path = dir.join("bench.nt.gz");
    let hdt_path = dir.join("bench.hdt");

    if !nt_path.exists() || !gz_path.exists() || !hdt_path.exists() {
        generate(&nt_path, &gz_path, &hdt_path);
    }

    let size = |p: &Path| std::fs::metadata(p).unwrap().len();
    println!("file sizes:");
    println!("  bench.nt     {:>10} bytes", size(&nt_path));
    println!("  bench.nt.gz  {:>10} bytes", size(&gz_path));
    println!("  bench.hdt    {:>10} bytes", size(&hdt_path));

    // HDT -> Graph.
    let mut best_hdt = f64::MAX;
    let mut n = 0;
    for _ in 0..RUNS {
        let t = Instant::now();
        let g = sparq_hdt::load(&hdt_path).unwrap();
        let dt = t.elapsed().as_secs_f64();
        n = g.store.len();
        best_hdt = best_hdt.min(dt);
    }
    println!("HDT    load: {n} triples in {best_hdt:.3}s  ({:.0} triples/s)", n as f64 / best_hdt);

    // .nt.gz -> Graph (streaming decompress + parse).
    let mut best_gz = f64::MAX;
    for _ in 0..RUNS {
        let t = Instant::now();
        let f = std::fs::File::open(&gz_path).unwrap();
        let g = sparq_core::Graph::load_reader(flate2::read::GzDecoder::new(f), "ntriples").unwrap();
        let dt = t.elapsed().as_secs_f64();
        assert_eq!(g.store.len(), n);
        best_gz = best_gz.min(dt);
    }
    println!(".nt.gz load: {n} triples in {best_gz:.3}s  ({:.0} triples/s)", n as f64 / best_gz);
    println!("speedup: {:.2}x", best_gz / best_hdt);
}

/// Deterministic synthetic graph: clustered subjects, a small predicate set, and a
/// mix of IRI objects (drawn from the subject pool, exercising HDT's shared
/// section) and plain / language-tagged / numeric literals.
fn generate(nt_path: &Path, gz_path: &Path, hdt_path: &Path) {
    eprintln!("generating {N_TRIPLES} synthetic triples (one-time)...");
    let mut nt = String::with_capacity(N_TRIPLES * 100);
    let mut st = 0x243f6a88u64;
    let mut rng = || {
        st ^= st << 13;
        st ^= st >> 7;
        st ^= st << 17;
        st
    };
    for _ in 0..N_TRIPLES {
        let s = rng() % 100_000;
        let p = rng() % 50;
        nt.push_str(&format!("<http://bench.example/e{s}> <http://bench.example/vocab/p{p}> "));
        match rng() % 5 {
            0 => nt.push_str(&format!("<http://bench.example/e{}>", rng() % 100_000)),
            1 => nt.push_str(&format!("\"label {}\"", rng() % 1_000_000)),
            2 => nt.push_str(&format!("\"etikett {}\"@de", rng() % 1_000_000)),
            3 => nt.push_str(&format!("\"{}\"^^<http://www.w3.org/2001/XMLSchema#integer>", rng() % 100_000)),
            _ => nt.push_str(&format!(
                "\"{}.{:02}\"^^<http://www.w3.org/2001/XMLSchema#decimal>",
                rng() % 1000,
                rng() % 100
            )),
        }
        nt.push_str(" .\n");
    }
    std::fs::write(nt_path, &nt).unwrap();

    let gz_file = std::fs::File::create(gz_path).unwrap();
    let mut enc = flate2::write::GzEncoder::new(std::io::BufWriter::new(gz_file), flate2::Compression::default());
    enc.write_all(nt.as_bytes()).unwrap();
    enc.finish().unwrap().flush().unwrap();

    eprintln!("converting to HDT (one-time)...");
    let hdt = hdt::Hdt::read_nt(nt_path).unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(hdt_path).unwrap());
    hdt.write(&mut out).unwrap();
}
