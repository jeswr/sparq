//! Test chunk 0 for fn:tokenize
//! Contains 22 tests

use dep::xpath::stub_fntokenize;

#[test]
fn fn_tokenize_3() {
    //  Evaluation of tokenize function with pattern set to '\s+' as per example 1 for
    // XPath: fn:tokenize(\"The cat sat on the mat\", \"\s+\")
    // Expected: The cat sat on the mat
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_4() {
    //  Evaluation of tokenize function with pattern set to '\s*' as per example 2 for
    // XPath: fn:tokenize(\"1, 15, 24, 50\", \",\s*\")
    // Expected: 1 15 24 50
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_7() {
    //  Evaluation of tokenize function with $input set to empty sequence Uses fn:count
    // XPath: fn:count(fn:tokenize((), \"\s+\"))
    // Expected: 0
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_8() {
    //  Evaluation of tokenize function with $input set to zero length string. Uses fn:
    // XPath: fn:count(fn:tokenize(\"\", \"\s+\"))
    // Expected: 0
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_9() {
    //  Evaluation of tokenize function with two patterms matching the input string.
    // XPath: string-join(fn:tokenize(\"abracadabra\", \"(ab)|(a)\"), '#')
    // Expected: #r#c#d#r#
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_10() {
    //  Evaluation of tokenize function with pattern that does not match the input stri
    // XPath: fn:tokenize(\"abracadabra\", \"ww\")
    // Expected: abracadabra
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_34() {
    //  '.' does NOT match CR in default mode
    // XPath: fn:tokenize(concat('Mary', codepoints-to-string(13), 'Jones'
    // Expected: concat('Mary', codepoints-to-string(13),
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_43() {
    //  tokenize#1, single token
    // XPath: tokenize(\"abc\")
    // Expected: "abc"
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_44() {
    //  tokenize#1, single token with surrounding whitespace
    // XPath: tokenize(\" abc \")
    // Expected: "abc"
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_45() {
    //  tokenize#1, single token with surrounding whitespace
    // XPath: tokenize(codepoints-to-string((9, 10, 97, 98, 99, 13, 32)))
    // Expected: "abc"
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_49() {
    //  tokenize#1, many tokens
    // XPath: string-join(tokenize(\" the quick brown fox jumped over the
    // Expected: "the|quick|brown|fox|jumped|over|the|laz
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_50() {
    //  tokenize#1, form-feed is not a separator
    // XPath: count(tokenize(codepoints-to-string((97, 98, 99, 12, 100, 10
    // Expected: 1
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn fn_tokenize_51() {
    //  tokenize#1, nbsp is not a separator
    // XPath: count(fn:tokenize(codepoints-to-string((97, 98, 99, 160, 100
    // Expected: 1
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_1() {
    //  fn:tokenize with a positional predicate.
    // XPath: fn:tokenize((\"abracadabra\", current-time())[1] treat as xs
    // Expected: true
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_2() {
    //  fn:tokenize with a positional predicate.
    // XPath: empty(fn:tokenize((\"abracadabra\", current-time())[1] treat
    // Expected: true
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_3() {
    //  fn:tokenize with a positional predicate(#2).
    // XPath: fn:tokenize((\"abracadabra\", current-time())[1] treat as xs
    // Expected: r
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_4() {
    //  fn:tokenize with a positional predicate(#3).
    // XPath: fn:tokenize((\"abracadabra\", current-time())[1] treat as xs
    // Expected: c
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_5() {
    //  Tokenize a sequence of words.
    // XPath: deep-equal(fn:tokenize(\"The cat sat on the mat\", \"\s+\"),
    // Expected: true 6 8 The cat sat on the mat
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_6() {
    //  A regexp that some Java versions have trouble with.
    // XPath: replace('APXterms6', '\w{3}\d*([^TKR0-9]+).*$', '$1')
    // Expected: terms
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn k2_tokenizefunc_7() {
    //  Tokenize on a single whitespace.
    // XPath: count(tokenize(\"a b\", \" \")), count(tokenize(\"a b\", \"\
    // Expected: 2 2 a|b a|b a b a b
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn cbcl_fn_tokenize_001() {
    //  Test boolean on tokenize
    // XPath: for $x in xs:string(zero-or-one((1 to 10)[. mod 2 = -1])) re
    // Expected:
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}

#[test]
fn cbcl_fn_tokenize_003() {
    //  Test tokenize on empty string
    // XPath: tokenize(string-join(for $x in (1 to 10)[. mod 2 < 0] return
    // Expected:
    // Calls stub function - will fail until fn:tokenize is implemented
    let _ = stub_fntokenize();
}
