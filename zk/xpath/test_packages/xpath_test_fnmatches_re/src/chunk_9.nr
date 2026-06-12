//! Test chunk 9 for fn:matches.re
//! Contains 50 tests

use dep::xpath::stub_fnmatches_re;

#[test]
fn re00483() {
    // Test regex syntax
    // XPath: (every $s in tokenize('     a c    a         ,aa a', ',') sa
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00484() {
    // Test regex syntax
    // XPath: (every $s in tokenize('aa,a a,a   a', ',') satisfies matches
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00485() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:a\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00486() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a', ',') satisfies matches($s, '^(?:\
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00487() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\S
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00488() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\S
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00489() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a b , a   ', ',') satisfies matches($
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00490() {
    // Test regex syntax
    // XPath: (every $s in tokenize('_,:,a', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00491() {
    // Test regex syntax
    // XPath: (every $s in tokenize('_:abcdefghijklmnopqrstuvwxyzAZ:_', ',
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00492() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\i
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00493() {
    // Test regex syntax
    // XPath: (every $s in tokenize('zabcsdea', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00494() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a b  c  Z  :_   d y  b     ', ',') sa
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00495() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' , , , ', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00496() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1234', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00497() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a  123c', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00498() {
    // Test regex syntax
    // XPath: (every $s in tokenize('_,:,a', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00499() {
    // Test regex syntax
    // XPath: (every $s in tokenize('c?1 abc,?0 zzz', ',') satisfies match
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00500() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a,aa,aaaaaaaaaaaaaaaaaaaaaaaaaa', ','
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00501() {
    // Test regex syntax
    // XPath: (every $s in tokenize(' , , , ', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00502() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a*a**a***,aa*a', ',') satisfies match
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00503() {
    // Test regex syntax
    // XPath: (every $s in tokenize('0,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,9
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00504() {
    // Test regex syntax
    // XPath: (every $s in tokenize('),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00505() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\w
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00506() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\W
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00507() {
    // Test regex syntax
    // XPath: (every $s in tokenize('true', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00508() {
    // Test regex syntax
    // XPath: (every $s in tokenize('false', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00509() {
    // Test regex syntax
    // XPath: (every $s in tokenize('true,false', ',') satisfies matches($
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00510() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1', ',') satisfies matches($s, '^(?:(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00511() {
    // Test regex syntax
    // XPath: (every $s in tokenize('0', ',') satisfies matches($s, '^(?:(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00512() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1111,11001010', ',') satisfies matche
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00513() {
    // Test regex syntax
    // XPath: (every $s in tokenize('AF01D1', ',') satisfies matches($s, '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00514() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1.001,1.001', ',') satisfies matches(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00515() {
    // Test regex syntax
    // XPath: (every $s in tokenize('http://www.foo.com', ',') satisfies m
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00516() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a:b', ',') satisfies matches($s, '^(?
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00517() {
    // Test regex syntax
    // XPath: (every $s in tokenize('P1111Y12M', ',') satisfies matches($s
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00518() {
    // Test regex syntax
    // XPath: (every $s in tokenize('2001-06-06T12:12:00', ',') satisfies
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00519() {
    // Test regex syntax
    // XPath: (every $s in tokenize('11:00:00,13:20:00-05:00', ',') satisf
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00520() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1999-12-12', ',') satisfies matches($
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00522() {
    // Test regex syntax
    // XPath: (every $s in tokenize('1999', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00523() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\p
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00524() {
    // Test regex syntax
    // XPath: (every $s in tokenize('--03-14', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00525() {
    // Test regex syntax
    // XPath: (every $s in tokenize('---30', ',') satisfies matches($s, '^
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00526() {
    // Test regex syntax
    // XPath: (every $s in tokenize('--12--', ',') satisfies matches($s, '
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00527() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a,abcdef', ',') satisfies matches($s,
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00528() {
    // Test regex syntax
    // XPath: (every $s in tokenize('ch-a', ',') satisfies matches($s, '^(
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00529() {
    // Test regex syntax
    // XPath: (every $s in tokenize('ab', ',') satisfies matches($s, '^(?:
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00530() {
    // Test regex syntax
    // XPath: (every $s in tokenize('a1b,ab,ab,name1', ',') satisfies matc
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00531() {
    // Test regex syntax
    // XPath: (every $s in tokenize('10000101,10000201', ',') satisfies ma
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00532() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\-
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}

#[test]
fn re00533() {
    // Test regex syntax
    // XPath: (every $s in tokenize('', ',') satisfies matches($s, '^(?:\-
    // Expected: true
    // Calls stub function - will fail until fn:matches.re is implemented
    let _ = stub_fnmatches_re();
}
