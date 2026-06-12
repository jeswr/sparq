//! Test chunk 8 for fn:function-lookup
//! Contains 50 tests

use dep::xpath::stub_fnfunction_lookup;

#[test]
fn fn_function_lookup_455() {
    // Attempts to look up function xs:nonPositiveInteger.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_456() {
    // Attempts to invoke function xs:nonPositiveInteger.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: -1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_457() {
    // Attempts to look up function xs:negativeInteger.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_458() {
    // Attempts to invoke function xs:negativeInteger.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: -1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_459() {
    // Attempts to look up function xs:long.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_460() {
    // Attempts to invoke function xs:long.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_461() {
    // Attempts to look up function xs:int.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_462() {
    // Attempts to invoke function xs:int.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_463() {
    // Attempts to look up function xs:short.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_464() {
    // Attempts to invoke function xs:short.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_465() {
    // Attempts to look up function xs:byte.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_466() {
    // Attempts to invoke function xs:byte.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_467() {
    // Attempts to look up function xs:nonNegativeInteger.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_468() {
    // Attempts to invoke function xs:nonNegativeInteger.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_469() {
    // Attempts to look up function xs:unsignedLong.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_470() {
    // Attempts to invoke function xs:unsignedLong.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_471() {
    // Attempts to look up function xs:unsignedInt.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_472() {
    // Attempts to invoke function xs:unsignedInt.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_473() {
    // Attempts to look up function xs:unsignedShort.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_474() {
    // Attempts to invoke function xs:unsignedShort.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_475() {
    // Attempts to look up function xs:unsignedByte.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_476() {
    // Attempts to invoke function xs:unsignedByte.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_477() {
    // Attempts to look up function xs:nonNegativeInteger.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_478() {
    // Attempts to invoke function xs:nonNegativeInteger.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_479() {
    // Attempts to look up function xs:positiveInteger.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_480() {
    // Attempts to invoke function xs:positiveInteger.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_481() {
    // Attempts to look up function xs:gYearMonth.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_482() {
    // Attempts to invoke function xs:gYearMonth.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 2001-10Z
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_483() {
    // Attempts to look up function xs:gYear.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_484() {
    // Attempts to invoke function xs:gYear.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: 2012Z
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_485() {
    // Attempts to look up function xs:gMonthDay.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_486() {
    // Attempts to invoke function xs:gMonthDay.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: --11-01Z
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_487() {
    // Attempts to look up function xs:gDay.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_488() {
    // Attempts to invoke function xs:gDay.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: ---01Z
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_489() {
    // Attempts to look up function xs:gMonth.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_490() {
    // Attempts to invoke function xs:gMonth.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: --11Z
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_491() {
    // Attempts to look up function xs:string.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_492() {
    // Attempts to invoke function xs:string.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: string
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_493() {
    // Attempts to look up function xs:normalizeString.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_494() {
    // Attempts to invoke function xs:normalizeString.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: normalized string
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_495() {
    // Attempts to look up function xs:token.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_496() {
    // Attempts to invoke function xs:token.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: token
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_497() {
    // Attempts to look up function xs:language.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_498() {
    // Attempts to invoke function xs:language.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: en
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_499() {
    // Attempts to look up function xs:NMTOKEN.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_500() {
    // Attempts to invoke function xs:NMTOKEN.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: NMTOKEN
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_501() {
    // Attempts to look up function xs:Name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_502() {
    // Attempts to invoke function xs:Name.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: Name
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_503() {
    // Attempts to look up function xs:NCName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2001/XMLS
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_504() {
    // Attempts to invoke function xs:NCName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2001/XMLSchema',
    // Expected: NCName
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}
