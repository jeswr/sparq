//! Test chunk 0 for fn:avg
//! Contains 50 tests

use dep::xpath::stub_fnavg;

#[test]
fn fn_avgint1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:avg((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:int(m
    // XPath: fn:avg((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:int(u
    // XPath: fn:avg((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:avg((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:avg((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:avg((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:avg((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:avg((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:avg((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:avg((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:avg((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:avg((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:avg((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float('-3.4028235E38')
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:avg((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:avg((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:avg((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:avg((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:avg((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:avg((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:avg((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:avg((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:avg((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:avg((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:avg((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:avg((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:avg((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:avg((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:avg((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:avg((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:avg((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:avg((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht1args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:avg((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht1args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:avg((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht1args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:avg((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:avg((xs:int(\"-2147483648\"),xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:avg((xs:int(\"-1873914410\"),xs:int(\"-2147483648\")))
    // Expected: -2010699029
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:avg((xs:int(\"2147483647\"),xs:int(\"-2147483648\")))
    // Expected: -0.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:avg((xs:int(\"-2147483648\"),xs:int(\"-1873914410\")))
    // Expected: -2010699029
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgint2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:avg((xs:int(\"-2147483648\"),xs:int(\"2147483647\")))
    // Expected: -0.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:avg((xs:integer(\"-999999999999999999\"),xs:integer(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:avg((xs:integer(\"830993497117024304\"),xs:integer(\"-999
    // Expected: -84503251441487847.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:avg((xs:integer(\"999999999999999999\"),xs:integer(\"-999
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:avg((xs:integer(\"-999999999999999999\"),xs:integer(\"830
    // Expected: -84503251441487847.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgintg2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:avg((xs:integer(\"-999999999999999999\"),xs:integer(\"999
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:avg((xs:decimal(\"-999999999999999999\"),xs:decimal(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}
