//! [OPUS-4.8] sq-ncvq.16 — the CENTRAL conformance SCOREBOARD registry.
//!
//! Before this module the project's conformance ratchets were reported in
//! several disconnected places: the W3C SPARQL suites (this crate's
//! `sparq-conformance` binary), the inference suites (`sparq-inference-conformance`),
//! the W3C SHACL core + SHACL-SPARQL suites (crate-local `cargo test` runners in
//! `sparq-shacl`), the OGC GeoSPARQL topology ratchet (a crate-local `cargo test`
//! in `sparq-geo`), the Solid WAC + ACP library-level decision-parity suites
//! (crate-local `cargo test` runners in `sparq-solid`), and the SolidLab ODRL Test
//! Suite (a crate-local `cargo test` runner in `sparq-policy`, sq-tmsd6). The
//! drift-scanner
//! (`scripts/drift-scan.py` §5.E `conformance-split`) flagged the SHACL + geo
//! ratchets (sq-ncvq.16), then the Solid WAC/ACP ratchets (sq-j174), as living
//! OUTSIDE the central scoreboard, so no single artifact answered "what conformance
//! does sparq claim, and at what floor?".
//!
//! This registry is that single source of truth. It enumerates EVERY conformance
//! suite the project ratchets — across crates — with, for each: the spec family,
//! the runner that executes it (this crate's binaries, or the named crate-local
//! `cargo test`), the CI job that gates it, and its RATCHET FLOOR (a pass /
//! pass+divergence count that may only RISE). The `sparq-conformance-scoreboard`
//! binary renders it as ONE markdown table so CI (and humans) see all suites at
//! a glance.
//!
//! ## Why a registry, not a merged runner
//!
//! The SHACL + geo + Solid runners are NOT re-implemented here. They depend on
//! `sparq-shacl` / `sparq-geo` / `sparq-solid`, and this crate must stay free of
//! those deps — exactly the constraint `crates/sparq-shacl/tests/w3c_core.rs`
//! records in its own header ("Manifest-walking helpers are modelled on
//! `sparq-conformance`'s (copied, not shared — that crate is dev-only and must not
//! become a dependency)"). Pulling sparq-shacl/sparq-geo/sparq-solid into the
//! conformance crate would invert that and couple the SPARQL/inference scoreboard
//! to the SHACL/geo/Solid build.
//! So consolidation happens at the REPORTING layer: the floors live here as the
//! authoritative list, the runners stay where their dependencies are, and a guard
//! test (`tests/scoreboard_floors.rs`) hermetically reads the crate-local
//! sources to prove the centrally-declared floors still match the runner-enforced
//! ones (so the two can never silently diverge).

/// How a suite is executed — the command CI runs to enforce its ratchet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runner {
    /// This crate's `sparq-conformance` binary (W3C SPARQL query/update/syntax).
    SparqlBinary,
    /// This crate's `sparq-inference-conformance` binary (rdf-mt / OWL 2 RL / N3 /
    /// entailment / rdf-turtle).
    InferenceBinary,
    /// A crate-local `cargo test` runner: `(crate, test target)`. The runner
    /// asserts its own pinned floor; CI re-checks the printed scoreboard.
    CrateTest {
        /// Crate the test lives in, e.g. `sparq-shacl`.
        krate: &'static str,
        /// `--test <target>` name, e.g. `w3c_core`.
        target: &'static str,
    },
    /// [OPUS-4.8] sq-oy1f.2 — a crate-local `cargo test` runner that is OFF by
    /// default and only runs under an opt-in cargo `feature` (e.g. the JSON-LD
    /// lane behind `jsonld-suite`). Same as [`CrateTest`](Runner::CrateTest) but
    /// the rendered command names the required feature, so the scoreboard's
    /// "how to run" column is correct for a feature-gated ratchet (the lane
    /// compiles out / self-skips when the feature is off — the lean-core posture).
    FeatureGatedCrateTest {
        /// Crate the test lives in, e.g. `sparq-conformance`.
        krate: &'static str,
        /// `--test <target>` name, e.g. `jsonld_suite`.
        target: &'static str,
        /// The cargo `--features` flag the lane requires, e.g. `jsonld-suite`.
        feature: &'static str,
    },
}

impl Runner {
    /// The shell command a maintainer (or CI) runs to enforce this suite's
    /// ratchet, for the report's "how to run" column.
    pub fn command(&self) -> String {
        match self {
            Runner::SparqlBinary => {
                "cargo run --release -p sparq-conformance --bin sparq-conformance".into()
            }
            Runner::InferenceBinary => {
                "cargo run --release -p sparq-conformance --bin sparq-inference-conformance".into()
            }
            Runner::CrateTest { krate, target } => {
                format!("cargo test -p {krate} --test {target}")
            }
            Runner::FeatureGatedCrateTest { krate, target, feature } => {
                format!("cargo test -p {krate} --features {feature} --test {target}")
            }
        }
    }
}

/// One conformance suite the project ratchets.
#[derive(Clone, Copy, Debug)]
pub struct Suite {
    /// Human label, e.g. "W3C SHACL core".
    pub label: &'static str,
    /// Spec family / governing body, e.g. "W3C SHACL", "OGC GeoSPARQL".
    pub family: &'static str,
    /// How it is run.
    pub runner: Runner,
    /// The CI job (in `.github/workflows/ci.yml`) that gates it.
    pub ci_job: &'static str,
    /// The RATCHET FLOOR — the minimum pass (or pass+documented-divergence) count
    /// CI enforces. May only RISE. For the binary-runner suites this is the count
    /// parsed from the report's Overall line; for the crate-test suites it is the
    /// `const` floor the runner asserts (kept in sync by the guard test).
    pub ratchet_floor: usize,
    /// What "floor" counts (for the report footnote) — pass, or pass+divergence.
    pub floor_basis: &'static str,
    /// One-line scope note.
    pub note: &'static str,
}

