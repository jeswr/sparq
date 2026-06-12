//! Test chunk 0 for fn:substring-after
//! Contains 34 tests

use dep::xpath::stub_fnsubstring_after;

#[test]
fn fn_substring_after_8() {
    //  Evaluates The 'substring-after' function with the arguments set as follows: $ar
    // XPath: fn:substring-after(\"AAAAABBBBBCCCCC\",\"BBBBB\")
    // Expected: CCCCC
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_11() {
    //  Evaluates The 'substring-after' function using it as a argument of a fn:not - r
    // XPath: fn:not(fn:substring-after(\"A\",\"A\"))
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_12() {
    //  Evaluates The 'substring-after' function using it as a argument of a fn:not - r
    // XPath: fn:not(fn:substring-after(\"A\",\"B\"))
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_18() {
    //  Evaluates The 'substring-after' function with the arguments set as follows: $ar
    // XPath: fn:substring-after(\"substring-aftersubstring-after\",\"subs
    // Expected: substring-after
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_19() {
    //  Evaluates The 'substring-after' function with the arguments set as follows: $ar
    // XPath: fn:substring-after(\"****\",\"***\")
    // Expected: *
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_20() {
    //  Evaluates The 'substring-after' function with the arguments set as follows: $ar
    // XPath: fn:substring-after(\"12345\",\"1234\")
    // Expected: 5
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_22() {
    // Use absolute collation URI
    // XPath: substring-after(\"banana\", \"a\", \"http://www.w3.org/2005/
    // Expected: "nana"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_23() {
    // Use relative collation URI
    // XPath: substring-after(\"banana\", \"a\", \"collation/codepoint\")
    // Expected: "nana"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_24() {
    // Use case-blind collation (if supported)
    // XPath: substring-after(\"banana\", \"A\", \"http://www.w3.org/2010/
    // Expected: "nana"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_25() {
    // Use non-BMP characters
    // XPath: substring-after(\"???\", \"?\")
    // Expected: "?"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_26() {
    // Use HTML case-blind collation
    // XPath: substring-after(\"banana\", \"A\", \"http://www.w3.org/2005/
    // Expected: "nana"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_27() {
    //  Use substring-after() with a UCA collation
    // XPath: fn:substring-after(\"banana\", \"ana\", \"http://www.w3.org/
    // Expected: "na"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_28() {
    //  Use substring-after() with a UCA collation
    // XPath: fn:substring-after(\"banana\", \"ananas\", \"http://www.w3.o
    // Expected: ""
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_29() {
    //  Use substring-after() with a UCA collation and primary matching
    // XPath: fn:substring-after(\"database\", \"DATA\", \"http://www.w3.o
    // Expected: "base"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_30() {
    //  Use substring-after() with a UCA collation and primary matching
    // XPath: fn:substring-after(\"database\", \"tabaS\", \"http://www.w3.
    // Expected: "e"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_31() {
    //  Use substring-after() with a UCA collation and primary matching
    // XPath: fn:substring-after(\"database\", \"?ta\", \"http://www.w3.or
    // Expected: "base"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_32() {
    //  Use substring-after() with a UCA collation and primary matching
    // XPath: fn:substring-after(\"d?tabase\", \"d?tab\", \"http://www.w3.
    // Expected: "ase"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_33() {
    //  Use substring-after() with a UCA collation and primary matching
    // XPath: fn:substring-after(\"d?t?base\", \"?tab?\", \"http://www.w3.
    // Expected: "se"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_34() {
    //  Use substring-after() with a UCA collation and secondary matching
    // XPath: fn:substring-after(\"database\", \"DATA\", \"http://www.w3.o
    // Expected: "base"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_35() {
    //  Use substring-after() with a UCA collation and secondary matching
    // XPath: fn:substring-after(\"database\", \"tabaS\", \"http://www.w3.
    // Expected: "e"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_36() {
    //  Use substring-after() with a UCA collation and secondary matching
    // XPath: fn:substring-after(\"database\", \"b?s\", \"http://www.w3.or
    // Expected: ""
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_37() {
    //  Use substring-after() with a UCA collation and secondary matching
    // XPath: fn:substring-after(\"d?tabase\", \"?\", \"http://www.w3.org/
    // Expected: "tabase"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_38() {
    //  Use substring-after() with a UCA collation and tertiary matching
    // XPath: fn:substring-after(\"database\", \"DATA\", \"http://www.w3.o
    // Expected: ""
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_39() {
    //  Use substring-after() with a UCA collation and tertiary matching
    // XPath: fn:substring-after(\"dataBase\", \"taBas\", \"http://www.w3.
    // Expected: "e"
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_40() {
    //  Use substring-after() with a UCA collation and tertiary matching
    // XPath: fn:substring-after(\"d?tabase\", \"data\", \"http://www.w3.o
    // Expected: ""
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn fn_substring_after_41() {
    //  Use substring-after() with a UCA collation and tertiary matching
    // XPath: fn:substring-after(\"database\", \"Data\", \"http://www.w3.o
    // Expected: ""
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_5() {
    //  A test whose essence is: `substring-after('foo', 'fo', 'http://www.w3.org/2005/
    // XPath: substring-after(\"foo\", \"fo\", \"http://www.w3.org/2005/xp
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_6() {
    //  A test whose essence is: `substring-after('tattoo', 'tat') eq 'too'`.
    // XPath: substring-after(\"tattoo\", \"tat\") eq \"too\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_7() {
    //  A test whose essence is: `substring-after('tattoo', 'tattoo') eq ''`.
    // XPath: substring-after(\"tattoo\", \"tattoo\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_8() {
    //  A test whose essence is: `substring-after('abcdefgedij', 'def') eq 'gedij'`.
    // XPath: substring-after(\"abcdefgedij\", \"def\") eq \"gedij\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_9() {
    //  A test whose essence is: `substring-after((), ()) eq ''`.
    // XPath: substring-after((), ()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_10() {
    //  A test whose essence is: `substring-after('a string', ()) eq 'a string'`.
    // XPath: substring-after(\"a string\", ()) eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn k_substringafterfunc_11() {
    //  A test whose essence is: `substring-after('a string', 'not in other') eq ''`.
    // XPath: substring-after(\"a string\", \"not in other\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}

#[test]
fn cbcl_substring_after_001() {
    //  test fn:substring-after with collation and empty string
    // XPath: fn:boolean(fn:substring-after('input', '', 'http://www.w3.or
    // Expected: true
    // Calls stub function - will fail until fn:substring-after is implemented
    let _ = stub_fnsubstring_after();
}
