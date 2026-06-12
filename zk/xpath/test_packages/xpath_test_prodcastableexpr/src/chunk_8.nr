//! Test chunk 8 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas401() {
    // Try xs:time(13:20:00-05:00) castable as xs:boolean
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas402() {
    // Try xs:time(13:20:00-05:00) castable as xs:base64Binary
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas403() {
    // Try xs:time(13:20:00-05:00) castable as xs:hexBinary
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas404() {
    // Try xs:time(13:20:00-05:00) castable as xs:anyURI
    // XPath: xs:time(\"13:20:00-05:00\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas405() {
    // Try xs:date(1999-05-31) castable as xs:untypedAtomic
    // XPath: xs:date(\"1999-05-31\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas407() {
    // Try xs:date(1999-05-31) castable as xs:string
    // XPath: xs:date(\"1999-05-31\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas409() {
    // Try xs:date(1999-05-31) castable as xs:float
    // XPath: xs:date(\"1999-05-31\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas410() {
    // Try xs:date(1999-05-31) castable as xs:double
    // XPath: xs:date(\"1999-05-31\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas411() {
    // Try xs:date(1999-05-31Z) castable as xs:double
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas412() {
    // Try xs:date(1999-05-31) castable as xs:integer
    // XPath: xs:date(\"1999-05-31\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas413() {
    // Try xs:date(1999-05-31) castable as xs:duration
    // XPath: xs:date(\"1999-05-31\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas414() {
    // Try xs:date(1999-05-31) castable as xs:yearMonthDuration
    // XPath: xs:date(\"1999-05-31\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas415() {
    // Try xs:date(1999-05-31) castable as xs:dayTimeDuration
    // XPath: xs:date(\"1999-05-31\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas416() {
    // Try xs:date(1999-05-31) castable as xs:dateTime
    // XPath: xs:date(\"1999-05-31\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas417() {
    // Try xs:date(1999-05-31) castable as xs:time
    // XPath: xs:date(\"1999-05-31\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas418() {
    // Try xs:date(1999-05-31) castable as xs:date
    // XPath: xs:date(\"1999-05-31\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas420() {
    // Try xs:date(1999-05-31Z) castable as xs:date
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas421() {
    // Try xs:date(1999-05-31) castable as xs:gYearMonth
    // XPath: xs:date(\"1999-05-31\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas422() {
    // Try xs:date(1999-05-31) castable as xs:gYear
    // XPath: xs:date(\"1999-05-31\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas424() {
    // Try xs:date(1999-05-31Z) castable as xs:gYear
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas425() {
    // Try xs:date(1999-05-31) castable as xs:gMonthDay
    // XPath: xs:date(\"1999-05-31\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas427() {
    // Try xs:date(1999-05-31Z) castable as xs:gMonthDay
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas428() {
    // Try xs:date(1999-05-31Z) castable as xs:gDay
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas430() {
    // Try xs:date(1999-05-31Z) castable as xs:gMonth
    // XPath: xs:date(\"1999-05-31Z\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas431() {
    // Try xs:date(1999-05-31) castable as xs:boolean
    // XPath: xs:date(\"1999-05-31\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas432() {
    // Try xs:date(1999-05-31) castable as xs:base64Binary
    // XPath: xs:date(\"1999-05-31\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas433() {
    // Try xs:date(1999-05-31) castable as xs:hexBinary
    // XPath: xs:date(\"1999-05-31\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas434() {
    // Try xs:date(1999-05-31) castable as xs:anyURI
    // XPath: xs:date(\"1999-05-31\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas435() {
    // Try xs:gYearMonth(1999-05) castable as xs:untypedAtomic
    // XPath: xs:gYearMonth(\"1999-05\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas436() {
    // Try xs:gYearMonth(1999-05) castable as xs:string
    // XPath: xs:gYearMonth(\"1999-05\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas437() {
    // Try xs:gYearMonth(1999-05) castable as xs:float
    // XPath: xs:gYearMonth(\"1999-05\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas438() {
    // Try xs:gYearMonth(1999-05) castable as xs:double
    // XPath: xs:gYearMonth(\"1999-05\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas439() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:double
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas440() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:decimal
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas441() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:integer
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas442() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:duration
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas443() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:yearMonthDuration
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas444() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:dayTimeDuration
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas445() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:dateTime
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas446() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:time
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas447() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:date
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas448() {
    // Try xs:gYearMonth(1999-05) castable as xs:gYearMonth
    // XPath: xs:gYearMonth(\"1999-05\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas449() {
    // Try xs:gYearMonth(-0012-12-05:00) castable as xs:gYearMonth
    // XPath: xs:gYearMonth(\"-0012-12-05:00\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas450() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:gYearMonth
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas451() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:gYear
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas452() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:gMonthDay
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas453() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:gDay
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas454() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:gMonth
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas455() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:boolean
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas456() {
    // Try xs:gYearMonth(1999-05Z) castable as xs:base64Binary
    // XPath: xs:gYearMonth(\"1999-05Z\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
