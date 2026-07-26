//! [SONNET-4.6] (sq-2ch27, Phase E6) The OWL 2 EL classifier surfaces end-to-end: the
//! `classify <data-file> <format> [out.nt]` subcommand and the `--reason el` profile on
//! `reason` / `query`.
//!
//! Spawns the *built* `sparq-cli` binary (via `CARGO_BIN_EXE_sparq-cli`, the same mechanism as
//! `dump_cli.rs` / `terse_cli.rs`). Both surfaces live behind the opt-in `reason-el` feature, so
//! this whole file compiles and runs only under it:
//! `cargo test -p sparq-cli --features reason-el`.
#![cfg(feature = "reason-el")]

use std::path::{Path, PathBuf};
use std::process::Command;

/// An RL-incompleteness witness, in Turtle: `A ⊑ ∃r.B ⊓ X`, `B ⊑ C`, `(∃r.C ⊓ X) ⊑ D` entails
/// `A ⊑ D`, which EL derives (CR3 links `A →ʳ B`, CR1 puts `C ∈ S(B)`, CR4 fires `∃r.C`, CR2
/// closes the conjunction) and OWL 2 RL cannot: RL has no rule concluding MEMBERSHIP in an
/// intersection class expression (`scm-int` only decomposes one), so the `∃r.C ⊓ X` premise
/// never fires for `A`. This is the whole reason the `el` surfaces exist, so it is the fixture
/// every test here classifies — and `owl_rl_misses_what_el_derives` below is the tripwire
/// keeping the claim true.
///
/// NOT the design record's §1.3 ontology (`A ⊑ ∃r.B`, `B ⊑ C`, `∃r.C ⊑ D`): under the RL/RDF
/// ruleset `scm-svf1` relates the two told restriction nodes (`∃r.B ⊑ ∃r.C`, same property,
/// `B ⊑ C`) and `scm-sco` chains it, so sparq's own RL DOES derive `A ⊑ D` there. Asserting
/// otherwise would have made this suite pass on a false premise.
const WITNESS: &str = r#"
@prefix :     <http://ex/> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ; owl:someValuesFrom :B ] .
:A rdfs:subClassOf :X .
:B rdfs:subClassOf :C .
[ a owl:Class ; owl:intersectionOf (
    [ a owl:Restriction ; owl:onProperty :r ; owl:someValuesFrom :C ]
    :X ) ] rdfs:subClassOf :D .
"#;

/// The derived edge RL misses and EL finds, in the N-Triples form the closure is written in.
const DERIVED: &str = "<http://ex/A> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/D> .";

