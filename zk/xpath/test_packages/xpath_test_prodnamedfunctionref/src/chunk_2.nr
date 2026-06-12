//! Test chunk 2 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_107() {
    // Attempts to look up function fn:substring.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_108() {
    // Attempts to invoke function fn:substring.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring#3('string
    // Expected: s
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_109() {
    // Attempts to look up function fn:string-length.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string-lengt
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_110() {
    // Attempts to invoke function fn:string-length.
    // XPath: /root/child/Q{http://www.w3.org/2005/xpath-functions}string-
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_111() {
    // Attempts to look up function fn:string-length.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string-lengt
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_112() {
    // Attempts to invoke function fn:string-length.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}string-length#1('st
    // Expected: 6
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_113() {
    // Attempts to look up function fn:normalize-space.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}normalize-sp
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_114() {
    // Attempts to invoke function fn:normalize-space.
    // XPath: /root/child/Q{http://www.w3.org/2005/xpath-functions}normali
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_115() {
    // Attempts to look up function fn:normalize-space.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}normalize-sp
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_116() {
    // Attempts to invoke function fn:normalize-space.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}normalize-space#1('
    // Expected: string
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_117() {
    // Attempts to look up function fn:normalize-unicode.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}normalize-un
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_118() {
    // Attempts to invoke function fn:normalize-unicode.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}normalize-unicode#1
    // Expected: string
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_119() {
    // Attempts to look up function fn:normalize-unicode.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}normalize-un
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_120() {
    // Attempts to invoke function fn:normalize-unicode.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}normalize-unicode#2
    // Expected: string
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_121() {
    // Attempts to look up function fn:upper-case.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}upper-case#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_122() {
    // Attempts to invoke function fn:upper-case.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}upper-case#1('strin
    // Expected: STRING
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_123() {
    // Attempts to look up function fn:lower-case.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}lower-case#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_124() {
    // Attempts to invoke function fn:lower-case.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}lower-case#1('STRIN
    // Expected: string
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_125() {
    // Attempts to look up function fn:translate.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}translate#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_126() {
    // Attempts to invoke function fn:translate.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}translate#3('string
    // Expected: strong
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_127() {
    // Attempts to look up function fn:contains.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}contains#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_128() {
    // Attempts to invoke function fn:contains.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}contains#2('string'
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_129() {
    // Attempts to look up function fn:contains.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}contains#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_130() {
    // Attempts to invoke function fn:contains.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}contains#3('string'
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_131() {
    // Attempts to look up function fn:starts-with.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}starts-with#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_132() {
    // Attempts to invoke function fn:starts-with.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}starts-with#2('stri
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_133() {
    // Attempts to look up function fn:starts-with.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}starts-with#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_134() {
    // Attempts to invoke function fn:starts-with.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}starts-with#3('stri
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_135() {
    // Attempts to look up function fn:ends-with.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}ends-with#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_136() {
    // Attempts to invoke function fn:ends-with.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}ends-with#2('string
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_137() {
    // Attempts to look up function fn:ends-with.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}ends-with#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_138() {
    // Attempts to invoke function fn:ends-with.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}ends-with#3('string
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_139() {
    // Attempts to look up function fn:substring-before.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring-be
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_140() {
    // Attempts to invoke function fn:substring-before.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring-before#2(
    // Expected: str
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_141() {
    // Attempts to look up function fn:substring-before.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring-be
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_142() {
    // Attempts to invoke function fn:substring-before.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring-before#3(
    // Expected: str
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_143() {
    // Attempts to look up function fn:substring-after.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring-af
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_144() {
    // Attempts to invoke function fn:substring-after.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring-after#2('
    // Expected: ing
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_145() {
    // Attempts to look up function fn:substring-after.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}substring-af
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_146() {
    // Attempts to invoke function fn:substring-after.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}substring-after#3('
    // Expected: ing
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_147() {
    // Attempts to look up function fn:matches.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}matches#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_148() {
    // Attempts to invoke function fn:matches.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}matches#2('string',
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_149() {
    // Attempts to look up function fn:matches.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}matches#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_150() {
    // Attempts to invoke function fn:matches.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}matches#3('string',
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_151() {
    // Attempts to look up function fn:replace.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}replace#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_152() {
    // Attempts to invoke function fn:replace.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}replace#3('string',
    // Expected: strong
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_153() {
    // Attempts to look up function fn:replace.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}replace#4)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_154() {
    // Attempts to invoke function fn:replace.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}replace#4('string',
    // Expected: strong
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_155() {
    // Attempts to look up function fn:tokenize.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}tokenize#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_155a() {
    // Attempts to look up function fn:tokenize with single argument
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}tokenize#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
