//! Test chunk 5 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas392() {
    // Try casting xs:time(13:20:00-05:00) to xs:time
    // XPath: xs:time(\"13:20:00-05:00\") cast as xs:time
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas393() {
    // Try casting xs:time(13:20:02.123) to xs:time
    // XPath: xs:time(\"13:20:02.123\") cast as xs:time
    // Expected: 13:20:02.123
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas394() {
    // Try casting xs:time(13:20:00Z) to xs:time
    // XPath: xs:time(\"13:20:00Z\") cast as xs:time
    // Expected: 13:20:00Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas405() {
    // Try casting xs:date(1999-05-31) to xs:untypedAtomic
    // XPath: xs:date(\"1999-05-31\") cast as xs:untypedAtomic
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas407() {
    // Try casting xs:date(1999-05-31) to xs:string
    // XPath: xs:date(\"1999-05-31\") cast as xs:string
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas416() {
    // Try casting xs:date(1999-05-31) to xs:dateTime
    // XPath: xs:date(\"1999-05-31\") cast as xs:dateTime
    // Expected: 1999-05-31T00:00:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas418() {
    // Try casting xs:date(1999-05-31) to xs:date
    // XPath: xs:date(\"1999-05-31\") cast as xs:date
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas420() {
    // Try casting xs:date(1999-05-31Z) to xs:date
    // XPath: xs:date(\"1999-05-31Z\") cast as xs:date
    // Expected: 1999-05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas421() {
    // Try casting xs:date(1999-05-31) to xs:gYearMonth
    // XPath: xs:date(\"1999-05-31\") cast as xs:gYearMonth
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas422() {
    // Try casting xs:date(1999-05-31) to xs:gYear
    // XPath: xs:date(\"1999-05-31\") cast as xs:gYear
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas424() {
    // Try casting xs:date(1999-05-31Z) to xs:gYear
    // XPath: xs:date(\"1999-05-31Z\") cast as xs:gYear
    // Expected: 1999Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas425() {
    // Try casting xs:date(1999-05-31) to xs:gMonthDay
    // XPath: xs:date(\"1999-05-31\") cast as xs:gMonthDay
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas427() {
    // Try casting xs:date(1999-05-31Z) to xs:gMonthDay
    // XPath: xs:date(\"1999-05-31Z\") cast as xs:gMonthDay
    // Expected: --05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas428() {
    // Try casting xs:date(1999-05-31Z) to xs:gDay
    // XPath: xs:date(\"1999-05-31Z\") cast as xs:gDay
    // Expected: ---31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas430() {
    // Try casting xs:date(1999-05-31Z) to xs:gMonth
    // XPath: xs:date(\"1999-05-31Z\") cast as xs:gMonth
    // Expected: --05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas435() {
    // Try casting xs:gYearMonth(1999-05) to xs:untypedAtomic
    // XPath: xs:gYearMonth(\"1999-05\") cast as xs:untypedAtomic
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas436() {
    // Try casting xs:gYearMonth(1999-05) to xs:string
    // XPath: xs:gYearMonth(\"1999-05\") cast as xs:string
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas448() {
    // Try casting xs:gYearMonth(1999-05) to xs:gYearMonth
    // XPath: xs:gYearMonth(\"1999-05\") cast as xs:gYearMonth
    // Expected: 1999-05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas449() {
    // Try casting xs:gYearMonth(-0012-12-05:00) to xs:gYearMonth
    // XPath: xs:gYearMonth(\"-0012-12-05:00\") cast as xs:gYearMonth
    // Expected: -0012-12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas450() {
    // Try casting xs:gYearMonth(1999-05Z) to xs:gYearMonth
    // XPath: xs:gYearMonth(\"1999-05Z\") cast as xs:gYearMonth
    // Expected: 1999-05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas459() {
    // Try casting xs:gYear(1999) to xs:untypedAtomic
    // XPath: xs:gYear(\"1999\") cast as xs:untypedAtomic
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas460() {
    // Try casting xs:gYear(1999) to xs:string
    // XPath: xs:gYear(\"1999\") cast as xs:string
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas471() {
    // Try casting xs:gYear(1999) to xs:gYear
    // XPath: xs:gYear(\"1999\") cast as xs:gYear
    // Expected: 1999
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas472() {
    // Try casting xs:gYear(-0012-05:00) to xs:gYear
    // XPath: xs:gYear(\"-0012-05:00\") cast as xs:gYear
    // Expected: -0012-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas473() {
    // Try casting xs:gYear(1999Z) to xs:gYear
    // XPath: xs:gYear(\"1999Z\") cast as xs:gYear
    // Expected: 1999Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas481() {
    // Try casting xs:gMonthDay(--05-31) to xs:untypedAtomic
    // XPath: xs:gMonthDay(\"--05-31\") cast as xs:untypedAtomic
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas482() {
    // Try casting xs:gMonthDay(--05-31) to xs:string
    // XPath: xs:gMonthDay(\"--05-31\") cast as xs:string
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas495() {
    // Try casting xs:gMonthDay(--05-31) to xs:gMonthDay
    // XPath: xs:gMonthDay(\"--05-31\") cast as xs:gMonthDay
    // Expected: --05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas496() {
    // Try casting xs:gMonthDay(--12-03-05:00) to xs:gMonthDay
    // XPath: xs:gMonthDay(\"--12-03-05:00\") cast as xs:gMonthDay
    // Expected: --12-03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas497() {
    // Try casting xs:gMonthDay(--05-31Z) to xs:gMonthDay
    // XPath: xs:gMonthDay(\"--05-31Z\") cast as xs:gMonthDay
    // Expected: --05-31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas504() {
    // Try casting xs:gDay(---31) to xs:untypedAtomic
    // XPath: xs:gDay(\"---31\") cast as xs:untypedAtomic
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas505() {
    // Try casting xs:gDay(---31) to xs:string
    // XPath: xs:gDay(\"---31\") cast as xs:string
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas519() {
    // Try casting xs:gDay(---31) to xs:gDay
    // XPath: xs:gDay(\"---31\") cast as xs:gDay
    // Expected: ---31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas520() {
    // Try casting xs:gDay(---03-05:00) to xs:gDay
    // XPath: xs:gDay(\"---03-05:00\") cast as xs:gDay
    // Expected: ---03-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas521() {
    // Try casting xs:gDay(---31Z) to xs:gDay
    // XPath: xs:gDay(\"---31Z\") cast as xs:gDay
    // Expected: ---31Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas527() {
    // Try casting xs:gMonth(--05) to xs:untypedAtomic
    // XPath: xs:gMonth(\"--05\") cast as xs:untypedAtomic
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas528() {
    // Try casting xs:gMonth(--05) to xs:string
    // XPath: xs:gMonth(\"--05\") cast as xs:string
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas543() {
    // Try casting xs:gMonth(--05) to xs:gMonth
    // XPath: xs:gMonth(\"--05\") cast as xs:gMonth
    // Expected: --05
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas544() {
    // Try casting xs:gMonth(--12-05:00) to xs:gMonth
    // XPath: xs:gMonth(\"--12-05:00\") cast as xs:gMonth
    // Expected: --12-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas545() {
    // Try casting xs:gMonth(--05Z) to xs:gMonth
    // XPath: xs:gMonth(\"--05Z\") cast as xs:gMonth
    // Expected: --05Z
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas550() {
    // Try casting xs:boolean(true) to xs:untypedAtomic
    // XPath: xs:boolean(\"true\") cast as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas551() {
    // Try casting xs:boolean(true) to xs:string
    // XPath: xs:boolean(\"true\") cast as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas552() {
    // Try casting xs:boolean(true) to xs:float
    // XPath: xs:boolean(\"true\") cast as xs:float
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas553() {
    // Try casting xs:boolean(false) to xs:float
    // XPath: xs:boolean(\"false\") cast as xs:float
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas554() {
    // Try casting xs:boolean(true) to xs:double
    // XPath: xs:boolean(\"true\") cast as xs:double
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas555() {
    // Try casting xs:boolean(false) to xs:double
    // XPath: xs:boolean(\"false\") cast as xs:double
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas556() {
    // Try casting xs:boolean(true) to xs:decimal
    // XPath: xs:boolean(\"true\") cast as xs:decimal
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas557() {
    // Try casting xs:boolean(false) to xs:decimal
    // XPath: xs:boolean(\"false\") cast as xs:decimal
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas558() {
    // Try casting xs:boolean(true) to xs:integer
    // XPath: xs:boolean(\"true\") cast as xs:integer
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas559() {
    // Try casting xs:boolean(false) to xs:integer
    // XPath: xs:boolean(\"false\") cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
