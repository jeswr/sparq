//! Test chunk 2 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas101() {
    // Try xs:string(1e-5) castable as xs:decimal
    // XPath: xs:string(\"1e-5\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas102() {
    // Try xs:string(5.5432) castable as xs:decimal
    // XPath: xs:string(\"5.5432\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas103() {
    // Try xs:string(true) castable as xs:decimal
    // XPath: xs:string(\"true\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas104() {
    // Try xs:string(-0.0E0) castable as xs:integer
    // XPath: xs:string(\"-0.0E0\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas105() {
    // Try xs:string(NaN) castable as xs:integer
    // XPath: xs:string(\"NaN\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas106() {
    // Try xs:string(1e-5) castable as xs:integer
    // XPath: xs:string(\"1e-5\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas107() {
    // Try xs:string(-1.1234) castable as xs:integer
    // XPath: xs:string(\"-1.1234\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas108() {
    // Try xs:string(true) castable as xs:integer
    // XPath: xs:string(\"true\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas109() {
    // Try xs:string(P1Y2M3DT10H30M23S) castable as xs:duration
    // XPath: xs:string(\"P1Y2M3DT10H30M23S\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas110() {
    // Try xs:string(-P1Y1M1DT1H1M1.123S) castable as xs:duration
    // XPath: xs:string(\"-P1Y1M1DT1H1M1.123S\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas111() {
    // Try xs:string(true) castable as xs:duration
    // XPath: xs:string(\"true\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas112() {
    // Try xs:string(-P1Y1M1DT1H1M1.123S) castable as xs:yearMonthDuration
    // XPath: xs:string(\"-P1Y1M1DT1H1M1.123S\") castable as xs:yearMonthD
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas113() {
    // Try xs:string(P24M) castable as xs:yearMonthDuration
    // XPath: xs:string(\"P24M\") castable as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas114() {
    // Try xs:string(-P21M) castable as xs:yearMonthDuration
    // XPath: xs:string(\"-P21M\") castable as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas115() {
    // Try xs:string(true) castable as xs:yearMonthDuration
    // XPath: xs:string(\"true\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas116() {
    // Try xs:string(-P1Y1M1DT1H1M1.123S) castable as xs:dayTimeDuration
    // XPath: xs:string(\"-P1Y1M1DT1H1M1.123S\") castable as xs:dayTimeDur
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas117() {
    // Try xs:string(P3DT10H30M) castable as xs:dayTimeDuration
    // XPath: xs:string(\"P3DT10H30M\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas118() {
    // Try xs:string(-PT100M) castable as xs:dayTimeDuration
    // XPath: xs:string(\"-PT100M\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas119() {
    // Try xs:string(true) castable as xs:dayTimeDuration
    // XPath: xs:string(\"true\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas120() {
    // Try xs:string(1999-05-31T13:20:00) castable as xs:dateTime
    // XPath: xs:string(\"1999-05-31T13:20:00\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas121() {
    // Try xs:string(-1999-05-31T13:20:00+14:00) castable as xs:dateTime
    // XPath: xs:string(\"-1999-05-31T13:20:00+14:00\") castable as xs:dat
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas122() {
    // Try xs:string(2000-01-16T00:00:00Z) castable as xs:dateTime
    // XPath: xs:string(\"2000-01-16T00:00:00Z\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas123() {
    // Try xs:string(true) castable as xs:dateTime
    // XPath: xs:string(\"true\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas124() {
    // Try xs:string(13:20:00-05:00) castable as xs:time
    // XPath: xs:string(\"13:20:00-05:00\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas125() {
    // Try xs:string(13:20:02.123) castable as xs:time
    // XPath: xs:string(\"13:20:02.123\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas126() {
    // Try xs:string(13:20:00Z) castable as xs:time
    // XPath: xs:string(\"13:20:00Z\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas127() {
    // Try xs:string(true) castable as xs:time
    // XPath: xs:string(\"true\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas128() {
    // Try xs:string(1999-05-31) castable as xs:date
    // XPath: xs:string(\"1999-05-31\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas129() {
    // Try xs:string(-0012-12-03-05:00) castable as xs:date
    // XPath: xs:string(\"-0012-12-03-05:00\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas130() {
    // Try xs:string(1999-05-31Z) castable as xs:date
    // XPath: xs:string(\"1999-05-31Z\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas131() {
    // Try xs:string(true) castable as xs:date
    // XPath: xs:string(\"true\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas132() {
    // Try xs:string(1999-05) castable as xs:gYearMonth
    // XPath: xs:string(\"1999-05\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas133() {
    // Try xs:string(-0012-12-05:00) castable as xs:gYearMonth
    // XPath: xs:string(\"-0012-12-05:00\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas134() {
    // Try xs:string(1999-05Z) castable as xs:gYearMonth
    // XPath: xs:string(\"1999-05Z\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas135() {
    // Try xs:string(true) castable as xs:gYearMonth
    // XPath: xs:string(\"true\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas136() {
    // Try xs:string(1999) castable as xs:gYear
    // XPath: xs:string(\"1999\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas137() {
    // Try xs:string(-0012-05:00) castable as xs:gYear
    // XPath: xs:string(\"-0012-05:00\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas138() {
    // Try xs:string(1999Z) castable as xs:gYear
    // XPath: xs:string(\"1999Z\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas139() {
    // Try xs:string(true) castable as xs:gYear
    // XPath: xs:string(\"true\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas140() {
    // Try xs:string(--05-31) castable as xs:gMonthDay
    // XPath: xs:string(\"--05-31\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas141() {
    // Try xs:string(--05-31+14:00) castable as xs:gMonthDay
    // XPath: xs:string(\"--05-31+14:00\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas142() {
    // Try xs:string(--05-31Z) castable as xs:gMonthDay
    // XPath: xs:string(\"--05-31Z\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas143() {
    // Try xs:string(true) castable as xs:gMonthDay
    // XPath: xs:string(\"true\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas144() {
    // Try xs:string(---31) castable as xs:gDay
    // XPath: xs:string(\"---31\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas145() {
    // Try xs:string(---03-05:00) castable as xs:gDay
    // XPath: xs:string(\"---03-05:00\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas146() {
    // Try xs:string(---31Z) castable as xs:gDay
    // XPath: xs:string(\"---31Z\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas147() {
    // Try xs:string(true) castable as xs:gDay
    // XPath: xs:string(\"true\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas148() {
    // Try xs:string(--05) castable as xs:gMonth
    // XPath: xs:string(\"--05\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas149() {
    // Try xs:string(--12-05:00) castable as xs:gMonth
    // XPath: xs:string(\"--12-05:00\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas150() {
    // Try xs:string(--05Z) castable as xs:gMonth
    // XPath: xs:string(\"--05Z\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