/// The CENTRAL registry: every conformance suite, in report order. Adding a new
/// ratchet anywhere in the workspace means adding a row HERE (and the drift
/// scanner's `conformance-split` exemption references this list).
///
/// FLOORS — keep each in lock-step with its enforcer (the guard test checks the
/// crate-test ones against source):
/// * SPARQL  1229  — `ci.yml` job `conformance` (`RATCHET=1229`).
/// * inference 1967 — `ci.yml` job `inference-conformance` (`RATCHET=1967`).
/// * SHACL core 98 — `sparq-shacl` `w3c_core.rs` `BASELINE_PASS = 98`.
/// * SHACL-SPARQL 5 — `sparq-shacl` `w3c_sparql.rs` `SHACL_SPARQL_FLOOR = 5`.
/// * OGC GeoSPARQL 197 — `sparq-geo` `ogc_compliance_ratchet.rs`
///   `OGC_RATCHET_FLOOR = 197` (sq-cbe4t raised it 119 -> 158;
///   sq-lk3aw.1 raised it 158 -> 197: 39 net-new assertions covering
///   edge-adjacent polygons / disjoint line+polygon / parallel lines /
///   point-on-line / and multi-pair rcc8/eh disjoint cells).
/// * OGC GeoSPARQL query-rewrite 48 — `sparq-geo` `ogc_query_rewrite_ratchet.rs`
///   `OGC_QUERY_REWRITE_FLOOR = 48` (sq-wf9qg raised to 38; sq-lk3aw.2 raised 38→48:
///   [SONNET-4.6] strengthened semantics-preserving comparison + extended coverage;
///   opt-in `geosparql_rewrite` feature; topology PROPERTY forms answered via the
///   rewrite, MEASURED pass count).
/// * Solid WAC 12 — `sparq-solid` `tests/common/mod.rs` `WAC_SCENARIO_FLOOR = 12`
///   (sq-j174; floor const moved to the shared parity-corpus module in sq-t58w.6).
/// * Solid ACP 12 — `sparq-solid` `tests/common/mod.rs` `ACP_SCENARIO_FLOOR = 12`
///   (sq-j174; floor const moved to the shared parity-corpus module in sq-t58w.6).
/// * JSON-LD toRdf 413 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `TORDF_FLOOR = 413` (sq-oy1f.2; opt-in `jsonld-suite` feature).
/// * JSON-LD fromRdf 51 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `FROMRDF_FLOOR = 51` (sq-oy1f.2; opt-in `jsonld-suite` feature).
/// * JSON-LD compact 186 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `COMPACT_FLOOR = 186` (sq-3uos5; RAISED 163→186 by sq-oy1f.16 after #978's
///   faithfulness fixes; opt-in `jsonld-suite` feature; RDF → compacted JSON-LD via
///   the native Compaction Algorithm, lossless round-trip).
/// * JSON-LD frame 61 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `FRAME_FLOOR = 61` (sq-oy1f.19; opt-in `jsonld-suite` feature; RDF → framed
///   JSON-LD via the native Framing Algorithm over the SEPARATE w3c/json-ld-framing
///   suite, compared by re-parse RDF-equivalence to the normative expected output).
/// * JSON-LD expand 240 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `EXPAND_FLOOR = 240` (sq-kk1mq oracle-correction re-baseline; opt-in
///   `jsonld-suite` feature; the expand lane now calls `sparq_jsonld::expand()`
///   directly and compares the result to the expected document via `json_ld_equal`
///   — a document-level JSON comparator measuring JSON-LD data-model (semantic)
///   equivalence, NOT structural identity: object key order insignificant, array
///   order significant only inside `@list`, integers compared exactly (i64/u64),
///   non-integral numbers as f64.  ~18 of 240 passes are semantically-equal-but-
///   reordered vs. the W3C reference (strict-ordered count 222).  OLD floor was
///   247 under the RDF-equivalence oracle (sq-oy1f); the rebase reveals a net 7
///   fewer passes (20 flips minus 13 recoveries) and 26 new honest fails).
/// * JSON-LD flatten 50 — `sparq-conformance` `tests/jsonld_suite.rs`
///   `FLATTEN_FLOOR = 50` (sq-oy1f; opt-in `jsonld-suite` feature; RDF → flattened
///   JSON-LD via the shipping `graph_to_jsonld(JsonLdForm::Flattened)` writer,
///   compared by re-parse RDF-equivalence to the normative expected document).
/// * Solid WAC differential 0 — `sparq-solid` `tests/differential_oracle.rs`
///   `DIVERGENCE_FLOOR = 0` (sq-t58w.8; a divergence-count floor, hard 0 — the WAC
///   and ACP differential rows share this one const).
/// * Solid ACP differential 0 — same `DIVERGENCE_FLOOR = 0` (sq-t58w.8).
/// * SolidLab ODRL 67 — `sparq-policy` `tests/odrl_test_suite.rs`
///   `ODRL_SUITE_FLOOR = 67` (sq-tmsd6 wired it at 59; the constraint-matching batch
///   sq-euhr3/sq-k7itg/sq-a0zef raised it to 67 of 68 cases pass, 1 in a documented
///   not-implemented bucket).
/// * D-entailment 1 — `sparq-conformance` `tests/d_entail_suite.rs`
///   `D_ENTAIL_FLOOR = 1` (sq-e5atd; opt-in `d-entail` feature; the D-only
///   `sparql11/entailment` tests graduated from OutOfScope to Pass through
///   sparq-reason's `Profile::D` — rdfD1 typing + typed value-space equality).
/// * sparql11/service evaluation 6 — `sparq-conformance`
///   `tests/service_eval_suite.rs` `SERVICE_EVAL_FLOOR = 6` (sq-ddpgx + sq-my8wd.1;
///   opt-in `service` feature; the deferred SERVICE/federation tests graduated from the
///   SPARQL binary's skip bucket to Pass by serving each `qt:serviceData` block on
///   a REAL in-process loopback endpoint — the sq-ushvx harness — and driving the
///   federated query end-to-end through the engine's REAL ureq transport; [SONNET-4.6]
///   nested non-SILENT SERVICE now handled by configuring outer-endpoint egress
///   allowlists; a variable SERVICE endpoint is the one remaining documented Skip).
/// * SPARQL 1.1 Protocol (HTTP) 21 — `sparq-conformance`
///   `tests/http_protocol_suite.rs` `HTTP_PROTOCOL_FLOOR = 21` (sq-jaj38; opt-in
///   `http-protocol` feature; RAW HTTP requests at the in-process loopback server
///   exercising the SPARQL 1.1 Protocol contract — GET/POST query+update, the QUERY
///   method, dataset overrides, SRJ/SRX/CSV/TSV negotiation, 200/400/405/406/415; the
///   present-but-unsatisfiable Accept now 406 (Oxigraph parity, sq-406acc); the
///   absent/`*/*` Accept JSON default + ASK-in-CSV are documented divergences, NOT summed in).
/// * SPARQL 1.1 Service Description + Graph Store Protocol 39 — `sparq-conformance`
///   `tests/sd_gsp_suite.rs` `SD_GSP_FLOOR = 39` (sq-1uuxz; opt-in
///   `federation-descriptors` feature; the `GET /sparql` (no query) Service-Description
///   advertises exactly the formats/languages/versions/features the server genuinely
///   implements — no over-advertising — PLUS a GET/PUT/POST/DELETE Graph-Store-Protocol
///   round-trip over RAW HTTP verifying store state after each op; the absent-graph
///   200-empty read is a documented divergence, NOT summed in).
/// * text-search differential oracle 18750 — `sparq-text`
///   `tests/bm25_oracle.rs` `TEXT_ORACLE_FLOOR = 18750` (sq-ripcg; a sparq
///   EXTENSION ratchet, NOT standards conformance — no normative full-text-over-RDF
///   / BM25 suite exists; floor = the MEASURED count of bit-exact BM25 score
///   assertions the fixed-seed corpus battery makes against a from-scratch
///   independent reference scorer; default-on, no opt-in feature required).
/// * RSP expressivity / SRBench correctness 317 — `sparq-rsp`
///   `tests/srbench_oracle.rs` `RSP_EXPRESSIVITY_FLOOR = 317` (sq-2n1q3.3; raised
///   from 303 sq-2n1q3.1; from 149 sq-mcb3q baseline; a sparq EXTENSION ratchet,
///   NOT standards conformance — RSP-QL is a W3C-COMMUNITY spec and SRBench is a
///   benchmark, so there is NO normative RDF-Stream-Processing Recommendation or its
///   conformance suite; floor = the MEASURED count of deterministic per-window
///   correctness assertions the fixed SRBench-shaped battery makes across window
///   types / R2S operators / EvalModes / multi-window joins (incl. 3+-window joins
///   and ISTREAM/DSTREAM over multi-window joins) against an INDEPENDENT
///   batch-rebuild + closed-form oracle; default-on, no opt-in feature required).
///   [SONNET-4.6]
/// * OWL 2 QL (DL-Lite_R) certain-answer oracle 11 — `sparq-conformance`
///   `tests/ql_dllite_suite.rs` `QL_DLLITE_FLOOR = 11` (sq-qo1a9; opt-in
///   `ql-experimental` feature; a sparq EXTENSION ratchet, NOT a
///   full-OWL-2-QL-conformance claim — no runnable normative W3C QL certain-answer
///   suite exists; floor = the MEASURED count of formal DL-Lite_R cases on which
///   `sparq_reason_ql::rewrite_production` is sound AND complete, i.e. the rewritten
///   UCQ evaluated over the unmodified ABox returns EXACTLY the hand-derived certain
///   answers; the broader `pr:QL` entailment-arm intensional gap stays
///   experimental/OutOfScope, never summed in).
/// * OWL 2 QL entailment-regime graduated subset 11 — `sparq-conformance`
///   `tests/ql_entailment_floor.rs` `QL_ENTAILMENT_FLOOR = 11` (sq-pbz04.3.4; opt-in
///   `ql-experimental` feature; a sparq EXTENSION ratchet, NOT an OWL 2 QL /
///   entailment-regime conformance claim — the floor is the PINNED NAMED-CASE list
///   of `pr:QL` `sparql11/entailment` cases passing ALL SIX graduation conditions,
///   exact set equality: regressions AND unpinned additions both fail CI; every
///   non-graduated case carries an exhaustive hold-reason taxonomy; raised 9→11 by
///   sq-pbz04.3.1 B2 literal-object broadening: `lang` + `plainLit` both graduate
///   [SONNET-4.6]).
/// * OWL 2 EL classification 50 — `sparq-conformance` `tests/el_suite.rs`
///   `EL_SUITE_FLOOR = 50` (sq-pbz04.2.4; opt-in `el-suite` feature; a sparq EXTENSION
///   ratchet, NOT a full-OWL-2-EL-conformance claim — CR7–CR9 concrete domains + ABox
///   inconsistency are deferred; floor = the MEASURED count of W3C OWL 2 EL
///   (test:EL ∧ test:RDF-BASED, Approved) check rows on which `sparq_reason_el`'s
///   consequence-based classifier computes the expected outcome via `classify_graph` +
///   the shared bnode-homomorphism entailment check; 28 audited PERMANENT divergences
///   (ABox / RBox / owl:unionOf / owl:equivalentClass-form) are reported separately,
///   never summed in). [SONNET-4.6]
pub const SUITES: &[Suite] = &[
    Suite {
        label: "W3C SPARQL (1.0 / 1.1 / 1.2, query+update+syntax)",
        family: "W3C SPARQL",
        runner: Runner::SparqlBinary,
        ci_job: "conformance",
        ratchet_floor: 1229,
        floor_basis: "pass + documented divergence",
        note: "query/update evaluation + the four mf:*SyntaxTest* kinds",
    },
    Suite {
        label: "Inference (rdf-mt / OWL 2 RL / N3 / entailment / rdf-turtle)",
        family: "W3C RDF Semantics + OWL + N3",
        runner: Runner::InferenceBinary,
        ci_job: "inference-conformance",
        ratchet_floor: 1967,
        floor_basis: "pass + documented divergence",
        note: "reasoning suites run against sparq-reason, plus the Turtle oracle",
    },
    Suite {
        label: "W3C SHACL core",
        family: "W3C SHACL",
        runner: Runner::CrateTest { krate: "sparq-shacl", target: "w3c_core" },
        ci_job: "shacl-conformance",
        ratchet_floor: 98,
        floor_basis: "pass",
        note: "data-shapes sht:Validate core suite (w3c/data-shapes)",
    },
    Suite {
        label: "W3C SHACL-SPARQL",
        family: "W3C SHACL",
        runner: Runner::CrateTest { krate: "sparq-shacl", target: "w3c_sparql" },
        ci_job: "shacl-conformance",
        ratchet_floor: 5,
        floor_basis: "pass",
        note: "sh:sparql node + property constraint sub-suites",
    },
    Suite {
        label: "OGC GeoSPARQL topology compliance",
        family: "OGC GeoSPARQL",
        runner: Runner::CrateTest { krate: "sparq-geo", target: "ogc_compliance_ratchet" },
        ci_job: "geo-conformance",
        // [OPUS-4.8] sq-cbe4t — raised 119 -> 158: 39 net-new hand-derived DE-9IM
        // assertions (reverse-order/symmetry coverage + MULTI* operands). The
        // crate-local `OGC_RATCHET_FLOOR` const moved in lock-step; the floor-sync
        // guard (`tests/scoreboard_floors.rs`) pins the two together.
        // [SONNET-4.6] sq-lk3aw.1 — raised 158 -> 197: 39 more net-new assertions
        // (edge-adjacent polygons / disjoint line+polygon / parallel lines /
        // point-on-line / multi-pair rcc8/eh disjoint cells).
        ratchet_floor: 197,
        floor_basis: "pass",
        // [OPUS-4.8] sq-cbe4t — DISTANCE-APPROXIMATION HONESTY NOTE. This topology
        // ratchet is exact DE-9IM (no approximation). The `geof:distance` METRIC
        // path (a separate, non-topological surface, scored under the R1-R30
        // requirements probe) is exact HAVERSINE only for point↔point /
        // point↔geometry; between two EXTENDED geometries it uses a LOCAL
        // EQUIRECTANGULAR approximation about mean latitude (accurate locally,
        // degrading at continental scale / near the poles). See the sparq-geo
        // README "Distance accuracy caveat" — no exactness is claimed there.
        note: "hand-curated sf/eh/rcc8 topology + WKT/GML equivalence assertions \
               (exact DE-9IM); geof:distance is exact haversine point↔point, local \
               equirectangular approximation extended↔extended (see sparq-geo README)",
    },
    // [OPUS-4.8] sq-wf9qg — the OGC GeoSPARQL QUERY-REWRITE extension
    // (`/conf/query-rewrite-extension`) ratchet, graduated alongside the topology
    // FILTER-function ratchet above. The runner is crate-local in sparq-geo
    // (`tests/ogc_query_rewrite_ratchet.rs`) but behind the OPT-IN
    // `geosparql_rewrite` feature (OFF by default) so the default + `--workspace`
    // builds never compile the rewrite surface and the STANDARD SPARQL behaviour
    // stays untouched — the lean opt-in posture. Each case drives a topology
    // PROPERTY pattern (`?f geo:sfWithin ?g`) end-to-end through the real rewrite
    // + engine, asserting result-equivalence to the lexical `geof:` oracle AND
    // that the standard entry point binds zero rows (no asserted topology triple).
    // The floor is the MEASURED pass count; kept in lock-step by
    // `tests/scoreboard_floors.rs`.
    Suite {
        label: "OGC GeoSPARQL query-rewrite extension",
        family: "OGC GeoSPARQL",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-geo",
            target: "ogc_query_rewrite_ratchet",
            feature: "geosparql_rewrite",
        },
        ci_job: "geo-conformance",
        ratchet_floor: 48,
        floor_basis: "pass",
        note: "topology PROPERTY forms (geo:sf*/eh*/rcc8*) answered end-to-end via the \
               opt-in query-rewrite extension, result-equivalent to the geof: oracle",
    },
    // [OPUS-4.8] sq-j174 — the Solid WAC + ACP library-level decision-parity suites
    // (harness landed under sq-3jtd.8). The runners stay crate-local in sparq-solid
    // (the dev-only conformance crate must not take sparq-solid as a dep — same
    // constraint as SHACL/geo); the central scoreboard REPORTS them with their
    // ratchet floors, kept in lock-step by `tests/scoreboard_floors.rs`. The floor
    // is the scenario COUNT each corpus must cover (a corpus that silently shrinks is
    // a coverage regression), exactly the in-source ratchet shape geo uses.
    Suite {
        label: "Solid WAC decision parity",
        family: "Solid WAC",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "conformance_wac" },
        ci_job: "solid-conformance",
        ratchet_floor: 12,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over minimal per-construct WAC .acl scenarios",
    },
    Suite {
        label: "Solid ACP decision parity",
        family: "Solid ACP",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "conformance_acp" },
        ci_job: "solid-conformance",
        ratchet_floor: 12,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over minimal per-construct ACP ACR scenarios",
    },
    // [OPUS-4.8] sq-t58w.8 — the Solid WAC + ACP DIFFERENTIAL ORACLE (harness landed
    // under sq-t58w.7, `crates/sparq-solid/tests/differential_oracle.rs`). It runs the
    // SAME shared parity corpus through THREE independent deciders (the engine, a
    // from-scratch procedural reference evaluator, and the hand `Expect` table) and
    // asserts they never disagree. The ratchet is a DIVERGENCE count whose ONLY
    // acceptable value is 0 — so unlike the scenario-COUNT floors above (which rise as
    // the corpus grows), this floor is the hard `DIVERGENCE_FLOOR = 0` the runner
    // asserts. Both rows share that ONE source const; the floor-sync guard
    // (`tests/scoreboard_floors.rs`) keeps them locked to it. The runner stays
    // crate-local in sparq-solid (this dev-only crate must not take sparq-solid as a
    // dep — same constraint as SHACL/geo/the conformance suites above). `ci_job` is the
    // Solid conformance lane; the dedicated `<WAC|ACP> differential … divergences N`
    // grep-ratchet is sq-t58w.3.
    Suite {
        label: "Solid WAC differential oracle",
        family: "Solid WAC",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "differential_oracle" },
        ci_job: "solid-conformance",
        ratchet_floor: 0,
        floor_basis: "0 divergences",
        note: "engine vs an independent reference evaluator vs the hand Expect table, \
               over the WAC parity corpus (zero divergence)",
    },
    Suite {
        label: "Solid ACP differential oracle",
        family: "Solid ACP",
        runner: Runner::CrateTest { krate: "sparq-solid", target: "differential_oracle" },
        ci_job: "solid-conformance",
        ratchet_floor: 0,
        floor_basis: "0 divergences",
        note: "engine vs an independent reference evaluator vs the hand Expect table, \
               over the ACP parity corpus (zero divergence)",
    },
    // [OPUS-4.8] sq-oy1f.2 / sq-3uos5 / sq-oy1f.19 / sq-oy1f — the W3C JSON-LD 1.1
    // conformance ratchets. The runner is crate-local here (`tests/jsonld_suite.rs`)
    // but behind the OPT-IN `jsonld-suite` feature (forwards to sparq-core/jsonld +
    // sparq-engine/serialize-rdf) so the default + `--workspace` builds neither link
    // oxjsonld nor go red — the lean-core posture. Six gated categories: toRdf
    // (JSON-LD → RDF through the real oxjsonld parse path), fromRdf (RDF → JSON-LD
    // through the native serialize-rdf writer, re-parse round-trip), compact (RDF →
    // compacted JSON-LD via the native Compaction Algorithm, lossless round-trip),
    // frame (RDF → framed JSON-LD via the native Framing Algorithm over the SEPARATE
    // w3c/json-ld-framing suite, RDF-equivalence to the normative expected output),
    // and expand + flatten (RDF → expanded/flattened JSON-LD via the shipping
    // `graph_to_jsonld` writer, RDF-equivalence to the normative expected document).
    // The floors are the MEASURED pass counts at the pinned suite revisions (NOT
    // 100% — remote-context/option/writer-shape divergences are honest, recorded
    // gaps); they may only RISE. html + remote-doc remain the documented
    // NOT-IMPLEMENTED buckets the runner reports separately (never failed). Floors
    // kept in lock-step
    // by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 toRdf",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 413,
        floor_basis: "pass",
        note: "JSON-LD → RDF through the real oxjsonld parse path (jsonld feature); \
               compact + frame are now gated; expand/flatten remain not-implemented buckets",
    },
    Suite {
        label: "W3C JSON-LD 1.1 fromRdf",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 51,
        floor_basis: "pass",
        note: "RDF → JSON-LD through the native serialize-rdf writer, compared by a \
               re-parse RDF-dataset round-trip (expanded + prefix-@context forms)",
    },
    // [OPUS-4.8] sq-3uos5 — the W3C JSON-LD 1.1 `compact` ratchet (extends sq-oy1f.2,
    // epic sq-oy1f). Each `jld:CompactTest` input is parsed to RDF (the real oxjsonld
    // path), compacted against the case `@context` through the native hand-rolled
    // Compaction Algorithm (`graph_to_jsonld_compact`, serialize-rdf), then the
    // compacted document is re-parsed and required to reconstruct the SAME RDF dataset
    // (`reparse(compact(D, ctx)) ≡ D` — the lossless-compaction invariant, the same
    // oxjsonld self-reparse oracle toRdf/fromRdf use). The floor is the MEASURED pass
    // count at the pinned revision; the remaining cases are honest compaction
    // divergences (below the floor, to RISE) or documented SKIP buckets (negatives
    // sparq does not raise, JSON-LD-1.0-only, non-inline/remote @context, empty RDF).
    // Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 compact",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        // [OPUS-4.8] sq-oy1f.16 — RAISED 163 → 186 after the #978 compaction
        // faithfulness fixes landed (re-measured on current main: 186 pass).
        ratchet_floor: 186,
        floor_basis: "pass",
        note: "RDF → compacted JSON-LD through the native Compaction Algorithm \
               (serialize-rdf), compared by a re-parse RDF-dataset round-trip \
               (lossless-compaction invariant)",
    },
    // [OPUS-4.8] sq-oy1f.19 — the W3C JSON-LD 1.1 `frame` ratchet (epic sq-oy1f),
    // over the SEPARATE w3c/json-ld-framing suite (fetch-jsonld-framing-tests.sh).
    // Each `jld:FrameTest` input (an arbitrary EXPANDED JSON-LD document) is parsed
    // to RDF (the real oxjsonld path), framed against the case frame document
    // through the native hand-rolled Framing Algorithm (`graph_to_jsonld_framed`,
    // serialize-rdf), then the framed output is re-parsed and required to
    // reconstruct the SAME RDF dataset as the suite's NORMATIVE expected output
    // (`reparse(frame(D, F)) ≡ reparse(expected)` — framing is a SELECT+RESHAPE, so
    // the oracle anchors on the expected document, NOT the input). The floor is the
    // MEASURED pass count; the remaining cases are honest framer divergences (below
    // the floor, to RISE) or documented SKIP (the 3 frame-validation negatives
    // sparq's TOTAL framer does not raise). Floor kept in lock-step by
    // `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 frame",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 61,
        floor_basis: "pass",
        note: "RDF → framed JSON-LD through the native Framing Algorithm \
               (serialize-rdf) over the w3c/json-ld-framing suite, compared by a \
               re-parse RDF-equivalence to the normative expected output",
    },
    // [OPUS-4.8] sq-oy1f — the W3C JSON-LD 1.1 `expand` + `flatten` ratchets (epic
    // sq-oy1f). Each `jld:ExpandTest` / `jld:FlattenTest` input is parsed to RDF (the
    // [SONNET-4.6] sq-kk1mq oracle-correction re-baseline: the expand lane now calls
    // sparq_jsonld::expand() directly and compares the result to the suite's expected
    // document via json_ld_equal (document-level JSON comparator measuring JSON-LD
    // data-model (semantic) equivalence, NOT structural identity; object key order
    // insignificant; array order significant only inside @list; integers compared exactly
    // via i64/u64, non-integral numbers as f64).  ~18 of 240 passes are semantically-
    // equal-but-reordered vs. the W3C reference (strict-ordered count 222).
    // The old floor was 247 under the RDF-equivalence oracle (sq-oy1f); the rebase
    // reveals a net 7 fewer passes (20 old-pass→new-fail flips minus 13 recoveries:
    // 8 old-fail→new-pass via oracle precision + 5 old-skip→new-pass via options
    // forwarding) and 26 new honest failures.  The new floor 240 is the MEASURED pass
    // count with the corrected oracle at the pinned suite revision (sq-kk1mq).  The
    // flatten lane keeps the old RDF-equivalence oracle (native flatten algorithm
    // deferred; writer path is the correct oracle there).
    // Floors kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C JSON-LD 1.1 expand",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 240,
        floor_basis: "pass",
        note: "native sparq_jsonld::expand() + json_ld_equal semantic-equivalence comparator \
               (sq-kk1mq; NOT structural identity — ~18/240 passes are reordered, \
               strict-ordered count 222; re-baseline from 247 under RDF-equivalence \
               oracle sq-oy1f); options forwarded (base, expandContext, processingMode)",
    },
    Suite {
        label: "W3C JSON-LD 1.1 flatten",
        family: "W3C JSON-LD",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "jsonld_suite",
            feature: "jsonld-suite",
        },
        ci_job: "jsonld-conformance",
        ratchet_floor: 50,
        floor_basis: "pass",
        note: "RDF → flattened JSON-LD through the shipping graph_to_jsonld(Flattened) \
               writer (serialize-rdf), compared by a re-parse RDF-equivalence to the \
               normative expected document",
    },
    // [OPUS-4.8] sq-tmsd6 — the SolidLab ODRL Test Suite, wired as a crate-local
    // decision-parity ratchet in sparq-policy (mirrors the Solid WAC/ACP pattern:
    // the dev-only conformance crate must not take sparq-policy as a dep). Each of
    // the 68 self-describing Turtle cases is driven through the REAL
    // `parse_policy_str` + `evaluate` path; the oracle is the case's expected
    // compliance report (`report:activationState` ⇒ ALLOW/DENY). The floor is the
    // pass COUNT (67 at the pinned revision after the constraint-matching batch
    // sq-euhr3/sq-k7itg/sq-a0zef — `odrl:LogicalConstraint`, party/asset collection
    // membership, and the `odrl:use` action hierarchy now PASS); the 1 remaining
    // case is a documented NOT-IMPLEMENTED divergence (a duty whose discharge state
    // is unknown/`report:NonSet` — sparq is fail-closed) that does not fail the gate.
    // Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "SolidLab ODRL Test Suite",
        family: "SolidLab ODRL",
        runner: Runner::CrateTest { krate: "sparq-policy", target: "odrl_test_suite" },
        ci_job: "odrl-conformance",
        ratchet_floor: 67,
        floor_basis: "scenario",
        note: "library-level allow/deny parity over the SolidLab self-describing ODRL \
               cases through sparq-policy's real evaluate() path",
    },
    // [OPUS-4.8] sq-e5atd (epic sq-pbz04) — the W3C SPARQL 1.1 D-entailment
    // (datatype / value-space) ratchet. The runner is crate-local here
    // (`tests/d_entail_suite.rs`) but behind the OPT-IN `d-entail` feature (forwards
    // to sparq-reason/d-entail) so the default + `--workspace` builds neither link
    // the `Profile::D` materializer nor go red — the lean-core posture, and the
    // inference BINARY keeps these tests OutOfScope when the feature is off so its
    // own ratchet floor is byte-for-byte unchanged. The genuinely D-only
    // `sparql11/entailment` tests (regime `ent:D` without a stronger RDFS/RDF/
    // OWL-RDF-Based regime) GRADUATE from that OutOfScope bucket to Pass here:
    // premise → `sparq_reason::Profile::D` (rdfD1 typing + the typed value-space
    // comparator — "1"^^xsd:integer ≡ "1.0"^^xsd:decimal, NOT an f64 fast path) →
    // query over the closure through the same evaluation path as the SPARQL harness,
    // with the entailment-regime answer restriction applied. The floor is the
    // MEASURED D-only pass count at the pinned revision (NOT 100%; the broader
    // D-inconsistency / value-space-subset surface is tracked-not-asserted — a child
    // of sq-pbz04). Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C SPARQL 1.1 D-entailment",
        family: "W3C RDF Semantics",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "d_entail_suite",
            feature: "d-entail",
        },
        ci_job: "inference-conformance",
        ratchet_floor: 1,
        floor_basis: "pass",
        note: "D-only sparql11/entailment tests graduated from OutOfScope to Pass \
               through sparq-reason's opt-in Profile::D (rdfD1 typing + typed \
               value-space equality)",
    },
    // [OPUS-4.8] sq-ddpgx (epic sq-my8wd) — the W3C SPARQL 1.1 `sparql11/service`
    // EVALUATION ratchet. The runner is crate-local here
    // (`tests/service_eval_suite.rs`) but behind the OPT-IN `service` feature
    // (forwards to `service-loopback` → tokio + axum via sparq-server/service AND
    // ureq via sparq-engine/service) so the default + `--workspace` builds neither
    // link the async server stack nor go red — the lean-core posture; the SPARQL
    // BINARY keeps these tests skipped (`unsupported_feature = "SERVICE /
    // federation"`) so its ratchet floor is byte-for-byte unchanged. These tests
    // GRADUATE from that skip bucket to Pass here: each `qt:serviceData` block is
    // served by a REAL in-process `sparq_server::serve` loopback endpoint (the
    // merged sq-ushvx #1291 harness) on an ephemeral 127.0.0.1:0 port, the
    // well-known endpoint IRIs are rewritten to the bound loopback URLs, and the
    // federated SERVICE query runs end-to-end through the engine's REAL ureq
    // transport, compared to the `.srx` oracle. The floor is the MEASURED pass
    // count at the pinned rdf-tests revision (NOT 100%: a variable SERVICE endpoint
    // is the one remaining honest tracked-not-asserted divergence — documented Skip,
    // never skip-laundered into the count; [SONNET-4.6] sq-my8wd.1 raised the floor
    // from 5→6 by handling nested non-SILENT SERVICE via per-endpoint egress config).
    // Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C SPARQL 1.1 sparql11/service evaluation",
        family: "W3C SPARQL",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "service_eval_suite",
            feature: "service",
        },
        ci_job: "service-federation-conformance",
        ratchet_floor: 6,
        floor_basis: "pass",
        note: "sparql11/service federated SERVICE queries run end-to-end through \
               REAL in-process loopback endpoints (the sq-ushvx harness) + the \
               engine's REAL ureq transport, compared to the .srx oracle",
    },
    // [OPUS-4.8] sq-jaj38 (epic sq-my8wd) — the W3C SPARQL 1.1 PROTOCOL (HTTP layer)
    // conformance ratchet. Where the sibling sparql11/service row above graduates the
    // federated SERVICE EVALUATION tests, THIS row covers the HTTP PROTOCOL itself: the
    // request/response contract a SPARQL endpoint MUST honour. The runner is crate-local here
    // (`tests/http_protocol_suite.rs`) but behind the OPT-IN `http-protocol` feature (forwards
    // to `service-loopback` → tokio + axum via sparq-server/server; NO new third-party dep —
    // the raw HTTP client is a std-only TcpStream helper) so the default + `--workspace` builds
    // neither link the async server stack nor go red — the lean-core posture. Each assertion is
    // a RAW HTTP request to the merged sq-ushvx in-process loopback server (an ephemeral
    // 127.0.0.1:0 port) with an exact method / Content-Type / Accept / body, checking the
    // status code, the response Content-Type and the payload shape: query via GET /
    // POST-urlencoded / POST-direct, the HTTP QUERY method (#1304), update via POST, the
    // default-graph-uri / named-graph-uri dataset overrides, result-format content negotiation
    // (SRJ / SRX / CSV / TSV), the present-but-unsatisfiable Accept → 406 (Oxigraph parity,
    // sq-406acc) and the 200/400/405/406/415 status codes. The floor is the MEASURED PASS count;
    // documented protocol divergences (an absent/*/* Accept defaults to JSON per the W3C-permitted
    // default representation; an ASK boolean in CSV/TSV falling back to JSON) are reported
    // separately and NOT summed into it, so a documented gap can never inflate the conformance
    // number — an honest W3C-Protocol claim, scoped to what the server genuinely satisfies. Floor
    // kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "W3C SPARQL 1.1 Protocol (HTTP)",
        family: "W3C SPARQL",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "http_protocol_suite",
            feature: "http-protocol",
        },
        ci_job: "service-federation-conformance",
        ratchet_floor: 21,
        floor_basis: "pass",
        note: "SPARQL 1.1 Protocol operations (GET/POST query+update, the QUERY method, \
               dataset overrides, SRJ/SRX/CSV/TSV negotiation, 200/400/405/406/415) over RAW \
               HTTP against the in-process loopback server; a present-but-unsatisfiable Accept \
               now returns 406 (Oxigraph parity, sq-406acc); the absent/*/* Accept JSON default + \
               ASK-in-CSV are documented divergences, NOT summed into the floor",
    },
    // [OPUS-4.8] sq-1uuxz (epic sq-my8wd) — the SPARQL 1.1 SERVICE-DESCRIPTION + GRAPH-STORE
    // PROTOCOL conformance ratchet. Where the sibling http-protocol row above covers the
    // query/update Protocol contract, THIS row covers the two federation-descriptor + write
    // surfaces the server exposes behind `federation-descriptors`: (A) the `GET /sparql` (no
    // query) Service-Description document — asserting it advertises EXACTLY the result/input
    // formats + query/update languages + SPARQL versions + BasicFederatedQuery the server
    // GENUINELY implements (no over-advertising; each advertised result format is cross-checked
    // against a real SELECT, and JSON-LD — not served in this build — must NOT appear); and (B) a
    // full GET/PUT/POST/DELETE Graph-Store-Protocol round-trip on a named graph (indirect
    // `?graph=` + direct `/graphs/<path>`) and the default graph (`?default`), VERIFYING store
    // state after every op (PUT→GET-back-equal; PUT replaces; POST merges; DELETE removes;
    // 200/201/204/400/404/405/415). The runner is crate-local here (`tests/sd_gsp_suite.rs`) but
    // behind the OPT-IN `federation-descriptors` feature (forwards to `service-loopback` → tokio +
    // axum via sparq-server/server, AND turns on sparq-server/federation-descriptors so the SD
    // endpoint is live; NO new third-party dep — the raw HTTP client is the same std-only
    // TcpStream helper) so the default + `--workspace` builds neither link the async server stack
    // nor go red — the lean-core posture. The floor is the MEASURED PASS count; the one documented
    // divergence (a GSP read of an absent named graph is 200+empty, not 404 — GSP-permitted) is
    // reported separately and NOT summed into it, so a documented gap can never inflate the
    // conformance number. Floor kept in lock-step by `tests/scoreboard_floors.rs`.
    Suite {
        label: "SPARQL 1.1 Service Description + Graph Store Protocol",
        family: "W3C SPARQL",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "sd_gsp_suite",
            feature: "federation-descriptors",
        },
        ci_job: "service-federation-conformance",
        ratchet_floor: 39,
        floor_basis: "pass",
        note: "the GET /sparql (no query) Service-Description advertises exactly the \
               formats/languages/versions/features the server genuinely implements (no \
               over-advertising), PLUS a GET/PUT/POST/DELETE Graph-Store-Protocol round-trip \
               (named + default graph, indirect + direct identification) verifying store state \
               after each op; absent-graph 200-empty read is a documented divergence, NOT summed in",
    },
    // [OPUS-4.8] sq-ripcg (epic sq-lk3aw) — the sparq-text DIFFERENTIAL BM25 ORACLE
    // (runner lives crate-local in `sparq-text/tests/bm25_oracle.rs`). This is
    // HONESTLY a sparq EXTENSION ratchet, NOT a standards-conformance claim: BM25 is
    // an information-retrieval scoring function (Robertson & Spärck Jones), and there
    // is NO normative "full-text search over RDF" recommendation or its accompanying
    // test suite to point at — so `family = "sparq extension"` (the only family in
    // this registry with NO standards body behind it; even the OGC GeoSPARQL
    // query-rewrite EXTENSION row stays `family = OGC GeoSPARQL` because it extends a
    // standard, whereas BM25-over-RDF extends nothing normative). Deliberately
    // distinct so the row never reads as a W3C/OGC/IETF/Solid conformance claim. The
    // runner recomputes
    // the exact BM25 score of every `search`/`search_any` hit with a from-scratch
    // independent reference scorer and asserts the index reproduces it bit-for-bit
    // (`f32::to_bits`) AND in the same best-first order, over a fixed-seed corpus
    // battery. The floor is the ACTUAL measured count of score-exact assertions the
    // battery makes (the `assertions N (floor F)` line the runner prints) — it may
    // only RISE, and a drop is a scoring regression. The runner stays crate-local
    // (the dev-only conformance crate must NOT take sparq-text as a dep — same
    // constraint as SHACL/geo/Solid/ODRL), so this row takes NO new dependency edge
    // on sparq-text; the floor const `TEXT_ORACLE_FLOOR` is mirrored here and kept in
    // lock-step by `tests/scoreboard_floors.rs` (read textually, no cross-crate dep).
    Suite {
        label: "text-search differential oracle",
        family: "sparq extension",
        runner: Runner::CrateTest { krate: "sparq-text", target: "bm25_oracle" },
        ci_job: "text-oracle",
        ratchet_floor: 18750,
        floor_basis: "score-exact assertions (sparq EXTENSION, NOT standards conformance)",
        note: "EXTENSION ratchet — no normative full-text-over-RDF / BM25 standard \
               exists: sparq-text's BM25 search/ranking vs a from-scratch independent \
               reference scorer, bit-exact per-hit scores + best-first order over a \
               fixed corpus battery",
    },
    // [OPUS-4.8] sq-mcb3q (epic sq-2n1q3) — the sparq-rsp RSP EXPRESSIVITY /
    // SRBench CORRECTNESS oracle (runner lives crate-local in
    // `sparq-rsp/tests/srbench_oracle.rs`). HONESTLY a sparq EXTENSION ratchet,
    // NOT a standards-conformance claim: SRBench (Zhang/Della Valle/Calbimonte et
    // al., ISWC 2012) is the canonical RSP correctness/expressivity BENCHMARK and
    // RSP-QL is a W3C-COMMUNITY spec — there is NO W3C/OGC/IETF Recommendation for
    // RDF Stream Processing and no normative RSP conformance test suite to point
    // at (unlike SPARQL/SHACL/GeoSPARQL/JSON-LD, the real-conformance rows above).
    // So `family = "sparq extension"` (the same NOT-a-standards-body marker the
    // BM25 row carries). The runner drives the crate's REAL public pipeline
    // (ContinuousQuery / ContinuousMultiQuery over WindowSpec windows,
    // EvalMode-materialised, R2S-filtered) across the SRBench expressivity axes —
    // window TYPES (tumbling + sliding time + CQL count windows), stream OPERATORS
    // (RSTREAM/ISTREAM/DSTREAM), all four EvalModes (pinned identical), and
    // multi-window JOINS (observations ⋈ station-metadata) with
    // aggregate-after-join — and asserts every closed window against an INDEPENDENT
    // oracle (a batch-rebuild via `sparq_engine::query` plus closed-form
    // hand-derived row multisets) that shares no code with the streaming closure /
    // materialisation path. Documented RSP-QL gaps (window VARIABLES, textual ROWS
    // windows, relative NOW bounds, I/DSTREAM over a join) are NOT inflated into
    // passes — they are asserted genuinely-rejected. The floor is the ACTUAL
    // measured count of those per-window correctness assertions (the
    // `assertions N (floor F)` line the runner prints) — it may only RISE, and a
    // drop is an S2R/R2R/EvalMode/join/R2S-diff regression. The runner stays
    // crate-local (the dev-only conformance crate must NOT take sparq-rsp as a dep
    // — same constraint as SHACL/geo/Solid/ODRL/text), so this row takes NO new
    // dependency edge on sparq-rsp; `RSP_EXPRESSIVITY_FLOOR` is mirrored here and
    // kept in lock-step by `tests/scoreboard_floors.rs` (read textually).
    Suite {
        label: "RSP expressivity / SRBench correctness",
        family: "sparq extension",
        runner: Runner::CrateTest { krate: "sparq-rsp", target: "srbench_oracle" },
        ci_job: "rsp-oracle",
        ratchet_floor: 317, // [SONNET-4.6] sq-2n1q3.3 raised from 303 (from 149 sq-mcb3q baseline)
        floor_basis: "per-window correctness assertions (sparq EXTENSION, NOT standards conformance)",
        note: "EXTENSION ratchet — no normative RDF-Stream-Processing standard / RSP \
               conformance suite exists (RSP-QL is a W3C-community spec; SRBench a \
               benchmark): sparq-rsp's windowed continuous queries vs an INDEPENDENT \
               batch-rebuild + closed-form oracle across window types / R2S operators \
               / EvalModes / multi-window joins; documented RSP-QL gaps asserted \
               genuinely-rejected, not faked as passes",
    },
    // [OPUS-4.8] sq-rh4gu (epic sq-pbz04) — the RIF-Core EXPRESSIVITY ratchet
    // (runner lives crate-local in `sparq-conformance/tests/rif_core_suite.rs`,
    // behind the opt-in `rif-core` feature). HONESTLY tallied as a sparq EXTENSION
    // ratchet, NOT folded into the conformance total — even though RIF-Core is a
    // real W3C dialect, this lane runs sparq's OWN faithful expressivity battery
    // over the RIF-Core (monotone Horn) subset the `sparq_reason::rif` front-end
    // implements, NOT the normative W3C SPARQL-RIF Core Entailment Regime test
    // suite (sparql11/entailment rif01..rif06, which is a strictly larger
    // SPARQL-protocol integration this bead does NOT deliver — it is a documented
    // tracked-not-asserted out-of-scope item in the runner's OUT_OF_SCOPE list).
    // So the unit is per-feature EXPRESSIVITY assertions (not normative-suite
    // pass-counts), and it is tallied SEPARATELY like the BM25 / RSP extension rows
    // — never inflating a conformance number with a non-conformance one. The runner
    // drives the REAL `rif::Document::{validate, closure}` path across the RIF-Core
    // axes (frame/membership/subclass/equality, recursion, the numeric/string/list
    // builtins with range-restriction SAFETY enforced, monotonicity, and the
    // canonical W3C `rif01` uncle rule), asserting every unsafe rule is GENUINELY
    // rejected and the NAF/nonmonotonic surface genuinely-absent. The floor is the
    // MEASURED assertion count (the `RIF-Core expressivity assertions N` line) — it
    // may only RISE; `RIF_CORE_FLOOR` is mirrored here and kept in lock-step by
    // `tests/scoreboard_floors.rs` (read textually).
    Suite {
        label: "RIF-Core expressivity (monotone Horn subset)",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "rif_core_suite",
            feature: "rif-core",
        },
        ci_job: "inference-conformance",
        // [SONNET-4.6] sq-pbz04.5.2 — raised 47 → 58: 11 new assertions for the 5
        // new soundly-mapped builtins (NumericNotEqual, StringUpperCase, StringLowerCase,
        // StringEncodeForUri, ListConcatenate).
        // [SONNET-4.6] sq-pbz04.5.4 — raised 58 → 61: Equal-atom audit (+3):
        // conclusion-rejection + ground-identity body Equal + DistinctGroundEqual
        // fail-closed (net +3: 4 added, 1 removed).
        ratchet_floor: 61,
        floor_basis: "expressivity assertions (sparq EXTENSION over the RIF-Core subset, \
                      NOT the normative W3C SPARQL-RIF conformance suite)",
        note: "EXTENSION ratchet — sparq's own faithful expressivity battery over the \
               RIF-CORE (monotone Horn) subset implemented by sparq-reason's rif:: \
               front-end (frame/membership/subclass/equality, recursion, numeric/string/\
               list builtins with range-restriction safety, monotonicity, the canonical \
               rif01 uncle rule); the normative SPARQL-RIF Core Entailment Regime + full \
               RIF-BLD/PRD are documented out-of-scope, asserted genuinely-rejected, \
               never faked as passes",
    },
    // [OPUS-4.8] sq-qo1a9 (epic sq-pbz04, the LAST conformance bead) — the GRADUATED
    // OWL 2 QL (DL-Lite_R) CERTAIN-ANSWER oracle ratchet (runner lives crate-local
    // in `sparq-conformance/tests/ql_dllite_suite.rs`, behind the opt-in
    // `ql-experimental` feature). HONESTLY tallied as a sparq EXTENSION ratchet, NOT
    // folded into the conformance total — even though OWL 2 QL is a real W3C profile,
    // there is NO runnable normative W3C "QL certain-answer conformance suite" to
    // point a harness at (the W3C QL material is structural/classification, not an
    // answer-comparison corpus over the query-rewriting semantics). So — exactly like
    // the RIF-Core / RSP / BM25 extension rows — this lane runs sparq's OWN faithful
    // DL-Lite_R certain-answer oracle (the hand-derived suite from sq-g19x0): each
    // case is a conjunctive query within SOUND DL-Lite_R rewriting, with a
    // certain-answer set derived BY HAND from the DL-Lite_R semantics. The runner
    // rewrites each case with the REAL `sparq_reason_ql::rewrite_production`
    // (PerfectRef ∪ tree-witness ∪ UCQ-min) and evaluates the UCQ over the UNMODIFIED
    // ABox through the REAL engine, asserting it returns EXACTLY the certain answers —
    // sound (no extra) AND complete (no missing) case by case. The floor is the
    // MEASURED count of sound-and-complete cases (the `QL DL-Lite_R sound-and-complete
    // N (floor F)` line) — it may only RISE; a divergence is a soundness/completeness
    // regression. The BROADER `pr:QL` `sparql11/entailment` arm (intensional /
    // non-DL-Lite certain-answer cases the rewriter cannot soundly answer) stays
    // EXPERIMENTAL / OutOfScope (`tests/ql_experimental_arm.rs`), NEVER summed in.
    // `QL_DLLITE_FLOOR` is mirrored here and kept in lock-step by
    // `tests/scoreboard_floors.rs` (read textually).
    Suite {
        label: "OWL 2 QL (DL-Lite_R) certain-answer oracle",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "ql_dllite_suite",
            feature: "ql-experimental",
        },
        ci_job: "inference-conformance",
        ratchet_floor: 11,
        floor_basis: "sound-and-complete certain-answer cases (sparq EXTENSION over the \
                      DL-Lite_R oracle, NOT a full-OWL-2-QL-conformance claim)",
        note: "EXTENSION ratchet — no runnable normative W3C OWL 2 QL certain-answer \
               conformance suite exists (the W3C QL material is structural): sparq's own \
               faithful DL-Lite_R certain-answer oracle (the hand-derived suite from \
               sq-g19x0), each conjunctive query rewritten by sparq_reason_ql's REAL \
               rewrite_production (PerfectRef ∪ tree-witness ∪ UCQ-min) and evaluated over \
               the UNMODIFIED ABox, asserting EXACTLY the hand-derived certain answers — \
               sound AND complete case by case; the broader pr:QL entailment-arm \
               intensional gap stays experimental/OutOfScope, never faked as a pass",
    },
    // [FABLE-5] sq-pbz04.3.4 (epic sq-pbz04.3) — the OWL 2 QL ENTAILMENT-REGIME
    // GRADUATED-SUBSET ratchet (runner lives crate-local in
    // `sparq-conformance/tests/ql_entailment_floor.rs`, behind the opt-in
    // `ql-experimental` feature). HONESTLY tallied as a sparq EXTENSION ratchet, NOT
    // folded into the conformance total: sparq implements a FRAGMENT of the W3C
    // SPARQL 1.1 QL entailment regime (holding everything else with an exhaustive
    // reason taxonomy), so no full-regime / full-profile OWL 2 QL conformance claim
    // is made anywhere. A `pr:QL` `sparql11/entailment` case is in the floor iff ALL
    // SIX graduation conditions pass — each CHECKED in code by
    // `inference::sparql_entail::run_ql_graduation`: (1) the fail-closed CQ-shape
    // gate accepts it and it carries no intensional schema-vocabulary atom; (2) the
    // DL-Lite_R TBox is totally captured (`fully_captured()`, sq-pbz04.3.3); (3)
    // zero consistency-relevant (negative/disjointness) axioms; (4) default-graph
    // dataset only; (5) the regime-coincidence guard (certain-answer vs
    // solution-mapping semantics provably coincide: all body terms distinguished, or
    // no existential-generating inclusions); (6) the rewritten UCQ evaluated over
    // the UNMODIFIED data is result-equivalent to the W3C oracle. The floor is a
    // PINNED NAMED-CASE list (exact set equality): a pinned case regressing fails
    // CI, and a newly-eligible case fails CI until pinned deliberately with
    // evidence. Distinct from the DL-Lite_R certain-answer oracle row above (the
    // hand-derived sq-g19x0 corpus, untouched); the inference BINARY keeps every QL
    // row OutOfScope so this floor can never leak into a conformance number.
    // `QL_ENTAILMENT_FLOOR` is mirrored here and kept in lock-step by
    // `tests/scoreboard_floors.rs` (read textually).
    Suite {
        label: "OWL 2 QL entailment-regime graduated subset",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "ql_entailment_floor",
            feature: "ql-experimental",
        },
        ci_job: "inference-conformance",
        // [SONNET-4.6] sq-pbz04.3.1 raised 9 → 11: `lang` + `plainLit` graduate under the
        // B2 literal-object broadening (SELECT ?x WHERE { ?x foaf:name "name"@en }, TBox
        // foaf:name a owl:DatatypeProperty — fully_captured, no existential generators,
        // identity rewrite returns exactly {:b}, result-equivalent to the W3C oracle).
        ratchet_floor: 11,
        floor_basis: "graduated named pr:QL cases — six-condition soundness predicate (sparq \
                      EXTENSION over the QL fragment sparq rewrites, NOT an OWL 2 QL / \
                      entailment-regime conformance claim)",
        note: "EXTENSION ratchet — the pr:QL sparql11/entailment cases that pass ALL SIX \
               graduation conditions (CQ-shape gate + intensional guard, total TBox capture, \
               zero consistency-relevant axioms, default-graph dataset, the regime-coincidence \
               guard, and empirical result-equivalence to the W3C oracle through the REAL \
               rewrite_production + engine), pinned as an exact named-case list; every \
               non-graduated case is held with an exhaustive reason taxonomy \
               (permanently-outside / pending-gate / pending-capture / pending-consistency / \
               pending-coincidence / oracle-divergent), never faked as a pass",
    },
    // [SONNET-4.6] sq-pbz04.2.4 (epic sq-pbz04) — the OWL 2 EL classification ratchet
    // (runner lives crate-local in `sparq-conformance/tests/el_suite.rs`, behind the
    // opt-in `el-suite` feature). HONESTLY tallied as a sparq EXTENSION ratchet, NOT
    // folded into the conformance total — even though OWL 2 EL is a real W3C profile,
    // this lane compares each W3C OWL 2 EL test (test:EL ∧ test:RDF-BASED, Approved,
    // no-imports) against what `sparq-reason-el`'s consequence-based classifier
    // (CR1–CR6 + safe nominals; `rbox`/`cdomain` are SEPARATELY gated) genuinely
    // computes over the EL fragment it implements — it is NOT a full OWL 2 EL
    // conformance claim. The runner classifies each premise through the REAL
    // `sparq_reason_el::classify_graph` (materializing the complete rdfs:subClassOf
    // subsumption lattice IN PLACE) and checks: consistency (no unsatisfiable named
    // class), inconsistency (some unsatisfiable named class — the TBox clash it can
    // see), positive-entailment (the materialized lattice ENTAILS the conclusion under
    // the shared bnode-homomorphism `entail::entails`), and negative-entailment (the
    // non-conclusion is NOT entailed). The floor is the MEASURED PASS count; the 28
    // audited PERMANENT divergences (ABox-only inconsistency / individual facts, RBox
    // property reasoning, owl:unionOf, or the owl:equivalentClass output-form) are
    // reported separately and NEVER summed into the floor. `EL_SUITE_FLOOR` is mirrored
    // here and kept in lock-step by `tests/scoreboard_floors.rs` (read textually).
    Suite {
        label: "OWL 2 EL classification (sparq-reason-el)",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "el_suite",
            feature: "el-suite",
        },
        ci_job: "inference-conformance",
        ratchet_floor: 50,
        floor_basis: "pass — classifier computes the expected outcome (sparq EXTENSION over \
                      the EL fragment the classifier implements, NOT a full-OWL-2-EL-conformance \
                      claim; CR7–CR9 concrete domains + ABox inconsistency deferred)",
        note: "EXTENSION ratchet — the W3C OWL 2 EL suite checks classification/subsumption \
               semantics and this runner compares against what sparq-reason-el's classifier \
               computes over the EL fragment it implements (CR1–CR6 + safe nominals): each \
               premise classified through the REAL classify_graph, then consistency / \
               inconsistency / positive- / negative-entailment via the shared \
               bnode-homomorphism check; the 28 audited ABox / RBox / owl:unionOf / \
               equivalentClass-form divergences are reported separately, never faked as passes",
    },
    // [FABLE-5] sq-pbz04.4.5 (epic sq-pbz04.4) — the OWL 2 DIRECT-SEMANTICS arm's two
    // ratchets (runner: `inference::dl_suite` + the crate-local `tests/dl_suite.rs`,
    // behind the opt-in `dl-direct` feature → `sparq-reason-dl/dispatch`). HONESTLY
    // tallied as sparq EXTENSION rows over the SCOPED FRAGMENT the layered
    // `sparq-reason-dl` checker implements — **scoped fragment, NOT full OWL 2 DL** —
    // and never folded into the standards-conformance total. TRI-STATE accounting
    // {Pass, Fail, OutOfFragment(reason)}: an ABSTENTION IS NEVER A PASS, every
    // divergent definitive verdict is pinned BY NAME with an audited mechanism in the
    // runner (exact set equality), and the floors are EXACT-pinned in-runner (`==`,
    // not `>=`, so abstention-inflation and regression BOTH fail CI). The
    // profile-identification row checks POSITIVE `test:profile` tags only (the design
    // record §4 fallback — L2's `In` is fragment-grammar membership and cannot refute
    // full-profile membership, a limit the lane MEASURED); the Direct row runs
    // consistency / inconsistency / positive- / negative-entailment through the L4
    // `DirectChecker` dispatch under a PINNED deterministic count budget (wall-clock
    // budgets banned). The dual-tagged tests' RDF-Based runs stay in the inference
    // binary's RL `owl_suite` / the `el-suite` lane — one test may appear in both
    // tallies because the two runs test DIFFERENT semantics (record §4). Floors read
    // TEXTUALLY by `tests/scoreboard_floors.rs`.
    Suite {
        label: "OWL 2 DL profile identification (Direct arm)",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "dl_suite",
            feature: "dl-direct",
        },
        ci_job: "inference-conformance",
        ratchet_floor: 68,
        floor_basis: "positive-tag membership passes, EXACT-pinned (sparq EXTENSION over the \
                      L1/L2 ALCH-fragment checker — scoped fragment, NOT full OWL 2 DL and NOT \
                      a W3C ProfileIdentificationTest conformance claim)",
        note: "EXTENSION ratchet — the DIRECT-arm ProfileIdentificationTest cases whose \
               POSITIVE test:profile tags the L2 syntactic checker reproduces through the \
               REAL fail-closed L1 extraction + grammar walk; explicit-negative and species \
               assertions are not checked (documented), abstentions are never passes, and \
               the 30 singleton-intersection divergences are pinned by name",
    },
    Suite {
        label: "OWL 2 Direct-Semantics consistency + entailment (scoped fragment)",
        family: "sparq extension",
        runner: Runner::FeatureGatedCrateTest {
            krate: "sparq-conformance",
            target: "dl_suite",
            feature: "dl-direct",
        },
        ci_job: "inference-conformance",
        ratchet_floor: 192,
        floor_basis: "definitive expected verdicts through the L4 dispatch, EXACT-pinned \
                      (sparq EXTENSION over the scoped fragment — NOT full OWL 2 DL); \
                      re-pinned by sq-pbz04.4.11 (M1 named-composite fix: +12 positive- \
                      entailment passes, -4 consistency passes shifted to abstain, net +8)",
        note: "EXTENSION ratchet — the DIRECT-arm consistency / inconsistency / positive- / \
               negative-entailment tests decided by the REAL sparq-reason-dl L4 dispatch \
               (RL guarded / EL guarded / QL deferred / ALCH tableau) under a pinned \
               deterministic count budget; fail-closed abstentions are reported, never \
               passes, and all 11 wrong-verdict divergences are pinned by name with audited \
               mechanisms (M1 fixed by sq-pbz04.4.11; M4 orphan-list fidelity gap still \
               open, held by follow-up bead sq-pbz04.4.12)",
    },
];

