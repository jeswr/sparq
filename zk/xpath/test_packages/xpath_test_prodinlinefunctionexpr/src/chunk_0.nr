//! Test chunk 0 for prod:InlineFunctionExpr
//! Contains 19 tests

use dep::xpath::stub_prodinlinefunctionexpr;

#[test]
fn inline_fn_001() {
    // Basic inline function
    // XPath: let $f := function($x){$x+1} return $f(2)
    // Expected: 3
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_002() {
    // Zero-arity function
    // XPath: let $pi := function(){3.14159} return $pi()
    // Expected: 3.14159
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_003() {
    // Arity two function
    // XPath: let $add := function($x, $y){$x + $y} return $add(4, 2)
    // Expected: 6
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_004() {
    // Function accessing local variables
    // XPath: sum(for $i in 1 to 6 return              let $add := functio
    // Expected: 45
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_008() {
    // Nested inline functions
    // XPath: let $outer := function($x) {             let $inner := funct
    // Expected: 8
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_009() {
    // Nested inline functions accessing a 'grandfather' variable
    // XPath: let $bonus := 10,              $outer := function($x) {
    // Expected: 18
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_010() {
    // Function with declared parameter types
    // XPath: let $add := function($x as xs:integer, $y as xs:integer){$x
    // Expected: 7
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_012() {
    // Function with declared return type
    // XPath: let $add := function($x, $y) as xs:integer {$x + $y}
    // Expected: 7
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_014() {
    // Function returning a function containing a closure
    // XPath: let $process := function($op, $x, $y) as function(*) {if ($o
    // Expected: 7
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_015() {
    // Annotations are allowed in XQuery
    // XPath: let $add := %Q{http://example.com/speed}fast %Q{http://examp
    // Expected: 4
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_022() {
    // Arity of an inline function
    // XPath: let $add := function($x, $y) {$x + $y} return function-arity
    // Expected: 2
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_023() {
    // Apply an inline function
    // XPath: let $add := function($x, $y) {$x + $y} return apply($add, [1
    // Expected: 17
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_024() {
    // An inline function is a primary
    // XPath: function($x, $y) {$x + $y} (12, 5)
    // Expected: 17
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_025() {
    // An inline function is a primary
    // XPath: function($x, $y) {$x + $y} [function-arity(.) = 2] (12, 5)
    // Expected: 17
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_026() {
    // Pass a function to itself
    // XPath: let $weirdo := function($x, $f) {if ($x=0) then 0 else $f($x
    // Expected: 0
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_027() {
    // A map containing inline functions
    // XPath: let $calc := map{             \"+\" : function($a, $b) {$a +
    // Expected: 6
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_034() {
    // Inline function accessing global variable in XQuery
    // XPath: declare variable $p as xs:integer external;          declare
    // Expected: 19
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_035() {
    // Inline function with closure accessing global variables and functions in XQuery
    // XPath: declare variable $p as xs:integer external;          declare
    // Expected: -26
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}

#[test]
fn inline_fn_037() {
    // No-namespace annotations are allowed
    // XPath: let $add := %Q{}fast(\"\")  function($x, $y) {$x + $y} retur
    // Expected: 4
    // Calls stub function - will fail until prod:InlineFunctionExpr is implemented
    let _ = stub_prodinlinefunctionexpr();
}
