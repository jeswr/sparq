//! Test chunk 1 for fn:round-half-to-even
//! Contains 16 tests

use dep::xpath::stub_fnround_half_to_even;

#[test]
fn k2_roundevenfunc_14() {
    //  Invoked fn:round-half-to-even() on xs:float -INF.
    // XPath: round-half-to-even(xs:float(\"-INF\"))
    // Expected: -INF
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_15() {
    //  Invoked fn:round-half-to-even() on xs:double INF.
    // XPath: round-half-to-even(xs:double(\"INF\"))
    // Expected: INF
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_16() {
    //  Invoked fn:round-half-to-even() on xs:float INF.
    // XPath: round-half-to-even(xs:float(\"INF\"))
    // Expected: INF
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_001() {
    //  test fn:found-half-to-even on xs:double with large $precision argment
    // XPath: fn:round-half-to-even(3.567812E+3, 4294967296)
    // Expected: 3567.812
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_002() {
    //  test fn:found-half-to-even on integer with positive precision
    // XPath: fn:round-half-to-even(12345, 2)
    // Expected: 12345
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_003() {
    //  test fn:found-half-to-even on integer with negative precision
    // XPath: fn:round-half-to-even(12345, -2)
    // Expected: 12300
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_004() {
    //  test fn:found-half-to-even on short with positive precision
    // XPath: fn:round-half-to-even(xs:short(12345), 2)
    // Expected: 12345
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_005() {
    //  test fn:found-half-to-even on short with negative precision
    // XPath: fn:round-half-to-even(xs:short(12345), -2)
    // Expected: 12300
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_006() {
    //  test fn:found-half-to-even on decimal with positive precision
    // XPath: fn:round-half-to-even(12345.6, 2)
    // Expected: 12345.6
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_007() {
    //  test fn:found-half-to-even on decimal with negative precision
    // XPath: fn:round-half-to-even(12345.6, -2)
    // Expected: 12300
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_008() {
    //  test fn:found-half-to-even on float with positive precision
    // XPath: fn:round-half-to-even(xs:float(12345.6), 2)
    // Expected: 12345.6
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_009() {
    //  test fn:found-half-to-even on float with negative precision
    // XPath: fn:round-half-to-even(xs:float(12345.6), -2)
    // Expected: 12300
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_010() {
    //  test fn:found-half-to-even on double with positive precision
    // XPath: fn:round-half-to-even(123456e-2, 2)
    // Expected: 1234.56
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_011() {
    //  test fn:found-half-to-even on double with negative precision
    // XPath: fn:round-half-to-even(123456e-2, -2)
    // Expected: 1200
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_012() {
    //  test fn:found-half-to-even on xs:decimal with large $precision argment
    // XPath: fn:round-half-to-even(3.567812, 4294967296)
    // Expected: 3.567812
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn cbcl_round_half_to_even_013() {
    //  test EBV of fn:found-half-to-even with invalid type for argument 2
    // XPath: fn:boolean(fn:round-half-to-even(5, 1))
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}
