//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `compact` lane runner (split out of
//! the former monolithic `tests/jsonld_suite.rs`; behaviour byte-identical).

use super::common::*;
use sparq_core::Graph;
use std::path::Path;

/// [OPUS-4.8] sq-3uos5 — run the W3C JSON-LD `compact` category against sparq's
/// hand-rolled Compaction Algorithm (`sparq_engine::serialize::
/// graph_to_jsonld_compact`, the `serialize-rdf` feature).
///
/// ## Pipeline (the REAL compaction path)
///
/// 1. Parse the case `input` (`.jsonld`) → RDF via the real oxjsonld ingest path
///    (`parse_jsonld_dataset`); this dataset is the losslessness ORACLE.
/// 2. Re-emit that dataset as N-Quads and load it into a sparq [`Graph`]
///    (preserving named graphs) — the writer takes a `Graph`, not a JSON-LD doc,
///    because sparq compacts RDF, not arbitrary documents.
/// 3. Read the case `@context` (`read_context_member`) and run
///    `graph_to_jsonld_compact(&graph, &ctx)`.
/// 4. **Invariant (answer-safety):** re-parse the compacted document through
///    oxjsonld back to a canonical [`Dataset`] and require it equals the input
///    dataset — `reparse(compact(D, ctx)) ≡ D`. Compaction must be LOSSLESS
///    w.r.t. the RDF. This is the SAME round-trip oracle `from_rdf::run_fromrdf`
///    uses (oxjsonld self-reparse equivalence), NOT a JSON-structural diff against
///    the suite's `expect.jsonld` (whose layout sparq's writer does not reproduce
///    byte-for-byte).
///
/// ## Honest oracle caveat
///
/// The oracle is *oxjsonld self-reparse equivalence*, exactly like toRdf/fromRdf.
/// A case where sparq's `@reverse` compaction double-inverts, or where a
/// non-string `@language`/`@none` value is mis-shaped, can still PASS here when
/// our OWN re-parse round-trips it (oxjsonld reads back what oxjsonld would emit)
/// even though a strict third-party processor (pyld) would read it inverted.
/// Strict third-party (pyld) faithfulness for those shapes is tracked separately
/// (a child of sq-oy1f) and is NOT claimed by this ratchet — see the crate README.
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * `requires` optional-feature cases — out of the gated surface.
/// * Non-object / remote / array `@context` — not drivable through the inline
///   single-object writer (no network).
/// * NegativeEvaluationTests — sparq's compaction is TOTAL (it never raises the
///   spec's compaction/context errors: list-of-lists, invalid term definition,
///   `@protected` redefinition, processing-mode conflict, …). A 1.1 writer that
///   does not model those errors cannot honestly "pass" by rejecting, so these
///   are SKIPPED rather than counted.
/// * Positive cases whose input → RDF is EMPTY (free-floating-node drops, pure
///   `@context`/`@graph` framing with no triples): there is no RDF to compact, so
///   the round-trip is vacuous and tells us nothing about the algorithm — SKIP.
pub fn run_compact(root: &Path) -> Score {
    use sparq_engine::serialize::graph_to_jsonld_compact;
    let mut s = Score::default();
    let entries = match read_manifest(root, "compact") {
        Ok(e) => e,
        Err(why) => {
            s.fail("compact-manifest", why);
            return s;
        }
    };
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        // NegativeEvaluationTest: sparq's compaction does not raise the spec's
        // compaction/context errors, so a faithful 1.1 writer cannot "pass" by
        // rejecting — SKIP (honest), never count as a pass.
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
        let Some(ctx_rel) = &e.context else {
            s.skip();
            continue;
        };

        // 1. Parse the input JSON-LD → RDF (the oracle dataset). Skip remote
        //    inputs (no network).
        if e.input.starts_with("http://") || e.input.starts_with("https://") {
            s.skip();
            continue;
        }
        let input_path = root.join(&e.input);
        let in_text = match std::fs::read_to_string(&input_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read input: {why}"));
                continue;
            }
        };
        let base = doc_base(e);
        let want = match parse_jsonld_dataset(&in_text, &base) {
            Ok(ds) => ds,
            // An input the real oxjsonld path rejects is out of scope for the
            // compaction round-trip (the toRdf lane already gates ingest); SKIP.
            Err(_) => {
                s.skip();
                continue;
            }
        };
        // No triples → nothing to compact; the round-trip is vacuous. SKIP.
        if want.is_empty() {
            s.skip();
            continue;
        }

        // 2. Read the case @context (skip non-inline / remote / multi forms).
        let ctx_path = root.join(ctx_rel);
        let ctx = match read_context_member(&ctx_path) {
            Ok(c) => c,
            Err(_) => {
                s.skip();
                continue;
            }
        };

        // 3. Load the input RDF into a sparq Graph (named graphs preserved) and
        //    run the REAL compaction writer.
        let nq = dataset_to_nquads(&want);
        let graph = match Graph::load_dataset(&nq, "nquads") {
            Ok(g) => g,
            Err(why) => {
                s.fail(&e.id, format!("load input rdf into graph: {why}"));
                continue;
            }
        };
        let compacted = graph_to_jsonld_compact(&graph, &ctx);

        // 4. The losslessness invariant: re-parse the compacted document and
        //    require RDF-dataset equivalence with the input.
        let got = match jsonld_to_canonical_dataset(&compacted, &base) {
            Ok(ds) => ds,
            Err(why) => {
                s.fail(&e.id, format!("re-parse compacted output: {why}"));
                continue;
            }
        };
        if got == want {
            s.pass();
        } else {
            s.fail(
                &e.id,
                format!(
                    "compaction not lossless ({} vs {} quads)",
                    got.len(),
                    want.len()
                ),
            );
        }
    }
    s
}
