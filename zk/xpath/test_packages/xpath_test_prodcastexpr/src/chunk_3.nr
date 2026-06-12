//! Test chunk 3 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas163() {
    // Try casting xs:string(true) to xs:anyURI
    // XPath: xs:string(\"true\") cast as xs:anyURI
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas164() {
    // Try casting xs:string(http://www.ietf.org/rfc/rfc2396.txt) to xs:anyURI
    // XPath: xs:string(\"http://www.ietf.org/rfc/rfc2396.txt\") cast as x
    // Expected: http://www.ietf.org/rfc/rfc2396.txt
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas165() {
    // Try casting xs:float(1e5) to xs:untypedAtomic
    // XPath: xs:float(\"1e5\") cast as xs:untypedAtomic
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas166() {
    // Try casting xs:float(-INF) to xs:untypedAtomic
    // XPath: xs:float(\"-INF\") cast as xs:untypedAtomic
    // Expected: -INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas167() {
    // Try casting xs:float(-0.0E0) to xs:string
    // XPath: xs:float(\"-0.0E0\") cast as xs:string
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas168() {
    // Try casting xs:float(NaN) to xs:string
    // XPath: xs:float(\"NaN\") cast as xs:string
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas170() {
    // Try casting xs:float(-1.75e-3) to xs:float
    // XPath: xs:float(\"-1.75e-3\") cast as xs:float
    // Expected: -0.00175
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas171() {
    // Try casting xs:float(INF) to xs:float
    // XPath: xs:float(\"INF\") cast as xs:float
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas172() {
    // Try casting xs:float(-0.0E0) to xs:double
    // XPath: xs:float(\"-0.0E0\") cast as xs:double
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas173() {
    // Try casting xs:float(-INF) to xs:double
    // XPath: xs:float(\"-INF\") cast as xs:double
    // Expected: -INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas174() {
    // Try casting xs:float(-0.0E0) to xs:decimal
    // XPath: xs:float(\"-0.0E0\") cast as xs:decimal
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas175() {
    // Try casting xs:float(5.4321E-100) to xs:decimal
    // XPath: xs:float(\"5.4321E-100\") cast as xs:decimal
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas176() {
    // Try casting xs:float(1e5) to xs:decimal
    // XPath: xs:float(\"1e5\") cast as xs:decimal
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas177() {
    // Try casting xs:float(-1.75e-3) to xs:decimal
    // XPath: round-half-to-even(xs:float(\"-1.75e-3\") cast as xs:decimal
    // Expected: -0.00175
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas178() {
    // Try casting xs:float(-0.0E0) to xs:integer
    // XPath: xs:float(\"-0.0E0\") cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas179() {
    // Try casting xs:float(1e5) to xs:integer
    // XPath: xs:float(\"1e5\") cast as xs:integer
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas192() {
    // Try casting xs:float(-0.0E0) to xs:boolean
    // XPath: xs:float(\"-0.0E0\") cast as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas193() {
    // Try casting xs:float(1e5) to xs:boolean
    // XPath: xs:float(\"1e5\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas194() {
    // Try casting xs:float(-INF) to xs:boolean
    // XPath: xs:float(\"-INF\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas195() {
    // Try casting xs:float(NaN) to xs:boolean
    // XPath: xs:float(\"NaN\") cast as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas199() {
    // Try casting xs:double(1e5) to xs:untypedAtomic
    // XPath: xs:double(\"1e5\") cast as xs:untypedAtomic
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas200() {
    // Try casting xs:double(INF) to xs:untypedAtomic
    // XPath: xs:double(\"INF\") cast as xs:untypedAtomic
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas201() {
    // Try casting xs:double(1e8) to xs:string
    // XPath: xs:double(\"1e8\") cast as xs:string
    // Expected: 1.0E8
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas202() {
    // Try casting xs:double(INF) to xs:string
    // XPath: xs:double(\"INF\") cast as xs:string
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas203() {
    // Try casting xs:double(-0.0E0) to xs:float
    // XPath: xs:double(\"-0.0E0\") cast as xs:float
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas205() {
    // Try casting xs:double(1e5) to xs:float
    // XPath: xs:double(\"1e5\") cast as xs:float
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas206() {
    // Try casting xs:double(-1.75e-3) to xs:float
    // XPath: xs:double(\"-1.75e-3\") cast as xs:float
    // Expected: -0.00175
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas207() {
    // Try casting xs:double(NaN) to xs:float
    // XPath: xs:double(\"NaN\") cast as xs:float
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas208() {
    // Try casting xs:double(1e5) to xs:double
    // XPath: xs:double(\"1e5\") cast as xs:double
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas209() {
    // Try casting xs:double(INF) to xs:double
    // XPath: xs:double(\"INF\") cast as xs:double
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas210() {
    // Try casting xs:double(-0.0E0) to xs:decimal
    // XPath: xs:double(\"-0.0E0\") cast as xs:decimal
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas211() {
    // Try casting xs:double(5.4321E-1001) to xs:decimal
    // XPath: xs:double(\"5.4321E-1001\") cast as xs:decimal
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas212() {
    // Try casting xs:double(-1.75e-3) to xs:decimal
    // XPath: round-half-to-even(xs:double(\"-1.75e-3\") cast as xs:decima
    // Expected: -0.00175
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas214() {
    // Try casting xs:double(-0.0E0) to xs:integer
    // XPath: xs:double(\"-0.0E0\") cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas215() {
    // Try casting xs:double(1e5) to xs:integer
    // XPath: xs:double(\"1e5\") cast as xs:integer
    // Expected: 100000
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas216() {
    // Try casting xs:double(-1.75e-3) to xs:integer
    // XPath: xs:double(\"-1.75e-3\") cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas230() {
    // Try casting xs:double(1e5) to xs:boolean
    // XPath: xs:double(\"1e5\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas234() {
    // Try casting xs:decimal(-1.1234) to xs:untypedAtomic
    // XPath: xs:decimal(\"-1.1234\") cast as xs:untypedAtomic
    // Expected: -1.1234
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas235() {
    // Try casting xs:decimal(-1.1234) to xs:string
    // XPath: xs:decimal(\"-1.1234\") cast as xs:string
    // Expected: -1.1234
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas236() {
    // Try casting xs:decimal(-1.1234) to xs:float
    // XPath: xs:decimal(\"-1.1234\") cast as xs:float
    // Expected: -1.1234
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas237() {
    // Try casting xs:decimal(-1.1234) to xs:double
    // XPath: xs:decimal(\"-1.1234\") cast as xs:double
    // Expected: -1.1234
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas238() {
    // Try casting xs:decimal(-1.1234) to xs:decimal
    // XPath: xs:decimal(\"-1.1234\") cast as xs:decimal
    // Expected: -1.1234
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas239() {
    // Try casting xs:decimal(-1.1234) to xs:integer
    // XPath: xs:decimal(\"-1.1234\") cast as xs:integer
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas240() {
    // Try casting xs:decimal(5.5432) to xs:integer
    // XPath: xs:decimal(\"5.5432\") cast as xs:integer
    // Expected: 5
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas252() {
    // Try casting xs:decimal(-1.1234) to xs:boolean
    // XPath: xs:decimal(\"-1.1234\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas256() {
    // Try casting xs:integer(1) to xs:untypedAtomic
    // XPath: xs:integer(\"1\") cast as xs:untypedAtomic
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas257() {
    // Try casting xs:integer(-100) to xs:string
    // XPath: xs:integer(\"-100\") cast as xs:string
    // Expected: -100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas258() {
    // Try casting xs:integer(-100) to xs:float
    // XPath: xs:integer(\"-100\") cast as xs:float
    // Expected: -100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas259() {
    // Try casting xs:integer(-100) to xs:double
    // XPath: xs:integer(\"-100\") cast as xs:double
    // Expected: -100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas260() {
    // Try casting xs:integer(-100) to xs:decimal
    // XPath: xs:integer(\"-100\") cast as xs:decimal
    // Expected: -100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
