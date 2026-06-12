//! Test chunk 0 for fn:function-arity
//! Contains 9 tests

use dep::xpath::stub_fnfunction_arity;

#[test]
fn fn_function_arity_005() {
    // Attempts to reference the 'function-arity' function with arity one.
    // XPath: exists(fn:function-arity#1)
    // Expected: true
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_015() {
    // Evaluates the 'function-arity' function with the argument set as follows: $func
    // XPath: fn:function-arity( fn:dateTime#2 )
    // Expected: 2
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_016() {
    // Evaluates the 'function-arity' function with the argument set as follows: $func
    // XPath: fn:function-arity( fn:concat#99 )
    // Expected: 99
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_018() {
    // Evaluates the 'function-arity' function with the argument set as follows: functi
    // XPath: fn:function-arity( function($node){name($node)} )
    // Expected: 1
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_019() {
    // Evaluates the 'function-arity' function with the argument set as follows: functi
    // XPath: fn:function-arity( function($arg1, $arg2)
    // Expected: 2
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_020() {
    // Evaluates the 'function-arity' function with the argument referring to a partial
    // XPath: let $initial := fn:substring(?, 1, 1)              return fn
    // Expected: 1
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_021() {
    // Evaluates the 'function-arity' function with the argument referring to a partial
    // XPath: fn:function-arity( math:pow(?, 10) )
    // Expected: 1
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_022() {
    // Evaluates the 'function-arity' function with the argument referring a global use
    // XPath: declare function local:add($arg1, $arg2, $arg3)         {
    // Expected: 3
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}

#[test]
fn fn_function_arity_023() {
    // Evaluates the 'function-arity' function with the argument referring a partial ap
    // XPath: declare function local:add($arg1, $arg2, $arg3)         {
    // Expected: 1
    // Calls stub function - will fail until fn:function-arity is implemented
    let _ = stub_fnfunction_arity();
}
