//! W3C RDF Dataset Canonicalization (RDFC-1.0) test suite, manifest-driven
//! over the vendored snapshot (`tests/rdf-canon-testdata/`, see
//! PROVENANCE.md). This validates the `rdf-canon` dependency THROUGH
//! sparq-zk's own bridge (`canon::canonicalize_triples` round-trips oxrdf 0.3
//! → text → oxrdf 0.2 → canonical text), so a bridge bug fails the suite
//! exactly like a canonicalization bug would.
//!
//! The manifest itself is parsed with sparq (dogfooding the engine).
//!
//! Test types:
//! - `rdfc:RDFC10EvalTest`: canonical N-Quads must match byte-for-byte.
//! - `rdfc:RDFC10MapTest`: the issued identifier map must match.
//! - `rdfc:RDFC10NegativeEvalTest`: canonicalization must FAIL (poison
//!   graphs; the HNDQ call-limit guard — plan §8's ingest fail-closed rule).

use oxrdf::Triple;
use sparq_core::Graph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn testdata() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rdf-canon-testdata")
}

struct Entry {
    name: String,
    kind: String,
    action: String,
    result: Option<String>,
    /// `rdfc:hashAlgorithm` (test075 is parameterized over SHA-384).
    hash_algorithm: Option<String>,
}

/// Reads the manifest with sparq itself.
fn manifest_entries() -> Vec<Entry> {
    let ttl = std::fs::read_to_string(testdata().join("manifest.ttl")).unwrap();
    // The manifest uses relative IRIs; anchor them at the canonical W3C base
    // (the vendored file is untouched — the base is prepended at read time).
    let ttl = format!("@base <https://w3c.github.io/rdf-canon/tests/> .\n{ttl}");
    let g = Graph::load_str(&ttl, "turtle").unwrap();
    let q = r#"
        PREFIX mf: <http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfc: <https://w3c.github.io/rdf-canon/tests/vocab#>
        SELECT ?test ?type ?action ?result ?hash WHERE {
            ?test a ?type ; mf:action ?action .
            OPTIONAL { ?test mf:result ?result }
            OPTIONAL { ?test rdfc:hashAlgorithm ?hash }
        } ORDER BY ?test
    "#;
    let res = sparq_engine::query(&g, q).unwrap();
    let mut out = Vec::new();
    for row in &res.rows {
        let iri = |i: usize| -> Option<String> {
            match &row[i] {
                Some(oxrdf::Term::NamedNode(n)) => Some(n.as_str().to_string()),
                _ => None,
            }
        };
        let hash_algorithm = match &row[4] {
            Some(oxrdf::Term::Literal(l)) => Some(l.value().to_string()),
            _ => None,
        };
        out.push(Entry {
            name: iri(0).unwrap(),
            kind: iri(1).unwrap(),
            action: iri(2).unwrap(),
            result: iri(3),
            hash_algorithm,
        });
    }
    assert_eq!(out.len(), 86, "expected the full 86-entry manifest");
    out
}

