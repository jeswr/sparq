//! Test chunk 3 for prod:CastableExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastableexpr;

#[test]
fn castableas151() {
    // Try xs:string(true) castable as xs:gMonth
    // XPath: xs:string(\"true\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas152() {
    // Try xs:string(0.0) castable as xs:boolean
    // XPath: xs:string(\"0.0\") castable as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas153() {
    // Try xs:string(0) castable as xs:boolean
    // XPath: xs:string(\"0\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas154() {
    // Try xs:string(true) castable as xs:boolean
    // XPath: xs:string(\"true\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas155() {
    // Try xs:string(true) castable as xs:base64Binary
    // XPath: xs:string(\"true\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas156() {
    // Try xs:string(00000000) castable as xs:base64Binary
    // XPath: xs:string(\"00000000\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas157() {
    // Try xs:string(D74D35D35D35) castable as xs:base64Binary
    // XPath: xs:string(\"D74D35D35D35\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas158() {
    // Try xs:string(true) castable as xs:hexBinary
    // XPath: xs:string(\"true\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas159() {
    // Try xs:string(010010101) castable as xs:hexBinary
    // XPath: xs:string(\"010010101\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas160() {
    // Try xs:string(0fb7) castable as xs:hexBinary
    // XPath: xs:string(\"0fb7\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas161() {
    // Try xs:string(D74D35D35D35) castable as xs:hexBinary
    // XPath: xs:string(\"D74D35D35D35\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas162() {
    // Try xs:string(-0012-05:00) castable as xs:anyURI
    // XPath: xs:string(\"http://www.example.com/~b%C3%A9b%C3%A9\") castab
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas163() {
    // Try xs:string(true) castable as xs:anyURI
    // XPath: xs:string(\"true\") castable as xs:anyURI
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas164() {
    // Try xs:string(http://www.ietf.org/rfc/rfc2396.txt) castable as xs:anyURI
    // XPath: xs:string(\"http://www.ietf.org/rfc/rfc2396.txt\") castable
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas165() {
    // Try xs:float(1e5) castable as xs:untypedAtomic
    // XPath: xs:float(\"1e5\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas166() {
    // Try xs:float(-INF) castable as xs:untypedAtomic
    // XPath: xs:float(\"-INF\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas167() {
    // Try xs:float(-0.0E0) castable as xs:string
    // XPath: xs:float(\"-0.0E0\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas168() {
    // Try xs:float(NaN) castable as xs:string
    // XPath: xs:float(\"NaN\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas169() {
    // Try xs:float(5.4321E-100) castable as xs:float
    // XPath: xs:float(\"5.4321E-100\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas170() {
    // Try xs:float(-1.75e-3) castable as xs:float
    // XPath: xs:float(\"-1.75e-3\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas171() {
    // Try xs:float(INF) castable as xs:float
    // XPath: xs:float(\"INF\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas172() {
    // Try xs:float(-0.0E0) castable as xs:double
    // XPath: xs:float(\"-0.0E0\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas173() {
    // Try xs:float(-INF) castable as xs:double
    // XPath: xs:float(\"-INF\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas174() {
    // Try xs:float(-0.0E0) castable as xs:decimal
    // XPath: xs:float(\"-0.0E0\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas175() {
    // Try xs:float(5.4321E-100) castable as xs:decimal
    // XPath: xs:float(\"5.4321E-100\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas176() {
    // Try xs:float(1e5) castable as xs:decimal
    // XPath: xs:float(\"1e5\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas177() {
    // Try xs:float(-1.75e-3) castable as xs:decimal
    // XPath: xs:float(\"-1.75e-3\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas178() {
    // Try xs:float(-0.0E0) castable as xs:integer
    // XPath: xs:float(\"-0.0E0\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas179() {
    // Try xs:float(1e5) castable as xs:integer
    // XPath: xs:float(\"1e5\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas180() {
    // Try xs:float(INF) castable as xs:integer
    // XPath: xs:float(\"INF\") castable as xs:integer
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas181() {
    // Try xs:float(-1.75e-3) castable as xs:duration
    // XPath: xs:float(\"-1.75e-3\") castable as xs:duration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas182() {
    // Try xs:float(5.4321E-100) castable as xs:yearMonthDuration
    // XPath: xs:float(\"5.4321E-100\") castable as xs:yearMonthDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas183() {
    // Try xs:float(5.4321E-100) castable as xs:dayTimeDuration
    // XPath: xs:float(\"5.4321E-100\") castable as xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas184() {
    // Try xs:float(5.4321E-100) castable as xs:dateTime
    // XPath: xs:float(\"5.4321E-100\") castable as xs:dateTime
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas185() {
    // Try xs:float(5.4321E-100) castable as xs:time
    // XPath: xs:float(\"5.4321E-100\") castable as xs:time
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas186() {
    // Try xs:float(5.4321E-100) castable as xs:date
    // XPath: xs:float(\"5.4321E-100\") castable as xs:date
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas187() {
    // Try xs:float(5.4321E-100) castable as xs:gYearMonth
    // XPath: xs:float(\"5.4321E-100\") castable as xs:gYearMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas188() {
    // Try xs:float(5.4321E-100) castable as xs:gYear
    // XPath: xs:float(\"5.4321E-100\") castable as xs:gYear
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas189() {
    // Try xs:float(5.4321E-100) castable as xs:gMonthDay
    // XPath: xs:float(\"5.4321E-100\") castable as xs:gMonthDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas190() {
    // Try xs:float(5.4321E-100) castable as xs:gDay
    // XPath: xs:float(\"5.4321E-100\") castable as xs:gDay
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas191() {
    // Try xs:float(5.4321E-100) castable as xs:gMonth
    // XPath: xs:float(\"5.4321E-100\") castable as xs:gMonth
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas192() {
    // Try xs:float(-0.0E0) castable as xs:boolean
    // XPath: xs:float(\"-0.0E0\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas193() {
    // Try xs:float(1e5) castable as xs:boolean
    // XPath: xs:float(\"1e5\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas194() {
    // Try xs:float(-INF) castable as xs:boolean
    // XPath: xs:float(\"-INF\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas195() {
    // Try xs:float(NaN) castable as xs:boolean
    // XPath: xs:float(\"NaN\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas196() {
    // Try xs:float(5.4321E-100) castable as xs:base64Binary
    // XPath: xs:float(\"5.4321E-100\") castable as xs:base64Binary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas197() {
    // Try xs:float(5.4321E-100) castable as xs:hexBinary
    // XPath: xs:float(\"5.4321E-100\") castable as xs:hexBinary
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas198() {
    // Try xs:float(5.4321E-100) castable as xs:anyURI
    // XPath: xs:float(\"5.4321E-100\") castable as xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas199() {
    // Try xs:double(1e5) castable as xs:untypedAtomic
    // XPath: xs:double(\"1e5\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}

#[test]
fn castableas200() {
    // Try xs:double(INF) castable as xs:untypedAtomic
    // XPath: xs:double(\"INF\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastableExpr is implemented
    let _ = stub_prodcastableexpr();
}
