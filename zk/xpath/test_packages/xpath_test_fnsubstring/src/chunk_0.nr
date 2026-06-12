//! Test chunk 0 for fn:substring
//! Contains 45 tests

use dep::xpath::stub_fnsubstring;

#[test]
fn fn_substring_2() {
    // Evaluation of substring function as per example 2 (for this function) from the F
    // XPath: fn:substring(\"metadata\", 4, 3)
    // Expected: ada
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_3() {
    // Evaluation of substring function as per example 3 (for this function) from the F
    // XPath: fn:substring(\"12345\", 1.5, 2.6)
    // Expected: 234
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_4() {
    // Evaluation of substring function as per example 4 (for this function) from the F
    // XPath: fn:substring(\"12345\", 0, 3)
    // Expected: 12
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_5() {
    // Evaluation of substring function as per example 5 (for this function) from the F
    // XPath: fn:count(fn:substring(\"12345\", 5, -3))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_6() {
    // Evaluation of substring function as per example 6 (for this function) from the F
    // XPath: fn:substring(\"12345\", -3, 5)
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_7() {
    // Evaluation of substring function as per example 7 (for this function) from the F
    // XPath: fn:count(fn:substring(\"12345\", 0 div 0E0, 3))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_8() {
    // Evaluation of substring function as per example 8 (for this function) from the F
    // XPath: fn:count(fn:substring(\"12345\", 1, 0 div 0E0))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_9() {
    // Evaluation of substring function as per example 9 (for this function) from the F
    // XPath: fn:count(fn:substring((), 1, 3))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_10() {
    // Evaluation of substring function as per example 10 (for this function) from the
    // XPath: fn:substring(\"12345\", -42, 1 div 0E0)
    // Expected: 12345
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_11() {
    // Evaluation of substring function as per example 11 (for this function) from the
    // XPath: fn:count(fn:substring(\"12345\", -1 div 0E0, 1 div 0E0))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_12() {
    // Evaluation of substring function, where the source string is the empty string Us
    // XPath: fn:count(fn:substring(\"\",0))
    // Expected: 1
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_13() {
    // Evaluation of substring function, as an argument to an 'fn:boolean' function'
    // XPath: fn:boolean(fn:substring(\"ABC\",1))
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_14() {
    // Evaluation of substring function, as an argument to an 'fn:not' function'
    // XPath: fn:not(fn:substring(\"ABC\",1))
    // Expected: false
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_15() {
    // Evaluation of substring function, as an argument to another 'fn:substring' funct
    // XPath: fn:substring(fn:substring(\"ABCDE\",1),1)
    // Expected: ABCDE
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_16() {
    // Evaluation of substring function, where the source string is the string 'substri
    // XPath: fn:substring(\"substring\",1)
    // Expected: substring
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_17() {
    // Evaluation of substring function as an argument to a concat function.
    // XPath: fn:concat(fn:substring(\"ABC\",1),\"DEF\")
    // Expected: ABCDEF
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_18() {
    // Evaluation of substring function as an argument to a contains function.
    // XPath: fn:contains(fn:substring(\"ABCDEF\",1),\"DEF\")
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_19() {
    // Evaluation of substring function using the special chracter '!@#$%^&*()'.
    // XPath: fn:substring(\"!@#$%^*()\",1)
    // Expected: !@#$%^*()
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_20() {
    // Evaluation of substring function, where the start location uses a 'double' const
    // XPath: fn:substring(\"ABCD\",xs:double(1))
    // Expected: ABCD
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_21() {
    // Evaluation of substring function, where the starting locatin is an addition expr
    // XPath: fn:substring(\"ABCDE\",1+1)
    // Expected: BCDE
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_22() {
    // Evaluation of substring function, where start<0 and (start+len)<0
    // XPath: concat('#', fn:substring(./concepts/@id, string-length(./con
    // Expected: ##
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_23() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"abcd?efgh\", 6)
    // Expected: "efgh"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_24() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"abcd?efgh\", 5, 2)
    // Expected: "?e"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_25() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"?\", 1, 2)
    // Expected: "?"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_26() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"?\", 2, 1)
    // Expected: ""
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_27() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"?\", 0, 2)
    // Expected: "?"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_28() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"?\", 0, 3)
    // Expected: "?"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_29() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"??\", 3)
    // Expected: ""
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_30() {
    // substring() of a string containing non-BMP characters
    // XPath: substring(\"??\", 0)
    // Expected: "??"
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn fn_substring_31() {
    //  start is a double less than zero
    // XPath: substring(\"12345\", -3.1e0, 5.2e0) eq \"1\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_3() {
    //  A test whose essence is: `substring((), 1, 2) eq ''`.
    // XPath: substring((), 1, 2) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_4() {
    //  A test whose essence is: `substring((), 1) eq ''`.
    // XPath: substring((), 1) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_5() {
    //  A test whose essence is: `substring('12345', 1.5, 2.6) eq '234'`.
    // XPath: substring(\"12345\", 1.5, 2.6) eq \"234\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_6() {
    //  A test whose essence is: `substring((), 1, 3) eq ''`.
    // XPath: substring((), 1, 3) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_7() {
    //  A test whose essence is: `substring('motor car', 6) eq ' car'`.
    // XPath: substring(\"motor car\", 6) eq \" car\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_8() {
    //  A test whose essence is: `substring('12345', 0, 3) eq '12'`.
    // XPath: substring(\"12345\", 0, 3) eq \"12\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_9() {
    //  A test whose essence is: `substring('metadata', 4, 3) eq 'ada'`.
    // XPath: substring(\"metadata\", 4, 3) eq \"ada\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_10() {
    //  A test whose essence is: `substring('12345', 0 div 0E0, 3) eq ''`.
    // XPath: substring(\"12345\", 0 div 0E0, 3) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_11() {
    //  A test whose essence is: `substring('12345', 1, 0 div 0E0) eq ''`.
    // XPath: substring(\"12345\", 1, 0 div 0E0) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_12() {
    //  A test whose essence is: `substring('12345', -3, 5) eq '1'`.
    // XPath: substring(\"12345\", -3, 5) eq \"1\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_13() {
    //  A test whose essence is: `substring('12345', -42, 1 div 0E0) eq '12345'`.
    // XPath: substring(\"12345\", -42, 1 div 0E0) eq \"12345\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_14() {
    //  A test whose essence is: `substring('12345', -1 div 0E0, 1 div 0E0) eq ''`.
    // XPath: substring(\"12345\", -1 div 0E0, 1 div 0E0) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn k_substringfunc_15() {
    //  A test whose essence is: `substring('12345', 5, -3) eq ''`.
    // XPath: substring(\"12345\", 5, -3) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn cbcl_substring_001() {
    //  test fn:substring empty string
    // XPath: fn:boolean(fn:substring('', 1, 1))
    // Expected: false
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}

#[test]
fn cbcl_substring_002() {
    //  test fn:substring with index past end of string
    // XPath: fn:boolean(fn:substring('five', 5, 1))
    // Expected: false
    // Calls stub function - will fail until fn:substring is implemented
    let _ = stub_fnsubstring();
}
