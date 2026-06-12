//! Test chunk 1 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas009() {
    // Try casting xs:untypedAtomic(1e-5) to xs:float
    // XPath: xs:untypedAtomic(\"1e-5\") cast as xs:float
    // Expected: 0.00001
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas010() {
    // Try casting xs:untypedAtomic(-10000000) to xs:float
    // XPath: xs:untypedAtomic(\"-10000000\") cast as xs:float
    // Expected: -1.0E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas012() {
    // Try casting xs:untypedAtomic(-0.0E0) to xs:double
    // XPath: xs:untypedAtomic(\"-0.0E0\") cast as xs:double
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas013() {
    // Try casting xs:untypedAtomic(NaN) to xs:double
    // XPath: xs:untypedAtomic(\"NaN\") cast as xs:double
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas014() {
    // Try casting xs:untypedAtomic(1e-5) to xs:double
    // XPath: xs:untypedAtomic(\"1e-5\") cast as xs:double
    // Expected: 0.00001
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas015() {
    // Try casting xs:untypedAtomic(-10000000) to xs:double
    // XPath: xs:untypedAtomic(\"-10000000\") cast as xs:double
    // Expected: -1.0E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas020() {
    // Try casting xs:untypedAtomic(5.5432) to xs:decimal
    // XPath: xs:untypedAtomic(\"5.5432\") cast as xs:decimal
    // Expected: 5.5432
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas027() {
    // Try casting xs:untypedAtomic(P1Y2M3DT10H30M23S) to xs:duration
    // XPath: xs:untypedAtomic(\"P1Y2M3DT10H30M23S\") cast as xs:duration
    // Expected: P1Y2M3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas028() {
    // Try casting xs:untypedAtomic(-P1Y1M1DT1H1M1.123S) to xs:duration
    // XPath: xs:untypedAtomic(\"-P1Y1M1DT1H1M1.123S\") cast as xs:duratio
    // Expected: -P1Y1M1DT1H1M1.123S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas031() {
    // Try casting xs:untypedAtomic(P24M) to xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"P24M\") cast as xs:yearMonthDuration
    // Expected: P2Y
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas032() {
    // Try casting xs:untypedAtomic(-P21M) to xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"-P21M\") cast as xs:yearMonthDuration
    // Expected: -P1Y9M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas035() {
    // Try casting xs:untypedAtomic(P3DT10H30M) to xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"P3DT10H30M\") cast as xs:dayTimeDuration
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas036() {
    // Try casting xs:untypedAtomic(-PT100M) to xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"-PT100M\") cast as xs:dayTimeDuration
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas038() {
    // Try casting xs:untypedAtomic(1999-05-31T13:20:00) to xs:dateTime
    // XPath: xs:untypedAtomic(\"1999-05-31T13:20:00\") cast as xs:dateTim
    // Expected: 1999-05-31T13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas039() {
    // Try casting xs:untypedAtomic(-1999-05-31T13:20:00+14:00) to xs:dateTime
    // XPath: xs:untypedAtomic(\"-1999-05-31T13:20:00+14:00\") cast as xs:
    // Expected: -1999-05-31T13:20:00+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas040() {
    // Try casting xs:untypedAtomic(2000-01-16T00:00:00Z) to xs:dateTime
    // XPath: xs:untypedAtomic(\"2000-01-16T00:00:00Z\") cast as xs:dateTi
    // Expected: 2000-01-16T00:00:00Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas042() {
    // Try casting xs:untypedAtomic(13:20:00-05:00) to xs:time
    // XPath: xs:untypedAtomic(\"13:20:00-05:00\") cast as xs:time
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas043() {
    // Try casting xs:untypedAtomic(13:20:02.123) to xs:time
    // XPath: xs:untypedAtomic(\"13:20:02.123\") cast as xs:time
    // Expected: 13:20:02.123
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas044() {
    // Try casting xs:untypedAtomic(13:20:00Z) to xs:time
    // XPath: xs:untypedAtomic(\"13:20:00Z\") cast as xs:time
    // Expected: 13:20:00Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas046() {
    // Try casting xs:untypedAtomic(1999-05-31) to xs:date
    // XPath: xs:untypedAtomic(\"1999-05-31\") cast as xs:date
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas047() {
    // Try casting xs:untypedAtomic(-0012-12-03-05:00) to xs:date
    // XPath: xs:untypedAtomic(\"-0012-12-03-05:00\") cast as xs:date
    // Expected: -0012-12-03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas048() {
    // Try casting xs:untypedAtomic(1999-05-31Z) to xs:date
    // XPath: xs:untypedAtomic(\"1999-05-31Z\") cast as xs:date
    // Expected: 1999-05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas050() {
    // Try casting xs:untypedAtomic(1999-05) to xs:gYearMonth
    // XPath: xs:untypedAtomic(\"1999-05\") cast as xs:gYearMonth
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas051() {
    // Try casting xs:untypedAtomic(-0012-12-05:00) to xs:gYearMonth
    // XPath: xs:untypedAtomic(\"-0012-12-05:00\") cast as xs:gYearMonth
    // Expected: -0012-12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas052() {
    // Try casting xs:untypedAtomic(1999-05Z) to xs:gYearMonth
    // XPath: xs:untypedAtomic(\"1999-05Z\") cast as xs:gYearMonth
    // Expected: 1999-05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas054() {
    // Try casting xs:untypedAtomic(1999) to xs:gYear
    // XPath: xs:untypedAtomic(\"1999\") cast as xs:gYear
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas055() {
    // Try casting xs:untypedAtomic(-0012-05:00) to xs:gYear
    // XPath: xs:untypedAtomic(\"-0012-05:00\") cast as xs:gYear
    // Expected: -0012-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas056() {
    // Try casting xs:untypedAtomic(1999Z) to xs:gYear
    // XPath: xs:untypedAtomic(\"1999Z\") cast as xs:gYear
    // Expected: 1999Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas058() {
    // Try casting xs:untypedAtomic(--05-31) to xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31\") cast as xs:gMonthDay
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas059() {
    // Try casting xs:untypedAtomic(--05-31+14:00) to xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31+14:00\") cast as xs:gMonthDay
    // Expected: --05-31+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas060() {
    // Try casting xs:untypedAtomic(--05-31Z) to xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31Z\") cast as xs:gMonthDay
    // Expected: --05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas062() {
    // Try casting xs:untypedAtomic(---31) to xs:gDay
    // XPath: xs:untypedAtomic(\"---31\") cast as xs:gDay
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas063() {
    // Try casting xs:untypedAtomic(---03-05:00) to xs:gDay
    // XPath: xs:untypedAtomic(\"---03-05:00\") cast as xs:gDay
    // Expected: ---03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas064() {
    // Try casting xs:untypedAtomic(---31Z) to xs:gDay
    // XPath: xs:untypedAtomic(\"---31Z\") cast as xs:gDay
    // Expected: ---31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas066() {
    // Try casting xs:untypedAtomic(--05) to xs:gMonth
    // XPath: xs:untypedAtomic(\"--05\") cast as xs:gMonth
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas067() {
    // Try casting xs:untypedAtomic(--12-05:00) to xs:gMonth
    // XPath: xs:untypedAtomic(\"--12-05:00\") cast as xs:gMonth
    // Expected: --12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas068() {
    // Try casting xs:untypedAtomic(--05Z) to xs:gMonth
    // XPath: xs:untypedAtomic(\"--05Z\") cast as xs:gMonth
    // Expected: --05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas071() {
    // Try casting xs:untypedAtomic(0) to xs:boolean
    // XPath: xs:untypedAtomic(\"0\") cast as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas072() {
    // Try casting xs:untypedAtomic(true) to xs:boolean
    // XPath: xs:untypedAtomic(\"true\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas073() {
    // Try casting xs:untypedAtomic(true) to xs:base64Binary
    // XPath: xs:untypedAtomic(\"true\") cast as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas074() {
    // Try casting xs:untypedAtomic(00000000) to xs:base64Binary
    // XPath: xs:untypedAtomic(\"00000000\") cast as xs:base64Binary
    // Expected: 00000000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas075() {
    // Try casting xs:untypedAtomic(D74D35D35D35) to xs:base64Binary
    // XPath: xs:untypedAtomic(\"D74D35D35D35\") cast as xs:base64Binary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas078() {
    // Try casting xs:untypedAtomic(0fb7) to xs:hexBinary
    // XPath: xs:untypedAtomic(\"0fb7\") cast as xs:hexBinary
    // Expected: 0FB7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas079() {
    // Try casting xs:untypedAtomic(D74D35D35D35) to xs:hexBinary
    // XPath: xs:untypedAtomic(\"D74D35D35D35\") cast as xs:hexBinary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas080() {
    // Try casting xs:untypedAtomic(-0012-05:00) to xs:anyURI
    // XPath: xs:untypedAtomic(\"http://www.example.com/~b%C3%A9b%C3%A9\")
    // Expected: http://www.example.com/~b%C3%A9b%C3%A9
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas081() {
    // Try casting xs:untypedAtomic(true) to xs:anyURI
    // XPath: xs:untypedAtomic(\"true\") cast as xs:anyURI
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas082() {
    // Try casting xs:untypedAtomic(http://www.ietf.org/rfc/rfc2396.txt) to xs:anyURI
    // XPath: xs:untypedAtomic(\"http://www.ietf.org/rfc/rfc2396.txt\") ca
    // Expected: http://www.ietf.org/rfc/rfc2396.txt
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas083() {
    // Try casting xs:string(INF) to xs:untypedAtomic
    // XPath: xs:string(\"INF\") cast as xs:untypedAtomic
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas084() {
    // Try casting xs:string(0.0E0) to xs:untypedAtomic
    // XPath: xs:string(\"0.0E0\") cast as xs:untypedAtomic
    // Expected: 0.0E0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas085() {
    // Try casting xs:string(true) to xs:untypedAtomic
    // XPath: xs:string(\"true\") cast as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
