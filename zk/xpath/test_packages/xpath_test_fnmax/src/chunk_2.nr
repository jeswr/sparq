//! Test chunk 2 for fn:max
//! Contains 50 tests

use dep::xpath::stub_fnmax;

#[test]
fn fn_maxsht2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:max((xs:short(\"-5324\"),xs:short(\"-32768\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:max((xs:short(\"32767\"),xs:short(\"-32768\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:max((xs:short(\"-32768\"),xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:max((xs:short(\"-32768\"),xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_3() {
    //  A test whose essence is: `empty(max(()))`.
    // XPath: empty(max(()))
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_4() {
    //  A test whose essence is: `max((3, 3, 3, 3, 3, 3)) eq 3`.
    // XPath: max((3, 3, 3, 3, 3, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_5() {
    //  A test whose essence is: `max((3, 1, 1, 1, 1, 1)) eq 3`.
    // XPath: max((3, 1, 1, 1, 1, 1)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_6() {
    //  A test whose essence is: `max((1, 1, 1, 1, 1, 3)) eq 3`.
    // XPath: max((1, 1, 1, 1, 1, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_7() {
    //  A test whose essence is: `max((3, 1, 5, 1, 1, 3)) eq 5`.
    // XPath: max((3, 1, 5, 1, 1, 3)) eq 5
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_8() {
    //  A test whose essence is: `max((3, -5.0, 5, 1, -3, 3)) eq 5`.
    // XPath: max((3, -5.0, 5, 1, -3, 3)) eq 5
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_9() {
    //  A test whose essence is: `max(xs:untypedAtomic('3')) instance of xs:double`.
    // XPath: max(xs:untypedAtomic(\"3\")) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_10() {
    //  A test whose essence is: `max(xs:untypedAtomic('3')) eq 3`.
    // XPath: max(xs:untypedAtomic(\"3\")) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_11() {
    //  A test whose essence is: `max((xs:untypedAtomic('3'), 1, 2)) instance of xs:dou
    // XPath: max((xs:untypedAtomic(\"3\"), 1, 2)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_12() {
    //  A test whose essence is: `max((1, xs:float(2), xs:untypedAtomic('3'))) eq 3`.
    // XPath: max((1, xs:float(2), xs:untypedAtomic(\"3\"))) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_13() {
    //  A test whose essence is: `max((1, xs:float(2), xs:untypedAtomic('3'))) instance
    // XPath: max((1, xs:float(2), xs:untypedAtomic(\"3\"))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_14() {
    //  A test whose essence is: `max((1, xs:float(2), xs:decimal(3))) instance of xs:f
    // XPath: max((1, xs:float(2), xs:decimal(3))) instance of xs:float
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_15() {
    //  A test whose essence is: `max((1, xs:untypedAtomic('3'), xs:float(2))) instance
    // XPath: max((1, xs:untypedAtomic(\"3\"), xs:float(2))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_16() {
    //  A test whose essence is: `string(max((1, xs:untypedAtomic('NaN'), xs:float(2)))
    // XPath: string(max((1, xs:untypedAtomic(\"NaN\"), xs:float(2)))) eq
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_17() {
    //  A test whose essence is: `string(max((xs:float('NaN'), xs:untypedAtomic('3'), x
    // XPath: string(max((xs:float(\"NaN\"), xs:untypedAtomic(\"3\"), xs:f
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_18() {
    //  A test whose essence is: `max((xs:float('NaN'), xs:untypedAtomic('3'), xs:float
    // XPath: max((xs:float(\"NaN\"), xs:untypedAtomic(\"3\"), xs:float(2)
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_19() {
    //  A test whose essence is: `max((xs:float('NaN'), 1, 1, 2, xs:double('NaN'))) ins
    // XPath: max((xs:float(\"NaN\"), 1, 1, 2, xs:double(\"NaN\"))) instan
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_20() {
    //  A test whose essence is: `max((xs:double('NaN'), 1, 1, 2, xs:float('NaN'))) ins
    // XPath: max((xs:double(\"NaN\"), 1, 1, 2, xs:float(\"NaN\"))) instan
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_25() {
    //  A test whose essence is: `string(max((xs:double('NaN'), xs:double('NaN')))) eq
    // XPath: string(max((xs:double(\"NaN\"), xs:double(\"NaN\")))) eq \"N
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_26() {
    //  A test whose essence is: `string(max((xs:float('NaN'), xs:float('NaN')))) eq 'N
    // XPath: string(max((xs:float(\"NaN\"), xs:float(\"NaN\")))) eq \"NaN
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_27() {
    //  A test whose essence is: `string(max((3, xs:double('NaN')))) eq 'NaN'`.
    // XPath: string(max((3, xs:double(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_28() {
    //  A test whose essence is: `string(max((3, xs:float('NaN')))) eq 'NaN'`.
    // XPath: string(max((3, xs:float(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_29() {
    //  A test whose essence is: `max((3, xs:double('NaN'))) instance of xs:double`.
    // XPath: max((3, xs:double(\"NaN\"))) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_30() {
    //  A test whose essence is: `max((3, xs:float('NaN'))) instance of xs:float`.
    // XPath: max((3, xs:float(\"NaN\"))) instance of xs:float
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_31() {
    //  A test whose essence is: `string(max((xs:float(-3), xs:untypedAtomic('3'), xs:d
    // XPath: string(max((xs:float(-3), xs:untypedAtomic(\"3\"), xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_32() {
    //  A test whose essence is: `max((xs:float(-3), xs:untypedAtomic('3'), xs:double('
    // XPath: max((xs:float(-3), xs:untypedAtomic(\"3\"), xs:double(\"NaN\
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_33() {
    //  A test whose essence is: `string(max(xs:float('NaN'))) eq 'NaN'`.
    // XPath: string(max(xs:float(\"NaN\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_34() {
    //  A test whose essence is: `string(max(xs:double('NaN'))) eq 'NaN'`.
    // XPath: string(max(xs:double(\"NaN\"))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_39() {
    //  A test whose essence is: `max(xs:anyURI('example.com/')) eq xs:anyURI('example.
    // XPath: max(xs:anyURI(\"example.com/\")) eq xs:anyURI(\"example.com/
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_40() {
    //  A test whose essence is: `max((xs:anyURI('example.com/'), xs:anyURI('example.co
    // XPath: max((xs:anyURI(\"example.com/\"), xs:anyURI(\"example.com/\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_41() {
    //  A test whose essence is: `max(('a string')) eq 'a string'`.
    // XPath: max((\"a string\")) eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_43() {
    //  A test whose essence is: `max((5, 5.0e0)) eq 5.0e0`.
    // XPath: max((5, 5.0e0)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_44() {
    //  A test whose essence is: `max((5, 5.0e0)) instance of xs:double`.
    // XPath: max((5, 5.0e0)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_45() {
    //  A test whose essence is: `max((5, 3.0e0)) instance of xs:double`.
    // XPath: max((5, 3.0e0)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_46() {
    //  A test whose essence is: `max((5.0e0, 5)) instance of xs:double`.
    // XPath: max((5.0e0, 5)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_47() {
    //  A test whose essence is: `max((3, 5.0e0)) instance of xs:double`.
    // XPath: max((3, 5.0e0)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_48() {
    //  A test whose essence is: `max((5.0e0, 3)) instance of xs:double`.
    // XPath: max((5.0e0, 3)) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_49() {
    //  A test whose essence is: `max((1, 1, 1, 1, 1.0)) instance of xs:decimal`.
    // XPath: max((1, 1, 1, 1, 1.0)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_50() {
    //  A test whose essence is: `max((1.0, 1, 1, 1, 1)) instance of xs:decimal`.
    // XPath: max((1.0, 1, 1, 1, 1)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_51() {
    //  A test whose essence is: `max((1.0, 1, 1.0, 1, 1)) instance of xs:decimal`.
    // XPath: max((1.0, 1, 1.0, 1, 1)) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_52() {
    //  A test whose essence is: `max((5.0e0, 5)) eq 5.0e0`.
    // XPath: max((5.0e0, 5)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_53() {
    //  A test whose essence is: `max((3, 5.0e0)) eq 5.0e0`.
    // XPath: max((3, 5.0e0)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_54() {
    //  A test whose essence is: `max((5.0e0, 3)) eq 5.0e0`.
    // XPath: max((5.0e0, 3)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_55() {
    //  Testing max() with current-date().
    // XPath: max((current-date(), xs:date(\"1999-01-01\"))) eq current-da
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_56() {
    //  A test whose essence is: `max((3,4,5)) eq 5`.
    // XPath: max((3,4,5)) eq 5
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn k_seqmaxfunc_57() {
    //  A test whose essence is: `max((5, 5.0e0)) eq 5.0e0`.
    // XPath: max((5, 5.0e0)) eq 5.0e0
    // Expected: true
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}
