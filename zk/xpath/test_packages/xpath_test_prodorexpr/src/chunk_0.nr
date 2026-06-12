//! Test chunk 0 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_and_001() {
    //  Logical 'and' using Boolean values
    // XPath: false() and false()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_002() {
    //  Logical 'and' using Boolean values
    // XPath: true() and false()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_003() {
    //  Logical 'and' using Boolean values
    // XPath: false() and true()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_004() {
    //  Logical 'and' using Boolean values
    // XPath: true() and true()
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_005() {
    //  Logical 'and' using sequences
    // XPath: () and ()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_006() {
    //  Logical 'and' using sequences
    // XPath: (1) and ()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_007() {
    //  Logical 'and' using sequences
    // XPath: () and (1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_008() {
    //  Logical 'and' using sequences
    // XPath: (1) and (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_009() {
    //  Logical 'and' using sequences
    // XPath: (0) and ()
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_010() {
    //  Logical 'and' using sequences
    // XPath: () and (0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_011() {
    //  Logical 'and' using sequences
    // XPath: (0) and (0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_012() {
    //  Logical 'and' using sequences
    // XPath: (1) and (0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_013() {
    //  Logical 'and' using sequences
    // XPath: (0) and (1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_014() {
    //  Logical 'and' using sequences
    // XPath: (0) and (/bib/book/price/text())
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_015() {
    //  Logical 'and' using sequences
    // XPath: (/bib/book/price/text()) and (1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_016() {
    //  Logical 'and' using sequences
    // XPath: (/bib/book/price/text()) and (/bib/book/price/text())
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_017() {
    //  Logical 'and' using string values
    // XPath: \"\" and ''
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_018() {
    //  Logical 'and' using string values
    // XPath: \"\" and 'a'
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_019() {
    //  Logical 'and' using string values
    // XPath: \"0\" and ''
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_020() {
    //  Logical 'and' using string values
    // XPath: \"a\" and '0'
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_021() {
    //  Logical 'and' using string values
    // XPath: xs:string(\"\") and xs:string('')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_022() {
    //  Logical 'and' using string values
    // XPath: xs:string(\"\") and xs:string('abc')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_023() {
    //  Logical 'and' using string values
    // XPath: xs:string(\"abc\") and xs:string('')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_024() {
    //  Logical 'and' using string values
    // XPath: xs:string(\"0\") and xs:string('abc')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_025() {
    //  Logical 'and' using untypedAtomic values
    // XPath: xs:untypedAtomic('') and xs:untypedAtomic(\"\")
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_026() {
    //  Logical 'and' using untypedAtomic values
    // XPath: xs:untypedAtomic('abc') and xs:untypedAtomic(\"\")
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_027() {
    //  Logical 'and' using untypedAtomic values
    // XPath: xs:untypedAtomic('') and xs:untypedAtomic(\"0\")
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_028() {
    //  Logical 'and' using untypedAtomic values
    // XPath: xs:untypedAtomic('abc') and xs:untypedAtomic(\"0\")
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_029() {
    //  Logical 'and' using number values
    // XPath: 0 and 0
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_030() {
    //  Logical 'and' using number values
    // XPath: 0 and 1
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_031() {
    //  Logical 'and' using number values
    // XPath: 1 and 0
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_032() {
    //  Logical 'and' using number values
    // XPath: 0 and -1
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_033() {
    //  Logical 'and' using number values
    // XPath: -1 and 1
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_034() {
    //  Logical 'and' using float values
    // XPath: xs:float(0) and xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_035() {
    //  Logical 'and' using float values
    // XPath: xs:float(0) and xs:float(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_036() {
    //  Logical 'and' using float values
    // XPath: xs:float(-1) and xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_037() {
    //  Logical 'and' using float values
    // XPath: xs:float(1) and xs:float(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_038() {
    //  Logical 'and' using float values
    // XPath: xs:float('NaN') and xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_039() {
    //  Logical 'and' using float values
    // XPath: xs:float('NaN') and xs:float(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_040() {
    //  Logical 'and' using float values
    // XPath: xs:float('NaN') and xs:float('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_041() {
    //  Logical 'and' using float values
    // XPath: xs:float('INF') and xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_042() {
    //  Logical 'and' using float values
    // XPath: xs:float('INF') and xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_043() {
    //  Logical 'and' using float values
    // XPath: xs:float('INF') and xs:float('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_044() {
    //  Logical 'and' using float values
    // XPath: xs:float('-INF') and xs:float(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_045() {
    //  Logical 'and' using float values
    // XPath: xs:float('-INF') and xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_046() {
    //  Logical 'and' using float values
    // XPath: xs:float('-INF') and xs:float('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_047() {
    //  Logical 'and' using float values
    // XPath: xs:float('-INF') and xs:float('INF')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_048() {
    //  Logical 'and' using double values
    // XPath: xs:double(0) and xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_049() {
    //  Logical 'and' using double values
    // XPath: xs:double(0) and xs:double(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_050() {
    //  Logical 'and' using double values
    // XPath: xs:double(-1) and xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
