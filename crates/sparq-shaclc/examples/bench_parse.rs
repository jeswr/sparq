//! [GPT-5.6] Parse-throughput benchmark over sparq-shaclc's committed corpus.

use sparq_shaclc::{parse, Profile, DEFAULT_BASE};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Clone)]
struct Fixture {
    text: String,
}

fn collect_shaclc(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("read {} entry: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_shaclc(&path, out)?;
        } else if path.extension().and_then(|v| v.to_str()) == Some("shaclc") {
            out.push(path);
        }
    }
    Ok(())
}

fn load(paths: &[PathBuf]) -> Result<Vec<Fixture>, String> {
    paths
        .iter()
        .map(|path| {
            let text = std::fs::read_to_string(path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            Ok(Fixture { text })
        })
        .collect()
}

fn parse_positive(value: Option<&String>, name: &str, default: usize) -> Result<usize, String> {
    let Some(value) = value else {
        return Ok(default);
    };
    value
        .parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("{name} must be a positive integer"))
}

fn measure(
    label: &str,
    profile: Profile,
    fixtures: &[Fixture],
    passes: usize,
    samples: usize,
) -> Result<(), String> {
    let bytes_per_pass: usize = fixtures.iter().map(|f| f.text.len()).sum();
    let mut expected_triples = None;
    let mut best_micros = u128::MAX;

    for _ in 0..samples {
        let started = Instant::now();
        let mut triples_this_sample = 0usize;
        for _ in 0..passes {
            let mut triples_this_pass = 0usize;
            for fixture in fixtures {
                let (triples, outcome) = parse(&fixture.text, DEFAULT_BASE, profile)
                    .map_err(|e| format!("{label} fixture failed: {e}"))?;
                triples_this_pass += black_box(triples.len());
                black_box(outcome.base);
            }
            if let Some(expected) = expected_triples {
                if triples_this_pass != expected {
                    return Err(format!(
                        "{label} triple count drifted: {triples_this_pass} != {expected}"
                    ));
                }
            } else {
                expected_triples = Some(triples_this_pass);
            }
            triples_this_sample += triples_this_pass;
        }
        black_box(triples_this_sample);
        best_micros = best_micros.min(started.elapsed().as_micros().max(1));
    }

    let documents = fixtures.len() * passes;
    let bytes = bytes_per_pass * passes;
    let triples = expected_triples.unwrap_or(0) * passes;
    let mib_per_s = bytes as f64 / (1024.0 * 1024.0) / (best_micros as f64 / 1_000_000.0);
    println!("{label}\t{documents}\t{bytes}\t{triples}\t{best_micros}\t{mib_per_s:.3}");
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let passes = parse_positive(args.first(), "passes", 20)?;
    let samples = parse_positive(args.get(1), "samples", 5)?;
    if args.len() > 2 {
        return Err("usage: bench_parse [passes] [samples]".into());
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut strict_paths = Vec::new();
    collect_shaclc(&root.join("valid"), &mut strict_paths)?;
    collect_shaclc(&root.join("rdf12"), &mut strict_paths)?;
    strict_paths.sort();

    let mut extended_paths = strict_paths.clone();
    collect_shaclc(&root.join("extended"), &mut extended_paths)?;
    extended_paths.sort();

    let strict = load(&strict_paths)?;
    let extended = load(&extended_paths)?;
    if strict.is_empty() || extended.len() <= strict.len() {
        return Err("fixture corpus is missing strict or extended documents".into());
    }

    println!("profile\tdocuments\tbytes\ttriples\tbest_micros\tmib_per_s");
    measure("strict", Profile::Strict, &strict, passes, samples)?;
    measure("extended", Profile::Extended, &extended, passes, samples)?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("bench_parse: {error}");
        std::process::exit(2);
    }
}
