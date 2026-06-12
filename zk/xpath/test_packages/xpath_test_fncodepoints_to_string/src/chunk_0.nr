//! Test chunk 0 for fn:codepoints-to-string
//! Contains 42 tests

use dep::xpath::stub_fncodepoints_to_string;

#[test]
fn fn_codepoints_to_string1args_1() {
    // Test codepoints-to-string with variety of characters
    // XPath: codepoints-to-string((98,223,1682,12365,63744))
    // Expected: b????
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string1args_2() {
    // Test codepoints-to-string with an empty sequence argument
    // XPath: codepoints-to-string(())
    // Expected:
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_3() {
    //  Evaluation of an 'fn:codepoints-to-string' with argument set to codepoint 49 (s
    // XPath: fn:codepoints-to-string(49)
    // Expected: 1
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_4() {
    //  Evaluation of an 'fn-codepoints-to-string' with argument set to codepoint 97 (a
    // XPath: fn:codepoints-to-string(97)
    // Expected: a
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_5() {
    //  Evaluation of an 'fn:codepoints-to-string' with argument set to codepoints 49,
    // XPath: fn:codepoints-to-string((49,97))
    // Expected: 1a
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_6() {
    //  Evaluation of an 'fn:codepoints-to-string' with argument set to the codepoints
    // XPath: fn:codepoints-to-string((35, 42, 94, 36))
    // Expected: #*^$
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_7() {
    //  Evaluation of an 'fn:codepoints-to-string' with argument set to codepoints
    // XPath: fn:codepoints-to-string((99,111,100,101,112,111,105,110,116,
    // Expected: codepoints-to-string
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_8() {
    //  Evaluation of an 'fn:codepoints-to-string' used as argument to 'xs:string()' fu
    // XPath: xs:string(fn:codepoints-to-string((65,32,83,116,114,105,110,
    // Expected: A String
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_9() {
    //  Evaluation of an 'fn:codepoints-to-string' used as argument to the 'fn:upper-ca
    // XPath: fn:upper-case(fn:codepoints-to-string((65,32,83,84,82,73,78,
    // Expected: A STRING
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_10() {
    //  Evaluation of an 'fn:codepoints-to-string' used as argument to the 'fn:lower-ca
    // XPath: fn:lower-case(fn:codepoints-to-string((97,32,115,116,114,105
    // Expected: a string
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_11() {
    //  Evaluation of an 'fn:codepoints-to-string' that uses xs:integer as argument and
    // XPath: fn:codepoints-to-string(xs:integer(97))
    // Expected: a
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_12() {
    //  Evaluation of an 'fn:codepoints-to-string' that uses fn:avg/xs:integer function
    // XPath: fn:codepoints-to-string(xs:integer(fn:avg((65,32,83,116,114,
    // Expected: [
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_13() {
    //  Evaluation of an 'fn:codepoints-to-string' that is used as argument to fn:conca
    // XPath: fn:concat(fn:codepoints-to-string((49,97)),\"1a\")
    // Expected: 1a1a
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_15() {
    //  Evaluation of an 'fn:codepoints-to-string' that is used as argument to fn:strin
    // XPath: fn:string-length(fn:codepoints-to-string((49,97)))
    // Expected: 2
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn fn_codepoints_to_string_16() {
    //  Evaluation of an 'fn:codepoints-to-string' that is used as argument to fn:strin
    // XPath: fn:string-join((fn:codepoints-to-string((49,97)),'ab'),'')
    // Expected: 1aab
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_3() {
    //  A test whose essence is: `codepoints-to-string(()) eq ''`.
    // XPath: codepoints-to-string(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_4() {
    //  A test whose essence is: `codepoints-to-string((87, 36, 56, 87, 102, 96)) eq 'W
    // XPath: codepoints-to-string((87, 36, 56, 87, 102, 96)) eq \"W$8Wf`\
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_9() {
    //  A test whose essence is: `codepoints-to-string(9) eq '&#x9;'`.
    // XPath: codepoints-to-string(9) eq \" \"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_10() {
    //  A test whose essence is: `codepoints-to-string(10) eq '&#xA;'`.
    // XPath: codepoints-to-string(10) eq \" \"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_11b() {
    //  Codepoint 11 is invalid in XML 1.0 but valid in XML 1.1.
    // XPath: string-to-codepoints(codepoints-to-string(11))
    // Expected: 11
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_12b() {
    //  Codepoint 12 is invalid in XML 1.0 but valid in XML 1.1.
    // XPath: string-to-codepoints(codepoints-to-string(12))
    // Expected: 12
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_13() {
    //  A test whose essence is: `codepoints-to-string(13) eq '&#xd;'`.
    // XPath: codepoints-to-string(13) eq \"&#xD;\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_16() {
    //  A test whose essence is: `codepoints-to-string(32) eq '&#x20;'`.
    // XPath: codepoints-to-string(32) eq \" \"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_17() {
    //  A test whose essence is: `codepoints-to-string(27637) eq '&#x6bf5;'`.
    // XPath: codepoints-to-string(27637) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_18() {
    //  A test whose essence is: `codepoints-to-string(55295) eq '&#xD7FF;'`.
    // XPath: codepoints-to-string(55295) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_21() {
    //  A test whose essence is: `codepoints-to-string(57344) eq '&#xE000;'`.
    // XPath: codepoints-to-string(57344) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_22() {
    //  A test whose essence is: `codepoints-to-string(61438) eq '&#xEFFE;'`.
    // XPath: codepoints-to-string(61438) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_23() {
    //  A test whose essence is: `codepoints-to-string(65533) eq '&#xFFFD;'`.
    // XPath: codepoints-to-string(65533) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_26() {
    //  A test whose essence is: `codepoints-to-string(65536) eq '&#x10000;'`.
    // XPath: codepoints-to-string(65536) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_27() {
    //  A test whose essence is: `codepoints-to-string(589823) eq '&#x8FFFF;'`.
    // XPath: codepoints-to-string(589823) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn k_codepointtostringfunc_28() {
    //  A test whose essence is: `codepoints-to-string(1114111) eq '&#x10FFFF;'`.
    // XPath: codepoints-to-string(1114111) eq \"?\"
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_010() {
    //  test fn:boolean on fn:codepoints-to-string
    // XPath: fn:boolean(fn:codepoints-to-string( 65 to 76 ))
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_013() {
    //  test fn:boolean on fn:codepoints-to-string
    // XPath: fn:boolean(fn:codepoints-to-string( 65 ))
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_014() {
    //  test fn:boolean on fn:codepoints-to-string
    // XPath: fn:boolean(fn:codepoints-to-string( () ))
    // Expected: false
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_015() {
    //  fn:codepoints-to-string on multibyte ranges
    // XPath: deep-equal( fn:string-to-codepoints(fn:codepoints-to-string(
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_016() {
    //  fn:codepoints-to-string on multibyte ranges
    // XPath: deep-equal( fn:string-to-codepoints(fn:codepoints-to-string(
    // Expected: true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_017() {
    //  Test codepoints to string on a range, returning a boolean
    // XPath: for $x in 32 to 64 return boolean(codepoints-to-string($x to
    // Expected: true true true true true true true true
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_018() {
    //  Tries to force evaluate to item on codepoint-to-string
    // XPath: if(5 < exactly-one((1 to 10)[. div 2 = 5])) then codepoints-
    // Expected: !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGH
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_019() {
    //  Tests for calling EvaluateToOptionalItem
    // XPath: for $x in 65 to 75 return string-length(codepoints-to-string
    // Expected: 11 11 11 11 11 11 11 11 11 11 11
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_020() {
    //  Tests false returns from TryArguments
    // XPath: for $x in 65 to 75 return boolean(codepoints-to-string($x[.
    // Expected: false false false false false false fals
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_025() {
    //  Tests valid codepoints 0x9, 0xA
    // XPath: for $x in 9 to 9 return codepoints-to-string($x to $x+1)
    // Expected: '  '
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}

#[test]
fn cbcl_codepoints_to_string_026() {
    //  Tests valid codepoint 0xD
    // XPath: for $x in 13 to 13 return codepoints-to-string($x to $x)
    // Expected:
    // Calls stub function - will fail until fn:codepoints-to-string is implemented
    let _ = stub_fncodepoints_to_string();
}