/// A fresh scratch dir under the cargo per-test-target tmp dir.
fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("classify-cli-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write fixture");
    p
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// (exit code, stdout, stderr) from the built binary.
fn run3(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_sparq-cli")).args(args).output().expect("spawning sparq-cli");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// `classify` reports the classified TBox and, with `out.nt`, writes the materialized lattice
/// carrying the subsumption RL cannot reach.
#[test]
fn classify_writes_the_complete_lattice() {
    let dir = scratch("lattice");
    let ttl = write(&dir, "witness.ttl", WITNESS);
    let out = dir.join("closure.nt");

    let (code, stdout, stderr) = run3(&["classify", s(&ttl), "turtle", s(&out)]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("classified [EL]"), "stdout: {stdout}");
    // Exactly one edge is NEW: A ⊑ D. (A ⊑ X and B ⊑ C are asserted, so they are not re-emitted.)
    assert!(stdout.contains("+1 subsumption(s)"), "stdout: {stdout}");

    let closure = std::fs::read_to_string(&out).expect("closure written");
    assert!(closure.contains(DERIVED), "EL must derive A ⊑ D through the ∃r successor:\n{closure}");
    // The asserted axioms survive materialization (the lattice is appended, not replaced).
    assert!(closure.contains("<http://ex/B> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/C> ."), "{closure}");
}

/// The honest contrast that motivates the flag: the SAME fixture through `reason … owl` (OWL 2
/// RL) does NOT yield `A ⊑ D`. If RL ever did, the docs' "use `el` for a complete class
/// hierarchy" note would be overclaiming — so this test is the tripwire on that claim, not
/// decoration.
#[test]
fn owl_rl_misses_what_el_derives() {
    let dir = scratch("rl-gap");
    let ttl = write(&dir, "witness.ttl", WITNESS);
    let rl = dir.join("rl.nt");
    let el = dir.join("el.nt");

    let (code, _, stderr) = run3(&["reason", s(&ttl), "turtle", "owl", s(&rl)]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let (code, _, stderr) = run3(&["reason", s(&ttl), "turtle", "el", s(&el)]);
    assert_eq!(code, 0, "stderr: {stderr}");

    let rl = std::fs::read_to_string(&rl).expect("RL closure");
    let el = std::fs::read_to_string(&el).expect("EL closure");
    assert!(!rl.contains(DERIVED), "OWL 2 RL is not supposed to derive A ⊑ D:\n{rl}");
    assert!(el.contains(DERIVED), "EL must derive A ⊑ D:\n{el}");
}

/// `query --reason el` classifies before evaluating, so a plain BGP over the materialized
/// lattice returns the derived super-class — the end-to-end flag path.
#[test]
fn query_reason_el_sees_the_derived_subsumption() {
    let dir = scratch("query");
    let ttl = write(&dir, "witness.ttl", WITNESS);
    let q = "SELECT ?d WHERE { <http://ex/A> <http://www.w3.org/2000/01/rdf-schema#subClassOf> ?d }";

    let (code, stdout, stderr) = run3(&["query", s(&ttl), "turtle", q, "--reason", "el", "--format", "tsv"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("http://ex/D"), "stdout: {stdout}\nstderr: {stderr}");
    assert!(stderr.contains("classified [EL]"), "stderr: {stderr}");
}

/// An `owl:disjointWith` clash makes a class unsatisfiable, and `classify` NAMES it (the typed
/// view's readoff — `classify_graph` only counts them).
#[test]
fn classify_names_unsatisfiable_classes() {
    let dir = scratch("unsat");
    let ttl = write(
        &dir,
        "clash.ttl",
        r#"
@prefix :     <http://ex/> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:B owl:disjointWith :C .
:A rdfs:subClassOf :B .
:A rdfs:subClassOf :C .
"#,
    );
    let (code, stdout, stderr) = run3(&["classify", s(&ttl), "turtle"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("unsatisfiable: <http://ex/A>"), "stdout: {stdout}");
    assert!(stderr.contains("UNSATISFIABLE"), "stderr: {stderr}");
}

/// An axiom outside the recognised EL fragment is REPORTED, never silently applied.
#[test]
fn out_of_fragment_axioms_are_reported() {
    let dir = scratch("skips");
    let ttl = write(
        &dir,
        "union.ttl",
        r#"
@prefix :     <http://ex/> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:A rdfs:subClassOf [ a owl:Class ; owl:unionOf ( :B :C ) ] .
"#,
    );
    let (code, _, stderr) = run3(&["classify", s(&ttl), "turtle"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.contains("OUTSIDE the recognised EL fragment"), "stderr: {stderr}");
}

/// An axiom that is INSIDE the OWL 2 EL profile but outside what THIS binary recognises — a
/// faceted concrete-domain restriction (`owl:onDatatype` + `owl:withRestrictions`), legal EL,
/// needing `sparq-reason-el/cdomain` which the CLI's `reason-el` feature does NOT enable — is the
/// case that makes an unqualified "complete OWL 2 EL lattice" claim FALSE. So it must be a
/// counted, reported skip whose note names the gap, and the result must never be presented as
/// unconditionally complete. (The CLI's feature set is what fixes this: the `reason-el`
/// feature-matrix leg resolves sparq-cli's graph alone, so `cdomain` cannot unify on.)
#[test]
fn in_profile_concrete_domain_axiom_is_a_reported_skip() {
    let dir = scratch("cdomain-skip");
    let ttl = write(
        &dir,
        "faceted.ttl",
        r#"
@prefix :     <http://ex/> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

:Adult rdfs:subClassOf
  [ owl:onProperty :age ;
    owl:someValuesFrom
      [ owl:onDatatype xsd:integer ;
        owl:withRestrictions ( [ xsd:minInclusive 18 ] ) ] ] .
:G rdfs:subClassOf :H .
"#,
    );
    let (code, stdout, stderr) = run3(&["classify", s(&ttl), "turtle"]);
    assert_eq!(code, 0, "stderr: {stderr}");
    // The EL part still classifies — the skip is scoped to the concrete-domain axiom, not fatal.
    assert!(stdout.contains("classified [EL]"), "stdout: {stdout}");
    assert!(stderr.contains("OUTSIDE the recognised EL fragment"), "the skip must be reported: {stderr}");
    // …and the note must say WHY it is in-profile-but-skipped, else a reader concludes their
    // ontology was out of EL rather than that this build cannot decide concrete domains.
    assert!(
        stderr.contains("cdomain"),
        "the note must name the in-profile concrete-domain gap, not lump it with non-EL constructs: {stderr}"
    );
    // Nothing the command prints about THIS result may call it complete.
    assert!(
        !stdout.to_ascii_lowercase().contains("complete"),
        "a run with skipped in-profile axioms must not be described as complete: {stdout}"
    );
}

/// `classify` with no arguments is a loud usage error, not a silent no-op — and the usage text
/// SCOPES its completeness claim (the claim above is only true for the recognised fragment).
#[test]
fn classify_without_args_loud_fails() {
    let (code, _, stderr) = run3(&["classify"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("usage: sparq-cli classify"), "stderr: {stderr}");
    assert!(
        stderr.contains("RECOGNISED EL fragment") && stderr.contains("NOT for the whole OWL 2 EL"),
        "the usage text must scope its completeness claim: {stderr}"
    );
}

/// An unknown `--reason` profile still loud-fails, and its message now advertises `el`.
#[test]
fn unknown_reason_profile_lists_el() {
    let dir = scratch("unknown");
    let ttl = write(&dir, "witness.ttl", WITNESS);
    let (code, _, stderr) = run3(&["reason", s(&ttl), "turtle", "ql"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("rdfs | owl | n3 | el"), "stderr: {stderr}");
}
