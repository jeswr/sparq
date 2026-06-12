//! Test chunk 1 for fn:sum
//! Contains 50 tests

use dep::xpath::stub_fnsum;

#[test]
fn fn_sumdec2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:decim
    // XPath: fn:sum((xs:decimal(\"-999999999999999999\"),xs:decimal(\"999
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"0\"),xs:double(\"-1.7976931348623157E308
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"1.7976931348623157E308\"),xs:double(\"-1
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"-1.7976931348623157E308\"),xs:double(\"0
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:doubl
    // XPath: fn:sum((xs:double(\"-1.7976931348623157E308\"),xs:double(\"1
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"0\"),xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E38\
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"-3.4028235E38\"),xs:float(\"0\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:float
    // XPath: fn:sum((xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E38\
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"-47175562203048468\"),xs:long(\"-450581581
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"92233720368547758\"),xs:long(\"-9223372036
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"-45058158165499290\"),xs:long(\"-471755622
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:long(
    // XPath: fn:sum((xs:long(\"-92233720368547758\"),xs:long(\"9223372036
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht2args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-297014075999096793\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-999
    // Expected: -1000000000000000000
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-702985924000903206\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:negat
    // XPath: fn:sum((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -1000000000000000000
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"))
    // Expected: 2
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"52704602390610033\"),xs:positiv
    // Expected: 52704602390610034
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"999999999999999998\"),xs:positi
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"52704
    // Expected: 52704602390610034
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint2args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:posit
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"99999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"130747108607674654\"),xs:unsignedL
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"184467440737095516\"),xs:unsignedL
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"13074710860
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng2args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:unsig
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"18446744073
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"-475688437271870490\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"-524311562728129509\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonPo
    // XPath: fn:sum((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"303884545991464527\"),xs:non
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"999999999999999999\"),xs:non
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni2args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:nonNe
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht2args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"-5324\"),xs:short(\"-27444\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht2args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"32767\"),xs:short(\"-32768\")))
    // Expected: -1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht2args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"-27444\"),xs:short(\"-5324\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht2args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg = xs:short
    // XPath: fn:sum((xs:short(\"-32768\"),xs:short(\"32767\")))
    // Expected: -1
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:sum((xs:int(\"-1873914410\"),xs:int(\"-273569238\"),xs:in
    // Expected: -2421052886
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:sum((xs:int(\"-273569238\"),xs:int(\"-1873914410\"),xs:in
    // Expected: -2147483648
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:sum((xs:int(\"0\"),xs:int(\"0\"),xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumint3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:int(
    // XPath: fn:sum((xs:int(\"0\"),xs:int(\"0\"),xs:int(\"2147483647\")))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:sum((xs:integer(\"999999999999999999\"),xs:integer(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}
