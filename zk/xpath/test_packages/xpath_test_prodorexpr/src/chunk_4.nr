//! Test chunk 4 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_or_044() {
    //  Logical 'or' using float values
    // XPath: xs:float('-INF') or xs:float(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_045() {
    //  Logical 'or' using float values
    // XPath: xs:float('-INF') or xs:float(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_046() {
    //  Logical 'or' using float values
    // XPath: xs:float('-INF') or xs:float('NaN')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_047() {
    //  Logical 'or' using float values
    // XPath: xs:float('-INF') or xs:float('INF')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_048() {
    //  Logical 'or' using double values
    // XPath: xs:double(0) or xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_049() {
    //  Logical 'or' using double values
    // XPath: xs:double(0) or xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_050() {
    //  Logical 'or' using double values
    // XPath: xs:double(-1) or xs:double(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_051() {
    //  Logical 'or' using double values
    // XPath: xs:double(1) or xs:double(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_052() {
    //  Logical 'or' using double values
    // XPath: xs:double('NaN') or xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_053() {
    //  Logical 'or' using double values
    // XPath: xs:double('NaN') or xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_054() {
    //  Logical 'or' using double values
    // XPath: xs:double('NaN') or xs:double('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_055() {
    //  Logical 'or' using double values
    // XPath: xs:double('INF') or xs:double(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_056() {
    //  Logical 'or' using double values
    // XPath: xs:double('INF') or xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_057() {
    //  Logical 'or' using double values
    // XPath: xs:double('INF') or xs:double('NaN')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_058() {
    //  Logical 'or' using double values
    // XPath: xs:double('-INF') or xs:double(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_059() {
    //  Logical 'or' using double values
    // XPath: xs:double('-INF') or xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_060() {
    //  Logical 'or' using double values
    // XPath: xs:double('-INF') or xs:double('NaN')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_061() {
    //  Logical 'or' using double values
    // XPath: xs:double('-INF') or xs:double('INF')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_062() {
    //  Logical 'or' using decimal values
    // XPath: xs:decimal(0) or xs:decimal(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_063() {
    //  Logical 'or' using decimal values
    // XPath: xs:decimal(0) or xs:decimal(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_064() {
    //  Logical 'or' using decimal values
    // XPath: xs:decimal(-1) or xs:decimal(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_065() {
    //  Logical 'or' using decimal values
    // XPath: xs:decimal(1) or xs:decimal(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_067() {
    //  Logical 'or' using decimal values
    // XPath: xs:decimal(-123456789.123456789123456789) or xs:decimal(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_069() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(0) or xs:integer(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_070() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(0) or xs:integer(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_071() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(-1) or xs:integer(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_072() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(1) or xs:integer(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_073() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(99999999999999999) or xs:integer(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_074() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(-99999999999999999) or xs:integer(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_075() {
    //  Logical 'or' using integer values
    // XPath: xs:integer(99999999999999999) or xs:integer(-999999999999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_076() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(0) or xs:nonPositiveInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_077() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(0) or xs:nonPositiveInteger(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_078() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) or xs:nonPositiveInteger(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_079() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) or xs:nonPositiveInteger(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_080() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-99999999999999999) or xs:nonPositiveI
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_081() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) or xs:nonPositiveInteger(-99999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_082() {
    //  Logical 'or' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-99999999999999999) or xs:nonPositiveI
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_083() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(0) or xs:nonNegativeInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_084() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(0) or xs:nonNegativeInteger(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_085() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) or xs:nonNegativeInteger(0)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_086() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) or xs:nonNegativeInteger(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_087() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(99999999999999999) or xs:nonNegativeIn
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_088() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) or xs:nonNegativeInteger(9999999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_089() {
    //  Logical 'or' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(99999999999999999) or xs:nonNegativeIn
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_090() {
    //  Logical 'or' using negativeInteger values
    // XPath: xs:negativeInteger(-1) or xs:negativeInteger(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_091() {
    //  Logical 'or' using negativeInteger values
    // XPath: xs:negativeInteger(-99999999999999999) or xs:negativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_092() {
    //  Logical 'or' using negativeInteger values
    // XPath: xs:negativeInteger(-99999999999999999) or xs:negativeInteger
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_093() {
    //  Logical 'or' using positiveInteger values
    // XPath: xs:positiveInteger(1) or xs:positiveInteger(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_094() {
    //  Logical 'or' using positiveInteger values
    // XPath: xs:positiveInteger(99999999999999999) or xs:positiveInteger(
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_or_095() {
    //  Logical 'or' using positiveInteger values
    // XPath: xs:positiveInteger(99999999999999999) or xs:positiveInteger(
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
