// Fixture-only Rust source for the paper-evidence verifier self-test (rust-anchor tier).
// NOT compiled — the verifier only reads it as text (anchor existence + literal adjacency).

/// A conformance ratchet floor the paper cites: the anchor const name is stable, the literal
/// sits within the anchor window.
const DEMO_SHACL_FLOOR: usize = 98;

#[test]
fn demo_recall_floor_holds() {
    // The value literal 0.90 appears within the anchor test-fn window.
    let recall_floor = 0.90;
    assert!(recall_floor >= 0.90, "recall floor regressed");
}
