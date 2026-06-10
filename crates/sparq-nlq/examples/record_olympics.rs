//! Re-records the olympics replay fixture (`tests/fixtures/olympics_replay.json`).
//!
//! The "LLM" here is scripted: the completions are hand-written, realistic answers a
//! competent model would give for the grounding prompt this crate builds (they use
//! only vocabulary visible in the schema summary; one of them — Q8 — is deliberately
//! malformed on the first round so the fixture exercises the repair path). The
//! script is wrapped in [`RecordingLlm`], driven through the REAL `Nlq::ask` loop
//! against the REAL dataset, and the recorded (prompt, completion) pairs are written
//! out. Run this again whenever the prompt template, the default `NlqConfig`, or the
//! dataset changes:
//!
//! ```sh
//! cargo run -p sparq-nlq --example record_olympics --release
//! ```
//!
//! Needs `bench/qlever-olympics/olympics.nt` (downloaded benchmark fixture; override
//! the path with `SPARQ_OLYMPICS_NT`).

use std::rc::Rc;

use sparq_core::Graph;
use sparq_nlq::{Llm, Nlq, RecordingLlm};

/// The eval set: (question, scripted first completion, scripted repair completion).
/// Kept in sync by hand with `tests/olympics_eval.rs` (which hardcodes the same
/// questions + expectations).
pub fn script() -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    vec![
        (
            "How many athletes are on each team?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             SELECT ?team (COUNT(?athlete) AS ?count)\n\
             WHERE {\n  ?athlete rdf:type foaf:Person ;\n           dbo:team ?team .\n}\n\
             GROUP BY ?team\n\
             ORDER BY DESC(?count)\n\
             ```",
            None,
        ),
        (
            "How many athletes are in the dataset?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
             SELECT (COUNT(?athlete) AS ?count)\n\
             WHERE { ?athlete rdf:type foaf:Person }\n\
             ```",
            None,
        ),
        (
            "List the year and host city of every Olympic games.",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             PREFIX dbp: <http://dbpedia.org/property/>\n\
             SELECT ?games ?year ?city\n\
             WHERE {\n  ?games rdf:type dbo:Olympics ;\n         dbp:year ?year ;\n         dbp:location ?city .\n}\n\
             ORDER BY ?year\n\
             ```",
            None,
        ),
        (
            // The participation records are untyped — the LLM has to read them off
            // the characteristic-set section of the summary ({ns6:athlete, ns6:event,
            // ns6:games, ns6:medal}) and join medals to their labels.
            "How many medals of each type were awarded?",
            "```sparql\n\
             PREFIX ns6: <http://wallscope.co.uk/ontology/olympics/>\n\
             PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
             SELECT ?medal (COUNT(?participation) AS ?count)\n\
             WHERE {\n  ?participation ns6:medal ?m .\n  ?m rdfs:label ?medal .\n}\n\
             GROUP BY ?medal\n\
             ORDER BY DESC(?count)\n\
             ```",
            None,
        ),
        (
            "What is the average height of the athletes?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             SELECT (AVG(?height) AS ?avgHeight)\n\
             WHERE { ?athlete rdf:type foaf:Person ; dbo:height ?height }\n\
             ```",
            None,
        ),
        (
            "Which team has the most athletes?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             SELECT ?team (COUNT(?athlete) AS ?count)\n\
             WHERE { ?athlete rdf:type foaf:Person ; dbo:team ?team }\n\
             GROUP BY ?team\n\
             ORDER BY DESC(?count)\n\
             LIMIT 1\n\
             ```",
            None,
        ),
        (
            "List all sports with their labels.",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             SELECT ?sport ?label\n\
             WHERE { ?sport rdf:type dbo:Sport ; rdfs:label ?label }\n\
             ORDER BY ?label\n\
             ```",
            None,
        ),
        (
            // THE REPAIR CASE: the first completion drops the closing parenthesis of
            // the aggregate alias (a classic LLM slip) — spargebra rejects it, the
            // loop sends the parse error back, the second completion is fixed.
            "How many events does each sport have?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             SELECT ?sport (COUNT(?event) AS ?count\n\
             WHERE { ?event rdf:type dbo:SportsEvent ; rdfs:subClassOf ?sport }\n\
             GROUP BY ?sport\n\
             ```",
            Some(
                "```sparql\n\
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                 PREFIX dbo: <http://dbpedia.org/ontology/>\n\
                 SELECT ?sport (COUNT(?event) AS ?count)\n\
                 WHERE { ?event rdf:type dbo:SportsEvent ; rdfs:subClassOf ?sport }\n\
                 GROUP BY ?sport\n\
                 ORDER BY DESC(?count)\n\
                 ```",
            ),
        ),
        (
            "Are there any athletes taller than 200 centimetres?",
            "```sparql\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
             PREFIX dbo: <http://dbpedia.org/ontology/>\n\
             ASK {\n  ?athlete rdf:type foaf:Person ;\n           dbo:height ?height .\n  FILTER(?height > 200)\n}\n\
             ```",
            None,
        ),
    ]
}

/// Answers by matching the question line in the prompt; serves the repair completion
/// when the prompt carries the loop's failure marker.
struct ScriptedLlm {
    script: Vec<(&'static str, &'static str, Option<&'static str>)>,
}

impl Llm for ScriptedLlm {
    fn complete(&self, prompt: &str) -> Result<String, String> {
        let repair = prompt.contains("Your previous query failed");
        for (q, first, fix) in &self.script {
            if prompt.contains(&format!("Question: {q}")) {
                return if repair {
                    fix.map(|f| f.to_string())
                        .ok_or_else(|| format!("no scripted repair for {q:?}"))
                } else {
                    Ok(first.to_string())
                };
            }
        }
        Err(format!(
            "no scripted answer for prompt ending {:?}",
            prompt.lines().last()
        ))
    }
}

fn main() {
    let path = std::env::var("SPARQ_OLYMPICS_NT").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../bench/qlever-olympics/olympics.nt"
        )
        .into()
    });
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("olympics.nt not found at {path} (set SPARQ_OLYMPICS_NT) — nothing recorded");
        return;
    };
    let graph = Graph::load_str(&text, "ntriples").expect("parse olympics.nt");
    drop(text);
    println!("loaded {} triples", graph.len());

    let recorder = Rc::new(RecordingLlm::new(ScriptedLlm { script: script() }));
    let nlq = Nlq::new(&graph, Box::new(Rc::clone(&recorder)));

    for (question, _, _) in script() {
        let t0 = std::time::Instant::now();
        let a = nlq
            .ask(question)
            .unwrap_or_else(|e| panic!("{question:?} failed: {e}\n{:#?}", e.transcript));
        println!(
            "ok ({} rows, {} repair(s), {:.1} ms): {question}",
            a.result.len(),
            a.repairs,
            t0.elapsed().as_secs_f64() * 1e3
        );
        // Show the head of the result so a human can sanity-check the recording.
        for row in a.result.rows.iter().take(3) {
            let cells: Vec<String> = row
                .iter()
                .map(|c| {
                    c.as_ref()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "∅".into())
                })
                .collect();
            println!("    {}", cells.join(" | "));
        }
    }

    let out = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/olympics_replay.json"
    );
    std::fs::create_dir_all(std::path::Path::new(out).parent().unwrap()).unwrap();
    recorder.save(out).expect("write fixture");
    println!("wrote {} exchanges to {out}", recorder.exchanges().len());
}
