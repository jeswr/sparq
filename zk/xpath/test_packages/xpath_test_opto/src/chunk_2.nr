//! Test chunk 2 for op:to
//! Contains 23 tests

use dep::xpath::stub_opto;

#[test]
fn rangeexpr_410b() {
    // multiple range, large negative numbers
    // XPath: (-100000000003 to -100000000000)[2]
    // Expected: -100000000002
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_410c() {
    // multiple range, large negative numbers
    // XPath: -100000000003 = -100000000003 to -100000000000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_410d() {
    // multiple range, large negative numbers
    // XPath: -100000000008 < -100000000006 to -100000000003
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_411a() {
    // multiple range, very large negative numbers
    // XPath: count(-1000000000000000000003 to -1000000000000000000000)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_411b() {
    // multiple range, very large negative numbers
    // XPath: (-1000000000000000000003 to -1000000000000000000000)[4]
    // Expected: -1000000000000000000000
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_411c() {
    // multiple range, very large negative numbers
    // XPath: -1000000000000000000000 = -1000000000000000000003 to -100000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_411d() {
    // multiple range, very large negative numbers
    // XPath: -1000000000000000000000 < -1000000000000000500003 to -100000
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_412a() {
    // multiple range, spanning zero
    // XPath: count(-5 to +5)
    // Expected: 11
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_412b() {
    // multiple range, spanning zero
    // XPath: (-5 to +5)[6]
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_412c() {
    // multiple range, spanning zero
    // XPath: 3 = -5 to +5
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_412d() {
    // multiple range, spanning zero
    // XPath: -5 to +5 > 29
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_413a() {
    // multiple range, not known statically
    // XPath: count(0 to year-from-date(current-date()) idiv 1000)
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_413b() {
    // multiple range, not known statically
    // XPath: (0 to year-from-date(current-date()) idiv 1000)[2]
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_413c() {
    // multiple range, not known statically
    // XPath: 2 = 0 to year-from-date(current-date()) idiv 1000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_413d() {
    // multiple range, not known statically
    // XPath: 3 >= 0 to year-from-date(current-date()) idiv 1000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_414a() {
    // multiple range, not known statically
    // XPath: count(year-from-date(current-date()) idiv 1000 to 5)
    // Expected: 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_414b() {
    // multiple range, not known statically
    // XPath: (year-from-date(current-date()) idiv 1000 to 5)[2]
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_414c() {
    // multiple range, not known statically
    // XPath: year-from-date(current-date()) idiv 1000 to 5 = 4
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_414d() {
    // multiple range, not known statically
    // XPath: year-from-date(current-date()) idiv 1000 to 5 > 6
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_415() {
    // singleton range, not known statically
    // XPath: year-from-date(current-date()) idiv 1000 to 2
    // Expected: 2
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_415a() {
    // singleton range, not known statically
    // XPath: count(year-from-date(current-date()) idiv 1000 to 2)
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_415b() {
    // singleton range, not known statically
    // XPath: (year-from-date(current-date()) idiv 1000 to 2)[1]
    // Expected: 2
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_416a() {
    // empty range, not known statically
    // XPath: count(year-from-date(current-date()) idiv 1000 to -42)
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}
