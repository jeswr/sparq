//! Test chunk 0 for fn:exists
//! Contains 50 tests

use dep::xpath::stub_fnexists;

#[test]
fn fn_existsint1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:int(\"-2147483648\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsint1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:int(\"-1873914410\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsint1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:int(\"2147483647\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsintg1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:integer(\"-999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsintg1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:integer(\"830993497117024304\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsintg1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:in
    // XPath: fn:exists((xs:integer(\"999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdec1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:exists((xs:decimal(\"-999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdec1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:exists((xs:decimal(\"617375191608514839\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdec1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:de
    // XPath: fn:exists((xs:decimal(\"999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdbl1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:exists((xs:double(\"-1.7976931348623157E308\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdbl1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:exists((xs:double(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsdbl1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:do
    // XPath: fn:exists((xs:double(\"1.7976931348623157E308\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsflt1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:fl
    // XPath: fn:exists((xs:float(\"-3.4028235E38\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsflt1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:fl
    // XPath: fn:exists((xs:float(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsflt1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:fl
    // XPath: fn:exists((xs:float(\"3.4028235E38\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existslng1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:exists((xs:long(\"-92233720368547758\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existslng1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:exists((xs:long(\"-47175562203048468\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existslng1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:lo
    // XPath: fn:exists((xs:long(\"92233720368547758\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsusht1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedShort(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsusht1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedShort(\"44633\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsusht1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedShort(\"65535\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnint1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:exists((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnint1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:exists((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnint1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:ne
    // XPath: fn:exists((xs:negativeInteger(\"-1\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existspint1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:exists((xs:positiveInteger(\"1\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existspint1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:exists((xs:positiveInteger(\"52704602390610033\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existspint1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:po
    // XPath: fn:exists((xs:positiveInteger(\"999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsulng1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedLong(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsulng1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedLong(\"130747108607674654\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsulng1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:un
    // XPath: fn:exists((xs:unsignedLong(\"184467440737095516\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnpi1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnpi1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnpi1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonPositiveInteger(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnni1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonNegativeInteger(\"0\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnni1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existsnni1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:no
    // XPath: fn:exists((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existssht1args_1() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:exists((xs:short(\"-32768\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existssht1args_2() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:exists((xs:short(\"-5324\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn fn_existssht1args_3() {
    //  Evaluates The 'exists' function with the arguments set as follows: $arg = xs:sh
    // XPath: fn:exists((xs:short(\"32767\")))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_3() {
    //  A test whose essence is: `not(exists(()))`.
    // XPath: not(exists(()))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_4() {
    //  A test whose essence is: `exists(1)`.
    // XPath: exists(1)
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_5() {
    //  A test whose essence is: `exists((1))`.
    // XPath: exists((1))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_6() {
    //  A test whose essence is: `exists((1, 2, 3))`.
    // XPath: exists((1, 2, 3))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_7() {
    //  A test whose essence is: `not(exists(()))`.
    // XPath: not(exists(()))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_8() {
    //  A test whose essence is: `not(exists( ((), (), (), ()) ))`.
    // XPath: not(exists( ((), (), (), ()) ))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_9() {
    //  A test whose essence is: `exists(reverse((1, 2, 3)))`.
    // XPath: exists(reverse((1, 2, 3)))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_10() {
    //  The Dynamic Context property 'current dateTime' must have an explicit timezone
    // XPath: exists(timezone-from-dateTime(current-dateTime()))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_11() {
    //  The Dynamic Context property 'current dateTime' must have an explicit timezone
    // XPath: exists(timezone-from-date(current-date()))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_12() {
    //  The Dynamic Context property 'current dateTime' must have an explicit timezone
    // XPath: exists(timezone-from-time(current-time()))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn k_seqexistsfunc_13() {
    //  fn:empty combined with fn:remove.
    // XPath: empty(remove(current-time(), 1))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}
