//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `fromRdf` lane runner (split out of
//! the former monolithic `tests/jsonld_suite.rs`; behaviour byte-identical).
//!
//! **fromRdf** (RDF → JSON-LD): each `jld:FromRDFTest` input `.nq` is driven
//! through the REAL write path — `sparq_engine::serialize::graph_to_jsonld` (the
//! `serialize-rdf` feature) in both **expanded** and prefix-`@context` ("compacted")
//! forms — then that JSON-LD is RE-PARSED through `oxjsonld` and compared, by the
//! same dataset isomorphism, against the input dataset. The load-bearing invariant
//! is the round-trip `reparse(write(D)) ≡ D`, not byte equality.

use super::common::*;
use sparq_core::Graph;
use std::path::Path;

/// Run the fromRdf category through the native writer + a re-parse round-trip.
pub fn run_fromrdf(root: &Path) -> Score {
    use sparq_engine::serialize::{graph_to_jsonld, JsonLdForm};
    let mut s = Score::default();
    let entries = match read_manifest(root, "fromRdf") {
        Ok(e) => e,
        Err(why) => {
            s.fail("fromRdf-manifest", why);
            return s;
        }
    };
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        // fromRdf inputs are N-Quads; sparq writes JSON-LD; we re-parse and
        // require RDF-dataset equivalence (the round-trip invariant).
        let input_path = root.join(&e.input);
        let nq_text = match std::fs::read_to_string(&input_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read input: {why}"));
                continue;
            }
        };
        // The expected dataset is exactly the input N-Quads (canonicalized).
        let want = match nquads_to_canonical_dataset(&nq_text) {
            Ok(ds) => ds,
            Err(why) => {
                s.fail(&e.id, format!("parse input nq: {why}"));
                continue;
            }
        };
        // Build the sparq Graph (preserving named graphs) and write JSON-LD.
        let graph = match Graph::load_dataset(&nq_text, "nquads") {
            Ok(g) => g,
            Err(why) => {
                s.fail(&e.id, format!("load nq into graph: {why}"));
                continue;
            }
        };
        // Both shipped output forms must round-trip; require BOTH to count a
        // pass (answer-safety: every form sparq emits must be lossless).
        let mut ok = true;
        let mut detail = String::new();
        for form in [JsonLdForm::Expanded, JsonLdForm::Compacted] {
            let doc = graph_to_jsonld(&graph, form);
            // Must be valid JSON.
            if let Err(why) = serde_json::from_str::<serde_json::Value>(&doc) {
                ok = false;
                detail = format!("{form:?} invalid JSON: {why}");
                break;
            }
            // The writer emits absolute IRIs (no base needed); use the suite
            // base only as a fallback for any relative form.
            let got = match jsonld_to_canonical_dataset(&doc, SUITE_BASE) {
                Ok(ds) => ds,
                Err(why) => {
                    ok = false;
                    detail = format!("{form:?} re-parse: {why}");
                    break;
                }
            };
            if got != want {
                ok = false;
                detail = format!(
                    "{form:?} round-trip mismatch ({} vs {} quads)",
                    got.len(),
                    want.len()
                );
                break;
            }
        }
        if ok {
            s.pass();
        } else {
            s.fail(&e.id, detail);
        }
    }
    s
}
