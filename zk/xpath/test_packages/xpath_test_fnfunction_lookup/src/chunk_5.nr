//! Test chunk 5 for fn:function-lookup
//! Contains 50 tests

use dep::xpath::stub_fnfunction_lookup;

#[test]
fn fn_function_lookup_268() {
    // Attempts to invoke function fn:local-name.
    // XPath: /root/function-lookup(fn:QName('http://www.w3.org/2005/xpath
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_269() {
    // Attempts to look up function fn:local-name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_270() {
    // Attempts to invoke function fn:local-name.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_271() {
    // Attempts to look up function fn:namespace-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_272() {
    // Attempts to invoke function fn:namespace-uri.
    // XPath: /root/*[2]/function-lookup(fn:QName('http://www.w3.org/2005/
    // Expected: http://www.example.org/
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_273() {
    // Attempts to look up function fn:namespace-uri.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_274() {
    // Attempts to invoke function fn:namespace-uri.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: http://www.example.org/
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_275() {
    // Attempts to look up function fn:lang.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_276() {
    // Attempts to invoke function fn:lang.
    // XPath: /root/function-lookup(fn:QName('http://www.w3.org/2005/xpath
    // Expected: false
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_277() {
    // Attempts to look up function fn:lang.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_278() {
    // Attempts to invoke function fn:lang.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: false
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_279() {
    // Attempts to look up function fn:root.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_281() {
    // Attempts to look up function fn:root.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_283() {
    // Attempts to look up function fn:path.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_284() {
    // Attempts to invoke function fn:path.
    // XPath: /function-lookup(fn:QName('http://www.w3.org/2005/xpath-func
    // Expected: /
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_285() {
    // Attempts to look up function fn:path.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_286() {
    // Attempts to invoke function fn:path.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: /
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_287() {
    // Attempts to look up function fn:has-children.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_288() {
    // Attempts to invoke function fn:has-children.
    // XPath: /function-lookup(fn:QName('http://www.w3.org/2005/xpath-func
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_289() {
    // Attempts to look up function fn:has-children.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_290() {
    // Attempts to invoke function fn:has-children.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_291() {
    // Attempts to look up function fn:innermost.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_293() {
    // Attempts to look up function fn:outermost.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_295() {
    // Attempts to look up function fn:empty.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_296() {
    // Attempts to invoke function fn:empty.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: false
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_297() {
    // Attempts to look up function fn:exists.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_298() {
    // Attempts to invoke function fn:exists.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_299() {
    // Attempts to look up function fn:head.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_300() {
    // Attempts to invoke function fn:head.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_301() {
    // Attempts to look up function fn:tail.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_302() {
    // Attempts to invoke function fn:tail.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_303() {
    // Attempts to look up function fn:insert-before.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_304() {
    // Attempts to invoke function fn:insert-before.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1 a b c 2 3
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_305() {
    // Attempts to look up function fn:remove.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_306() {
    // Attempts to invoke function fn:remove.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: a c
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_307() {
    // Attempts to look up function fn:reverse.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_308() {
    // Attempts to invoke function fn:reverse.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 3 2 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_309() {
    // Attempts to look up function fn:subsequence.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_310() {
    // Attempts to invoke function fn:subsequence.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_311() {
    // Attempts to look up function fn:subsequence.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_312() {
    // Attempts to invoke function fn:subsequence.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_313() {
    // Attempts to look up function fn:unordered.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_314() {
    // Attempts to invoke function fn:unordered.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_315() {
    // Attempts to look up function fn:distinct-values.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_316() {
    // Attempts to invoke function fn:distinct-values.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_317() {
    // Attempts to look up function fn:distinct-values.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_318() {
    // Attempts to invoke function fn:distinct-values.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_319() {
    // Attempts to look up function fn:index-of.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_320() {
    // Attempts to invoke function fn:index-of.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_321() {
    // Attempts to look up function fn:index-of.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}
