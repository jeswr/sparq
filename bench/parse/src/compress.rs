//! D4 measurements: compressed (multi-member gzip / multi-frame zstd)
//! serialization of query results. Numbers land in
//! research/custom-parsers-D4-compressed-serialization.md.
//!
//! Subcommands:
//!   big <file.nt>             large-result benchmark: serialize a full
//!                             SELECT ?s ?p ?o over the dataset to SPARQL-JSON
//!                             chunks (the engine's streaming path), then
//!                             ratio / serial-vs-parallel compress wall /
//!                             end-to-end serialize+compress / TTFB proxy.
//!   small <file.nt>           dictionary benchmark: point-query-sized JSON
//!                             responses; zstd levels 1-3 with and without a
//!                             trained vocabulary dictionary vs gzip -1/-6.
//!   artifacts <file.nt> <dir> write multi-member/multi-frame streams + dict
//!                             for external reference-decoder validation
//!                             (gzip -d, python gzip, zstd CLI).

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use sparq_core::Graph;
use sparq_engine::{query, query_json, query_json_chunks_with_budget, QueryBudget};
use sparq_parse::{
    compress_chunks, compress_member, decode_gzip_concat, decode_zstd_concat,
    decode_zstd_concat_with_dict, gzip_single_member, train_dictionary, Codec, CompressedSink,
    Mode, PreparedDict, SingleMemberGzipSink,
};
use std::hint::black_box;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

const ITERS: usize = 3;
const ALL_ROWS: &str = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("big") => big(&args[2]),
        Some("small") => small(&args[2]),
        Some("artifacts") => artifacts(&args[2], &args[3]),
        _ => {
            eprintln!("usage: compress-bench big|small|artifacts ...");
            std::process::exit(2);
        }
    }
}

