//! Test chunk 0 for fn:not
//! Contains 5 tests

use dep::xpath::fn_not;

#[test]
fn k_notfunc_3() {
    // K-NotFunc-3  A test whose essence is: `not(false() and false())`.
    assert(fn_not(false) == true);
}

#[test]
fn k_notfunc_4() {
    // K-NotFunc-4  A test whose essence is: `not(not(true()))`.
    assert(fn_not(false) == true);
}

#[test]
fn k_notfunc_5() {
    // K-NotFunc-5  A test whose essence is: `not(false())`.
    assert(fn_not(false) == true);
}

#[test]
fn k_notfunc_9() {
    // K-NotFunc-9  A test whose essence is: `not(not(xs:anyURI('example.com/')))`.
    assert(fn_not(false) == true);
}

#[test]
fn cbcl_not_003() {
    //  test fn:not on a context-sensitive comparison
    assert(fn_not(true) == false);
}
