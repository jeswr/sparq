//! [OPUS-4.8] sq-oy1f.2 — manifest-driven runner for the official W3C JSON-LD
//! 1.1 API test suite (`w3c/json-ld-api`, `tests/`), wired as a RATCHETED
//! conformance gate that mirrors the SPARQL / SHACL / GeoSPARQL / Solid ratchets
//! in this crate (crate-local `cargo test` + a pinned pass-count FLOOR that may
//! only RISE, registered in the central `scoreboard::SUITES` and guarded by
//! `tests/scoreboard_floors.rs` — the textual floor-sync guard that pins each
//! registry `ratchet_floor` to the `const … FLOOR` the runner asserts here).
//!
//! ## What is gated (v1) — only what sparq runs TODAY
//!
//! * **toRdf** (JSON-LD → RDF): each `jld:ToRDFTest` input is driven through the
//!   REAL parse path — `sparq_core::Graph::load_*` over `oxjsonld` (the `jsonld`
//!   feature). The produced RDF is compared, by blank-node-canonical dataset
//!   isomorphism, against the suite's expected N-Quads. Negative tests
//!   (`jld:NegativeEvaluationTest`) pass when the parse fails.
//! * **fromRdf** (RDF → JSON-LD): each `jld:FromRDFTest` input `.nq` is driven
//!   through the REAL write path — `sparq_engine::serialize::graph_to_jsonld`
//!   (the `serialize-rdf` feature) in both **expanded** and prefix-`@context`
//!   ("compacted") forms — then that JSON-LD is RE-PARSED through `oxjsonld` and
//!   compared, by the same dataset isomorphism, against the input dataset. The
//!   suite's `expect.jsonld` is one valid layout; ours differs syntactically but
//!   must encode the SAME RDF — so the load-bearing invariant is the round-trip
//!   `reparse(write(D)) ≡ D`, not byte equality.
//! * **compact** (RDF → compacted JSON-LD) — [OPUS-4.8] sq-3uos5: each
//!   `jld:CompactTest` input is parsed to RDF (the real oxjsonld path), compacted
//!   against the case `@context` through the native hand-rolled **Compaction
//!   Algorithm** — `sparq_engine::serialize::graph_to_jsonld_compact` (the
//!   `serialize-rdf` feature) — then that compacted document is RE-PARSED through
//!   `oxjsonld` and required to reconstruct the SAME RDF dataset:
//!   `reparse(compact(D, ctx)) ≡ D`. sparq compacts RDF, not arbitrary documents,
//!   so the case input is first reduced to its RDF; the suite's `expect.jsonld`
//!   layout is NOT compared byte-wise (ours differs) — the load-bearing invariant
//!   is **lossless compaction**. See `run_compact` for the honest below-floor
//!   divergences + the SKIP buckets (negatives sparq does not raise,
//!   JSON-LD-1.0-only, non-inline/remote `@context`, empty-RDF inputs).
//!
//! ### Oracle caveat (the same one toRdf/fromRdf carry)
//!
//! The comparison oracle is **oxjsonld self-reparse equivalence** — sparq's output
//! is read back by the SAME parser that produced the input's RDF. A compact case
//! where sparq's `@reverse` compaction double-inverts vs a strict third-party
//! processor (pyld), or a non-string `@language`/`@none` value, can therefore PASS
//! here if our own re-parse round-trips it, even though pyld would read it
//! inverted. Strict third-party (pyld) faithfulness for `@reverse` /
//! non-string-language shapes is NOT claimed by this ratchet and is tracked
//! separately (a child of sq-oy1f).
//!
//! * **frame** (RDF → framed JSON-LD) — [OPUS-4.8] sq-oy1f.19: each
//!   `jld:FrameTest` from the SEPARATE `w3c/json-ld-framing` suite (an arbitrary
//!   EXPANDED JSON-LD input) is parsed to RDF (the real oxjsonld path), framed
//!   against the case frame document through the native **Framing Algorithm** —
//!   `sparq_engine::serialize::graph_to_jsonld_framed` (the `serialize-rdf`
//!   feature) — then re-parsed and required to reconstruct the SAME RDF dataset as
//!   the suite's NORMATIVE expected output: `reparse(frame(D, F)) ≡ reparse(expected)`.
//!   Framing is a SELECT + RESHAPE (it prunes/fills/drops), so the oracle anchors on
//!   the expected document, NOT the input. See `run_frame` for the SKIP buckets (the
//!   3 frame-validation negatives sparq's TOTAL framer does not raise).
//!
//! ## Honest known-gap buckets (NOT failed, recorded as not-implemented)
//!
//! `expand`, `flatten`, `html`, and `remote-doc` are the algorithm categories sparq
//! does **not** yet ship as gateable surfaces: expand/flatten as *output* algorithms
//! are subsumed by the writer but have no W3C expected-document comparison here yet;
//! html/remote-doc need an HTML extractor / a remote `@context` loader. (`compact`
//! GRADUATED out of this bucket under sq-3uos5, and `frame` under sq-oy1f.19 — both
//! are now gated categories above.) These categories are reported in a separate
//! **not-implemented** column of the scoreboard and DO NOT fail the gate, so the
//! ratchet measures only what is shipped and GROWS as those land. The scoreboard
//! prints them honestly as not-implemented — it does not inflate the pass count.
//!
//! ## Feature gating (both states)
//!
//! The whole lane is behind this crate's opt-in `jsonld-suite` feature
//! (forwards to `sparq-core/jsonld` + `sparq-engine/serialize-rdf`). With the
//! feature OFF this file compiles to a single self-SKIP `#[test]` (no oxjsonld /
//! writer code links), so the default `cargo test -p sparq-conformance` and the
//! `--workspace` shards stay green and lean. With it ON the runner executes and
//! asserts the pinned floors. The toRdf/fromRdf/compact fixtures are fetched by
//! `scripts/fetch-jsonld-tests.sh` into the gitignored `tests/w3c/json-ld-api/`,
//! and the frame fixtures by `scripts/fetch-jsonld-framing-tests.sh` into
//! `tests/w3c/json-ld-framing/`; when either is absent the runner SKIPS that lane
//! so a fresh offline checkout stays green.
//!
//! Manifest-walking helpers are modelled on this crate's SPARQL machinery and on
//! `sparq-shacl`'s W3C runner (copied, not shared). Comparison uses oxrdf's
//! blank-node canonicalization (`Dataset::canonicalize`), never line-by-line.

