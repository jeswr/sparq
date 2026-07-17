//! [FABLE-5] sq-oy1f.27 — the W3C JSON-LD 1.1 `compact` lane runner, on the NATIVE
//! DOCUMENT-LEVEL oracle: `sparq_jsonld::compact::compact()` (the spec Compaction
//! Algorithm over the expanded document) deep-compared against the suite's NORMATIVE
//! expected document with `json_ld_equal` — the same oracle shape as the `expand` and
//! `flatten` lanes (sq-kk1mq / sq-oy1f.26). This REPLACES the old oxjsonld self-reparse
//! round-trip oracle (`reparse(compact(D, ctx)) ≡ D`), which measured RDF losslessness
//! of the engine's RDF-first writer, not the Compaction Algorithm — and could pass a
//! document whose JSON shape a strict third-party processor reads differently (the exact
//! honesty gap the oracle correction closes; see the floor doc for the side-by-side
//! re-pin).

use super::common::*;
use serde_json::Value;
use sparq_jsonld::{FsLoader, JsonLdOptions, ProcessingMode};
use std::path::Path;

/// [FABLE-5] sq-oy1f.27 — run the W3C JSON-LD `compact` category with the NATIVE
/// DOCUMENT-LEVEL oracle.
///
/// ## Pipeline (the native compact path)
///
/// 1. Read the case `input` (`.jsonld`) and parse it as a `sparq_jsonld::Json`.
/// 2. Read the case `context` (the sibling `*-context.jsonld`) and parse it likewise.
/// 3. Build `JsonLdOptions` from the manifest entry: `base` (the document IRI or the
///    `option.base` override), `processingMode`/`specVersion`, `compactArrays`,
///    `compactToRelative`.
/// 4. Call `sparq_jsonld::compact::compact(input, context, opts, loader)` — this expands
///    the input through the native Expansion Algorithm, then runs the spec Compaction
///    Algorithm. The `FsLoader` maps the suite URL prefix to the local fixture tree so
///    remote `@context`/`@import` references resolve offline (deny-by-default otherwise).
/// 5. Deep-compare against the suite's EXPECTED document via `json_ld_equal` (object key
///    order insignificant; array order significant only inside explicit `@list` values;
///    numbers compared as in the expand lane).
///
/// ## Honest oracle caveat
///
/// `json_ld_equal` treats arrays outside an explicit `"@list"` key as SETS. In a
/// *compacted* document a `@container: @list` term's array carries list order without
/// the `@list` marker, so the comparator cannot see it — a wrongly-ordered compacted
/// list would still pass. The strict order-sensitive pass count is measured and recorded
/// alongside the floor (see `floors::compact`); list-order fidelity is covered by the
/// crate-local `tests/compact.rs` unit tests, which assert exact shapes.
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * `requires` optional-feature cases — out of the gated surface.
/// * NegativeEvaluationTests — compaction raises `invalid @nest value`, but error-code
///   completeness across context processing/expansion/compaction is unverified; the
///   negative lanes are bead sq-oy1f.31. SKIP (honest), never a counted pass.
/// * `specVersion: json-ld-1.0` positives (sq-uzdw7) — 1.0-only expectations a
///   REC-conformant 1.1 processor intentionally does not reproduce (`#t0038` expects
///   1.0-era compact-IRI creation through an EXPANDED term definition, which `#tp001`
///   pins as NOT happening in 1.0 processing mode). Same convention as the
///   flatten/fromRdf lanes. Scoped to `specVersion` ONLY — `processingMode:
///   json-ld-1.0` cases (e.g. `#tp001`) are 1.1-conformance pins and still RUN.
/// * Remote `input` URL — no network.
/// * No `expect` / no `context` member — nothing to compare / compact against.
pub fn run_compact(root: &Path) -> Score {
    let mut s = Score::default();
    let entries = match read_manifest(root, "compact") {
        Ok(e) => e,
        Err(why) => {
            s.fail("compact-manifest", why);
            return s;
        }
    };
    // Map the suite's base URL prefix to the local fixture directory (same as the
    // expand lane) so remote-context references in inputs resolve offline.
    let loader = FsLoader::new().map_prefix(SUITE_BASE, root);
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        if e.is_negative {
            s.skip();
            continue;
        }
        // [FABLE-5] sq-uzdw7 — JSON-LD-1.0-only positives: honest SKIP (see the doc
        // comment). Deliberately NOT matching `processing_mode` here: `#tp001` runs
        // in 1.0 processing mode and pins the very behaviour `#t0038` contradicts.
        if e.spec_version.as_deref() == Some("json-ld-1.0") {
            s.skip();
            continue;
        }
        let Some(expect_rel) = &e.expect else {
            s.skip();
            continue;
        };
        let Some(ctx_rel) = &e.context else {
            s.skip();
            continue;
        };
        if e.input.starts_with("http://") || e.input.starts_with("https://") {
            s.skip();
            continue;
        }

        // 1-2. Read and parse the input document and the caller context.
        let input_path = root.join(&e.input);
        let in_text = match std::fs::read_to_string(&input_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read input: {}", why));
                continue;
            }
        };
        let input_json = match sparq_jsonld::Json::parse(&in_text) {
            Ok(j) => j,
            Err(why) => {
                s.fail(&e.id, format!("parse input JSON: {}", why));
                continue;
            }
        };
        let ctx_path = root.join(ctx_rel);
        let ctx_text = match std::fs::read_to_string(&ctx_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read context: {}", why));
                continue;
            }
        };
        let ctx_json = match sparq_jsonld::Json::parse(&ctx_text) {
            Ok(j) => j,
            Err(why) => {
                s.fail(&e.id, format!("parse context JSON: {}", why));
                continue;
            }
        };

        // 3. Options from the manifest entry.
        let processing_mode = match (e.processing_mode.as_deref(), e.spec_version.as_deref()) {
            (Some("json-ld-1.0"), _) | (_, Some("json-ld-1.0")) => ProcessingMode::JsonLd10,
            _ => ProcessingMode::JsonLd11,
        };
        // JsonLdOptions is #[non_exhaustive] — build via default() + field mutation.
        let mut opts = JsonLdOptions::default();
        opts.base = Some(doc_base(e));
        opts.processing_mode = processing_mode;
        if let Some(ca) = e.compact_arrays {
            opts.compact_arrays = ca;
        }
        if let Some(cr) = e.compact_to_relative {
            opts.compact_to_relative = cr;
        }

        // 4. The native document-level Compaction Algorithm.
        let compacted =
            match sparq_jsonld::compact::compact(&input_json, &ctx_json, &opts, &loader) {
                Ok(j) => j,
                Err(why) => {
                    s.fail(&e.id, format!("compact() error: {}", why));
                    continue;
                }
            };

        // 5. Compare against the suite's NORMATIVE expected document.
        let got: Value = match sparq_json_to_serde(&compacted) {
            Ok(v) => v,
            Err(why) => {
                s.fail(&e.id, format!("convert compact output: {}", why));
                continue;
            }
        };
        let expect_path = root.join(expect_rel);
        let exp_text = match std::fs::read_to_string(&expect_path) {
            Ok(t) => t,
            Err(why) => {
                s.fail(&e.id, format!("read expect: {}", why));
                continue;
            }
        };
        let want: Value = match serde_json::from_str(&exp_text) {
            Ok(v) => v,
            Err(why) => {
                s.fail(&e.id, format!("parse expect JSON: {}", why));
                continue;
            }
        };
        if json_ld_equal(&got, &want) {
            s.pass();
        } else {
            // Truncated got/want dumps keep a regression diagnosable from the
            // failure log without re-running locally.
            let mut g = got.to_string();
            let mut w = want.to_string();
            g.truncate(240);
            w.truncate(240);
            s.fail(
                &e.id,
                format!("compacted JSON differs\n    got:  {}\n    want: {}", g, w),
            );
        }
    }
    s
}
