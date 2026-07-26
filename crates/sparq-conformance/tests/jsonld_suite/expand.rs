//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `expand` lane runner (split out of the
//! former monolithic `tests/jsonld_suite.rs`; behaviour byte-identical). The `expand`
//! category uses the NATIVE DOCUMENT-LEVEL oracle (`sparq_jsonld::expand()` +
//! `json_ld_equal`), NOT the RDF-writer path.

use super::common::*;
use serde_json::Value;
// [SONNET-4.6] sq-kk1mq — native expand() for the document-level expand oracle.
// [SONNET-4.6] sq-oy1f.45 — FsLoader maps the W3C suite URL prefix to local fixtures so
// `@context`/`@import` relative-URL references (e.g. t0126, tc034, tso08) are resolved
// from the local checkout rather than hitting the network.
use sparq_jsonld::{expand as jsonld_expand, FsLoader, JsonLdOptions, ProcessingMode};
use std::path::Path;

/// [SONNET-4.6] sq-kk1mq — run the W3C JSON-LD `expand` category with the
/// NATIVE DOCUMENT-LEVEL oracle: call `sparq_jsonld::expand()` directly on the
/// input document and deep-compare the result to the suite's expected expanded
/// document via `json_ld_equal`.
///
/// ## Pipeline (the native expand path)
///
/// 1. Parse the case `input` (`.jsonld`) as a `sparq_jsonld::Json` AST —
///    the expander's native input type.
/// 2. Build `JsonLdOptions` from the manifest entry (`base`, `expandContext`,
///    `processingMode`/`specVersion`).  `expandContext` is a path to a context
///    file; it is read and parsed as a `sparq_jsonld::Json` and forwarded as
///    `options.expand_context`.
/// 3. Call `sparq_jsonld::expand(&input_json, &opts, &NoopLoader)`.
///    Remote `@context` / `@import` references raise `loading document failed`
///    from the `NoopLoader` (deny-by-default).
/// 4. Convert the `Result<Json, JsonLdError>` output to a `serde_json::Value`
///    by writing and re-parsing (no structural loss — both ASTs are JSON).
/// 5. Parse the suite's expected document as a `serde_json::Value`.
/// 6. Compare with `json_ld_equal` (object key order insignificant; array
///    order significant only inside `@list`; integers compared exactly,
///    non-integral numbers compared as f64 so `1` ≡ `1.0`).
///
/// ## [OPUS-5] sq-gzsky — NegativeEvaluationTests are RUN, not skipped
///
/// The 109 negatives used to be the ENTIRE `expand` gap (276/385 = 71.6% strict,
/// against peers at 97–100%). They are now driven exactly like the `frame` lane's
/// (sq-oy1f.29): the case passes **iff** `expand()` errors with EXACTLY the
/// manifest's `expectErrorCode`. A raised-but-WRONG code is a FAILURE, never a
/// pass — which is the whole reason `JsonLdErrorCode` is a closed enum carrying the
/// verbatim spec strings (`error.rs` module doc).
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * `requires` optional-feature cases (same as all other lanes).
/// * **`option.specVersion: json-ld-1.0` NEGATIVES** — the suite's `vocab.jsonld`
///   defines `specVersion` as "the JSON-LD version to which the test applies", so
///   these assert what a **1.0** processor must reject. sparq is a 1.1 processor
///   and several of them are cases 1.1 deliberately made LEGAL, so raising would be
///   wrong: `#ter24`/`#ter32` want `list of lists` and `#ter02`/`#ter03` want
///   `recursive context inclusion` — two codes 1.1 RETIRED (they are absent from
///   the 1.1 error registry; 1.1 reports cyclic contexts as `context overflow`),
///   and `#t0115`/`#t0116` want `invalid vocab mapping` for a relative `@vocab`,
///   which 1.1 explicitly permits. SKIP is the honest outcome, never a pass.
///   The exact skipped set is pinned by `expand_1_0_negative_skips_are_pinned` so a
///   suite-pin bump that adds one fails loudly instead of silently absorbing it.
///   Note this is NARROW: it does not touch `option.processingMode: json-ld-1.0`
///   cases of the 1.1 suite (e.g. `#tes01`), which a 1.1 processor MUST honour and
///   which this lane RUNS.
/// * A NegativeEvaluationTest with no `expectErrorCode` — nothing to assert.
/// * Remote `input` URL — no network.
/// * No `expect` file on a positive — nothing to compare.
pub fn run_expand_native(root: &Path) -> Score {
    let mut s = Score::default();
    let entries = match read_manifest(root, "expand") {
        Ok(e) => e,
        Err(why) => {
            s.fail("expand-manifest", why);
            return s;
        }
    };
    // [SONNET-4.6] sq-oy1f.45 — FsLoader maps the suite's base URL prefix to the local
    // fixture directory so `@context` / `@import` IRI references in test inputs are
    // resolved against the checked-out files instead of hitting the network.  The mapping
    // mirrors the `SUITE_BASE` constant (`https://w3c.github.io/json-ld-api/tests/`).
    let loader = FsLoader::new().map_prefix(SUITE_BASE, root);
    for e in &entries {
        if e.requires.is_some() {
            s.skip();
            continue;
        }
        // [OPUS-5] sq-gzsky — NegativeEvaluationTests are now RUN (see the module
        // doc), except the two honest buckets: a 1.0-only negative (`specVersion`,
        // NOT `processingMode` — see `is_one_zero_only_negative`) and one with no
        // `expectErrorCode` to assert against.
        if e.is_negative && (is_one_zero_only_negative(e) || e.expect_error_code.is_none()) {
            s.skip();
            continue;
        }
        if !e.is_negative && e.expect.is_none() {
            s.skip();
            continue;
        }
        // Remote input (no network) — guard.
        if e.input.starts_with("http://") || e.input.starts_with("https://") {
            s.skip();
            continue;
        }

        // 1. Read and parse the input document as sparq_jsonld::Json.
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

        // 2. Build JsonLdOptions from the manifest entry.
        let base = doc_base(e);
        let processing_mode = match (e.processing_mode.as_deref(), e.spec_version.as_deref()) {
            (Some("json-ld-1.0"), _) | (_, Some("json-ld-1.0")) => ProcessingMode::JsonLd10,
            _ => ProcessingMode::JsonLd11,
        };
        // JsonLdOptions is #[non_exhaustive] — build via default() + field mutation.
        let mut opts = JsonLdOptions::default();
        opts.base = Some(base.clone());
        opts.processing_mode = processing_mode;

        // Forward expandContext if present: read the context file and parse as
        // sparq_jsonld::Json.  The expander unwraps one "@context" layer itself
        // (options.expand_context.get("@context").unwrap_or(&ctx)).
        if let Some(ctx_rel) = &e.expand_context_path {
            let ctx_path = root.join(ctx_rel);
            let ctx_text = match std::fs::read_to_string(&ctx_path) {
                Ok(t) => t,
                Err(why) => {
                    s.fail(&e.id, format!("read expandContext file: {}", why));
                    continue;
                }
            };
            match sparq_jsonld::Json::parse(&ctx_text) {
                Ok(ctx_json) => opts.expand_context = Some(ctx_json),
                Err(why) => {
                    s.fail(&e.id, format!("parse expandContext JSON: {}", why));
                    continue;
                }
            }
        }

        // 3. Call the native expand() algorithm. The FsLoader resolves any `@context` /
        // `@import` relative-URL references to local fixture files (sq-oy1f.45).
        let result = jsonld_expand(&input_json, &opts, &loader);

        // [OPUS-5] sq-gzsky — the NEGATIVE lane: the case passes iff expand()
        // errors with EXACTLY the manifest's `expectErrorCode`. A raised-but-wrong
        // code is a FAILURE, not a pass (honesty over score) — that is the whole
        // point of modelling the registry as a closed enum.
        if e.is_negative {
            let want_code = e.expect_error_code.as_deref().unwrap_or("");
            match &result {
                Err(err) if err.code().as_str() == want_code => s.pass(),
                Err(err) => s.fail(
                    &e.id,
                    format!("wrong error code: got '{}', want '{}'", err.code(), want_code),
                ),
                Ok(_) => s.fail(
                    &e.id,
                    format!("negative test expanded without error (want '{}')", want_code),
                ),
            }
            continue;
        }

        let expanded = match result {
            Ok(j) => j,
            Err(why) => {
                s.fail(&e.id, format!("expand() error: {}", why));
                continue;
            }
        };

        // 4. Convert the expand() output to serde_json::Value.
        let got: Value = match sparq_json_to_serde(&expanded) {
            Ok(v) => v,
            Err(why) => {
                s.fail(&e.id, format!("convert expand output: {}", why));
                continue;
            }
        };

        // 5. Read and parse the expected document. (Positive cases without an
        // `expect` member were skipped above, so this is always present here.)
        let expect_path = root.join(e.expect.as_deref().unwrap_or_default());
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

        // 6. Document-level JSON-LD equality (oracle pinned per sq-kk1mq).
        // Report the JSON kind (with size for array/object) so a mismatch on
        // a non-array structure is immediately diagnosable rather than always
        // showing "0 nodes".
        fn json_kind_desc(v: &Value) -> String {
            match v {
                Value::Array(a) => format!("array({} items)", a.len()),
                Value::Object(o) => format!("object({} keys)", o.len()),
                Value::String(_) => "string".to_owned(),
                Value::Null => "null".to_owned(),
                Value::Bool(b) => format!("bool({})", b),
                Value::Number(n) => format!("number({})", n),
            }
        }
        if json_ld_equal(&got, &want) {
            s.pass();
        } else {
            s.fail(
                &e.id,
                format!(
                    "expand JSON mismatch: got {}, want {}",
                    json_kind_desc(&got),
                    json_kind_desc(&want),
                ),
            );
        }
    }
    s
}

