//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `frame` lane runner (split out of the
//! former monolithic `tests/jsonld_suite.rs`; behaviour byte-identical). Framing
//! runs against the SEPARATE `w3c/json-ld-framing` suite.

use super::common::*;
use sparq_core::Graph;
use std::path::Path;

/// [OPUS-4.8] sq-oy1f.19 — run the SEPARATE W3C `w3c/json-ld-framing` suite
/// against sparq's hand-rolled Framing Algorithm (`sparq_engine::serialize::
/// graph_to_jsonld_framed`, the `serialize-rdf` feature).
///
/// ## Pipeline (the REAL framing path)
///
/// 1. Parse the case `input` (an arbitrary EXPANDED/`@graph` JSON-LD document) →
///    RDF via the real oxjsonld ingest path (`parse_jsonld_dataset`). This is the
///    expanded-document framing ENTRY PATH the bead asked for: the framer takes a
///    `Graph`, not a JSON-LD doc, so the suite's expanded input is reduced to its
///    RDF and loaded into a `Graph` — exactly the bridge `compact::run_compact` uses.
/// 2. Read the case `frame` document (`*-frame.jsonld`: the frame pattern + its
///    `@context`) with the writer's own JSON reader.
/// 3. Run `graph_to_jsonld_framed(&graph, &frame)`.
/// 4. **Invariant (normative answer-equivalence):** re-parse BOTH sparq's framed
///    output AND the suite's NORMATIVE expected output (`*-out.jsonld`) to
///    canonical [`Dataset`]s and require they are equal —
///    `reparse(frame(D, F)) ≡ reparse(expected)`. Framing is a SELECT + RESHAPE
///    (it legitimately prunes / fills / drops), so the oracle anchors on the
///    W3C-expected framed document, NOT the input (see the `frame` floor). This is
///    envelope-insensitive and value-faithful while not requiring byte-identical
///    JSON layout (the same oxjsonld self-reparse oracle the other lanes use).
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * `requires` optional-feature cases — out of the gated surface.
/// * NegativeEvaluationTests (`expectErrorCode`) — sparq's framer is TOTAL and
///   never raises the spec's frame-validation errors, so it cannot honestly
///   "pass" by rejecting. SKIP (the compact-lane posture), never a counted pass.
/// * A positive case with no `expect` document, or whose `expect` the real
///   oxjsonld path cannot re-parse (out of the gated surface) — SKIP.
/// * Remote `input`/`frame` URLs — none in the pinned suite, but guarded (SKIP).
pub fn run_frame(root: &Path) -> Score {
    use sparq_engine::serialize::{graph_to_jsonld_framed, parse_context_json};
    let mut s = Score::default();
    let entries = match read_manifest(root, "frame") {
        Ok(e) => e,
        Err(why) => {
            s.fail("frame-manifest", why);
            return s;
        }
    };
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        // NegativeEvaluationTest: sparq's framer does not raise the spec's
        // frame-validation errors, so a faithful 1.1 framer cannot "pass" by
        // rejecting — SKIP (honest), never count as a pass.
        if e.is_negative || e.expect_error_code.is_some() {
            s.skip();
            continue;
        }
        let Some(frame_rel) = &e.frame else {
            s.skip();
            continue;
        };
        let Some(expect_rel) = &e.expect else {
            s.skip();
            continue;
        };
        // Remote input/frame (no network) — none in the pinned suite; guard.
        if e.input.starts_with("http://")
            || e.input.starts_with("https://")
            || frame_rel.starts_with("http://")
            || frame_rel.starts_with("https://")
        {
            s.skip();
            continue;
        }

        // 1. Parse the EXPANDED input document → RDF (the framing input).
        let input_path = root.join(&e.input);
        let in_text = match std::fs::read_to_string(&input_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read input: {why}"));
                continue;
            }
        };
        let base = frame_doc_base(e);
        let input_ds = match parse_jsonld_dataset(&in_text, &base) {
            Ok(ds) => ds,
            // An input the real oxjsonld path rejects is out of scope for the
            // framing round-trip (the toRdf lane gates ingest) — SKIP.
            Err(_) => {
                s.skip();
                continue;
            }
        };

        // 2. Read the frame document (the whole frame JSON: pattern + @context).
        let frame_path = root.join(frame_rel);
        let frame_text = match std::fs::read_to_string(&frame_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read frame: {why}"));
                continue;
            }
        };
        let Some(frame) = parse_context_json(&frame_text) else {
            // A non-object frame (array/string) is not drivable through the
            // single-object framer entry — SKIP.
            s.skip();
            continue;
        };

        // 3. Load the input RDF into a sparq Graph (named graphs preserved) and
        //    run the REAL framing writer over the expanded-document-derived RDF.
        let nq = dataset_to_nquads(&input_ds);
        let graph = match Graph::load_dataset(&nq, "nquads") {
            Ok(g) => g,
            Err(why) => {
                s.fail(&e.id, format!("load input rdf into graph: {why}"));
                continue;
            }
        };
        let framed = graph_to_jsonld_framed(&graph, &frame);

        // 4. The normative answer-equivalence invariant: re-parse BOTH sparq's
        //    framed output and the suite's expected output, require RDF equality.
        let got = match jsonld_to_canonical_dataset(&framed, &base) {
            Ok(ds) => ds,
            Err(why) => {
                s.fail(&e.id, format!("re-parse framed output: {why}"));
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
                    "framed RDF != expected RDF ({} vs {} quads)",
                    got.len(),
                    want.len()
                ),
            );
        }
    }
    s
}
