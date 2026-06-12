//! Test chunk 0 for fn:apply
//! Contains 10 tests

use dep::xpath::stub_fnapply;

#[test]
fn fn_apply_01() {
    //  basic use of apply
    // XPath: apply(abs#1, [-2])
    // Expected: 2
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_02() {
    // Call built-in function with 0 arguments
    // XPath: apply(true#0, array {})
    // Expected: true
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_03() {
    // Invoke built-in function with sequence as argument
    // XPath: apply(string-join#1, [reverse(1 to 5) ! string()])
    // Expected: "54321"
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_04() {
    // Invoke inline function
    // XPath: let $func := function($a,$b,$c) { $a + $b + $c }
    // Expected: 6
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_05() {
    // Invoke user-defined function
    // XPath: declare function local:square($n as xs:integer) { $n * $n };
    // Expected: 100
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_06() {
    // Partial function application
    // XPath: apply(substring('flower', ?, ?), [ 3, 2 ])
    // Expected: "ow"
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_07() {
    // Invoke function retrieved via function-lookup
    // XPath: let $f := function-lookup(xs:QName('fn:count'), 1)
    // Expected: 2
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_14() {
    // Invoke atomization
    // XPath: fn:apply(lower-case#1, [<a>ABC</a>])
    // Expected: "abc"
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_15() {
    // Apply user-defined function, with atomization
    // XPath: declare function local:l($x as xs:string){lower-case($x)};
    // Expected: "BANANA"
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}

#[test]
fn fn_apply_16() {
    // An XPath test to defeat rewriting as a static function call
    // XPath: apply(if (current-date() lt xs:date('1900-01-01')) then abs#
    // Expected: -2
    // Calls stub function - will fail until fn:apply is implemented
    let _ = stub_fnapply();
}
