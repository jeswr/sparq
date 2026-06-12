//! Test chunk 0 for fn:round-half-to-even
//! Contains 50 tests

use dep::xpath::stub_fnround_half_to_even;

#[test]
fn fn_round_half_to_evenint1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:int(\"-2147483648\"))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenint1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenint1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:int(\"2147483647\"))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenintg1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:integer(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenintg1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:integer(\"830993497117024304\"))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenintg1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:integer(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evendec1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:decimal(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evendec1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:decimal(\"617375191608514839\"))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evendec1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:decimal(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evendbl1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenflt1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenlng1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:long(\"-92233720368547758\"))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenlng1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:long(\"-47175562203048468\"))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenlng1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:long(\"92233720368547758\"))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenusht1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenusht1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenusht1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennint1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:negativeInteger(\"-9999999999999999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennint1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:negativeInteger(\"-2970140759990967
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennint1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenpint1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenpint1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:positiveInteger(\"52704602390610033
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenpint1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:positiveInteger(\"99999999999999999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenulng1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenulng1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedLong(\"130747108607674654\"
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evenulng1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:unsignedLong(\"184467440737095516\"
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennpi1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonPositiveInteger(\"-9999999999999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennpi1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonPositiveInteger(\"-4756884372718
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennpi1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennni1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennni1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonNegativeInteger(\"30388454599146
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evennni1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:nonNegativeInteger(\"99999999999999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evensht1args_1() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evensht1args_2() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn fn_round_half_to_evensht1args_3() {
    //  Evaluates The 'round-half-to-even' function with the arguments set as follows:
    // XPath: fn:round-half-to-even(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k_roundevenfunc_3() {
    //  A test whose essence is: `empty(round-half-to-even(()))`.
    // XPath: empty(round-half-to-even(()))
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k_roundevenfunc_4() {
    //  A test whose essence is: `empty(round-half-to-even((), 3))`.
    // XPath: empty(round-half-to-even((), 3))
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_1() {
    //  Invoked fn:round-half-to-even() on 1.
    // XPath: round-half-to-even(1) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_2() {
    //  Invoked fn:round-half-to-even() on 1.0.
    // XPath: round-half-to-even(1.0) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_3() {
    //  Invoked fn:round-half-to-even() on 0.5.
    // XPath: round-half-to-even(0.5) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_4() {
    //  Invoked fn:round-half-to-even() on 1.5.
    // XPath: round-half-to-even(1.5) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_5() {
    //  Invoked fn:round-half-to-even() on 2.5.
    // XPath: round-half-to-even(2.5) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_6() {
    //  Invoked fn:round-half-to-even() on a large double.
    // XPath: round-half-to-even(3.567812E+3, 2) eq 3567.81E0
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_7() {
    //  Invoked fn:round-half-to-even() on a large double with 2 in precision.
    // XPath: round-half-to-even(4.7564E-3, 2) eq 0.0E0
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_8() {
    //  Invoked fn:round-half-to-even() on a large double with -2 in precision.
    // XPath: round-half-to-even(35612.25, -2) eq 35600
    // Expected: true
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_9() {
    //  Invoked fn:round-half-to-even() on xs:double -0.
    // XPath: round-half-to-even(xs:double(\"-0\"))
    // Expected: -0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_10() {
    //  Invoked fn:round-half-to-even() on xs:float -0.
    // XPath: round-half-to-even(xs:float(\"-0\"))
    // Expected: -0
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_11() {
    //  Invoked fn:round-half-to-even() on xs:double NaN.
    // XPath: round-half-to-even(xs:double(\"NaN\"))
    // Expected: NaN
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_12() {
    //  Invoked fn:round-half-to-even() on xs:float NaN.
    // XPath: round-half-to-even(xs:float(\"NaN\"))
    // Expected: NaN
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}

#[test]
fn k2_roundevenfunc_13() {
    //  Invoked fn:round-half-to-even() on xs:double -INF.
    // XPath: round-half-to-even(xs:double(\"-INF\"))
    // Expected: -INF
    // Calls stub function - will fail until fn:round-half-to-even is implemented
    let _ = stub_fnround_half_to_even();
}
