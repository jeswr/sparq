//! Test chunk 0 for fn:data
//! Contains 42 tests

use dep::xpath::stub_fndata;

#[test]
fn fn_dataint1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:int(
    // XPath: fn:data((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataint1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:int(
    // XPath: fn:data((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataint1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:int(
    // XPath: fn:data((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataintg1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:inte
    // XPath: fn:data((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataintg1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:inte
    // XPath: fn:data((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataintg1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:inte
    // XPath: fn:data((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadec1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:deci
    // XPath: fn:data((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadec1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:deci
    // XPath: fn:data((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadec1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:deci
    // XPath: fn:data((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadbl1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:doub
    // XPath: fn:data((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadbl1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:doub
    // XPath: fn:data((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datadbl1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:doub
    // XPath: fn:data((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataflt1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:floa
    // XPath: fn:data((xs:float(\"-3.4028235E38\")))
    // Expected: -3.4028235E38
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataflt1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:floa
    // XPath: fn:data((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataflt1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:floa
    // XPath: fn:data((xs:float(\"3.4028235E38\")))
    // Expected: 3.4028235E38
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datalng1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:long
    // XPath: fn:data((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datalng1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:long
    // XPath: fn:data((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datalng1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:long
    // XPath: fn:data((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datausht1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datausht1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datausht1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanint1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nega
    // XPath: fn:data((xs:negativeInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanint1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nega
    // XPath: fn:data((xs:negativeInteger(\"-297014075999096793\")))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanint1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nega
    // XPath: fn:data((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datapint1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:posi
    // XPath: fn:data((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datapint1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:posi
    // XPath: fn:data((xs:positiveInteger(\"52704602390610033\")))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datapint1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:posi
    // XPath: fn:data((xs:positiveInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataulng1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataulng1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedLong(\"130747108607674654\")))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_dataulng1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:unsi
    // XPath: fn:data((xs:unsignedLong(\"184467440737095516\")))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanpi1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonP
    // XPath: fn:data((xs:nonPositiveInteger(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanpi1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonP
    // XPath: fn:data((xs:nonPositiveInteger(\"-475688437271870490\")))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanpi1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonP
    // XPath: fn:data((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanni1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonN
    // XPath: fn:data((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanni1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonN
    // XPath: fn:data((xs:nonNegativeInteger(\"303884545991464527\")))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datanni1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:nonN
    // XPath: fn:data((xs:nonNegativeInteger(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datasht1args_1() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:shor
    // XPath: fn:data((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datasht1args_2() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:shor
    // XPath: fn:data((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn fn_datasht1args_3() {
    //  Evaluates The 'data' function with the arguments set as follows: $arg = xs:shor
    // XPath: fn:data((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn k_datafunc_3() {
    //  A test whose essence is: `count(data((1, 2, 3, 4, 5))) eq 5`.
    // XPath: count(data((1, 2, 3, 4, 5))) eq 5
    // Expected: true
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn k_datafunc_4() {
    //  A test whose essence is: `empty(data( () ))`.
    // XPath: empty(data( () ))
    // Expected: true
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}

#[test]
fn k2_datafunc_1() {
    //  fn:data() allows zero arguments in XP30/XQ30.
    // XPath: 1!data()
    // Expected: 1
    // Calls stub function - will fail until fn:data is implemented
    let _ = stub_fndata();
}
