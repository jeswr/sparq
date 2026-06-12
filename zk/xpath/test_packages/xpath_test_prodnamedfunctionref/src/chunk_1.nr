//! Test chunk 1 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_056() {
    // Attempts to invoke function fn:format-integer.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}format-integer#3(1,
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_057() {
    // Attempts to look up function fn:format-number.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}format-numbe
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_058() {
    // Attempts to invoke function fn:format-number.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}format-number#2(1,
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_059() {
    // Attempts to look up function fn:format-number.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}format-numbe
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_060() {
    // Attempts to invoke function fn:format-number.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}format-number#3(1,
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_061() {
    // Attempts to look up function math:pi.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}pi#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_062() {
    // Attempts to invoke function math:pi.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}pi#0()
    // Expected: 3.141592653589793
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_063() {
    // Attempts to look up function math:exp.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}exp#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_064() {
    // Attempts to invoke function math:exp.
    // XPath: format-number(Q{http://www.w3.org/2005/xpath-functions/math}
    // Expected: 2.7182818
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_065() {
    // Attempts to look up function math:exp10.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}exp10#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_066() {
    // Attempts to invoke function math:exp10.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}exp10#1(1e0)
    // Expected: 10
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_067() {
    // Attempts to look up function math:log.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}log#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_068() {
    // Attempts to invoke function math:log.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}log#1(1e0)
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_069() {
    // Attempts to look up function math:log10.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}log10#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_070() {
    // Attempts to invoke function math:log10.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}log10#1(1e0)
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_071() {
    // Attempts to look up function math:pow.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}pow#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_072() {
    // Attempts to invoke function math:pow.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}pow#2(1e0, 1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_073() {
    // Attempts to look up function math:sqrt.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}sqrt#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_074() {
    // Attempts to invoke function math:sqrt.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}sqrt#1(1e0)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_075() {
    // Attempts to look up function math:sin.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}sin#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_076() {
    // Attempts to invoke function math:sin.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}sin#1(1e0)
    // Expected: 0.8414709848078965
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_077() {
    // Attempts to look up function math:cos.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}cos#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_078() {
    // Attempts to invoke function math:cos.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}cos#1(1e0)
    // Expected: 0.5403023058681398
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_079() {
    // Attempts to look up function math:tan.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}tan#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_081() {
    // Attempts to look up function math:asin.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}asin#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_082() {
    // Attempts to invoke function math:asin.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}asin#1(1e0)
    // Expected: 1.5707963267948966
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_083() {
    // Attempts to look up function math:acos.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}acos#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_084() {
    // Attempts to invoke function math:acos.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}acos#1(1e0)
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_085() {
    // Attempts to look up function math:atan.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}atan#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_086() {
    // Attempts to invoke function math:atan.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}atan#1(1e0)
    // Expected: 0.7853981633974483
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_087() {
    // Attempts to look up function math:atan2.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions/math}atan2#2
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_088() {
    // Attempts to invoke function math:atan2.
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}atan2#2(1e0, 1
    // Expected: 0.7853981633974483
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_089() {
    // Attempts to look up function fn:codepoints-to-string.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}codepoints-t
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_090() {
    // Attempts to invoke function fn:codepoints-to-string.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}codepoints-to-strin
    // Expected: AB
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_091() {
    // Attempts to look up function fn:string-to-codepoints.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string-to-co
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_092() {
    // Attempts to invoke function fn:string-to-codepoints.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}string-to-codepoint
    // Expected: 65
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_093() {
    // Attempts to look up function fn:compare.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}compare#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_094() {
    // Attempts to invoke function fn:compare.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}compare#2('string',
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_095() {
    // Attempts to look up function fn:compare.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}compare#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_096() {
    // Attempts to invoke function fn:compare.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}compare#3('string',
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_097() {
    // Attempts to look up function fn:codepoint-equal.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}codepoint-eq
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_098() {
    // Attempts to invoke function fn:codepoint-equal.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}codepoint-equal#2('
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_099() {
    // Attempts to look up function fn:concat.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}concat#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_100() {
    // Attempts to invoke function fn:concat.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}concat#3('a', 'bc',
    // Expected: abcdef
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_101() {
    // Attempts to look up function fn:string-join.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string-join#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_102() {
    // Attempts to invoke function fn:string-join.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}string-join#1(('abc
    // Expected: abcdef
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_103() {
    // Attempts to look up function fn:string-join.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string-join#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_104() {
    // Attempts to invoke function fn:string-join.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}string-join#2(('abc
    // Expected: abc-def
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_105() {
    // Attempts to look up function fn:substring.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_106() {
    // Attempts to invoke function fn:substring.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring#2('string
    // Expected: tring
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
