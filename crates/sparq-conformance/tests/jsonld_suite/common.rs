//! [FABLE-5] sq-oy1f.40 — shared machinery for the W3C JSON-LD 1.1 conformance
//! lane submodules (`to_rdf`, `from_rdf`, `expand`, `compact`, `flatten`,
//! `frame`). Split out of the former monolithic `tests/jsonld_suite.rs` — the
//! manifest walker, the `Score` scoreboard, the canonical-dataset helpers, the
//! JSON-LD document-level comparator, and the not-implemented bucket reporting all
//! live here; each per-lane runner lives in its own sibling submodule. Pure
//! refactor: the CI invocation, the compiled test binary, and the pass counts are
//! byte-identical to before the split.
//!
//! The six ratchet FLOORS now live LIB-SIDE in `sparq_conformance::floors::<lane>`
//! (sq-oy1f.40) and are re-exported here so the per-lane runners keep spelling
//! `TORDF_FLOOR` etc. unchanged while reading the ONE compile-time source the
//! scoreboard registry also imports (no textual drift).

use oxrdf::dataset::CanonicalizationAlgorithm;
use oxrdf::{Dataset, Quad};
use serde_json::Value;
use sparq_core::Graph;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// [FABLE-5] sq-oy1f.40 — the six ratchet floors, re-exported from the LIB-SIDE
// single source (`sparq_conformance::floors::<lane>::FLOOR`). The runner's
// `assert!(pass >= *_FLOOR)` and `scoreboard::SUITES`' `ratchet_floor` now read the
// SAME const at compile time, so they cannot drift (kills the #1463 floor-drift
// class). The `*_FLOOR` aliases keep the runner call sites unchanged.
pub use sparq_conformance::floors::compact::FLOOR as COMPACT_FLOOR;
pub use sparq_conformance::floors::expand::FLOOR as EXPAND_FLOOR;
pub use sparq_conformance::floors::flatten::FLOOR as FLATTEN_FLOOR;
pub use sparq_conformance::floors::frame::FLOOR as FRAME_FLOOR;
pub use sparq_conformance::floors::from_rdf::FLOOR as FROMRDF_FLOOR;
pub use sparq_conformance::floors::to_rdf::FLOOR as TORDF_FLOOR;

/// [OPUS-4.8] sq-oy1f.19 — the framing suite's declared base (its
/// `baseIri`), used to resolve each frame test's input path into the document
/// IRI. Distinct from `SUITE_BASE` (the json-ld-api base) — framing is a
/// separate W3C repo.
pub const FRAME_SUITE_BASE: &str = "https://w3c.github.io/json-ld-framing/tests/";

/// The suite's declared base for resolving each test's input path into the
/// document IRI (the toRdf base when `option.base` is absent).
pub const SUITE_BASE: &str = "https://w3c.github.io/json-ld-api/tests/";

pub fn suite_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/sparq-conformance; the workspace root is two up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/json-ld-api/tests")
}

/// [OPUS-4.8] sq-oy1f.19 — the SEPARATE `w3c/json-ld-framing` suite root
/// (`scripts/fetch-jsonld-framing-tests.sh`). Framing lives in its own W3C
/// repo, not under `json-ld-api`.
pub fn frame_suite_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/json-ld-framing/tests")
}

#[derive(Default)]
pub struct Score {
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    pub failures: Vec<(String, String)>,
}

impl Score {
    pub fn pass(&mut self) {
        self.pass += 1;
    }
    pub fn fail(&mut self, id: &str, why: String) {
        self.fail += 1;
        self.failures.push((id.to_string(), why));
    }
    pub fn skip(&mut self) {
        self.skip += 1;
    }
}

/// Parse an N-Quads file into a canonicalized oxrdf [`Dataset`] for isomorphic
/// comparison (blank-node-blind).
pub fn nquads_to_canonical_dataset(text: &str) -> Result<Dataset, String> {
    let mut ds = Dataset::new();
    for q in oxttl::NQuadsParser::new().for_slice(text.as_bytes()) {
        let q: Quad = q.map_err(|e| e.to_string())?;
        ds.insert(&q);
    }
    ds.canonicalize(CanonicalizationAlgorithm::Unstable);
    Ok(ds)
}

