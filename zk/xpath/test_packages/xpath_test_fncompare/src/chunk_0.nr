//! Test chunk 0 for fn:compare
//! Contains 50 tests

use dep::xpath::stub_fncompare;

#[test]
fn fn_compare2args_1() {
    // Evaluates The 'compare' function with the arguments set as follows: $comparand1
    // XPath: fn:compare(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare2args_2() {
    // Evaluates The 'compare' function with the arguments set as follows: $comparand1
    // XPath: fn:compare(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare2args_3() {
    // Evaluates The 'compare' function with the arguments set as follows: $comparand1
    // XPath: fn:compare(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare2args_4() {
    // Evaluates The 'compare' function with the arguments set as follows: $comparand1
    // XPath: fn:compare(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare2args_5() {
    // Evaluates The 'compare' function with the arguments set as follows: $comparand1
    // XPath: fn:compare(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_1() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = '' $
    // XPath: fn:compare(\"\",\"\")
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_2() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = '' $
    // XPath: fn:compare(\"\",\"A Character String\")
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_3() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'A C
    // XPath: fn:compare(\"A Character String\",\"\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_4() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = () $
    // XPath: fn:count(fn:compare((),\"\"))
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_5() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = '' $
    // XPath: fn:count(fn:compare(\"\",()))
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_6() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'A C
    // XPath: fn:count(fn:compare(\"A Character String\",()))
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_7() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = () $
    // XPath: fn:count(fn:compare((),\"A Character String\"))
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_8() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'AAA
    // XPath: fn:compare(\"AAAAABBBBBCCCCC\",\"BBBBB\")
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_9() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'AAA
    // XPath: fn:compare(\"AAAAABBBBB\",\" \")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_10() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = ' '
    // XPath: fn:compare(\" \",\"AAAAABBBBB\")
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_11() {
    // Evaluates The 'compare' function using it as a argument of a fn:not - returns tr
    // XPath: fn:not(fn:compare(\"A\",\"A\"))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_12() {
    // Evaluates The 'compare' function using it as a argument of a fn:not - returns fa
    // XPath: fn:not(fn:compare(\"A\",\"B\"))
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_13() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = xs:s
    // XPath: fn:compare(xs:string(\"A\"),\"A\")
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_14() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'A'
    // XPath: fn:compare(\"A\",xs:string(\"A\"))
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_15() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'A'
    // XPath: fn:compare(\"A\",\"a\")
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_16() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'a'
    // XPath: fn:compare(\"a\",\"A\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_17() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'com
    // XPath: fn:compare(\"compare\",\"compare\")
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_18() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'com
    // XPath: fn:compare(\"comparecompare\",\"compare\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_19() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = '***
    // XPath: fn:compare(\"****\",\"***\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_20() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = '123
    // XPath: fn:compare(\"12345\",\"1234\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn fn_compare_21() {
    // Evaluates The 'compare' function with the arguments set as follows: $arg1 = 'com
    // XPath: fn:compare(\"compare\",\"erapmoc\")
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_4() {
    // A test whose essence is: `empty(codepoint-equal((), 'a string'))`.
    // XPath: empty(codepoint-equal((), \"a string\"))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_5() {
    // A test whose essence is: `empty(codepoint-equal('a string', ()))`.
    // XPath: empty(codepoint-equal(\"a string\", ()))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_6() {
    // A test whose essence is: `codepoint-equal('a string', 'a string')`.
    // XPath: codepoint-equal(\"a string\", \"a string\")
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_7() {
    // A test whose essence is: `not(codepoint-equal('cow', 'a string'))`.
    // XPath: not(codepoint-equal(\"cow\", \"a string\"))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_11() {
    // A test whose essence is: `empty(compare((), 'a string'))`.
    // XPath: empty(compare((), \"a string\"))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_12() {
    // A test whose essence is: `empty(compare('a string', ()))`.
    // XPath: empty(compare(\"a string\", ()))
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_13() {
    // A test whose essence is: `empty(compare('a string', (), 'http://www.w3.org/2005/
    // XPath: empty(compare(\"a string\", (), \"http://www.w3.org/2005/xpa
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k_comparefunc_14() {
    // A test whose essence is: `compare('str', 'str') instance of xs:integer`.
    // XPath: compare(\"str\", \"str\") instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_002() {
    // Simple use of compare to compare strings
    // XPath: compare(\"b\", \"a\")
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_003() {
    // Simple use of compare to compare strings
    // XPath: compare(\"b\", \"b\")
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_007() {
    // compare() with non-BMP characters
    // XPath: compare(\"?\", \"?\", \"http://www.w3.org/2005/xpath-functio
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_008() {
    // check that Unicode codepoint collation is used, not comparison of UTF16 surrogat
    // XPath: compare(\"?\", \"?\", \"http://www.w3.org/2005/xpath-functio
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_009() {
    // check that Unicode codepoint collation is used, not comparison of UTF16 surrogat
    // XPath: compare(\"?\", \"?\", \"http://www.w3.org/2005/xpath-functio
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_010() {
    // Use case-blind collation, if supported
    // XPath: compare(\"a\", \"A\", \"http://www.w3.org/2010/09/qt-fots-ca
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_012() {
    // compare() only works on strings, but xs:anyURI gets promoted
    // XPath: compare(xs:anyURI('http://www.example.com/'), 'http://www.ex
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_013() {
    // compare() only works on strings, but xs:untypedAtomic gets promoted
    // XPath: compare(xs:untypedAtomic('http://www.example.com/'), 'http:/
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_014() {
    // Use HTML case-blind collation
    // XPath: compare(\"a\", \"A\", \"http://www.w3.org/2005/xpath-functio
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_015() {
    // Use HTML case-blind collation: test with non-letter characters
    // XPath: compare(\"123 - ; ^ a\", \"123 -  ; ^ a\", \"http://www.w3.o
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_016() {
    // Use HTML case-blind collation: test with non-ASCII letters (actual order is not
    // XPath: compare(\"?\", \"?\", \"http://www.w3.org/2005/xpath-functio
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_017() {
    //  Use compare() with a UCA collation and primary matching
    // XPath: fn:compare(\"database\", \"DATABASE\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_018() {
    //  Use compare() with a UCA collation and primary matching
    // XPath: fn:compare(\"database\", \"D?TABASE\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_019() {
    //  Use compare() with a UCA collation and primary matching
    // XPath: fn:compare(\"database\", \"database\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_020() {
    //  Use compare() with a UCA collation and primary matching
    // XPath: fn:compare(\"database\", \"Databases\", \"http://www.w3.org/
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_021() {
    //  Use compare() with a UCA collation and primary matching
    // XPath: fn:compare(\"databases\", \"Database\", \"http://www.w3.org/
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}
