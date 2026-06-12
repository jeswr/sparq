//! Test chunk 0 for fn:lower-case
//! Contains 23 tests

use dep::xpath::stub_fnlower_case;

#[test]
fn fn_lower_case1args_1() {
    //  Evaluates The 'lower-case' function with the arguments set as follows: $arg = x
    // XPath: fn:lower-case(xs:string(\"This is a characte\"))
    // Expected: this is a characte
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case1args_2() {
    //  Evaluates The 'lower-case' function with the arguments set as follows: $arg = x
    // XPath: fn:lower-case(xs:string(\"This is a characte\"))
    // Expected: this is a characte
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case1args_3() {
    //  Evaluates The 'lower-case' function with the arguments set as follows: $arg = x
    // XPath: fn:lower-case(xs:string(\"This is a characte\"))
    // Expected: this is a characte
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_1() {
    //  Evaluation of lower-case function as per example 1 (for this function) from the
    // XPath: fn:lower-case(\"ABc!D\")
    // Expected: abc!d
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_2() {
    //  Evaluation of lower-case function using the empty sequence Uses the count funct
    // XPath: fn:count(fn:lower-case(()))
    // Expected: 1
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_3() {
    //  Evaluation of lower-case function that uses only numbers as part of argument Us
    // XPath: fn:lower-case(\"12345\")
    // Expected: 12345
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_4() {
    //  Evaluation of lower-case function that uses both numbers and letters as part of
    // XPath: fn:lower-case(\"12345abcd\")
    // Expected: 12345abcd
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_5() {
    //  Evaluation of lower-case function that uses only upper case letters as part of
    // XPath: fn:lower-case(\"ABCD\")
    // Expected: abcd
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_6() {
    //  Evaluation of lower-case function that uses only lower case letters as part of
    // XPath: fn:lower-case(\"abcde\")
    // Expected: abcde
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_7() {
    //  Evaluation of lower-case function that uses both upper and lower case letters a
    // XPath: fn:lower-case(\"ABCDEabcde\")
    // Expected: abcdeabcde
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_8() {
    //  Evaluation of lower-case function that uses the empty string as part of argumen
    // XPath: fn:count(fn:lower-case(\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_9() {
    //  Evaluation of lower-case function that uses the 'upper-case' as part of argumen
    // XPath: fn:lower-case(upper-case(\"AbcDH\"))
    // Expected: abcdh
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_10() {
    //  Evaluation of lower-case function with argument set to '*****'
    // XPath: fn:lower-case(\"*****\")
    // Expected: *****
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_11() {
    //  Evaluation of lower-case function with argument set to another lower-case funct
    // XPath: fn:lower-case(lower-case(\"zzzzz\"))
    // Expected: zzzzz
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_12() {
    //  Evaluation of lower-case function as an argument to the 'fn:boolean' function
    // XPath: fn:boolean(fn:lower-case(\"abcde\"))
    // Expected: true
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_13() {
    //  Evaluation of lower-case function as an argument to the 'fn:concat' function
    // XPath: fn:concat(fn:lower-case(\"abcde\"), fn:lower-case(\"fghi\"))
    // Expected: abcdefghi
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_14() {
    //  Evaluation of lower-case function as an argument to the 'fn:not' function
    // XPath: fn:not(fn:lower-case(\"abcde\"))
    // Expected: false
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_15() {
    //  Evaluation of lower-case function with argument set to '%$#@!'
    // XPath: fn:lower-case(\"%$#@!\")
    // Expected: %$#@!
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_16() {
    //  Evaluation of lower-case function with argument set to 'lower-case'
    // XPath: fn:lower-case(\"lower-case\")
    // Expected: lower-case
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_17() {
    //  Evaluation of lower-case function as part of a boolean expression
    // XPath: fn:lower-case(\"abc\") and fn:lower-case(\"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn fn_lower_case_20() {
    //  Lower-case of a title-case character
    // XPath: fn:string-to-codepoints(fn:lower-case(fn:codepoints-to-strin
    // Expected: 460
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn k_lowercasefunc_3() {
    //  A test whose essence is: `lower-case(()) eq ''`.
    // XPath: lower-case(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}

#[test]
fn k_lowercasefunc_4() {
    //  A test whose essence is: `upper-case('abCd0') eq 'ABCD0'`.
    // XPath: upper-case(\"abCd0\") eq \"ABCD0\"
    // Expected: true
    // Calls stub function - will fail until fn:lower-case is implemented
    let _ = stub_fnlower_case();
}
