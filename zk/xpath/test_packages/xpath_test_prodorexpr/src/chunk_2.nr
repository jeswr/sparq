//! Test chunk 2 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_and_103() {
    //  Logical 'and' using long values
    // XPath: xs:long(-99999999999999999) and xs:long(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_104() {
    //  Logical 'and' using long values
    // XPath: xs:long(99999999999999999) and xs:long(-99999999999999999)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_105() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(0) and xs:unsignedLong(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_106() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(0) and xs:unsignedLong(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_107() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(1) and xs:unsignedLong(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_108() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(1) and xs:unsignedLong(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_109() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(9223372036854775807) and xs:unsignedLong(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_110() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(9223372036854775807) and xs:unsignedLong(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_111() {
    //  Logical 'and' using unsignedLong values
    // XPath: xs:unsignedLong(99999999999999999) and xs:unsignedLong(99999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_112() {
    //  Logical 'and' using int values
    // XPath: xs:int(0) and xs:int(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_113() {
    //  Logical 'and' using int values
    // XPath: xs:int(0) and xs:int(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_114() {
    //  Logical 'and' using int values
    // XPath: xs:int(-1) and xs:int(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_115() {
    //  Logical 'and' using int values
    // XPath: xs:int(1) and xs:int(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_116() {
    //  Logical 'and' using int values
    // XPath: xs:int(2147483647) and xs:int(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_117() {
    //  Logical 'and' using int values
    // XPath: xs:int(2147483647) and xs:int(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_118() {
    //  Logical 'and' using int values
    // XPath: xs:int(-2147483648) and xs:int(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_119() {
    //  Logical 'and' using int values
    // XPath: xs:int(-2147483648) and xs:int(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_120() {
    //  Logical 'and' using int values
    // XPath: xs:int(2147483647) and xs:int(-2147483648)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_121() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(0) and xs:unsignedInt(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_122() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(0) and xs:unsignedInt(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_123() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(1) and xs:unsignedInt(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_124() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(1) and xs:unsignedInt(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_125() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) and xs:unsignedInt(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_126() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) and xs:unsignedInt(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_127() {
    //  Logical 'and' using unsignedInt values
    // XPath: xs:unsignedInt(4294967295) and xs:unsignedInt(4294967295)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_128() {
    //  Logical 'and' using short values
    // XPath: xs:short(0) and xs:short(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_129() {
    //  Logical 'and' using short values
    // XPath: xs:short(0) and xs:short(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_130() {
    //  Logical 'and' using short values
    // XPath: xs:short(-1) and xs:short(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_131() {
    //  Logical 'and' using short values
    // XPath: xs:short(1) and xs:short(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_132() {
    //  Logical 'and' using short values
    // XPath: xs:short(32767) and xs:short(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_133() {
    //  Logical 'and' using short values
    // XPath: xs:short(32767) and xs:short(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_134() {
    //  Logical 'and' using short values
    // XPath: xs:short(-32768) and xs:short(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_135() {
    //  Logical 'and' using short values
    // XPath: xs:short(-32768) and xs:short(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_136() {
    //  Logical 'and' using short values
    // XPath: xs:short(32767) and xs:short(-32768)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_137() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(0) and xs:unsignedShort(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_138() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(0) and xs:unsignedShort(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_139() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(1) and xs:unsignedShort(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_140() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(1) and xs:unsignedShort(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_141() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(65535) and xs:unsignedShort(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_142() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(65535) and xs:unsignedShort(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_143() {
    //  Logical 'and' using unsignedShort values
    // XPath: xs:unsignedShort(65535) and xs:unsignedShort(65535)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_144() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(0) and xs:byte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_145() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(0) and xs:byte(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_146() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(-1) and xs:byte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_147() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(1) and xs:byte(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_148() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(127) and xs:byte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_149() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(127) and xs:byte(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_150() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(-128) and xs:byte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_151() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(-128) and xs:byte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_152() {
    //  Logical 'and' using byte values
    // XPath: xs:byte(127) and xs:byte(-128)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
