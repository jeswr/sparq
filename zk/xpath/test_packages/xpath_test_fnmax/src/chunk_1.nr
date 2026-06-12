//! Test chunk 1 for fn:max
//! Contains 50 tests

use dep::xpath::stub_fnmax;

#[test]
fn fn_maxdec2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:max((xs:decimal(\"617375191608514839\"),xs:decimal(\"-999
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:max((xs:decimal(\"999999999999999999\"),xs:decimal(\"-999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:max((xs:decimal(\"-999999999999999999\"),xs:decimal(\"617
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdec2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:max((xs:decimal(\"-999999999999999999\"),xs:decimal(\"999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:max((xs:double(\"-1.7976931348623157E308\"),xs:double(\"-
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:max((xs:double(\"0\"),xs:double(\"-1.7976931348623157E308
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:max((xs:double(\"1.7976931348623157E308\"),xs:double(\"-1
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:max((xs:double(\"-1.7976931348623157E308\"),xs:double(\"0
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxdbl2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:max((xs:double(\"-1.7976931348623157E308\"),xs:double(\"1
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:max((xs:float(\"-3.4028235E38\"),xs:float(\"-3.4028235E38
    // Expected: xs:float(-3.4028235E38)
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:max((xs:float(\"0\"),xs:float(\"-3.4028235E38\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:max((xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E38\
    // Expected: xs:float(3.4028235E38)
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:max((xs:float(\"-3.4028235E38\"),xs:float(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxflt2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:max((xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E38\
    // Expected: xs:float(3.4028235E38)
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:max((xs:long(\"-92233720368547758\"),xs:long(\"-922337203
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:max((xs:long(\"-47175562203048468\"),xs:long(\"-922337203
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:max((xs:long(\"92233720368547758\"),xs:long(\"-9223372036
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:max((xs:long(\"-92233720368547758\"),xs:long(\"-471755622
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxlng2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:max((xs:long(\"-92233720368547758\"),xs:long(\"9223372036
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxusht2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:max((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:max((xs:negativeInteger(\"-297014075999096793\"),xs:negat
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:max((xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-999
    // Expected: -1
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:max((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnint2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:max((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -1
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:max((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:max((xs:positiveInteger(\"52704602390610033\"),xs:positiv
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:max((xs:positiveInteger(\"999999999999999999\"),xs:positi
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:max((xs:positiveInteger(\"1\"),xs:positiveInteger(\"52704
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxpint2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:max((xs:positiveInteger(\"1\"),xs:positiveInteger(\"99999
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedLong(\"130747108607674654\"),xs:unsignedL
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedLong(\"184467440737095516\"),xs:unsignedL
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedLong(\"0\"),xs:unsignedLong(\"13074710860
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxulng2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:max((xs:unsignedLong(\"0\"),xs:unsignedLong(\"18446744073
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:max((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:max((xs:nonPositiveInteger(\"-475688437271870490\"),xs:no
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:max((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:max((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnpi2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:max((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:max((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni2args_2() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:max((xs:nonNegativeInteger(\"303884545991464527\"),xs:non
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni2args_3() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:max((xs:nonNegativeInteger(\"999999999999999999\"),xs:non
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni2args_4() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:max((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxnni2args_5() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:max((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}

#[test]
fn fn_maxsht2args_1() {
    //  Evaluates The 'max' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:max((xs:short(\"-32768\"),xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:max is implemented
    let _ = stub_fnmax();
}
