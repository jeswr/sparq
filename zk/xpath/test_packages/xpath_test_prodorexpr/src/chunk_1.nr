//! Test chunk 1 for prod:OrExpr
//! Contains 50 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn op_logical_and_051() {
    //  Logical 'and' using double values
    // XPath: xs:double(1) and xs:double(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_052() {
    //  Logical 'and' using double values
    // XPath: xs:double('NaN') and xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_053() {
    //  Logical 'and' using double values
    // XPath: xs:double('NaN') and xs:double(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_054() {
    //  Logical 'and' using double values
    // XPath: xs:double('NaN') and xs:double('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_055() {
    //  Logical 'and' using double values
    // XPath: xs:double('INF') and xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_056() {
    //  Logical 'and' using double values
    // XPath: xs:double('INF') and xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_057() {
    //  Logical 'and' using double values
    // XPath: xs:double('INF') and xs:double('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_058() {
    //  Logical 'and' using double values
    // XPath: xs:double('-INF') and xs:double(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_059() {
    //  Logical 'and' using double values
    // XPath: xs:double('-INF') and xs:double(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_060() {
    //  Logical 'and' using double values
    // XPath: xs:double('-INF') and xs:double('NaN')
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_061() {
    //  Logical 'and' using double values
    // XPath: xs:double('-INF') and xs:double('INF')
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_062() {
    //  Logical 'and' using decimal values
    // XPath: xs:decimal(0) and xs:decimal(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_063() {
    //  Logical 'and' using decimal values
    // XPath: xs:decimal(0) and xs:decimal(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_064() {
    //  Logical 'and' using decimal values
    // XPath: xs:decimal(-1) and xs:decimal(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_065() {
    //  Logical 'and' using decimal values
    // XPath: xs:decimal(1) and xs:decimal(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_067() {
    //  Logical 'and' using decimal values
    // XPath: xs:decimal(-123456789.123456789123456789) and xs:decimal(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_069() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(0) and xs:integer(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_070() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(0) and xs:integer(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_071() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(-1) and xs:integer(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_072() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(1) and xs:integer(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_073() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(99999999999999999) and xs:integer(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_074() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(-99999999999999999) and xs:integer(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_075() {
    //  Logical 'and' using integer values
    // XPath: xs:integer(99999999999999999) and xs:integer(-99999999999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_076() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(0) and xs:nonPositiveInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_077() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(0) and xs:nonPositiveInteger(-1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_078() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) and xs:nonPositiveInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_079() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) and xs:nonPositiveInteger(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_080() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-99999999999999999) and xs:nonPositive
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_081() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-1) and xs:nonPositiveInteger(-9999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_082() {
    //  Logical 'and' using nonPositiveInteger values
    // XPath: xs:nonPositiveInteger(-99999999999999999) and xs:nonPositive
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_083() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(0) and xs:nonNegativeInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_084() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(0) and xs:nonNegativeInteger(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_085() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) and xs:nonNegativeInteger(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_086() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) and xs:nonNegativeInteger(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_087() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(9999999999) and xs:nonNegativeInteger(
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_088() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(1) and xs:nonNegativeInteger(999999999
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_089() {
    //  Logical 'and' using nonNegativeInteger values
    // XPath: xs:nonNegativeInteger(99999999999999999) and xs:nonNegativeI
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_090() {
    //  Logical 'and' using negativeInteger values
    // XPath: xs:negativeInteger(-1) and xs:negativeInteger(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_091() {
    //  Logical 'and' using negativeInteger values
    // XPath: xs:negativeInteger(-99999999999999999) and xs:negativeIntege
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_092() {
    //  Logical 'and' using negativeInteger values
    // XPath: xs:negativeInteger(-99999999999999999) and xs:negativeIntege
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_093() {
    //  Logical 'and' using positiveInteger values
    // XPath: xs:positiveInteger(1) and xs:positiveInteger(1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_094() {
    //  Logical 'and' using positiveInteger values
    // XPath: xs:positiveInteger(99999999999999999) and xs:positiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_095() {
    //  Logical 'and' using positiveInteger values
    // XPath: xs:positiveInteger(99999999999999999) and xs:positiveInteger
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_096() {
    //  Logical 'and' using long values
    // XPath: xs:long(0) and xs:long(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_097() {
    //  Logical 'and' using long values
    // XPath: xs:long(0) and xs:long(1)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_098() {
    //  Logical 'and' using long values
    // XPath: xs:long(-1) and xs:long(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_099() {
    //  Logical 'and' using long values
    // XPath: xs:long(1) and xs:long(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_100() {
    //  Logical 'and' using long values
    // XPath: xs:long(9223372036854775807) and xs:long(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_101() {
    //  Logical 'and' using long values
    // XPath: xs:long(9223372036854775807) and xs:long(-1)
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn op_logical_and_102() {
    //  Logical 'and' using long values
    // XPath: xs:long(-99999999999999999) and xs:long(0)
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
