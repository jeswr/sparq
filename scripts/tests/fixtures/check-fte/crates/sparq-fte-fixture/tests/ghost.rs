// [SONNET-4.6] sq-qcnn.31: Negative fixture for check-feature-test-execution.py --self-test.
//
// This file is gated on `ghost-feature` — a default-OFF feature with NO CI executor and
// NO allowlist entry. The structural guard (C1) MUST detect this as UNMAPPED. If the guard
// reports no violation on this fixture, --self-test exits 1 (the guard is broken).
//
// This file is intentionally NOT compiled into any real crate; it lives under
// scripts/tests/fixtures/ and is only scanned by the Python detector's self-test path.

#![cfg(feature = "ghost-feature")]

#[test]
fn ghost_test_this_must_be_flagged_as_unmapped() {
    // This test body is intentionally trivial — the fixture exists only to verify the
    // structural guard detects the missing CI executor, not to test any real logic.
    assert!(true);
}
