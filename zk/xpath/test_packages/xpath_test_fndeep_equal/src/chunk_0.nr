//! Test chunk 0 for fn:deep-equal
//! Contains 50 tests

use dep::xpath::stub_fndeep_equal;

#[test]
fn fn_deep_equalint2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:int(\"-2147483648\")),(xs:int(\"-214748364
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalint2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:int(\"-1873914410\")),(xs:int(\"-214748364
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalint2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:int(\"2147483647\")),(xs:int(\"-2147483648
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalint2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:int(\"-2147483648\")),(xs:int(\"-187391441
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalint2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:int(\"-2147483648\")),(xs:int(\"2147483647
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalintg2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:integer(\"-999999999999999999\")),(xs:inte
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalintg2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:integer(\"830993497117024304\")),(xs:integ
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalintg2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:integer(\"999999999999999999\")),(xs:integ
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalintg2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:integer(\"-999999999999999999\")),(xs:inte
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalintg2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:integer(\"-999999999999999999\")),(xs:inte
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldec2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:decimal(\"-999999999999999999\")),(xs:deci
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldec2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:decimal(\"617375191608514839\")),(xs:decim
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldec2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:decimal(\"999999999999999999\")),(xs:decim
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldec2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:decimal(\"-999999999999999999\")),(xs:deci
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldec2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:decimal(\"-999999999999999999\")),(xs:deci
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldbl2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:double(\"-1.7976931348623157E308\")),(xs:d
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldbl2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:double(\"0\")),(xs:double(\"-1.79769313486
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldbl2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:double(\"1.7976931348623157E308\")),(xs:do
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldbl2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:double(\"-1.7976931348623157E308\")),(xs:d
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equaldbl2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:double(\"-1.7976931348623157E308\")),(xs:d
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalflt2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:float(\"-3.4028235E38\")),(xs:float(\"-3.4
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalflt2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:float(\"0\")),(xs:float(\"-3.4028235E38\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalflt2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:float(\"3.4028235E38\")),(xs:float(\"-3.40
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalflt2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:float(\"-3.4028235E38\")),(xs:float(\"0\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalflt2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:float(\"-3.4028235E38\")),(xs:float(\"3.40
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equallng2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:long(\"-92233720368547758\")),(xs:long(\"-
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equallng2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:long(\"-47175562203048468\")),(xs:long(\"-
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equallng2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:long(\"92233720368547758\")),(xs:long(\"-9
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equallng2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:long(\"-92233720368547758\")),(xs:long(\"-
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equallng2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:long(\"-92233720368547758\")),(xs:long(\"9
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalusht2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedShort(\"0\")),(xs:unsignedShort(\"
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalusht2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedShort(\"44633\")),(xs:unsignedShor
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalusht2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedShort(\"65535\")),(xs:unsignedShor
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalusht2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedShort(\"0\")),(xs:unsignedShort(\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalusht2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedShort(\"0\")),(xs:unsignedShort(\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnint2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:negativeInteger(\"-999999999999999999\")),
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnint2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:negativeInteger(\"-297014075999096793\")),
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnint2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:negativeInteger(\"-1\")),(xs:negativeInteg
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnint2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:negativeInteger(\"-999999999999999999\")),
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnint2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:negativeInteger(\"-999999999999999999\")),
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalpint2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:positiveInteger(\"1\")),(xs:positiveIntege
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalpint2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:positiveInteger(\"52704602390610033\")),(x
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalpint2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:positiveInteger(\"999999999999999999\")),(
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalpint2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:positiveInteger(\"1\")),(xs:positiveIntege
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalpint2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:positiveInteger(\"1\")),(xs:positiveIntege
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalulng2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedLong(\"0\")),(xs:unsignedLong(\"0\
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalulng2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedLong(\"130747108607674654\")),(xs:
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalulng2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedLong(\"184467440737095516\")),(xs:
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalulng2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedLong(\"0\")),(xs:unsignedLong(\"13
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalulng2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:unsignedLong(\"0\")),(xs:unsignedLong(\"18
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}
