//! RESEARCH SPIKE — result-cache hit-path ceiling (research/concurrent-serving.md §d.ii).
//!
//! Models the proposed fast path: hash the raw query string → pre-serialized response
//! bytes (`Arc<Vec<u8>>`). Measures, on this machine:
//!
//! * raw SPARQL parse cost (spargebra) for a point query — the cost every NON-cached
//!   request pays before anything else;
//! * single-thread cache-hit ops/s under a Zipfian repeat distribution (the optimistic
//!   in-process ceiling the Mreq/s claim must be calibrated against);
//! * multi-thread ops/s for (a) `RwLock<HashMap>` and (b) a sharded read-mostly map,
//!   showing what lock choice costs at this concurrency;
//! * the DEGENERATE case: all-distinct queries (100% miss) — the pure overhead the
//!   cache machinery adds to requests it cannot help (hash + lookup + insert).
//!
//! No HTTP here on purpose: the loadgen baseline tells us what the HTTP layer costs;
//! this isolates the cache data structure itself.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const KEYS: usize = 10_000;
const RESPONSE_BYTES: usize = 1024;
const ZIPF_S: f64 = 1.0;

fn main() {
    let threads: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);

    // --- parse cost calibration -------------------------------------------------
    let q = "PREFIX foaf: <http://xmlns.com/foaf/0.1/> SELECT ?n WHERE { <http://example.org/person/123> foaf:name ?n }";
    let t = Instant::now();
    let iters = 20_000;
    for _ in 0..iters {
        let p = spargebra::SparqlParser::new().parse_query(q).unwrap();
        std::hint::black_box(&p);
    }
    let per = t.elapsed().as_nanos() as f64 / iters as f64;
    println!("spargebra parse (point query, {} chars): {:.2} us/parse", q.len(), per / 1000.0);

    // --- build the cache ----------------------------------------------------------
    let queries: Vec<String> = (0..KEYS)
        .map(|i| format!("SELECT ?n WHERE {{ <http://example.org/person/{i}> <http://xmlns.com/foaf/0.1/name> ?n }}"))
        .collect();
    let blob: Arc<Vec<u8>> = Arc::new(vec![b'x'; RESPONSE_BYTES]);
    let mut map: HashMap<String, Arc<Vec<u8>>> = HashMap::with_capacity(KEYS * 2);
    for s in &queries {
        map.insert(s.clone(), blob.clone());
    }
    let cache = Arc::new(RwLock::new(map));

    // Zipfian sampler over KEYS ranks (inverse-CDF on a precomputed table).
    let cdf = zipf_cdf(KEYS, ZIPF_S);

    // --- single-thread hit path -----------------------------------------------------
    let ops = 5_000_000u64;
    let t = Instant::now();
    let mut rng = 0xdeadbeefu64;
    let mut sink = 0usize;
    for _ in 0..ops {
        let k = &queries[zipf_sample(&cdf, &mut rng)];
        let guard = cache.read().unwrap();
        if let Some(v) = guard.get(k) {
            sink = sink.wrapping_add(v.len());
        }
    }
    let el = t.elapsed().as_secs_f64();
    println!(
        "cache hit, 1 thread (RwLock<HashMap>, SipHash): {:.2} M ops/s ({:.0} ns/op) [{sink}]",
        ops as f64 / el / 1e6,
        el / ops as f64 * 1e9
    );

    // --- multi-thread hit path: RwLock<HashMap> ----------------------------------
    bench_threads("RwLock<HashMap>", threads, &queries, &cdf, {
        let cache = cache.clone();
        move |k| cache.read().unwrap().get(k).map(|v| v.len()).unwrap_or(0)
    });

    // --- multi-thread hit path: sharded maps (no writer contention modelled) ------
    const SHARDS: usize = 64;
    let mut shards: Vec<HashMap<String, Arc<Vec<u8>>>> = (0..SHARDS).map(|_| HashMap::new()).collect();
    for s in &queries {
        shards[shard_of(s, SHARDS)].insert(s.clone(), blob.clone());
    }
    let shards: Arc<Vec<RwLock<HashMap<String, Arc<Vec<u8>>>>>> =
        Arc::new(shards.into_iter().map(RwLock::new).collect());
    bench_threads("sharded(64) RwLock", threads, &queries, &cdf, {
        let shards = shards.clone();
        move |k| {
            let g = shards[shard_of(k, SHARDS)].read().unwrap();
            g.get(k).map(|v| v.len()).unwrap_or(0)
        }
    });

    // --- multi-thread: lock-free reads via an immutable Arc-swapped map ----------
    // Models the "generation cache": readers clone an Arc to the whole map (as the
    // server already does for the graph) and look up with zero locking.
    let frozen: Arc<HashMap<String, Arc<Vec<u8>>>> = Arc::new(cache.read().unwrap().clone());
    bench_threads("Arc-swapped immutable map", threads, &queries, &cdf, {
        let frozen = frozen.clone();
        move |k| frozen.get(k).map(|v| v.len()).unwrap_or(0)
    });

    // --- degenerate: all-distinct queries, 100% miss ------------------------------
    // The overhead the cache adds to a workload it cannot help: hash + miss lookup
    // (we do NOT model insert-on-miss here; a real cache would also pay an insert,
    // measured separately below).
    let distinct: Vec<String> = (0..1_000_000)
        .map(|i| format!("SELECT ?n WHERE {{ <http://example.org/distinct/{i}> <http://xmlns.com/foaf/0.1/name> ?n }}"))
        .collect();
    let t = Instant::now();
    let mut sink = 0usize;
    for k in &distinct {
        let g = cache.read().unwrap();
        if let Some(v) = g.get(k) {
            sink = sink.wrapping_add(v.len());
        }
    }
    let el = t.elapsed().as_secs_f64();
    println!(
        "cache MISS lookup (all-distinct, 1 thread): {:.0} ns/op — pure per-request overhead when the cache cannot help [{sink}]",
        el / distinct.len() as f64 * 1e9
    );

    // Insert-on-miss cost (what an all-distinct workload would also pay, with the
    // eviction policy ignored — i.e. a lower bound).
    let t = Instant::now();
    {
        let mut g = cache.write().unwrap();
        for k in distinct.iter().take(200_000) {
            g.insert(k.clone(), blob.clone());
        }
    }
    println!(
        "cache insert (lower bound, lock held once): {:.0} ns/op",
        t.elapsed().as_secs_f64() / 200_000.0 * 1e9
    );
}

