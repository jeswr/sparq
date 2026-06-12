//! Test chunk 9 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas457() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:hexBinary
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas458() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:anyURI
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas459() {
    // Try xs:gYear(1999) castable as xs:untypedAtomic
    // XPath: xs:gYear(\"1999\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas460() {
    // Try xs:gYear(1999) castable as xs:string
    // XPath: xs:gYear(\"1999\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas461() {
    // Try xs:gYear(1999) castable as xs:float
    // XPath: xs:gYear(\"1999\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas462() {
    // Try xs:gYear(1999) castable as xs:double
    // XPath: xs:gYear(\"1999\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas463() {
    // Try xs:gYear(1999) castable as xs:decimal
    // XPath: xs:gYear(\"1999\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas464() {
    // Try xs:gYear(1999) castable as xs:integer
    // XPath: xs:gYear(\"1999\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas465() {
    // Try xs:gYear(1999) castable as xs:duration
    // XPath: xs:gYear(\"1999\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas466() {
    // Try xs:gYear(1999) castable as xs:yearMonthDuration
    // XPath: xs:gYear(\"1999\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas467() {
    // Try xs:gYear(1999) castable as xs:dateTime
    // XPath: xs:gYear(\"1999\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas468() {
    // Try xs:gYear(1999) castable as xs:time
    // XPath: xs:gYear(\"1999\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas469() {
    // Try xs:gYear(1999) castable as xs:date
    // XPath: xs:gYear(\"1999\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas470() {
    // Try xs:gYear(1999) castable as xs:gYearMonth
    // XPath: xs:gYear(\"1999\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas471() {
    // Try xs:gYear(1999) castable as xs:gYear
    // XPath: xs:gYear(\"1999\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas472() {
    // Try xs:gYear(-0012-05:00) castable as xs:gYear
    // XPath: xs:gYear(\"-0012-05:00\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas473() {
    // Try xs:gYear(1999Z) castable as xs:gYear
    // XPath: xs:gYear(\"1999Z\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas474() {
    // Try xs:gYear(1999) castable as xs:gMonthDay
    // XPath: xs:gYear(\"1999\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas475() {
    // Try xs:gYear(1999) castable as xs:gDay
    // XPath: xs:gYear(\"1999\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas476() {
    // Try xs:gYear(1999) castable as xs:gMonth
    // XPath: xs:gYear(\"1999\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas477() {
    // Try xs:gYear(1999) castable as xs:boolean
    // XPath: xs:gYear(\"1999\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas478() {
    // Try xs:gYear(1999) castable as xs:base64Binary
    // XPath: xs:gYear(\"1999\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas479() {
    // Try xs:gYear(1999) castable as xs:hexBinary
    // XPath: xs:gYear(\"1999\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas480() {
    // Try xs:gYear(1999) castable as xs:anyURI
    // XPath: xs:gYear(\"1999\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas481() {
    // Try xs:gMonthDay(--05-31) castable as xs:untypedAtomic
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas482() {
    // Try xs:gMonthDay(--05-31) castable as xs:string
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas483() {
    // Try xs:gMonthDay(--05-31) castable as xs:float
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas484() {
    // Try xs:gMonthDay(--05-31) castable as xs:double
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas485() {
    // Try xs:gMonthDay(--05-31) castable as xs:decimal
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas486() {
    // Try xs:gMonthDay(--05-31) castable as xs:integer
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas487() {
    // Try xs:gMonthDay(--05-31) castable as xs:duration
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas488() {
    // Try xs:gMonthDay(--05-31) castable as xs:yearMonthDuration
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas489() {
    // Try xs:gMonthDay(--05-31) castable as xs:dayTimeDuration
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas490() {
    // Try xs:gMonthDay(--05-31) castable as xs:dateTime
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas491() {
    // Try xs:gMonthDay(--05-31) castable as xs:time
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas492() {
    // Try xs:gMonthDay(--05-31) castable as xs:date
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas493() {
    // Try xs:gMonthDay(--05-31) castable as xs:gYearMonth
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas494() {
    // Try xs:gMonthDay(--05-31) castable as xs:gYear
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas495() {
    // Try xs:gMonthDay(--05-31) castable as xs:gMonthDay
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas496() {
    // Try xs:gMonthDay(--12-03-05:00) castable as xs:gMonthDay
    // XPath: xs:gMonthDay(\"--12-03-05:00\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas497() {
    // Try xs:gMonthDay(--05-31Z) castable as xs:gMonthDay
    // XPath: xs:gMonthDay(\"--05-31Z\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas498() {
    // Try xs:gMonthDay(--05-31) castable as xs:gDay
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas499() {
    // Try xs:gMonthDay(--05-31) castable as xs:gMonth
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas500() {
    // Try xs:gMonthDay(--05-31) castable as xs:boolean
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas501() {
    // Try xs:gMonthDay(--05-31) castable as xs:base64Binary
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas502() {
    // Try xs:gMonthDay(--05-31) castable as xs:hexBinary
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas503() {
    // Try xs:gMonthDay(--05-31) castable as xs:anyURI
    // XPath: xs:gMonthDay(\"--05-31\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas504() {
    // Try xs:gDay(---31) castable as xs:untypedAtomic
    // XPath: xs:gDay(\"---31\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas505() {
    // Try xs:gDay(---31) castable as xs:string
    // XPath: xs:gDay(\"---31\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas506() {
    // Try xs:gDay(---31) castable as xs:float
    // XPath: xs:gDay(\"---31\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
