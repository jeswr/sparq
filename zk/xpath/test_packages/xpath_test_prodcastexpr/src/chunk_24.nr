//! Test chunk 24 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k2_seqexprcast_11() {
    //  Pass a small, the smallest if such a limit exists, to unsignedInt.
    // XPath: xs:unsignedInt(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_12() {
    //  Pass a small, the smallest if such a limit exists, to unsignedByte.
    // XPath: xs:unsignedByte(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_13() {
    //  Pass a small, the smallest if such a limit exists, to unsignedShort.
    // XPath: xs:unsignedShort(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_14() {
    //  Pass a small, the smallest if such a limit exists, to positiveInteger.
    // XPath: xs:positiveInteger(\"1\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_15() {
    //  Pass a large, the largest if such a limit exists, to nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"-999999999999999999\")
    // Expected: -999999999999999999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_16() {
    //  Pass a large, the largest if such a limit exists, to negativeInteger.
    // XPath: xs:negativeInteger(\"-999999999999999999\")
    // Expected: -999999999999999999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_18() {
    //  Pass a large, the largest if such a limit exists, to int.
    // XPath: xs:int(\"-2147483648\")
    // Expected: -2147483648
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_19() {
    //  Pass a large, the largest if such a limit exists, to short.
    // XPath: xs:short(\"32767\")
    // Expected: 32767
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_20() {
    //  Pass a large, the largest if such a limit exists, to byte.
    // XPath: xs:byte(\"127\")
    // Expected: 127
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_21() {
    //  Pass a large, the largest if such a limit exists, to nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(\"999999999999999999\")
    // Expected: 999999999999999999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_23() {
    //  Pass a large, the largest if such a limit exists, to unsignedInt.
    // XPath: xs:unsignedInt(\"4294967295\")
    // Expected: 4294967295
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_24() {
    //  Pass a large, the largest if such a limit exists, to unsignedShort.
    // XPath: xs:unsignedShort(\"65535\")
    // Expected: 65535
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_25() {
    //  Pass a large, the largest if such a limit exists, to unsignedByte.
    // XPath: xs:unsignedByte(\"255\")
    // Expected: 255
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_26() {
    //  Pass a large, the largest if such a limit exists, to positiveInteger.
    // XPath: xs:positiveInteger(\"999999999999999999\")
    // Expected: 999999999999999999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_27() {
    //  Cast fn:false() to nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_31() {
    //  Cast fn:false() to long.
    // XPath: xs:long(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_32() {
    //  Cast fn:true() to long.
    // XPath: xs:long(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_33() {
    //  Cast fn:false() to int.
    // XPath: xs:int(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_34() {
    //  Cast fn:true() to int.
    // XPath: xs:int(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_35() {
    //  Cast fn:false() to short.
    // XPath: xs:short(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_36() {
    //  Cast fn:true() to short.
    // XPath: xs:short(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_37() {
    //  Cast fn:false() to byte.
    // XPath: xs:byte(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_38() {
    //  Cast fn:true() to byte.
    // XPath: xs:byte(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_39() {
    //  Cast fn:false() to nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_40() {
    //  Cast fn:true() to nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_41() {
    //  Cast fn:false() to unsignedLong.
    // XPath: xs:unsignedLong(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_42() {
    //  Cast fn:true() to unsignedLong.
    // XPath: xs:unsignedLong(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_43() {
    //  Cast fn:false() to unsignedInt.
    // XPath: xs:unsignedInt(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_44() {
    //  Cast fn:true() to unsignedInt.
    // XPath: xs:unsignedInt(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_45() {
    //  Cast fn:false() to unsignedShort.
    // XPath: xs:unsignedShort(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_46() {
    //  Cast fn:true() to unsignedShort.
    // XPath: xs:unsignedShort(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_47() {
    //  Cast fn:false() to unsignedByte.
    // XPath: xs:unsignedByte(false())
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_48() {
    //  Cast fn:true() to unsignedByte.
    // XPath: xs:unsignedByte(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_50() {
    //  Cast fn:true() to positiveInteger.
    // XPath: xs:positiveInteger(true())
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_52() {
    //  The minus sign is allowed unsignedLong.        S4S is considered normative, and
    // XPath: xs:unsignedLong(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_53() {
    //  The minus sign is allowed unsignedInt.        S4S is considered normative, and
    // XPath: xs:unsignedInt(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_54() {
    //  The minus sign is allowed unsignedShort.        S4S is considered normative, an
    // XPath: xs:unsignedShort(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_55() {
    //  The minus sign is allowed in unsignedByte.        S4S is considered normative,
    // XPath: xs:unsignedByte(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_58() {
    //  The plus sign is allowed in unsignedLong.        S4S is considered normative, a
    // XPath: xs:unsignedLong(\"+0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_59() {
    //  The plus sign is allowed in unsignedInt.        S4S is considered normative, an
    // XPath: xs:unsignedInt(\"+0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_60() {
    //  The plus sign is allowed in unsignedShort.        S4S is considered normative,
    // XPath: xs:unsignedShort(\"+0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_61() {
    //  The plus sign is allowed in unsignedByte.        S4S is considered normative, a
    // XPath: xs:unsignedByte(\"+0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_64() {
    //  The minus sign is allowed in nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_65() {
    //  The minus sign is allowed in negativeInteger.
    // XPath: xs:negativeInteger(\"-1\")
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_66() {
    //  The minus sign is allowed in nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"-0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_67() {
    //  The minus sign is allowed in nonPositiveInteger(#2).
    // XPath: xs:nonPositiveInteger(\"-1\")
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_68() {
    //  Ensure a plus sign is accepted for positiveInteger.
    // XPath: xs:positiveInteger(\"+1\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_69() {
    //  Ensure a plus sign is accepted for nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(\"+1\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_70() {
    //  Ensure a plus sign is accepted for nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"+0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_132() {
    //  Pass preceding zeros to nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"-000000001\")
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