/// [OPUS-5] sq-gzsky — True iff `e` is a NegativeEvaluationTest that applies ONLY to
/// the 2014 JSON-LD **1.0** REC, and so is formally inapplicable to sparq (a 1.1
/// processor). Scope pinned by [`expand_1_0_negative_skips_are_pinned`].
///
/// The predicate is `option.specVersion == "json-ld-1.0"`, which the pinned suite's
/// own `vocab.jsonld` defines as "the JSON-LD version to which the test applies".
/// It is deliberately NOT `option.processingMode`: `processingMode: json-ld-1.0` is
/// a JSON-LD **1.1** API option that a 1.1 processor MUST honour, and those cases
/// (e.g. `#tes01`) RUN in this lane.
///
/// Why raising on these would be WRONG, not merely unimplemented — every one of the
/// six is behaviour 1.1 deliberately changed:
///
/// * `#ter24`, `#ter32` expect `list of lists`, and `#ter02`, `#ter03` expect
///   `recursive context inclusion`. Neither string is in the JSON-LD 1.1 error
///   registry (`JsonLdErrorCode` is a closed enum over that registry, so neither is
///   even expressible): 1.1 ALLOWS lists of lists, and reports a cyclic context as
///   `context overflow` instead.
/// * `#t0115`, `#t0116` expect `invalid vocab mapping` for a relative `@vocab`,
///   which 1.1 explicitly permits (§4.1.2 resolves it against the base/vocab).
///
/// This is a broader shape than the compact lane's single pinned `#t0038` because
/// `specVersion: json-ld-1.0` NEGATIVES are a whole class here, not one case — but
/// it carries the same anti-silence guarantee: the exact matched id set is asserted
/// below, so a suite-pin bump that adds one fails loudly and forces a decision.
fn is_one_zero_only_negative(e: &Entry) -> bool {
    e.is_negative && e.spec_version.as_deref() == Some("json-ld-1.0")
}

