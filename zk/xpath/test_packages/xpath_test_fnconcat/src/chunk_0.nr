//! Test chunk 0 for fn:concat
//! Contains 50 tests

use dep::xpath::stub_fnconcat;

#[test]
fn fn_concatint2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:int(\"-2147483648\"),xs:int(\"-2147483648\"))
    // Expected: -2147483648-2147483648
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatint2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:int(\"-1873914410\"),xs:int(\"-2147483648\"))
    // Expected: -1873914410-2147483648
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatint2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:int(\"2147483647\"),xs:int(\"-2147483648\"))
    // Expected: 2147483647-2147483648
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatint2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:int(\"-2147483648\"),xs:int(\"-1873914410\"))
    // Expected: -2147483648-1873914410
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatint2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:int(\"-2147483648\"),xs:int(\"2147483647\"))
    // Expected: -21474836482147483647
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatintg2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:integer(\"-999999999999999999\"),xs:integer(\"-
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatintg2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:integer(\"830993497117024304\"),xs:integer(\"-9
    // Expected: 830993497117024304-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatintg2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:integer(\"999999999999999999\"),xs:integer(\"-9
    // Expected: 999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatintg2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:integer(\"-999999999999999999\"),xs:integer(\"8
    // Expected: -999999999999999999830993497117024304
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatintg2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:i
    // XPath: fn:concat(xs:integer(\"-999999999999999999\"),xs:integer(\"9
    // Expected: -999999999999999999999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdec2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:decimal(\"-999999999999999999\"),xs:decimal(\"-
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdec2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:decimal(\"617375191608514839\"),xs:decimal(\"-9
    // Expected: 617375191608514839-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdec2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:decimal(\"999999999999999999\"),xs:decimal(\"-9
    // Expected: 999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdec2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:decimal(\"-999999999999999999\"),xs:decimal(\"6
    // Expected: -999999999999999999617375191608514839
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdec2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:decimal(\"-999999999999999999\"),xs:decimal(\"9
    // Expected: -999999999999999999999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdbl2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:double(\"-1.7976931348623157E308\"),xs:double(\
    // Expected: -1.7976931348623157E308-1.79769313486231
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdbl2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:double(\"0\"),xs:double(\"-1.7976931348623157E3
    // Expected: 0-1.7976931348623157E308
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdbl2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:double(\"1.7976931348623157E308\"),xs:double(\"
    // Expected: 1.7976931348623157E308-1.797693134862315
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdbl2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:double(\"-1.7976931348623157E308\"),xs:double(\
    // Expected: -1.7976931348623157E3080
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatdbl2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:d
    // XPath: fn:concat(xs:double(\"-1.7976931348623157E308\"),xs:double(\
    // Expected: -1.7976931348623157E3081.797693134862315
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatflt2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:f
    // XPath: fn:concat(xs:float(\"-3.4028235E38\"),xs:float(\"-3.4028235E
    // Expected: -3.4028235E38-3.4028235E38
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatflt2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:f
    // XPath: fn:concat(xs:float(\"0\"),xs:float(\"-3.4028235E38\"))
    // Expected: 0-3.4028235E38
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatflt2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:f
    // XPath: fn:concat(xs:float(\"3.4028235E38\"),xs:float(\"-3.4028235E3
    // Expected: 3.4028235E38-3.4028235E38
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatflt2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:f
    // XPath: fn:concat(xs:float(\"-3.4028235E38\"),xs:float(\"0\"))
    // Expected: -3.4028235E380
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatflt2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:f
    // XPath: fn:concat(xs:float(\"-3.4028235E38\"),xs:float(\"3.4028235E3
    // Expected: -3.4028235E383.4028235E38
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatlng2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:l
    // XPath: fn:concat(xs:long(\"-92233720368547758\"),xs:long(\"-9223372
    // Expected: -92233720368547758-92233720368547758
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatlng2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:l
    // XPath: fn:concat(xs:long(\"-47175562203048468\"),xs:long(\"-9223372
    // Expected: -47175562203048468-92233720368547758
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatlng2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:l
    // XPath: fn:concat(xs:long(\"92233720368547758\"),xs:long(\"-92233720
    // Expected: 92233720368547758-92233720368547758
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatlng2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:l
    // XPath: fn:concat(xs:long(\"-92233720368547758\"),xs:long(\"-4717556
    // Expected: -92233720368547758-47175562203048468
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatlng2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:l
    // XPath: fn:concat(xs:long(\"-92233720368547758\"),xs:long(\"92233720
    // Expected: -9223372036854775892233720368547758
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatusht2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedShort(\"0\"),xs:unsignedShort(\"0\"))
    // Expected: 00
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatusht2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedShort(\"44633\"),xs:unsignedShort(\"0\"
    // Expected: 446330
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatusht2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedShort(\"65535\"),xs:unsignedShort(\"0\"
    // Expected: 655350
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatusht2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedShort(\"0\"),xs:unsignedShort(\"44633\"
    // Expected: 044633
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatusht2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedShort(\"0\"),xs:unsignedShort(\"65535\"
    // Expected: 065535
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnint2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:negativeInteger(\"-999999999999999999\"),xs:neg
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnint2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:negativeInteger(\"-297014075999096793\"),xs:neg
    // Expected: -297014075999096793-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnint2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:negativeInteger(\"-1\"),xs:negativeInteger(\"-9
    // Expected: -1-999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnint2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:negativeInteger(\"-999999999999999999\"),xs:neg
    // Expected: -999999999999999999-297014075999096793
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatnint2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:n
    // XPath: fn:concat(xs:negativeInteger(\"-999999999999999999\"),xs:neg
    // Expected: -999999999999999999-1
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatpint2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:p
    // XPath: fn:concat(xs:positiveInteger(\"1\"),xs:positiveInteger(\"1\"
    // Expected: 11
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatpint2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:p
    // XPath: fn:concat(xs:positiveInteger(\"52704602390610033\"),xs:posit
    // Expected: 527046023906100331
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatpint2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:p
    // XPath: fn:concat(xs:positiveInteger(\"999999999999999999\"),xs:posi
    // Expected: 9999999999999999991
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatpint2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:p
    // XPath: fn:concat(xs:positiveInteger(\"1\"),xs:positiveInteger(\"527
    // Expected: 152704602390610033
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatpint2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:p
    // XPath: fn:concat(xs:positiveInteger(\"1\"),xs:positiveInteger(\"999
    // Expected: 1999999999999999999
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatulng2args_1() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedLong(\"0\"),xs:unsignedLong(\"0\"))
    // Expected: 00
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatulng2args_2() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedLong(\"130747108607674654\"),xs:unsigne
    // Expected: 1307471086076746540
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatulng2args_3() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedLong(\"184467440737095516\"),xs:unsigne
    // Expected: 1844674407370955160
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatulng2args_4() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedLong(\"0\"),xs:unsignedLong(\"130747108
    // Expected: 0130747108607674654
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}

#[test]
fn fn_concatulng2args_5() {
    //  Evaluates The 'concat' function with the arguments set as follows: $arg1 = xs:u
    // XPath: fn:concat(xs:unsignedLong(\"0\"),xs:unsignedLong(\"184467440
    // Expected: 0184467440737095516
    // Calls stub function - will fail until fn:concat is implemented
    let _ = stub_fnconcat();
}
