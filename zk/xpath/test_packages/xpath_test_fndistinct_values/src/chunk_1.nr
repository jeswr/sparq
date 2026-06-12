//! Test chunk 1 for fn:distinct-values
//! Contains 32 tests

use dep::xpath::stub_fndistinct_values;

#[test]
fn fn_distinct_values_mixed_args_020() {
    //  arg: sequence of float & float
    // XPath: fn:distinct-values((xs:float('NaN'), xs:float('NaN')))
    // Expected: NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_021() {
    //  arg: sequence of float & double
    // XPath: fn:distinct-values((xs:float('NaN'), xs:double('NaN')))
    // Expected: NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_022() {
    //  arg: sequence of float & double
    // XPath: fn:distinct-values((xs:float('INF'), xs:double('INF')))
    // Expected: INF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_023() {
    //  arg: sequence of float & double
    // XPath: fn:distinct-values((xs:float('-INF'), xs:double('-INF')))
    // Expected: -INF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_025() {
    //  arg: sequence of double & double
    // XPath: fn:distinct-values((xs:double('NaN'), xs:double('NaN')))
    // Expected: NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_026() {
    //  arg: sequence of double & double
    // XPath: fn:distinct-values((xs:double('NaN'), xs:double('NaN')))
    // Expected: NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_029() {
    //  arg: sequence of boolean
    // XPath: fn:distinct-values((xs:boolean('true'), true()))
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_030() {
    //  arg: sequence of boolean
    // XPath: fn:distinct-values((xs:boolean('true'), xs:boolean('1')))
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_031() {
    //  arg: sequence of boolean
    // XPath: fn:distinct-values((xs:boolean('false'), xs:boolean('0')))
    // Expected: false
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_033() {
    // Mixed args, different numerics compare equal
    // XPath: count(distinct-values((           xs:integer(1),           x
    // Expected: 3
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_4() {
    //  A test whose essence is: `distinct-values('a string', 'http://www.w3.org/2005/x
    // XPath: distinct-values(\"a string\", \"http://www.w3.org/2005/xpath
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_6() {
    //  A test whose essence is: `deep-equal(distinct-values( ('1', 1, 2, 1, 1, 3, 1, 1
    // XPath: declare function local:canon($arg) {             for $i in
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_9() {
    //  A test whose essence is: `count(distinct-values((1, 2.0, 3, 2))) eq 3`.
    // XPath: count(distinct-values((1, 2.0, 3, 2))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_10() {
    //  A test whose essence is: `deep-equal(distinct-values((1, 2.0, 3, 2)), (1, 2.0,
    // XPath: deep-equal(distinct-values((1, 2.0, 3, 2)), (1, 2.0, 3))
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_11() {
    //  A test whose essence is: `exists(distinct-values((1, 2, 3, 1)))`.
    // XPath: exists(distinct-values((1, 2, 3, 1)))
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_12() {
    //  A test whose essence is: `not(empty(distinct-values((1, 1))))`.
    // XPath: not(empty(distinct-values((1, 1))))
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_13() {
    //  A test whose essence is: `count(distinct-values((1, 2, 2, current-time()))) eq
    // XPath: count(distinct-values((1, 2, 2, current-time()))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn k_seqdistinctvaluesfunc_15() {
    //  fn:distinct-values() applied on an argument of cardinality exactly-one.
    // XPath: count(distinct-values(current-time())) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_1() {
    //  Test Bugzilla #5183, [FO] Effect of type promotion in fn:distinct-values
    // XPath: let $input := (xs:float('1.0'), xs:decimal('1.00000000001000
    // Expected: true
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_001() {
    //  Test fn:distinct-values on a mixture of numeric types containing several NaN va
    // XPath: count(distinct-values((xs:integer(\"3\"), xs:float(\"3\"), x
    // Expected: 2
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_002b() {
    //  test fn:distinct-values with a mix of types
    // XPath: declare function local:create($arg) as xs:anyAtomicType* {
    // Expected: ---01 ---01 ---15Z ---15Z --06 --06 --06
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_004() {
    //  test fn:distinct-values with xs:date
    // XPath: for $value in fn:distinct-values(( xs:date('2008-06-01'), xs
    // Expected: 1918-11-11Z 1972-06-01Z 2008-06-01 2012-
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_005() {
    //  test fn:distinct-values with xs:dateTime
    // XPath: for $value in fn:distinct-values(( xs:dateTime('2008-06-01T1
    // Expected: 1918-11-11T11:00:00Z 1972-06-01T13:00:00
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_006() {
    //  test fn:distinct-values with xs:time
    // XPath: for $value in fn:distinct-values(( xs:time('12:00:00'), xs:t
    // Expected: 11:00:00 12:00:00
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_007() {
    //  test fn:distinct-values with xs:time
    // XPath: for $value at $p in ( for $time in fn:distinct-values(( xs:t
    // Expected: 13:00:00Z 12:00:00 20:00:00
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_008() {
    //  test fn:distinct-values with xs:hexBinary
    // XPath: for $value in fn:distinct-values(( xs:hexBinary('FFFF'), xs:
    // Expected: FF FFFE FFFF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_009() {
    //  test fn:distinct-values with xs:base64Binary
    // XPath: for $value in fn:distinct-values(( xs:base64Binary('aGVsbG8N
    // Expected: aGFsbG8NCg== aGVsbG8NCg== aGkNCg==
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_010() {
    //  test fn:distinct-values with xs:untypedAtomic
    // XPath: for $value in fn:distinct-values(( xs:untypedAtomic('a'), xs
    // Expected: a b
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_011() {
    //  test fn:distinct-values with xs:string
    // XPath: for $value in fn:distinct-values(( xs:string('a'), xs:string
    // Expected: a b
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_012() {
    //  test fn:distinct-values with xs:gYear
    // XPath: for $value in fn:distinct-values(( xs:gYear('2008'), xs:gYea
    // Expected: 1972 2008
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_013() {
    //  test fn:distinct-values with durations
    // XPath: for $value in fn:distinct-values(( xs:dayTimeDuration('P0D')
    // Expected: P1D P1Y PT0S
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn cbcl_distinct_values_014() {
    //  test fn:distinct-values with xs:gMnnthDay
    // XPath: for $value in fn:distinct-values(( xs:gMonthDay('--06-16'),
    // Expected: --06-16 --12-15
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}
