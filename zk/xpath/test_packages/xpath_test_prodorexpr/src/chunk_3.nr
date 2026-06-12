//! Test chunk 3 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_and_153() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(0) and xs:unsignedByte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_154() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(0) and xs:unsignedByte(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_155() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(1) and xs:unsignedByte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_156() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(1) and xs:unsignedByte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_157() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(255) and xs:unsignedByte(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_158() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(255) and xs:unsignedByte(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_159() {
    //  Logical 'and' using unsignedByte values
    // XPath: xs:unsignedByte(255) and xs:unsignedByte(255)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_001() {
    //  Logical 'or' using boolean values
    // XPath: false() or false()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_002() {
    //  Logical 'or' using boolean values
    // XPath: true() or false()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_003() {
    //  Logical 'or' using boolean values
    // XPath: false() or true()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_004() {
    //  Logical 'or' using boolean values
    // XPath: true() or true()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_005() {
    //  Logical 'or' using sequences values
    // XPath: () or ()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_006() {
    //  Logical 'or' using sequences values
    // XPath: (1) or ()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_007() {
    //  Logical 'or' using sequences values
    // XPath: () or (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_008() {
    //  Logical 'or' using sequences values
    // XPath: (1) or (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_009() {
    //  Logical 'or' using sequences values
    // XPath: (0) or ()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_010() {
    //  Logical 'or' using sequences values
    // XPath: () or (0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_011() {
    //  Logical 'or' using sequences values
    // XPath: (0) or (0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_012() {
    //  Logical 'or' using sequences values
    // XPath: (1) or (0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_013() {
    //  Logical 'or' using sequences values
    // XPath: (0) or (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_014() {
    //  Logical 'or' using sequences values
    // XPath: (0) or (/bib/book/price/text())
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_015() {
    //  Logical 'or' using sequences values
    // XPath: (/bib/book/price/text()) or (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_016() {
    //  Logical 'or' using sequences values
    // XPath: (/bib/book/price/text()) or (/bib/book/price/text())
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_017() {
    //  Logical 'or' using string values
    // XPath: \"\" or ''
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_018() {
    //  Logical 'or' using string values
    // XPath: \"\" or 'a'
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_019() {
    //  Logical 'or' using string values
    // XPath: \"0\" or ''
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_020() {
    //  Logical 'or' using string values
    // XPath: \"a\" or '0'
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_021() {
    //  Logical 'or' using string values
    // XPath: xs:string(\"\") or xs:string('')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_022() {
    //  Logical 'or' using string values
    // XPath: xs:string(\"\") or xs:string('abc')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_023() {
    //  Logical 'or' using string values
    // XPath: xs:string(\"abc\") or xs:string('')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_024() {
    //  Logical 'or' using string values
    // XPath: xs:string(\"0\") or xs:string('abc')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_025() {
    //  Logical 'or' using untypedAtomic values
    // XPath: xs:untypedAtomic('') or xs:untypedAtomic(\"\")
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_026() {
    //  Logical 'or' using untypedAtomic values
    // XPath: xs:untypedAtomic('abc') or xs:untypedAtomic(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_027() {
    //  Logical 'or' using untypedAtomic values
    // XPath: xs:untypedAtomic('') or xs:untypedAtomic(\"0\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_028() {
    //  Logical 'or' using untypedAtomic values
    // XPath: xs:untypedAtomic('abc') or xs:untypedAtomic(\"0\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_029() {
    //  Logical 'or' using number values
    // XPath: 0 or 0
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_030() {
    //  Logical 'or' using number values
    // XPath: 0 or 1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_031() {
    //  Logical 'or' using number values
    // XPath: 1 or 0
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_032() {
    //  Logical 'or' using number values
    // XPath: 0 or -1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_033() {
    //  Logical 'or' using number values
    // XPath: -1 or 1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_034() {
    //  Logical 'or' using float values
    // XPath: xs:float(0) or xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_035() {
    //  Logical 'or' using float values
    // XPath: xs:float(0) or xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_036() {
    //  Logical 'or' using float values
    // XPath: xs:float(-1) or xs:float(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_037() {
    //  Logical 'or' using float values
    // XPath: xs:float(1) or xs:float(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_038() {
    //  Logical 'or' using float values
    // XPath: xs:float('NaN') or xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_039() {
    //  Logical 'or' using float values
    // XPath: xs:float('NaN') or xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_040() {
    //  Logical 'or' using float values
    // XPath: xs:float('NaN') or xs:float('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_041() {
    //  Logical 'or' using float values
    // XPath: xs:float('INF') or xs:float(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_042() {
    //  Logical 'or' using float values
    // XPath: xs:float('INF') or xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_043() {
    //  Logical 'or' using float values
    // XPath: xs:float('INF') or xs:float('NaN')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
