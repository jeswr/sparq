//! [OPUS-4.8] sq-ncvq.16 — the CENTRAL conformance SCOREBOARD registry.
//!
//! Before this module the project's conformance ratchets were reported in
//! several disconnected places: the W3C SPARQL suites (this crate's
//! `sparq-conformance` binary), the inference suites (`sparq-inference-conformance`),
//! the W3C SHACL core + SHACL-SPARQL suites (crate-local `cargo test` runners in
//! `sparq-shacl`), and the OGC GeoSPARQL topology ratchet (a crate-local
//! `cargo test` in `sparq-geo`). The drift-scanner (`scripts/drift-scan.py`
//! §5.E `conformance-split`) flagged the SHACL + geo ratchets as living OUTSIDE
//! the central scoreboard, so no single artifact answered "what conformance does
//! sparq claim, and at what floor?".
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
//! The SHACL + geo runners are NOT re-implemented here. They depend on
//! `sparq-shacl` / `sparq-geo`, and this crate must stay free of those deps —
//! exactly the constraint `crates/sparq-shacl/tests/w3c_core.rs` records in its
//! own header ("Manifest-walking helpers are modelled on `sparq-conformance`'s
//! (copied, not shared — that crate is dev-only and must not become a
//! dependency)"). Pulling sparq-shacl/sparq-geo into the conformance crate would
//! invert that and couple the SPARQL/inference scoreboard to the SHACL/geo build.
//! So consolidation happens at the REPORTING layer: the floors live here as the
//! authoritative list, the runners stay where their dependencies are, and a guard
//! test ([`tests/scoreboard_floors.rs`]) hermetically reads the crate-local
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
/// * OGC GeoSPARQL 119 — `sparq-geo` `ogc_compliance_ratchet.rs`
///   `OGC_RATCHET_FLOOR = 119`.
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
        ratchet_floor: 119,
        floor_basis: "pass",
        note: "hand-curated sf/eh/rcc8 topology + WKT/GML equivalence assertions",
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
         Each suite has a pass-count (or pass+divergence) FLOOR that CI enforces and \
         that may only RISE. The per-suite detail reports are produced by the runners \
         in the *run* column; this table is the consolidated map (sq-ncvq.16 — \
         previously the SHACL + GeoSPARQL ratchets lived outside this scoreboard).\n"
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
    let total: usize = SUITES.iter().map(|s| s.ratchet_floor).sum();
    let _ = writeln!(
        md,
        "| **total ({} suites)** | | **{}** | | | |\n",
        SUITES.len(),
        total
    );
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
