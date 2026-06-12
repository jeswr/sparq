//! Test chunk 0 for fn:empty
//! Contains 50 tests

use dep::xpath::stub_fnempty;

#[test]
fn fn_emptyint1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:int(\"-2147483648\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyint1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:int(\"-1873914410\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyint1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:int(\"2147483647\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyintg1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:integer(\"-999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyintg1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:integer(\"830993497117024304\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyintg1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:empty((xs:integer(\"999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydec1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:empty((xs:decimal(\"-999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydec1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:empty((xs:decimal(\"617375191608514839\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydec1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:empty((xs:decimal(\"999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydbl1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:empty((xs:double(\"-1.7976931348623157E308\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydbl1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:empty((xs:double(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptydbl1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:empty((xs:double(\"1.7976931348623157E308\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyflt1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:empty((xs:float(\"-3.4028235E38\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyflt1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:empty((xs:float(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyflt1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:empty((xs:float(\"3.4028235E38\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptylng1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:empty((xs:long(\"-92233720368547758\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptylng1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:empty((xs:long(\"-47175562203048468\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptylng1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:empty((xs:long(\"92233720368547758\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyusht1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedShort(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyusht1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedShort(\"44633\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyusht1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedShort(\"65535\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynint1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:empty((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynint1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:empty((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynint1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:empty((xs:negativeInteger(\"-1\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptypint1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:empty((xs:positiveInteger(\"1\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptypint1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:empty((xs:positiveInteger(\"52704602390610033\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptypint1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:empty((xs:positiveInteger(\"999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyulng1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedLong(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyulng1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedLong(\"130747108607674654\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptyulng1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:empty((xs:unsignedLong(\"184467440737095516\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynpi1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynpi1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynpi1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonPositiveInteger(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynni1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonNegativeInteger(\"0\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynni1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptynni1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:empty((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptysht1args_1() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:empty((xs:short(\"-32768\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptysht1args_2() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:empty((xs:short(\"-5324\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn fn_emptysht1args_3() {
    //  Evaluates The 'empty' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:empty((xs:short(\"32767\")))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_3() {
    //  A test whose essence is: `not(empty('string'))`.
    // XPath: not(empty(\"string\"))
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_4() {
    //  A test whose essence is: `not(empty((1, (), 'string')))`.
    // XPath: not(empty((1, (), \"string\")))
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_5() {
    //  A test whose essence is: `not(empty((1, 'string')))`.
    // XPath: not(empty((1, \"string\")))
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_6() {
    //  A test whose essence is: `not(empty( ((), 1, 'string') ))`.
    // XPath: not(empty( ((), 1, \"string\") ))
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_7() {
    //  A test whose essence is: `empty(())`.
    // XPath: empty(())
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_8() {
    //  A test whose essence is: `empty( ((), (), ()) )`.
    // XPath: empty( ((), (), ()) )
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn k_seqemptyfunc_9() {
    //  fn:empty combined with fn:remove and fn:not.
    // XPath: not(exists(remove(remove((current-time(), 1), 1), 1)))
    // Expected: true
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn cbcl_empty_func_001() {
    //  test fn:empty on a mix of values and expressions
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn cbcl_empty_func_002() {
    //  Test empty with foreach
    // XPath: empty(for $x in (1 to 10)[. mod 2 = 0] return \"blah\")
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn cbcl_empty_func_003() {
    //  Test empty with foreach
    // XPath: boolean(empty(for $x in (1 to 10)[. mod 2 = 0] return \"blah
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn cbcl_empty_func_004() {
    //  Tests empty on text node
    // XPath: empty(text {(1 to 10)[. mod 2 = 0]})
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}
