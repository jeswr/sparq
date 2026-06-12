//! Test chunk 5 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_or_096() {
    //  Logical 'or' using long values
    // XPath: xs:long(0) or xs:long(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_097() {
    //  Logical 'or' using long values
    // XPath: xs:long(0) or xs:long(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_098() {
    //  Logical 'or' using long values
    // XPath: xs:long(-1) or xs:long(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_099() {
    //  Logical 'or' using long values
    // XPath: xs:long(1) or xs:long(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_100() {
    //  Logical 'or' using long values
    // XPath: xs:long(9223372036854775807) or xs:long(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_101() {
    //  Logical 'or' using long values
    // XPath: xs:long(9223372036854775807) or xs:long(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_102() {
    //  Logical 'or' using long values
    // XPath: xs:long(-99999999999999999) or xs:long(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_103() {
    //  Logical 'or' using long values
    // XPath: xs:long(-99999999999999999) or xs:long(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_104() {
    //  Logical 'or' using long values
    // XPath: xs:long(99999999999999999) or xs:long(-99999999999999999)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_105() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(0) or xs:unsignedLong(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_106() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(0) or xs:unsignedLong(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_107() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(1) or xs:unsignedLong(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_108() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(1) or xs:unsignedLong(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_109() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(9223372036854775807) or xs:unsignedLong(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_110() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(9223372036854775807) or xs:unsignedLong(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_111() {
    //  Logical 'or' using unsignedLong values
    // XPath: xs:unsignedLong(99999999999999999) or xs:unsignedLong(999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_112() {
    //  Logical 'or' using int values
    // XPath: xs:int(0) or xs:int(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_113() {
    //  Logical 'or' using int values
    // XPath: xs:int(0) or xs:int(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_114() {
    //  Logical 'or' using int values
    // XPath: xs:int(-1) or xs:int(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_115() {
    //  Logical 'or' using int values
    // XPath: xs:int(1) or xs:int(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_116() {
    //  Logical 'or' using int values
    // XPath: xs:int(2147483647) or xs:int(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_117() {
    //  Logical 'or' using int values
    // XPath: xs:int(2147483647) or xs:int(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_118() {
    //  Logical 'or' using int values
    // XPath: xs:int(-2147483648) or xs:int(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_119() {
    //  Logical 'or' using int values
    // XPath: xs:int(-2147483648) or xs:int(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_120() {
    //  Logical 'or' using int values
    // XPath: xs:int(2147483647) or xs:int(-2147483648)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_121() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(0) or xs:unsignedInt(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_122() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(0) or xs:unsignedInt(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_123() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(1) or xs:unsignedInt(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_124() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(1) or xs:unsignedInt(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_125() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) or xs:unsignedInt(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_126() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) or xs:unsignedInt(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_127() {
    //  Logical 'or' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) or xs:unsignedInt(4294967295)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_128() {
    //  Logical 'or' using short values
    // XPath: xs:short(0) or xs:short(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_129() {
    //  Logical 'or' using short values
    // XPath: xs:short(0) or xs:short(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_130() {
    //  Logical 'or' using short values
    // XPath: xs:short(-1) or xs:short(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_131() {
    //  Logical 'or' using short values
    // XPath: xs:short(1) or xs:short(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_132() {
    //  Logical 'or' using short values
    // XPath: xs:short(32767) or xs:short(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_133() {
    //  Logical 'or' using short values
    // XPath: xs:short(32767) or xs:short(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_134() {
    //  Logical 'or' using short values
    // XPath: xs:short(-32768) or xs:short(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_135() {
    //  Logical 'or' using short values
    // XPath: xs:short(-32768) or xs:short(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_136() {
    //  Logical 'or' using short values
    // XPath: xs:short(32767) or xs:short(-32768)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_137() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(0) or xs:unsignedShort(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_138() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(0) or xs:unsignedShort(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_139() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(1) or xs:unsignedShort(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_140() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(1) or xs:unsignedShort(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_141() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(65535) or xs:unsignedShort(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_142() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(65535) or xs:unsignedShort(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_143() {
    //  Logical 'or' using unsignedShort values
    // XPath: xs:unsignedShort(65535) or xs:unsignedShort(65535)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_144() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(0) or xs:byte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_145() {
    //  Logical 'or' using byte values
    // XPath: xs:byte(0) or xs:byte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
