//! Test chunk 4 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas201() {
    // Try xs:double(1e8) castable as xs:string
    // XPath: xs:double(\"1e8\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas202() {
    // Try xs:double(INF) castable as xs:string
    // XPath: xs:double(\"INF\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas203() {
    // Try xs:double(-0.0E0) castable as xs:float
    // XPath: xs:double(\"-0.0E0\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas204() {
    // Try xs:double(5.4321E-1001) castable as xs:float
    // XPath: xs:double(\"5.4321E-1001\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas205() {
    // Try xs:double(1e5) castable as xs:float
    // XPath: xs:double(\"1e5\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas206() {
    // Try xs:double(-1.75e-3) castable as xs:float
    // XPath: xs:double(\"-1.75e-3\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas207() {
    // Try xs:double(NaN) castable as xs:float
    // XPath: xs:double(\"NaN\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas208() {
    // Try xs:double(1e5) castable as xs:double
    // XPath: xs:double(\"1e5\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas209() {
    // Try xs:double(INF) castable as xs:double
    // XPath: xs:double(\"INF\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas210() {
    // Try xs:double(-0.0E0) castable as xs:decimal
    // XPath: xs:double(\"-0.0E0\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas211() {
    // Try xs:double(5.4321E-1001) castable as xs:decimal
    // XPath: xs:double(\"5.4321E-1001\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas212() {
    // Try xs:double(-1.75e-3) castable as xs:decimal
    // XPath: xs:double(\"-1.75e-3\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas213() {
    // Try xs:double(INF) castable as xs:decimal
    // XPath: xs:double(\"INF\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas214() {
    // Try xs:double(-0.0E0) castable as xs:integer
    // XPath: xs:double(\"-0.0E0\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas215() {
    // Try xs:double(1e5) castable as xs:integer
    // XPath: xs:double(\"1e5\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas216() {
    // Try xs:double(-1.75e-3) castable as xs:integer
    // XPath: xs:double(\"-1.75e-3\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas217() {
    // Try xs:double(INF) castable as xs:integer
    // XPath: xs:double(\"INF\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas218() {
    // Try xs:double(NaN) castable as xs:integer
    // XPath: xs:double(\"NaN\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas219() {
    // Try xs:double(1e5) castable as xs:duration
    // XPath: xs:double(\"1e5\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas220() {
    // Try xs:double(1e5) castable as xs:yearMonthDuration
    // XPath: xs:double(\"1e5\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas221() {
    // Try xs:double(1e5) castable as xs:dayTimeDuration
    // XPath: xs:double(\"1e5\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas222() {
    // Try xs:double(1e5) castable as xs:dateTime
    // XPath: xs:double(\"1e5\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas223() {
    // Try xs:double(1e5) castable as xs:time
    // XPath: xs:double(\"1e5\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas224() {
    // Try xs:double(1e5) castable as xs:date
    // XPath: xs:double(\"1e5\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas225() {
    // Try xs:double(1e5) castable as xs:gYearMonth
    // XPath: xs:double(\"1e5\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas226() {
    // Try xs:double(1e5) castable as xs:gYear
    // XPath: xs:double(\"1e5\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas227() {
    // Try xs:double(1e5) castable as xs:gMonthDay
    // XPath: xs:double(\"1e5\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas228() {
    // Try xs:double(1e5) castable as xs:gDay
    // XPath: xs:double(\"1e5\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas229() {
    // Try xs:double(1e5) castable as xs:gMonth
    // XPath: xs:double(\"1e5\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas230() {
    // Try xs:double(1e5) castable as xs:boolean
    // XPath: xs:double(\"1e5\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas231() {
    // Try xs:double(1e5) castable as xs:base64Binary
    // XPath: xs:double(\"1e5\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas232() {
    // Try xs:double(1e5) castable as xs:hexBinary
    // XPath: xs:double(\"1e5\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas233() {
    // Try xs:double(1e5) castable as xs:anyURI
    // XPath: xs:double(\"1e5\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas234() {
    // Try xs:decimal(-1.1234) castable as xs:untypedAtomic
    // XPath: xs:decimal(\"-1.1234\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas235() {
    // Try xs:decimal(-1.1234) castable as xs:string
    // XPath: xs:decimal(\"-1.1234\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas236() {
    // Try xs:decimal(-1.1234) castable as xs:float
    // XPath: xs:decimal(\"-1.1234\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas237() {
    // Try xs:decimal(-1.1234) castable as xs:double
    // XPath: xs:decimal(\"-1.1234\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas238() {
    // Try xs:decimal(-1.1234) castable as xs:decimal
    // XPath: xs:decimal(\"-1.1234\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas239() {
    // Try xs:decimal(-1.1234) castable as xs:integer
    // XPath: xs:decimal(\"-1.1234\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas240() {
    // Try xs:decimal(5.5432) castable as xs:integer
    // XPath: xs:decimal(\"5.5432\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas241() {
    // Try xs:decimal(-1.1234) castable as xs:duration
    // XPath: xs:decimal(\"-1.1234\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas242() {
    // Try xs:decimal(-1.1234) castable as xs:yearMonthDuration
    // XPath: xs:decimal(\"-1.1234\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas243() {
    // Try xs:decimal(-1.1234) castable as xs:dayTimeDuration
    // XPath: xs:decimal(\"-1.1234\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas244() {
    // Try xs:decimal(-1.1234) castable as xs:dateTime
    // XPath: xs:decimal(\"-1.1234\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas245() {
    // Try xs:decimal(-1.1234) castable as xs:time
    // XPath: xs:decimal(\"-1.1234\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas246() {
    // Try xs:decimal(-1.1234) castable as xs:date
    // XPath: xs:decimal(\"-1.1234\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas247() {
    // Try xs:decimal(-1.1234) castable as xs:gYearMonth
    // XPath: xs:decimal(\"-1.1234\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas248() {
    // Try xs:decimal(-1.1234) castable as xs:gYear
    // XPath: xs:decimal(\"-1.1234\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas249() {
    // Try xs:decimal(-1.1234) castable as xs:gMonthDay
    // XPath: xs:decimal(\"-1.1234\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas250() {
    // Try xs:decimal(-1.1234) castable as xs:gDay
    // XPath: xs:decimal(\"-1.1234\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
