//! Test chunk 0 for fn:zero-or-one
//! Contains 46 tests

use dep::xpath::stub_fnzero_or_one;

#[test]
fn fn_zero_or_oneint1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:int(\"-2147483648\"))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneint1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneint1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:int(\"2147483647\"))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneintg1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:integer(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneintg1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:integer(\"830993497117024304\"))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneintg1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:integer(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedec1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:decimal(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedec1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:decimal(\"617375191608514839\"))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedec1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:decimal(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedbl1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:double(\"-1.7976931348623157E308\"))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedbl1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onedbl1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:double(\"1.7976931348623157E308\"))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneflt1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:float(\"-3.4028235E38\"))
    // Expected: -3.4028235E38
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneflt1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneflt1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:float(\"3.4028235E38\"))
    // Expected: 3.4028235E38
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onelng1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:long(\"-92233720368547758\"))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onelng1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:long(\"-47175562203048468\"))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onelng1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:long(\"92233720368547758\"))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneusht1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneusht1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneusht1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenint1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenint1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:negativeInteger(\"-297014075999096793\"))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenint1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onepint1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onepint1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:positiveInteger(\"52704602390610033\"))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onepint1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:positiveInteger(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneulng1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneulng1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedLong(\"130747108607674654\"))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_oneulng1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:unsignedLong(\"184467440737095516\"))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenpi1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenpi1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonPositiveInteger(\"-475688437271870490\"
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenpi1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenni1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenni1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonNegativeInteger(\"303884545991464527\")
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onenni1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:nonNegativeInteger(\"999999999999999999\")
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onesht1args_1() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onesht1args_2() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn fn_zero_or_onesht1args_3() {
    //  Evaluates The 'zero-or-one' function with the arguments set as follows: $arg =
    // XPath: fn:zero-or-one(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn k_seqzerooronefunc_3() {
    //  A test whose essence is: `zero-or-one(true())`.
    // XPath: zero-or-one(true())
    // Expected: true
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn k_seqzerooronefunc_4() {
    //  A test whose essence is: `empty(zero-or-one(()))`.
    // XPath: empty(zero-or-one(()))
    // Expected: true
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn k_seqzerooronefunc_5() {
    //  A test whose essence is: `count(zero-or-one( 'one' )) eq 1`.
    // XPath: count(zero-or-one( \"one\" )) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn k_seqzerooronefunc_6() {
    //  A test whose essence is: `count(zero-or-one( () )) eq 0`.
    // XPath: count(zero-or-one( () )) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn cbcl_if_not_empty_001() {
    //  Test an optimization in if-not-empoty for empty return
    // XPath: for $x in zero-or-one((1 to 10)[. div 2 = 0]) return ()
    // Expected:
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn cbcl_zero_or_one_001() {
    //  test fn:zero-or-one on a count-preserving function
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: true
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}

#[test]
fn cbcl_zero_or_one_002() {
    //  test fn:zero-or-one on a sequence of one-or-more items
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 2
    // Calls stub function - will fail until fn:zero-or-one is implemented
    let _ = stub_fnzero_or_one();
}