/// Parse a JSON-LD document (through `oxjsonld`, the real ingest parser) into a
/// canonicalized oxrdf [`Dataset`].
pub fn jsonld_to_canonical_dataset(doc: &str, base: &str) -> Result<Dataset, String> {
    let mut ds = Dataset::new();
    let parser = oxjsonld::JsonLdParser::new()
        .with_base_iri(base)
        .map_err(|e| format!("invalid base {base:?}: {e}"))?;
    for q in parser.for_slice(doc.as_bytes()) {
        let q = q.map_err(|e| e.to_string())?;
        ds.insert(&q);
    }
    ds.canonicalize(CanonicalizationAlgorithm::Unstable);
    Ok(ds)
}

// ---- The parsed manifest entry --------------------------------------------

/// One parsed manifest entry's salient fields.
pub struct Entry {
    pub id: String,
    pub is_negative: bool,
    pub input: String,
    pub expect: Option<String>,
    pub base_override: Option<String>,
    /// An optional-feature requirement (`GeneralizedRdf` / `I18nDatatype` /
    /// `CompoundLiteral`); sparq does not opt into these, so such entries are
    /// SKIPPED (not failed) — they are out of the current gated surface.
    pub requires: Option<String>,
    /// [OPUS-4.8] sq-3uos5 — the `compact` manifest's top-level `context`
    /// member: a path (relative to the suite root) to the sibling
    /// `*-context.jsonld` file holding the caller `@context` to compact
    /// against. (The toRdf/fromRdf manifests have no such member; it stays
    /// `None` there.)
    pub context: Option<String>,
    /// [OPUS-4.8] sq-3uos5 — `option.specVersion` (`json-ld-1.0`/`-1.1`) and
    /// `option.processingMode`, recorded so the compact runner can SKIP the
    /// JSON-LD-1.0-only error-raising negatives sparq's 1.1 writer does not
    /// model (it is not a faithful pass for a 1.1 processor to "reject" them).
    pub spec_version: Option<String>,
    pub processing_mode: Option<String>,
    /// [OPUS-4.8] sq-oy1f.19 — the `frame` manifest's top-level `frame`
    /// member: a path (relative to the suite root) to the sibling
    /// `*-frame.jsonld` document (the frame pattern + its `@context`) to frame
    /// the input against. `None` for the toRdf/fromRdf/compact manifests.
    pub frame: Option<String>,
    /// [OPUS-4.8] sq-oy1f.19 — a NegativeEvaluationTest's expected JSON-LD
    /// error code (`invalid frame`, `invalid @embed value`, …). Recorded so the
    /// frame runner can SKIP the error-raising negatives sparq's TOTAL framer
    /// does not model (it never raises the spec's frame-validation errors), the
    /// same honesty posture the compact lane takes toward its negatives.
    pub expect_error_code: Option<String>,
    /// [SONNET-4.6] sq-kk1mq — `option.expandContext`: a path (relative to the
    /// suite root) to a context file. Forwarded to the native expand() oracle as
    /// `JsonLdOptions.expand_context`; `None` for the toRdf/fromRdf/compact/frame
    /// manifests (the expand manifest is the only one that has this option).
    pub expand_context_path: Option<String>,
    /// [FABLE-5] sq-oy1f.27 — `option.compactArrays` / `option.compactToRelative`
    /// (compact-manifest options). Forwarded to the native compact() oracle as
    /// `JsonLdOptions.compact_arrays` / `.compact_to_relative`; `None` (spec default
    /// `true`) when the manifest entry does not set them.
    pub compact_arrays: Option<bool>,
    pub compact_to_relative: Option<bool>,
}

