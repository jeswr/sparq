//! Test chunk 0 for fn:abs
//! Contains 13 tests

use dep::xpath::abs_int;

#[test]
fn k2_absfunc_1() {
    // Test: K2-ABSFunc-1 type xs:integer.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_2() {
    // Test: K2-ABSFunc-2 type xs:nonPositiveInteger.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_3() {
    // Test: K2-ABSFunc-3 type xs:negativeInteger.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_4() {
    // Test: K2-ABSFunc-4 type xs:long.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_5() {
    // Test: K2-ABSFunc-5 type xs:int.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_6() {
    // Test: K2-ABSFunc-6 type xs:short.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_7() {
    // Test: K2-ABSFunc-7 type xs:byte.
    assert(abs_int(-4) == 4);
}

#[test]
fn k2_absfunc_8() {
    // Test: K2-ABSFunc-8 type xs:nonNegativeInteger.
    assert(abs_int(4) == 4);
}

#[test]
fn k2_absfunc_9() {
    // Test: K2-ABSFunc-9 type xs:unsignedLong.
    assert(abs_int(4) == 4);
}

#[test]
fn k2_absfunc_10() {
    // Test: K2-ABSFunc-10 type xs:unsignedInt.
    assert(abs_int(4) == 4);
}

#[test]
fn k2_absfunc_11() {
    // Test: K2-ABSFunc-11 type xs:unsignedShort.
    assert(abs_int(4) == 4);
}

#[test]
fn k2_absfunc_12() {
    // Test: K2-ABSFunc-12 type xs:unsignedByte.
    assert(abs_int(4) == 4);
}

#[test]
fn k2_absfunc_13() {
    // Test: K2-ABSFunc-13 type xs:positiveInteger.
    assert(abs_int(4) == 4);
}
