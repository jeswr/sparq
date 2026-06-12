//! Test chunk 0 for fn:insert-before
//! Contains 18 tests

use dep::xpath::stub_fninsert_before;

#[test]
fn k_seqinsertbeforefunc_5() {
    //  A test whose essence is: `insert-before((), -31, 'a string') eq 'a string'`.
    // XPath: insert-before((), -31, \"a string\") eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_6() {
    //  A test whose essence is: `count(insert-before((1, 2, 3), 1, (4, 5, 6))) eq 6`.
    // XPath: count(insert-before((1, 2, 3), 1, (4, 5, 6))) eq 6
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_7() {
    //  A test whose essence is: `insert-before((), 1, 3) eq 3`.
    // XPath: insert-before((), 1, 3) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_8() {
    //  A test whose essence is: `insert-before((), 1, 'a string') eq 'a string'`.
    // XPath: insert-before((), 1, \"a string\") eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_9() {
    //  A test whose essence is: `count(insert-before((), 1, (1, 2, 3))) eq 3`.
    // XPath: count(insert-before((), 1, (1, 2, 3))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_10() {
    //  A test whose essence is: `insert-before((), 30, 7) eq 7`.
    // XPath: insert-before((), 30, 7) eq 7
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_11() {
    //  A test whose essence is: `count(insert-before((1, 2, 3, 4), 30, ())) eq 4`.
    // XPath: count(insert-before((1, 2, 3, 4), 30, ())) eq 4
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_12() {
    //  A test whose essence is: `insert-before(9, 30, ()) eq 9`.
    // XPath: insert-before(9, 30, ()) eq 9
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_13() {
    //  A test whose essence is: `count(insert-before((1, 2, 3, 4), 1, ())) eq 4`.
    // XPath: count(insert-before((1, 2, 3, 4), 1, ())) eq 4
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_14() {
    //  A test whose essence is: `count(insert-before((1, 2, 3), 30, (4, 5, 6))) eq 6`.
    // XPath: count(insert-before((1, 2, 3), 30, (4, 5, 6))) eq 6
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_15() {
    //  A test whose essence is: `count(insert-before((), 30, (1, 2, 3))) eq 3`.
    // XPath: count(insert-before((), 30, (1, 2, 3))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_17() {
    //  Apply a predicate to the result of fn:insert-before().
    // XPath: (insert-before((1, current-time(), 3), 1, (4, 5, 6))[last()]
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_18() {
    //  Apply a predicate to the result of fn:insert-before().
    // XPath: (insert-before((1, current-time(), 3), 10, (4, 5, 6))[last()
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_19() {
    //  Apply a predicate to the result of fn:insert-before().
    // XPath: (insert-before((1, current-time(), 3), 10, (4, 5, 6))[last()
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_20() {
    //  Apply a predicate to the result of fn:insert-before().
    // XPath: (insert-before((1, current-time(), 3), 10, ())[last()] treat
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn k_seqinsertbeforefunc_21() {
    //  Apply a predicate to the result of fn:insert-before().
    // XPath: empty(insert-before((1, current-time(), 3), 0, (4, 5, 6))[la
    // Expected: true
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn cbcl_fn_insert_before_001() {
    //  tests insert-before at known positions
    // XPath: insert-before(1 to 10,5,20 to 30)
    // Expected: 1 2 3 4 20 21 22 23 24 25 26 27 28 29 30
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}

#[test]
fn cbcl_fn_insert_before_002() {
    //  Tests insert-before for known positions
    // XPath: insert-before((1 to 10,(20 to 30)[. mod 2 = 0],30 to 40),12,
    // Expected: 1 2 3 4 5 6 7 8 9 10 20 blah 22 24 26 28
    // Calls stub function - will fail until fn:insert-before is implemented
    let _ = stub_fninsert_before();
}
