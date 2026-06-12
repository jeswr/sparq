//! Test chunk 3 for fn:min
//! Contains 25 tests

use dep::xpath::stub_fnmin;

#[test]
fn k2_seqminfunc_11() {
    //  fn:min() and xs:double promotion.
    // XPath: min((1, 1, 1, 1, 1.0)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_12() {
    //  fn:min() and xs:double promotion.
    // XPath: min((1.0, 1, 1, 1, 1)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_13() {
    //  fn:min() and xs:double promotion.
    // XPath: min((1.0, 1, 1.0, 1, 1)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_14() {
    //  fn:min() and xs:double promotion.
    // XPath: min((\"a\", \"b\", \"c\")) eq \"a\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_15() {
    //  Ensure the return type is properly inferred.
    // XPath: min(xs:unsignedShort(\"1\")) instance of xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_2() {
    //  Evaluation of type promotion when using mixed typed with fn:min function
    // XPath: fn:min((xs:integer(5000000),xs:double(3e8)))
    // Expected: 5.0E6
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_3() {
    // Evaluation of type promotion when using mixed typed with fn:min function (used d
    // XPath: let $var := fn:min((xs:long(22),xs:short(10))) return $var i
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_4() {
    // Test fn:min on a sequence of xs:dayTimeDuration arguments
    // XPath: min((xs:dayTimeDuration(\"P1D\"), xs:dayTimeDuration(\"PT2H\
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_5() {
    // Test fn:min on a sequence of xs:dayTimeDuration arguments
    // XPath: min(for $x in 1 to 10 return xs:dayTimeDuration(concat(\"PT\
    // Expected: PT1H
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_6() {
    // Test fn:min on a sequence of xs:yearMonthDuration arguments
    // XPath: min((xs:yearMonthDuration(\"P1Y\"), xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_7() {
    // Test fn:min on a sequence of xs:yearMonthDuration arguments
    // XPath: min(for $x in 1 to 10 return xs:yearMonthDuration(concat(\"P
    // Expected: P1M
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_10() {
    // Test fn:min on a sequence of numeric arguments
    // XPath: for $p in 1 to 4 let $x := (xs:integer(4), xs:decimal(3), xs
    // Expected: integer decimal float double
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_11() {
    // Test fn:min on a sequence of strings
    // XPath: min((\"a\", \"b\", \"c\", \"d\"))
    // Expected: a
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_12() {
    // Test fn:min on a sequence of dates
    // XPath: min((xs:date('1066-10-02'), xs:date('1588-08-08'), xs:date('
    // Expected: 1066-10-02
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_min_19() {
    //  Test fn:min on an array
    // XPath: min([1,2,3,4,5])
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_001() {
    //  test fn:min on xs:boolean arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 0) the
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_002() {
    //  test fn:min on xs:boolean arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 2) the
    // Expected: false
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_005() {
    //  test fn:min with xs:dateTime arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 2) the
    // Expected: 1996-12-01T12:00:00
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_007() {
    //  Test fn:min with xs:dayTimeDuration arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 2) the
    // Expected: PT3S
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_009() {
    //  Test fn:min with numeric arguments
    // XPath: declare function local:f($x as xs:integer) { (xs:decimal(1.1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_010() {
    //  Test fn:min with numeric arguments
    // XPath: declare function local:f($x as xs:integer) { (xs:decimal(1.3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_012() {
    //  Test fn:min with numeric arguments
    // XPath: declare function local:f($x as xs:integer) { (xs:decimal(1.1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_013() {
    //  test fn:min with xs:time arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 2) the
    // Expected: 12:00:00+01:00
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_015() {
    //  Test fn:min with xs:yearMonthDuration arguments
    // XPath: declare function local:f($x as xs:integer) { if ($x = 2) the
    // Expected: P11M
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn cbcl_min_018() {
    //  Test fn:min on a range expression
    // XPath: declare function local:f($x as xs:integer) as xs:integer { $
    // Expected: 4
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}
