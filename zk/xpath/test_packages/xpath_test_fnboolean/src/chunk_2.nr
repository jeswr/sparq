//! Test chunk 2 for fn:boolean
//! Contains 23 tests

use dep::xpath::stub_fnboolean;

#[test]
fn k_seqbooleanfunc_19() {
    // zzzA test whose essence is: `boolean(1.1)`.
    // XPath: boolean(1.1)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_20() {
    // sA test whose essence is: `boolean(1.1e1)`.
    // XPath: boolean(1.1e1)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_21() {
    // sA test whose essence is: `not(0.0e0)`.
    // XPath: not(0.0e0)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_22() {
    // sA test whose essence is: `not(0.0)`.
    // XPath: not(0.0)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_23() {
    // sA test whose essence is: `boolean(-1)`.
    // XPath: boolean(-1)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_24() {
    // A test whose essence is: `not(())`.
    // XPath: not(())
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_25() {
    // A test whose essence is: `not(boolean(()))`.
    // XPath: not(boolean(()))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_26() {
    // A test whose essence is: `not(())`.
    // XPath: not(())
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_27() {
    //  A test whose essence is: `not(empty((1, 2)))`.
    // XPath: not(empty((1, 2)))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_28() {
    // A test whose essence is: `not(empty(((), 1, 2)))`.
    // XPath: not(empty(((), 1, 2)))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_29() {
    // A test whose essence is: `boolean('nada')`.
    // XPath: boolean('nada')
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_30() {
    // A test whose essence is: `boolean('''')`.
    // XPath: boolean('\"\"')
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_31() {
    // A test whose essence is: `not('')`.
    // XPath: not('')
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn k_seqbooleanfunc_32() {
    // Apply fn:boolean() on fn:count().
    // XPath: fn:boolean(count((1, 2, 3, timezone-from-time(current-time()
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn boolean_001() {
    // boolean() applied to a node sequence
    // XPath: boolean(//*:Open)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn boolean_002() {
    // boolean() applied to an empty node sequence
    // XPath: boolean(//*:NotAtAllOpen)
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn boolean_003() {
    // boolean() applied to a sequence whose first item is a node
    // XPath: boolean((/, 93.7))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn boolean_008() {
    // boolean() applied to a sequence of of a boolean and then an empty          node-
    // XPath: boolean((true(), //aspidistra))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn boolean_009() {
    // boolean() applied to the string 'false'
    // XPath: boolean(string(false()))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn cbcl_boolean_001() {
    // test fn:boolean on a count-preserving function
    // XPath: declare function local:generate($count as xs:integer) {
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn cbcl_boolean_002() {
    //  test fn:boolean on fn:count function
    // XPath: declare function local:generate($count as xs:integer) {
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn cbcl_boolean_005() {
    // test fn:boolean on a text node
    // XPath: declare function local:repeat($count as xs:integer, $arg as
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn cbcl_boolean_006() {
    // test fn:boolean on a text node
    // XPath: declare function local:repeat($count as xs:integer, $arg as
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}
