//! Test chunk 0 for op:bang
//! Contains 5 tests

use dep::xpath::stub_opbang;

#[test]
fn bang_7() {
    // Precedence of '!' is less than '[]'
    // XPath: / works ! employee[2] ! hours[2] ! number()
    // Expected: 20
    // Calls stub function - will fail until op:bang is implemented
    let _ = stub_opbang();
}

#[test]
fn bang_8() {
    // Precedence of '!' is greater than '+'
    // XPath: 2 + (/ works ! employee[2] ! hours[2]) ! number() ! (-.)
    // Expected: -18
    // Calls stub function - will fail until op:bang is implemented
    let _ = stub_opbang();
}

#[test]
fn bang_9() {
    // Precedence of '!' is greater than unary '-'
    // XPath: -2!(.+1)
    // Expected: -3
    // Calls stub function - will fail until op:bang is implemented
    let _ = stub_opbang();
}

#[test]
fn bang_13() {
    // backwards-axes semantics of [position()] work with '!' operator
    // XPath: / works ! employee[4] ! preceding-sibling::*[1] ! string(@na
    // Expected: "Jane Doe 3"
    // Calls stub function - will fail until op:bang is implemented
    let _ = stub_opbang();
}

#[test]
fn bang_14() {
    // Interaction of '!' with '/'
    // XPath: count(/ ! works)
    // Expected: 1
    // Calls stub function - will fail until op:bang is implemented
    let _ = stub_opbang();
}
