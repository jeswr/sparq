// [GPT-5.6] sq-no6iy — content-level RSP window differential replay driver.
use std::env;
use std::fs;
use std::str::FromStr;
use std::time::Instant;

use oxrdf::{NamedNode, Term};
use sparq_rsp::{ContinuousMultiQuery, WindowResult};

const QUERY: &str = r#"REGISTER STREAM <http://ex/out> AS
SELECT ?st ?state ?v WHERE {
  WINDOW <http://ex/wo> { ?st <http://ex/value> ?v }
  WINDOW <http://ex/wm> { ?st <http://ex/state> ?state }
}
FROM NAMED WINDOW <http://ex/wo> ON <http://ex/obs> RANGE 10 STEP 10
FROM NAMED WINDOW <http://ex/wm> ON <http://ex/meta> RANGE 10 STEP 10"#;

fn parse_replay(path: &str) -> Result<Vec<(u64, NamedNode, [Term; 3])>, String> {
    let input = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut events = Vec::new();
    for (index, line) in input.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("{path}:{}: expected five TSV fields", index + 1));
        }
        let ts = fields[0]
            .parse()
            .map_err(|e| format!("{path}:{}: invalid timestamp: {e}", index + 1))?;
        let stream = NamedNode::from_str(fields[1])
            .map_err(|e| format!("{path}:{}: invalid stream: {e}", index + 1))?;
        let term = |field: usize| {
            Term::from_str(fields[field])
                .map_err(|e| format!("{path}:{}: invalid term: {e}", index + 1))
        };
        events.push((ts, stream, [term(2)?, term(3)?, term(4)?]));
    }
    Ok(events)
}

fn canonical_rows(result: WindowResult) -> String {
    let mut rows: Vec<String> = result
        .rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|term| term.map_or_else(|| "UNBOUND".into(), |value| value.to_string()))
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();
    rows.sort_unstable();
    rows.join(" ; ")
}

fn run(path: &str) -> Result<(), String> {
    let events = parse_replay(path)?;
    let mut query = ContinuousMultiQuery::register(QUERY)?;
    let mut emitted = Vec::new();
    for (ts, stream, triple) in events {
        let started = Instant::now();
        query.push(&stream, triple, ts, |result| {
            emitted.push((
                result.end,
                canonical_rows(result),
                started.elapsed().as_nanos(),
            ));
        })?;
    }
    let started = Instant::now();
    query.flush(|result| {
        emitted.push((
            result.end,
            canonical_rows(result),
            started.elapsed().as_nanos(),
        ));
    })?;
    println!("report_ts\tcanonical_multiset\temit_latency_ns");
    for (report_ts, rows, latency) in emitted {
        println!("{report_ts}\t{rows}\t{latency}");
    }
    Ok(())
}

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/srbench.ts.tsv".to_owned());
    if let Err(error) = run(&path) {
        eprintln!("rsp-window-differential: {error}");
        std::process::exit(1);
    }
}
