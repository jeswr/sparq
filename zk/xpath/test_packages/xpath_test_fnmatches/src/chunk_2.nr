//! Test chunk 2 for fn:matches
//! Contains 13 tests

use dep::xpath::stub_fnmatches;

#[test]
fn cbcl_matches_042() {
    //  test the multi-character escape \i
    // XPath: fn:matches('a_:', '\i+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_043() {
    //  test the multi-character escape \i
    // XPath: fn:matches('1.0', '\i+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_044() {
    //  test the multi-character escape \I
    // XPath: fn:matches('1.0', '\I+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_045() {
    //  test the multi-character escape \I
    // XPath: fn:matches('a_:', '\I+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_046() {
    //  test the multi-character escape \c
    // XPath: fn:matches('abc', '\c+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_047() {
    //  test the multi-character escape \c
    // XPath: fn:matches('&#x20;&#x9;&#xD;', '\c+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_047b() {
    //  test the multi-character escape \c
    // XPath: fn:matches('   ', '\c+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_048() {
    //  test the multi-character escape \C
    // XPath: fn:matches('&#x20;&#x9;&#xD;', '\C+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_049() {
    //  test the multi-character escape \C
    // XPath: fn:matches('abc', '\C+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_050() {
    //  A back-reference is compared using case-blind comparison: that is, each charact
    // XPath: fn:matches('Mum', '([md])[aeiou]\1', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_051() {
    //  Test back-reference to character above &#FFFF;
    // XPath: fn:matches('&#x10000;&#x10000;', '(&#x10000;)\1')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_052() {
    //  Test back-reference to character above &#FFFF;
    // XPath: fn:matches('&#x10000;&#x10001;', '(&#x10000;)\1')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_053() {
    //  A back-reference is compared using case-blind comparison: that is, each charact
    // XPath: fn:matches('Mud', '([md])[aeiou]\1', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}
