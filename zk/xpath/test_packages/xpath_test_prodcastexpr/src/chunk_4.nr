//! Test chunk 4 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas261() {
    // Try casting xs:integer(-100) to xs:integer
    // XPath: xs:integer(\"-100\") cast as xs:integer
    // Expected: -100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas273() {
    // Try casting xs:integer(-100) to xs:boolean
    // XPath: xs:integer(\"-100\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas277() {
    // Try casting xs:duration(P1Y2M3DT10H30M23S) to xs:untypedAtomic
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") cast as xs:untypedAtomic
    // Expected: P1Y2M3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas278() {
    // Try casting xs:duration(P1Y2M3DT10H30M23S) to xs:string
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") cast as xs:string
    // Expected: P1Y2M3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas283() {
    // Try casting xs:duration(P1Y2M3DT10H30M23S) to xs:duration
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") cast as xs:duration
    // Expected: P1Y2M3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas284() {
    // Try casting xs:duration(PT10H) to xs:duration
    // XPath: xs:duration(\"PT10H\") cast as xs:duration
    // Expected: PT10H
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas285() {
    // Try casting xs:duration(P1Y2M3DT10H30M23S) to xs:yearMonthDuration
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") cast as xs:yearMonthDurat
    // Expected: P1Y2M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas286() {
    // Try casting xs:duration(PT10H) to xs:yearMonthDuration
    // XPath: xs:duration(\"PT10H\") cast as xs:yearMonthDuration
    // Expected: P0M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas287() {
    // Try casting xs:duration(P24M) to xs:yearMonthDuration
    // XPath: xs:duration(\"P24M\") cast as xs:yearMonthDuration
    // Expected: P2Y
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas288() {
    // Try casting xs:duration(P1Y2M3DT10H30M23S) to xs:dayTimeDuration
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") cast as xs:dayTimeDuratio
    // Expected: P3DT10H30M23S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas289() {
    // Try casting xs:duration(PT10H) to xs:dayTimeDuration
    // XPath: xs:duration(\"PT10H\") cast as xs:dayTimeDuration
    // Expected: PT10H
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas290() {
    // Try casting xs:duration(P24M) to xs:dayTimeDuration
    // XPath: xs:duration(\"P24M\") cast as xs:dayTimeDuration
    // Expected: PT0S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas303() {
    // Try casting xs:yearMonthDuration(P1Y2M) to xs:untypedAtomic
    // XPath: xs:yearMonthDuration(\"P1Y2M\") cast as xs:untypedAtomic
    // Expected: P1Y2M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas304() {
    // Try casting xs:yearMonthDuration(P1Y2M) to xs:string
    // XPath: xs:yearMonthDuration(\"P1Y2M\") cast as xs:string
    // Expected: P1Y2M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas309() {
    // Try casting xs:yearMonthDuration(P1Y2M) to xs:duration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") cast as xs:duration
    // Expected: P1Y2M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas310() {
    // Try casting xs:yearMonthDuration(P1Y2M) to xs:yearMonthDuration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") cast as xs:yearMonthDuration
    // Expected: P1Y2M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas311() {
    // Try casting xs:yearMonthDuration(-P21M) to xs:yearMonthDuration
    // XPath: xs:yearMonthDuration(\"-P21M\") cast as xs:yearMonthDuration
    // Expected: -P1Y9M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas312() {
    // Try casting xs:yearMonthDuration(P1Y2M) to xs:dayTimeDuration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") cast as xs:dayTimeDuration
    // Expected: PT0S
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas325() {
    // Try casting xs:dayTimeDuration(P3DT10H30M) to xs:untypedAtomic
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") cast as xs:untypedAtomic
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas326() {
    // Try casting xs:dayTimeDuration(-PT100M) to xs:untypedAtomic
    // XPath: xs:dayTimeDuration(\"-PT100M\") cast as xs:untypedAtomic
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas327() {
    // Try casting xs:dayTimeDuration(P3DT10H30M) to xs:string
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") cast as xs:string
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas328() {
    // Try casting xs:dayTimeDuration(-PT100M) to xs:string
    // XPath: xs:dayTimeDuration(\"-PT100M\") cast as xs:string
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas333() {
    // Try casting xs:dayTimeDuration(P3DT10H30M) to xs:duration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") cast as xs:duration
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas334() {
    // Try casting xs:dayTimeDuration(PT24H) to xs:duration
    // XPath: xs:dayTimeDuration(\"PT24H\") cast as xs:duration
    // Expected: P1D
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas335() {
    // Try casting xs:dayTimeDuration(-PT100M) to xs:duration
    // XPath: xs:dayTimeDuration(\"-PT100M\") cast as xs:duration
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas336() {
    // Try casting xs:dayTimeDuration(P3DT10H30M) to xs:yearMonthDuration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") cast as xs:yearMonthDurat
    // Expected: P0M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas337() {
    // Try casting xs:dayTimeDuration(P3DT10H30M) to xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") cast as xs:dayTimeDuratio
    // Expected: P3DT10H30M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas338() {
    // Try casting xs:dayTimeDuration(P14D) to xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"P14D\") cast as xs:dayTimeDuration
    // Expected: P14D
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas339() {
    // Try casting xs:dayTimeDuration(-PT100M) to xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"-PT100M\") cast as xs:dayTimeDuration
    // Expected: -PT1H40M
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas352() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:untypedAtomic
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:untypedAtomi
    // Expected: 1999-05-31T13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas353() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:untypedAtomic
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:untyp
    // Expected: -1999-05-31T13:20:00+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas354() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:string
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:string
    // Expected: 1999-05-31T13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas355() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:string
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:strin
    // Expected: -1999-05-31T13:20:00+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas363() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:dateTime
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:dateTime
    // Expected: 1999-05-31T13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas364() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:time
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:time
    // Expected: 13:20:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas365() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:time
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:time
    // Expected: 13:20:00+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas366() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:date
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:date
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas367() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:date
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:date
    // Expected: -1999-05-31+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas368() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:gYearMonth
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:gYearMonth
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas369() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:gYearMonth
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:gYear
    // Expected: -1999-05+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas370() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:gYear
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:gYear
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas371() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:gYear
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:gYear
    // Expected: -1999+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas372() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:gMonthDay
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:gMonthDay
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas373() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:gMonthDay
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:gMont
    // Expected: --05-31+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas374() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:gDay
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:gDay
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas375() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:gDay
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:gDay
    // Expected: ---31+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas376() {
    // Try casting xs:dateTime(1999-05-31T13:20:00) to xs:gMonth
    // XPath: xs:dateTime(\"1999-05-31T13:20:00\") cast as xs:gMonth
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas377() {
    // Try casting xs:dateTime(-1999-05-31T13:20:00+14:00) to xs:gMonth
    // XPath: xs:dateTime(\"-1999-05-31T13:20:00+14:00\") cast as xs:gMont
    // Expected: --05+14:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas382() {
    // Try casting xs:time(13:20:00-05:00) to xs:untypedAtomic
    // XPath: xs:time(\"13:20:00-05:00\") cast as xs:untypedAtomic
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas383() {
    // Try casting xs:time(13:20:00-05:00) to xs:string
    // XPath: xs:time(\"13:20:00-05:00\") cast as xs:string
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
