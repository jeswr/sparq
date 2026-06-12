//! Test chunk 0 for fn:subsequence
//! Contains 50 tests

use dep::xpath::stub_fnsubsequence;

#[test]
fn fn_subsequence_mix_args_011() {
    //  arg1: sequence of string & float , arg2 & arg3: integer
    // XPath: fn:subsequence ( (\"a\", xs:float(\"NaN\"), \"b\", \"c\"), 0
    // Expected: "a"
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn fn_subsequence_mix_args_016() {
    //  arg1: sequence of string & double , arg2 & arg3: integer
    // XPath: fn:subsequence ( (\"a\", xs:double(\"INF\"), \"b\", \"c\"),
    // Expected: xs:double("INF")
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn fn_subsequence_mix_args_018() {
    //  arg1: sequence of string & boolean, arg2 & arg3: integer
    // XPath: fn:subsequence ( (\"a\", xs:boolean(\"0\"), \"b\", \"c\"), 2
    // Expected: false
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn fn_subsequence_mix_args_022() {
    //  arg1: sequence of string & dateTime, arg2 & arg3: integer
    // XPath: fn:subsequence ( (\"a\", xs:dateTime(\"1972-12-31T00:00:00\"
    // Expected: "a"
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn fn_subsequence_mix_args_025() {
    // composite query of subsequence with map operator and count function
    // XPath: document{<M id=\"1\"/>}/M/count(subsequence(. ! (., @id), 1,
    // Expected: 2
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn fn_subsequence_mix_args_026() {
    // mixed sequence operations with tail and subsequence
    // XPath: tail(subsequence((1 to 2),1,2))
    // Expected: 2
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_5() {
    //  A test whose essence is: `empty(subsequence((1, 2, 3), 2, -10))`.
    // XPath: empty(subsequence((1, 2, 3), 2, -10))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_6() {
    //  A test whose essence is: `count(subsequence((1, 2, 3, 'four'), 4)) eq 1`.
    // XPath: count(subsequence((1, 2, 3, \"four\"), 4)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_7() {
    //  A test whose essence is: `count(subsequence((1, 2, 3, 'four'), 4, 1)) eq 1`.
    // XPath: count(subsequence((1, 2, 3, \"four\"), 4, 1)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_8() {
    //  A test whose essence is: `subsequence((1, 2, 3.1, 'four'), 4)`.
    // XPath: subsequence((1, 2, 3.1, \"four\"), 4)
    // Expected: "four"
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_9() {
    //  A test whose essence is: `subsequence((1, 2, 3.1, 'four'), 4, 1)`.
    // XPath: subsequence((1, 2, 3.1, \"four\"), 4, 1)
    // Expected: "four"
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_10() {
    //  A test whose essence is: `subsequence((1, 2, 3.1, 'four'), 3, 1) instance of xs
    // XPath: subsequence((1, 2, 3.1, \"four\"), 3, 1) instance of xs:deci
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_11() {
    //  A test whose essence is: `subsequence((1, 2, 3.1, 'four'), 1, 1)`.
    // XPath: subsequence((1, 2, 3.1, \"four\"), 1, 1)
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_12() {
    //  A test whose essence is: `empty(subsequence((1, 2, 3, 'four'), 4, -3))`.
    // XPath: empty(subsequence((1, 2, 3, \"four\"), 4, -3))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_13() {
    //  A test whose essence is: `empty(subsequence((1, 2, 3, 'four'), -4, -3))`.
    // XPath: empty(subsequence((1, 2, 3, \"four\"), -4, -3))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_14() {
    //  A test whose essence is: `subsequence((1, 2, 3), 1, 1) eq 1`.
    // XPath: subsequence((1, 2, 3), 1, 1) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_15() {
    //  A test whose essence is: `subsequence((1, 2, 3), 1, 1) eq 1`.
    // XPath: subsequence((1, 2, 3), 1, 1) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_16() {
    //  A test whose essence is: `subsequence((1, 2, 3), 3) eq 3`.
    // XPath: subsequence((1, 2, 3), 3) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_17() {
    //  A test whose essence is: `count(subsequence((1, 2, 3), 1, 1)) eq 1`.
    // XPath: count(subsequence((1, 2, 3), 1, 1)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_18() {
    //  A test whose essence is: `count(subsequence((1, 2, 3), 1, 3)) eq 3`.
    // XPath: count(subsequence((1, 2, 3), 1, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_19() {
    //  A test whose essence is: `count(subsequence((1, 2, 3, 'four'), 4)) eq 1`.
    // XPath: count(subsequence((1, 2, 3, \"four\"), 4)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_20() {
    //  A test whose essence is: `count(subsequence(1 to 3, 1, 1)) eq 1`.
    // XPath: count(subsequence(1 to 3, 1, 1)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_21() {
    //  A test whose essence is: `empty(subsequence((1, 2), 4))`.
    // XPath: empty(subsequence((1, 2), 4))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_22() {
    //  A test whose essence is: `subsequence((5, 6, 7, 8), 2, 1) eq 6`.
    // XPath: subsequence((5, 6, 7, 8), 2, 1) eq 6
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_23() {
    //  A test whose essence is: `count(subsequence((1, 2), 2)) eq 1`.
    // XPath: count(subsequence((1, 2), 2)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_24() {
    //  A test whose essence is: `count(subsequence((1, 2, 3, 'four'), 2)) eq 3`.
    // XPath: count(subsequence((1, 2, 3, \"four\"), 2)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_25() {
    //  A test whose essence is: `count(subsequence((1, 2, 3, 'four'), 2, 2)) eq 2`.
    // XPath: count(subsequence((1, 2, 3, \"four\"), 2, 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_26() {
    //  A test whose essence is: `subsequence((1, 2, 3.1, 'four'), 1, 1)`.
    // XPath: subsequence((1, 2, 3.1, \"four\"), 1, 1)
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_31() {
    //  A test whose essence is: `count(subsequence((1, 2, 2, current-time()), 2, 2)) e
    // XPath: count(subsequence((1, 2, 2, current-time()), 2, 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_32() {
    //  A test whose essence is: `count(subsequence(remove(current-time(), 1), 1, 1)) e
    // XPath: count(subsequence(remove(current-time(), 1), 1, 1)) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_33() {
    //  A test whose essence is: `deep-equal(1, subsequence((1, 2, current-time()), 1,
    // XPath: deep-equal(1, subsequence((1, 2, current-time()), 1, 1))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_34() {
    //  Using subsequence inside a predicate.
    // XPath: (1)[deep-equal(1, subsequence((1, 2, current-time()), 1, 1))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_35() {
    //  A test whose essence is: `empty(subsequence((current-time(), 1), 4))`.
    // XPath: empty(subsequence((current-time(), 1), 4))
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_36() {
    //  A test whose essence is: `count(subsequence((current-time(), 1), 4)) eq 0`.
    // XPath: count(subsequence((current-time(), 1), 4)) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_37() {
    //  A test whose essence is: `count(subsequence((current-time(), 2 , 3), 1)) eq 3`.
    // XPath: count(subsequence((current-time(), 2 , 3), 1)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_38() {
    //  A test whose essence is: `count(subsequence((current-time(), 2 , 3), 3)) eq 1`.
    // XPath: count(subsequence((current-time(), 2 , 3), 3)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_39() {
    //  A test whose essence is: `count(subsequence((current-time(), 2 , 3, 4), 2, 2))
    // XPath: count(subsequence((current-time(), 2 , 3, 4), 2, 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_40() {
    //  Apply a predicate to the result of fn:subsequence().
    // XPath: subsequence((1, 2, 3, current-time(), 5, 6, 7), 1, 1)[1]
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_41() {
    //  Apply a predicate to the result of fn:subsequence().
    // XPath: subsequence((1, 2, 3, current-time(), 5, 6, 9), 7)[last()]
    // Expected: 9
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_42() {
    //  Apply a predicate to the result of fn:subsequence().
    // XPath: subsequence((1, 2, 3, current-time(), 5, 6, 7), 4)[last() -
    // Expected: 6
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_43() {
    //  Apply a predicate to the result of fn:subsequence().
    // XPath: empty(subsequence((1, 2, 3, current-time(), 5, 6, 7), 4, 1)[
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k_seqsubsequencefunc_44() {
    //  Apply a predicate to the result of fn:subsequence().
    // XPath: empty(subsequence((1, 2, 3, current-time(), 5, 6, 7), 1, 1)[
    // Expected: true
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k2_seqsubsequencefunc_1() {
    //  Use fn:subsequence where the input is via variable references.
    // XPath: let $start := (current-time(), 2)[2] treat as xs:integer,
    // Expected: 2
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k2_seqsubsequencefunc_4() {
    //  Ensure rounding is done properly, 1.1.
    // XPath: fn:subsequence((1,2,3), 1.1, 1)
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k2_seqsubsequencefunc_5() {
    //  Ensure rounding is done properly, 1.8.
    // XPath: fn:subsequence((1,2,3), 1.8, 1)
    // Expected: 2
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k2_seqsubsequencefunc_6() {
    //  Ensure rounding is done properly, 1.4.
    // XPath: fn:subsequence((1,2,3), 1.4, 1.4)
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn k2_seqsubsequencefunc_8() {
    //  Have subsequence inside a function body, to trap bugs related to inference, rew
    // XPath: let $f :=function() { subsequence(subsequence((1, 2, 3, 4),
    // Expected: 3
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_001() {
    //  Test fn:subsequence on -INF and INF
    // XPath: count(subsequence(1 to 10, xs:double(\"-INF\"), xs:double(\"
    // Expected: 0
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_002() {
    //  Test fn:subsequence with length NaN
    // XPath: count(subsequence(1 to 10, 2, xs:double(\"NaN\")))
    // Expected: 0
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_003() {
    //  Test fn:subsequence with starting position NaN
    // XPath: count(subsequence(1 to 10, xs:double(\"NaN\"), 4))
    // Expected: 0
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}
