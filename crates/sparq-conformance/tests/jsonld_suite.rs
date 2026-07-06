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
    // [SONNET-4.6] sq-kk1mq — native expand() for the document-level expand oracle.
    use sparq_jsonld::{expand as jsonld_expand, JsonLdOptions, NoopLoader, ProcessingMode};
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

    /// [SONNET-4.6] sq-kk1mq (oracle-correction re-baseline; supersedes the old
    /// sq-oy1f RDF-equivalence oracle below) — `expand` pass floor over the
    /// `expand` category of `w3c/json-ld-api`, now measured with the NATIVE
    /// DOCUMENT-LEVEL oracle: `sparq_jsonld::expand(input, opts, &NoopLoader)`
    /// produces a `Json` value; that value is deep-compared against the suite's
    /// expected expanded document using [`json_ld_equal`] (object key order
    /// insignificant; array order SIGNIFICANT only inside `@list` values,
    /// insignificant elsewhere; numeric equality as f64).
    ///
    /// ## Oracle-correction rationale
    ///
    /// The OLD oracle (sq-oy1f, merged before this bead) compared the RDF dataset
    /// produced by the engine's JSON-LD WRITER (`graph_to_jsonld(Expanded)`) against
    /// the RDF produced by re-parsing the suite's expected document. That oracle
    /// measured the writer's RDF fidelity, not the expansion algorithm. A case could
    /// "pass" by coincidence (writer happened to produce equivalent RDF) even when the
    /// expansion JSON shape differed from the spec, or fail spuriously when the input
    /// had no RDF projection but the expander output was correct. The new oracle
    /// measures JSON-LD data-model (semantic) equivalence — order-insensitive outside
    /// `@list` per bead sq-kk1mq — NOT structural identity with the reference output.
    ///
    /// NOTE: ~18 of the 240 passes are semantically-equal-but-reordered vs. the W3C
    /// reference (@type order #tpr30, @id-map #tm001, @index #tpi06, multi-value
    /// #tn004/#t0030 families); a strict order-sensitive harness would fail these 18
    /// (strict-ordered count = 222).  The comparator is intentionally order-insensitive
    /// per the JSON-LD data model (elements outside `@list` are a set).
    ///
    /// This is an HONEST REBASE, not a ratchet weakening: the new floor may be lower
    /// than 247 (old oracle measured 247/385) because some old passes were oracle
    /// artefacts (the writer round-tripped to equivalent RDF while the JSON was wrong).
    /// The gain is precision: passes now mean the expander produced a semantically
    /// correct expanded document (data-model equivalence, not byte-identical structure).
    /// See `run_expand_native` for the skip buckets and the PR body for the full
    /// old-vs-new breakdown (bead sq-kk1mq).
    ///
    /// ## What changed from the old oracle
    ///
    /// * OLD (sq-oy1f, EXPAND_FLOOR = 247): RDF-equivalence —
    ///   `reparse_rdf(write_expanded(ingest_rdf(input))) ≡ reparse_rdf(expected)`.
    ///   Forwarded NO options; skipped empty-RDF inputs, 1.0-mode cases.
    /// * NEW (sq-kk1mq, EXPAND_FLOOR = measured): document-level JSON comparison —
    ///   `json_ld_equal(expand(input, opts), expected_json)`. Forwards base,
    ///   expandContext, processingMode from the manifest; attempts all positive tests
    ///   regardless of RDF projection emptiness.
    ///
    /// ## Honest SKIP buckets (recorded, not passed, not failed)
    ///
    /// * `requires` optional-feature cases — out of the gated surface (same as before).
    /// * NegativeEvaluationTests — sparq's expander raises some spec errors but
    ///   error-code completeness is unverified; deferred to a child bead of sq-oy1f.
    ///   SKIP (honest), never a counted pass.
    /// * Remote `input` URLs — no network (SKIP).
    /// * No `expect` file — nothing to compare (SKIP).
    ///
    /// ## RATCHET: may only RISE
    ///
    /// This floor is the MEASURED pass count with the new oracle at the pinned
    /// suite revision. It is NOT aspirational. Future rises land when the native
    /// expander fixes known divergences (tracked as children of sq-oy1f).
    /// MEASURED 240/385 at the pinned revision under the new document-level oracle
    /// (sq-kk1mq): 240 pass / 36 fail / 109 skip.  Breakdown vs. old oracle (247/10/128):
    ///   - 7 fewer passes (247→240): NET of 20 old-pass→new-fail flips (old oracle over-passed;
    ///     writer happened to produce equivalent RDF while JSON structure differed from spec),
    ///     offset by 13 recoveries: 8 old-fail→new-pass via oracle precision (new oracle
    ///     correctly passes cases the old oracle spuriously failed), plus 5 old-skip→new-pass
    ///     via options forwarding (processingMode/expandContext now forwarded; previously skipped)
    ///   - 26 more fails (10→36): honest divergences the new oracle reveals (JSON-level
    ///     mismatches invisible to the old RDF oracle, options-driven, or @direction shapes)
    ///   - 19 fewer skips (128→109): the old 1.0-mode and empty-RDF skip buckets are gone
    ///     (the new oracle forwards processingMode and doesn't require a non-empty RDF
    ///     projection); the 109 that remain are the NegativeEvaluationTests (deferred)
    pub const EXPAND_FLOOR: usize = 240;

    /// [OPUS-4.8] sq-oy1f — `flatten` (RDF → flattened JSON-LD via the
    /// ALREADY-SHIPPING writer `graph_to_jsonld(JsonLdForm::Flattened)`, the
    /// `serialize-rdf` feature) pass floor over the `flatten` category of
    /// `w3c/json-ld-api`. RATCHET: may only RISE. This is the MEASURED pass count
    /// at the pinned suite revision — the number of `jld:FlattenTest` cases for
    /// which flattening the input's RDF and re-parsing the produced flattened
    /// document reconstructs the SAME RDF dataset as re-parsing the suite's
    /// NORMATIVE expected flattened document (`reparse(flatten(D)) ≡ reparse(expected)`).
    /// Same oracle, SKIP buckets, and caveat as `EXPAND_FLOOR` (flattening is the
    /// node-merged normal form; the oracle anchors on the expected document, not the
    /// input). MEASURED 50/58 at the pinned revision: flatten 50 pass / 0 fail / 8
    /// skip — every flatten case the writer drives round-trips to the normative
    /// expected document; the 8 SKIP are the documented buckets (1
    /// NegativeEvaluationTest, JSON-LD-1.0-only positives, and empty-RDF inputs).
    pub const FLATTEN_FLOOR: usize = 50;

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
        /// [SONNET-4.6] sq-kk1mq — `option.expandContext`: a path (relative to the
        /// suite root) to a context file. Forwarded to the native expand() oracle as
        /// `JsonLdOptions.expand_context`; `None` for the toRdf/fromRdf/compact/frame
        /// manifests (the expand manifest is the only one that has this option).
        expand_context_path: Option<String>,
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
            // [SONNET-4.6] sq-kk1mq — expand-only manifest option: `option.expandContext`
            // is a path (relative to the suite root) to a context file.
            let expand_context_path = opt
                .and_then(|o| o.get("expandContext"))
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
                expand_context_path,
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

    /// [OPUS-4.8] sq-oy1f — run a W3C JSON-LD `flatten` category (or the superseded
    /// RDF-equivalence expand oracle) against the ALREADY-SHIPPING native writer
    /// (`graph_to_jsonld(graph, form)`, the `serialize-rdf` feature).
    ///
    /// [SONNET-4.6] sq-kk1mq — the `expand` category now uses `run_expand_native`
    /// (document-level JSON oracle via `sparq_jsonld::expand()`); this function is
    /// retained for the `flatten` category whose native algorithm is deferred.
    /// `form` = [`JsonLdForm::Flattened`] for `flatten`.
    ///
    /// ## Pipeline (the REAL writer path)
    ///
    /// 1. Parse the case `input` (`.jsonld`) → RDF via the real oxjsonld ingest path
    ///    (`parse_jsonld_dataset`).
    /// 2. Re-emit that dataset as N-Quads and load it into a sparq [`Graph`]
    ///    (preserving named graphs) — the writer takes a `Graph`, not a JSON-LD doc,
    ///    because sparq's expand/flatten OUTPUT is a projection of RDF (exactly the
    ///    bridge `run_compact` / `run_frame` use).
    /// 3. Run `graph_to_jsonld(&graph, form)` — the shipping writer, NOT a stub.
    /// 4. **Invariant (normative answer-equivalence):** re-parse BOTH sparq's output
    ///    AND the suite's NORMATIVE expected document (`*-out.jsonld`) to canonical
    ///    [`Dataset`]s and require they are equal:
    ///    `reparse(write(D, form)) ≡ reparse(expected)`. Expansion/flattening are the
    ///    JSON-LD normal forms (they drop free-floating nodes / merge nodes), so the
    ///    oracle anchors on the W3C-expected document, NOT the input — the same
    ///    posture as the frame lane. This is envelope-insensitive and value-faithful
    ///    while NOT requiring sparq's JSON layout to match byte-for-byte.
    ///
    /// ## Honest SKIP buckets (recorded, not passed, not failed) — see `EXPAND_FLOOR`
    fn run_expand_or_flatten(
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
    fn json_ld_equal(a: &Value, b: &Value) -> bool {
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

    // ── Native expand() oracle ────────────────────────────────────────────────
    //
    // [SONNET-4.6] sq-kk1mq — replaces the old RDF-equivalence oracle for the
    // `expand` category.  The flatten category KEEPS `run_expand_or_flatten`
    // (still RDF-based; a separate bead will switch it when the native flatten
    // algorithm lands).

    /// Convert a `sparq_jsonld::Json` value to a `serde_json::Value` by
    /// round-tripping through a JSON string.  Used to bridge the two ASTs so the
    /// `json_ld_equal` comparator can operate on both the expand() output and the
    /// expected-document JSON that `serde_json::from_str` produces.
    fn sparq_json_to_serde(j: &sparq_jsonld::Json) -> Result<Value, String> {
        let mut buf = String::new();
        j.write(&mut buf);
        serde_json::from_str(&buf).map_err(|e| format!("parse serialized JSON as serde_json::Value: {}", e))
    }

    /// [SONNET-4.6] sq-kk1mq — run the W3C JSON-LD `expand` category with the
    /// NATIVE DOCUMENT-LEVEL oracle: call `sparq_jsonld::expand()` directly on the
    /// input document and deep-compare the result to the suite's expected expanded
    /// document via [`json_ld_equal`].
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
    /// ## Honest SKIP buckets (recorded, not passed, not failed)
    ///
    /// * `requires` optional-feature cases (same as all other lanes).
    /// * NegativeEvaluationTests — expander error-code completeness is unverified;
    ///   deferred to a child bead of sq-oy1f.  SKIP (honest), never a counted pass.
    /// * Remote `input` URL — no network.
    /// * No `expect` file — nothing to compare.
    fn run_expand_native(root: &Path) -> Score {
        let mut s = Score::default();
        let entries = match read_manifest(root, "expand") {
            Ok(e) => e,
            Err(why) => {
                s.fail("expand-manifest", why);
                return s;
            }
        };
        for e in &entries {
            if e.requires.is_some() {
                s.skip();
                continue;
            }
            // NegativeEvaluationTests: expander raises some errors but error-code
            // completeness is unverified — SKIP honestly (deferred).
            if e.is_negative {
                s.skip();
                continue;
            }
            let Some(expect_rel) = &e.expect else {
                s.skip();
                continue;
            };
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
            let processing_mode = match (
                e.processing_mode.as_deref(),
                e.spec_version.as_deref(),
            ) {
                (Some("json-ld-1.0"), _) | (_, Some("json-ld-1.0")) => {
                    ProcessingMode::JsonLd10
                }
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

            // 3. Call the native expand() algorithm.
            let expanded = match jsonld_expand(&input_json, &opts, &NoopLoader) {
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

            // 5. Read and parse the expected document.
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

    /// The known-gap categories: present in the W3C suite but NOT a sparq-shipped
    /// gateable surface yet. Reported as not-implemented (never failed). The size
    /// is read from each manifest so the scoreboard shows the real backlog and
    /// shrinks honestly as categories light up.
    ///
    /// [OPUS-4.8] sq-3uos5 — `compact` GRADUATED out of this bucket: it is now a
    /// gated category (`run_compact`, `COMPACT_FLOOR`). [OPUS-4.8] sq-oy1f.19 —
    /// `frame` likewise GRADUATED. [OPUS-4.8] sq-oy1f — `expand` + `flatten` now
    /// GRADUATED too: each is a gated category (`run_expand_or_flatten`,
    /// `EXPAND_FLOOR` / `FLATTEN_FLOOR`) driving the shipping
    /// `graph_to_jsonld(JsonLdForm::Expanded|Flattened)` writer, compared by
    /// re-parse RDF-equivalence to the suite's normative expected document.
    const NOT_IMPLEMENTED_CATS: &[(&str, &str)] = &[
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

        use sparq_engine::serialize::JsonLdForm;
        let tordf = run_tordf(&root);
        let fromrdf = run_fromrdf(&root);
        let compact = run_compact(&root);
        // [SONNET-4.6] sq-kk1mq — expand now uses the NATIVE DOCUMENT-LEVEL oracle
        // (sparq_jsonld::expand() + json_ld_equal comparator) instead of the old
        // RDF-equivalence oracle.  See run_expand_native() and EXPAND_FLOOR doc
        // for the oracle-correction rationale and old-vs-new breakdown.
        let expand = run_expand_native(&root);
        // flatten keeps the RDF-equivalence oracle (native flatten algorithm is
        // deferred to a separate bead; the writer path is still the right oracle).
        let flatten = run_expand_or_flatten(&root, "flatten", JsonLdForm::Flattened);
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
        // [OPUS-4.8] sq-oy1f — the expand + flatten ratchets. The CI grep depends on
        // these exact `^TOTAL expand `/`^TOTAL flatten ` prefixes with the pass count
        // in field $3 (same shape as the other lanes).
        println!(
            "TOTAL expand {} {} {} (floor {})",
            expand.pass, expand.fail, expand.skip, EXPAND_FLOOR
        );
        println!(
            "TOTAL flatten {} {} {} (floor {})",
            flatten.pass, flatten.fail, flatten.skip, FLATTEN_FLOOR
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
        if !expand.failures.is_empty() {
            println!("\nexpand failures (first 40):");
            for (id, why) in expand.failures.iter().take(40) {
                println!("  {}: {}", id, why);
            }
        }
        if !flatten.failures.is_empty() {
            println!("\nflatten failures (first 40):");
            for (id, why) in flatten.failures.iter().take(40) {
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
        // [OPUS-4.8] sq-oy1f — the expand + flatten ratchets (normative
        // answer-equivalence floors over the shipping writer).
        assert!(
            expand.pass >= EXPAND_FLOOR,
            "JSON-LD expand pass count regressed: {} < floor {} — see failures above",
            expand.pass,
            EXPAND_FLOOR
        );
        assert!(
            flatten.pass >= FLATTEN_FLOOR,
            "JSON-LD flatten pass count regressed: {} < floor {} — see failures above",
            flatten.pass,
            FLATTEN_FLOOR
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
