//! [OPUS-4.8] sq-55c1 — DIFFERENTIAL SHACL fuzzing against reference engines.
//!
//! Generates random but VALID SHACL shapes graphs + data graphs, validates each
//! through `sparq-shacl` AND through one-or-more reference SHACL engines, and
//! asserts the reports agree. A disagreement is a candidate sparq-shacl bug; the
//! harness prints the seed and the (small) reproducing Turtle so it replays
//! deterministically.
//!
//! ## Why a seeded RNG loop (not proptest / cargo-fuzz)
//! This mirrors the established differential idiom in `sparq-bench/src/fuzz.rs`
//! (the engine-vs-Oxigraph SPARQL differential): a deterministic SplitMix64 over
//! a `seed_start..seed_start+count` range, one independent case per seed, so any
//! failure reproduces from its printed seed alone. `cargo-fuzz` in `fuzz/` is for
//! a different invariant (hostile bytes must not panic the parser); semantic
//! cross-engine differentials want reproducible *valid* inputs, which the seeded
//! loop gives directly.
//!
//! ## Reference engines (pluggable — sq-eifd report-cli adapters)
//! The reference side is a "report-cli" adapter (bead sq-eifd): a subprocess that
//! reads `{data, shapes}` Turtle and emits a normalised JSON report
//! (`conforms` + a `{focus, component, path}` violation list). The first wired
//! reference is **pySHACL** (`tests/diff_fuzz/pyshacl_adapter.py`), the canonical
//! SHACL reference implementation. Other engines (rdf-validate-shacl /
//! shacl-engine via Node, Jena-SHACL via a jar) are left as TODO beads; each is a
//! drop-in alternative `RefEngine` that produces the same JSON shape.
//!
//! ## How to run
//! Off by default (`#[ignore]`) so the per-PR `cargo nextest run` fast path is
//! untouched — this is a NIGHTLY/heavy-tier check. Run it explicitly:
//! ```text
//! # point at a Python with pyshacl + rdflib installed:
//! SHACL_DIFF_PYTHON=/path/to/venv/bin/python \
//!   cargo test -p sparq-shacl --test diff_fuzz -- --ignored --nocapture
//! # bound the run: SHACL_DIFF_SEED_START / SHACL_DIFF_COUNT (defaults 0 / 200).
//! ```
//! When no reference engine resolves, the test SKIPS (prints why and returns) so
//! it stays green on a box without the reference toolchain.
//!
//! ## Comparison policy (per bead sq-55c1)
//! 1. `conforms` bit — exact (cheap, highest signal).
//! 2. The violation SET, compared as a deduplicated set of
//!    `(focusNode, sourceConstraintComponent, path-presence)` tuples, with
//!    blank-node and complex-path tolerance. We deliberately compare the SET, not
//!    the multiset count: sparq's report intentionally does not deduplicate the
//!    way pySHACL does (the bead's "no-dedup caveat"), and `sh:resultMessage`s are
//!    impl-specific. A focus node + violated component that one engine reports and
//!    the other does not is the real signal.

use std::io::Write;
use std::process::{Command, Stdio};

use sparq_core::Graph;
use sparq_shacl::validate;

mod gen;
use gen::{Rng, Scenario};

// ---------------------------------------------------------------------------
// Reference engine: a report-cli adapter (sq-eifd "report-cli" kind).
// ---------------------------------------------------------------------------

