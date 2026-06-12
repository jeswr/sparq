//! Test chunk 2 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas086() {
    // Try casting xs:string(INF) to xs:string
    // XPath: xs:string(\"INF\") cast as xs:string
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas087() {
    // Try casting xs:string(0.0E0) to xs:string
    // XPath: xs:string(\"0.0E0\") cast as xs:string
    // Expected: 0.0E0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas088() {
    // Try casting xs:string(true) to xs:string
    // XPath: xs:string(\"true\") cast as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas089() {
    // Try casting xs:string(-0.0E0) to xs:float
    // XPath: xs:string(\"-0.0E0\") cast as xs:float
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas090() {
    // Try casting xs:string(NaN) to xs:float
    // XPath: xs:string(\"NaN\") cast as xs:float
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas091() {
    // Try casting xs:string(1e-5) to xs:float
    // XPath: xs:string(\"1e-5\") cast as xs:float
    // Expected: 0.00001
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas092() {
    // Try casting xs:string(-10000000) to xs:float
    // XPath: xs:string(\"-10000000\") cast as xs:float
    // Expected: -1.0E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas094() {
    // Try casting xs:string(-0.0E0) to xs:double
    // XPath: xs:string(\"-0.0E0\") cast as xs:double
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas095() {
    // Try casting xs:string(NaN) to xs:double
    // XPath: xs:string(\"NaN\") cast as xs:double
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas096() {
    // Try casting xs:string(1e-5) to xs:double
    // XPath: xs:string(\"1e-5\") cast as xs:double
    // Expected: 0.00001
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas097() {
    // Try casting xs:string(-10000000) to xs:double
    // XPath: xs:string(\"-10000000\") cast as xs:double
    // Expected: -1.0E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas102() {
    // Try casting xs:string(5.5432) to xs:decimal
    // XPath: xs:string(\"5.5432\") cast as xs:decimal
    // Expected: 5.5432
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas109() {
    // Try casting xs:string(P1Y2M3DT10H30M23S) to xs:duration
    // XPath: xs:string(\"P1Y2M3DT10H30M23S\") cast as xs:duration
    // Expected: P1Y2M3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas110() {
    // Try casting xs:string(-P1Y1M1DT1H1M1.123S) to xs:duration
    // XPath: xs:string(\"-P1Y1M1DT1H1M1.123S\") cast as xs:duration
    // Expected: -P1Y1M1DT1H1M1.123S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas113() {
    // Try casting xs:string(P24M) to xs:yearMonthDuration
    // XPath: xs:string(\"P24M\") cast as xs:yearMonthDuration
    // Expected: P2Y
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas114() {
    // Try casting xs:string(-P21M) to xs:yearMonthDuration
    // XPath: xs:string(\"-P21M\") cast as xs:yearMonthDuration
    // Expected: -P1Y9M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas117() {
    // Try casting xs:string(P3DT10H30M) to xs:dayTimeDuration
    // XPath: xs:string(\"P3DT10H30M\") cast as xs:dayTimeDuration
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas118() {
    // Try casting xs:string(-PT100M) to xs:dayTimeDuration
    // XPath: xs:string(\"-PT100M\") cast as xs:dayTimeDuration
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas120() {
    // Try casting xs:string(1999-05-31T13:20:00) to xs:dateTime
    // XPath: xs:string(\"1999-05-31T13:20:00\") cast as xs:dateTime
    // Expected: 1999-05-31T13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas121() {
    // Try casting xs:string(-1999-05-31T13:20:00+14:00) to xs:dateTime
    // XPath: xs:string(\"-1999-05-31T13:20:00+14:00\") cast as xs:dateTim
    // Expected: -1999-05-31T13:20:00+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas122() {
    // Try casting xs:string(2000-01-16T00:00:00Z) to xs:dateTime
    // XPath: xs:string(\"2000-01-16T00:00:00Z\") cast as xs:dateTime
    // Expected: 2000-01-16T00:00:00Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas124() {
    // Try casting xs:string(13:20:00-05:00) to xs:time
    // XPath: xs:string(\"13:20:00-05:00\") cast as xs:time
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas125() {
    // Try casting xs:string(13:20:02.123) to xs:time
    // XPath: xs:string(\"13:20:02.123\") cast as xs:time
    // Expected: 13:20:02.123
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas126() {
    // Try casting xs:string(13:20:00Z) to xs:time
    // XPath: xs:string(\"13:20:00Z\") cast as xs:time
    // Expected: 13:20:00Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas128() {
    // Try casting xs:string(1999-05-31) to xs:date
    // XPath: xs:string(\"1999-05-31\") cast as xs:date
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas129() {
    // Try casting xs:string(-0012-12-03-05:00) to xs:date
    // XPath: xs:string(\"-0012-12-03-05:00\") cast as xs:date
    // Expected: -0012-12-03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas130() {
    // Try casting xs:string(1999-05-31Z) to xs:date
    // XPath: xs:string(\"1999-05-31Z\") cast as xs:date
    // Expected: 1999-05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas132() {
    // Try casting xs:string(1999-05) to xs:gYearMonth
    // XPath: xs:string(\"1999-05\") cast as xs:gYearMonth
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas133() {
    // Try casting xs:string(-0012-12-05:00) to xs:gYearMonth
    // XPath: xs:string(\"-0012-12-05:00\") cast as xs:gYearMonth
    // Expected: -0012-12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas134() {
    // Try casting xs:string(1999-05Z) to xs:gYearMonth
    // XPath: xs:string(\"1999-05Z\") cast as xs:gYearMonth
    // Expected: 1999-05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas136() {
    // Try casting xs:string(1999) to xs:gYear
    // XPath: xs:string(\"1999\") cast as xs:gYear
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas137() {
    // Try casting xs:string(-0012-05:00) to xs:gYear
    // XPath: xs:string(\"-0012-05:00\") cast as xs:gYear
    // Expected: -0012-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas138() {
    // Try casting xs:string(1999Z) to xs:gYear
    // XPath: xs:string(\"1999Z\") cast as xs:gYear
    // Expected: 1999Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas140() {
    // Try casting xs:string(--05-31) to xs:gMonthDay
    // XPath: xs:string(\"--05-31\") cast as xs:gMonthDay
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas141() {
    // Try casting xs:string(--05-31+14:00) to xs:gMonthDay
    // XPath: xs:string(\"--05-31+14:00\") cast as xs:gMonthDay
    // Expected: --05-31+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas142() {
    // Try casting xs:string(--05-31Z) to xs:gMonthDay
    // XPath: xs:string(\"--05-31Z\") cast as xs:gMonthDay
    // Expected: --05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas144() {
    // Try casting xs:string(---31) to xs:gDay
    // XPath: xs:string(\"---31\") cast as xs:gDay
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas145() {
    // Try casting xs:string(---03-05:00) to xs:gDay
    // XPath: xs:string(\"---03-05:00\") cast as xs:gDay
    // Expected: ---03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas146() {
    // Try casting xs:string(---31Z) to xs:gDay
    // XPath: xs:string(\"---31Z\") cast as xs:gDay
    // Expected: ---31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas148() {
    // Try casting xs:string(--05) to xs:gMonth
    // XPath: xs:string(\"--05\") cast as xs:gMonth
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas149() {
    // Try casting xs:string(--12-05:00) to xs:gMonth
    // XPath: xs:string(\"--12-05:00\") cast as xs:gMonth
    // Expected: --12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas150() {
    // Try casting xs:string(--05Z) to xs:gMonth
    // XPath: xs:string(\"--05Z\") cast as xs:gMonth
    // Expected: --05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas153() {
    // Try casting xs:string(0) to xs:boolean
    // XPath: xs:string(\"0\") cast as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas154() {
    // Try casting xs:string(true) to xs:boolean
    // XPath: xs:string(\"true\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas155() {
    // Try casting xs:string(true) to xs:base64Binary
    // XPath: xs:string(\"true\") cast as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas156() {
    // Try casting xs:string(00000000) to xs:base64Binary
    // XPath: xs:string(\"00000000\") cast as xs:base64Binary
    // Expected: 00000000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas157() {
    // Try casting xs:string(D74D35D35D35) to xs:base64Binary
    // XPath: xs:string(\"D74D35D35D35\") cast as xs:base64Binary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas160() {
    // Try casting xs:string(0fb7) to xs:hexBinary
    // XPath: xs:string(\"0fb7\") cast as xs:hexBinary
    // Expected: 0FB7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas161() {
    // Try casting xs:string(D74D35D35D35) to xs:hexBinary
    // XPath: xs:string(\"D74D35D35D35\") cast as xs:hexBinary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas162() {
    // Try casting xs:string(-0012-05:00) to xs:anyURI
    // XPath: xs:string(\"http://www.example.com/~b%C3%A9b%C3%A9\") cast a
    // Expected: http://www.example.com/~b%C3%A9b%C3%A9
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