/// Loads a test input `.nq` as oxrdf 0.3 triples (all suite inputs are
/// default-graph datasets... except some use named graphs — those quads keep
/// their graph component through the rdf-canon API, so we canonicalize at
/// the quad level here).
fn load_quads(path: &Path) -> Vec<oxrdf::Quad> {
    let bytes = std::fs::read(path).unwrap();
    oxttl::NQuadsParser::new()
        .for_slice(&bytes)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

/// Bridges oxrdf 0.3 quads to canonical N-Quads via the same text seam the
/// crate uses, calling rdf-canon directly (the suite covers full datasets;
/// `canon::canonicalize_triples` is the single-graph special case used by
/// the commitment pipeline and is exercised by `default_graph_tests_via_crate_api`).
/// N-Quads serialization of oxrdf 0.3 quads (explicit, because `GraphName`'s
/// `Display` renders the default graph as the word `DEFAULT`).
fn quads_doc(quads: &[oxrdf::Quad]) -> String {
    let mut doc = String::new();
    for q in quads {
        match &q.graph_name {
            oxrdf::GraphName::DefaultGraph => {
                doc.push_str(&format!("{} {} {} .\n", q.subject, q.predicate, q.object));
            }
            g => {
                doc.push_str(&format!("{} {} {} {} .\n", q.subject, q.predicate, q.object, g));
            }
        }
    }
    doc
}

fn canonicalize_quads_bridge(quads: &[oxrdf::Quad], sha384: bool) -> Result<String, String> {
    let doc = quads_doc(quads);
    let quads02: Vec<_> = oxttl01::NQuadsParser::new()
        .for_reader(doc.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let opts = rdf_canon::CanonicalizationOptions::default();
    if sha384 {
        rdf_canon::canonicalize_quads_with::<sha2::Sha384>(&quads02, &opts).map_err(|e| e.to_string())
    } else {
        rdf_canon::canonicalize_quads(&quads02).map_err(|e| e.to_string())
    }
}

fn issue_quads_bridge(quads: &[oxrdf::Quad], sha384: bool) -> Result<HashMap<String, String>, String> {
    let doc = quads_doc(quads);
    let quads02: Vec<_> = oxttl01::NQuadsParser::new()
        .for_reader(doc.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let opts = rdf_canon::CanonicalizationOptions::default();
    let m = if sha384 {
        rdf_canon::issue_quads_with::<sha2::Sha384>(&quads02, &opts).map_err(|e| e.to_string())?
    } else {
        rdf_canon::issue_quads(&quads02).map_err(|e| e.to_string())?
    };
    Ok(m.into_iter().collect())
}

fn local_path(iri: &str) -> PathBuf {
    // Manifest IRIs are relative references resolved against the manifest
    // location; sparq resolves them to file-less relative IRIs or keeps them
    // relative — extract the trailing `rdfc10/...` component.
    let tail = iri.rsplit_once("rdfc10/").map(|(_, t)| t).unwrap_or(iri);
    testdata().join("rdfc10").join(tail)
}

#[test]
fn w3c_rdf_canon_suite() {
    let entries = manifest_entries();
    let (mut eval, mut map, mut neg) = (0, 0, 0);
    let mut failures: Vec<String> = Vec::new();

    for e in &entries {
        let input = load_quads(&local_path(&e.action));
        match e.kind.as_str() {
            t if t.ends_with("RDFC10EvalTest") => {
                eval += 1;
                let expected =
                    std::fs::read_to_string(local_path(e.result.as_ref().unwrap())).unwrap();
                match canonicalize_quads_bridge(&input, e.hash_algorithm.as_deref() == Some("SHA384")) {
                    Ok(got) if got == expected => {}
                    Ok(got) => failures.push(format!(
                        "{}: canonical output mismatch\n  got:      {:?}\n  expected: {:?}",
                        e.name, got, expected
                    )),
                    Err(err) => failures.push(format!("{}: failed: {err}", e.name)),
                }
            }
            t if t.ends_with("RDFC10MapTest") => {
                map += 1;
                let expected_json =
                    std::fs::read_to_string(local_path(e.result.as_ref().unwrap())).unwrap();
                let expected: HashMap<String, String> =
                    serde_json::from_str(&expected_json).unwrap();
                match issue_quads_bridge(&input, e.hash_algorithm.as_deref() == Some("SHA384")) {
                    Ok(got) if got == expected => {}
                    Ok(got) => failures.push(format!(
                        "{}: issued map mismatch\n  got:      {:?}\n  expected: {:?}",
                        e.name, got, expected
                    )),
                    Err(err) => failures.push(format!("{}: failed: {err}", e.name)),
                }
            }
            t if t.ends_with("RDFC10NegativeEvalTest") => {
                neg += 1;
                if let Ok(got) = canonicalize_quads_bridge(&input, e.hash_algorithm.as_deref() == Some("SHA384")) {
                    failures.push(format!(
                        "{}: poison graph canonicalized (must fail closed): {} quads -> {} bytes",
                        e.name,
                        input.len(),
                        got.len()
                    ));
                }
            }
            other => failures.push(format!("{}: unknown test type {other}", e.name)),
        }
    }

    assert_eq!((eval, map, neg), (64, 21, 1), "suite composition drifted");
    assert!(
        failures.is_empty(),
        "{} of {} W3C rdf-canon tests failed:\n{}",
        failures.len(),
        entries.len(),
        failures.join("\n")
    );
}

/// The crate's own single-graph API (`canon::canonicalize_triples`, what the
/// commitment pipeline calls) must agree with the suite on every
/// default-graph-only eval test.
#[test]
fn default_graph_tests_via_crate_api() {
    let entries = manifest_entries();
    let mut covered = 0;
    for e in &entries {
        if !e.kind.ends_with("RDFC10EvalTest") || e.hash_algorithm.is_some() {
            continue; // the commitment pipeline canonicalizes with the default SHA-256
        }
        let quads = load_quads(&local_path(&e.action));
        if !quads.iter().all(|q| q.graph_name == oxrdf::GraphName::DefaultGraph) {
            continue; // named-graph datasets are not single-graph content
        }
        let triples: Vec<Triple> = quads
            .iter()
            .map(|q| Triple::new(q.subject.clone(), q.predicate.clone(), q.object.clone()))
            .collect();
        let expected = std::fs::read_to_string(local_path(e.result.as_ref().unwrap())).unwrap();
        let got = sparq_zk::canon::canonicalize_triples(&triples)
            .unwrap_or_else(|err| panic!("{}: {err}", e.name));
        let got_doc: String = got.lines.iter().map(|l| format!("{l}\n")).collect();
        assert_eq!(got_doc, expected, "{} via crate API", e.name);
        covered += 1;
    }
    assert!(covered >= 40, "expected most eval tests to be default-graph-only, got {covered}");
}
