//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `flatten` lane runner (split out of
//! the former monolithic `tests/jsonld_suite.rs`; behaviour byte-identical). The
//! `flatten` category keeps the RDF-equivalence oracle over the shipping writer
//! (`graph_to_jsonld(JsonLdForm::Flattened)`); the native flatten algorithm is a
//! separate deferred bead.

use super::common::*;
use sparq_core::Graph;
use std::path::Path;

/// [OPUS-4.8] sq-oy1f — run a W3C JSON-LD `flatten` category against the
/// ALREADY-SHIPPING native writer (`graph_to_jsonld(graph, form)`, the
/// `serialize-rdf` feature). `form` = `JsonLdForm::Flattened` for `flatten`.
///
/// ## Pipeline (the REAL writer path)
///
/// 1. Parse the case `input` (`.jsonld`) → RDF via the real oxjsonld ingest path
///    (`parse_jsonld_dataset`).
/// 2. Re-emit that dataset as N-Quads and load it into a sparq [`Graph`]
///    (preserving named graphs) — the writer takes a `Graph`, not a JSON-LD doc,
///    because sparq's expand/flatten OUTPUT is a projection of RDF (exactly the
///    bridge `compact::run_compact` / `frame::run_frame` use).
/// 3. Run `graph_to_jsonld(&graph, form)` — the shipping writer, NOT a stub.
/// 4. **Invariant (normative answer-equivalence):** re-parse BOTH sparq's output
///    AND the suite's NORMATIVE expected document (`*-out.jsonld`) to canonical
///    [`Dataset`]s and require they are equal:
///    `reparse(write(D, form)) ≡ reparse(expected)`. Flattening is a JSON-LD normal
///    form (it merges nodes), so the oracle anchors on the W3C-expected document,
///    NOT the input — the same posture as the frame lane. This is
///    envelope-insensitive and value-faithful while NOT requiring sparq's JSON
///    layout to match byte-for-byte.
///
/// ## Honest SKIP buckets (recorded, not passed, not failed) — see the `flatten` floor
pub fn run_expand_or_flatten(
    root: &Path,
    cat: &str,
    form: sparq_engine::serialize::JsonLdForm,
) -> Score {
    use sparq_engine::serialize::graph_to_jsonld;
    let mut s = Score::default();
    let entries = match read_manifest(root, cat) {
        Ok(e) => e,
        Err(why) => {
            s.fail(&format!("{cat}-manifest"), why);
            return s;
        }
    };
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        // NegativeEvaluationTest: sparq's writer is TOTAL and does not raise the
        // spec's expansion/flattening errors, so a faithful 1.1 writer cannot
        // "pass" by rejecting — SKIP (honest), never a counted pass.
        if e.is_negative {
            s.skip();
            continue;
        }
        // JSON-LD-1.0-only positives (processing-mode shape differences a 1.1
        // writer is not obliged to reproduce): SKIP.
        if e.spec_version.as_deref() == Some("json-ld-1.0")
            || e.processing_mode.as_deref() == Some("json-ld-1.0")
        {
            s.skip();
            continue;
        }
        let Some(expect_rel) = &e.expect else {
            s.skip();
            continue;
        };
        // Remote input (no network) — guard (the toRdf lane already gates ingest).
        if e.input.starts_with("http://") || e.input.starts_with("https://") {
            s.skip();
            continue;
        }

        // 1. Parse the input JSON-LD → RDF (the source dataset). An input the
        //    real oxjsonld path rejects, or `option`-driven cases the writer does
        //    not apply (e.g. `expandContext`), produce RDF that legitimately
        //    differs from `expected`; those that fail to parse are SKIPPED (out of
        //    the gated surface), those that parse but diverge are honest FAILs.
        let input_path = root.join(&e.input);
        let in_text = match std::fs::read_to_string(&input_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read input: {why}"));
                continue;
            }
        };
        let base = doc_base(e);
        let input_ds = match parse_jsonld_dataset(&in_text, &base) {
            Ok(ds) => ds,
            Err(_) => {
                s.skip();
                continue;
            }
        };
        // No triples → nothing to project; the round-trip is vacuous. SKIP.
        if input_ds.is_empty() {
            s.skip();
            continue;
        }

        // 2. Load the input RDF into a sparq Graph (named graphs preserved) and
        //    run the REAL shipping writer in the requested form.
        let nq = dataset_to_nquads(&input_ds);
        let graph = match Graph::load_dataset(&nq, "nquads") {
            Ok(g) => g,
            Err(why) => {
                s.fail(&e.id, format!("load input rdf into graph: {why}"));
                continue;
            }
        };
        let out_doc = graph_to_jsonld(&graph, form);

        // 3. Re-parse sparq's output and the suite's NORMATIVE expected document,
        //    require RDF-dataset equivalence (the normative answer-equivalence
        //    invariant).
        let got = match jsonld_to_canonical_dataset(&out_doc, &base) {
            Ok(ds) => ds,
            Err(why) => {
                s.fail(&e.id, format!("re-parse {cat} output: {why}"));
                continue;
            }
        };
        let expect_path = root.join(expect_rel);
        let exp_text = match std::fs::read_to_string(&expect_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read expect: {why}"));
                continue;
            }
        };
        let want = match jsonld_to_canonical_dataset(&exp_text, &base) {
            Ok(ds) => ds,
            Err(why) => {
                s.fail(&e.id, format!("re-parse expected output: {why}"));
                continue;
            }
        };
        if got == want {
            s.pass();
        } else {
            s.fail(
                &e.id,
                format!(
                    "{cat} RDF != expected RDF ({} vs {} quads)",
                    got.len(),
                    want.len()
                ),
            );
        }
    }
    s
}
