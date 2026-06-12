//! Test chunk 10 for fn:matches.re
//! Contains 50 tests

use dep::xpath::stub_fnmatches_re;

#[test]
fn re00534() {
    // Test regex syntax
    // XPath: (every $s in tokenize('123,12', ',') satisfies matches($s, '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00535() {
    // Test regex syntax
    // XPath: (every $s in tokenize('-300', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00536() {
    // Test regex syntax
    // XPath: (every $s in tokenize('-128', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00537() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1111', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00538() {
    // Test regex syntax
    // XPath: (every $s in tokenize('123', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00539() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\d
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00540() {
    // Test regex syntax
    // XPath: (every $s in tokenize('+1,1,+9', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00542() {
    // Test regex syntax
    // XPath: (every $s in tokenize('9,0', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00543() {
    // Test regex syntax
    // XPath: (every $s in tokenize('-11111,-9', ',') satisfies matches($s
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00544() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1,3', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00545() {
    // Test regex syntax
    // XPath: (every $s in tokenize('112233123,abcaabbccabc', ',') satisfi
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00546() {
    // Test regex syntax
    // XPath: (every $s in tokenize('112233123,abcaabbccabc,abab', ',') sa
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00547() {
    // Test regex syntax
    // XPath: (every $s in tokenize('abab', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00548() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00549() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00550() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00551() {
    // Test regex syntax
    // XPath: (every $s in tokenize('hellO worlD', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00552() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?ello ?orld', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00553() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00554() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[@
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00555() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[>
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00557() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[X
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00559() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[X
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00560() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[X
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00565() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Foo Bar,Foo Bar', ',') satisfies matc
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00575() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:fo
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00576() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:([
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00577() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:fo
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00578() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:fo
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00579() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:fo
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00580() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:fo
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00581() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World,Hello World', ',') satisf
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00582() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World,Hello World', ',') satisf
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00599() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00601() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00602() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\D
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00603() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00604() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00605() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00606() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00607() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00608() {
    // Test regex syntax
    // XPath: (every $s in tokenize('Hello World', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00611() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00612() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00617() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00639() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00640() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00641() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00643() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00644() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(f
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}
