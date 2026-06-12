//! Test chunk 1 for fn:position
//! Contains 7 tests

use dep::xpath::stub_fnposition;

#[test]
fn k_contextpositionfunc_37() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[3 < position
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_38() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[position() g
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_39() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[position() >
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_40() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[4 le positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_41() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[4 <= positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_42() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[position() g
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_43() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[position() >
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}
