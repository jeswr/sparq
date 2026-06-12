//! Test chunk 0 for fn:max
//! Contains 50 tests

use dep::xpath::stub_fnmax;

#[test]
fn fn_maxint1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:max((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:int(m
    // XPath: fn:max((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:int(u
    // XPath: fn:max((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:max((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:max((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:max((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:max((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:max((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:max((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:max((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:max((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:max((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:max((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:max((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:max((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:max((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:max((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:max((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:max((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:max((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:max((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:max((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:max((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:max((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:max((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:max((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:max((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:max((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:max((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:max((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:max((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht1args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:max((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht1args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:max((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht1args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:max((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:max((xs:int(\"-2147483648\"),xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:max((xs:int(\"-1873914410\"),xs:int(\"-2147483648\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:max((xs:int(\"2147483647\"),xs:int(\"-2147483648\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:max((xs:int(\"-2147483648\"),xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxint2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:max((xs:int(\"-2147483648\"),xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:max((xs:integer(\"-999999999999999999\"),xs:integer(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:max((xs:integer(\"830993497117024304\"),xs:integer(\"-999
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:max((xs:integer(\"999999999999999999\"),xs:integer(\"-999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:max((xs:integer(\"-999999999999999999\"),xs:integer(\"830
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxintg2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:max((xs:integer(\"-999999999999999999\"),xs:integer(\"999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:max((xs:decimal(\"-999999999999999999\"),xs:decimal(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}
