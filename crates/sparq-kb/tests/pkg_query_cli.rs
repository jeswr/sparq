//! End-to-end CLI contract tests for the `pkg-query --citations` mode (sq-2489d.11).
//!
//! Spawns the *built* `pkg-query` binary (via `CARGO_BIN_EXE_pkg-query`, the same
//! mechanism as `sparq-cli/tests/cli_contract.rs`) so the load-bearing CLI wiring
//! itself is exercised — not just the renderer the binary calls. Pins the mode
//! contract:
//!
//! 1. `--citations` succeeds, selects the `FINDING_PROVENANCE` canned query
//!    (executed SPARQL surfaced on stderr), prints rendered `[source]` citations
//!    on stdout and the measured metric summary on stderr.
//! 2. `--citations --json`, `--citations --query/--sparql …` and
//!    `--citations --arg …` are rejected with a non-zero exit, per the
//!    documented incompatibilities.
//! 3. `--citations --sparql-only` prints the executed SPARQL without running it.
//!
//! [FABLE-5] 🤖 SPARQ agent — pins the CLI mode contract for the citation renderer.
//!
//! Run: `cargo test -p sparq-kb --features query --test pkg_query_cli`
#![cfg(feature = "query")]

use std::process::{Command, Output};

/// Run the built `pkg-query` binary with `args`, capturing stdout/stderr.
fn pkg_query(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pkg-query"))
        .args(args)
        .output()
        .expect("pkg-query binary runs")
}

/// `--citations` end-to-end: success, FINDING_PROVENANCE selected, citations on
/// stdout, measured metrics on stderr.
#[test]
fn citations_mode_renders_citations_and_metrics_end_to_end() {
    let out = pkg_query(&["--citations"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "--citations must exit 0; stderr: {stderr}"
    );

    // The wiring must select the FINDING_PROVENANCE canned query — its
    // signature join is surfaced by the executed-SPARQL banner on stderr.
    assert!(
        stderr.contains("--- executed SPARQL ---"),
        "executed SPARQL banner missing from stderr: {stderr}"
    );
    assert!(
        stderr.contains("prov:wasDerivedFrom"),
        "executed SPARQL must be the FINDING_PROVENANCE query: {stderr}"
    );

    // Rendered citation text on stdout (the real PKG has the AGENTS.md Findings
    // tier, so at least one [source: …] citation must render).
    assert!(
        stdout.contains("[source:"),
        "rendered [source:] citations missing from stdout: {stdout}"
    );

    // Measured metric summary on stderr, with the load-bearing invariant the
    // renderer tests already pin in-process: 0 fabricated, resolution rate 1.0.
    assert!(
        stderr.contains("citation(s); 0 fabricated; resolution rate 1.00."),
        "metric summary (0 fabricated / rate 1.00) missing from stderr: {stderr}"
    );
}

/// `--citations --json` is documented as incompatible (the NL-tool envelope
/// carries no citation fields) and must be rejected.
#[test]
fn citations_rejects_json_mode() {
    let out = pkg_query(&["--citations", "--json"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "--citations --json must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("no --json form"),
        "rejection must explain the --json incompatibility: {stderr}"
    );
}

/// `--citations` is its own mode: combining it with `--query` or `--sparql`
/// must be rejected.
#[test]
fn citations_rejects_query_and_sparql_modes() {
    for extra in [
        &["--query", "finding-provenance"][..],
        &["--sparql", "SELECT * WHERE { ?s ?p ?o }"][..],
    ] {
        let mut args = vec!["--citations"];
        args.extend_from_slice(extra);
        let out = pkg_query(&args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !out.status.success(),
            "--citations {} must exit non-zero; stderr: {stderr}",
            extra[0]
        );
        assert!(
            stderr.contains("do not combine it with --query/--sparql"),
            "rejection must explain the mode conflict for {}: {stderr}",
            extra[0]
        );
    }
}

/// `--citations --arg <value>` must be rejected: the finding-provenance query is
/// unparameterised, so a passed argument would otherwise be silently discarded.
#[test]
fn citations_rejects_arg() {
    let out = pkg_query(&["--citations", "--arg", "sq-8thu"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "--citations --arg must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("takes no --arg"),
        "rejection must explain that --citations is unparameterised: {stderr}"
    );
}

/// `--citations --sparql-only` surfaces the executed SPARQL without running it:
/// no citations rendered, no metric summary.
#[test]
fn citations_sparql_only_prints_query_without_running_it() {
    let out = pkg_query(&["--citations", "--sparql-only"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        out.status.success(),
        "--citations --sparql-only must exit 0; stderr: {stderr}"
    );
    assert!(
        stderr.contains("prov:wasDerivedFrom"),
        "executed SPARQL must still be surfaced: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "--sparql-only must not render citations on stdout: {stdout}"
    );
    assert!(
        !stderr.contains("resolution rate"),
        "--sparql-only must not run the query or emit metrics: {stderr}"
    );
}
