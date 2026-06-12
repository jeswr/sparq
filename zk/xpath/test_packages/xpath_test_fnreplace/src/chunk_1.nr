//! Test chunk 1 for fn:replace
//! Contains 15 tests

use dep::xpath::stub_fnreplace;

#[test]
fn fn_replace_45() {
    // fn:replace, using $0
    // XPath: replace(\"Now, let's SEND OUT for QUICHE!!\", \"[A-Z][A-Z]+\
    // Expected: "Now, let's SENDSEND OUTOUT for QUICHEQU
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_46() {
    // fn:replace, Saxon bug 2166
    // XPath: replace(\"12-34\", \"^\d+(-(\d+))?$\", \"$2\")
    // Expected: "34"
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_47() {
    // fn:replace, Saxon bug 2390 (reluctant quantifiers, backtracking, and captured gr
    // XPath: replace('abc', '^.+?(b+)?$', '$1')
    // Expected: ""
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_48() {
    // fn:replace, Saxon bug 2390 (choice, backtracking, and captured groups)
    // XPath: replace('abcd', '^a(.).$|^a...$', '$1')
    // Expected: ""
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_49() {
    // Evaluation of replace function with q flag, with pattern set to '\'.
    // XPath: fn:replace(\"a/b/c\", \"/\", \"\\", \"q\")
    // Expected: a\b\c
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_50() {
    // Evaluation of replace function with q flag, with pattern set to '\\', example
    // XPath: fn:replace(\"a\b\c\", \"\\", \"\\\", \"q\")
    // Expected: a\\b\\c
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_51() {
    // Evaluation of replace function with q flag, with pattern set to '$', example fro
    // XPath: fn:replace(\"a/b/c\", \"/\", \"$\", \"q\")
    // Expected: a$b$c
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_52() {
    // Evaluation of replace function with q flag, with pattern set to '$''. (See bug #
    // XPath: fn:replace(\"a/b/c\", \"/\", \"$'\", \"q\")
    // Expected: a$'b$'c
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_53() {
    // Evaluation of replace function with q flag, with pattern set to '$`'. (See bug #
    // XPath: fn:replace(\"a/b/c\", \"/\", \"$`\", \"q\")
    // Expected: a$`b$`c
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_54() {
    // Evaluation of replace with single-digit capture at end of replacement string. Sa
    // XPath: fn:replace(\"abracadabra\", \"((((( ((((( (((((a))))) )))))
    // Expected: aabraacaadaabraa
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_55() {
    // Test case with undefined captures. Saxon bug 2865.
    // XPath: replace('abcdefghijk', '(a)|(b)|(c)|(d)|(e)|(f)|(g)|(h)|(i)|
    // Expected: ak
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_56() {
    // Test case based on Saxon bug 3429.
    // XPath: replace('10%', '^\d*\.?\d+', '')
    // Expected: %
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_57() {
    // Evaluation of replace with unmatched double-digit capturing group. Saxon bug 407
    // XPath: replace('987654321A', '^(9)(8)(7)(6)(5)(4)(3)(2)(1)((A*?)|(.
    // Expected: 1A
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_59() {
    // Evaluation of replace with curly braces. SaxonJS bug 6318.
    // XPath: replace('15/01/24', '(.{2})/(.{2})/(.{2})', '$3$2$1')
    // Expected: 240115
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn cbcl_fn_replace_001() {
    //  Tests a replace with prepared value evaluated to a boolean
    // XPath: boolean(replace(if(exists((1 to 10)[. mod 2 = 0])) then \"bl
    // Expected: true
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}
