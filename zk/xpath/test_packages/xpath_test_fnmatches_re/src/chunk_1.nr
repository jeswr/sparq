//! Test chunk 1 for fn:matches.re
//! Contains 50 tests

use dep::xpath::stub_fnmatches_re;

#[test]
fn re00070() {
    // Test regex syntax
    // XPath: (every $s in tokenize('}-', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00073() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[a
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00074() {
    // Test regex syntax
    // XPath: (every $s in tokenize('123', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00075() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00076() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1234567890:;<=>?@Azaz', ',') satisfie
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00077() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' ', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00078() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' ', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00079() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\|.?*+(){}-[]^', ',') satisfies match
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00080() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00081() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00082() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?a?,?b?,?c?', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00083() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00084() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00085() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:[\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00086a() {
    // Test regex syntax
    // XPath: (every $s in tokenize(',a-1x-7,c-4z-9,a-1z-8a-1z-9,a1z-9,a-1
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00089() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a*a****aaaaa*', ',') satisfies matche
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00091() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1]', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00092() {
    // Test regex syntax
    // XPath: (every $s in tokenize('=,>', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00094() {
    // Test regex syntax
    // XPath: (every $s in tokenize('@', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00095() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00096() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00098() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\,[,],\[,\[],[],[\\,\]\,[][', ',') sa
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00099() {
    // Test regex syntax
    // XPath: (every $s in tokenize('-', ',') satisfies matches($s, '^(?:[
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00100() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a--aa---', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00102() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a-x', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00103() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\|.-^?*+[]{}()*[[]{}}))       *()', '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00104() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a**,aa*,a', ',') satisfies matches($s
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00105() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a?,a?a?a?,a,a??,aa?', ',') satisfies
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00106() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\t', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00107() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\n', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00108() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\r', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00109() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' ', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00110() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' ', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00111() {
    // Test regex syntax
    // XPath: (every $s in tokenize('\', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00112() {
    // Test regex syntax
    // XPath: (every $s in tokenize('|', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00113() {
    // Test regex syntax
    // XPath: (every $s in tokenize('.', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00114() {
    // Test regex syntax
    // XPath: (every $s in tokenize('-', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00115() {
    // Test regex syntax
    // XPath: (every $s in tokenize('^', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00116() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00117() {
    // Test regex syntax
    // XPath: (every $s in tokenize('*', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00118() {
    // Test regex syntax
    // XPath: (every $s in tokenize('+', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00119() {
    // Test regex syntax
    // XPath: (every $s in tokenize('{', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00120() {
    // Test regex syntax
    // XPath: (every $s in tokenize('}', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00121() {
    // Test regex syntax
    // XPath: (every $s in tokenize('(', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00122() {
    // Test regex syntax
    // XPath: (every $s in tokenize(')', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00123() {
    // Test regex syntax
    // XPath: (every $s in tokenize('[', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00124() {
    // Test regex syntax
    // XPath: (every $s in tokenize(']', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00125() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\n
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00126() {
    // Test regex syntax
    // XPath: not(matches('', '^(?:\n\na\n\nb\n\n)$')) and          (every
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00128() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\t
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}
