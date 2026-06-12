//! Test chunk 0 for fn:min
//! Contains 50 tests

use dep::xpath::stub_fnmin;

#[test]
fn fn_minint1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:min((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:int(m
    // XPath: fn:min((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:int(u
    // XPath: fn:min((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:min((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:min((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:min((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:min((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:min((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:min((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:min((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:min((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:min((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:min((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:min((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:min((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:min((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:min((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:min((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:min((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:min((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:min((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:min((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:min((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:min((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:min((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:min((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:min((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:min((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:min((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:min((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:min((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht1args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:min((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht1args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:min((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht1args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:min((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:min((xs:int(\"-2147483648\"),xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:min((xs:int(\"-1873914410\"),xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:min((xs:int(\"2147483647\"),xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:min((xs:int(\"-2147483648\"),xs:int(\"-1873914410\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minint2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:min((xs:int(\"-2147483648\"),xs:int(\"2147483647\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:min((xs:integer(\"-999999999999999999\"),xs:integer(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:min((xs:integer(\"830993497117024304\"),xs:integer(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:min((xs:integer(\"999999999999999999\"),xs:integer(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:min((xs:integer(\"-999999999999999999\"),xs:integer(\"830
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minintg2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:min((xs:integer(\"-999999999999999999\"),xs:integer(\"999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:min((xs:decimal(\"-999999999999999999\"),xs:decimal(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}
