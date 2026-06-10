//! Data-gated smoke test: only runs when the W3C suites have been fetched
//! (scripts/fetch-conformance.sh), so `cargo test --workspace` stays green on
//! a clean checkout / CI without test data.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn runner_produces_a_report_when_data_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/w3c/rdf-tests");
    if !root.join("sparql").is_dir() {
        eprintln!("skipping: W3C test data absent (run scripts/fetch-conformance.sh)");
        return;
    }
    let report = std::env::temp_dir().join("sparq-conformance-smoke.md");
    let status = Command::new(env!("CARGO_BIN_EXE_sparq-conformance"))
        .arg("--root")
        .arg(&root)
        .arg("--filter")
        .arg("sparql10/basic")
        .arg("--report")
        .arg(&report)
        .status()
        .expect("run sparq-conformance");
    assert!(status.success(), "runner must exit 0 without --strict");
    let text = std::fs::read_to_string(&report).expect("report written");
    assert!(
        text.contains("sparql10/basic"),
        "report covers the filtered suite"
    );
}
