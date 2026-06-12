//! Test chunk 14 for fn:matches.re
//! Contains 37 tests

use dep::xpath::stub_fnmatches_re;

#[test]
fn re00943() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d)*(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00944() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d)+?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00945() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d)+(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00946() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){2}(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00947() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){4,5}(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00948() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){4,5}?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00949() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){6,7}(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00950() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){6,7}?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00951() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){5,6}(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00952() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){5,6}?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00953() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){5,7}(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00954() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|c|d){5,7}?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00955() {
    // Test regex syntax
    // XPath: matches('qwerty','a(?:b|(c|e){1,2}?|d)+?(.)')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00957() {
    // Test regex syntax
    // XPath: (every $s in tokenize('ab9ab', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00972() {
    // Test regex syntax
    // XPath: matches('33a34', '^(\d){0,2}a\1$')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00973() {
    // Test regex syntax
    // XPath: matches('333a334', '^(\d*){2,}a\1$')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00974() {
    // Test regex syntax
    // XPath: matches('22a3', '^(\d*){0,2}?a\1$')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00975() {
    // Test regex syntax
    // XPath: matches('22a3', '^(\d*){2,}?a\1$')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00976a() {
    // Test regex syntax
    // XPath: matches('22a22', '^(\d*){2,}?a\1z$')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00976b() {
    // Test regex syntax
    // XPath: matches('22a22', '^(\d{2,})a\1$')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00979() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?????????????????????????????????????
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00980() {
    // Test regex syntax
    // XPath: (every $s in tokenize('?????????????????????????????????????
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00982() {
    // Test regex syntax
    // XPath: (every $s in tokenize('test@someverylongemailaddress.com', '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00983() {
    // Test regex syntax
    // XPath: (every $s in tokenize('first.last@seznam.cz,first-last@sezna
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00984() {
    // Test regex syntax
    // XPath: (every $s in tokenize('2,3,4,5,6,7,8,9,A,B,C,D,E,F,G,H,I,P,Q
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00985() {
    // Test regex syntax
    // XPath: (every $s in tokenize('0,1,2,3,4,5,6,7,8,9,?,?,?,?,?,?,?,?,?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00990() {
    // Test regex syntax: escaped dollar is OK in XPath
    // XPath: matches('$', '^\$$') and not(matches('\$', '^\$$'))
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00991() {
    // Test regex syntax: escaped dollar is OK in XPath
    // XPath: matches('$', '[\$]') and not(matches('\$', '^[\$]$'))
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00992() {
    // Backreference at end of string
    // XPath: matches('$$', '(.)\1')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00993() {
    // Backreference followed by digit
    // XPath: matches('$$9', '(.)\19')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00994() {
    // Two-digit Backreference
    // XPath: matches('$$9', '(((((((((((.)))))))))))\119')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00998() {
    // Quantifier after '^' (useless but allowed)
    // XPath: matches('alpha', 'alp^?ha')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00999() {
    // Quantifier after '^' (useless but allowed)
    // XPath: matches('alpha', 'alp^+ha')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re01000() {
    // Quantifier after '^' (useless but allowed)
    // XPath: matches('alpha', '^{2}alpha') and not(matches('zalpha', '^{2
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re01001() {
    // Quantifier after '$' (useless but allowed)
    // XPath: matches('alpha', 'alp$?ha')
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re01002() {
    // Quantifier after '$' (useless but allowed)
    // XPath: matches('alpha', 'alp${2,4}ha')
    // Expected: false
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re01003() {
    // Quantifier after '$' (useless but allowed)
    // XPath: matches('alpha', 'alpha$+') and not(matches('alphax', 'alpha
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}