/// A normalised reference-engine report, comparable to sparq's `ValidationReport`.
#[derive(Debug, serde::Deserialize)]
struct RefReport {
    conforms: bool,
    violations: Vec<RefViolation>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct RefViolation {
    focus: Option<String>,
    component: Option<String>,
    path: Option<String>,
}

/// Resolves the Python interpreter to drive the pySHACL adapter, or `None` if
/// pySHACL is not importable (so the test can SKIP cleanly). Honours
/// `SHACL_DIFF_PYTHON` (a venv's `python`); otherwise tries `python3`.
fn resolve_pyshacl() -> Option<String> {
    let candidates = std::env::var("SHACL_DIFF_PYTHON")
        .ok()
        .into_iter()
        .chain(["python3".to_string(), "python".to_string()]);
    for py in candidates {
        let ok = Command::new(&py)
            .args(["-c", "import pyshacl, rdflib"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(py);
        }
    }
    None
}

fn adapter_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/diff_fuzz/pyshacl_adapter.py")
}

/// Runs the reference engine over one (data, shapes) pair. `Err` means the
/// adapter itself failed (engine error / bad invocation) — distinct from a
/// produced non-conforming report.
fn run_reference(python: &str, data: &str, shapes: &str) -> Result<RefReport, String> {
    let req = serde_json::json!({ "data": data, "shapes": shapes }).to_string();
    let mut child = Command::new(python)
        .arg(adapter_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn adapter: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(req.as_bytes())
        .map_err(|e| format!("write request: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("wait adapter: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "adapter exit {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "parse adapter JSON: {e}; stdout={}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

// ---------------------------------------------------------------------------
// Normalisation: map both engines' reports to a comparable violation set.
// ---------------------------------------------------------------------------

/// A graph-independent violation key: (focus, component, has-simple-path).
///
/// - focus: full IRI, or `_:bnode` for any blank node (no cross-graph identity).
/// - component: the `sh:sourceConstraintComponent` IRI.
/// - path: `None` (node-level result), `Some(<predicate IRI>)` for a simple
///   predicate path (where both engines agree on the bare IRI), or
///   `Some("_:path")` for any complex / blank-rooted path (we don't byte-match
///   the impl-specific Turtle serialisation of complex paths — presence + the
///   (focus, component) pair carry the signal).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    focus: String,
    component: String,
    path: Option<String>,
}

fn norm_term(t: &oxrdf::Term) -> String {
    match t {
        oxrdf::Term::BlankNode(_) => "_:bnode".to_string(),
        oxrdf::Term::NamedNode(n) => n.as_str().to_string(),
        other => other.to_string(),
    }
}

/// sparq's path → the same comparable string the adapter emits: a bare predicate
/// IRI for a simple path, else the `_:path` complex-path sentinel.
fn norm_path(p: &sparq_shacl::Path) -> Option<String> {
    match p {
        sparq_shacl::Path::Predicate(iri) => Some(iri.clone()),
        _ => Some("_:path".to_string()),
    }
}

fn sparq_keys(report: &sparq_shacl::ValidationReport) -> std::collections::BTreeSet<Key> {
    report
        .results
        .iter()
        .map(|r| Key {
            focus: norm_term(&r.focus_node),
            component: r.source_component.clone(),
            path: r.path.as_ref().and_then(norm_path),
        })
        .collect()
}

fn ref_keys(report: &RefReport) -> std::collections::BTreeSet<Key> {
    report
        .violations
        .iter()
        .map(|v| Key {
            // [OPUS-4.8] A genuinely-absent `focus` in the reference JSON is a
            // distinct fact from a blank-node focus: collapsing both to "_:bnode"
            // would let a report-shape bug (adapter dropping the focus field)
            // silently compare equal to sparq emitting a blank-node focus, masking
            // the disagreement. sparq's `norm_term` never yields this sentinel
            // (a real blank node maps to "_:bnode", a named node to its IRI), so a
            // missing-focus reference result can only ever match another
            // missing-focus result — and will mismatch any sparq result, surfacing
            // the adapter/engine shape bug instead of hiding it.
            focus: v
                .focus
                .clone()
                .unwrap_or_else(|| "<missing-focus>".to_string()),
            component: v
                .component
                .clone()
                .unwrap_or_else(|| "<no-component>".to_string()),
            path: v.path.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// One differential case.
// ---------------------------------------------------------------------------

enum CaseOutcome {
    Agree,
    /// Engine could not run this case (e.g. the reference choked on a construct);
    /// not a sparq bug — counted separately.
    Skipped(String),
    /// A genuine disagreement: the formatted diagnostic (seed + reproducer).
    Mismatch(String),
}

fn run_case(python: &str, seed: u64) -> CaseOutcome {
    let mut rng = Rng::new(seed);
    let scenario = Scenario::generate(&mut rng);
    let data_ttl = scenario.data_turtle();
    let shapes_ttl = scenario.shapes_turtle();

    // sparq must at least parse its own generated graphs — a parse failure here is
    // a generator bug, surfaced as a mismatch so it cannot pass silently.
    let data = match Graph::load_str(&data_ttl, "turtle") {
        Ok(g) => g,
        Err(e) => {
            return mismatch(
                seed,
                &data_ttl,
                &shapes_ttl,
                &format!("sparq data parse: {e}"),
            )
        }
    };
    let shapes = match Graph::load_str(&shapes_ttl, "turtle") {
        Ok(g) => g,
        Err(e) => {
            return mismatch(
                seed,
                &data_ttl,
                &shapes_ttl,
                &format!("sparq shapes parse: {e}"),
            )
        }
    };
    let sparq_report = validate(&data, &shapes);

    let reference = match run_reference(python, &data_ttl, &shapes_ttl) {
        Ok(r) => r,
        Err(e) => return CaseOutcome::Skipped(format!("reference engine error: {e}")),
    };

    // 1. conforms bit — exact.
    if sparq_report.conforms != reference.conforms {
        return mismatch(
            seed,
            &data_ttl,
            &shapes_ttl,
            &format!(
                "CONFORMS differs: sparq={} reference={} (sparq {} results, reference {} violations)",
                sparq_report.conforms,
                reference.conforms,
                sparq_report.results.len(),
                reference.violations.len(),
            ),
        );
    }

    // 2. violation set (deduplicated; blank-node + complex-path tolerant).
    let s = sparq_keys(&sparq_report);
    let r = ref_keys(&reference);
    if s != r {
        let only_sparq: Vec<_> = s.difference(&r).collect();
        let only_ref: Vec<_> = r.difference(&s).collect();
        return mismatch(
            seed,
            &data_ttl,
            &shapes_ttl,
            &format!(
                "VIOLATION SET differs (conforms agree = {}):\n  only in sparq ({}): {:?}\n  only in reference ({}): {:?}",
                sparq_report.conforms,
                only_sparq.len(),
                only_sparq,
                only_ref.len(),
                only_ref,
            ),
        );
    }

    CaseOutcome::Agree
}

fn mismatch(seed: u64, data: &str, shapes: &str, msg: &str) -> CaseOutcome {
    CaseOutcome::Mismatch(format!(
        "MISMATCH seed={seed}\n{msg}\n--- shapes.ttl ---\n{shapes}\n--- data.ttl ---\n{data}"
    ))
}

// ---------------------------------------------------------------------------
// The driver.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "differential SHACL fuzz — nightly/heavy tier; needs a reference engine (pyshacl)"]
fn differential_shacl_fuzz() {
    let python = match resolve_pyshacl() {
        Some(p) => p,
        None => {
            eprintln!(
                "SKIP: no reference SHACL engine — install pyshacl+rdflib and set \
                 SHACL_DIFF_PYTHON to its interpreter (e.g. a venv's bin/python)."
            );
            return;
        }
    };
    eprintln!("differential SHACL fuzz: reference = pySHACL via {python}");

    let seed_start: u64 = env_u64("SHACL_DIFF_SEED_START", 0);
    let count: u64 = env_u64("SHACL_DIFF_COUNT", 200);

    let mut agree = 0u64;
    let mut skipped = 0u64;
    let mut mismatches: Vec<String> = Vec::new();
    let mut skip_examples: Vec<String> = Vec::new();

    for seed in seed_start..seed_start + count {
        match run_case(&python, seed) {
            CaseOutcome::Agree => agree += 1,
            CaseOutcome::Skipped(why) => {
                skipped += 1;
                if skip_examples.len() < 5 {
                    skip_examples.push(format!("seed={seed}: {why}"));
                }
            }
            CaseOutcome::Mismatch(detail) => {
                eprintln!("MISMATCH seed={seed}");
                // Cap stored reproducers so a systematically-wrong run does not
                // produce megabytes of output; the count is still exact.
                if mismatches.len() < 10 {
                    mismatches.push(detail);
                }
            }
        }
    }

    let total_mismatch = count - agree - skipped;
    eprintln!(
        "\nSHACL diff fuzz: seeds {seed_start}..{} — agree={agree} skipped={skipped} mismatch={total_mismatch}",
        seed_start + count,
    );
    for ex in &skip_examples {
        eprintln!("SKIP example: {ex}");
    }

    if total_mismatch > 0 {
        for d in &mismatches {
            eprintln!("\n{d}");
        }
        panic!(
            "{total_mismatch} disagreement(s) between sparq-shacl and the reference engine \
             (showing up to {} reproducers above) — file a bead per distinct bug with the seed.",
            mismatches.len()
        );
    }

    // [OPUS-4.8] Guard against a VACUOUS pass. We only reach this point with the
    // adapter RESOLVED (an absent pySHACL short-circuits via `resolve_pyshacl`
    // returning `None` above), so the comparison is expected to actually run.
    // `assess_run` makes "adapter present but compared zero cases" a hard error
    // instead of a silent agreement; see its doc + the unit test below.
    if let Err(why) = assess_run(agree, skipped) {
        for ex in &skip_examples {
            eprintln!("SKIP example: {ex}");
        }
        panic!("{why}");
    }
}

/// Decide whether a *resolved-adapter* run produced a meaningful comparison.
///
/// [OPUS-4.8] `agree` is the number of cases that ran the full sparq-vs-reference
/// comparison and matched; by the time the driver calls this, any mismatch has
/// already panicked, so `agree == compared` (the count of cases that actually
/// compared). If every case was instead `Skipped` — the adapter resolved yet
/// errored on every seed (wrong venv, missing rdflib, adapter regression) —
/// `compared` is 0 and the lane would otherwise "pass" while comparing nothing.
/// That is a broken reference setup, not agreement, so we return `Err`.
///
/// This is intentionally separate from the "adapter intentionally absent" path,
/// which the driver handles earlier by returning before any case runs.
fn assess_run(agree: u64, skipped: u64) -> Result<(), String> {
    let compared = agree;
    if compared == 0 {
        return Err(format!(
            "SHACL diff fuzz compared ZERO cases (skipped={skipped}): the reference adapter \
             was resolved yet errored on every seed — a broken reference setup/adapter, not \
             agreement. Check pySHACL/rdflib in the configured interpreter."
        ));
    }
    Ok(())
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Fast self-tests of the GENERATOR (no reference engine; run in the per-PR
// fast path). These guard the differential from going vacuous: a generator that
// silently degenerated to always-conforming, or that produced shapes sparq
// can't parse, would make the ignored differential pass for the wrong reason.
// ---------------------------------------------------------------------------

/// Every generated scenario must parse (shapes + data) and validate without
/// panicking, and across a batch the generator must produce BOTH conforming and
/// violating reports (so the differential exercises real disagreement surface).
#[test]
fn generator_is_well_formed_and_mixes_outcomes() {
    let mut conforming = 0u32;
    let mut violating = 0u32;
    let mut total_violations = 0usize;
    for seed in 0..400u64 {
        let mut rng = Rng::new(seed);
        let scenario = Scenario::generate(&mut rng);
        let data_ttl = scenario.data_turtle();
        let shapes_ttl = scenario.shapes_turtle();
        let data = Graph::load_str(&data_ttl, "turtle")
            .unwrap_or_else(|e| panic!("seed {seed}: data parse: {e}\n{data_ttl}"));
        let shapes = Graph::load_str(&shapes_ttl, "turtle")
            .unwrap_or_else(|e| panic!("seed {seed}: shapes parse: {e}\n{shapes_ttl}"));
        let report = validate(&data, &shapes);
        if report.conforms {
            conforming += 1;
        } else {
            violating += 1;
            total_violations += report.results.len();
        }
    }
    // The generator is healthy iff it produces a real mix — neither all-conforming
    // (the differential would never test violation reporting) nor all-violating
    // (it would never test that BOTH engines agree a graph conforms). Each instance
    // has several properties each violated ~half the time, so violations dominate by
    // construction (a scenario conforms only if EVERY value satisfies EVERY shape);
    // the floor is set to "both outcomes meaningfully represented", not 50/50.
    assert!(
        conforming >= 10 && violating >= 10,
        "generator is lopsided: {conforming} conforming, {violating} violating over 400 seeds \
         — the differential needs both outcomes represented"
    );
    assert!(total_violations > 0, "no violations generated at all");
}

/// The two normalisation directions must agree on a hand-built case so a refactor
/// of either `sparq_keys` or `ref_keys` can't silently drift the comparison.
#[test]
fn key_normalisation_is_consistent() {
    use sparq_shacl::Path;
    let report = sparq_shacl::ValidationReport {
        conforms: false,
        results: vec![sparq_shacl::ValidationResult {
            focus_node: oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
                "http://example.org/x",
            )),
            path: Some(Path::Predicate("http://example.org/p".into())),
            value: None,
            source_shape: oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(
                "http://example.org/S",
            )),
            source_component: "http://www.w3.org/ns/shacl#DatatypeConstraintComponent".into(),
            severity: "http://www.w3.org/ns/shacl#Violation".into(),
            messages: vec![],
            default_message: String::new(),
        }],
    };
    let sk = sparq_keys(&report);
    let rr = RefReport {
        conforms: false,
        violations: vec![RefViolation {
            focus: Some("http://example.org/x".into()),
            component: Some("http://www.w3.org/ns/shacl#DatatypeConstraintComponent".into()),
            path: Some("http://example.org/p".into()),
        }],
    };
    assert_eq!(
        sk,
        ref_keys(&rr),
        "normalisation must agree on the same logical result"
    );
}

/// [OPUS-4.8] Fast guard for the line-354 fix: a resolved adapter that errored on
/// every seed (all `Skipped`, zero compared) must be a FAILURE, not a pass; a run
/// that compared at least one case is OK. Runs in the per-PR fast lane (no
/// reference engine needed), so the floor logic can't silently regress.
#[test]
fn all_skipped_run_is_a_failure_not_a_vacuous_pass() {
    // Adapter resolved but errored on every one of 200 seeds.
    assert!(
        assess_run(0, 200).is_err(),
        "an all-skipped run (compared==0) must fail — otherwise a broken reference \
         adapter masquerades as agreement"
    );
    // At least one real comparison ran → not vacuous.
    assert!(assess_run(1, 199).is_ok());
    assert!(assess_run(200, 0).is_ok());
}

/// [OPUS-4.8] Fast guard for the line-199 fix: a reference violation with a
/// genuinely-absent `focus` must NOT normalise to the same key as a blank-node
/// focus, so a dropped-focus report-shape bug can't silently compare equal.
#[test]
fn missing_focus_is_distinct_from_blank_node_focus() {
    let comp = "http://www.w3.org/ns/shacl#MinCountConstraintComponent".to_string();
    let missing = ref_keys(&RefReport {
        conforms: false,
        violations: vec![RefViolation {
            focus: None,
            component: Some(comp.clone()),
            path: None,
        }],
    });
    // sparq never yields a missing focus; the closest is a blank-node focus,
    // which `norm_term` maps to "_:bnode". The two keys must differ.
    let blank = missing
        .iter()
        .next()
        .map(|k| Key {
            focus: "_:bnode".to_string(),
            component: k.component.clone(),
            path: k.path.clone(),
        })
        .unwrap();
    assert!(
        !missing.contains(&blank),
        "missing focus collapsed into the blank-node sentinel — masks report-shape bugs"
    );
}
