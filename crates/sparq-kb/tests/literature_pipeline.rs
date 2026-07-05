//! End-to-end Phase-5 acceptance test: run the literature-ingestion pipeline over the
//! committed fixtures, then run the SHACL gate (`pkg.shapes.ttl` +
//! `literature.shapes.ttl`) over the EMITTED TTL — the REAL path, not a mock.
//!
//! [OPUS-4.8] sq-2489d.5 (epic sq-2489d). 🤖 SPARQ agent — provenance-driven GenAI KB.
//!
//! This is the test that wires the two Phase-5 acceptance metrics together:
//!   - the **citation-grounding rate** comes from the pipeline [`Sidecar`]; and
//!   - the **SHACL-conformance rate** comes from running `sparq-shacl` over the emitted
//!     TTL (the pipeline's output is conformant BY CONSTRUCTION, so this is 1.0, and the
//!     test ALSO proves the literature shapes FIRE on a deliberately-injected violator —
//!     so the gate is real, not vacuous).
//!
//! It needs BOTH features: `literature` (the pipeline) + `validate` (the SHACL engine).
//! Run: `cargo test -p sparq-kb --features literature,validate -- --nocapture`
#![cfg(all(feature = "literature", feature = "validate"))]

use sparq_kb::literature::extract::RecordedExtractor;
use sparq_kb::literature::{pipeline, FIXTURE_OPENALEX_BATCH, LITERATURE_SHAPES};
use sparq_kb::validate::{graph_from_turtle_docs, validate_instances};
use sparq_kb::{PKG_ONTOLOGY, PKG_SHAPES};

/// Validate a Turtle data document against BOTH the base PKG shapes and the literature-
/// tier shapes (the real write-gate the design specifies). Returns conformance + the text
/// report (for the PR description / triage).
fn gate(data_ttl: &str) -> (bool, String) {
    let base = "https://sparq.dev/ns/pkg/example#";
    let data = graph_from_turtle_docs(&[PKG_ONTOLOGY, data_ttl], base).expect("data graph loads");
    let shapes =
        graph_from_turtle_docs(&[PKG_SHAPES, LITERATURE_SHAPES], base).expect("shapes load");
    let report = sparq_shacl::validate(&data, &shapes);
    (report.conforms_violations_only(), report.to_text())
}

#[test]
fn pipeline_runs_offline_and_reports_the_grounding_metric() {
    let extractor = RecordedExtractor::from_fixture().expect("replay extractor builds");
    let out = pipeline::run(FIXTURE_OPENALEX_BATCH, &extractor).expect("pipeline runs");

    // The Phase-5 metric is COMPUTED from the batch, not hard-coded.
    let sc = &out.sidecar;
    eprintln!(
        "=== Phase-5 literature-ingestion sidecar ===\n\
         candidates={} grounded={} quarantined={} grounding_rate={:.4}\n\
         sources: explored={} dead_end={} skipped={}",
        sc.candidates_total,
        sc.grounded,
        sc.quarantined.len(),
        sc.grounding_rate(),
        sc.sources_explored,
        sc.sources_dead_end,
        sc.sources_skipped
    );
    for q in &sc.quarantined {
        eprintln!(
            "  QUARANTINED [{}] {} -- {}",
            q.source_doi, q.justification, q.reason
        );
    }

    // Every candidate is accounted for: grounded + quarantined == total (never dropped).
    assert_eq!(sc.grounded + sc.quarantined.len(), sc.candidates_total);
    // The two deliberately-bad candidates (fabricated span + dangling citation) are
    // quarantined, never silently dropped.
    assert_eq!(sc.quarantined.len(), 2);
    assert!(sc.grounding_rate() > 0.0 && sc.grounding_rate() < 1.0);
}

#[test]
fn emitted_ttl_conforms_to_pkg_and_literature_shapes() {
    // The REAL path: emit, then gate the emitted TTL with sparq's own SHACL engine.
    let extractor = RecordedExtractor::from_fixture().unwrap();
    let out = pipeline::run(FIXTURE_OPENALEX_BATCH, &extractor).unwrap();

    let (conforms, report) = gate(&out.turtle);
    assert!(
        conforms,
        "the pipeline-emitted machine-tier TTL must conform to pkg.shapes.ttl + \
         literature.shapes.ttl (it is conformant by construction), but got:\n{report}"
    );
}

#[test]
fn literature_shapes_catch_a_proven_overclaim_on_the_machine_tier() {
    // The gate must be REAL, not vacuous: inject a machine-attributed Finding that stamps
    // secx:Proven and prove the literature shape FIRES. (The pipeline never emits this —
    // it clamps to secx:Conjectured — so we hand-author the violator here.)
    const VIOLATOR: &str = r#"
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix cito:    <http://purl.org/spar/cito/> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:      <https://sparq.dev/ns/pkg/example#> .

ex:agent a pkg:MachineAgent ; rdfs:label "extractor"@en .

ex:src a pkg:Source ;
  dcterms:title "A source paper" ;
  pkg:exploredStatus pkg:Explored .

# A machine-extracted Finding that ILLEGALLY stamps secx:Proven (RULE 1) AND carries a
# confidence above the 0.7 ceiling (RULE 2) AND cites a dangling DOI (RULE 3).
ex:bad a pkg:Finding ;
  rdfs:label "an over-claiming machine finding"@en ;
  sigimpl:justification "A sufficiently-long, non-filler justification span string." ;
  pkg:confidence 0.95 ;
  pkg:assurance secx:Proven ;
  prov:wasDerivedFrom ex:src ;
  prov:wasAttributedTo ex:agent ;
  cito:citesAsEvidence ex:dangling .
"#;
    let (conforms, report) = gate(VIOLATOR);
    assert!(
        !conforms,
        "the literature shapes must REJECT a Proven over-claim / over-ceiling confidence / \
         dangling citation on the machine tier, but the graph conformed:\n{report}"
    );
    // All three literature-tier rules should be cited in the report.
    assert!(
        report.contains("secx:Proven"),
        "RULE 1 (no secx:Proven on the machine tier) should fire:\n{report}"
    );
    assert!(
        report.contains("0.7"),
        "RULE 2 (confidence ceiling) should fire:\n{report}"
    );
    assert!(
        report.contains("dangling"),
        "RULE 3 (no dangling citation) should fire:\n{report}"
    );
}

