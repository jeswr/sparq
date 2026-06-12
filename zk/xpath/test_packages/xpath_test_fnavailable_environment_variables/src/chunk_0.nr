//! Test chunk 0 for fn:available-environment-variables
//! Contains 9 tests

use dep::xpath::stub_fnavailable_environment_variables;

#[test]
fn fn_available_environment_variables_001() {
    // Check that the function exists
    // XPath: fn:exists(fn:available-environment-variables#0)
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_002() {
    // Check that we can look the function up
    // XPath: not(fn:empty(fn:function-lookup(    fn:QName('http://www.w3.
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_005() {
    // Check that we get back () or a sequence that accepts a predicate
    // XPath: let $all := fn:available-environment-variables()    return e
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_006() {
    // Check that no returned values are non-strings
    // XPath: (for $e in fn:available-environment-variables()     return f
    // Expected: false
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_007() {
    // Check that we can call environment-variable on each returned result
    // XPath: let $all := fn:available-environment-variables(),         $n
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_008() {
    // check there are no duplicates in the variable names
    // XPath: let $all := fn:available-environment-variables()    return c
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_009() {
    // check that the result is at least moderately stable
    // XPath: let $first := <all>{     for $e in fn:available-environment-
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_010() {
    // check that requesting a not-available variable returns ()
    // XPath: let $all := fn:available-environment-variables()    return i
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}

#[test]
fn fn_available_environment_variables_011() {
    // check that if any env var is available, QTTEST is a string, '42'
    // XPath: let $all := fn:available-environment-variables()    return i
    // Expected: true
    // Calls stub function - will fail until fn:available-environment-variables is implemented
    let _ = stub_fnavailable_environment_variables();
}
