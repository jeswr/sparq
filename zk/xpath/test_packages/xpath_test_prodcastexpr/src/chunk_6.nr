//! Test chunk 6 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas571() {
    // Try casting xs:boolean(true) to xs:boolean
    // XPath: xs:boolean(\"true\") cast as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas572() {
    // Try casting xs:boolean(false) to xs:boolean
    // XPath: xs:boolean(\"false\") cast as xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas576() {
    // Try casting xs:base64Binary(01001010) to xs:untypedAtomic
    // XPath: xs:base64Binary(\"01001010\") cast as xs:untypedAtomic
    // Expected: 01001010
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas577() {
    // Try casting xs:base64Binary(0FB7) to xs:untypedAtomic
    // XPath: xs:base64Binary(\"0FB7\") cast as xs:untypedAtomic
    // Expected: 0FB7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas578() {
    // Try casting xs:base64Binary(01001010) to xs:string
    // XPath: xs:base64Binary(\"01001010\") cast as xs:string
    // Expected: 01001010
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas579() {
    // Try casting xs:base64Binary(0FB7) to xs:string
    // XPath: xs:base64Binary(\"0FB7\") cast as xs:string
    // Expected: 0FB7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas596() {
    // Try casting xs:base64Binary(01001010) to xs:base64Binary
    // XPath: xs:base64Binary(\"01001010\") cast as xs:base64Binary
    // Expected: 01001010
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas597() {
    // Try casting xs:base64Binary(0FB7) to xs:base64Binary
    // XPath: xs:base64Binary(\"0FB7\") cast as xs:base64Binary
    // Expected: 0FB7
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas598() {
    // Try casting xs:base64Binary(00000000) to xs:hexBinary
    // XPath: xs:base64Binary(\"00000000\") cast as xs:hexBinary
    // Expected: D34D34D34D34
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas599() {
    // Try casting xs:base64Binary(10010101) to xs:hexBinary
    // XPath: xs:base64Binary(\"10010101\") cast as xs:hexBinary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas600() {
    // Try casting xs:base64Binary(aA+zZ/09) to xs:hexBinary
    // XPath: xs:base64Binary(\"aA+zZ/09\") cast as xs:hexBinary
    // Expected: 680FB367FD3D
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas601() {
    // Try casting xs:base64Binary(0FB7) to xs:hexBinary
    // XPath: xs:base64Binary(\"0FB7\") cast as xs:hexBinary
    // Expected: D0507B
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas603() {
    // Try casting xs:hexBinary(D74D35D35D35) to xs:untypedAtomic
    // XPath: xs:hexBinary(\"D74D35D35D35\") cast as xs:untypedAtomic
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas604() {
    // Try casting xs:hexBinary(D74D35D35D35) to xs:string
    // XPath: xs:hexBinary(\"D74D35D35D35\") cast as xs:string
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas621() {
    // Try casting xs:hexBinary(D74D35D35D35) to xs:base64Binary
    // XPath: xs:hexBinary(\"D74D35D35D35\") cast as xs:base64Binary
    // Expected: 10010101
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas622() {
    // Try casting xs:hexBinary(0fb7) to xs:base64Binary
    // XPath: xs:hexBinary(\"0fb7\") cast as xs:base64Binary
    // Expected: D7c=
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas623() {
    // Try casting xs:hexBinary(D74D35D35D35) to xs:hexBinary
    // XPath: xs:hexBinary(\"D74D35D35D35\") cast as xs:hexBinary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas624() {
    // Try casting xs:hexBinary(d74d35d35d35) to xs:hexBinary
    // XPath: xs:hexBinary(\"d74d35d35d35\") cast as xs:hexBinary
    // Expected: D74D35D35D35
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas626() {
    // Try casting xs:anyURI(http://www.ietf.org/rfc/rfc2396.txt) to xs:untypedAtomic
    // XPath: xs:anyURI(\"http://www.ietf.org/rfc/rfc2396.txt\") cast as x
    // Expected: http://www.ietf.org/rfc/rfc2396.txt
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas627() {
    // Try casting xs:anyURI(http://www.ietf.org/rfc/rfc2396.txt) to xs:string
    // XPath: xs:anyURI(\"http://www.ietf.org/rfc/rfc2396.txt\") cast as x
    // Expected: http://www.ietf.org/rfc/rfc2396.txt
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas646() {
    // Try casting xs:anyURI(http://www.ietf.org/rfc/rfc2396.txt) to xs:anyURI
    // XPath: xs:anyURI(\"http://www.ietf.org/rfc/rfc2396.txt\") cast as x
    // Expected: http://www.ietf.org/rfc/rfc2396.txt
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas648() {
    //  Evaluates casting a string into a nonPositiveInteger.
    // XPath: xs:string(-20) cast as xs:nonPositiveInteger
    // Expected: -20
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas649() {
    //  Evaluates casting a string into an xs:long.
    // XPath: xs:string(20) cast as xs:long
    // Expected: 20
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas650() {
    //  Evaluates casting a string into an xs:nonNegativeInteger
    // XPath: xs:string(200) cast as xs:nonNegativeInteger
    // Expected: 200
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas651() {
    //  Evaluates casting a string into an xs:negativeInteger
    // XPath: xs:string(-201) cast as xs:negativeInteger
    // Expected: -201
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas652() {
    //  Evaluates casting a string into an xs:int
    // XPath: xs:string(20) cast as xs:int
    // Expected: 20
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas653() {
    //  Evaluates casting a string into an xs:unsignedLong
    // XPath: xs:string(20) cast as xs:unsignedLong
    // Expected: 20
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas654() {
    //  Evaluates casting a string into an xs:positiveInteger
    // XPath: xs:string(300) cast as xs:positiveInteger
    // Expected: 300
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas655() {
    //  Evaluates casting a string into an xs:short
    // XPath: xs:string(30) cast as xs:short
    // Expected: 30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas656() {
    //  Evaluates casting a string into an xs:unsignedInt
    // XPath: xs:string(300) cast as xs:unsignedInt
    // Expected: 300
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas657() {
    //  Evaluates casting a string into an xs:byte
    // XPath: xs:string(120) cast as xs:byte
    // Expected: 120
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas658() {
    //  Evaluates casting a string into an xs:unsignedShort
    // XPath: xs:string(12) cast as xs:unsignedShort
    // Expected: 12
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas659() {
    //  Evaluates casting a string into an xs:unsignedByte
    // XPath: xs:string(12) cast as xs:unsignedByte
    // Expected: 12
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas660() {
    //  Evaluates casting a long into an xs:short
    // XPath: let $var := xs:long(120) cast as xs:short return $var instan
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas661() {
    //  Evaluates casting a short into an xs:long
    // XPath: let $var := xs:short(120) cast as xs:long return $var instan
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas662() {
    //  Evaluates casting a xs:nonPositiveInteger into an xs:negativeInteger
    // XPath: let $var := xs:nonPositiveInteger(-120) cast as xs:negativeI
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas663() {
    //  Evaluates casting a xs:nonNegativeInteger into an xs:positiveInteger
    // XPath: let $var := xs:nonNegativeInteger(120) cast as xs:positiveIn
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas664() {
    //  Evaluates casting a xs:short into an xs:unsignedShort
    // XPath: let $var := xs:short(120) cast as xs:unsignedShort return $v
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas665() {
    //  Evaluates casting a xs:int into an xs:long
    // XPath: let $var := xs:int(120) cast as xs:long return $var instance
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas666() {
    //  Evaluates casting a xs:long into an xs:int
    // XPath: let $var := xs:long(120) cast as xs:int return $var instance
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas667() {
    //  Evaluates casting a xs:unsignedShort into an xs:unsignedInt
    // XPath: let $var := xs:unsignedShort(120) cast as xs:unsignedInt ret
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas668() {
    //  Evaluates casting a xs:unsignedInt into an xs:unsignedShort
    // XPath: let $var := xs:unsignedInt(120) cast as xs:unsignedShort ret
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas669() {
    //  Evaluates casting a xs:byte into an xs:unsignedByte
    // XPath: let $var := xs:byte(120) cast as xs:unsignedByte return $var
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas671() {
    //  Evaluates casting a string as an xs:ENTITY
    // XPath: let $var := \"abc\" cast as xs:ENTITY return $var instance o
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas673b() {
    //  Try to cast a variable to xs:QName
    // XPath: let $var := \"ABC\" return $var cast as xs:QName
    // Expected: ABC
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas679() {
    // Negative timezone with zero hour part
    // XPath: xs:dateTime('2017-07-19T12:00:00-00:30')
    // Expected: 2017-07-19T12:00:00-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas680() {
    // Negative timezone with zero hour part
    // XPath: xs:time('12:00:00-00:30')
    // Expected: 12:00:00-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas681() {
    // Negative timezone with zero hour part
    // XPath: xs:date('2017-07-19-00:30')
    // Expected: 2017-07-19-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas682() {
    // Negative timezone with zero hour part
    // XPath: xs:gYear('2017-00:30')
    // Expected: 2017-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas683() {
    // Negative timezone with zero hour part
    // XPath: xs:gYearMonth('2017-07-00:30')
    // Expected: 2017-07-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