fn median_secs<F: FnMut()>(mut f: F) -> f64 {
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

fn load(path: &str) -> Graph {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    eprintln!("loading {path} ({} bytes)…", text.len());
    Graph::load_str(&text, "ntriples").expect("dataset must load")
}

fn big_codecs() -> Vec<Codec> {
    vec![
        Codec::Gzip { level: 1 },
        Codec::Gzip { level: 6 },
        Codec::Zstd { level: 1 },
        Codec::Zstd { level: 3 },
    ]
}

// ---------------------------------------------------------------------------
// big: large-result serialize + compress
// ---------------------------------------------------------------------------

fn big(path: &str) {
    let g = load(path);
    let budget = QueryBudget::unlimited();
    let threads = rayon::current_num_threads();

    // Serialize-alone reference: the engine's chunked SPARQL-JSON path.
    let mut chunks: Vec<String> = Vec::new();
    let t_ser = median_secs(|| {
        chunks = query_json_chunks_with_budget(&g, ALL_ROWS, &budget).unwrap();
        black_box(&chunks);
    });
    let raw: usize = chunks.iter().map(String::len).sum();
    eprintln!(
        "JSON result: {} bytes in {} chunks (serialize-alone median {t_ser:.3}s, {:.0} MB/s, rayon {threads} threads)",
        raw,
        chunks.len(),
        raw as f64 / 1e6 / t_ser
    );

    println!("| codec | comp bytes | ratio | serial s (MB/s) | parallel {threads}T s (MB/s) | par speedup | single-stream ratio |");
    println!("|---|---|---|---|---|---|---|");
    let whole: Vec<u8> = chunks.iter().flat_map(|c| c.bytes()).collect();
    for codec in big_codecs() {
        let mut members: Vec<Vec<u8>> = Vec::new();
        let t_serial = median_secs(|| {
            members = compress_chunks(&chunks, &codec, Mode::Serial).unwrap();
            black_box(&members);
        });
        let t_par = median_secs(|| {
            members = compress_chunks(&chunks, &codec, Mode::Parallel).unwrap();
            black_box(&members);
        });
        let comp: usize = members.iter().map(Vec::len).sum();
        // Ratio sanity reference: the same codec over the whole body as ONE
        // member/frame (what a non-chunked Content-Encoding pass would emit).
        let single = compress_member(&codec, &whole).unwrap().len();
        println!(
            "| {} | {} | {:.2}x | {:.3} ({:.0}) | {:.3} ({:.0}) | {:.2}x | {:.2}x |",
            codec.name(),
            comp,
            raw as f64 / comp as f64,
            t_serial,
            raw as f64 / 1e6 / t_serial,
            t_par,
            raw as f64 / 1e6 / t_par,
            t_serial / t_par,
            raw as f64 / single as f64,
        );
    }
    // Browser-safe parallel gzip (ONE member, pigz-style full-flush segments):
    // the variant every gzip consumer decodes, including the first-member-only
    // truncators (browsers, curl, DecompressionStream).
    for level in [1u32, 6] {
        let mut pieces: Vec<Vec<u8>> = Vec::new();
        let t_serial = median_secs(|| {
            pieces = gzip_single_member(&chunks, level, Mode::Serial).unwrap();
            black_box(&pieces);
        });
        let t_par = median_secs(|| {
            pieces = gzip_single_member(&chunks, level, Mode::Parallel).unwrap();
            black_box(&pieces);
        });
        let comp: usize = pieces.iter().map(Vec::len).sum();
        println!(
            "| gzip -{level} 1-member (pigz) | {} | {:.2}x | {:.3} ({:.0}) | {:.3} ({:.0}) | {:.2}x | — |",
            comp,
            raw as f64 / comp as f64,
            t_serial,
            raw as f64 / 1e6 / t_serial,
            t_par,
            raw as f64 / 1e6 / t_par,
            t_serial / t_par,
        );
    }

    // End-to-end: serialize THEN compress (today's batch API, compression is a
    // tail), vs the paced replay below (a streaming producer, Wave D's shape).
    println!();
    println!("| codec | serialize-alone s | + serial compress s | + parallel {threads}T s |");
    println!("|---|---|---|---|");
    for codec in big_codecs() {
        let e2e_serial = median_secs(|| {
            let c = query_json_chunks_with_budget(&g, ALL_ROWS, &budget).unwrap();
            black_box(compress_chunks(&c, &codec, Mode::Serial).unwrap());
        });
        let e2e_par = median_secs(|| {
            let c = query_json_chunks_with_budget(&g, ALL_ROWS, &budget).unwrap();
            black_box(compress_chunks(&c, &codec, Mode::Parallel).unwrap());
        });
        println!("| {} | {t_ser:.3} | {e2e_serial:.3} | {e2e_par:.3} |", codec.name());
    }

    // Paced replay: chunks become available at the measured serialization rate
    // (chunk i at (i+1)/n * t_ser) and are pushed into the parallel sink as they
    // appear — the overlap a streaming server gets. Reports total wall (vs t_ser),
    // the compression tail past the last chunk, and the TTFB proxy (first
    // compressed member on the wire vs full uncompressed serialization t_ser).
    println!();
    println!("| codec | paced replay wall s | tail past serialize s | first member at s | first member bytes |");
    println!("|---|---|---|---|---|");
    for codec in big_codecs() {
        let (mut wall, mut tail, mut first, mut first_bytes) = (f64::MAX, 0.0, 0.0, 0usize);
        for _ in 0..ITERS {
            let r = paced_replay(&chunks, &codec, t_ser);
            if r.0 < wall {
                (wall, tail, first, first_bytes) = r;
            }
        }
        println!("| {} | {wall:.3} | {tail:+.3} | {first:.3} | {first_bytes} |", codec.name());
    }
    // Same paced replay through the single-member gzip sink.
    for level in [1u32, 6] {
        let (mut wall, mut tail, mut first, mut first_bytes) = (f64::MAX, 0.0, 0.0, 0usize);
        for _ in 0..ITERS {
            let r = paced_replay_pigz(&chunks, level, t_ser);
            if r.0 < wall {
                (wall, tail, first, first_bytes) = r;
            }
        }
        println!("| gzip -{level} 1-member (pigz) | {wall:.3} | {tail:+.3} | {first:.3} | {first_bytes} |");
    }
    println!();
    println!("(t_ser = {t_ser:.3}s: chunk i is pushed at (i+1)/n * t_ser; tail = wall - t_ser; best of {ITERS})");
}

fn paced_replay_pigz(chunks: &[String], level: u32, t_ser: f64) -> (f64, f64, f64, usize) {
    let n = chunks.len();
    let mut sink = SingleMemberGzipSink::new(level, Mode::Parallel);
    let mut first: Option<(f64, usize)> = None;
    let start = Instant::now();
    for (i, c) in chunks.iter().enumerate() {
        let due = t_ser * (i + 1) as f64 / n as f64;
        let now = start.elapsed().as_secs_f64();
        if now < due {
            std::thread::sleep(std::time::Duration::from_secs_f64(due - now));
        }
        sink.push(c.as_bytes());
        let got = sink.try_drain().unwrap();
        if first.is_none() {
            if let Some(m) = got.first() {
                first = Some((start.elapsed().as_secs_f64(), m.len()));
            }
        }
        black_box(got);
    }
    let rest = sink.finish().unwrap();
    let wall = start.elapsed().as_secs_f64();
    let (tf, fb) = first.unwrap_or_else(|| (wall, rest.first().map(Vec::len).unwrap_or(0)));
    (wall, wall - t_ser, tf, fb)
}

/// Returns (total wall, wall - t_ser, time of first drained member, its size).
fn paced_replay(chunks: &[String], codec: &Codec, t_ser: f64) -> (f64, f64, f64, usize) {
    let n = chunks.len();
    let mut sink = CompressedSink::new(codec.clone(), Mode::Parallel);
    let mut first: Option<(f64, usize)> = None;
    let start = Instant::now();
    for (i, c) in chunks.iter().enumerate() {
        let due = t_ser * (i + 1) as f64 / n as f64;
        let now = start.elapsed().as_secs_f64();
        if now < due {
            std::thread::sleep(std::time::Duration::from_secs_f64(due - now));
        }
        sink.push(c.as_bytes());
        if first.is_none() {
            let got = sink.try_drain().unwrap();
            if let Some(m) = got.first() {
                first = Some((start.elapsed().as_secs_f64(), m.len()));
            }
            black_box(got);
        } else {
            black_box(sink.try_drain().unwrap());
        }
    }
    let rest = sink.finish().unwrap();
    let wall = start.elapsed().as_secs_f64();
    let (tf, fb) = first.unwrap_or_else(|| (wall, rest.first().map(Vec::len).unwrap_or(0)));
    (wall, wall - t_ser, tf, fb)
}

// ---------------------------------------------------------------------------
// small: point-query responses, dictionary vs not
// ---------------------------------------------------------------------------

/// Point-query-sized SPARQL-JSON responses from the dataset: one
/// `SELECT ?p ?o WHERE { <s> ?p ?o }` body per distinct subject.
fn point_responses(g: &Graph, n: usize) -> Vec<Vec<u8>> {
    let subjects = query(g, &format!("SELECT DISTINCT ?s WHERE {{ ?s ?p ?o }} LIMIT {n}")).unwrap();
    subjects
        .rows
        .iter()
        .filter_map(|r| match r[0].as_ref() {
            Some(oxrdf::Term::NamedNode(s)) => {
                Some(query_json(g, &format!("SELECT ?p ?o WHERE {{ <{}> ?p ?o }}", s.as_str())).unwrap().into_bytes())
            }
            _ => None, // skip blank-node subjects
        })
        .collect()
}

fn small(path: &str) {
    let g = load(path);
    let responses = point_responses(&g, 2400);
    let (train, eval) = responses.split_at(responses.len() / 2);
    let eval: Vec<Vec<u8>> = eval.to_vec();
    let raw: usize = eval.iter().map(Vec::len).sum();
    let mut sizes: Vec<usize> = eval.iter().map(Vec::len).collect();
    sizes.sort_unstable();
    eprintln!(
        "{} point responses held out for eval (train {}): total {} B, median {} B, p10 {} B, p90 {} B",
        eval.len(),
        train.len(),
        raw,
        sizes[sizes.len() / 2],
        sizes[sizes.len() / 10],
        sizes[sizes.len() * 9 / 10]
    );

    let dict64 = train_dictionary(train, 64 * 1024).unwrap();
    let dict16 = train_dictionary(train, 16 * 1024).unwrap();
    eprintln!("dictionaries: 64K-cap -> {} B, 16K-cap -> {} B", dict64.len(), dict16.len());

    let mut codecs = vec![Codec::Gzip { level: 1 }, Codec::Gzip { level: 6 }];
    for level in 1..=3 {
        codecs.push(Codec::Zstd { level });
    }
    for level in 1..=3 {
        codecs.push(Codec::ZstdDict(Arc::new(PreparedDict::new(dict64.clone(), level))));
    }
    codecs.push(Codec::ZstdDict(Arc::new(PreparedDict::new(dict16.clone(), 3))));

    // Per-response compressed sizes per codec, for the size-bucketed view below
    // (the total is dominated by a few huge subjects; the dictionary case is
    // specifically the SMALL responses).
    let mut per_codec_sizes: Vec<(String, Vec<usize>)> = Vec::new();

    println!("| codec | comp bytes | ratio | avg comp B/resp | compress s | comp MB/s | decode s | dec MB/s |");
    println!("|---|---|---|---|---|---|---|---|");
    for codec in codecs {
        let mut frames: Vec<Vec<u8>> = Vec::new();
        // One frame per response, serially: the production shape is one small
        // HTTP response at a time, so per-response (not batch-parallel) cost is
        // the honest number.
        let t_comp = median_secs(|| {
            frames = eval.iter().map(|r| compress_member(&codec, r).unwrap()).collect();
            black_box(&frames);
        });
        let comp: usize = frames.iter().map(Vec::len).sum();
        let t_dec = median_secs(|| match &codec {
            Codec::Gzip { .. } => {
                for f in &frames {
                    black_box(decode_gzip_concat(f).unwrap());
                }
            }
            Codec::Zstd { .. } => {
                let mut d = zstd::bulk::Decompressor::new().unwrap();
                for (f, r) in frames.iter().zip(&eval) {
                    black_box(d.decompress(f, r.len() + 1).unwrap());
                }
            }
            Codec::ZstdDict(p) => {
                let mut d = zstd::bulk::Decompressor::with_prepared_dictionary(p.decoder()).unwrap();
                for (f, r) in frames.iter().zip(&eval) {
                    black_box(d.decompress(f, r.len() + 1).unwrap());
                }
            }
        });
        println!(
            "| {} | {} | {:.2}x | {:.0} | {:.4} | {:.0} | {:.4} | {:.0} |",
            codec.name(),
            comp,
            raw as f64 / comp as f64,
            comp as f64 / eval.len() as f64,
            t_comp,
            raw as f64 / 1e6 / t_comp,
            t_dec,
            raw as f64 / 1e6 / t_dec,
        );
        per_codec_sizes.push((codec.name(), frames.iter().map(Vec::len).collect()));
    }
    println!();
    println!("(eval = held-out half, never seen by the trainer; decode via bulk one-shot, dict pre-digested)");

    // Size-bucketed ratios: the dictionary's value concentrates where there is
    // not enough self-context to compress against.
    println!();
    print!("| raw size bucket | n | raw B |");
    for (name, _) in &per_codec_sizes {
        print!(" {name} |");
    }
    println!();
    println!("|---|---|---|{}", "---|".repeat(per_codec_sizes.len()));
    for (lo, hi, label) in [(0, 1 << 10, "<=1 KiB"), (1 << 10, 4 << 10, "1-4 KiB"), (4 << 10, usize::MAX, ">4 KiB")] {
        let idx: Vec<usize> =
            (0..eval.len()).filter(|&i| eval[i].len() > lo && eval[i].len() <= hi).collect();
        let braw: usize = idx.iter().map(|&i| eval[i].len()).sum();
        print!("| {label} | {} | {braw} |", idx.len());
        for (_, sizes) in &per_codec_sizes {
            let bcomp: usize = idx.iter().map(|&i| sizes[i]).sum();
            print!(" {:.2}x ({:.0} B/resp) |", braw as f64 / bcomp as f64, bcomp as f64 / idx.len().max(1) as f64);
        }
        println!();
    }
}

// ---------------------------------------------------------------------------
// artifacts: external reference-decoder validation inputs
// ---------------------------------------------------------------------------

fn artifacts(path: &str, dir: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let g = load(path);
    let budget = QueryBudget::unlimited();
    let chunks = query_json_chunks_with_budget(&g, ALL_ROWS, &budget).unwrap();
    let raw: Vec<u8> = chunks.iter().flat_map(|c| c.bytes()).collect();

    let write = |name: &str, data: &[u8]| {
        let p = format!("{dir}/{name}");
        std::fs::File::create(&p).unwrap().write_all(data).unwrap();
        eprintln!("wrote {p} ({} B)", data.len());
    };
    write("big.json", &raw);

    let gz: Vec<u8> = compress_chunks(&chunks, &Codec::Gzip { level: 6 }, Mode::Parallel).unwrap().concat();
    assert_eq!(decode_gzip_concat(&gz).unwrap(), raw, "in-process gzip roundtrip");
    write("big.multi.gz", &gz);

    let zst: Vec<u8> = compress_chunks(&chunks, &Codec::Zstd { level: 3 }, Mode::Parallel).unwrap().concat();
    assert_eq!(decode_zstd_concat(&zst).unwrap(), raw, "in-process zstd roundtrip");
    write("big.multi.zst", &zst);

    let pigz: Vec<u8> = gzip_single_member(&chunks, 6, Mode::Parallel).unwrap().concat();
    assert_eq!(decode_gzip_concat(&pigz).unwrap(), raw, "in-process single-member gzip roundtrip");
    write("big.pigz.gz", &pigz);

    // Small responses + dictionary.
    let responses = point_responses(&g, 2400);
    let (train, eval) = responses.split_at(responses.len() / 2);
    let dict = train_dictionary(train, 64 * 1024).unwrap();
    let codec = Codec::ZstdDict(Arc::new(PreparedDict::new(dict.clone(), 3)));
    let frames: Vec<u8> = compress_chunks(eval, &codec, Mode::Parallel).unwrap().concat();
    let small_raw: Vec<u8> = eval.concat();
    assert_eq!(decode_zstd_concat_with_dict(&frames, &dict).unwrap(), small_raw, "in-process dict roundtrip");
    write("small.json", &small_raw);
    write("small.dict", &dict);
    write("small.dict.zst", &frames);
    eprintln!("validate externally, e.g.:");
    eprintln!("  gzip -dc {dir}/big.multi.gz | cmp - {dir}/big.json");
    eprintln!("  python3 -c 'import gzip;import sys;sys.stdout.buffer.write(gzip.decompress(open(\"{dir}/big.multi.gz\",\"rb\").read()))' | cmp - {dir}/big.json");
    eprintln!("  zstd -dc {dir}/big.multi.zst | cmp - {dir}/big.json");
    eprintln!("  zstd -dc -D {dir}/small.dict {dir}/small.dict.zst | cmp - {dir}/small.json");
}
