//! The **pipeline** — wires `[connector] -> [normalise] -> [extract (replay)] ->
//! [ground] -> [emit TTL] -> [sidecar]` over a committed fixture batch, with ZERO network
//! and ZERO live-model calls. (§4.7, the projector that extends `ingest_pkg.py` — realised
//! in Rust behind the `literature` feature.)
//!
//! [OPUS-4.8] sq-2489d.5. The pipeline is the Phase-5 deliverable: it produces (a) a
//! Turtle document of the *grounded, machine-tier* `pkg:Source` + `pkg:Finding` triples
//! with full PROV-O lineage, and (b) a [`Sidecar`] reporting the citation-grounding rate +
//! the quarantine list. The caller runs the SHACL gate (`pkg.shapes.ttl` +
//! `shapes/literature.shapes.ttl`) over the emitted TTL to obtain the SHACL-conformance
//! rate — see `tests/literature_pipeline.rs`.
//!
//! The committed Findings are deliberately NOT appended to `ingest/pkg-instances.ttl`:
//! Phase 5 ships the *scaffolding* and proves it on fixtures; bulk ingestion of real
//! literature is gated behind the Phase-6 live pilot's per-topic recommend-adopt verdict.

use std::fmt::Write as _;

use super::connector::{parse_openalex_batch, SourceStub};
use super::extract::{CandidateFinding, Extractor};
use super::ground::{self, GroundingFailure};

/// The machine-extraction agent IRI every emitted Finding is `prov:wasAttributedTo`. Typed
/// `pkg:MachineAgent`, so the literature-tier SHACL shapes bind to it (and a hand-authored
/// Finding, which is NOT attributed to it, is unconstrained by those shapes).
pub const MACHINE_AGENT_IRI: &str = "https://sparq.dev/ns/pkg/agent#literature-extractor";

/// The confidence ceiling the machine tier is capped at (mirrors the
/// `shapes/literature.shapes.ttl` RULE 2). A candidate proposing more is clamped to this
/// at emit time so the emitted TTL is conformant by construction; the SHACL shape is the
/// durable backstop. NOT a calibrated number (calibration is Phase 6) — a declarative
/// "a cheap extractor is not high-confidence" guard.
pub const MACHINE_CONFIDENCE_CEILING: f64 = 0.7;

/// One quarantined candidate (recorded, never silently dropped — the sidecar-honesty
/// pattern §4.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quarantined {
    /// The source DOI the quarantined candidate came from.
    pub source_doi: String,
    /// The (truncated) justification, for human triage.
    pub justification: String,
    /// Why it was quarantined.
    pub reason: String,
}

/// The pipeline sidecar — the Phase-5 metric carrier. Reports the citation-grounding rate
/// and the quarantine list for one fixture batch. No hard-coded number lives here: the
/// rate is computed from the batch at run time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sidecar {
    /// Total candidate Findings the extractor proposed across all sources.
    pub candidates_total: usize,
    /// Candidates that PASSED the deterministic grounding-resolver (committed to the TTL).
    pub grounded: usize,
    /// Candidates QUARANTINED by grounding (recorded here, not dropped).
    pub quarantined: Vec<Quarantined>,
    /// Sources the connector skipped (no DOI / no title) — surfaced, not lost.
    pub sources_skipped: usize,
    /// Distinct sources that produced ≥1 grounded Finding (flipped to `pkg:Explored`).
    pub sources_explored: usize,
    /// Distinct sources that produced 0 grounded Findings (`pkg:DeadEnd`).
    pub sources_dead_end: usize,
}

impl Sidecar {
    /// The **citation-grounding rate**: grounded candidates / total candidates, in `[0,1]`.
    /// `1.0` when there were no candidates (vacuously grounded — nothing ungrounded slipped
    /// through). This is one of the two Phase-5 acceptance metrics.
    pub fn grounding_rate(&self) -> f64 {
        if self.candidates_total == 0 {
            return 1.0;
        }
        self.grounded as f64 / self.candidates_total as f64
    }
}

/// The output of one pipeline run: the emitted Turtle (grounded machine-tier triples with
/// full PROV-O lineage) plus the [`Sidecar`].
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// The emitted Turtle document (a complete, parseable graph: prefixes + the agent +
    /// the Sources + the grounded Findings + their extraction Activities). Conforms to
    /// `pkg.shapes.ttl` + `shapes/literature.shapes.ttl` by construction.
    pub turtle: String,
    /// The run sidecar (the Phase-5 metric + quarantine list).
    pub sidecar: Sidecar,
    /// The timestamp (UTC ISO 8601 xsd:dateTime) stamped on all emitted Findings and
    /// their extraction Activities via `prov:generatedAtTime`. In CI, this is injectable
    /// from the test fixtures; in live use, it defaults to the wall-clock instant.
    /// (sq-tzars.2 [HAIKU-4.5])
    pub generated_at_time: String,
}

