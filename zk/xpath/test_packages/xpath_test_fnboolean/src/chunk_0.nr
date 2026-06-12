//! Test chunk 0 for fn:boolean
//! Contains 50 tests

use dep::xpath::stub_fnboolean;

#[test]
fn fn_booleanint1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:int(\"-2147483648\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanint1args_2() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:int(\"-1873914410\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanint1args_3() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:int(\"2147483647\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanintg1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:integer(\"-999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanintg1args_2() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:integer(\"830993497117024304\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanintg1args_3() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:integer(\"999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandec1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:decimal(\"-999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandec1args_2() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:decimal(\"617375191608514839\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandec1args_3() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:decimal(\"999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandbl1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:double(\"-1.7976931348623157E308\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandbl1args_2() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:double(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleandbl1args_3() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:double(\"1.7976931348623157E308\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanflt1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:float(\"-3.4028235E38\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanflt1args_2() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:float(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanflt1args_3() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:float(\"3.4028235E38\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanlng1args_1() {
    // Evaluates The 'boolean' function with the arguments set as follows: $arg =
    // XPath: fn:boolean(xs:long(\"-92233720368547758\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanlng1args_2() {
    //  Test: booleanlng1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:long(\"-47175562203048468\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanlng1args_3() {
    //  Test: booleanlng1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:long(\"92233720368547758\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanusht1args_1() {
    //  Test: booleanusht1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedShort(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanusht1args_2() {
    //  Test: booleanusht1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedShort(\"44633\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanusht1args_3() {
    //  Test: booleanusht1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedShort(\"65535\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannint1args_1() {
    //  Test: booleannint1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannint1args_2() {
    //  Test: booleannint1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:negativeInteger(\"-297014075999096793\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannint1args_3() {
    //  Test: booleannint1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:negativeInteger(\"-1\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanpint1args_1() {
    //  Test: booleanpint1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:positiveInteger(\"1\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanpint1args_2() {
    //  Test: booleanpint1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:positiveInteger(\"52704602390610033\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanpint1args_3() {
    //  Test: booleanpint1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:positiveInteger(\"999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanulng1args_1() {
    //  Test: booleanulng1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedLong(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanulng1args_2() {
    //  Test: booleanulng1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedLong(\"130747108607674654\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleanulng1args_3() {
    //  Test: booleanulng1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:unsignedLong(\"184467440737095516\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannpi1args_1() {
    //  Test: booleannpi1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonPositiveInteger(\"-999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannpi1args_2() {
    //  Test: booleannpi1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonPositiveInteger(\"-475688437271870490\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannpi1args_3() {
    //  Test: booleannpi1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonPositiveInteger(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannni1args_1() {
    //  Test: booleannni1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonNegativeInteger(\"0\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannni1args_2() {
    //  Test: booleannni1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonNegativeInteger(\"303884545991464527\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleannni1args_3() {
    //  Test: booleannni1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:nonNegativeInteger(\"999999999999999999\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleansht1args_1() {
    //  Test: booleansht1args-1 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:short(\"-32768\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleansht1args_2() {
    //  Test: booleansht1args-2 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:short(\"-5324\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_booleansht1args_3() {
    //  Test: booleansht1args-3 Written By: Carmelo Montanez Date: Fri Dec 10 10:15:47
    // XPath: fn:boolean(xs:short(\"32767\"))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_001() {
    //  Test: fn-boolean-mixed-args-001.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(())
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_002() {
    //  Test: fn-boolean-mixed-args-002.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(\"\")
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_003() {
    //  Test: fn-boolean-mixed-args-003.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(false())
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_004() {
    //  Test: fn-boolean-mixed-args-004.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(true())
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_005() {
    //  Test: fn-boolean-mixed-args-005.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(xs:string(\"\"))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_006() {
    //  Test: fn-boolean-mixed-args-006.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(('a'))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_007() {
    //  Test: fn-boolean-mixed-args-007.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(xs:string('abc'))
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_008() {
    //  Test: fn-boolean-mixed-args-008.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(0)
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_009() {
    //  Test: fn-boolean-mixed-args-009.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(1)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_010() {
    //  Test: fn-boolean-mixed-args-010.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(-1)
    // Expected: true
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}

#[test]
fn fn_boolean_mixed_args_011() {
    //  Test: fn-boolean-mixed-args-011.xq Written By: Pulkita Tyagi Date: Mon May 23
    // XPath: fn:boolean(xs:float('NaN'))
    // Expected: false
    // Calls stub function - will fail until fn:boolean is implemented
    let _ = stub_fnboolean();
}
