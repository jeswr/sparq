//! Test chunk 2 for fn:min
//! Contains 50 tests

use dep::xpath::stub_fnmin;

#[test]
fn fn_minsht2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:min((xs:short(\"-5324\"),xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:min((xs:short(\"32767\"),xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:min((xs:short(\"-32768\"),xs:short(\"-5324\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:min((xs:short(\"-32768\"),xs:short(\"32767\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_3() {
    //  A test whose essence is: `empty(min(()))`.
    // XPath: empty(min(()))
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_4() {
    //  A test whose essence is: `min((3, 3, 3, 3, 3, 3)) eq 3`.
    // XPath: min((3, 3, 3, 3, 3, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_5() {
    //  A test whose essence is: `min((3, 1, 1, 1, 1, 1)) eq 1`.
    // XPath: min((3, 1, 1, 1, 1, 1)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_6() {
    //  A test whose essence is: `min((1, 1, 1, 1, 1, 3)) eq 1`.
    // XPath: min((1, 1, 1, 1, 1, 3)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_7() {
    //  A test whose essence is: `min((3, 1, 5, 1, 1, 3)) eq 1`.
    // XPath: min((3, 1, 5, 1, 1, 3)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_8() {
    //  A test whose essence is: `min((3, -5.0, 5, 1, -3, 3)) eq -5.0`.
    // XPath: min((3, -5.0, 5, 1, -3, 3)) eq -5.0
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_9() {
    //  A test whose essence is: `min(xs:untypedAtomic('3')) instance of xs:double`.
    // XPath: min(xs:untypedAtomic(\"3\")) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_10() {
    //  A test whose essence is: `min(xs:untypedAtomic('3')) eq 3`.
    // XPath: min(xs:untypedAtomic(\"3\")) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_11() {
    //  A test whose essence is: `min((xs:untypedAtomic('1'), 3, 2)) instance of xs:dou
    // XPath: min((xs:untypedAtomic(\"1\"), 3, 2)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_12() {
    //  A test whose essence is: `min((3, xs:float(2), xs:untypedAtomic('1'))) eq 1`.
    // XPath: min((3, xs:float(2), xs:untypedAtomic(\"1\"))) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_13() {
    //  A test whose essence is: `min((3, xs:float(2), xs:untypedAtomic('1'))) instance
    // XPath: min((3, xs:float(2), xs:untypedAtomic(\"1\"))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_14() {
    //  A test whose essence is: `min((1, xs:float(2), xs:decimal(3))) instance of xs:f
    // XPath: min((1, xs:float(2), xs:decimal(3))) instance of xs:float
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_15() {
    //  A test whose essence is: `min((3, xs:untypedAtomic('1'), xs:float(2))) instance
    // XPath: min((3, xs:untypedAtomic(\"1\"), xs:float(2))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_16() {
    //  A test whose essence is: `string(min((1, xs:untypedAtomic('NaN'), xs:float(2)))
    // XPath: string(min((1, xs:untypedAtomic(\"NaN\"), xs:float(2)))) eq
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_17() {
    //  A test whose essence is: `string(min((xs:float('NaN'), xs:untypedAtomic('3'), x
    // XPath: string(min((xs:float(\"NaN\"), xs:untypedAtomic(\"3\"), xs:f
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_18() {
    //  A test whose essence is: `min((xs:float('NaN'), xs:untypedAtomic('3'), xs:doubl
    // XPath: min((xs:float(\"NaN\"), xs:untypedAtomic(\"3\"), xs:double(2
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_19() {
    //  A test whose essence is: `min((xs:float('NaN'), 1, 1, 2, xs:double('NaN'))) ins
    // XPath: min((xs:float(\"NaN\"), 1, 1, 2, xs:double(\"NaN\"))) instan
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_20() {
    //  A test whose essence is: `min((xs:double('NaN'), 1, 1, 2, xs:float('NaN'))) ins
    // XPath: min((xs:double(\"NaN\"), 1, 1, 2, xs:float(\"NaN\"))) instan
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_25() {
    //  A test whose essence is: `string(min((xs:double('NaN'), xs:double('NaN')))) eq
    // XPath: string(min((xs:double(\"NaN\"), xs:double(\"NaN\")))) eq \"N
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_26() {
    //  A test whose essence is: `string(min((xs:float('NaN'), xs:float('NaN')))) eq 'N
    // XPath: string(min((xs:float(\"NaN\"), xs:float(\"NaN\")))) eq \"NaN
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_27() {
    //  A test whose essence is: `string(min((3, xs:double('NaN')))) eq 'NaN'`.
    // XPath: string(min((3, xs:double(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_28() {
    //  A test whose essence is: `string(min((3, xs:float('NaN')))) eq 'NaN'`.
    // XPath: string(min((3, xs:float(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_29() {
    //  A test whose essence is: `min((3, xs:double('NaN'))) instance of xs:double`.
    // XPath: min((3, xs:double(\"NaN\"))) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_30() {
    //  A test whose essence is: `min((3, xs:float('NaN'))) instance of xs:float`.
    // XPath: min((3, xs:float(\"NaN\"))) instance of xs:float
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_31() {
    //  A test whose essence is: `string(min((xs:float(-3), xs:untypedAtomic('3'), xs:d
    // XPath: string(min((xs:float(-3), xs:untypedAtomic(\"3\"), xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_32() {
    //  A test whose essence is: `min((xs:float(-3), xs:untypedAtomic('3'), xs:double('
    // XPath: min((xs:float(-3), xs:untypedAtomic(\"3\"), xs:double(\"NaN\
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_33() {
    //  A test whose essence is: `string(min(xs:float('NaN'))) eq 'NaN'`.
    // XPath: string(min(xs:float(\"NaN\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_34() {
    //  A test whose essence is: `string(min(xs:double('NaN'))) eq 'NaN'`.
    // XPath: string(min(xs:double(\"NaN\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_39() {
    //  A test whose essence is: `min(xs:anyURI('example.com/')) eq xs:anyURI('example.
    // XPath: min(xs:anyURI(\"example.com/\")) eq xs:anyURI(\"example.com/
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_40() {
    //  A test whose essence is: `min((xs:anyURI('example.com/'), xs:anyURI('example.co
    // XPath: min((xs:anyURI(\"example.com/\"), xs:anyURI(\"example.com/\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_41() {
    //  A test whose essence is: `min(('a string')) eq 'a string'`.
    // XPath: min((\"a string\")) eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_43() {
    //  A test whose essence is: `min((5, 5.0e0)) eq 5.0e0`.
    // XPath: min((5, 5.0e0)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_44() {
    //  A test whose essence is: `min((5.0e0, 5)) eq 5.0e0`.
    // XPath: min((5.0e0, 5)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_45() {
    //  A test whose essence is: `min((3, 5.0e0)) eq 3`.
    // XPath: min((3, 5.0e0)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_46() {
    //  A test whose essence is: `min((5.0e0, 3)) eq 3`.
    // XPath: min((5.0e0, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_47() {
    //  Simple test for min() involving xs:date.
    // XPath: min((xs:date(\"2005-01-01\"), xs:date(\"2001-01-01\"))) eq x
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_48() {
    //  A test whose essence is: `min((3,4,5)) eq 3`.
    // XPath: min((3,4,5)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_49() {
    //  A test whose essence is: `min((5, 5.0e0)) eq 5.0e0`.
    // XPath: min((5, 5.0e0)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k_seqminfunc_51() {
    //  A test whose essence is: `min((xs:float(0.0E0), xs:float(-0.0E0))) eq xs:float(
    // XPath: min((xs:float(0.0E0), xs:float(-0.0E0))) eq xs:float(\"-0\")
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_1() {
    //  Invoke fn:min() on two identical xs:anyURI values.
    // XPath: min((xs:anyURI(\"http://example.com/A\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_2() {
    //  Invoke fn:min() on two different xs:anyURI values.
    // XPath: min((xs:anyURI(\"http://example.com/B\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_3() {
    //  Invoke fn:min() on two different xs:anyURI values(#2).
    // XPath: min((xs:anyURI(\"http://example.com/8\"), xs:anyURI(\"http:/
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_7() {
    //  fn:min() and xs:double promotion.
    // XPath: min((5, 5.0e0)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_8() {
    //  fn:min() and xs:double promotion.
    // XPath: min((5.0e0, 5)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_9() {
    //  fn:min() and xs:double promotion.
    // XPath: min((3, 5.0e0)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn k2_seqminfunc_10() {
    //  fn:min() and xs:double promotion.
    // XPath: min((5.0e0, 3)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}
