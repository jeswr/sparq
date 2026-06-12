//! Test chunk 1 for fn:matches
//! Contains 50 tests

use dep::xpath::stub_fnmatches;

#[test]
fn fn_matches_53() {
    //  A use case involving repetition of a back-reference. Saxon bug 3712.
    // XPath: fn:matches(\"A\", \"([A-Z])\1*\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_54() {
    //  A use case involving optional matching of start-of-string. Saxon bug 3782.
    // XPath: fn:matches(\"kZ\", \"(^|:)?Z\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch01() {
    //  Simple call of matches() with 'i' flag
    // XPath: matches('abc', 'ABC', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch02() {
    //  Call of matches() with 'i' flag and a character range
    // XPath: matches('abZ', '[A-Z]*', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch03() {
    //  Call of matches() with 'i' flag and a character range
    // XPath: matches('abZ', '[a-z]*', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch04() {
    //  Call of matches() with 'i' flag and Kelvin sign Kelvin sign
    // XPath: matches(codepoints-to-string(8490), '[A-Z]', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch05() {
    //  Call of matches() with 'i' flag and Kelvin sign Kelvin sign
    // XPath: matches(codepoints-to-string(8490), '[a-z]', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch06() {
    //  Call of matches() with 'i' flag and Kelvin sign Kelvin sign
    // XPath: matches(codepoints-to-string(8490), 'K', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch07() {
    //  Call of matches() with 'i' flag and Kelvin sign Kelvin sign
    // XPath: matches(codepoints-to-string(8490), 'k', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch08() {
    //  Call of matches() with 'i' flag and range subtraction
    // XPath: matches('x', '[A-Z-[OI]]', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch09() {
    //  Call of matches() with 'i' flag and range subtraction
    // XPath: matches('X', '[A-Z-[OI]]', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch10() {
    //  Call of matches() with 'i' flag and range subtraction
    // XPath: matches('O', '[A-Z-[OI]]', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch11() {
    //  Call of matches() with 'i' flag and range subtraction
    // XPath: matches('i', '[A-Z-[OI]]', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch12() {
    //  Call of matches() with 'i' flag and negation
    // XPath: matches('Q', '[^Q]', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch13() {
    //  Call of matches() with 'i' flag and negation
    // XPath: matches('q', '[^Q]', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch14() {
    //  Call of matches() with 'i' flag and upper-case category
    // XPath: matches('m', '\p{Lu}', 'i')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn caselessmatch15() {
    //  Call of matches() with 'i' flag and upper-case category
    // XPath: matches('m', '\P{Lu}', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_1() {
    //  Whitespace in the regexp is collapsed.
    // XPath: fn:matches(\"hello world\", \"hello\ sworld\", \"x\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_2() {
    //  Whitespace(before) in the regexp is collapsed, but not inside a character class
    // XPath: fn:matches(\"hello world\", \" hello[ ]world\", \"x\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_3() {
    //  Whitespace(after) in the regexp is collapsed, but not inside a character class.
    // XPath: fn:matches(\"hello world\", \"hello[ ]world \", \"x\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_4() {
    //  Whitespace(in the middle) in the regexp is collapsed, but not inside a characte
    // XPath: fn:matches(\"hello world\", \"he ll o[ ]worl d\", \"x\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_5() {
    //  whitespace in the regexp is collapsed, and should therefore compile.
    // XPath: fn:matches(\"hello world\", \"\p{ IsBasicLatin}+\", \"x\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_6() {
    //  whitespace in the regexp is collapsed completely, and should therefore compile
    // XPath: fn:matches(\"hello world\", \"\p{ I s B a s i c L a t i n }+
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_16() {
    //  Use a pattern whose interpretation is unknown. See public report 4466 and 21425
    // XPath: fn:matches(\"input\", \"[0-9-.]*/\")
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn k2_matchesfunc_17() {
    //  Caseless match with back-reference.
    // XPath: matches('aA', '(a)\1', 'i')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_003() {
    //  test a two-digit back reference
    // XPath: fn:matches('abcdefghijkabcdefghijk', '(a)(b)(c)(d)(e)(f)(g)(
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_006() {
    //  test with a character class containing an escaped character
    // XPath: fn:matches('&#x9;', '[\t]')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_006b() {
    //  test with a character class containing an escaped character
    // XPath: fn:matches(' ', '[\t]')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_007() {
    //  test with a character class beginning with a '-'
    // XPath: fn:matches('-abba-', '[-ab]+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_015() {
    //  test category name L
    // XPath: fn:matches('foo', '\P{L}')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_016() {
    //  test category name M
    // XPath: fn:matches('foo', '\P{M}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_017() {
    //  test category name N
    // XPath: fn:matches('foo', '\P{N}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_018() {
    //  test category name P
    // XPath: fn:matches('foo', '\P{P}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_019() {
    //  test category name Z
    // XPath: fn:matches('foo', '\P{Z}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_020() {
    //  test category name S
    // XPath: fn:matches('foo', '\P{S}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_021() {
    //  test category name C
    // XPath: fn:matches('foo', '\P{C}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_022() {
    //  test category name Lu
    // XPath: fn:matches('foo', '\P{Lu}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_023() {
    //  test category name Me
    // XPath: fn:matches('foo', '\P{Me}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_024() {
    //  test category name No
    // XPath: fn:matches('foo', '\P{No}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_025() {
    //  test category name Pf
    // XPath: fn:matches('foo', '\P{Pf}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_026() {
    //  test category name Zs
    // XPath: fn:matches('foo', '\P{Zs}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_027() {
    //  test category name Sk
    // XPath: fn:matches('foo', '\P{Sk}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_028() {
    //  test category name Cc
    // XPath: fn:matches('foo', '\P{Cc}')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_036() {
    //  test an empty branch
    // XPath: fn:matches('foo', 'a()b')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_037() {
    //  test a multibyte Unicode character
    // XPath: fn:matches('&#x10000;', '&#x10000;')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_038() {
    //  test a large exact quantifier
    // XPath: fn:matches('aaa', 'a{2147483647}')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_039() {
    //  test a two-digit back reference
    // XPath: fn:matches('abcdefghiabcdefghia0a1', '(a)(b)(c)(d)(e)(f)(g)(
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_040() {
    //  test the multi-character escape \S
    // XPath: fn:matches('abc', '\S+')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_041() {
    //  test the multi-character escape \S
    // XPath: fn:matches('&#xD;&#x20;&#x9;', '\S+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn cbcl_matches_041b() {
    //  test the multi-character escape \S
    // XPath: fn:matches('   ', '\S+')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}