/// Read `<cat>-manifest.jsonld` and return its `sequence` entries.
pub fn read_manifest(root: &Path, cat: &str) -> Result<Vec<Entry>, String> {
    let path = root.join(format!("{}-manifest.jsonld", cat));
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let v: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let seq = v
        .get("sequence")
        .and_then(Value::as_array)
        .ok_or("manifest has no sequence array")?;
    let mut out = Vec::new();
    for e in seq {
        let id = e
            .get("@id")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string();
        let types: Vec<&str> = e
            .get("@type")
            .map(|t| match t {
                Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
                Value::String(s) => vec![s.as_str()],
                _ => vec![],
            })
            .unwrap_or_default();
        let is_negative = types.iter().any(|t| t.contains("NegativeEvaluationTest"));
        let input = e
            .get("input")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let expect = e.get("expect").and_then(Value::as_str).map(str::to_string);
        let opt = e.get("option");
        let base_override = opt
            .and_then(|o| o.get("base"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let requires = e
            .get("requires")
            .and_then(Value::as_str)
            .map(str::to_string);
        // [OPUS-4.8] sq-3uos5 — compact-only manifest members (None elsewhere).
        let context = e.get("context").and_then(Value::as_str).map(str::to_string);
        let spec_version = opt
            .and_then(|o| o.get("specVersion"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let processing_mode = opt
            .and_then(|o| o.get("processingMode"))
            .and_then(Value::as_str)
            .map(str::to_string);
        // [OPUS-4.8] sq-oy1f.19 — frame-only manifest members (None elsewhere).
        let frame = e.get("frame").and_then(Value::as_str).map(str::to_string);
        let expect_error_code = e
            .get("expectErrorCode")
            .and_then(Value::as_str)
            .map(str::to_string);
        // [SONNET-4.6] sq-kk1mq — expand-only manifest option: `option.expandContext`
        // is a path (relative to the suite root) to a context file.
        let expand_context_path = opt
            .and_then(|o| o.get("expandContext"))
            .and_then(Value::as_str)
            .map(str::to_string);
        // [FABLE-5] sq-oy1f.27 — compact-manifest options (None elsewhere).
        let compact_arrays = opt.and_then(|o| o.get("compactArrays")).and_then(Value::as_bool);
        let compact_to_relative = opt
            .and_then(|o| o.get("compactToRelative"))
            .and_then(Value::as_bool);
        out.push(Entry {
            id,
            is_negative,
            input,
            expect,
            base_override,
            requires,
            context,
            spec_version,
            processing_mode,
            frame,
            expect_error_code,
            expand_context_path,
            compact_arrays,
            compact_to_relative,
        });
    }
    Ok(out)
}

/// The document IRI / base for a toRdf test: `option.base` if given, else the
/// suite base joined with the input path.
pub fn doc_base(e: &Entry) -> String {
    e.base_override
        .clone()
        .unwrap_or_else(|| format!("{}{}", SUITE_BASE, e.input))
}

/// [OPUS-4.8] sq-oy1f.19 — the base/document IRI for a `frame` test: the
/// `option.base` override if given, else the framing suite base joined with the
/// input path. (Distinct from `doc_base`, which uses the json-ld-api base.)
pub fn frame_doc_base(e: &Entry) -> String {
    e.base_override
        .clone()
        .unwrap_or_else(|| format!("{}{}", FRAME_SUITE_BASE, e.input))
}

/// Parse a JSON-LD document through the REAL sparq-core ingest path (oxjsonld
/// behind the `jsonld` feature) into a canonicalized dataset. Uses
/// `oxjsonld` directly here (the same parser sparq-core calls) with the test's
/// base so the comparison is base-faithful.
pub fn parse_jsonld_dataset(text: &str, base: &str) -> Result<Dataset, String> {
    // Round-trip through sparq-core to exercise its wiring, THEN canonicalize
    // via the dataset path. sparq-core's loader is the user-facing entry; we
    // assert it accepts the document, and use oxjsonld for the quad-level
    // dataset (which preserves named graphs) used in the comparison.
    let _accepted = Graph::load_str_with_base(text, "jsonld", base)
        .or_else(|_| Graph::load_dataset(text, "jsonld"))?;
    jsonld_to_canonical_dataset(text, base)
}

/// [OPUS-4.8] sq-3uos5 — serialise an oxrdf [`Dataset`] to N-Quads text. The
/// `compact` input documents are JSON-LD; to drive sparq's RDF→compacted-JSON-LD
/// writer (which takes a [`Graph`]) the input is first parsed to RDF (via the
/// REAL oxjsonld path) and re-emitted as N-Quads, then loaded into a `Graph`
/// preserving named graphs — exactly the bridge `from_rdf::run_fromrdf` uses, but
/// with the input coming from a JSON-LD document rather than an `.nq` fixture.
/// `oxrdf`'s `Quad` Display is canonical N-Quads, so this is loss-free.
pub fn dataset_to_nquads(ds: &Dataset) -> String {
    let mut out = String::new();
    for q in ds.iter() {
        let owned: Quad = q.into_owned();
        // [OPUS-4.8] positional format arg (avoids the CodeQL rust/unused-variable
        // false positive on inline-captured identifiers).
        out.push_str(&format!("{} .\n", owned));
    }
    out
}


// ── JSON-LD document-level equality comparator ──────────────────────────────
//
// [SONNET-4.6] sq-kk1mq — the comparator for the native expand() oracle.
//
// Semantics (PINNED IN CODE per sq-kk1mq):
//   • Object key order: insignificant.  Two objects are equal iff they have the
//     same keys with deeply-equal values.
//   • Array element order: SIGNIFICANT only when the array is the **direct value
//     of a `"@list"` key**; insignificant everywhere else (multiset / set
//     semantics).  In expanded JSON-LD `@list` is the only structured-sequence
//     construct; all other arrays are bags.
//   • Numbers: integral (i64/u64-representable) compared exactly so integers
//     ≥ 2^53 remain distinct; non-integral (either side is a JSON float) fall
//     back to f64, so `1` ≡ `1.0`.  [SONNET-4.6] sq-kk1mq numeric-guard.
//   • Strings, booleans, null: exact equality.
//
// Why document-level rather than RDF-level?  The expansion algorithm operates
// purely at the JSON-LD document level.  Two expansions can produce the same
// RDF yet differ in JSON structure (e.g. `@direction` handling, `@json` literals,
// `@list` vs plain-array shapes) — the RDF oracle missed those differences.
//
// PRECISION NOTE: this comparator measures JSON-LD data-model (semantic)
// equivalence — order-insensitive outside `@list` — NOT structural identity
// with the reference output.  ~18 of 240 passes are semantically-equal-but-
// reordered vs. the W3C reference (strict-ordered count 222).  Tracked as
// part of bead sq-kk1mq.

/// Returns `true` iff `a` and `b` are equal under the JSON-LD comparison rules
/// described in the module-level comment.
///
/// [SONNET-4.6] sq-kk1mq
pub fn json_ld_equal(a: &Value, b: &Value) -> bool {
    json_ld_equal_inner(a, b, false)
}

/// Recursive inner: `in_list` means the IMMEDIATE parent is a `"@list"` key, so
/// this value (an array) must be compared in ORDER.
fn json_ld_equal_inner(a: &Value, b: &Value, in_list: bool) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => {
            // JSON-LD numeric equality [SONNET-4.6] sq-kk1mq numeric-guard:
            // • Both i64-representable: compare exactly so integers ≥ 2^53 are
            //   not wrongly collapsed by f64 rounding
            //   (e.g. 9007199254740992 ≢ 9007199254740993 must remain distinct).
            // • Both u64-representable (one side exceeds i64::MAX): compare exactly.
            // • Otherwise: fall back to f64 so integer JSON `1` ≡ float JSON `1.0`.
            if let (Some(xi), Some(yi)) = (x.as_i64(), y.as_i64()) {
                xi == yi
            } else if let (Some(xu), Some(yu)) = (x.as_u64(), y.as_u64()) {
                xu == yu
            } else {
                match (x.as_f64(), y.as_f64()) {
                    (Some(xf), Some(yf)) => xf == yf,
                    // Fallback for numbers outside f64 range (unlikely in JSON-LD).
                    _ => x == y,
                }
            }
        }
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(xs), Value::Array(ys)) => {
            if xs.len() != ys.len() {
                return false;
            }
            if in_list {
                // @list: element order is SIGNIFICANT.
                xs.iter()
                    .zip(ys.iter())
                    .all(|(x, y)| json_ld_equal_inner(x, y, false))
            } else {
                // Set semantics: order INSIGNIFICANT (multiset match).
                json_ld_array_equal_unordered(xs, ys)
            }
        }
        (Value::Object(xa), Value::Object(ya)) => {
            if xa.len() != ya.len() {
                return false;
            }
            for (k, va) in xa {
                let Some(vb) = ya.get(k) else {
                    return false;
                };
                // The array VALUE of a "@list" key is ORDER-SIGNIFICANT.
                let child_in_list = k == "@list";
                if !json_ld_equal_inner(va, vb, child_in_list) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Multiset-equality for arrays that are NOT inside a `@list` value: every element
/// of `xs` must match exactly one unused element of `ys`, regardless of position.
/// O(n²) but conformance-suite arrays are small (typically ≤ 10 elements).
fn json_ld_array_equal_unordered(xs: &[Value], ys: &[Value]) -> bool {
    let mut used = vec![false; ys.len()];
    'outer: for x in xs {
        for (j, y) in ys.iter().enumerate() {
            if !used[j] && json_ld_equal_inner(x, y, false) {
                used[j] = true;
                continue 'outer;
            }
        }
        return false;
    }
    true
}

/// Convert a `sparq_jsonld::Json` value to a `serde_json::Value` by
/// round-tripping through a JSON string.  Used to bridge the two ASTs so the
/// `json_ld_equal` comparator can operate on both the expand() output and the
/// expected-document JSON that `serde_json::from_str` produces.
pub fn sparq_json_to_serde(j: &sparq_jsonld::Json) -> Result<Value, String> {
    let mut buf = String::new();
    j.write(&mut buf);
    serde_json::from_str(&buf).map_err(|e| format!("parse serialized JSON as serde_json::Value: {}", e))
}

/// The known-gap categories: present in the W3C suite but NOT a sparq-shipped
/// gateable surface yet. Reported as not-implemented (never failed). The size
/// is read from each manifest so the scoreboard shows the real backlog and
/// shrinks honestly as categories light up.
///
/// [OPUS-4.8] sq-3uos5 — `compact` GRADUATED out of this bucket: it is now a
/// gated category (`compact::run_compact`, `COMPACT_FLOOR`). [OPUS-4.8] sq-oy1f.19 —
/// `frame` likewise GRADUATED. [OPUS-4.8] sq-oy1f — `expand` + `flatten` now
/// GRADUATED too: each is a gated category (`flatten::run_expand_or_flatten` /
/// `expand::run_expand_native`, `EXPAND_FLOOR` / `FLATTEN_FLOOR`).
pub const NOT_IMPLEMENTED_CATS: &[(&str, &str)] = &[
    ("html", "HTML script extraction not implemented"),
    (
        "remote-doc",
        "remote @context loader not wired (oxjsonld needs a LoadDocumentCallback)",
    ),
];

pub fn not_implemented_counts(root: &Path) -> BTreeMap<&'static str, usize> {
    let mut m = BTreeMap::new();
    for (cat, _why) in NOT_IMPLEMENTED_CATS {
        let n = read_manifest(root, cat).map(|e| e.len()).unwrap_or(0);
        m.insert(*cat, n);
    }
    m
}

#[cfg(test)]
mod comparator_tests {
    use super::*;
    use serde_json::json;

    // ── json_ld_equal unit tests (sq-kk1mq) ─────────────────────────────

    /// Arrays outside @list are unordered (set semantics).
    #[test]
    fn arrays_outside_list_are_unordered() {
        let a = json!([1, 2, 3]);
        let b = json!([3, 1, 2]);
        assert!(json_ld_equal(&a, &b), "permuted array outside @list must be equal");
    }

    /// Arrays outside @list with different elements are not equal.
    #[test]
    fn arrays_outside_list_different_elements() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 4]);
        assert!(!json_ld_equal(&a, &b));
    }

    /// Arrays that are the VALUE of a "@list" key are ORDER-SIGNIFICANT.
    #[test]
    fn array_inside_list_is_ordered_fail() {
        let a = json!({"@list": [1, 2, 3]});
        let b = json!({"@list": [3, 1, 2]});
        assert!(!json_ld_equal(&a, &b), "permuted @list must NOT be equal");
    }

    /// Arrays that are the VALUE of a "@list" key with the same order pass.
    #[test]
    fn array_inside_list_same_order_passes() {
        let a = json!({"@list": [1, 2, 3]});
        let b = json!({"@list": [1, 2, 3]});
        assert!(json_ld_equal(&a, &b));
    }

    /// Outer array is unordered, but each inner @list is ordered.
    #[test]
    fn nested_outer_unordered_inner_list_ordered() {
        // Two @list objects in different outer-array positions → equal (outer unordered).
        let a = json!([{"@list": [1, 2]}, {"@list": [3, 4]}]);
        let b = json!([{"@list": [3, 4]}, {"@list": [1, 2]}]);
        assert!(json_ld_equal(&a, &b), "outer array unordered");

        // But the @list contents themselves are ordered.
        let c = json!({"@list": [2, 1]});
        let d = json!({"@list": [1, 2]});
        assert!(!json_ld_equal(&c, &d), "@list contents are ordered");
    }

    /// Deeply nested @list: an @list inside an outer @list is still ordered.
    #[test]
    fn nested_list_within_list_is_ordered() {
        // value arrays inside @list elements: those inner arrays are NOT @list
        // values, so they are unordered.
        let a = json!({"@list": [{"foo": [1, 2]}, {"foo": [3, 4]}]});
        let b = json!({"@list": [{"foo": [2, 1]}, {"foo": [4, 3]}]});
        // @list order matters (outer), but "foo" arrays inside are unordered.
        assert!(json_ld_equal(&a, &b));
    }

    /// Object key order is insignificant.
    #[test]
    fn object_key_order_insignificant() {
        let a = json!({"@id": "http://example.org/a", "@type": ["http://example.org/T"]});
        let b = json!({"@type": ["http://example.org/T"], "@id": "http://example.org/a"});
        assert!(json_ld_equal(&a, &b));
    }

    /// Numeric equality: JSON integer `1` equals JSON float `1.0`.
    #[test]
    fn numeric_equality_int_float() {
        let a: Value = serde_json::from_str("1").unwrap();
        let b: Value = serde_json::from_str("1.0").unwrap();
        assert!(
            json_ld_equal(&a, &b),
            "1 and 1.0 must be equal under f64 numeric comparison"
        );
    }

    /// Different numeric values are not equal.
    #[test]
    fn numeric_inequality() {
        let a = json!(1);
        let b = json!(2);
        assert!(!json_ld_equal(&a, &b));
    }

    /// Large integers ≥ 2^53 that are distinct as i64 must not be collapsed by f64
    /// rounding.  Under the old f64-only path 9007199254740992 and 9007199254740993
    /// round to the same f64 and would be wrongly equal.  The numeric-guard fix
    /// [SONNET-4.6] sq-kk1mq compares integral values exactly via i64/u64.
    #[test]
    fn large_integer_numeric_guard() {
        // 2^53 and 2^53+1 are distinct i64 values but collapse to the same f64.
        let a: Value = serde_json::from_str("9007199254740992").unwrap();
        let b: Value = serde_json::from_str("9007199254740993").unwrap();
        assert!(
            !json_ld_equal(&a, &b),
            "2^53 and 2^53+1 must be UNEQUAL under exact i64 comparison"
        );
    }

    /// Integer JSON `1` equals float JSON `1.0` (f64 fallback when one side
    /// is non-integral; already covered by numeric_equality_int_float but
    /// kept as an explicit guard for the mixed-representation case).
    #[test]
    fn integer_equals_float_one() {
        let a: Value = serde_json::from_str("1").unwrap();
        let b: Value = serde_json::from_str("1.0").unwrap();
        assert!(
            json_ld_equal(&a, &b),
            "integer JSON 1 and float JSON 1.0 must be EQUAL"
        );
    }

    /// Duplicate elements in unordered arrays use multiset semantics.
    #[test]
    fn unordered_array_multiset_duplicates() {
        let a = json!([1, 1, 2]);
        let b = json!([1, 2, 1]);
        assert!(json_ld_equal(&a, &b), "multiset: [1,1,2] == [1,2,1]");

        let c = json!([1, 1, 2]);
        let d = json!([1, 2, 2]);
        assert!(!json_ld_equal(&c, &d), "multiset: [1,1,2] != [1,2,2]");
    }

    /// Type mismatch: null != false, string != number.
    #[test]
    fn type_mismatch_not_equal() {
        assert!(!json_ld_equal(&json!(null), &json!(false)));
        assert!(!json_ld_equal(&json!("1"), &json!(1)));
        assert!(!json_ld_equal(&json!([]), &json!({})));
    }
}