/// Generate the current UTC time as an ISO 8601 xsd:dateTime string (e.g.,
/// `2026-07-05T14:30:00Z`). Used as the default `prov:generatedAtTime` when the caller
/// does not inject a fixed timestamp (as CI tests do for determinism).
/// (sq-tzars.2 [HAIKU-4.5])
fn current_generated_at_time() -> String {
    // [HAIKU-4.5] In production, this would use `std::time::SystemTime::now()` or
    // similar to get the wall-clock instant; for now, we use a deterministic format.
    // Tests inject a fixed instant via the caller's `generated_at_time` param.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // Format as ISO 8601 UTC.
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    // Days since UNIX epoch (1970-01-01).
    let days = secs / 86400;
    let secs_today = secs % 86400;
    let hours = secs_today / 3600;
    let mins = (secs_today % 3600) / 60;
    let secs_min = secs_today % 60;
    // Julian day to calendar date (Gregorian calendar).
    let a = (days + 32044) as i64;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4800 + m / 10;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        year,
        month,
        day,
        hours,
        mins,
        secs_min,
        nanos / 1000
    )
}

/// Run the full pipeline over a recorded connector batch + a pluggable [`Extractor`]
/// (in CI, the replay [`super::extract::RecordedExtractor`]). Pure + deterministic; the
/// ONLY non-determinism a real run would have is inside the extractor, which is exactly
/// why it is a trait. NO network.
///
/// # Arguments
///
/// * `connector_json` - The OpenAlex batch JSON fixture (string)
/// * `extractor` - A trait object implementing [`Extractor`]
/// * `generated_at_time` - Optional override for the `prov:generatedAtTime` timestamp.
///   If `None`, uses the current UTC time. CI tests pass a fixed instant for determinism.
///   (sq-tzars.2 [HAIKU-4.5])
pub fn run_with_time<E: Extractor>(
    connector_json: &str,
    extractor: &E,
    generated_at_time: Option<String>,
) -> Result<PipelineOutput, String> {
    // 1. connector -> normalise.
    let (stubs, sources_skipped) = parse_openalex_batch(connector_json)?;
    // 2. extract (replay in CI).
    let candidates = extractor.extract(&stubs)?;
    // 3. ground (propose-then-verify).
    let (abstracts, source_dois) = ground::index(&stubs);

    let mut sidecar = Sidecar {
        candidates_total: candidates.len(),
        sources_skipped,
        ..Sidecar::default()
    };

    // Per source: collect the grounded candidates so we can flip exploredStatus.
    let mut grounded_by_doi: std::collections::HashMap<String, Vec<&CandidateFinding>> =
        std::collections::HashMap::new();

    for c in &candidates {
        match ground::verify(c, &abstracts, &source_dois) {
            Ok(()) => {
                sidecar.grounded += 1;
                grounded_by_doi
                    .entry(c.source_doi.clone())
                    .or_default()
                    .push(c);
            }
            Err(failure) => sidecar.quarantined.push(quarantine(c, &failure)),
        }
    }

    // exploredStatus per source: Explored iff it produced >=1 grounded Finding, else DeadEnd.
    for stub in &stubs {
        if grounded_by_doi
            .get(&stub.doi)
            .is_some_and(|v| !v.is_empty())
        {
            sidecar.sources_explored += 1;
        } else {
            sidecar.sources_dead_end += 1;
        }
    }

    let gen_time = generated_at_time.unwrap_or_else(current_generated_at_time);
    // 4. emit TTL.
    let turtle = emit_turtle(&stubs, &grounded_by_doi, &gen_time)?;
    Ok(PipelineOutput {
        turtle,
        sidecar,
        generated_at_time: gen_time,
    })
}

/// Run the full pipeline with the default (current) timestamp.
/// Convenience wrapper around [`run_with_time`] for callers that do not need to inject
/// a fixed instant. (sq-tzars.2 [HAIKU-4.5])
pub fn run<E: Extractor>(connector_json: &str, extractor: &E) -> Result<PipelineOutput, String> {
    run_with_time(connector_json, extractor, None)
}

/// Record a quarantined candidate (truncating the justification for triage).
fn quarantine(c: &CandidateFinding, failure: &GroundingFailure) -> Quarantined {
    const MAX: usize = 80;
    let just = if c.justification.chars().count() > MAX {
        let truncated: String = c.justification.chars().take(MAX).collect();
        format!("{}…", truncated)
    } else {
        c.justification.clone()
    };
    Quarantined {
        source_doi: c.source_doi.clone(),
        justification: just,
        reason: failure.reason(),
    }
}

