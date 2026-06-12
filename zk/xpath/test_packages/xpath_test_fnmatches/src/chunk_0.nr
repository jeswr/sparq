//! Test chunk 0 for fn:matches
//! Contains 50 tests

use dep::xpath::stub_fnmatches;

#[test]
fn fn_matches2args_1() {
    //  Evaluates The 'matches' function with the arguments set as follows: $input = xs
    // XPath: fn:matches(\"This is a characte\",\"This is a characte\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches2args_2() {
    //  Evaluates The 'matches' function with the arguments set as follows: $input = xs
    // XPath: fn:matches(\"This is a characte\",\"This is a characte\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches2args_3() {
    //  Evaluates The 'matches' function with the arguments set as follows: $input = xs
    // XPath: fn:matches(\"This is a characte\",\"This is a characte\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches2args_4() {
    //  Evaluates The 'matches' function with the arguments set as follows: $input = xs
    // XPath: fn:matches(\"This is a characte\",\"This is a characte\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches2args_5() {
    //  Evaluates The 'matches' function with the arguments set as follows: $input = xs
    // XPath: fn:matches(\"This is a characte\",\"This is a characte\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_1() {
    //  Evaluation of matches function as per example 1 (for this function)
    // XPath: fn:matches(\"abracadabra\", \"bra\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_2() {
    //  Evaluation of matches function as per example 2 (for this function). Pattern se
    // XPath: fn:matches(\"abracadabra\", \"^a.*a$\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_3() {
    //  Evaluation of matches function as per example 3 (for this function). Pattern se
    // XPath: fn:matches(\"abracadabra\", \"^bra\")
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_4() {
    //  Test that calling the function with flags set to the empty string is the same a
    // XPath: fn:concat(fn:matches(\"abracadabra\", \"^bra\"),fn:matches(\
    // Expected: falsefalse
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_5() {
    //  Evaluate the fn:mathes function with the input string set to the empty sequence
    // XPath: fn:count(fn:matches(\"()\", \"^bra\"))
    // Expected: 1
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_6() {
    //  Evaluation of matches function with pattern set to '\^'.
    // XPath: fn:matches(\"abracadabra^abracadabra\", \"\^\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_7() {
    //  Evaluation of matches function with pattern set to '\?' for an input string tha
    // XPath: fn:matches(\"abracadabra?abracadabra\", \"\?\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_8() {
    //  Evaluation of matches function with pattern set to '\*' for an input string tha
    // XPath: fn:matches(\"abracadabra*abracadabra\", \"\*\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_9() {
    //  Evaluation of matches function with pattern set to '\+' for an input string tha
    // XPath: fn:matches(\"abracadabra+abracadabra\", \"\+\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_10() {
    //  Evaluation of matches function with pattern set to '\{' for an input string tha
    // XPath: fn:matches(\"abracadabra{abracadabra\", \"\{\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_11() {
    //  Evaluation of matches function with pattern set to '\}' for an input string tha
    // XPath: fn:matches(\"abracadabra}abracadabra\", \"\}\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_12() {
    //  Evaluation of matches function with pattern set to '\(' for an input string tha
    // XPath: fn:matches(\"abracadabra(abracadabra\", \"\(\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_13() {
    //  Evaluation of matches function with pattern set to '\)' for an input string tha
    // XPath: fn:matches(\"abracadabra)abracadabra\", \"\)\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_14() {
    //  Evaluation of matches function with pattern set to '\[' for an input string tha
    // XPath: fn:matches(\"abracadabra[abracadabra\", \"\[\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_15() {
    //  Evaluation of matches function with pattern set to '\]' for an input string tha
    // XPath: fn:matches(\"abracadabra]abracadabra\", \"\]\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_16() {
    //  Evaluation of matches function with pattern set to '\-' for an input string tha
    // XPath: fn:matches(\"abracadabra-abracadabra\", \"\-\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_17() {
    //  Evaluation of matches function with pattern set to '\.' for an input string tha
    // XPath: fn:matches(\"abracadabra.abracadabra\", \"\.\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_18() {
    //  Evaluation of matches function with pattern set to '\|' for an input string tha
    // XPath: fn:matches(\"abracadabra|abracadabra\", \"\|\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_19() {
    //  Evaluation of matches function with pattern set to '\\' for an input string tha
    // XPath: fn:matches(\"abracadabra\abracadabra\", \"\\\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_20() {
    //  Evaluation of matches function with pattern set to '\t' for an input string tha
    // XPath: fn:matches(\"abracadabra abracadabra\", \"\t\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_21() {
    //  Evaluation of matches function with pattern set to '\n' for an input string tha
    // XPath: fn:matches(\"abracadabra abracadabra\", \"\n\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_22() {
    //  Evaluation of matches function with pattern set to 'aa{1}' (exact quantity) for
    // XPath: fn:matches(\"abracadabraabracadabra\", \"aa{1}\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_23() {
    //  Evaluation of matches function with pattern set to 'aa{1,}' (min quantity) for
    // XPath: fn:matches(\"abracadabraabracadabraabracadabra\", \"aa{1,}\"
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_24() {
    //  Evaluation of matches function with pattern set to 'aa{1,2}' (range quantity) f
    // XPath: fn:matches(\"abracadabraabracadabraabracadabra\", \"aa{1,2}\
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_26() {
    //  Check for the correct behavior of ^ and $ in multi-line mode This test case was
    // XPath: fn:matches(concat('abcd', codepoints-to-string(10), 'defg',
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_27() {
    //  Check for the correct behavior of ^ and $ in multi-line mode This test case was
    // XPath: fn:matches(\" abcd defg \", \"^$\", \"m\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_28() {
    //  Check for the correct behavior of ^ and $ in multi-line mode This test case was
    // XPath: fn:matches(\"abcd  defg \", \"^$\", \"m\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_29() {
    //  2-digits not treated as a back-reference See erratum FO.E24
    // XPath: fn:matches(\"#abc#1\", \"^(#)abc\11$\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_30() {
    //  2-digits treated as a back-reference See erratum FO.E24
    // XPath: fn:matches(\"#abcdefghijklmnopq#1\", \"^(#)(a)(b)(c)(d)(e)(f
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_31() {
    //  Evaluation of matches function with non-capturing groups (allowed in XPath 3.0)
    // XPath: fn:matches(\"abracadabra\", \"(?:abra(?:cad)?)*\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_32() {
    //  Evaluation of matches function with 'q' flag (allowed in XQuery 3.0)
    // XPath: fn:matches(\"abracadabra\", \"(?:abra(?:cad)?)*\", \"q\")
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_33() {
    //  Evaluation of matches function with 'q' flag (allowed in XQuery 3.0)
    // XPath: fn:matches(\"x[y-z]\", \"x[y-z]\", \"q\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_34() {
    //  Evaluation of matches function with 'q' and 'i' flags (allowed in XQuery 3.0)
    // XPath: fn:matches(\"x[Y-z]\", \"X[y-Z]\", \"qi\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_36() {
    //  Test for bug fix of 5348 in Errata for F+O. ok match here
    // XPath: fn:matches('abcdefghijj', '(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)\10
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_41() {
    //  Handling of final newline with non-multiline mode
    // XPath: fn:matches(concat('Mary', codepoints-to-string(10)), 'Mary$'
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_42() {
    //  Handling of final newline with $ in dot-all mode
    // XPath: fn:matches(concat('Mary', codepoints-to-string(10)), 'Mary$'
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_43() {
    //  '.' doesn't normally match newline
    // XPath: fn:matches(concat('Mary', codepoints-to-string(10), 'Jones')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_44() {
    //  '.' does match newline in dot-all mode
    // XPath: fn:matches(concat('Mary', codepoints-to-string(10), 'Jones')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_45() {
    //  '.' does NOT match CR in default mode
    // XPath: fn:matches(concat('Mary', codepoints-to-string(13), 'Jones')
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_46() {
    //  '.' does match CR in dot-all mode
    // XPath: fn:matches(concat('Mary', codepoints-to-string(13), 'Jones')
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_47() {
    //  Check for the correct behavior of $ when not in multi-line mode.          The c
    // XPath: fn:matches(concat('abcd', codepoints-to-string(10), 'defg',
    // Expected: false
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_48() {
    //  Edge condition: match occurs at last character.
    // XPath: fn:matches(\"abracadabra-abracadabra.\", \"\.\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_49() {
    //  Edge condition: match occurs at last character.
    // XPath: fn:matches(\"abracadabra-abracadabra-3\", \"(124|864|377|3)\
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_51() {
    //  Unescaped left parens inside a charClass are allowed and don't affect the subex
    // XPath: fn:matches(\"ab()cd()ef()gh\", \"^(ab)([()]*)(cd)([)(]*)ef\4
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}

#[test]
fn fn_matches_52() {
    //  A use case involving backtracking and ambiguity
    // XPath: fn:matches(\"aaababaaabaa\", \"^(a*b?a*){3,3}$\")
    // Expected: true
    // Calls stub function - will fail until fn:matches is implemented
    let _ = stub_fnmatches();
}
