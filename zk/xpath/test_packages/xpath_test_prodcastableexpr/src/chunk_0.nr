//! Test chunk 0 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas001() {
    // Try xs:untypedAtomic(INF) castable as xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"INF\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas002() {
    // Try xs:untypedAtomic(0.0E0) castable as xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"0.0E0\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas003() {
    // Try xs:untypedAtomic(true) castable as xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"true\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas004() {
    // Try xs:untypedAtomic(INF) castable as xs:string
    // XPath: xs:untypedAtomic(\"INF\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas005() {
    // Try xs:untypedAtomic(0.0E0) castable as xs:string
    // XPath: xs:untypedAtomic(\"0.0E0\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas006() {
    // Try xs:untypedAtomic(true) castable as xs:string
    // XPath: xs:untypedAtomic(\"true\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas007() {
    // Try xs:untypedAtomic(-0.0E0) castable as xs:float
    // XPath: xs:untypedAtomic(\"-0.0E0\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas008() {
    // Try xs:untypedAtomic(NaN) castable as xs:float
    // XPath: xs:untypedAtomic(\"NaN\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas009() {
    // Try xs:untypedAtomic(1e-5) castable as xs:float
    // XPath: xs:untypedAtomic(\"1e-5\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas010() {
    // Try xs:untypedAtomic(-10000000) castable as xs:float
    // XPath: xs:untypedAtomic(\"-10000000\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas011() {
    // Try xs:untypedAtomic(true) castable as xs:float
    // XPath: xs:untypedAtomic(\"true\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas012() {
    // Try xs:untypedAtomic(-0.0E0) castable as xs:double
    // XPath: xs:untypedAtomic(\"-0.0E0\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas013() {
    // Try xs:untypedAtomic(NaN) castable as xs:double
    // XPath: xs:untypedAtomic(\"NaN\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas014() {
    // Try xs:untypedAtomic(1e-5) castable as xs:double
    // XPath: xs:untypedAtomic(\"1e-5\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas015() {
    // Try xs:untypedAtomic(-10000000) castable as xs:double
    // XPath: xs:untypedAtomic(\"-10000000\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas016() {
    // Try xs:untypedAtomic(true) castable as xs:double
    // XPath: xs:untypedAtomic(\"true\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas017() {
    // Try xs:untypedAtomic(-0.0E0) castable as xs:decimal
    // XPath: xs:untypedAtomic(\"-0.0E0\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas018() {
    // Try xs:untypedAtomic(NaN) castable as xs:decimal
    // XPath: xs:untypedAtomic(\"NaN\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas019() {
    // Try xs:untypedAtomic(1e-5) castable as xs:decimal
    // XPath: xs:untypedAtomic(\"1e-5\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas020() {
    // Try xs:untypedAtomic(5.5432) castable as xs:decimal
    // XPath: xs:untypedAtomic(\"5.5432\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas021() {
    // Try xs:untypedAtomic(true) castable as xs:decimal
    // XPath: xs:untypedAtomic(\"true\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas022() {
    // Try xs:untypedAtomic(-0.0E0) castable as xs:integer
    // XPath: xs:untypedAtomic(\"-0.0E0\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas023() {
    // Try xs:untypedAtomic(NaN) castable as xs:integer
    // XPath: xs:untypedAtomic(\"NaN\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas024() {
    // Try xs:untypedAtomic(1e-5) castable as xs:integer
    // XPath: xs:untypedAtomic(\"1e-5\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas025() {
    // Try xs:untypedAtomic(-1.1234) castable as xs:integer
    // XPath: xs:untypedAtomic(\"-1.1234\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas026() {
    // Try xs:untypedAtomic(true) castable as xs:integer
    // XPath: xs:untypedAtomic(\"true\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas027() {
    // Try xs:untypedAtomic(P1Y2M3DT10H30M23S) castable as xs:duration
    // XPath: xs:untypedAtomic(\"P1Y2M3DT10H30M23S\") castable as xs:durat
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas028() {
    // Try xs:untypedAtomic(-P1Y1M1DT1H1M1.123S) castable as xs:duration
    // XPath: xs:untypedAtomic(\"-P1Y1M1DT1H1M1.123S\") castable as xs:dur
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas029() {
    // Try xs:untypedAtomic(true) castable as xs:duration
    // XPath: xs:untypedAtomic(\"true\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas030() {
    // Try xs:untypedAtomic(-P1Y1M1DT1H1M1.123S) castable as xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"-P1Y1M1DT1H1M1.123S\") castable as xs:yea
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas031() {
    // Try xs:untypedAtomic(P24M) castable as xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"P24M\") castable as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas032() {
    // Try xs:untypedAtomic(-P21M) castable as xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"-P21M\") castable as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas033() {
    // Try xs:untypedAtomic(true) castable as xs:yearMonthDuration
    // XPath: xs:untypedAtomic(\"true\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas034() {
    // Try xs:untypedAtomic(-P1Y1M1DT1H1M1.123S) castable as xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"-P1Y1M1DT1H1M1.123S\") castable as xs:day
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas035() {
    // Try xs:untypedAtomic(P3DT10H30M) castable as xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"P3DT10H30M\") castable as xs:dayTimeDurat
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas036() {
    // Try xs:untypedAtomic(-PT100M) castable as xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"-PT100M\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas037() {
    // Try xs:untypedAtomic(true) castable as xs:dayTimeDuration
    // XPath: xs:untypedAtomic(\"true\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas038() {
    // Try xs:untypedAtomic(1999-05-31T13:20:00) castable as xs:dateTime
    // XPath: xs:untypedAtomic(\"1999-05-31T13:20:00\") castable as xs:dat
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas039() {
    // Try xs:untypedAtomic(-1999-05-31T13:20:00+14:00) castable as xs:dateTime
    // XPath: xs:untypedAtomic(\"-1999-05-31T13:20:00+14:00\") castable as
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas040() {
    // Try xs:untypedAtomic(2000-01-16T00:00:00Z) castable as xs:dateTime
    // XPath: xs:untypedAtomic(\"2000-01-16T00:00:00Z\") castable as xs:da
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas041() {
    // Try xs:untypedAtomic(true) castable as xs:dateTime
    // XPath: xs:untypedAtomic(\"true\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas042() {
    // Try xs:untypedAtomic(13:20:00-05:00) castable as xs:time
    // XPath: xs:untypedAtomic(\"13:20:00-05:00\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas043() {
    // Try xs:untypedAtomic(13:20:02.123) castable as xs:time
    // XPath: xs:untypedAtomic(\"13:20:02.123\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas044() {
    // Try xs:untypedAtomic(13:20:00Z) castable as xs:time
    // XPath: xs:untypedAtomic(\"13:20:00Z\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas045() {
    // Try xs:untypedAtomic(true) castable as xs:time
    // XPath: xs:untypedAtomic(\"true\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas046() {
    // Try xs:untypedAtomic(1999-05-31) castable as xs:date
    // XPath: xs:untypedAtomic(\"1999-05-31\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas047() {
    // Try xs:untypedAtomic(-0012-12-03-05:00) castable as xs:date
    // XPath: xs:untypedAtomic(\"-0012-12-03-05:00\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas048() {
    // Try xs:untypedAtomic(1999-05-31Z) castable as xs:date
    // XPath: xs:untypedAtomic(\"1999-05-31Z\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas049() {
    // Try xs:untypedAtomic(true) castable as xs:date
    // XPath: xs:untypedAtomic(\"true\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas050() {
    // Try xs:untypedAtomic(1999-05) castable as xs:gYearMonth
    // XPath: xs:untypedAtomic(\"1999-05\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
