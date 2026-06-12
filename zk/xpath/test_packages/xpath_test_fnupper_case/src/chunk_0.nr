//! Test chunk 0 for fn:upper-case
//! Contains 23 tests

use dep::xpath::stub_fnupper_case;

#[test]
fn fn_upper_case1args_1() {
    //  Evaluates The 'upper-case' function with the arguments set as follows: $arg = x
    // XPath: fn:upper-case(xs:string(\"This is a characte\"))
    // Expected: THIS IS A CHARACTE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case1args_2() {
    //  Evaluates The 'upper-case' function with the arguments set as follows: $arg = x
    // XPath: fn:upper-case(xs:string(\"This is a characte\"))
    // Expected: THIS IS A CHARACTE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case1args_3() {
    //  Evaluates The 'upper-case' function with the arguments set as follows: $arg = x
    // XPath: fn:upper-case(xs:string(\"This is a characte\"))
    // Expected: THIS IS A CHARACTE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_1() {
    //  Evaluation of upper-case function as per example 1 (for this function) from the
    // XPath: fn:upper-case(\"ABc!D\")
    // Expected: ABC!D
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_2() {
    //  Evaluation of upper-case function using the empty sequence Uses the count funct
    // XPath: fn:count(fn:upper-case(()))
    // Expected: 1
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_3() {
    //  Evaluation of upper-case function that uses only numbers as part of argument Us
    // XPath: fn:upper-case(\"12345\")
    // Expected: 12345
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_4() {
    //  Evaluation of upper-case function that uses both numbers and letters as part of
    // XPath: fn:upper-case(\"12345abcd\")
    // Expected: 12345ABCD
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_5() {
    //  Evaluation of upper-case function that uses only upper case letters as part of
    // XPath: fn:upper-case(\"ABCD\")
    // Expected: ABCD
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_6() {
    //  Evaluation of upper-case function that uses only lower case letters as part of
    // XPath: fn:upper-case(\"abcde\")
    // Expected: ABCDE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_7() {
    //  Evaluation of upper-case function that uses both upper and lower case letters a
    // XPath: fn:upper-case(\"ABCDEabcde\")
    // Expected: ABCDEABCDE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_8() {
    //  Evaluation of upper-case function that uses the empty string as part of argumen
    // XPath: fn:count(fn:upper-case(\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_9() {
    //  Evaluation of upper-case function that uses the 'lower-case' as part of argumen
    // XPath: fn:upper-case(lower-case(\"AbcDH\"))
    // Expected: ABCDH
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_10() {
    //  Evaluation of upper-case function with argument set to '*****'
    // XPath: fn:upper-case(\"*****\")
    // Expected: *****
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_11() {
    //  Evaluation of upper-case function with argument set to another upper case funct
    // XPath: fn:upper-case(upper-case(\"zzzzz\"))
    // Expected: ZZZZZ
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_12() {
    //  Evaluation of upper-case function as an argument to the 'fn:boolean' function
    // XPath: fn:boolean(fn:upper-case(\"abcde\"))
    // Expected: true
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_13() {
    //  Evaluation of upper-case function as an argument to the 'fn:concat' function
    // XPath: fn:concat(fn:upper-case(\"abcde\"), fn:upper-case(\"fghi\"))
    // Expected: ABCDEFGHI
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_14() {
    //  Evaluation of upper-case function as an argument to the 'fn:not' function
    // XPath: fn:not(fn:upper-case(\"abcde\"))
    // Expected: false
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_15() {
    //  Evaluation of upper-case function with argument set to '%$#@!'
    // XPath: fn:upper-case(\"%$#@!\")
    // Expected: %$#@!
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_16() {
    //  Evaluation of upper-case function with argument set to 'upper-case'
    // XPath: fn:upper-case(\"upper-case\")
    // Expected: UPPER-CASE
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_17() {
    //  Evaluation of upper-case function as part of a boolean expression
    // XPath: fn:upper-case(\"abc\") and fn:upper-case(\"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn fn_upper_case_20() {
    //  Upper-case of a title-case character
    // XPath: fn:string-to-codepoints(fn:upper-case(fn:codepoints-to-strin
    // Expected: 458
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn k_uppercasefunc_3() {
    //  A test whose essence is: `upper-case(()) eq ''`.
    // XPath: upper-case(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}

#[test]
fn k_uppercasefunc_4() {
    //  A test whose essence is: `lower-case('ABc!D') eq 'abc!d'`.
    // XPath: lower-case(\"ABc!D\") eq \"abc!d\"
    // Expected: true
    // Calls stub function - will fail until fn:upper-case is implemented
    let _ = stub_fnupper_case();
}
