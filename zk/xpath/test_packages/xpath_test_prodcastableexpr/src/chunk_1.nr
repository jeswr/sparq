//! Test chunk 1 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas051() {
    // Try xs:untypedAtomic(-0012-12-05:00) castable as xs:gYearMonth
    // XPath: xs:untypedAtomic(\"-0012-12-05:00\") castable as xs:gYearMon
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas052() {
    // Try xs:untypedAtomic(1999-05Z) castable as xs:gYearMonth
    // XPath: xs:untypedAtomic(\"1999-05Z\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas053() {
    // Try xs:untypedAtomic(true) castable as xs:gYearMonth
    // XPath: xs:untypedAtomic(\"true\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas054() {
    // Try xs:untypedAtomic(1999) castable as xs:gYear
    // XPath: xs:untypedAtomic(\"1999\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas055() {
    // Try xs:untypedAtomic(-0012-05:00) castable as xs:gYear
    // XPath: xs:untypedAtomic(\"-0012-05:00\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas056() {
    // Try xs:untypedAtomic(1999Z) castable as xs:gYear
    // XPath: xs:untypedAtomic(\"1999Z\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas057() {
    // Try xs:untypedAtomic(true) castable as xs:gYear
    // XPath: xs:untypedAtomic(\"true\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas058() {
    // Try xs:untypedAtomic(--05-31) castable as xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas059() {
    // Try xs:untypedAtomic(--05-31+14:00) castable as xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31+14:00\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas060() {
    // Try xs:untypedAtomic(--05-31Z) castable as xs:gMonthDay
    // XPath: xs:untypedAtomic(\"--05-31Z\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas061() {
    // Try xs:untypedAtomic(true) castable as xs:gMonthDay
    // XPath: xs:untypedAtomic(\"true\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas062() {
    // Try xs:untypedAtomic(---31) castable as xs:gDay
    // XPath: xs:untypedAtomic(\"---31\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas063() {
    // Try xs:untypedAtomic(---03-05:00) castable as xs:gDay
    // XPath: xs:untypedAtomic(\"---03-05:00\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas064() {
    // Try xs:untypedAtomic(---31Z) castable as xs:gDay
    // XPath: xs:untypedAtomic(\"---31Z\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas065() {
    // Try xs:untypedAtomic(true) castable as xs:gDay
    // XPath: xs:untypedAtomic(\"true\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas066() {
    // Try xs:untypedAtomic(--05) castable as xs:gMonth
    // XPath: xs:untypedAtomic(\"--05\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas067() {
    // Try xs:untypedAtomic(--12-05:00) castable as xs:gMonth
    // XPath: xs:untypedAtomic(\"--12-05:00\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas068() {
    // Try xs:untypedAtomic(--05Z) castable as xs:gMonth
    // XPath: xs:untypedAtomic(\"--05Z\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas069() {
    // Try xs:untypedAtomic(true) castable as xs:gMonth
    // XPath: xs:untypedAtomic(\"true\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas070() {
    // Try xs:untypedAtomic(0.0) castable as xs:boolean
    // XPath: xs:untypedAtomic(\"0.0\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas071() {
    // Try xs:untypedAtomic(0) castable as xs:boolean
    // XPath: xs:untypedAtomic(\"0\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas072() {
    // Try xs:untypedAtomic(true) castable as xs:boolean
    // XPath: xs:untypedAtomic(\"true\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas073() {
    // Try xs:untypedAtomic(true) castable as xs:base64Binary
    // XPath: xs:untypedAtomic(\"true\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas074() {
    // Try xs:untypedAtomic(00000000) castable as xs:base64Binary
    // XPath: xs:untypedAtomic(\"00000000\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas075() {
    // Try xs:untypedAtomic(D74D35D35D35) castable as xs:base64Binary
    // XPath: xs:untypedAtomic(\"D74D35D35D35\") castable as xs:base64Bina
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas076() {
    // Try xs:untypedAtomic(true) castable as xs:hexBinary
    // XPath: xs:untypedAtomic(\"true\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas077() {
    // Try xs:untypedAtomic(010010101) castable as xs:hexBinary
    // XPath: xs:untypedAtomic(\"010010101\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas078() {
    // Try xs:untypedAtomic(0fb7) castable as xs:hexBinary
    // XPath: xs:untypedAtomic(\"0fb7\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas079() {
    // Try xs:untypedAtomic(D74D35D35D35) castable as xs:hexBinary
    // XPath: xs:untypedAtomic(\"D74D35D35D35\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas080() {
    // Try xs:untypedAtomic(-0012-05:00) castable as xs:anyURI
    // XPath: xs:untypedAtomic(\"http://www.example.com/~b%C3%A9b%C3%A9\")
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas081() {
    // Try xs:untypedAtomic(true) castable as xs:anyURI
    // XPath: xs:untypedAtomic(\"true\") castable as xs:anyURI
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas082() {
    // Try xs:untypedAtomic(http://www.ietf.org/rfc/rfc2396.txt) castable as xs:anyURI
    // XPath: xs:untypedAtomic(\"http://www.ietf.org/rfc/rfc2396.txt\") ca
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas083() {
    // Try xs:string(INF) castable as xs:untypedAtomic
    // XPath: xs:string(\"INF\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas084() {
    // Try xs:string(0.0E0) castable as xs:untypedAtomic
    // XPath: xs:string(\"0.0E0\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas085() {
    // Try xs:string(true) castable as xs:untypedAtomic
    // XPath: xs:string(\"true\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas086() {
    // Try xs:string(INF) castable as xs:string
    // XPath: xs:string(\"INF\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas087() {
    // Try xs:string(0.0E0) castable as xs:string
    // XPath: xs:string(\"0.0E0\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas088() {
    // Try xs:string(true) castable as xs:string
    // XPath: xs:string(\"true\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas089() {
    // Try xs:string(-0.0E0) castable as xs:float
    // XPath: xs:string(\"-0.0E0\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas090() {
    // Try xs:string(NaN) castable as xs:float
    // XPath: xs:string(\"NaN\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas091() {
    // Try xs:string(1e-5) castable as xs:float
    // XPath: xs:string(\"1e-5\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas092() {
    // Try xs:string(-10000000) castable as xs:float
    // XPath: xs:string(\"-10000000\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas093() {
    // Try xs:string(true) castable as xs:float
    // XPath: xs:string(\"true\") castable as xs:float
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas094() {
    // Try xs:string(-0.0E0) castable as xs:double
    // XPath: xs:string(\"-0.0E0\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas095() {
    // Try xs:string(NaN) castable as xs:double
    // XPath: xs:string(\"NaN\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas096() {
    // Try xs:string(1e-5) castable as xs:double
    // XPath: xs:string(\"1e-5\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas097() {
    // Try xs:string(-10000000) castable as xs:double
    // XPath: xs:string(\"-10000000\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas098() {
    // Try xs:string(true) castable as xs:double
    // XPath: xs:string(\"true\") castable as xs:double
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas099() {
    // Try xs:string(-0.0E0) castable as xs:decimal
    // XPath: xs:string(\"-0.0E0\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas100() {
    // Try xs:string(NaN) castable as xs:decimal
    // XPath: xs:string(\"NaN\") castable as xs:decimal
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