// [OPUS-4.8] When the lane feature is OFF the runner is a single self-SKIP test
// so the default + `--workspace` builds neither link oxjsonld/the writer nor go
// red on a fresh checkout. (cfg gate, not a runtime branch, so zero JSON-LD code
// compiles in the default state — the lean-core invariant.)
#[cfg(not(feature = "jsonld-suite"))]
#[test]
fn jsonld_suite_skipped_without_feature() {
    eprintln!(
        "SKIP: W3C JSON-LD conformance lane is OFF — build with \
         `--features jsonld-suite` (and run scripts/fetch-jsonld-tests.sh) to run it."
    );
}

#[cfg(feature = "jsonld-suite")]
mod gated {
    use oxrdf::dataset::CanonicalizationAlgorithm;
    use oxrdf::{Dataset, Quad};
    use serde_json::Value;
    use sparq_core::Graph;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    // ---- Floors (the RATCHET). Calibrated against the pinned suite revisions in
    // scripts/fetch-jsonld-tests.sh (toRdf/fromRdf/compact) + scripts/
    // fetch-jsonld-framing-tests.sh (frame); MIRRORED in the central scoreboard
    // (scoreboard::SUITES) and read textually by the guard test
    // tests/scoreboard_floors.rs. They may only RISE — never lower them (raise as
    // oxjsonld coverage / the native writer improve). These are the ACTUAL
    // current pass counts at the pinned revisions, not aspirational targets.
    // The `tests/scoreboard_floors.rs` floor-sync guard reads these `const … : usize
    // = N;` lines textually (the same shape as SHACL's `BASELINE_PASS`), so the
    // central scoreboard can never silently drift from what this runner asserts.

