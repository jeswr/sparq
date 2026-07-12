//! [GPT-5.6] sq-139od — drive committed SERVICE fixtures through sparq and Comunica.
//!
//! The endpoint servers are `sparq_conformance::service_loopback::LoopbackEndpoint`
//! instances. They remain alive in this process while sparq evaluates each query and
//! while the child Comunica process evaluates the identical query. Standard output is
//! one raw JSON document; `compare.py` owns canonical multiset comparison and envelope
//! generation.
#![forbid(unsafe_code)]

use oxrdf::{NamedOrBlankNode, Term};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sparq_conformance::service_loopback::LoopbackEndpoint;
use sparq_core::Graph;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    suite_id: String,
    endpoints: Vec<EndpointFixture>,
    fixtures: Vec<QueryFixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EndpointFixture {
    id: String,
    data: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryFixture {
    id: String,
    query: PathBuf,
    expected_rows: usize,
}

fn usage() -> ! {
    eprintln!("usage: service-fed-loopback-driver --manifest FILE --comunica-runner FILE");
    std::process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf) {
    let mut manifest = None;
    let mut comunica_runner = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--comunica-runner" => comunica_runner = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }
    match (manifest, comunica_runner) {
        (Some(manifest), Some(comunica_runner)) => (manifest, comunica_runner),
        _ => usage(),
    }
}

fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read manifest {}: {error}", path.display()))?;
    let manifest: Manifest = serde_json::from_str(&text)
        .map_err(|error| format!("parse manifest {}: {error}", path.display()))?;
    if manifest.endpoints.len() < 2 {
        return Err("manifest must define at least two endpoints".into());
    }
    if manifest.fixtures.is_empty() {
        return Err("manifest must define at least one query fixture".into());
    }
    Ok(manifest)
}

fn load_graph(path: &Path) -> Result<Graph, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read endpoint data {}: {error}", path.display()))?;
    Graph::load_str(&text, "turtle")
        .map_err(|error| format!("parse endpoint data {}: {error}", path.display()))
}

fn subject_json(subject: &NamedOrBlankNode) -> Value {
    match subject {
        NamedOrBlankNode::NamedNode(node) => json!({"type": "uri", "value": node.as_str()}),
        NamedOrBlankNode::BlankNode(node) => json!({"type": "bnode", "value": node.as_str()}),
    }
}

fn term_json(term: &Term) -> Value {
    match term {
        Term::NamedNode(node) => json!({"type": "uri", "value": node.as_str()}),
        Term::BlankNode(node) => json!({"type": "bnode", "value": node.as_str()}),
        Term::Literal(literal) => {
            let mut value = Map::new();
            value.insert("type".into(), Value::String("literal".into()));
            value.insert("value".into(), Value::String(literal.value().into()));
            if let Some(language) = literal.language() {
                value.insert("xml:lang".into(), Value::String(language.into()));
            } else if literal.datatype().as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                value.insert(
                    "datatype".into(),
                    Value::String(literal.datatype().as_str().into()),
                );
            }
            Value::Object(value)
        }
        Term::Triple(triple) => json!({
            "type": "triple",
            "value": {
                "subject": subject_json(&triple.subject),
                "predicate": {"type": "uri", "value": triple.predicate.as_str()},
                "object": term_json(&triple.object),
            }
        }),
    }
}

fn sparq_bindings(result: &sparq_engine::QueryResult) -> Vec<Value> {
    result
        .rows
        .iter()
        .map(|row| {
            let mut binding = Map::new();
            for (variable, term) in result.vars.iter().zip(row) {
                if let Some(term) = term {
                    binding.insert(variable.as_str().into(), term_json(term));
                }
            }
            Value::Object(binding)
        })
        .collect()
}

fn run_comunica(runner: &Path, query: &str) -> Result<(String, Vec<Value>), String> {
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawn Comunica runner {}: {error}", runner.display()))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "Comunica stdin was not piped".to_string())?
        .write_all(query.as_bytes())
        .map_err(|error| format!("write Comunica query: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("poll Comunica runner: {error}"))?
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .map_err(|error| format!("reap timed-out Comunica runner: {error}"))?;
            return Err(format!(
                "Comunica runner timed out: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("collect Comunica output: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Comunica runner exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let document: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("parse Comunica output: {error}"))?;
    if document.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err("Comunica output did not contain ok=true".into());
    }
    let version = document
        .get("engine_version")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let bindings = document
        .get("bindings")
        .and_then(Value::as_array)
        .ok_or_else(|| "Comunica output did not contain a bindings array".to_string())?
        .clone();
    Ok((version, bindings))
}

fn run() -> Result<Value, String> {
    let (manifest_path, comunica_runner) = parse_args();
    let manifest = read_manifest(&manifest_path)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_string())?;

    let mut seen_endpoint_ids = BTreeSet::new();
    let mut endpoints = BTreeMap::new();
    for fixture in &manifest.endpoints {
        if !seen_endpoint_ids.insert(fixture.id.clone()) {
            return Err(format!("duplicate endpoint id {:?}", fixture.id));
        }
        let graph = load_graph(&base.join(&fixture.data))?;
        endpoints.insert(fixture.id.clone(), LoopbackEndpoint::serve(graph));
    }

    let driver_endpoint = endpoints
        .values()
        .next()
        .ok_or_else(|| "manifest has no endpoint".to_string())?;
    let local = Graph::new();
    let mut seen_fixture_ids = BTreeSet::new();
    let mut rows = Vec::new();
    let mut comunica_version = None;

    for fixture in &manifest.fixtures {
        if !seen_fixture_ids.insert(fixture.id.clone()) {
            return Err(format!("duplicate fixture id {:?}", fixture.id));
        }
        if fixture.expected_rows == 0 {
            return Err(format!(
                "fixture {:?} has a vacuous expected_rows value",
                fixture.id
            ));
        }
        let query_path = base.join(&fixture.query);
        let mut query = fs::read_to_string(&query_path)
            .map_err(|error| format!("read query {}: {error}", query_path.display()))?;
        for (id, endpoint) in &endpoints {
            query = query.replace(&format!("{{{{{id}}}}}"), &endpoint.sparql_url());
        }
        if query.contains("{{") || query.contains("}}") {
            return Err(format!(
                "query {} contains an unresolved endpoint placeholder",
                query_path.display()
            ));
        }

        let sparq = driver_endpoint
            .run_federated(&local, &query)
            .map_err(|error| format!("sparq fixture {:?}: {error}", fixture.id))?;
        let (version, comunica) = run_comunica(&comunica_runner, &query)
            .map_err(|error| format!("Comunica fixture {:?}: {error}", fixture.id))?;
        if let Some(previous) = &comunica_version {
            if previous != &version {
                return Err("Comunica version changed during one run".into());
            }
        } else {
            comunica_version = Some(version);
        }

        rows.push(json!({
            "id": fixture.id,
            "query_file": fixture.query,
            "expected_rows": fixture.expected_rows,
            "sparq": sparq_bindings(&sparq),
            "comunica": comunica,
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "suite_id": manifest.suite_id,
        "bead": "sq-139od",
        "endpoint_count": endpoints.len(),
        "driver": "sparq-conformance/service-loopback",
        "comunica_version": comunica_version.unwrap_or_else(|| "unknown".into()),
        "fixtures": rows,
    }))
}

fn main() {
    match run() {
        Ok(document) => println!(
            "{}",
            serde_json::to_string_pretty(&document).expect("serialize driver output")
        ),
        Err(error) => {
            eprintln!("service-fed-loopback-driver: {error}");
            std::process::exit(1);
        }
    }
}
