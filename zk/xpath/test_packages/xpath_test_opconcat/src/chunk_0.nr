//! Test chunk 0 for op:concat
//! Contains 50 tests

use dep::xpath::stub_opconcat;

#[test]
fn op_concatint2args_1() {
    //  ******************************************************* Test: concatint2args-1
    // XPath: xs:int(\"-2147483648\")||xs:int(\"-2147483648\")
    // Expected: -2147483648-2147483648
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatint2args_2() {
    //  ******************************************************* Test: concatint2args-2
    // XPath: xs:int(\"-1873914410\")||xs:int(\"-2147483648\")
    // Expected: -1873914410-2147483648
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatint2args_3() {
    //  ******************************************************* Test: concatint2args-3
    // XPath: xs:int(\"2147483647\")||xs:int(\"-2147483648\")
    // Expected: 2147483647-2147483648
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatint2args_4() {
    //  ******************************************************* Test: concatint2args-4
    // XPath: (xs:int(\"-2147483648\")||xs:int(\"-1873914410\"))
    // Expected: -2147483648-1873914410
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatint2args_5() {
    //  ******************************************************* Test: concatint2args-5
    // XPath: (xs:int(\"-2147483648\")||xs:int(\"2147483647\"))
    // Expected: -21474836482147483647
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatintg2args_1() {
    //  ******************************************************* Test: concatintg2args-1
    // XPath: (xs:integer(\"-999999999999999999\")||xs:integer(\"-99999999
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatintg2args_2() {
    //  ******************************************************* Test: concatintg2args-2
    // XPath: (xs:integer(\"830993497117024304\")||xs:integer(\"-999999999
    // Expected: 830993497117024304-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatintg2args_3() {
    //  ******************************************************* Test: concatintg2args-3
    // XPath: (xs:integer(\"999999999999999999\")||xs:integer(\"-999999999
    // Expected: 999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatintg2args_4() {
    //  ******************************************************* Test: concatintg2args-4
    // XPath: (xs:integer(\"-999999999999999999\")||xs:integer(\"830993497
    // Expected: -999999999999999999830993497117024304
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatintg2args_5() {
    //  ******************************************************* Test: concatintg2args-5
    // XPath: (xs:integer(\"-999999999999999999\")||xs:integer(\"999999999
    // Expected: -999999999999999999999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdec2args_1() {
    //  ******************************************************* Test: concatdec2args-1
    // XPath: (xs:decimal(\"-999999999999999999\")||xs:decimal(\"-99999999
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdec2args_2() {
    //  ******************************************************* Test: concatdec2args-2
    // XPath: (xs:decimal(\"617375191608514839\")||xs:decimal(\"-999999999
    // Expected: 617375191608514839-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdec2args_3() {
    //  ******************************************************* Test: concatdec2args-3
    // XPath: (xs:decimal(\"999999999999999999\")||xs:decimal(\"-999999999
    // Expected: 999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdec2args_4() {
    //  ******************************************************* Test: concatdec2args-4
    // XPath: (xs:decimal(\"-999999999999999999\")||xs:decimal(\"617375191
    // Expected: -999999999999999999617375191608514839
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdec2args_5() {
    //  ******************************************************* Test: concatdec2args-5
    // XPath: (xs:decimal(\"-999999999999999999\")||xs:decimal(\"999999999
    // Expected: -999999999999999999999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdbl2args_1() {
    //  ******************************************************* Test: concatdbl2args-1
    // XPath: (xs:double(\"-1.7976931348623157E308\")||xs:double(\"-1.7976
    // Expected: -1.7976931348623157E308-1.79769313486231
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdbl2args_2() {
    //  ******************************************************* Test: concatdbl2args-2
    // XPath: (xs:double(\"0\")||xs:double(\"-1.7976931348623157E308\"))
    // Expected: 0-1.7976931348623157E308
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdbl2args_3() {
    //  ******************************************************* Test: concatdbl2args-3
    // XPath: (xs:double(\"1.7976931348623157E308\")||xs:double(\"-1.79769
    // Expected: 1.7976931348623157E308-1.797693134862315
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdbl2args_4() {
    //  ******************************************************* Test: concatdbl2args-4
    // XPath: (xs:double(\"-1.7976931348623157E308\") || xs:double(\"0\"))
    // Expected: -1.7976931348623157E3080
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatdbl2args_5() {
    //  ******************************************************* Test: concatdbl2args-5
    // XPath: (xs:double(\"-1.7976931348623157E308\") || xs:double(\"1.797
    // Expected: -1.7976931348623157E3081.797693134862315
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatflt2args_1() {
    //  ******************************************************* Test: concatflt2args-1
    // XPath: (xs:float(\"-3.4028235E38\") || xs:float(\"-3.4028235E38\"))
    // Expected: -3.4028235E38-3.4028235E38
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatflt2args_2() {
    //  ******************************************************* Test: concatflt2args-2
    // XPath: (xs:float(\"0\") || xs:float(\"-3.4028235E38\"))
    // Expected: 0-3.4028235E38
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatflt2args_3() {
    //  ******************************************************* Test: concatflt2args-3
    // XPath: (xs:float(\"3.4028235E38\") || xs:float(\"-3.4028235E38\"))
    // Expected: 3.4028235E38-3.4028235E38
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatflt2args_4() {
    //  ******************************************************* Test: concatflt2args-4
    // XPath: (xs:float(\"-3.4028235E38\") || xs:float(\"0\"))
    // Expected: -3.4028235E380
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatflt2args_5() {
    //  ******************************************************* Test: concatflt2args-5
    // XPath: (xs:float(\"-3.4028235E38\") || xs:float(\"3.4028235E38\"))
    // Expected: -3.4028235E383.4028235E38
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatlng2args_1() {
    //  ******************************************************* Test: concatlng2args-1
    // XPath: (xs:long(\"-92233720368547758\") || xs:long(\"-9223372036854
    // Expected: -92233720368547758-92233720368547758
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatlng2args_2() {
    //  ******************************************************* Test: concatlng2args-2
    // XPath: (xs:long(\"-47175562203048468\") || xs:long(\"-9223372036854
    // Expected: -47175562203048468-92233720368547758
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatlng2args_3() {
    //  ******************************************************* Test: concatlng2args-3
    // XPath: (xs:long(\"92233720368547758\") || xs:long(\"-92233720368547
    // Expected: 92233720368547758-92233720368547758
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatlng2args_4() {
    //  ******************************************************* Test: concatlng2args-4
    // XPath: (xs:long(\"-92233720368547758\") || xs:long(\"-4717556220304
    // Expected: -92233720368547758-47175562203048468
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatlng2args_5() {
    //  ******************************************************* Test: concatlng2args-5
    // XPath: (xs:long(\"-92233720368547758\") || xs:long(\"92233720368547
    // Expected: -9223372036854775892233720368547758
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatusht2args_1() {
    //  ******************************************************* Test: concatusht2args-1
    // XPath: (xs:unsignedShort(\"0\") || xs:unsignedShort(\"0\"))
    // Expected: 00
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatusht2args_2() {
    //  ******************************************************* Test: concatusht2args-2
    // XPath: (xs:unsignedShort(\"44633\") || xs:unsignedShort(\"0\"))
    // Expected: 446330
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatusht2args_3() {
    //  ******************************************************* Test: concatusht2args-3
    // XPath: (xs:unsignedShort(\"65535\") || xs:unsignedShort(\"0\"))
    // Expected: 655350
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatusht2args_4() {
    //  ******************************************************* Test: concatusht2args-4
    // XPath: (xs:unsignedShort(\"0\") || xs:unsignedShort(\"44633\"))
    // Expected: 044633
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatusht2args_5() {
    //  ******************************************************* Test: concatusht2args-5
    // XPath: (xs:unsignedShort(\"0\") || xs:unsignedShort(\"65535\"))
    // Expected: 065535
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnint2args_1() {
    //  ******************************************************* Test: concatnint2args-1
    // XPath: (xs:negativeInteger(\"-999999999999999999\") || xs:negativeI
    // Expected: -999999999999999999-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnint2args_2() {
    //  ******************************************************* Test: concatnint2args-2
    // XPath: (xs:negativeInteger(\"-297014075999096793\") || xs:negativeI
    // Expected: -297014075999096793-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnint2args_3() {
    //  ******************************************************* Test: concatnint2args-3
    // XPath: (xs:negativeInteger(\"-1\") || xs:negativeInteger(\"-9999999
    // Expected: -1-999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnint2args_4() {
    //  ******************************************************* Test: concatnint2args-4
    // XPath: (xs:negativeInteger(\"-999999999999999999\") || xs:negativeI
    // Expected: -999999999999999999-297014075999096793
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatnint2args_5() {
    //  ******************************************************* Test: concatnint2args-5
    // XPath: (xs:negativeInteger(\"-999999999999999999\") || xs:negativeI
    // Expected: -999999999999999999-1
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatpint2args_1() {
    //  ******************************************************* Test: concatpint2args-1
    // XPath: (xs:positiveInteger(\"1\") || xs:positiveInteger(\"1\"))
    // Expected: 11
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatpint2args_2() {
    //  ******************************************************* Test: concatpint2args-2
    // XPath: (xs:positiveInteger(\"52704602390610033\") || xs:positiveInt
    // Expected: 527046023906100331
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatpint2args_3() {
    //  ******************************************************* Test: concatpint2args-3
    // XPath: (xs:positiveInteger(\"999999999999999999\") || xs:positiveIn
    // Expected: 9999999999999999991
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatpint2args_4() {
    //  ******************************************************* Test: concatpint2args-4
    // XPath: (xs:positiveInteger(\"1\") || xs:positiveInteger(\"527046023
    // Expected: 152704602390610033
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatpint2args_5() {
    //  ******************************************************* Test: concatpint2args-5
    // XPath: (xs:positiveInteger(\"1\") || xs:positiveInteger(\"999999999
    // Expected: 1999999999999999999
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatulng2args_1() {
    //  ******************************************************* Test: concatulng2args-1
    // XPath: (xs:unsignedLong(\"0\") || xs:unsignedLong(\"0\"))
    // Expected: 00
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatulng2args_2() {
    //  ******************************************************* Test: concatulng2args-2
    // XPath: (xs:unsignedLong(\"130747108607674654\") || xs:unsignedLong(
    // Expected: 1307471086076746540
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatulng2args_3() {
    //  ******************************************************* Test: concatulng2args-3
    // XPath: (xs:unsignedLong(\"184467440737095516\") || xs:unsignedLong(
    // Expected: 1844674407370955160
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatulng2args_4() {
    //  ******************************************************* Test: concatulng2args-4
    // XPath: (xs:unsignedLong(\"0\") || xs:unsignedLong(\"130747108607674
    // Expected: 0130747108607674654
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}

#[test]
fn op_concatulng2args_5() {
    //  ******************************************************* Test: concatulng2args-5
    // XPath: (xs:unsignedLong(\"0\") || xs:unsignedLong(\"184467440737095
    // Expected: 0184467440737095516
    // Calls stub function - will fail until op:concat is implemented
    let _ = stub_opconcat();
}
