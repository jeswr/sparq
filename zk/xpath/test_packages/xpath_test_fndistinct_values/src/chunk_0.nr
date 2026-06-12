//! Test chunk 0 for fn:distinct-values
//! Contains 50 tests

use dep::xpath::stub_fndistinct_values;

#[test]
fn fn_distinct_valuesint1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:int(\"-2147483648\")))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesint1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:int(\"-1873914410\")))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesint1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesintg1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:integer(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesintg1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:integer(\"830993497117024304\")))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesintg1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:integer(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdec1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:decimal(\"-999999999999999999\")))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdec1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:decimal(\"617375191608514839\")))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdec1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:decimal(\"999999999999999999\")))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdbl1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:double(\"-1.7976931348623157E308\")))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdbl1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesdbl1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:double(\"1.7976931348623157E308\")))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesflt1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesflt1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesflt1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:float(\"3.4028235E38\")))
    // Expected: xs:float("3.4028235E38")
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valueslng1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:long(\"-92233720368547758\")))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valueslng1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:long(\"-47175562203048468\")))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valueslng1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:long(\"92233720368547758\")))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesusht1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesusht1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedShort(\"44633\")))
    // Expected: 44633
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesusht1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedShort(\"65535\")))
    // Expected: 65535
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnint1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:negativeInteger(\"-999999999999999999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnint1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:negativeInteger(\"-297014075999096793
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnint1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:negativeInteger(\"-1\")))
    // Expected: -1
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuespint1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:positiveInteger(\"1\")))
    // Expected: 1
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuespint1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:positiveInteger(\"52704602390610033\"
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuespint1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:positiveInteger(\"999999999999999999\
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesulng1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesulng1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedLong(\"130747108607674654\"))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesulng1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:unsignedLong(\"184467440737095516\"))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnpi1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonPositiveInteger(\"-999999999999999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnpi1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonPositiveInteger(\"-475688437271870
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnpi1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonPositiveInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnni1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonNegativeInteger(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnni1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonNegativeInteger(\"3038845459914645
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuesnni1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:nonNegativeInteger(\"9999999999999999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuessht1args_1() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuessht1args_2() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:short(\"-5324\")))
    // Expected: -5324
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_valuessht1args_3() {
    //  Evaluates The 'distinct-values' function with the arguments set as follows: $ar
    // XPath: fn:distinct-values((xs:short(\"32767\")))
    // Expected: 32767
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_002() {
    //  arg: sequence of integer
    // XPath: fn:distinct-values(( 1, (1), ((1)) ))
    // Expected: 1
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_004() {
    //  arg: sequence of integer
    // XPath: fn:distinct-values(( xs:integer(1), 1))
    // Expected: 1
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_007() {
    //  arg: sequence of string
    // XPath: fn:distinct-values(( xs:string(\"hello\"), \"hello\"))
    // Expected: hello
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_008() {
    //  arg: sequence of string
    // XPath: fn:distinct-values(( xs:string(\"\"), \"\", ''))
    // Expected:
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_012() {
    //  arg: sequence of decimal & float
    // XPath: fn:distinct-values((xs:decimal('1.2'), xs:float('1.2')))
    // Expected: 1.2
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_013() {
    //  arg: sequence of decimal & double
    // XPath: fn:distinct-values((xs:decimal('1.2'), xs:double('1.2')))
    // Expected: 1.2
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_014() {
    //  arg: sequence of float & string
    // XPath: fn:distinct-values((xs:float('NaN'), 'NaN'))
    // Expected: NaN NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_015() {
    //  arg: sequence of float & string
    // XPath: fn:distinct-values((xs:float('INF'), 'INF'))
    // Expected: INF INF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_016() {
    //  arg: sequence of float & string
    // XPath: fn:distinct-values((xs:float('-INF'), '-INF'))
    // Expected: -INF -INF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_017() {
    //  arg: sequence of float & float
    // XPath: fn:distinct-values((xs:float('INF'), xs:float('INF')))
    // Expected: INF
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}

#[test]
fn fn_distinct_values_mixed_args_019() {
    //  arg: sequence of float & float
    // XPath: fn:distinct-values((xs:float('NaN'), xs:float('NaN')))
    // Expected: NaN
    // Calls stub function - will fail until fn:distinct-values is implemented
    let _ = stub_fndistinct_values();
}
