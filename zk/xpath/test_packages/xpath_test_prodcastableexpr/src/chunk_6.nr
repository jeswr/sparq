//! Test chunk 6 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas301() {
    // Try xs:duration(P1Y2M3DT10H30M23S) castable as xs:hexBinary
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas302() {
    // Try xs:duration(P1Y2M3DT10H30M23S) castable as xs:anyURI
    // XPath: xs:duration(\"P1Y2M3DT10H30M23S\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas303() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:untypedAtomic
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas304() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:string
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas305() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:float
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas306() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:double
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas307() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:decimal
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas308() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:integer
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas309() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:duration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas310() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:yearMonthDuration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:yearMonthDura
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas311() {
    // Try xs:yearMonthDuration(-P21M) castable as xs:yearMonthDuration
    // XPath: xs:yearMonthDuration(\"-P21M\") castable as xs:yearMonthDura
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas312() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:dayTimeDuration
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:dayTimeDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas313() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:dateTime
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas314() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:time
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas315() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:date
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas316() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:gYearMonth
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas317() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:gYear
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas318() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:gMonthDay
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas319() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:gDay
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas320() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:gMonth
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas321() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:boolean
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas322() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:base64Binary
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas323() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:hexBinary
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas324() {
    // Try xs:yearMonthDuration(P1Y2M) castable as xs:anyURI
    // XPath: xs:yearMonthDuration(\"P1Y2M\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas325() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:untypedAtomic
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:untypedAto
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas326() {
    // Try xs:dayTimeDuration(-PT100M) castable as xs:untypedAtomic
    // XPath: xs:dayTimeDuration(\"-PT100M\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas327() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:string
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas328() {
    // Try xs:dayTimeDuration(-PT100M) castable as xs:string
    // XPath: xs:dayTimeDuration(\"-PT100M\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas329() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:float
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas330() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:double
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas331() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:decimal
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas332() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:integer
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas333() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:duration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas334() {
    // Try xs:dayTimeDuration(PT24H) castable as xs:duration
    // XPath: xs:dayTimeDuration(\"PT24H\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas335() {
    // Try xs:dayTimeDuration(-PT100M) castable as xs:duration
    // XPath: xs:dayTimeDuration(\"-PT100M\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas336() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:yearMonthDuration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:yearMonthD
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas337() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:dayTimeDur
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas338() {
    // Try xs:dayTimeDuration(P14D) castable as xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"P14D\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas339() {
    // Try xs:dayTimeDuration(-PT100M) castable as xs:dayTimeDuration
    // XPath: xs:dayTimeDuration(\"-PT100M\") castable as xs:dayTimeDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas340() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:dateTime
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas341() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:time
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas342() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:date
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas343() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:gYearMonth
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas344() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:gYear
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas345() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:gMonthDay
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas346() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:gDay
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas347() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:gMonth
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas348() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:boolean
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas349() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:base64Binary
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:base64Bina
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas350() {
    // Try xs:dayTimeDuration(P3DT10H30M) castable as xs:hexBinary
    // XPath: xs:dayTimeDuration(\"P3DT10H30M\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
