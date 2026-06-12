//! Test chunk 0 for fn:function-name
//! Contains 8 tests

use dep::xpath::stub_fnfunction_name;

#[test]
fn fn_function_name_005() {
    // Attempts to reference the 'function-name' function with arity one.
    // XPath: exists(fn:function-name#1)
    // Expected: true
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_015() {
    // Evaluates the 'function-name' function with the argument set as follows: $func =
    // XPath: fn:function-name( dateTime#2 )
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_016() {
    // Evaluates the 'function-name' function with the argument set as follows: $func =
    // XPath: fn:function-name( math:pow#2 )
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_017() {
    // Evaluates the 'function-name' function with the argument set as follows: $func =
    // XPath: fn:function-name( concat#99 )
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_022() {
    // Evaluates the 'function-name' function with the argument referring a global user
    // XPath: declare function local:add($arg1, $arg2, $arg3)         {
    // Expected: fn:QName("http://www.w3.org/2005/xquery-
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_024() {
    // Evaluates the 'function-name' function with the argument referring to a function
    // XPath: declare function local:coerce($arg as function(item()*) as i
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_025() {
    // Applies function-name to a context-dependent built-in function.
    // XPath: fn:function-name(fn:node-name#0)
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}

#[test]
fn fn_function_name_026() {
    // Applies function-name to a context-dependent built-in function.
    // XPath: fn:function-name(fn:lang#1)
    // Expected: fn:QName("http://www.w3.org/2005/xpath-f
    // Calls stub function - will fail until fn:function-name is implemented
    let _ = stub_fnfunction_name();
}
