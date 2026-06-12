//! Test chunk 0 for fn:unordered
//! Contains 14 tests

use dep::xpath::stub_fnunordered;

#[test]
fn k_sequnorderedfunc_3() {
    //  A test whose essence is: `unordered(1) eq 1`.
    // XPath: unordered(1) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn k_sequnorderedfunc_4() {
    //  A test whose essence is: `count(unordered((1, 2, 3))) eq 3`.
    // XPath: count(unordered((1, 2, 3))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn k_sequnorderedfunc_5() {
    //  A test whose essence is: `count(unordered((1, 2, current-time()))) eq 3`.
    // XPath: count(unordered((1, 2, current-time()))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn k_sequnorderedfunc_7() {
    //  A test whose essence is: `deep-equal((1, 2, 3), unordered((1, 2, 3)))`.
    // XPath: deep-equal((1, 2, 3), unordered((1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn cbcl_fn_unordered_001() {
    //  Tests optimizations for unordered
    // XPath: boolean(unordered(reverse((1 to 10)[. div 2 = 2])))
    // Expected: true
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn cbcl_fn_unordered_002() {
    //  Tries EvaluateOptionalItem on fn:unordered
    // XPath: local-name(unordered(<a><b/><c/></a>/*[1]))
    // Expected: b
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn cbcl_fn_unordered_003() {
    //  Tests corner evaluation case
    // XPath: floor(unordered((1 to 10)[. div 2 = 0]))
    // Expected:
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_101() {
    //  unordered() on preceding-sibling axis
    // XPath: count(/works/employee[last()]/unordered(preceding-sibling::*
    // Expected: 12
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_102() {
    //  unordered() on preceding axis
    // XPath: count(/works/employee[last()]/unordered(preceding::*))
    // Expected: 54
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_103() {
    //  unordered() on ancestor-or-self axis
    // XPath: count((/works//day)[last()]/unordered(ancestor-or-self::*))
    // Expected: 4
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_104() {
    //  unordered() does not eliminate duplicates
    // XPath: count(unordered(reverse(for $i in (1,2,1) return /works/empl
    // Expected: 39
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_105() {
    //  unordered() does not prevent duplicate elimination
    // XPath: count(unordered(reverse(/works/employee[@name] | /works/empl
    // Expected: 13
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_106() {
    //  unordered() applied to selection in FLWOR expression
    // XPath: count(unordered(for $e in reverse(/works/employee) return $e
    // Expected: 13
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}

#[test]
fn fn_unordered_107() {
    //  unordered() applied to selection in FLWOR expression
    // XPath: count(for $e in unordered(reverse(/works/employee)) return $
    // Expected: 13
    // Calls stub function - will fail until fn:unordered is implemented
    let _ = stub_fnunordered();
}
