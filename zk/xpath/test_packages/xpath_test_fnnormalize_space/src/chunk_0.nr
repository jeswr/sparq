//! Test chunk 0 for fn:normalize-space
//! Contains 20 tests

use dep::xpath::stub_fnnormalize_space;

#[test]
fn fn_normalize_space1args_1() {
    //  Evaluates The 'normalize-space' function with the arguments set as follows: $ar
    // XPath: fn:normalize-space(\"This is a charac\")
    // Expected: This is a charac
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space1args_2() {
    //  Evaluates The 'normalize-space' function with the arguments set as follows: $ar
    // XPath: fn:normalize-space(\"This is a ch\")
    // Expected: This is a ch
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space1args_3() {
    //  Evaluates The 'normalize-space' function with the arguments set as follows: $ar
    // XPath: fn:normalize-space(\"This is a charac\")
    // Expected: This is a charac
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space0args_1() {
    // Test normalize-space without argument
    // XPath: //doc/normalize-space(zero-or-one(a[normalize-space() = 'Hel
    // Expected: Hello, How are you?
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_1() {
    //  Evaluation of fn-normalize-space function as per example 1 for this function fr
    // XPath: fn:normalize-space(\" The wealthy curled darlings of our nat
    // Expected: The wealthy curled darlings of our natio
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_3() {
    //  Evaluation of fn-normalize-space function with argument containing one tab char
    // XPath: fn:normalize-space(\"This text should contains no tabs\")
    // Expected: This text should contains no tabs
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_4() {
    //  Evaluation of fn-normalize-space function with argument containing one newline
    // XPath: fn:normalize-space(\"This text should contains no newline ch
    // Expected: This text should contains no newline cha
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_5() {
    //  Evaluation of fn-normalize-space function with argument containing multiple tab
    // XPath: fn:normalize-space(\"This text should contains no tab charac
    // Expected: This text should contains no tab charact
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_6() {
    //  Evaluation of fn-normalize-space function with argument containing multiple new
    // XPath: fn:normalize-space(\"This text should contains no newline ch
    // Expected: This text should contains no newline cha
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_7() {
    //  Evaluation of fn-normalize-space function with argument containing a tab and a
    // XPath: fn:normalize-space(\"This text should contains no tabs or ne
    // Expected: This text should contains no tabs or new
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_8() {
    //  Evaluation of fn-normalize-space function with argument containing multiple tab
    // XPath: fn:normalize-space(\"This  text  should  contains  no tabs o
    // Expected: This text should contains no tabs or new
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_19() {
    //  Evaluation of fn-normalize-space function with argument string containing tabs
    // XPath: fn:normalize-space(\" 12345\")
    // Expected: 12345
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_20() {
    //  Evaluation of fn-normalize-space function with argument referencing fn:string f
    // XPath: fn:normalize-space(fn:string(\" ABC \"))
    // Expected: ABC
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn fn_normalize_space_21() {
    //  Evaluation of fn-normalize-space function with argument set to another fn:norma
    // XPath: fn:normalize-space(fn:normalize-space(\" ABC\"))
    // Expected: ABC
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_3() {
    //  A test whose essence is: `normalize-space('foo') eq 'foo'`.
    // XPath: normalize-space(\"foo\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_4() {
    //  A test whose essence is: `normalize-space(' foo') eq 'foo'`.
    // XPath: normalize-space(\" foo\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_5() {
    //  A test whose essence is: `normalize-space('foo ') eq 'foo'`.
    // XPath: normalize-space(\"foo \") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_6() {
    //  A test whose essence is: `normalize-space(()) eq ''`.
    // XPath: normalize-space(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_7() {
    //  A test whose essence is: `normalize-space('f o o ') eq 'f o o'`.
    // XPath: normalize-space(\"f o o \") eq \"f o o\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}

#[test]
fn k_normalizespacefunc_8() {
    //  A test whose essence is: `normalize-space(' 143 1239 fhjkls ') eq '143 1239 fhj
    // XPath: normalize-space(\" 143 1239 fhjkls \") eq \"143 1239 fhjkls\
    // Expected: true
    // Calls stub function - will fail until fn:normalize-space is implemented
    let _ = stub_fnnormalize_space();
}
