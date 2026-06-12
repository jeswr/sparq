//! Test chunk 1 for fn:compare
//! Contains 33 tests

use dep::xpath::stub_fncompare;

#[test]
fn compare_022() {
    //  Use compare() with a UCA collation and secondary matching
    // XPath: fn:compare(\"database\", \"DATABASE\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_023() {
    //  Use compare() with a UCA collation and secondary matching
    // XPath: fn:compare(\"database\", \"D?TABASE\", \"http://www.w3.org/2
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_024() {
    //  Use compare() with a UCA collation and secondary matching
    // XPath: fn:compare(\"database\", \"database\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_025() {
    //  Use compare() with a UCA collation and secondary matching
    // XPath: fn:compare(\"database\", \"Databases\", \"http://www.w3.org/
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_026() {
    //  Use compare() with a UCA collation and secondary matching
    // XPath: fn:compare(\"databases\", \"Database\", \"http://www.w3.org/
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_027() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"database\", \"d?tabase\", \"http://www.w3.org/2
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_028() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"database\", \"DATABASE\", \"http://www.w3.org/2
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_029() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"database\", \"database\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_030() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"database\", \"Database\", \"http://www.w3.org/2
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_031() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"database\", \"Database\", \"http://www.w3.org/2
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_032() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"databases\", \"Database\", \"http://www.w3.org/
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_033() {
    //  Use compare() with a UCA collation and tertiary matching
    // XPath: fn:compare(\"d?tabases\", \"Database\", \"http://www.w3.org/
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_034() {
    //  Use compare() with a UCA collation with numeric matching
    // XPath: fn:compare(\"Chap2\", \"Chap10\", \"http://www.w3.org/2013/c
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_035() {
    //  Use compare() with a UCA collation with numeric matching
    // XPath: fn:compare(\"Chap2-b\", \"Chap10-b\", \"http://www.w3.org/20
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_036() {
    //  Use compare() with a UCA collation : ignore accents but not case
    // XPath: fn:compare(\"DATABASE\", \"D?TABASE\", \"http://www.w3.org/2
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_037() {
    //  Use compare() with a UCA collation : ignore accents but not case
    // XPath: fn:compare(\"DATABASE\", \"D?TAbase\", \"http://www.w3.org/2
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_038() {
    //  Use compare() with a UCA collation : first accent is the most significant
    // XPath: fn:compare(\"DATAB?SE\", \"D?TABASE\", \"http://www.w3.org/2
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_039() {
    //  Use compare() with a UCA collation : last accent is the most significant
    // XPath: fn:compare(\"DATAB?SE\", \"D?TABASE\", \"http://www.w3.org/2
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_040() {
    //  Use compare() with a UCA collation : alternate=blanked : spaces insignificant a
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_041() {
    //  Use compare() with a UCA collation : alternate=blanked : spaces insignificant a
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_042() {
    //            Use compare() with a UCA collation : alternate=blanked : spaces signi
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_043() {
    //  Use compare() with a UCA collation : alternate=shifted : spaces insignificant a
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: true
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_044() {
    //  Use compare() with a UCA collation : alternate=shifted : spaces significant at
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn compare_045() {
    //  Use compare() with a UCA collation : alternate=shifted : spaces significant at
    // XPath: fn:compare(\"database\", \"data base\", \"http://www.w3.org/
    // Expected: false
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_1() {
    // Use a complex collation argument.
    // XPath: compare(\"a\", \"a\", (\"http://www.w3.org/2005/xpath-functi
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_2() {
    // Use a complex collation argument.
    // XPath: compare(\"a\", \"a\", ((), \"http://www.w3.org/2005/xpath-fu
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_3() {
    //  Use a complex collation argument.
    // XPath: compare(\"a\", \"a\", ((), \"http://www.w3.org/2005/xpath-fu
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_4() {
    //  Compare two values returned from fn:lower-case().
    // XPath: let $vA := (\"B STRING\", current-time())[1] treat as xs:str
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_5() {
    //  Compare two values returned from fn:upper-case().
    // XPath: let $vA := (\"B STRING\", current-time())[1] treat as xs:str
    // Expected: 0
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_6() {
    //  Compare two non-matching values returned from fn:lower-case().
    // XPath: let $vA  := (\"B STRING\", current-time())[1] treat as xs:st
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_7() {
    //  Compare two non-matching values returned from fn:upper-case().
    // XPath: let $vA  := (\"B STRING\", current-time())[1] treat as xs:st
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_8() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: let $vA  := (\"B STRING\", current-time())[1] treat as xs:st
    // Expected: -1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}

#[test]
fn k2_comparefunc_9() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: let $vA  := (\"B STRING\", current-time())[1] treat as xs:st
    // Expected: 1
    // Calls stub function - will fail until fn:compare is implemented
    let _ = stub_fncompare();
}