/// Regression pin for the scope of the 1.0-only NEGATIVE skip (sq-gzsky): the
/// predicate matches EXACTLY the six cases justified in
/// [`is_one_zero_only_negative`], and the `processingMode: json-ld-1.0` negative
/// `#tes01` — which a 1.1 processor MUST reject — is NOT among them.
#[test]
fn expand_1_0_negative_skips_are_pinned() {
    let root = suite_root();
    if !root.exists() {
        eprintln!(
            "SKIP: W3C JSON-LD suite not present at {} — run scripts/fetch-jsonld-tests.sh",
            root.display()
        );
        return;
    }
    let entries = read_manifest(&root, "expand").expect("read expand manifest");

    let skipped: Vec<&str> = entries
        .iter()
        .filter(|e| is_one_zero_only_negative(e))
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(
        skipped,
        ["#t0115", "#t0116", "#ter02", "#ter03", "#ter24", "#ter32"],
        "the 1.0-only expand negative skip set changed — a suite-pin bump must be \
         decided explicitly (is the new case really inapplicable to a 1.1 processor?), \
         never silently absorbed"
    );

    // #tes01 is `processingMode: json-ld-1.0` with `specVersion: json-ld-1.1` — a
    // 1.1-suite test of 1.0 PROCESSING MODE. It must RUN (and pass) here.
    let es01 = entries
        .iter()
        .find(|e| e.id == "#tes01")
        .expect("#tes01 missing from the pinned expand manifest");
    assert!(
        !is_one_zero_only_negative(es01)
            && es01.processing_mode.as_deref() == Some("json-ld-1.0")
            && es01.expect_error_code.is_some(),
        "#tes01 (processingMode: json-ld-1.0) must RUN, not skip"
    );
}
