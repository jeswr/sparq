//! Test chunk 7 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas351() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:anyURI
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas352() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:untypedAtomic
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:untypedA
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas353() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:untypedAtomic
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:u
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas354() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:string
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas355() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:string
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:s
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas356() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:float
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas357() {
    // Try xs:dateTime(2000-01-16T00:00:00Z) castable as xs:float
    // XPath: xs:dateTime(\"2000-01-16T00:00:00Z\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas358() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:decimal
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas359() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:integer
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas360() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:duration
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas361() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:yearMonthDuration
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:yearMont
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas362() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:dayTimeDuration
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:dayTimeD
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas363() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:dateTime
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas364() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:time
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas365() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:time
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:t
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas366() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:date
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas367() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:date
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:d
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas368() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:gYearMonth
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:gYearMon
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas369() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:gYearMonth
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:g
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas370() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:gYear
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas371() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:gYear
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:g
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas372() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:gMonthDay
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:gMonthDa
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas373() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:gMonthDay
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:g
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas374() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:gDay
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas375() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:gDay
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:g
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas376() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:gMonth
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas377() {
    // Try xs:dateTime(-1999-05-31T13:20:00+14:00) castable as xs:gMonth
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") castable as xs:g
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas378() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:boolean
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas379() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:base64Binary
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:base64Bi
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas380() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:hexBinary
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:hexBinar
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas381() {
    // Try xs:dateTime(1999-05-31T13:20:00) castable as xs:anyURI
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas382() {
    // Try xs:time(13:20:00-05:00) castable as xs:untypedAtomic
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas383() {
    // Try xs:time(13:20:00-05:00) castable as xs:string
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas384() {
    // Try xs:time(13:20:00-05:00) castable as xs:float
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas385() {
    // Try xs:time(13:20:00-05:00) castable as xs:double
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas386() {
    // Try xs:time(13:20:00-05:00) castable as xs:decimal
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas387() {
    // Try xs:time(13:20:00-05:00) castable as xs:integer
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas388() {
    // Try xs:time(13:20:00-05:00) castable as xs:duration
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas389() {
    // Try xs:time(13:20:00-05:00) castable as xs:yearMonthDuration
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas390() {
    // Try xs:time(13:20:00-05:00) castable as xs:dayTimeDuration
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas391() {
    // Try xs:time(13:20:00-05:00) castable as xs:dateTime
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas392() {
    // Try xs:time(13:20:00-05:00) castable as xs:time
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas393() {
    // Try xs:time(13:20:02.123) castable as xs:time
    // XPath: xs:time(\"13:20:02.123\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas394() {
    // Try xs:time(13:20:00Z) castable as xs:time
    // XPath: xs:time(\"13:20:00Z\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas395() {
    // Try xs:time(13:20:00-05:00) castable as xs:date
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas396() {
    // Try xs:time(13:20:00-05:00) castable as xs:gYearMonth
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas397() {
    // Try xs:time(13:20:00-05:00) castable as xs:gYear
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas398() {
    // Try xs:time(13:20:00-05:00) castable as xs:gMonthDay
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas399() {
    // Try xs:time(13:20:00-05:00) castable as xs:gDay
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas400() {
    // Try xs:time(13:20:00-05:00) castable as xs:gMonth
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
