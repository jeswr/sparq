//! Test chunk 0 for fn:remove
//! Contains 22 tests

use dep::xpath::stub_fnremove;

#[test]
fn fn_remove_mix_args_008() {
    //  arg1:sequence of string,integer,arg2: integer
    // XPath: fn:remove( (xs:integer(\"100\"), xs:string(\"abc\")), 1)
    // Expected: "abc"
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn fn_remove_mix_args_010() {
    //  arg1: sequence of string,float ,arg2: integer
    // XPath: fn:remove( (xs:float(\"1.01\"), xs:string(\"a\")), 2)
    // Expected: xs:float(1.01)
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn fn_remove_mix_args_017() {
    //  arg1: sequence of boolean ,arg2: integer
    // XPath: fn:remove( (xs:boolean(\"0\")), 2 )
    // Expected: false
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_3() {
    //  A test whose essence is: `count(remove((1, 2, 3), 0)) eq 3`.
    // XPath: count(remove((1, 2, 3), 0)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_4() {
    //  A test whose essence is: `count(remove((1, 2, 3), -4)) eq 3`.
    // XPath: count(remove((1, 2, 3), -4)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_6() {
    //  A test whose essence is: `remove(('one', 2, 3), 1) instance of xs:integer+`.
    // XPath: remove((\"one\", 2, 3), 1) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_7() {
    //  A test whose essence is: `remove((1, 'two', 3), 2) instance of xs:integer+`.
    // XPath: remove((1, \"two\", 3), 2) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_8() {
    //  A test whose essence is: `remove((1, 2, 'three'), 3) instance of xs:integer+`.
    // XPath: remove((1, 2, \"three\"), 3) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_9() {
    //  A test whose essence is: `count(remove(('one', 2, 3), 1)) eq 2`.
    // XPath: count(remove((\"one\", 2, 3), 1)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_10() {
    //  A test whose essence is: `count(remove((1, 'two', 3), 2)) eq 2`.
    // XPath: count(remove((1, \"two\", 3), 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_11() {
    //  A test whose essence is: `count(remove((1, 2, 'three'), 3)) eq 2`.
    // XPath: count(remove((1, 2, \"three\"), 3)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_12() {
    //  A test whose essence is: `remove((3.1, 'four'), 1)`.
    // XPath: remove((3.1, \"four\"), 1)
    // Expected: "four"
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_19() {
    //  An expression involving the eq operator that trigger certain optimization paths
    // XPath: count(remove(current-time(), 1)) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_20() {
    //  An expression involving the eq operator that trigger certain optimization paths
    // XPath: empty(remove(current-time(), 1))
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_21() {
    //  Apply a predicate to the result of fn:remove().
    // XPath: remove((1, 2, 3, current-time()), 4)[last()]
    // Expected: 3
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_22() {
    //  Apply a predicate to the result of fn:remove().
    // XPath: remove((1, 2, 3, current-time()), 4)[last() - 1]
    // Expected: 2
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_23() {
    //  Apply a predicate to the result of fn:remove().
    // XPath: remove((1, 2, 3, current-time()), 9)[last() - 1]
    // Expected: 3
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn k_seqremovefunc_24() {
    //  Apply a predicate to the result of fn:remove().
    // XPath: empty(remove((1, 2, 3, current-time()), 9)[last() - 10])
    // Expected: true
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn cbcl_fn_remove_001() {
    //  Tests with non-value position
    // XPath: remove(1 to 10,exactly-one((1 to 10)[. div 2 = 2]))
    // Expected: 1 2 3 5 6 7 8 9 10
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn cbcl_fn_remove_002() {
    //  Tests removal from range value
    // XPath: remove(1 to 10,4)
    // Expected: 1 2 3 5 6 7 8 9 10
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn cbcl_fn_remove_003() {
    //  Tests removal from non-range value
    // XPath: remove((1 to 10)[. mod 2 = 0],4)
    // Expected: 2 4 6 10
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}

#[test]
fn cbcl_fn_remove_004() {
    //  Tests removal from a sequence of values
    // XPath: remove(((1 to 10)[. mod 2 = 0],\"blah\",(1 to 10)[. mod 2 =
    // Expected: 2 4 6 10 blah 2 4 6 8 10
    // Calls stub function - will fail until fn:remove is implemented
    let _ = stub_fnremove();
}
