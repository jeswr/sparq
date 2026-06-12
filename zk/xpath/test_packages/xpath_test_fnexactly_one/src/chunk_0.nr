//! Test chunk 0 for fn:exactly-one
//! Contains 45 tests

use dep::xpath::stub_fnexactly_one;

#[test]
fn fn_exactly_oneint1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:int(\"-2147483648\"))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneint1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneint1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:int(\"2147483647\"))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneintg1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:integer(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneintg1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:integer(\"830993497117024304\"))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneintg1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:integer(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedec1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:decimal(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedec1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:decimal(\"617375191608514839\"))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedec1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:decimal(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedbl1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:double(\"-1.7976931348623157E308\"))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedbl1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onedbl1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:double(\"1.7976931348623157E308\"))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneflt1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:float(\"-3.4028235E38\"))
    // Expected: -3.4028235E38
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneflt1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneflt1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:float(\"3.4028235E38\"))
    // Expected: 3.4028235E38
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onelng1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:long(\"-92233720368547758\"))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onelng1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:long(\"-47175562203048468\"))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onelng1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:long(\"92233720368547758\"))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneusht1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneusht1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneusht1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenint1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenint1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:negativeInteger(\"-297014075999096793\"))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenint1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onepint1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onepint1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:positiveInteger(\"52704602390610033\"))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onepint1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:positiveInteger(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneulng1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneulng1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedLong(\"130747108607674654\"))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_oneulng1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:unsignedLong(\"184467440737095516\"))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenpi1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenpi1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonPositiveInteger(\"-475688437271870490\"
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenpi1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenni1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenni1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonNegativeInteger(\"303884545991464527\")
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onenni1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:nonNegativeInteger(\"999999999999999999\")
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onesht1args_1() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onesht1args_2() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn fn_exactly_onesht1args_3() {
    //  Evaluates The 'exactly-one' function with the arguments set as follows: $arg =
    // XPath: fn:exactly-one(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn k_seqexactlyonefunc_4() {
    //  A test whose essence is: `exactly-one('one') eq 'one'`.
    // XPath: exactly-one(\"one\") eq \"one\"
    // Expected: true
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn k_seqexactlyonefunc_5() {
    //  A test whose essence is: `count(exactly-one( 'one' )) eq 1`.
    // XPath: count(exactly-one( \"one\" )) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn cbcl_exactly_one_001() {
    //  Test fn:boolean(fn:exactly-one).
    // XPath: boolean( exactly-one( remove( (<a/>, 1), 1 ) ) )
    // Expected: true
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn cbcl_exactly_one_002() {
    //  test fn:exactly-one on a count-preserving function
    // XPath: declare function local:generate($arg as xs:integer) { if ($a
    // Expected: 1
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn cbcl_exactly_one_003() {
    //  test fn:exactly-one on a sequence of one-or-more items
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 2
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}

#[test]
fn cbcl_exactly_one_004() {
    //  test fn:exactly-one on a sequence of zero-or-one items
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 2
    // Calls stub function - will fail until fn:exactly-one is implemented
    let _ = stub_fnexactly_one();
}
