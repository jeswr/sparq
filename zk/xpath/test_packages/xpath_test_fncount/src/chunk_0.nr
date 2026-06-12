//! Test chunk 0 for fn:count
//! Contains 50 tests

use dep::xpath::stub_fncount;

#[test]
fn fn_countint1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:int(\"-2147483648\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countint1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:int(\"-1873914410\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countint1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:int(\"2147483647\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countintg1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:integer(\"-999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countintg1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:integer(\"830993497117024304\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countintg1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:int
    // XPath: fn:count((xs:integer(\"999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdec1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:count((xs:decimal(\"-999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdec1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:count((xs:decimal(\"617375191608514839\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdec1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dec
    // XPath: fn:count((xs:decimal(\"999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdbl1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:count((xs:double(\"-1.7976931348623157E308\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdbl1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:count((xs:double(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countdbl1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:dou
    // XPath: fn:count((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countflt1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:count((xs:float(\"-3.4028235E38\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countflt1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:count((xs:float(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countflt1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:flo
    // XPath: fn:count((xs:float(\"3.4028235E38\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countlng1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:count((xs:long(\"-92233720368547758\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countlng1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:count((xs:long(\"-47175562203048468\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countlng1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:lon
    // XPath: fn:count((xs:long(\"92233720368547758\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countusht1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedShort(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countusht1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedShort(\"44633\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countusht1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedShort(\"65535\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnint1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:count((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnint1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:count((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnint1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:neg
    // XPath: fn:count((xs:negativeInteger(\"-1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countpint1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:count((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countpint1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:count((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countpint1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:pos
    // XPath: fn:count((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countulng1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedLong(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countulng1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countulng1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:uns
    // XPath: fn:count((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnpi1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnpi1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnpi1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonPositiveInteger(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnni1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonNegativeInteger(\"0\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnni1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countnni1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:non
    // XPath: fn:count((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countsht1args_1() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:count((xs:short(\"-32768\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countsht1args_2() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:count((xs:short(\"-5324\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn fn_countsht1args_3() {
    //  Evaluates The 'count' function with the arguments set as follows: $arg = xs:sho
    // XPath: fn:count((xs:short(\"32767\")))
    // Expected: 1
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_3() {
    //  A test whose essence is: `count( () ) eq 0`.
    // XPath: count( () ) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_4() {
    //  A test whose essence is: `count( (1, 2, 3) ) eq 3`.
    // XPath: count( (1, 2, 3) ) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_5() {
    //  A test whose essence is: `count( (1, 2, ()) ) eq 2`.
    // XPath: count( (1, 2, ()) ) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_6() {
    //  A test whose essence is: `count(()) eq 0`.
    // XPath: count(()) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_7() {
    //  A test whose essence is: `count(((((()))))) eq 0`.
    // XPath: count(((((()))))) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_8() {
    //  A test whose essence is: `count( (((), (), ()), (), (), (), ()) ) eq 0`.
    // XPath: count( (((), (), ()), (), (), (), ()) ) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_9() {
    //  A test whose essence is: `count( (1, 2, 3) ) eq 3`.
    // XPath: count( (1, 2, 3) ) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_10() {
    //  A test whose essence is: `count( ((), 'one', 2, 'three')) eq 3`.
    // XPath: count( ((), \"one\", 2, \"three\")) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_11() {
    //  A test whose essence is: `count( ('one', (2, 'three')) ) eq 3`.
    // XPath: count( (\"one\", (2, \"three\")) ) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_12() {
    //  A test whose essence is: `count((1, 2)) eq 2`.
    // XPath: count((1, 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}

#[test]
fn k_seqcountfunc_13() {
    //  A test whose essence is: `count((1, 2, 3, 'four')) eq 4`.
    // XPath: count((1, 2, 3, \"four\")) eq 4
    // Expected: true
    // Calls stub function - will fail until fn:count is implemented
    let _ = stub_fncount();
}
