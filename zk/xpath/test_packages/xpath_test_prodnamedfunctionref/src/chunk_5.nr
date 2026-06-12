//! Test chunk 5 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_264() {
    // Attempts to invoke function fn:name.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}name#0()
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_265() {
    // Attempts to look up function fn:name.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}name#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_266() {
    // Attempts to invoke function fn:name.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}name#1(/root)
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_267() {
    // Attempts to look up function fn:local-name.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}local-name#0
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_268() {
    // Attempts to invoke function fn:local-name.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}local-name#0(
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_269() {
    // Attempts to look up function fn:local-name.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}local-name#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_270() {
    // Attempts to invoke function fn:local-name.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}local-name#1(/root)
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_271() {
    // Attempts to look up function fn:namespace-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}namespace-ur
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_272() {
    // Attempts to invoke function fn:namespace-uri.
    // XPath: /root/*[2]/Q{http://www.w3.org/2005/xpath-functions}namespac
    // Expected: http://www.example.org/
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_273() {
    // Attempts to look up function fn:namespace-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}namespace-ur
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_274() {
    // Attempts to invoke function fn:namespace-uri.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}namespace-uri#1(/ro
    // Expected: http://www.example.org/
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_275() {
    // Attempts to look up function fn:lang.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}lang#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_276() {
    // Attempts to invoke function fn:lang.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}lang#1('en')
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_277() {
    // Attempts to look up function fn:lang.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}lang#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_278() {
    // Attempts to invoke function fn:lang.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}lang#2('en', /root)
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_279() {
    // Attempts to look up function fn:root.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}root#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_281() {
    // Attempts to look up function fn:root.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}root#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_283() {
    // Attempts to look up function fn:path.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}path#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_284() {
    // Attempts to invoke function fn:path.
    // XPath: /Q{http://www.w3.org/2005/xpath-functions}path#0()
    // Expected: /
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_285() {
    // Attempts to look up function fn:path.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}path#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_286() {
    // Attempts to invoke function fn:path.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}path#1(/)
    // Expected: /
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_287() {
    // Attempts to look up function fn:has-children.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}has-children
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_288() {
    // Attempts to invoke function fn:has-children.
    // XPath: /Q{http://www.w3.org/2005/xpath-functions}has-children#0()
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_289() {
    // Attempts to look up function fn:has-children.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}has-children
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_290() {
    // Attempts to invoke function fn:has-children.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}has-children#1(/)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_291() {
    // Attempts to look up function fn:innermost.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}innermost#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_293() {
    // Attempts to look up function fn:outermost.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}outermost#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_295() {
    // Attempts to look up function fn:empty.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}empty#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_296() {
    // Attempts to invoke function fn:empty.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}empty#1((1, true())
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_297() {
    // Attempts to look up function fn:exists.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}exists#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_298() {
    // Attempts to invoke function fn:exists.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}exists#1((1, true()
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_299() {
    // Attempts to look up function fn:head.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}head#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_300() {
    // Attempts to invoke function fn:head.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}head#1((1, true()))
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_301() {
    // Attempts to look up function fn:tail.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}tail#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_302() {
    // Attempts to invoke function fn:tail.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}tail#1((1, true()))
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_303() {
    // Attempts to look up function fn:insert-before.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}insert-befor
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_304() {
    // Attempts to invoke function fn:insert-before.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}insert-before#3((1,
    // Expected: 1 a b c 2 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_305() {
    // Attempts to look up function fn:remove.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}remove#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_306() {
    // Attempts to invoke function fn:remove.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}remove#2(('a', 'b',
    // Expected: a c
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_307() {
    // Attempts to look up function fn:reverse.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}reverse#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_308() {
    // Attempts to invoke function fn:reverse.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}reverse#1(1 to 3)
    // Expected: 3 2 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_309() {
    // Attempts to look up function fn:subsequence.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}subsequence#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_310() {
    // Attempts to invoke function fn:subsequence.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}subsequence#2((1, t
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_311() {
    // Attempts to look up function fn:subsequence.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}subsequence#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_312() {
    // Attempts to invoke function fn:subsequence.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}subsequence#3((1, t
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_313() {
    // Attempts to look up function fn:unordered.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}unordered#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_314() {
    // Attempts to invoke function fn:unordered.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}unordered#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_315() {
    // Attempts to look up function fn:distinct-values.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}distinct-val
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_316() {
    // Attempts to invoke function fn:distinct-values.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}distinct-values#1((
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_317() {
    // Attempts to look up function fn:distinct-values.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}distinct-val
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
