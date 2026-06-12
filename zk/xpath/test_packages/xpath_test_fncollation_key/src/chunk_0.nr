//! Test chunk 0 for fn:collation-key
//! Contains 23 tests

use dep::xpath::stub_fncollation_key;

#[test]
fn collation_key_001() {
    // Basic test with default collation
    // XPath: collation-key(\"abc\") eq collation-key(\"123\")
    // Expected: false
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_002() {
    // Basic test with default collation
    // XPath: collation-key(\"abc\") eq collation-key(\"abc\")
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_003() {
    // Basic test with default collation
    // XPath: (collation-key(\"abc\") eq collation-key(\"ABC\")) eq (compa
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_004() {
    // Basic test with default collation
    // XPath: (collation-key(\"abc\") lt collation-key(\"ABC\")) eq (compa
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_005() {
    // Simple comparison using codepoint collation explicitly
    // XPath: collation-key(\"abc\", \"http://www.w3.org/2005/xpath-functi
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_006() {
    // URI promotion works
    // XPath: collation-key(\"http://www.w3.org/\") eq collation-key(xs:an
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_007() {
    // Simple comparison using default collation explicitly
    // XPath: collation-key(\"abc\")                 eq collation-key(\"ab
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_008() {
    // Simple comparison using a case-blind collation
    // XPath: let $C := \"http://www.w3.org/2013/collation/UCA?lang=en;str
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_009u() {
    // Ordered comparison using a case-sensitive collation
    // XPath: let $C := \"http://www.w3.org/2013/collation/UCA?lang=en;cas
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_009l() {
    // Ordered comparison using a case-sensitive collation
    // XPath: let $C := \"http://www.w3.org/2013/collation/UCA?lang=en;cas
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_010() {
    // Equality comparison using the HTML case-blind collation
    // XPath: let $C := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_015() {
    // Codepoint collation keys work correctly for surrogate pairs
    // XPath: collation-key(codepoints-to-string((37, 65500, 37))) lt coll
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_101() {
    // Basic test with HTML5 collation
    // XPath: let $c := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: false
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_102() {
    // Basic test with HTML5 collation
    // XPath: let $c := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_103() {
    // Basic test with HTML5 collation
    // XPath: let $c := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_104() {
    // Basic test with HTML5 collation
    // XPath: let $c := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: false
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_105() {
    // Use maps with HTML5 collation
    // XPath: let $C := \"http://www.w3.org/2005/xpath-functions/collation
    // Expected: 2
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_201() {
    // Basic test with case-blind, accent-blind collation
    // XPath: let $c := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: false
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_202() {
    // Basic test with case-blind, accent-blind collation
    // XPath: let $c := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_203() {
    // Basic test with case-blind, accent-blind collation
    // XPath: let $c := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_204() {
    // Basic test with case-blind, accent-blind collation
    // XPath: let $c := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: false
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_205() {
    // Basic test with case-blind, accent-blind collation
    // XPath: let $c := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: true
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}

#[test]
fn collation_key_206() {
    // Use maps with UCA collation
    // XPath: let $C := \"http://www.w3.org/2013/collation/UCA?strength=pr
    // Expected: 2
    // Calls stub function - will fail until fn:collation-key is implemented
    let _ = stub_fncollation_key();
}