fn bench_threads(
    label: &str,
    threads: usize,
    queries: &[String],
    cdf: &Arc<Vec<f64>>,
    f: impl Fn(&str) -> usize + Send + Sync + Clone + 'static,
) {
    let ops_per_thread = 3_000_000u64;
    let queries = Arc::new(queries.to_vec());
    let t = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let q = queries.clone();
            let cdf = cdf.clone();
            let f = f.clone();
            std::thread::spawn(move || {
                let mut rng = 0x12345678u64.wrapping_add(i as u64 * 0x9e3779b9);
                let mut sink = 0usize;
                for _ in 0..ops_per_thread {
                    let k = &q[zipf_sample(&cdf, &mut rng)];
                    sink = sink.wrapping_add(f(k));
                }
                sink
            })
        })
        .collect();
    let mut sink = 0usize;
    for h in handles {
        sink = sink.wrapping_add(h.join().unwrap());
    }
    let el = t.elapsed().as_secs_f64();
    let total = ops_per_thread * threads as u64;
    println!(
        "cache hit, {threads} threads ({label}): {:.2} M ops/s total ({:.0} ns/op/thread) [{sink}]",
        total as f64 / el / 1e6,
        el / (total as f64 / threads as f64) * 1e9
    );
}

fn zipf_cdf(n: usize, s: f64) -> Arc<Vec<f64>> {
    let mut weights: Vec<f64> = (1..=n).map(|k| 1.0 / (k as f64).powf(s)).collect();
    let sum: f64 = weights.iter().sum();
    let mut acc = 0.0;
    for w in weights.iter_mut() {
        acc += *w / sum;
        *w = acc;
    }
    Arc::new(weights)
}

fn zipf_sample(cdf: &[f64], rng: &mut u64) -> usize {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    let u = (*rng >> 11) as f64 / (1u64 << 53) as f64;
    cdf.partition_point(|&c| c < u).min(cdf.len() - 1)
}

fn shard_of(s: &str, shards: usize) -> usize {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    (h.finish() as usize) % shards
}
