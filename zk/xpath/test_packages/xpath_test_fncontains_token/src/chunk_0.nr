//! Test chunk 0 for fn:contains-token
//! Contains 44 tests

use dep::xpath::stub_fncontains_token;

#[test]
fn fn_contains_token_09() {
    //  contains-token#2, empty string
    // XPath: contains-token(\"\", \"\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_09_dyn() {
    //  contains-token#2, empty string, dynamic
    // XPath: declare variable $zls external; contains-token($zls, $zls)
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_10() {
    //  contains-token#2, empty sequence
    // XPath: contains-token((), \"\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_10_dyn() {
    //  contains-token#2, empty sequence, dynamic
    // XPath: declare variable $empty external; declare variable $zls exte
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_11() {
    //  contains-token#2, all-space string
    // XPath: contains-token(\"   \", \"   \")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_12() {
    //  contains-token#2, various whitespace chars
    // XPath: contains-token(codepoints-to-string((9, 10, 13, 32, 13, 10,
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_12_dyn() {
    //  contains-token#2, various whitespace chars, dynamic
    // XPath: declare variable $zero external; declare variable $zls exter
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_13() {
    //  contains-token#2, single token
    // XPath: contains-token(\"abc\", \"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_14() {
    //  contains-token#2, single token with surrounding whitespace
    // XPath: contains-token(\" abc \", \"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_15() {
    //  contains-token#2, single token with surrounding whitespace
    // XPath: contains-token(codepoints-to-string((9, 10, 97, 98, 99, 13,
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_16() {
    //  contains-token#2, two tokens
    // XPath: contains-token(\"abc def\", \"def\")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_17() {
    //  contains-token#2, two tokens, multiple whitespace separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 13, 32, 10,
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_18() {
    //  contains-token#2, two tokens, multiple whitespace in separator and surrounding
    // XPath: contains-token(codepoints-to-string((9, 9, 97, 98, 99, 13, 3
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_19() {
    //  contains-token#2, many tokens
    // XPath: contains-token(\" the quick brown fox jumped over the lazy d
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_20() {
    //  contains-token#2, form-feed is not a separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 12, 100, 10
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_21() {
    //  contains-token#2, nbsp is not a separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 160, 100, 1
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_39() {
    //  contains-token#2, empty string
    // XPath: contains-token(\"\", \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_39_dyn() {
    //  contains-token#2, empty string, dynamic
    // XPath: declare variable $zls external;           contains-token($zl
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_40() {
    //  contains-token#2, empty sequence
    // XPath: contains-token((), \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_40_dyn() {
    //  contains-token#2, empty sequence, dynamic
    // XPath: declare variable $empty external;           contains-token($
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_41() {
    //  contains-token#2, all-space string
    // XPath: contains-token(\"   \", \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_42() {
    //  contains-token#2, various whitespace chars
    // XPath: contains-token(codepoints-to-string((9, 10, 13, 32, 13, 10,
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_43() {
    //  contains-token#2, single token
    // XPath: contains-token(\"abc\", \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_44() {
    //  contains-token#2, single token with surrounding whitespace
    // XPath: contains-token(\" abc \", \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_45() {
    //  contains-token#2, single token with surrounding whitespace
    // XPath: contains-token(codepoints-to-string((9, 10, 97, 98, 99, 13,
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_46() {
    //  contains-token#2, two tokens
    // XPath: contains-token(\"abc def\", \"zz\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_47() {
    //  contains-token#2, two tokens, multiple whitespace separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 13, 32, 10,
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_48() {
    //  contains-token#2, two tokens, multiple whitespace in separator and surrounding
    // XPath: contains-token(codepoints-to-string((9, 9, 97, 98, 99, 13, 3
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_49() {
    //  contains-token#2, many tokens
    // XPath: contains-token(\" the quick brown fox jumped over the lazy d
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_50() {
    //  contains-token#2, form-feed is not a separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 12, 100, 10
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_51() {
    //  contains-token#2, nbsp is not a separator
    // XPath: contains-token(codepoints-to-string((97, 98, 99, 160, 100, 1
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_52() {
    //  contains-token#2, dynamic call
    // XPath: (contains-token#2, substring-before#2, substring-after#2)[1]
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_60() {
    //  contains-token#2, arg0 is sequence
    // XPath: contains-token((\"abc\", \"def\"), \"\")
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_61() {
    //  contains-token#2, arg0 is sequence, all-space string
    // XPath: contains-token((\"abc\", \"def\"), \" abc  \")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_62() {
    //  contains-token#2, arg0 is sequence, various whitespace chars
    // XPath: contains-token((\"abc\", \"def\"), codepoints-to-string((9,
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_63() {
    //  contains-token#2, arg0 is sequence, multiple token
    // XPath: contains-token((\"abc def\", \"ghi\"), \"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_64() {
    //  contains-token#2, arg0 is sequence, multiple token with surrounding whitespace
    // XPath: contains-token((\" abc def \", \"ghi\"), \"def\")
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_65() {
    //  contains-token#2, arg0 is sequence, single token with surrounding whitespace
    // XPath: contains-token(codepoints-to-string((9, 10, 97, 98, 99, 13,
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_70() {
    //  contains-token#2, HTML5 collation
    // XPath: contains-token(\"the quick brown fox\", \"Fox\", \"http://ww
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_71() {
    //  contains-token#2, HTML5 collation
    // XPath: contains-token(\"the quick brown fox\", \" QUICK \", \"http:
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_72() {
    //  contains-token#2, HTML5 collation
    // XPath: contains-token(\"the quick brown fox\", \"qui?k\", \"http://
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_72_dyn() {
    //  contains-token#2, HTML5 collation, dynamic collation
    // XPath: declare variable $zls external;           contains-token(\"t
    // Expected: true
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_73() {
    //  contains-token#2, empty string, HTML5 collation
    // XPath: contains-token(\"\", \"quick\", \"http://www.w3.org/2005/xpa
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}

#[test]
fn fn_contains_token_73_dyn() {
    //  contains-token#2, empty string, HTML5 collation, dynamic input
    // XPath: declare variable $zls external;           contains-token($zl
    // Expected: false
    // Calls stub function - will fail until fn:contains-token is implemented
    let _ = stub_fncontains_token();
}
