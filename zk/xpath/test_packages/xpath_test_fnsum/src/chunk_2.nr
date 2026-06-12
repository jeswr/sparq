//! Test chunk 2 for fn:sum
//! Contains 50 tests

use dep::xpath::stub_fnsum;

#[test]
fn fn_sumintg3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:sum((xs:integer(\"-999999999999999999\"),xs:integer(\"999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:sum((xs:integer(\"0\"),xs:integer(\"0\"),xs:integer(\"830
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumintg3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:inte
    // XPath: fn:sum((xs:integer(\"-999999999999999999\"),xs:integer(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:sum((xs:decimal(\"999999999999999999\"),xs:decimal(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:sum((xs:decimal(\"-999999999999999999\"),xs:decimal(\"999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:sum((xs:decimal(\"0\"),xs:decimal(\"0\"),xs:decimal(\"617
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdec3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:sum((xs:decimal(\"-999999999999999999\"),xs:decimal(\"-99
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:sum((xs:double(\"1.7976931348623157E308\"),xs:double(\"-1
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:sum((xs:double(\"-1.7976931348623157E308\"),xs:double(\"0
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:sum((xs:double(\"-1.7976931348623157E308\"),xs:double(\"1
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumdbl3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:sum((xs:double(\"0\"),xs:double(\"0\"),xs:double(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:sum((xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E38\
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:sum((xs:float(\"-3.4028235E38\"),xs:float(\"0\"),xs:float
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:sum((xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E38\
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumflt3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:sum((xs:float(\"0\"),xs:float(\"0\"),xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"-47175562203048468\"),xs:long(\"-450581581
    // Expected: -137291878534047048
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"92233720368547758\"),xs:long(\"-9223372036
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"-45058158165499290\"),xs:long(\"-471755622
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"-92233720368547758\"),xs:long(\"9223372036
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"0\"),xs:long(\"0\"),xs:long(\"-47175562203
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumlng3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:sum((xs:long(\"-92233720368547758\"),xs:long(\"-922337203
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\"),xs:u
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"),
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"),
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"),
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"),
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\"),xs:u
    // Expected: 44633
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumusht3args_7() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\"),xs:u
    // Expected: 65535
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:sum((xs:negativeInteger(\"-297014075999096793\"),xs:negat
    // Expected: -1702985924000903205
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnint3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:sum((xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-999
    // Expected: -1999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"),
    // Expected: 3
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"52704602390610033\"),xs:positiv
    // Expected: 52704602390610035
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"999999999999999998\"),xs:positi
    // Expected: 1000000000000000000
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"52704
    // Expected: 52704602390610035
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"99999
    // Expected: 1000000000000000000
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"),
    // Expected: 52704602390610035
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumpint3args_7() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:sum((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"),
    // Expected: 1000000000000000000
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\"),xs:uns
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"130747108607674654\"),xs:unsignedL
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"184467440737095516\"),xs:unsignedL
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"13074710860
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"18446744073
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\"),xs:uns
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumulng3args_7() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:sum((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\"),xs:uns
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"-475688437271870490\"),xs:no
    // Expected: -1524311562728129508
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: -1999999999999999998
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"-524311562728129509\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnpi3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:sum((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}