    /// toRdf (JSON-LD → RDF via oxjsonld) pass floor. RATCHET: may only RISE.
    /// Measured 413/467 at the pinned revision (the rest are honest divergences:
    /// remote/`@import` `@context` URLs — oxjsonld needs a LoadDocumentCallback —
    /// `expandContext`/`rdfDirection` options sparq does not apply, base
    /// dot-segment normalization edge cases, and a handful of negative tests
    /// oxjsonld accepts leniently; see the runner doc-comment + README).
    pub const TORDF_FLOOR: usize = 413;
    /// fromRdf (RDF → JSON-LD via the native writer, round-trip) pass floor.
    /// RATCHET: may only RISE. Measured 51/53 at the pinned revision (the two
    /// failures are lists whose cells are shared across graphs, which the
    /// writer's `@list` collapsing renames).
    pub const FROMRDF_FLOOR: usize = 51;
    /// [OPUS-4.8] sq-3uos5 — compact (RDF → compacted JSON-LD via the native
    /// hand-rolled Compaction Algorithm, round-trip) pass floor. RATCHET: may
    /// only RISE. This is the MEASURED pass count at the pinned revision — the
    /// number of `jld:CompactTest` cases for which compacting the input's RDF
    /// against the case `@context` and re-parsing the compacted document
    /// reconstructs the SAME RDF dataset (`reparse(compact(D, ctx)) ≡ D`). It is
    /// NOT the suite total: many cases exercise input-document compaction
    /// features sparq's fromRdf-then-compact writer does not target (scoped/typed
    /// contexts, `@nest`, `@index`/`@id` maps the writer never emits, `@protected`
    /// redefinition errors, processing-mode error-raising), and those are SKIPPED
    /// (not failed) — see `run_compact` for the honest skip buckets.
    ///
    /// [OPUS-4.8] sq-oy1f.16 — RAISED 163 → 186 after the compaction faithfulness
    /// fixes (#978, sq-oy1f.12/.13/.14: `@reverse` double-invert, non-string
    /// `@language`/`@none`, `@type:@id`-coerced-key-vs-plain-string IRI confusion,
    /// container round-trips) landed on main. Re-MEASURED on current main at the
    /// pinned revision: compact 186 pass / 35 fail / 25 skip (was 163/58/25). The
    /// +23 FAIL→PASS are the cases those writer fixes made lossless. The 35
    /// remaining failures are real writer gaps below the floor (scoped/typed
    /// contexts, `@nest`, `@index`/`@id` map shapes the writer does not emit),
    /// tracked for a future RISE; the 25 SKIP are unchanged (negatives sparq does
    /// not raise, JSON-LD-1.0-only, non-inline/remote/multi `@context`, empty-RDF).
    pub const COMPACT_FLOOR: usize = 186;

    /// [OPUS-4.8] sq-oy1f.19 — `frame` (RDF → framed+compacted JSON-LD via the
    /// native hand-rolled Framing Algorithm) pass floor over the SEPARATE
    /// `w3c/json-ld-framing` suite (`scripts/fetch-jsonld-framing-tests.sh`).
    /// RATCHET: may only RISE. This is the MEASURED pass count at the pinned
    /// framing-suite revision — the number of `jld:FrameTest` cases for which
    /// framing the input's RDF against the case frame document and re-parsing the
    /// framed output reconstructs the SAME RDF dataset as re-parsing the suite's
    /// NORMATIVE expected output (`reparse(frame(D, F)) ≡ reparse(expected)`).
    ///
    /// ## Why compare against `expect`, not the input
    ///
    /// Framing is a SELECT + RESHAPE, not a lossless transform: `@explicit` prunes
    /// properties, an unmatched frame yields an empty `@graph`, `@default` fills a
    /// value not in the input. So `reparse(frame(D)) ≡ D` is the WRONG oracle (the
    /// framed RDF legitimately differs from the input). The normative answer is the
    /// suite's `*-out.jsonld`; sparq must produce the SAME RDF as that expected
    /// document. Comparing the two as canonical RDF datasets is envelope-insensitive
    /// (both the bare-node `omitGraph` collapse and the `{"@graph":[…]}` envelope
    /// re-parse to the same triples) and value-faithful, while NOT requiring sparq's
    /// JSON layout to match pyld byte-for-byte (it does not — the same posture as
    /// the toRdf/fromRdf/compact lanes).
    ///
    /// ## Honest SKIP buckets (recorded, not passed, not failed)
    ///
    /// * NegativeEvaluationTests (`expectErrorCode`) — sparq's framer is TOTAL; it
    ///   never raises the spec's frame-validation errors (`invalid frame`, out-of-
    ///   range `@embed`). A 1.1 framer that does not model those errors cannot
    ///   honestly "pass" by rejecting, so these are SKIPPED (the compact-lane posture).
    /// * A positive case with no `expect`, a non-object `frame`, an `input` the real
    ///   oxjsonld path rejects, or a remote `input`/`frame` URL — out of the gated
    ///   surface (SKIP, never a counted pass).
    ///
    /// Note: the `ordered` option (1 case) governs JSON-array element ORDER, a
    /// concern the RDF-level oracle is blind to by design — such a case is still
    /// gated on RDF equality and passes iff the framed RDF matches the expected RDF,
    /// the honest verdict an RDF oracle can give.
    ///
    /// MEASURED 61/92 at the pinned framing-suite revision: 61 normative
    /// RDF-equivalent frames, 28 honest framer divergences (value-pattern matching
    /// over `@value` alternative arrays, `@explicit`/`@default` fill differences,
    /// named-graph `@graph` framing shapes, `@list`/`@set` re-emit, blank-node
    /// `@embed` table edge cases), and 3 SKIP (the suite's 3 NegativeEvaluationTests
    /// — sparq's TOTAL framer does not raise the spec's frame-validation errors).
    /// The 28 divergences are real framer gaps below the floor (tracked for a future
    /// RISE — a child of sq-oy1f); SKIP is reserved for genuinely-unsupported cases,
    /// never used to hide a divergence.
    pub const FRAME_FLOOR: usize = 61;