/// Escape a string for a Turtle `"…"` long-string-safe short literal: backslash, quote,
/// newline, CR, tab. (We emit short `"…"` literals; a justification is one line.)
fn ttl_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Clamp the machine-tier confidence to the ceiling and force the assurance to
/// `secx:Conjectured` (the machine tier never outranks a human, §4.3). The emitted TTL is
/// therefore conformant to the literature shapes BY CONSTRUCTION; the shapes are the
/// durable backstop, not the only line of defence.
fn machine_tier(confidence: f64) -> f64 {
    confidence.min(MACHINE_CONFIDENCE_CEILING)
}

/// Emit the Turtle document: prefixes + the machine agent + each Source (with its grounded
/// Findings + extraction Activity + full PROV-O lineage). Deterministic ordering (input
/// order) so the output is stable. Returns a parseable graph string.
///
/// Each Finding and its extraction Activity are stamped with `prov:generatedAtTime`
/// (the `generated_at_time` parameter, an ISO 8601 xsd:dateTime string), as required
/// by the literature-tier SHACL shapes. (sq-tzars.2 [HAIKU-4.5])
fn emit_turtle(
    stubs: &[SourceStub],
    grounded_by_doi: &std::collections::HashMap<String, Vec<&CandidateFinding>>,
    generated_at_time: &str,
) -> Result<String, String> {
    let mut t = String::new();
    // Prefixes (the secx: namespace is the CANONICAL pkg one: w3id zkp-sparql sec-prop).
    t.push_str(
        "@prefix pkg:     <https://sparq.dev/ns/pkg#> .\n\
         @prefix prov:    <http://www.w3.org/ns/prov#> .\n\
         @prefix dcterms: <http://purl.org/dc/terms/> .\n\
         @prefix cito:    <http://purl.org/spar/cito/> .\n\
         @prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .\n\
         @prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .\n\
         @prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .\n\n",
    );

    // The extraction agent (typed pkg:MachineAgent so the literature shapes bind).
    writeln!(
        t,
        "<{}> a pkg:MachineAgent ;\n  rdfs:label \"literature extraction agent (Phase-5 scaffolding, fixtures)\"@en .\n",
        MACHINE_AGENT_IRI
    )
    .map_err(|e| e.to_string())?;

    let mut finding_n = 0usize;
    for stub in stubs {
        let src = stub.source_iri();
        let grounded = grounded_by_doi.get(&stub.doi);
        let explored = grounded.is_some_and(|v| !v.is_empty());
        let status = if explored {
            "pkg:Explored"
        } else {
            "pkg:DeadEnd"
        };

        // The Source node (DOI-keyed, content-addressed).
        write!(
            t,
            "<{src}> a pkg:Source ;\n  dcterms:title \"{title}\" ;\n  pkg:exploredStatus {status}",
            src = src,
            title = ttl_escape(&stub.title),
            status = status,
        )
        .map_err(|e| e.to_string())?;
        if let Some(y) = stub.year {
            write!(t, " ;\n  dcterms:issued \"{}\"^^xsd:gYear", y).map_err(|e| e.to_string())?;
        }
        t.push_str(" .\n");

        // The grounded Findings for this source.
        if let Some(cands) = grounded {
            for c in cands {
                finding_n += 1;
                let f = format!("{}/finding/{}", MACHINE_AGENT_IRI, finding_n);
                let act = format!("{}/activity/{}", MACHINE_AGENT_IRI, finding_n);
                let conf = machine_tier(c.confidence);
                // Full PROV-O lineage: derived-from the Source, generated-by the extraction
                // Activity, attributed-to the machine agent. Machine tier => secx:Conjectured.
                // [HAIKU-4.5] sq-tzars.2: stamp prov:generatedAtTime on both the Finding
                // and the Activity so the SHACL timestamp requirement can be verified.
                writeln!(
                    t,
                    "<{f}> a pkg:Finding ;\n  \
                     rdfs:label \"{verdict} (machine-extracted)\"@en ;\n  \
                     sigimpl:justification \"{just}\" ;\n  \
                     pkg:confidence {conf:.4} ;\n  \
                     pkg:assurance secx:Conjectured ;\n  \
                     prov:wasDerivedFrom <{src}> ;\n  \
                     prov:wasGeneratedBy <{act}> ;\n  \
                     prov:wasAttributedTo <{agent}> ;\n  \
                     prov:generatedAtTime \"{generated_at_time}\"^^xsd:dateTime ;\n  \
                     cito:citesAsEvidence <{src}> .\n\
                     <{act}> a prov:Activity ;\n  \
                     prov:used <{src}> ;\n  \
                     prov:wasAssociatedWith <{agent}> ;\n  \
                     prov:generatedAtTime \"{generated_at_time}\"^^xsd:dateTime .\n",
                    f = f,
                    verdict = ttl_escape(&c.verdict),
                    just = ttl_escape(&c.justification),
                    conf = conf,
                    src = src,
                    act = act,
                    agent = MACHINE_AGENT_IRI,
                    generated_at_time = generated_at_time,
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::literature::extract::RecordedExtractor;

    fn run_fixture() -> PipelineOutput {
        let extractor = RecordedExtractor::from_fixture().unwrap();
        run(crate::literature::FIXTURE_OPENALEX_BATCH, &extractor).unwrap()
    }

    #[test]
    fn pipeline_grounds_some_and_quarantines_the_rest() {
        let out = run_fixture();
        // 6 candidates total; 3 are deliberately bad (fabricated span, Proven over-claim
        // grounds fine but stays Conjectured, dangling citation). The two bad-by-grounding
        // (fabricated span + dangling citation) are quarantined.
        assert_eq!(out.sidecar.candidates_total, 6);
        assert_eq!(out.sidecar.grounded + out.sidecar.quarantined.len(), 6);
        // Exactly 2 candidates fail grounding (the fabricated span + the dangling cite).
        assert_eq!(out.sidecar.quarantined.len(), 2);
        assert_eq!(out.sidecar.grounded, 4);
        // Nothing was silently dropped.
        assert!(out
            .sidecar
            .quarantined
            .iter()
            .any(|q| q.reason.contains("not a span")));
        assert!(out
            .sidecar
            .quarantined
            .iter()
            .any(|q| q.reason.contains("dangling") || q.reason.contains("does not resolve")));
    }

    #[test]
    fn grounding_rate_is_computed_not_hardcoded() {
        let out = run_fixture();
        let expected = out.sidecar.grounded as f64 / out.sidecar.candidates_total as f64;
        assert!((out.sidecar.grounding_rate() - expected).abs() < 1e-12);
    }

    #[test]
    fn emitted_turtle_caps_the_machine_tier() {
        let out = run_fixture();
        // The Proven over-claim (confidence 0.9) grounds fine but is emitted as
        // secx:Conjectured with confidence clamped to the 0.7 ceiling.
        assert!(out.turtle.contains("pkg:assurance secx:Conjectured"));
        assert!(!out.turtle.contains("secx:Proven"));
        assert!(out.turtle.contains("0.7000"));
        // Full PROV-O lineage is present.
        assert!(out.turtle.contains("prov:wasAttributedTo"));
        assert!(out.turtle.contains("prov:wasGeneratedBy"));
        assert!(out.turtle.contains("a pkg:MachineAgent"));
        // [HAIKU-4.5] sq-tzars.2: prov:generatedAtTime stamped on Findings + Activities.
        assert!(
            out.turtle.contains("prov:generatedAtTime"),
            "emitted Findings must carry prov:generatedAtTime for SHACL conformance"
        );
        assert!(
            out.turtle.contains("^^xsd:dateTime"),
            "prov:generatedAtTime must be typed as xsd:dateTime"
        );
    }

    #[test]
    fn explored_status_flips_per_source() {
        let out = run_fixture();
        // All three sources produced >=1 grounded finding, so all are Explored.
        assert_eq!(out.sidecar.sources_explored, 3);
        assert_eq!(out.sidecar.sources_dead_end, 0);
        assert!(out.turtle.contains("pkg:exploredStatus pkg:Explored"));
    }

    #[test]
    fn run_with_time_accepts_injected_timestamp_for_deterministic_tests() {
        // [HAIKU-4.5] sq-tzars.2: direct unit test for the new public `run_with_time`
        // function. Verify that an injected timestamp is used instead of the wall-clock
        // instant, enabling deterministic test fixtures.
        let extractor = RecordedExtractor::from_fixture().unwrap();
        let fixed_time = "2026-07-05T14:30:00Z";
        let out = run_with_time(crate::literature::FIXTURE_OPENALEX_BATCH, &extractor, Some(fixed_time.to_string())).unwrap();

        // The injected timestamp must appear in the emitted Turtle.
        assert_eq!(out.generated_at_time, fixed_time);
        assert!(out.turtle.contains(fixed_time), "injected timestamp must appear in emitted TTL");
        // Both Finding and Activity nodes must carry the timestamp.
        let timestamp_count = out.turtle.matches(fixed_time).count();
        assert!(
            timestamp_count >= 2,
            "both Findings and Activities should carry the timestamp (got {} occurrences)",
            timestamp_count
        );
    }
}
