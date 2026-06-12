//! Test chunk 0 for fn:function-lookup
//! Contains 50 tests

use dep::xpath::stub_fnfunction_lookup;

#[test]
fn fn_function_lookup_001() {
    // Attempts to look up function fn:node-name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_002() {
    // Attempts to invoke function fn:node-name.
    // XPath: /root/function-lookup(fn:QName('http://www.w3.org/2005/xpath
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_003() {
    // Attempts to look up function fn:node-name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_004() {
    // Attempts to invoke function fn:node-name.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_005() {
    // Attempts to look up function fn:nilled.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_006() {
    // Attempts to invoke function fn:nilled.
    // XPath: /root/function-lookup(fn:QName('http://www.w3.org/2005/xpath
    // Expected: false
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_007() {
    // Attempts to look up function fn:nilled.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_008() {
    // Attempts to invoke function fn:nilled.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: false
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_009() {
    // Attempts to look up function fn:string.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_010() {
    // Attempts to invoke function fn:string.
    // XPath: /root/child/function-lookup(fn:QName('http://www.w3.org/2005
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_011() {
    // Attempts to look up function fn:string.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_012() {
    // Attempts to invoke function fn:string.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_013() {
    // Attempts to look up function fn:data.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_014() {
    // Attempts to invoke function fn:data.
    // XPath: /root/child/function-lookup(fn:QName('http://www.w3.org/2005
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_015() {
    // Attempts to look up function fn:data.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_016() {
    // Attempts to invoke function fn:data.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_017() {
    // Attempts to look up function fn:base-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_018() {
    // Attempts to invoke function fn:base-uri.
    // XPath: let $baseUriFn := /root/function-lookup(fn:QName('http://www
    // Expected: "http://www.w3.org/fots/fn/function-look
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_019() {
    // Attempts to look up function fn:base-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_020() {
    // Attempts to invoke function fn:base-uri.
    // XPath: let $baseUriFn := function-lookup(fn:QName('http://www.w3.or
    // Expected: http://www.w3.org/fots/fn/function-looku
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_021() {
    // Attempts to look up function fn:document-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_023() {
    // Attempts to look up function fn:document-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_025() {
    // Attempts to look up function fn:error.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_027() {
    // Attempts to look up function fn:error.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_029() {
    // Attempts to look up function fn:error.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_031() {
    // Attempts to look up function fn:error.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_033() {
    // Attempts to look up function fn:trace.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_034() {
    // Attempts to invoke function fn:trace.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_035() {
    // Attempts to look up function fn:abs.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_036() {
    // Attempts to invoke function fn:abs.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_037() {
    // Attempts to look up function fn:ceiling.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_038() {
    // Attempts to invoke function fn:ceiling.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_039() {
    // Attempts to look up function fn:floor.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_040() {
    // Attempts to invoke function fn:floor.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_041() {
    // Attempts to look up function fn:round.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_042() {
    // Attempts to invoke function fn:round.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_043() {
    // Attempts to look up function fn:round.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_044() {
    // Attempts to invoke function fn:round.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_045() {
    // Attempts to look up function fn:round-half-to-even.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_046() {
    // Attempts to invoke function fn:round-half-to-even.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_047() {
    // Attempts to look up function fn:round-half-to-even.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_048() {
    // Attempts to invoke function fn:round-half-to-even.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_049() {
    // Attempts to look up function fn:number.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_050() {
    // Attempts to invoke function fn:number.
    // XPath: /root/child/function-lookup(fn:QName('http://www.w3.org/2005
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_051() {
    // Attempts to look up function fn:number.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_052() {
    // Attempts to invoke function fn:number.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_053() {
    // Attempts to look up function fn:format-integer.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_054() {
    // Attempts to invoke function fn:format-integer.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_055() {
    // Attempts to look up function fn:format-integer.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_056() {
    // Attempts to invoke function fn:format-integer.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}
