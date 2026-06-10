//! The accuracy-gate scaffold from `research/genai-design.md` §4 (`sparq-nlq` row):
//! runs the full ground → generate → validate → execute → repair loop against the
//! real olympics dataset (1.78M triples) with a [`ReplayLlm`] serving the committed
//! fixture — fully offline, no network, deterministic.
//!
//! SKIPS (passes, with a note on stderr) when `bench/qlever-olympics/olympics.nt` is
//! absent — the dataset is a downloaded benchmark fixture, not checked in. Override
//! the path with `SPARQ_OLYMPICS_NT`. The replay fixture itself IS checked in
//! (`tests/fixtures/olympics_replay.json`); re-record it with
//! `cargo run -p sparq-nlq --example record_olympics --release` whenever the prompt
//! template, the default `NlqConfig`, or the dataset changes.
//!
//! What is gated here is the **engine-side** loop (prompt grounding, extraction,
//! validation, execution, repair plumbing) with hand-written, realistic completions —
//! per the design doc's benchmark protocol this is the LLM-excluded measurement;
//! live-model exec-accuracy (QALD-style, vs an ungrounded baseline) is a documented
//! follow-up and runs through the same harness with `--features live`.

use sparq_core::Graph;
use sparq_nlq::{Nlq, ReplayLlm};

fn data_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("SPARQ_OLYMPICS_NT") {
        return Some(p.into());
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/qlever-olympics/olympics.nt");
    p.exists().then_some(p)
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/olympics_replay.json"
);

/// (question, expected repairs, expected row count if exact, must-be-non-empty).
/// Kept in sync by hand with `examples/record_olympics.rs`.
const EVAL: &[(&str, usize, Option<usize>)] = &[
    // 1184 teams in the dataset — one row per team.
    ("How many athletes are on each team?", 0, Some(1184)),
    // A single aggregate row.
    ("How many athletes are in the dataset?", 0, Some(1)),
    // 51 games; 52 rows because the 1956 games had two host cities.
    (
        "List the year and host city of every Olympic games.",
        0,
        Some(52),
    ),
    // Gold / Silver / Bronze.
    ("How many medals of each type were awarded?", 0, Some(3)),
    ("What is the average height of the athletes?", 0, Some(1)),
    ("Which team has the most athletes?", 0, Some(1)),
    // 66 sports.
    ("List all sports with their labels.", 0, Some(66)),
    // THE REPAIR CASE: first completion is malformed, second parses — 66 sports
    // again (every event is rdfs:subClassOf exactly one sport).
    ("How many events does each sport have?", 1, Some(66)),
    // ASK → unit-row encoding: zero vars, one empty row iff true.
    (
        "Are there any athletes taller than 200 centimetres?",
        0,
        Some(1),
    ),
];

#[test]
fn olympics_replay_eval() {
    let Some(path) = data_path() else {
        eprintln!("skipping: olympics.nt not present (downloaded benchmark fixture)");
        return;
    };
    let text = std::fs::read_to_string(&path).expect("read olympics.nt");
    let graph = Graph::load_str(&text, "ntriples").expect("parse olympics.nt");
    drop(text);
    assert!(
        graph.len() > 1_700_000,
        "expected the 1.78M-triple olympics dataset"
    );

    let replay = ReplayLlm::from_file(FIXTURE).expect("committed replay fixture loads");
    assert_eq!(
        replay.len(),
        EVAL.len() + 1,
        "9 questions + 1 repair round recorded"
    );
    let nlq = Nlq::new(&graph, Box::new(replay));

    let mut latencies_ms: Vec<f64> = Vec::new();
    let mut total_repairs = 0;
    for (question, expected_repairs, expected_rows) in EVAL {
        let t0 = std::time::Instant::now();
        let answer = nlq
            .ask(question)
            .unwrap_or_else(|e| panic!("{question:?} failed: {e}"));
        latencies_ms.push(t0.elapsed().as_secs_f64() * 1e3);

        // Parse + execution success is implied by Ok; gate the result shape too.
        assert!(
            !answer.result.is_empty(),
            "{question:?} returned an empty result"
        );
        if let Some(rows) = expected_rows {
            assert_eq!(
                answer.result.len(),
                *rows,
                "{question:?} row count (query:\n{})",
                answer.sparql
            );
        }
        assert_eq!(
            answer.repairs, *expected_repairs,
            "{question:?} repair rounds"
        );
        assert_eq!(
            answer.transcript.len(),
            expected_repairs + 1,
            "{question:?} transcript length"
        );
        total_repairs += answer.repairs;
    }

    // Exactly one question needed the repair path — the fixture exercises it, the
    // other eight prove the happy path.
    assert_eq!(
        total_repairs, 1,
        "exactly one repair round across the eval set"
    );

    // Result sanity on specific answers.
    let count_answer = nlq.ask("How many athletes are in the dataset?").unwrap();
    let n = count_answer.rows_first_cell();
    assert!(n.contains("134730"), "athlete count, got {n}");
    let top_team = nlq.ask("Which team has the most athletes?").unwrap();
    assert!(
        top_team.rows_first_cell().contains("UnitedStates"),
        "largest team"
    );
    let ask = nlq
        .ask("Are there any athletes taller than 200 centimetres?")
        .unwrap();
    assert!(
        ask.result.vars.is_empty(),
        "ASK answers carry zero variables"
    );

    // The performance gate (design doc §4: LLM-excluded store latency p50 < 50 ms)
    // is asserted in release builds only — the test profile is unoptimised, so debug
    // runs just report the numbers.
    latencies_ms.sort_by(|a, b| a.total_cmp(b));
    let p50 = latencies_ms[latencies_ms.len() / 2];
    eprintln!(
        "LLM-excluded ask() latency over {} questions: p50 {p50:.1} ms, max {:.1} ms",
        latencies_ms.len(),
        latencies_ms.last().unwrap()
    );
    if !cfg!(debug_assertions) {
        assert!(
            p50 < 50.0,
            "LLM-excluded p50 gate (50 ms), measured {p50:.1} ms"
        );
    }
}

/// Tiny helper: first cell of the first row, stringified.
trait FirstCell {
    fn rows_first_cell(&self) -> String;
}
impl FirstCell for sparq_nlq::Answer {
    fn rows_first_cell(&self) -> String {
        self.result.rows[0][0]
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_default()
    }
}
