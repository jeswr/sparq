//! Test chunk 0 for fn:environment-variable
//! Contains 6 tests

use dep::xpath::stub_fnenvironment_variable;

#[test]
fn environment_variable_001() {
    // Check that the function exists
    // XPath: fn:exists(environment-variable#1)
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}

#[test]
fn environment_variable_002() {
    // Check that we can look the function up
    // XPath: not(fn:empty(fn:function-lookup(    fn:QName('http://www.w3.
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}

#[test]
fn environment_variable_005() {
    // Check that we get back () or $QTTEST = 42
    // XPath: let $all := fn:available-environment-variables()    return e
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}

#[test]
fn environment_variable_006() {
    // Check that $QTTEST2 is 'other'
    // XPath: let $all := fn:available-environment-variables()    return e
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}

#[test]
fn environment_variable_007() {
    // Check we can distinguish an unset variable from an empty one
    // XPath: let $all := fn:available-environment-variables()    return e
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}

#[test]
fn environment_variable_012() {
    // check for overflow by constructing a one-megabyte name
    // XPath: let $var := string-join(    for $i in (1 to (1024 * 1024)) r
    // Expected: true
    // Calls stub function - will fail until fn:environment-variable is implemented
    let _ = stub_fnenvironment_variable();
}
