//! **The load-bearing deliverable of bead `sq-tag1q.18`**: a hermetic proof that
//! the opaque broker does NOT link the query engine.
//!
//! Design §7 is normative about it — *"A separate opt-in broker binary/crate
//! stores opaque blocks and topics; it MUST not link the query engine."* — and
//! §5 depends on it: a broker that could link `sparq-engine` could, in principle,
//! be handed a decrypted dataset and asked to evaluate SPARQL, which is exactly
//! the property the profile says a broker never has.
//!
//! The test resolves the crate's real dependency graph with `cargo tree` and
//! asserts that no forbidden workspace crate appears in it. `--all-features`
//! resolves every conditional edge, so an edge introduced under ANY feature is
//! visible here; `-e normal` ignores dev-dependencies (a test-only edge would not
//! be linked into the shipped broker, and this crate has none anyway).
//!
//! It fails LOUD if `cargo tree` cannot be run at all, so it can never pass
//! vacuously.

use std::process::Command;

/// Crates the broker must never (transitively) depend on.
const FORBIDDEN: &[&str] = &[
    "sparq-engine",
    "sparq-core",
    "sparq-substrate",
    "sparq-parse",
    "sparq-reason",
];

#[test]
fn broker_does_not_link_the_query_engine() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(&cargo)
        .args([
            "tree",
            "-p",
            "sparq-e2ee-ng-broker",
            "--all-features",
            "-e",
            "normal",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("`cargo tree` must be runnable for this boundary test to mean anything");
    assert!(
        out.status.success(),
        "`cargo tree` failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);
    assert!(
        tree.contains("sparq-e2ee-ng-broker"),
        "`cargo tree` produced no tree for the broker; the test would be vacuous:\n{}",
        tree
    );
    // Sanity: the one workspace edge the broker IS allowed must be present, so a
    // silently-empty tree cannot make the assertions below pass.
    assert!(
        tree.contains("sparq-e2ee-ng v"),
        "expected the broker to depend on sparq-e2ee-ng:\n{}",
        tree
    );
    for crate_name in FORBIDDEN {
        assert!(
            !tree
                .lines()
                .any(|l| l.split_whitespace().any(|tok| tok == *crate_name)),
            "BOUNDARY VIOLATION: the opaque broker gained a dependency on `{}` \
             (design §7: the broker MUST NOT link the query engine)",
            crate_name
        );
    }
}
