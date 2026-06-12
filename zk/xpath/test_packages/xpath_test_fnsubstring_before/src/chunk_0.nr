//! Test chunk 0 for fn:substring-before
//! Contains 30 tests

use dep::xpath::stub_fnsubstring_before;

#[test]
fn fn_substring_before_8() {
    // fn-substring-before-8 Evaluates The 'substring-before' function with the argumen
    // XPath: fn:substring-before(\"AAAAABBBBBCCCCC\",\"BBBBB\")
    // Expected: AAAAA
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_11() {
    // fn-substring-before-11 Evaluates The 'substring-before' function using it as a a
    // XPath: fn:not(fn:substring-before(\"A\",\"A\"))
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_12() {
    // fn-substring-before-12 Evaluates The 'substring-before' function using it as a a
    // XPath: fn:not(fn:substring-before(\"A\",\"B\"))
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_22() {
    // Use absolute collation URI
    // XPath: substring-before(\"banana\", \"a\",          \"http://www.w3
    // Expected: "b"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_23() {
    // Use relative collation URI
    // XPath: substring-before(\"banana\", \"a\", \"collation/codepoint\")
    // Expected: "b"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_24() {
    // Use case-blind collation (if supported)
    // XPath: substring-before(\"banana\", \"A\",          \"http://www.w3
    // Expected: "b"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_25() {
    // Use non-BMP characters
    // XPath: substring-before(\"???\", \"?\")
    // Expected: "?"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_26() {
    // Use case-blind collation
    // XPath: substring-before(\"banana\", \"A\",          \"http://www.w3
    // Expected: "b"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_27() {
    //  Use substring-before() with a UCA collation
    // XPath: fn:substring-before(\"banana\", \"ana\", \"http://www.w3.org
    // Expected: "b"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_28() {
    //  Use substring-before() with a UCA collation
    // XPath: fn:substring-before(\"banana\", \"ananas\", \"http://www.w3.
    // Expected: ""
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_29() {
    //  Use substring-before() with a UCA collation and primary matching
    // XPath: fn:substring-before(\"database\", \"BASE\", \"http://www.w3.
    // Expected: "data"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_30() {
    //  Use substring-before() with a UCA collation and primary matching
    // XPath: fn:substring-before(\"database\", \"tabaS\", \"http://www.w3
    // Expected: "da"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_31() {
    //  Use substring-before() with a UCA collation and primary matching
    // XPath: fn:substring-before(\"database\", \"?ta\", \"http://www.w3.o
    // Expected: "d"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_32() {
    //  Use substring-before() with a UCA collation and primary matching
    // XPath: fn:substring-before(\"d?tabase\", \"?tabase\", \"http://www.
    // Expected: "d"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_33() {
    //  Use substring-before() with a UCA collation and primary matching
    // XPath: fn:substring-before(\"d?t?base\", \"?tab?se\", \"http://www.
    // Expected: "d"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_34() {
    //  Use substring-before() with a UCA collation and secondary matching
    // XPath: fn:substring-before(\"database\", \"BASE\", \"http://www.w3.
    // Expected: "data"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_35() {
    //  Use substring-before() with a UCA collation and secondary matching
    // XPath: fn:substring-before(\"database\", \"tabaS\", \"http://www.w3
    // Expected: "da"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_36() {
    //  Use substring-before() with a UCA collation and secondary matching
    // XPath: fn:substring-before(\"database\", \"b?s\", \"http://www.w3.o
    // Expected: ""
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_37() {
    //  Use substring-before() with a UCA collation and secondary matching
    // XPath: fn:substring-before(\"d?tabase\", \"Base\", \"http://www.w3.
    // Expected: "d?ta"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_38() {
    //  Use substring-before() with a UCA collation and tertiary matching
    // XPath: fn:substring-before(\"database\", \"BASE\", \"http://www.w3.
    // Expected: ""
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_39() {
    //  Use substring-before() with a UCA collation and tertiary matching
    // XPath: fn:substring-before(\"dataBase\", \"taBas\", \"http://www.w3
    // Expected: "da"
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_40() {
    //  Use substring-before() with a UCA collation and tertiary matching
    // XPath: fn:substring-before(\"database\", \"b?se\", \"http://www.w3.
    // Expected: ""
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn fn_substring_before_41() {
    //  Use substring-before() with a UCA collation and tertiary matching
    // XPath: fn:substring-before(\"database\", \"Base\", \"http://www.w3.
    // Expected: ""
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_5() {
    // K-SubstringBeforeFunc-5  A test whose essence is: `substring-before('foo', 'oo',
    // XPath: substring-before(\"foo\", \"oo\",          \"http://www.w3.o
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_6() {
    // K-SubstringBeforeFunc-6  A test whose essence is: `substring-before('tattoo', 'a
    // XPath: substring-before(\"tattoo\", \"attoo\") eq \"t\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_7() {
    // K-SubstringBeforeFunc-7  A test whose essence is: `substring-before('tattoo', 't
    // XPath: substring-before(\"tattoo\", \"tatto\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_8() {
    // K-SubstringBeforeFunc-8  A test whose essence is: `substring-before((), ()) eq '
    // XPath: substring-before((), ()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_9() {
    // K-SubstringBeforeFunc-9  A test whose essence is: `substring-before('a string',
    // XPath: substring-before(\"a string\", \"\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn k_substringbeforefunc_10() {
    // K-SubstringBeforeFunc-10  A test whose essence is: `substring-before('a string',
    // XPath: substring-before(\"a string\", \"not in other\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}

#[test]
fn cbcl_substring_before_001() {
    //  test fn:substring-before with collation and empty string
    // XPath: fn:boolean(fn:substring-before('input', '', 'http://www.w3.o
    // Expected: false
    // Calls stub function - will fail until fn:substring-before is implemented
    let _ = stub_fnsubstring_before();
}
