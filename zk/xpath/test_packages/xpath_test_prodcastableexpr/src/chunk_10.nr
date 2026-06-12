//! Test chunk 10 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas507() {
    // Try xs:gDay(---31) castable as xs:double
    // XPath: xs:gDay(\"---31\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas508() {
    // Try xs:gDay(---31) castable as xs:decimal
    // XPath: xs:gDay(\"---31\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas509() {
    // Try xs:gDay(---31) castable as xs:integer
    // XPath: xs:gDay(\"---31\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas510() {
    // Try xs:gDay(---31) castable as xs:duration
    // XPath: xs:gDay(\"---31\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas511() {
    // Try xs:gDay(---31) castable as xs:yearMonthDuration
    // XPath: xs:gDay(\"---31\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas512() {
    // Try xs:gDay(---31) castable as xs:dayTimeDuration
    // XPath: xs:gDay(\"---31\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas513() {
    // Try xs:gDay(---31) castable as xs:dateTime
    // XPath: xs:gDay(\"---31\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas514() {
    // Try xs:gDay(---31) castable as xs:time
    // XPath: xs:gDay(\"---31\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas515() {
    // Try xs:gDay(---31) castable as xs:date
    // XPath: xs:gDay(\"---31\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas516() {
    // Try xs:gDay(---31) castable as xs:gYearMonth
    // XPath: xs:gDay(\"---31\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas517() {
    // Try xs:gDay(---31) castable as xs:gYear
    // XPath: xs:gDay(\"---31\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas518() {
    // Try xs:gDay(---31) castable as xs:gMonthDay
    // XPath: xs:gDay(\"---31\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas519() {
    // Try xs:gDay(---31) castable as xs:gDay
    // XPath: xs:gDay(\"---31\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas520() {
    // Try xs:gDay(---03-05:00) castable as xs:gDay
    // XPath: xs:gDay(\"---03-05:00\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas521() {
    // Try xs:gDay(---31Z) castable as xs:gDay
    // XPath: xs:gDay(\"---31Z\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas522() {
    // Try xs:gDay(---31) castable as xs:gMonth
    // XPath: xs:gDay(\"---31\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas523() {
    // Try xs:gDay(---31) castable as xs:boolean
    // XPath: xs:gDay(\"---31\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas524() {
    // Try xs:gDay(---31) castable as xs:base64Binary
    // XPath: xs:gDay(\"---31\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas525() {
    // Try xs:gDay(---31) castable as xs:hexBinary
    // XPath: xs:gDay(\"---31\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas526() {
    // Try xs:gDay(---31) castable as xs:anyURI
    // XPath: xs:gDay(\"---31\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas527() {
    // Try xs:gMonth(--05) castable as xs:untypedAtomic
    // XPath: xs:gMonth(\"--05\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas528() {
    // Try xs:gMonth(--05) castable as xs:string
    // XPath: xs:gMonth(\"--05\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas529() {
    // Try xs:gMonth(--05) castable as xs:float
    // XPath: xs:gMonth(\"--05\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas530() {
    // Try xs:gMonth(--05) castable as xs:double
    // XPath: xs:gMonth(\"--05\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas531() {
    // Try xs:gMonth(--05) castable as xs:decimal
    // XPath: xs:gMonth(\"--05\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas532() {
    // Try xs:gMonth(--05) castable as xs:integer
    // XPath: xs:gMonth(\"--05\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas533() {
    // Try xs:gMonth(--05) castable as xs:duration
    // XPath: xs:gMonth(\"--05\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas534() {
    // Try xs:gMonth(--05) castable as xs:yearMonthDuration
    // XPath: xs:gMonth(\"--05\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas535() {
    // Try xs:gMonth(--05) castable as xs:dayTimeDuration
    // XPath: xs:gMonth(\"--05\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas536() {
    // Try xs:gMonth(--05) castable as xs:dateTime
    // XPath: xs:gMonth(\"--05\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas537() {
    // Try xs:gMonth(--05) castable as xs:time
    // XPath: xs:gMonth(\"--05\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas538() {
    // Try xs:gMonth(--05) castable as xs:date
    // XPath: xs:gMonth(\"--05\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas539() {
    // Try xs:gMonth(--05) castable as xs:gYearMonth
    // XPath: xs:gMonth(\"--05\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas540() {
    // Try xs:gMonth(--05) castable as xs:gYear
    // XPath: xs:gMonth(\"--05\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas541() {
    // Try xs:gMonth(--05) castable as xs:gMonthDay
    // XPath: xs:gMonth(\"--05\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas542() {
    // Try xs:gMonth(--05) castable as xs:gDay
    // XPath: xs:gMonth(\"--05\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas543() {
    // Try xs:gMonth(--05) castable as xs:gMonth
    // XPath: xs:gMonth(\"--05\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas544() {
    // Try xs:gMonth(--12-05:00) castable as xs:gMonth
    // XPath: xs:gMonth(\"--12-05:00\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas545() {
    // Try xs:gMonth(--05Z) castable as xs:gMonth
    // XPath: xs:gMonth(\"--05Z\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas546() {
    // Try xs:gMonth(--05) castable as xs:boolean
    // XPath: xs:gMonth(\"--05\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas547() {
    // Try xs:gMonth(--05) castable as xs:base64Binary
    // XPath: xs:gMonth(\"--05\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas548() {
    // Try xs:gMonth(--05) castable as xs:hexBinary
    // XPath: xs:gMonth(\"--05\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas549() {
    // Try xs:gMonth(--05) castable as xs:anyURI
    // XPath: xs:gMonth(\"--05\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas550() {
    // Try xs:boolean(true) castable as xs:untypedAtomic
    // XPath: xs:boolean(\"true\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas551() {
    // Try xs:boolean(true) castable as xs:string
    // XPath: xs:boolean(\"true\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas552() {
    // Try xs:boolean(true) castable as xs:float
    // XPath: xs:boolean(\"true\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas553() {
    // Try xs:boolean(false) castable as xs:float
    // XPath: xs:boolean(\"false\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas554() {
    // Try xs:boolean(true) castable as xs:double
    // XPath: xs:boolean(\"true\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas555() {
    // Try xs:boolean(false) castable as xs:double
    // XPath: xs:boolean(\"false\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas556() {
    // Try xs:boolean(true) castable as xs:decimal
    // XPath: xs:boolean(\"true\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