/// Render the registry as one markdown scoreboard. This is a STATIC view of what
/// the project ratchets and where each suite is enforced — it does not re-run the
/// suites (each runner owns its own data fetch + execution), so it is hermetic and
/// fast, and serves as the single index CI surfaces alongside the per-suite
/// reports.
pub fn render_scoreboard() -> String {
    use std::fmt::Write;
    let mut md = String::new();
    let _ = writeln!(md, "# sparq conformance scoreboard\n");
    let _ = writeln!(
        md,
        "The single index of EVERY conformance suite sparq ratchets, across crates. \
         Each suite has a FLOOR that CI enforces: a pass-count (or pass+divergence) \
         floor that may only RISE, or — for the differential oracles — a hard \
         divergence-count floor of 0. The per-suite detail reports are produced by the \
         runners in the *run* column; this table is the consolidated map (sq-ncvq.16 \
         brought the SHACL + GeoSPARQL ratchets in; sq-j174 the Solid WAC + ACP \
         decision-parity ones; sq-t58w.8 the Solid WAC + ACP differential oracles — \
         all previously lived outside this scoreboard).\n\n\
         The `family = sparq extension` rows are HONESTLY sparq-extension ratchets, \
         NOT standards-conformance claims — there is no normative body or test suite \
         behind them (sq-ripcg's BM25 differential oracle: no full-text-over-RDF / BM25 \
         standard exists; sq-mcb3q's RSP expressivity / SRBench correctness oracle: no \
         normative RDF-Stream-Processing standard exists — RSP-QL is a W3C-community \
         spec and SRBench a benchmark). Their floors are in a different UNIT \
         (per-window / per-hit correctness assertions, not spec pass-counts), so the \
         consolidated total below counts ONLY the standards-conformance suites; the \
         extension rows are reported separately.\n"
    );
    let _ = writeln!(
        md,
        "| suite | family | floor | basis | CI job | run |"
    );
    let _ = writeln!(md, "|---|---|---:|---|---|---|");
    for s in SUITES {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | `{}` | `{}` |",
            s.label,
            s.family,
            s.ratchet_floor,
            s.floor_basis,
            s.ci_job,
            s.runner.command(),
        );
    }
    // The consolidated total counts ONLY standards-conformance suites — the
    // `sparq extension` rows (sq-ripcg) carry a different-unit floor (score-exact
    // assertions, not spec pass-counts), so summing them in would inflate a
    // "conformance" number with a non-conformance one. They are reported on their
    // own line. [OPUS-4.8] sq-ripcg
    let is_extension = |s: &&Suite| s.family == "sparq extension";
    let conformance: Vec<&Suite> = SUITES.iter().filter(|s| !is_extension(s)).collect();
    let extensions: Vec<&Suite> = SUITES.iter().filter(is_extension).collect();
    let total: usize = conformance.iter().map(|s| s.ratchet_floor).sum();
    let _ = writeln!(
        md,
        "| **conformance total ({} suites)** | | **{}** | | | |",
        conformance.len(),
        total
    );
    if !extensions.is_empty() {
        let ext_total: usize = extensions.iter().map(|s| s.ratchet_floor).sum();
        // [OPUS-4.8] sq-mcb3q: pluralise now there is more than one extension row.
        let row_word = if extensions.len() == 1 { "row" } else { "rows" };
        let _ = writeln!(
            md,
            "| **sparq-extension ({} {}, NOT conformance)** | | {} | (assertions) | | |\n",
            extensions.len(),
            row_word,
            ext_total
        );
    } else {
        let _ = writeln!(md);
    }
    let _ = writeln!(
        md,
        "Notes:\n\n{}",
        SUITES
            .iter()
            .map(|s| format!("- **{}**: {}", s.label, s.note))
            .collect::<Vec<_>>()
            .join("\n")
    );
    md
}
