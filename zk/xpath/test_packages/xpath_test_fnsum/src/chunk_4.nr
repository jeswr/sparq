//! Test chunk 4 for fn:sum
//! Contains 6 tests

use dep::xpath::stub_fnsum;

#[test]
fn cbcl_sum_func_001() {
    //  Test sum(1 to 10)
    // XPath: sum(1 to 10)
    // Expected: 55
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn cbcl_sum_func_002() {
    //  Test sum(foreach)
    // XPath: sum(for $x in 1 to 10 return 5)
    // Expected: 50
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn cbcl_sum_func_003() {
    //  Test sum($x to $y)
    // XPath: for $x in 1 to 10, $y in 1 to 10 return concat('sum(',$x,' t
    // Expected: sum(1 to 1) = 1 sum(1 to 2) = 3 sum(1 to
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn cbcl_sum_func_004() {
    //  Tests boolean call to sum
    // XPath: boolean(sum((1 to 10)[. mod 2 = 0]))
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn cbcl_sum_func_005() {
    //  Tests sum of a sum
    // XPath: sum(sum((1 to 10)[. mod 2 = 0],()))
    // Expected: 30
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn cbcl_sum_func_006() {
    //  Tests sum of two integer converts to plus
    // XPath: sum((1,exactly-one((1 to 10)[. div 2 = 2])))
    // Expected: 5
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}
