//! Test chunk 0 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn casthc1() {
    //  Simple test for evaluation of a simple expression casted as xs:integer
    // XPath: (5 + 5) cast as xs:integer
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc2() {
    //  Simple test for evaluation of a simple expression casted as xs:float
    // XPath: (5 + 5) cast as xs:float
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc3() {
    //  Simple test for evaluation of a simple expression casted as xs:boolean
    // XPath: (5 + 5) cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc4() {
    //  Simple test for evaluation of a simple expression casted as xs:double
    // XPath: (5 + 5) cast as xs:double
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc5() {
    //  Simple test for evaluation of a simple expression casted as xs:decimal
    // XPath: (5 + 5) cast as xs:decimal
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc6() {
    //  Simple test for evaluation of a simple expression casted as xs:string
    // XPath: (5 + 5) cast as xs:string
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc7() {
    //  Simple test for evaluation a string casted as xs:string
    // XPath: \"A String\" cast as xs:string
    // Expected: A String
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc8() {
    //  Simple test for casting a float as a float
    // XPath: xs:float(2.1E3) cast as xs:float
    // Expected: 2100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc9() {
    //  Simple test for casting a float as a string
    // XPath: xs:float(2.1E3) cast as xs:string
    // Expected: 2100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc10() {
    //  Simple test for casting a float as a double
    // XPath: xs:float(2.1E3) cast as xs:double
    // Expected: 2100
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc11() {
    //  Simple test for casting a float as a boolean
    // XPath: xs:float(2.1E3) cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc12() {
    //  Simple test for casting a double as a double
    // XPath: xs:double(1267.43233E12) cast as xs:double
    // Expected: 1.26743233E15
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc13() {
    //  Simple test for casting a double as a string
    // XPath: xs:double(1267.43233E12) cast as xs:string
    // Expected: 1.26743233E15
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc14() {
    //  Simple test for casting a double as a float
    // XPath: (xs:double(1267.43233E12) cast as xs:float) eq xs:float(1267
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc15() {
    //  Simple test for casting a double as a boolean
    // XPath: xs:double(1267.43233E12) cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc16() {
    //  Simple test for casting a decimal as a string
    // XPath: xs:decimal(12678967.543233) cast as xs:string
    // Expected: 12678967.543233
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc17() {
    //  Simple test for casting a decimal as a float
    // XPath: xs:decimal(12678967.543233) cast as xs:float
    // Expected: 1.2678968E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc18() {
    //  Simple test for casting a decimal as a double
    // XPath: xs:decimal(12678967.543233) cast as xs:double
    // Expected: 1.2678967543233E7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc19() {
    //  Simple test for casting a decimal as a decimal
    // XPath: xs:decimal(12678967.543233) cast as xs:decimal
    // Expected: 12678967.543233
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc20() {
    //  Simple test for casting a decimal as a integer
    // XPath: xs:decimal(12678967.543233) cast as xs:integer
    // Expected: 12678967
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc21() {
    //  Simple test for casting a decimal as a boolean
    // XPath: xs:decimal(12678967.543233) cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc22() {
    //  Simple test for casting an integer as a string
    // XPath: xs:integer(12678967543233) cast as xs:string
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc23() {
    //  Simple test for casting an integer as a float
    // XPath: (xs:integer(12678967543233) cast as xs:float) eq xs:float(12
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc24() {
    //  Simple test for casting an integer as a double
    // XPath: xs:integer(12678967543233) cast as xs:double
    // Expected: 1.2678967543233E13
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc25() {
    //  Simple test for casting an integer as a decimal
    // XPath: xs:integer(12678967543233) cast as xs:decimal
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc26() {
    //  Simple test for casting an integer as an integer
    // XPath: xs:integer(12678967543233) cast as xs:integer
    // Expected: 12678967543233
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc27() {
    //  Simple test for casting an integer as a boolean
    // XPath: xs:integer(12678967543233) cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc28() {
    //  Simple test for casting an dateTime as a string
    // XPath: xs:dateTime(\"1999-05-31T13:20:00-05:00\") cast as xs:string
    // Expected: 1999-05-31T13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc29() {
    //  Simple test for casting an dateTime as a dateTime
    // XPath: xs:dateTime(\"1999-05-31T13:20:00-05:00\") cast as xs:dateTi
    // Expected: 1999-05-31T13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc30() {
    //  Simple test for casting an dateTime as a time
    // XPath: xs:dateTime(\"1999-05-31T13:20:00-05:00\") cast as xs:time
    // Expected: 13:20:00-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc31() {
    //  Simple test for casting an dateTime as a date
    // XPath: xs:dateTime(\"1999-05-31T13:20:00-05:00\") cast as xs:date
    // Expected: 1999-05-31-05:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc32() {
    //  Simple test for casting an date as a string
    // XPath: xs:date(\"1999-05-31\") cast as xs:string
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc33() {
    //  Simple test for casting an date as a dateTime
    // XPath: xs:date(\"1999-05-31\") cast as xs:dateTime
    // Expected: 1999-05-31T00:00:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc34() {
    //  Simple test for casting an date as a date
    // XPath: xs:date(\"1999-05-31\") cast as xs:date
    // Expected: 1999-05-31
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc35() {
    //  Simple test for casting an xs:time as a string
    // XPath: xs:time(\"12:00:00\") cast as xs:string
    // Expected: 12:00:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc36() {
    //  Simple test for casting an xs:time as an xs:time
    // XPath: xs:time(\"12:00:00\") cast as xs:time
    // Expected: 12:00:00
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc37() {
    //  Simple test for casting a boolean as string
    // XPath: fn:true() cast as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc38() {
    //  Simple test for casting a boolean as float
    // XPath: fn:true() cast as xs:float
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc39() {
    //  Simple test for casting a boolean as double
    // XPath: fn:true() cast as xs:double
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc40() {
    //  Simple test for casting a boolean as decimal
    // XPath: fn:true() cast as xs:decimal
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc41() {
    //  Simple test for casting a boolean as integer
    // XPath: fn:true() cast as xs:integer
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn casthc42() {
    //  Simple test for casting a boolean as boolean
    // XPath: fn:true() cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas001() {
    // Try casting xs:untypedAtomic(INF) to xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"INF\") cast as xs:untypedAtomic
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas002() {
    // Try casting xs:untypedAtomic(0.0E0) to xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"0.0E0\") cast as xs:untypedAtomic
    // Expected: 0.0E0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas003() {
    // Try casting xs:untypedAtomic(true) to xs:untypedAtomic
    // XPath: xs:untypedAtomic(\"true\") cast as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas004() {
    // Try casting xs:untypedAtomic(INF) to xs:string
    // XPath: xs:untypedAtomic(\"INF\") cast as xs:string
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas005() {
    // Try casting xs:untypedAtomic(0.0E0) to xs:string
    // XPath: xs:untypedAtomic(\"0.0E0\") cast as xs:string
    // Expected: 0.0E0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas006() {
    // Try casting xs:untypedAtomic(true) to xs:string
    // XPath: xs:untypedAtomic(\"true\") cast as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas007() {
    // Try casting xs:untypedAtomic(-0.0E0) to xs:float
    // XPath: xs:untypedAtomic(\"-0.0E0\") cast as xs:float
    // Expected: -0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas008() {
    // Try casting xs:untypedAtomic(NaN) to xs:float
    // XPath: xs:untypedAtomic(\"NaN\") cast as xs:float
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