    /// [OPUS-4.8] sq-oy1f.19 — the framing suite's declared base (its
    /// `baseIri`), used to resolve each frame test's input path into the document
    /// IRI. Distinct from `SUITE_BASE` (the json-ld-api base) — framing is a
    /// separate W3C repo.
    const FRAME_SUITE_BASE: &str = "https://w3c.github.io/json-ld-framing/tests/";

    /// The suite's declared base for resolving each test's input path into the
    /// document IRI (the toRdf base when `option.base` is absent).
    const SUITE_BASE: &str = "https://w3c.github.io/json-ld-api/tests/";

    fn suite_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = crates/sparq-conformance; the workspace root is two up.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/w3c/json-ld-api/tests")
    }

    /// [OPUS-4.8] sq-oy1f.19 — the SEPARATE `w3c/json-ld-framing` suite root
    /// (`scripts/fetch-jsonld-framing-tests.sh`). Framing lives in its own W3C
    /// repo, not under `json-ld-api`.
    fn frame_suite_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/w3c/json-ld-framing/tests")
    }

    #[derive(Default)]
    struct Score {
        pass: usize,
        fail: usize,
        skip: usize,
        failures: Vec<(String, String)>,
    }

    impl Score {
        fn pass(&mut self) {
            self.pass += 1;
        }
        fn fail(&mut self, id: &str, why: String) {
            self.fail += 1;
            self.failures.push((id.to_string(), why));
        }
        fn skip(&mut self) {
            self.skip += 1;
        }
    }

    /// Parse an N-Quads file into a canonicalized oxrdf [`Dataset`] for isomorphic
    /// comparison (blank-node-blind).
    fn nquads_to_canonical_dataset(text: &str) -> Result<Dataset, String> {
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
    fn jsonld_to_canonical_dataset(doc: &str, base: &str) -> Result<Dataset, String> {
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

    // ---- The two gated categories -------------------------------------------

    /// One parsed manifest entry's salient fields.
    struct Entry {
        id: String,
        is_negative: bool,
        input: String,
        expect: Option<String>,
        base_override: Option<String>,
        /// An optional-feature requirement (`GeneralizedRdf` / `I18nDatatype` /
        /// `CompoundLiteral`); sparq does not opt into these, so such entries are
        /// SKIPPED (not failed) — they are out of the current gated surface.
        requires: Option<String>,
        /// [OPUS-4.8] sq-3uos5 — the `compact` manifest's top-level `context`
        /// member: a path (relative to the suite root) to the sibling
        /// `*-context.jsonld` file holding the caller `@context` to compact
        /// against. (The toRdf/fromRdf manifests have no such member; it stays
        /// `None` there.)
        context: Option<String>,
        /// [OPUS-4.8] sq-3uos5 — `option.specVersion` (`json-ld-1.0`/`-1.1`) and
        /// `option.processingMode`, recorded so the compact runner can SKIP the
        /// JSON-LD-1.0-only error-raising negatives sparq's 1.1 writer does not
        /// model (it is not a faithful pass for a 1.1 processor to "reject" them).
        spec_version: Option<String>,
        processing_mode: Option<String>,
        /// [OPUS-4.8] sq-oy1f.19 — the `frame` manifest's top-level `frame`
        /// member: a path (relative to the suite root) to the sibling
        /// `*-frame.jsonld` document (the frame pattern + its `@context`) to frame
        /// the input against. `None` for the toRdf/fromRdf/compact manifests.
        frame: Option<String>,
        /// [OPUS-4.8] sq-oy1f.19 — a NegativeEvaluationTest's expected JSON-LD
        /// error code (`invalid frame`, `invalid @embed value`, …). Recorded so the
        /// frame runner can SKIP the error-raising negatives sparq's TOTAL framer
        /// does not model (it never raises the spec's frame-validation errors), the
        /// same honesty posture the compact lane takes toward its negatives.
        expect_error_code: Option<String>,
    }

    /// Read `<cat>-manifest.jsonld` and return its `sequence` entries.
    fn read_manifest(root: &Path, cat: &str) -> Result<Vec<Entry>, String> {
        let path = root.join(format!("{}-manifest.jsonld", cat));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| format!("parse {}: {e}", path.display()))?;
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
            let expect = e
                .get("expect")
                .and_then(Value::as_str)
                .map(str::to_string);
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
            let context = e
                .get("context")
                .and_then(Value::as_str)
                .map(str::to_string);
            let spec_version = opt
                .and_then(|o| o.get("specVersion"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let processing_mode = opt
                .and_then(|o| o.get("processingMode"))
                .and_then(Value::as_str)
                .map(str::to_string);
            // [OPUS-4.8] sq-oy1f.19 — frame-only manifest members (None elsewhere).
            let frame = e
                .get("frame")
                .and_then(Value::as_str)
                .map(str::to_string);
            let expect_error_code = e
                .get("expectErrorCode")
                .and_then(Value::as_str)
                .map(str::to_string);
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
            });
        }
        Ok(out)
    }

    /// The document IRI / base for a toRdf test: `option.base` if given, else the
    /// suite base joined with the input path.
    fn doc_base(e: &Entry) -> String {
        e.base_override
            .clone()
            .unwrap_or_else(|| format!("{}{}", SUITE_BASE, e.input))
    }

    /// Run the toRdf category. Returns the scoreboard.
    fn run_tordf(root: &Path) -> Score {
        let mut s = Score::default();
        let entries = match read_manifest(root, "toRdf") {
            Ok(e) => e,
            Err(why) => {
                s.fail("toRdf-manifest", why);
                return s;
            }
        };
        for e in &entries {
            // Skip optional-feature tests sparq does not opt into (see `requires`).
            if e.requires.is_some() {
                s.skip();
                continue;
            }
            let input_path = root.join(&e.input);
            let text = match std::fs::read_to_string(&input_path) {
                Ok(t) => t,
                Err(why) => {
                    s.fail(&e.id, format!("read input: {why}"));
                    continue;
                }
            };
            let base = doc_base(e);
            // [OPUS-4.8] drive the REAL ingest path: load through sparq-core /
            // oxjsonld, then read the parsed quads back out for comparison.
            let parsed = parse_jsonld_dataset(&text, &base);
            if e.is_negative {
                // A NegativeEvaluationTest passes iff the parse fails (sparq
                // rejects the malformed/illegal document).
                match parsed {
                    Err(_) => s.pass(),
                    Ok(_) => s.fail(&e.id, "negative test parsed without error".into()),
                }
                continue;
            }
            let Some(expect) = &e.expect else {
                s.skip();
                continue;
            };
            let got = match parsed {
                Ok(ds) => ds,
                Err(why) => {
                    s.fail(&e.id, format!("parse: {why}"));
                    continue;
                }
            };
            let exp_text = match std::fs::read_to_string(root.join(expect)) {
                Ok(t) => t,
                Err(why) => {
                    s.fail(&e.id, format!("read expect: {why}"));
                    continue;
                }
            };
            let want = match nquads_to_canonical_dataset(&exp_text) {
                Ok(ds) => ds,
                Err(why) => {
                    s.fail(&e.id, format!("parse expect: {why}"));
                    continue;
                }
            };
            if got == want {
                s.pass();
            } else {
                s.fail(
                    &e.id,
                    format!("RDF mismatch ({} vs {} quads)", got.len(), want.len()),
                );
            }
        }
        s
    }

    /// Parse a JSON-LD document through the REAL sparq-core ingest path (oxjsonld
    /// behind the `jsonld` feature) into a canonicalized dataset. Uses
    /// `oxjsonld` directly here (the same parser sparq-core calls) with the test's
    /// base so the comparison is base-faithful.
    fn parse_jsonld_dataset(text: &str, base: &str) -> Result<Dataset, String> {
        // Round-trip through sparq-core to exercise its wiring, THEN canonicalize
        // via the dataset path. sparq-core's loader is the user-facing entry; we
        // assert it accepts the document, and use oxjsonld for the quad-level
        // dataset (which preserves named graphs) used in the comparison.
        let _accepted = Graph::load_str_with_base(text, "jsonld", base)
            .or_else(|_| Graph::load_dataset(text, "jsonld"))?;
        jsonld_to_canonical_dataset(text, base)
    }

    /// Run the fromRdf category through the native writer + a re-parse round-trip.
    fn run_fromrdf(root: &Path) -> Score {
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
                if let Err(why) = serde_json::from_str::<Value>(&doc) {
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

    /// [OPUS-4.8] sq-3uos5 — serialise an oxrdf [`Dataset`] to N-Quads text. The
    /// `compact` input documents are JSON-LD; to drive sparq's RDF→compacted-JSON-LD
    /// writer (which takes a [`Graph`]) the input is first parsed to RDF (via the
    /// REAL oxjsonld path) and re-emitted as N-Quads, then loaded into a `Graph`
    /// preserving named graphs — exactly the bridge `run_fromrdf` uses, but with the
    /// input coming from a JSON-LD document rather than an `.nq` fixture. `oxrdf`'s
    /// `Quad` Display is canonical N-Quads, so this is loss-free.
    fn dataset_to_nquads(ds: &Dataset) -> String {
        let mut out = String::new();
        for q in ds.iter() {
            let owned: Quad = q.into_owned();
            // [OPUS-4.8] positional format arg (avoids the CodeQL rust/unused-variable
            // false positive on inline-captured identifiers).
            out.push_str(&format!("{} .\n", owned));
        }
        out
    }

    /// [OPUS-4.8] sq-3uos5 — read a `compact` test's context file and extract the
    /// caller `@context` value as the writer's [`JsonLdValue`]. The suite's
    /// `*-context.jsonld` files wrap the context in `{"@context": …}`; sparq's
    /// `ActiveContext::parse` (and so `graph_to_jsonld_compact`) expects the INNER
    /// value (the term-definition object), so we unwrap one `@context` layer. When
    /// the inner value is an object we hand it straight to the writer; an
    /// array/string form (a remote-context reference or multi-context array) is NOT
    /// resolved here (no network, and the writer takes a single inline object) — the
    /// caller SKIPS such cases.
    fn read_context_member(path: &Path) -> Result<sparq_engine::serialize::JsonLdValue, String> {
        use sparq_engine::serialize::{parse_context_json, JsonLdValue};
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("read context {}: {e}", path.display()))?;
        // Parse the whole context document with the writer's own tiny JSON reader,
        // then pull out the `@context` member (the value the writer compacts against).
        let doc = parse_context_json(&text)
            .ok_or_else(|| "context file is not a JSON object".to_string())?;
        match doc {
            JsonLdValue::Obj(members) => {
                let inner = members
                    .into_iter()
                    .find(|(k, _)| k == "@context")
                    .map(|(_, v)| v)
                    .ok_or_else(|| "context file has no @context member".to_string())?;
                // Only a single inline object context is drivable through the writer.
                match inner {
                    JsonLdValue::Obj(_) => Ok(inner),
                    _ => {
                        Err("non-object @context (array/string/remote) — not drivable".to_string())
                    }
                }
            }
            _ => Err("context document is not an object".to_string()),
        }
    }

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
    ///    w.r.t. the RDF. This is the SAME round-trip oracle `run_fromrdf` uses
    ///    (oxjsonld self-reparse equivalence), NOT a JSON-structural diff against the
    ///    suite's `expect.jsonld` (whose layout sparq's writer does not reproduce
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
    /// (a child of sq-oy1f) and is NOT claimed by this ratchet — see the runner
    /// header + the crate README.
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
    fn run_compact(root: &Path) -> Score {
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

    /// [OPUS-4.8] sq-oy1f.19 — the base/document IRI for a `frame` test: the
    /// `option.base` override if given, else the framing suite base joined with the
    /// input path. (Distinct from `doc_base`, which uses the json-ld-api base.)
    fn frame_doc_base(e: &Entry) -> String {
        e.base_override
            .clone()
            .unwrap_or_else(|| format!("{}{}", FRAME_SUITE_BASE, e.input))
    }

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
    ///    RDF and loaded into a `Graph` — exactly the bridge `run_compact` uses.
    /// 2. Read the case `frame` document (`*-frame.jsonld`: the frame pattern + its
    ///    `@context`) with the writer's own JSON reader.
    /// 3. Run `graph_to_jsonld_framed(&graph, &frame)`.
    /// 4. **Invariant (normative answer-equivalence):** re-parse BOTH sparq's framed
    ///    output AND the suite's NORMATIVE expected output (`*-out.jsonld`) to
    ///    canonical [`Dataset`]s and require they are equal —
    ///    `reparse(frame(D, F)) ≡ reparse(expected)`. Framing is a SELECT + RESHAPE
    ///    (it legitimately prunes / fills / drops), so the oracle anchors on the
    ///    W3C-expected framed document, NOT the input (see `FRAME_FLOOR`). This is
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
    fn run_frame(root: &Path) -> Score {
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

    /// The known-gap categories: present in the W3C suite but NOT a sparq-shipped
    /// gateable surface yet. Reported as not-implemented (never failed). The size
    /// is read from each manifest so the scoreboard shows the real backlog and
    /// shrinks honestly as categories light up.
    ///
    /// [OPUS-4.8] sq-3uos5 — `compact` GRADUATED out of this bucket: it is now a
    /// gated category (`run_compact`, `COMPACT_FLOOR`). [OPUS-4.8] sq-oy1f.19 —
    /// `frame` likewise GRADUATED: it is now a gated category (`run_frame`,
    /// `FRAME_FLOOR`) over the separate `w3c/json-ld-framing` suite.
    const NOT_IMPLEMENTED_CATS: &[(&str, &str)] = &[
        ("expand", "Expansion — output-vs-W3C-document comparison not wired (sq-oy1f)"),
        ("flatten", "Flattening — output-vs-W3C-document comparison not wired (sq-oy1f)"),
        ("html", "HTML script extraction not implemented"),
        ("remote-doc", "remote @context loader not wired (oxjsonld needs a LoadDocumentCallback)"),
    ];

    fn not_implemented_counts(root: &Path) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for (cat, _why) in NOT_IMPLEMENTED_CATS {
            let n = read_manifest(root, cat).map(|e| e.len()).unwrap_or(0);
            m.insert(*cat, n);
        }
        m
    }

    #[test]
    fn jsonld_conformance_ratchet() {
        let root = suite_root();
        if !root.exists() {
            eprintln!(
                "SKIP: W3C JSON-LD suite not present at {} — run scripts/fetch-jsonld-tests.sh",
                root.display()
            );
            return;
        }

        let tordf = run_tordf(&root);
        let fromrdf = run_fromrdf(&root);
        let compact = run_compact(&root);
        let not_impl = not_implemented_counts(&root);

        // [OPUS-4.8] sq-oy1f.19 — framing lives in the SEPARATE w3c/json-ld-framing
        // suite (scripts/fetch-jsonld-framing-tests.sh), which a checkout may have
        // independently of json-ld-api. Run it only when present; otherwise the
        // `frame` line reports "suite absent" and the frame ratchet is not asserted
        // (a fresh offline checkout stays green). When present, the FRAME_FLOOR
        // ratchet is asserted below.
        let frame_root = frame_suite_root();
        let frame = frame_root.exists().then(|| run_frame(&frame_root));

        println!("\nW3C JSON-LD 1.1 conformance scoreboard (pinned w3c/json-ld-api + json-ld-framing)");
        println!("{:<10} {:>5} {:>5} {:>5}", "category", "pass", "fail", "skip");
        // [OPUS-4.8] The CI ratchet greps these `TOTAL <cat>` lines.
        println!(
            "TOTAL toRdf {} {} {} (floor {})",
            tordf.pass, tordf.fail, tordf.skip, TORDF_FLOOR
        );
        println!(
            "TOTAL fromRdf {} {} {} (floor {})",
            fromrdf.pass, fromrdf.fail, fromrdf.skip, FROMRDF_FLOOR
        );
        // [OPUS-4.8] sq-3uos5 — the compact ratchet. The CI grep depends on this
        // exact `^TOTAL compact ` prefix with the pass count in field $3.
        println!(
            "TOTAL compact {} {} {} (floor {})",
            compact.pass, compact.fail, compact.skip, COMPACT_FLOOR
        );
        // [OPUS-4.8] sq-oy1f.19 — the frame ratchet. The CI grep depends on this
        // exact `^TOTAL frame ` prefix with the pass count in field $3 (same shape
        // as the other lanes). Printed only when the framing suite is present.
        if let Some(frame) = &frame {
            println!(
                "TOTAL frame {} {} {} (floor {})",
                frame.pass, frame.fail, frame.skip, FRAME_FLOOR
            );
        } else {
            println!(
                "frame      (suite absent — run scripts/fetch-jsonld-framing-tests.sh; floor {})",
                FRAME_FLOOR
            );
        }
        println!("\nknown-gap (NOT-IMPLEMENTED — not gated, grows the ratchet as they land):");
        for (cat, why) in NOT_IMPLEMENTED_CATS {
            println!("  {:<10} {:>4} tests — {}", cat, not_impl.get(cat).copied().unwrap_or(0), why);
        }

        if !tordf.failures.is_empty() {
            println!("\ntoRdf failures (first 40):");
            for (id, why) in tordf.failures.iter().take(40) {
                println!("  {}: {}", id, why);
            }
        }
        if !fromrdf.failures.is_empty() {
            println!("\nfromRdf failures (first 40):");
            for (id, why) in fromrdf.failures.iter().take(40) {
                println!("  {}: {}", id, why);
            }
        }
        if !compact.failures.is_empty() {
            println!("\ncompact failures (first 40):");
            for (id, why) in compact.failures.iter().take(40) {
                println!("  {}: {}", id, why);
            }
        }
        if let Some(frame) = &frame {
            if !frame.failures.is_empty() {
                println!("\nframe failures (first 40):");
                for (id, why) in frame.failures.iter().take(40) {
                    println!("  {}: {}", id, why);
                }
            }
        }

        // The ratchet: pass counts may only RISE. A regression below the pinned
        // floor fails the build.
        assert!(
            tordf.pass >= TORDF_FLOOR,
            "JSON-LD toRdf pass count regressed: {} < floor {} — see failures above",
            tordf.pass,
            TORDF_FLOOR
        );
        assert!(
            fromrdf.pass >= FROMRDF_FLOOR,
            "JSON-LD fromRdf pass count regressed: {} < floor {} — see failures above",
            fromrdf.pass,
            FROMRDF_FLOOR
        );
        // [OPUS-4.8] sq-3uos5 — the compact ratchet (lossless round-trip floor).
        assert!(
            compact.pass >= COMPACT_FLOOR,
            "JSON-LD compact pass count regressed: {} < floor {} — see failures above",
            compact.pass,
            COMPACT_FLOOR
        );
        // [OPUS-4.8] sq-oy1f.19 — the frame ratchet (normative answer-equivalence
        // floor), asserted only when the separate framing suite is present.
        if let Some(frame) = &frame {
            assert!(
                frame.pass >= FRAME_FLOOR,
                "JSON-LD frame pass count regressed: {} < floor {} — see failures above",
                frame.pass,
                FRAME_FLOOR
            );
        }
    }
}
