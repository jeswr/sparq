//! Test chunk 3 for fn:max
//! Contains 26 tests

use dep::xpath::stub_fnmax;

#[test]
fn k_seqmaxfunc_59() {
    //  A test whose essence is: `max(('a', 'b', 'c')) eq 'c'`.
    // XPath: max((\"a\", \"b\", \"c\")) eq \"c\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k2_seqmaxfunc_1() {
    //  Invoke fn:max() on two identical xs:anyURI values.
    // XPath: max((xs:anyURI(\"http://example.com/A\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k2_seqmaxfunc_2() {
    //  Invoke fn:max() on two different xs:anyURI values.
    // XPath: max((xs:anyURI(\"http://example.com/B\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k2_seqmaxfunc_3() {
    //  Invoke fn:max() on two different xs:anyURI values(#2).
    // XPath: max((xs:anyURI(\"http://example.com/8\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k2_seqmaxfunc_7() {
    //  Ensure the return type is properly inferred.
    // XPath: max(xs:unsignedShort(\"1\")) instance of xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_2() {
    //  Evaluation of type promotion when using mixed typed with fn:max function
    // XPath: fn:max((xs:integer(5000000000),xs:double(3e0)))
    // Expected: 5.0E9
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_3() {
    //  Evaluation of type promotion when using mixed typed with fn:max function Uses d
    // XPath: let $var := fn:max((xs:long(20),xs:short(13))) return $var i
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_4() {
    //  Test fn:max on a sequence of xs:dayTimeDuration arguments
    // XPath: max((xs:dayTimeDuration(\"P1D\"), xs:dayTimeDuration(\"PT2H\
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_5() {
    //  Test fn:max on a sequence of xs:dayTimeDuration arguments
    // XPath: max(for $x in 1 to 10 return xs:dayTimeDuration(concat(\"PT\
    // Expected: PT10H
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_6() {
    //  Test fn:max on a sequence of xs:yearMonthDuration arguments
    // XPath: max((xs:yearMonthDuration(\"P1Y\"), xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_7() {
    //  Test fn:max on a sequence of xs:yearMonthDuration arguments
    // XPath: max(for $x in 1 to 10 return xs:yearMonthDuration(concat(\"P
    // Expected: P10M
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_10() {
    //  Test fn:max on a sequence of numeric arguments
    // XPath: for $p in 1 to 4 let $x := (xs:integer(1), xs:decimal(2), xs
    // Expected: integer decimal float double
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_11() {
    //  Test fn:max on a sequence of strings
    // XPath: max((\"a\", \"b\", \"c\", \"d\"))
    // Expected: d
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_12() {
    //  Test fn:max on a sequence of dates
    // XPath: max((xs:date('1066-10-02'), xs:date('1588-08-08'), xs:date('
    // Expected: 2011-06-29
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_max_19() {
    //  Test fn:max on an array
    // XPath: max([1,2,3,4,5])
    // Expected: 5
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_001() {
    //  test fn:max on xs:boolean arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: false
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_002() {
    //  test fn:max on xs:boolean arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_005() {
    //  test fn:max with xs:dateTime arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: 3000-12-01T12:00:00
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_007() {
    //  Test fn:max with xs:dayTimeDuration arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: P1D
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_009() {
    //  Test fn:max with numeric arguments
    // XPath: declare function local:f($x as xs:integer) {          (xs:de
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_010() {
    //  Test fn:max with numeric arguments
    // XPath: declare function local:f($x as xs:integer) {          (xs:de
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_012() {
    //  Test fn:max with numeric arguments
    // XPath: declare function local:f($x as xs:integer) {          (xs:de
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_013() {
    //  test fn:max with xs:time arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: 12:00:00-01:00
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_015() {
    //  Test fn:max with xs:yearMonthDuration arguments
    // XPath: declare function local:f($x as xs:integer) {          if ($x
    // Expected: P1Y1M
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_018() {
    //  Test fn:max on a range expression
    // XPath: declare function local:f($x as xs:integer) as xs:integer { $
    // Expected: 10
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn cbcl_max_019() {
    //  Test evaluating effective boolean value of max
    // XPath: if (max(for $x in 1 to 10 return $x mod 9 = 0)) then true()
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}
