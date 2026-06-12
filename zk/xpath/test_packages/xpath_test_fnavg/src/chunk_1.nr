//! Test chunk 1 for fn:avg
//! Contains 50 tests

use dep::xpath::stub_fnavg;

#[test]
fn fn_avgdec2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:avg((xs:decimal(\"617375191608514839\"),xs:decimal(\"-999
    // Expected: -191312404195742580
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:avg((xs:decimal(\"999999999999999999\"),xs:decimal(\"-999
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:avg((xs:decimal(\"-999999999999999999\"),xs:decimal(\"617
    // Expected: -191312404195742580
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdec2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:deci
    // XPath: fn:avg((xs:decimal(\"-999999999999999999\"),xs:decimal(\"999
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:avg((xs:double(\"-1.7976931348623157E150\"),xs:double(\"-
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:avg((xs:double(\"0\"),xs:double(\"-1.7976931348623157E308
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:avg((xs:double(\"1.7976931348623157E308\"),xs:double(\"-1
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:avg((xs:double(\"-1.7976931348623157E308\"),xs:double(\"0
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgdbl2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:doub
    // XPath: fn:avg((xs:double(\"-1.7976931348623157E308\"),xs:double(\"1
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:avg((xs:float(\"0\"),xs:float(\"-3.4028235E38\")))
    // Expected: xs:float('-1.7014117E38')
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:avg((xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E38\
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:avg((xs:float(\"-3.4028235E38\"),xs:float(\"0\")))
    // Expected: xs:float('-1.7014117E38')
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgflt2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:floa
    // XPath: fn:avg((xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E38\
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:avg((xs:long(\"-92233720368547758\"),xs:long(\"-922337203
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:avg((xs:long(\"-47175562203048468\"),xs:long(\"-922337203
    // Expected: -69704641285798113
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:avg((xs:long(\"92233720368547758\"),xs:long(\"-9223372036
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:avg((xs:long(\"-92233720368547758\"),xs:long(\"-471755622
    // Expected: -69704641285798113
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avglng2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:long
    // XPath: fn:avg((xs:long(\"-92233720368547758\"),xs:long(\"9223372036
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"))
    // Expected: 22316.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"))
    // Expected: 32767.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"))
    // Expected: 22316.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgusht2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"))
    // Expected: 32767.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:avg((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:avg((xs:negativeInteger(\"-297014075999096793\"),xs:negat
    // Expected: -648507037999548396
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:avg((xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-999
    // Expected: -500000000000000000
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:avg((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -648507037999548396
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnint2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nega
    // XPath: fn:avg((xs:negativeInteger(\"-999999999999999999\"),xs:negat
    // Expected: -500000000000000000
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:avg((xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:avg((xs:positiveInteger(\"52704602390610033\"),xs:positiv
    // Expected: 26352301195305017
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:avg((xs:positiveInteger(\"999999999999999999\"),xs:positi
    // Expected: 500000000000000000
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:avg((xs:positiveInteger(\"1\"),xs:positiveInteger(\"52704
    // Expected: 26352301195305017
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgpint2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:posi
    // XPath: fn:avg((xs:positiveInteger(\"1\"),xs:positiveInteger(\"99999
    // Expected: 500000000000000000
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\")))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedLong(\"130747108607674654\"),xs:unsignedL
    // Expected: 65373554303837327
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedLong(\"184467440737095516\"),xs:unsignedL
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedLong(\"0\"),xs:unsignedLong(\"13074710860
    // Expected: 65373554303837327
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgulng2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:unsi
    // XPath: fn:avg((xs:unsignedLong(\"0\"),xs:unsignedLong(\"18446744073
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:avg((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:avg((xs:nonPositiveInteger(\"-475688437271870490\"),xs:no
    // Expected: -737844218635935244.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:avg((xs:nonPositiveInteger(\"0\"),xs:nonPositiveInteger(\
    // Expected: -499999999999999999.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:avg((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -737844218635935244.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnpi2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonP
    // XPath: fn:avg((xs:nonPositiveInteger(\"-999999999999999999\"),xs:no
    // Expected: -499999999999999999.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:avg((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:avg((xs:nonNegativeInteger(\"303884545991464527\"),xs:non
    // Expected: 151942272995732263.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:avg((xs:nonNegativeInteger(\"999999999999999999\"),xs:non
    // Expected: 499999999999999999.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:avg((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 151942272995732263.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgnni2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:avg((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 499999999999999999.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht2args_1() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:avg((xs:short(\"-32768\"),xs:short(\"-32768\")))
    // Expected: -32768
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht2args_2() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:avg((xs:short(\"-5324\"),xs:short(\"-32768\")))
    // Expected: -19046
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}
