//! Test chunk 2 for fn:matches.re
//! Contains 50 tests

use dep::xpath::stub_fnmatches_re;

#[test]
fn re00130() {
    // Test regex syntax
    // XPath: let $d := codepoints-to-string(13) return (          (every
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00130a() {
    // Test regex syntax
    // XPath: (every $s in tokenize('&#xA;&#xD;a&#xA;&#xD;b', ',') satisfi
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00131() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' a b c ', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00132() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' a b c ', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00133() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' a       b, a  b, a  b , a      b', '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00134() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\c', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00135() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\.,\s,\S,\i,\I,\c,\C,\d,\D,\w,\W', ';
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00136() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\.abcd,\sssss,\SSSSSS,\iiiiiii,\,\c,\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00137() {
    // Test regex syntax
    // XPath: (every $s in tokenize('aX', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00138() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:(\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00139() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00140() {
    // Test regex syntax
    // XPath: (every $s in tokenize('A?', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00141() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a?', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00142() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00143() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00144() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00145() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?????????', ',') satisfies matches($s
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00146() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00147() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00148() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00149() {
    // Test regex syntax
    // XPath: (every $s in tokenize('????????', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00150() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00151() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00152() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00153() {
    // Test regex syntax
    // XPath: (every $s in tokenize('????????????????????', ',') satisfies
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00154() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00155() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00156() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00157() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00158() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00159() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00160() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00161() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??????', ',') satisfies matches($s, '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00162() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00163() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00164() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00165() {
    // Test regex syntax
    // XPath: (every $s in tokenize('???????????', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00166() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00167() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00168() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00169() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00170() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00171() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' ', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00172() {
    // Test regex syntax
    // XPath: (every $s in tokenize('??', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00173() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?????', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00174() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00175() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00176() {
    // Test regex syntax
    // XPath: (every $s in tokenize('_,?', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00177() {
    // Test regex syntax
    // XPath: (every $s in tokenize(',#$', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00178() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}