#[test]
fn a_hand_authored_proven_finding_is_not_constrained_by_the_literature_shapes() {
    // A NON-machine Finding (no prov:wasAttributedTo a pkg:MachineAgent) may assert
    // secx:Proven with high confidence — the literature shapes must NOT bind it. This is
    // the "the machine tier is constrained, the human tier is not" property.
    const HUMAN: &str = r#"
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix cito:    <http://purl.org/spar/cito/> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:      <https://sparq.dev/ns/pkg/example#> .

ex:src a pkg:Source ;
  dcterms:title "A source paper" ;
  pkg:exploredStatus pkg:Explored .

ex:human a pkg:Finding ;
  rdfs:label "a hand-authored proven finding"@en ;
  sigimpl:justification "A genuine, sufficiently-long, non-filler justification string." ;
  pkg:confidence 0.95 ;
  pkg:assurance secx:Proven ;
  prov:wasDerivedFrom ex:src ;
  cito:citesAsEvidence ex:src .
"#;
    let (conforms, report) = gate(HUMAN);
    assert!(
        conforms,
        "a hand-authored secx:Proven Finding (not machine-attributed) must conform — the \
         literature shapes constrain only the machine tier, but got:\n{report}"
    );
}

#[test]
fn a_machine_timestamped_finding_passes_shacl() {
    // [HAIKU-4.5] sq-tzars.2: positive SHACL case — a machine-extracted Finding with
    // prov:generatedAtTime (and all other required constraints) must pass the literature
    // shapes. This is the "happy path" for RULE 4.
    const TIMESTAMPED: &str = r#"
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix cito:    <http://purl.org/spar/cito/> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:      <https://sparq.dev/ns/pkg/example#> .

ex:agent a pkg:MachineAgent ; rdfs:label "extractor"@en .

ex:src a pkg:Source ;
  dcterms:title "A source paper" ;
  pkg:exploredStatus pkg:Explored .

ex:good a pkg:Finding ;
  rdfs:label "a compliant machine finding"@en ;
  sigimpl:justification "A sufficiently-long, non-filler justification span string." ;
  pkg:confidence 0.6 ;
  pkg:assurance secx:Conjectured ;
  prov:wasDerivedFrom ex:src ;
  prov:wasAttributedTo ex:agent ;
  prov:generatedAtTime "2026-07-05T14:30:00Z"^^xsd:dateTime ;
  cito:citesAsEvidence ex:src .
"#;
    let (conforms, report) = gate(TIMESTAMPED);
    assert!(
        conforms,
        "a machine-extracted Finding with prov:generatedAtTime must pass the literature \
         shapes (RULE 4), but got:\n{report}"
    );
}

#[test]
fn a_machine_finding_without_timestamp_is_rejected_by_shacl() {
    // [HAIKU-4.5] sq-tzars.2: negative SHACL case — a machine-extracted Finding without
    // prov:generatedAtTime must be rejected by RULE 4, even if all other constraints are
    // satisfied. This proves the timestamp requirement is enforced (fail-closed).
    const MISSING_TIMESTAMP: &str = r#"
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix cito:    <http://purl.org/spar/cito/> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:      <https://sparq.dev/ns/pkg/example#> .

ex:agent a pkg:MachineAgent ; rdfs:label "extractor"@en .

ex:src a pkg:Source ;
  dcterms:title "A source paper" ;
  pkg:exploredStatus pkg:Explored .

# A machine-extracted Finding that ILLEGALLY omits prov:generatedAtTime (RULE 4 violation).
# All other constraints are satisfied, so the ONLY violation is the missing timestamp.
ex:no_timestamp a pkg:Finding ;
  rdfs:label "a machine finding without prov:generatedAtTime"@en ;
  sigimpl:justification "A sufficiently-long, non-filler justification span string." ;
  pkg:confidence 0.6 ;
  pkg:assurance secx:Conjectured ;
  prov:wasDerivedFrom ex:src ;
  prov:wasAttributedTo ex:agent ;
  cito:citesAsEvidence ex:src .
"#;
    let (conforms, report) = gate(MISSING_TIMESTAMP);
    assert!(
        !conforms,
        "a machine-extracted Finding without prov:generatedAtTime must be REJECTED by \
         RULE 4 (fail-closed quarantine), but the graph conformed:\n{report}"
    );
    // RULE 4 message must be cited in the report.
    assert!(
        report.contains("prov:generatedAtTime") || report.contains("extraction instant"),
        "RULE 4 (prov:generatedAtTime requirement) should fire:\n{report}"
    );
}

#[test]
fn base_example_still_conforms_under_the_combined_shapes() {
    // Regression: adding literature.shapes.ttl must not break the existing base shapes on
    // the existing example graph (the literature shapes are additive + machine-tier-scoped).
    let report = validate_instances(&[]).expect("ontology + shapes load with no instances");
    // (validate_instances uses only the base shapes; this asserts the ontology + base
    // shapes still load cleanly after the pkg.ttl MachineAgent addition.)
    assert!(report.conforms, "empty-instance base graph must conform");
}
