//! Test chunk 1 for fn:min
//! Contains 50 tests

use dep::xpath::stub_fnmin;

#[test]
fn fn_mindec2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:min((xs:decimal(\"617375191608514839\"),xs:decimal(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:min((xs:decimal(\"999999999999999999\"),xs:decimal(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:min((xs:decimal(\"-999999999999999999\"),xs:decimal(\"617
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindec2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:min((xs:decimal(\"-999999999999999999\"),xs:decimal(\"999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:min((xs:double(\"-1.7976931348623157E308\"),xs:double(\"-
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:min((xs:double(\"0\"),xs:double(\"-1.7976931348623157E308
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:min((xs:double(\"1.7976931348623157E308\"),xs:double(\"-1
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:min((xs:double(\"-1.7976931348623157E308\"),xs:double(\"0
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_mindbl2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:min((xs:double(\"-1.7976931348623157E308\"),xs:double(\"1
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:min((xs:float(\"-3.4028235E38\"),xs:float(\"-3.4028235E38
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:min((xs:float(\"0\"),xs:float(\"-3.4028235E38\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:min((xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E38\
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:min((xs:float(\"-3.4028235E38\"),xs:float(\"0\")))
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minflt2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:min((xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E38\
    // Expected: xs:float("-3.4028235E38")
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:min((xs:long(\"-92233720368547758\"),xs:long(\"-922337203
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:min((xs:long(\"-47175562203048468\"),xs:long(\"-922337203
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:min((xs:long(\"92233720368547758\"),xs:long(\"-9223372036
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:min((xs:long(\"-92233720368547758\"),xs:long(\"-471755622
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minlng2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:min((xs:long(\"-92233720368547758\"),xs:long(\"9223372036
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minusht2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:min((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:min((xs:negativeInteger(\"-297014075999096793\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:min((xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-999
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:min((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnint2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:min((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:min((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:min((xs:positiveInteger(\"52704602390610033\"),xs:positiv
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:min((xs:positiveInteger(\"999999999999999999\"),xs:positi
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:min((xs:positiveInteger(\"1\"),xs:positiveInteger(\"52704
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minpint2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:min((xs:positiveInteger(\"1\"),xs:positiveInteger(\"99999
    // Expected: 1
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedLong(\"130747108607674654\"),xs:unsignedL
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedLong(\"184467440737095516\"),xs:unsignedL
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedLong(\"0\"),xs:unsignedLong(\"13074710860
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minulng2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:min((xs:unsignedLong(\"0\"),xs:unsignedLong(\"18446744073
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:min((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:min((xs:nonPositiveInteger(\"-475688437271870490\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:min((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:min((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnpi2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:min((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:min((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni2args_2() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:min((xs:nonNegativeInteger(\"303884545991464527\"),xs:non
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni2args_3() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:min((xs:nonNegativeInteger(\"999999999999999999\"),xs:non
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni2args_4() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:min((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minnni2args_5() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:min((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}

#[test]
fn fn_minsht2args_1() {
    //  Evaluates The 'min' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:min((xs:short(\"-32768\"),xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:min is implemented
    let _ = stub_fnmin();
}
