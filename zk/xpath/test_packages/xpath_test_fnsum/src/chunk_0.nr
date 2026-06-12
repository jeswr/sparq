//! Test chunk 0 for fn:sum
//! Contains 50 tests

use dep::xpath::stub_fnsum;

#[test]
fn fn_sumint1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:sum((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(m
    // XPath: fn:sum((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(u
    // XPath: fn:sum((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht1args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht1args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht1args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(m
    // XPath: fn:sum((xs:int(\"-1873914410\"),xs:int(\"-273569238\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(u
    // XPath: fn:sum((xs:int(\"2147483647\"),xs:int(\"-2147483648\")))
    // Expected: -1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:sum((xs:int(\"-273569238\"),xs:int(\"-1873914410\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:int(l
    // XPath: fn:sum((xs:int(\"-2147483648\"),xs:int(\"2147483647\")))
    // Expected: -1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"830993497117024304\"),xs:integer(\"-999
    // Expected: -169006502882975695
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"999999999999999999\"),xs:integer(\"-999
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"-999999999999999999\"),xs:integer(\"830
    // Expected: -169006502882975695
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:integ
    // XPath: fn:sum((xs:integer(\"-999999999999999999\"),xs:integer(\"999
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"617375191608514839\"),xs:decimal(\"-999
    // Expected: -382624808391485160
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"999999999999999999\"),xs:decimal(\"-999
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"-999999999999999999\"),xs:decimal(\"617
    // Expected: -382624808391485160
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}
