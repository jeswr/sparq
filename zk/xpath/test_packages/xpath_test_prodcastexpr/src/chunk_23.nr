//! Test chunk 23 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_1389() {
    //  'castable as' involving xs:anyURI as source type and xs:dayTimeDuration as targ
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1391() {
    //  'castable as' involving xs:anyURI as source type and xs:dateTime as target type
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1393() {
    //  'castable as' involving xs:anyURI as source type and xs:time as target type sho
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1395() {
    //  'castable as' involving xs:anyURI as source type and xs:date as target type sho
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1397() {
    //  'castable as' involving xs:anyURI as source type and xs:gYearMonth as target ty
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1399() {
    //  'castable as' involving xs:anyURI as source type and xs:gYear as target type sh
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1401() {
    //  'castable as' involving xs:anyURI as source type and xs:gMonthDay as target typ
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1403() {
    //  'castable as' involving xs:anyURI as source type and xs:gDay as target type sho
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1405() {
    //  'castable as' involving xs:anyURI as source type and xs:gMonth as target type s
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1407() {
    //  'castable as' involving xs:anyURI as source type and xs:boolean as target type
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1409() {
    //  'castable as' involving xs:anyURI as source type and xs:base64Binary as target
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1411() {
    //  'castable as' involving xs:anyURI as source type and xs:hexBinary as target typ
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1412() {
    //  Casting from xs:anyURI to xs:anyURI is allowed and should always succeed.
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1413() {
    //  'castable as' involving xs:anyURI as source type and xs:anyURI as target type s
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1415() {
    //  'castable as' involving xs:anyURI as source type and xs:QName as target type sh
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1419() {
    //  Verify with 'instance of' that the xs:QName constructor function produces value
    // XPath: subsequence((\"dummy\", 1.1, xs:QName(\"ncname\")), 3, 1) in
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1420() {
    //  A simple test exercising the whitespace facet for type xs:QName.
    // XPath: xs:QName(\" ncname \") eq xs:QName(\" ncname \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1423() {
    //  Casting from xs:QName to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:QName(\"ncname\") cast as xs:untypedAtomic ne xs:untypedA
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1424() {
    //  'castable as' involving xs:QName as source type and xs:untypedAtomic as target
    // XPath: xs:QName(\"ncname\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1425() {
    //  Casting from xs:QName to xs:string is allowed and should always succeed.
    // XPath: xs:QName(\"ncname\") cast as xs:string ne xs:string(\"an arb
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1426() {
    //  'castable as' involving xs:QName as source type and xs:string as target type sh
    // XPath: xs:QName(\"ncname\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1428() {
    //  'castable as' involving xs:QName as source type and xs:float as target type sho
    // XPath: not(xs:QName(\"ncname\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1430() {
    //  'castable as' involving xs:QName as source type and xs:double as target type sh
    // XPath: not(xs:QName(\"ncname\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1432() {
    //  'castable as' involving xs:QName as source type and xs:decimal as target type s
    // XPath: not(xs:QName(\"ncname\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1434() {
    //  'castable as' involving xs:QName as source type and xs:integer as target type s
    // XPath: not(xs:QName(\"ncname\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1436() {
    //  'castable as' involving xs:QName as source type and xs:duration as target type
    // XPath: not(xs:QName(\"ncname\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1438() {
    //  'castable as' involving xs:QName as source type and xs:yearMonthDuration as tar
    // XPath: not(xs:QName(\"ncname\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1440() {
    //  'castable as' involving xs:QName as source type and xs:dayTimeDuration as targe
    // XPath: not(xs:QName(\"ncname\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1442() {
    //  'castable as' involving xs:QName as source type and xs:dateTime as target type
    // XPath: not(xs:QName(\"ncname\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1444() {
    //  'castable as' involving xs:QName as source type and xs:time as target type shou
    // XPath: not(xs:QName(\"ncname\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1446() {
    //  'castable as' involving xs:QName as source type and xs:date as target type shou
    // XPath: not(xs:QName(\"ncname\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1448() {
    //  'castable as' involving xs:QName as source type and xs:gYearMonth as target typ
    // XPath: not(xs:QName(\"ncname\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1450() {
    //  'castable as' involving xs:QName as source type and xs:gYear as target type sho
    // XPath: not(xs:QName(\"ncname\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1452() {
    //  'castable as' involving xs:QName as source type and xs:gMonthDay as target type
    // XPath: not(xs:QName(\"ncname\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1454() {
    //  'castable as' involving xs:QName as source type and xs:gDay as target type shou
    // XPath: not(xs:QName(\"ncname\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1456() {
    //  'castable as' involving xs:QName as source type and xs:gMonth as target type sh
    // XPath: not(xs:QName(\"ncname\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1458() {
    //  'castable as' involving xs:QName as source type and xs:boolean as target type s
    // XPath: not(xs:QName(\"ncname\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1460() {
    //  'castable as' involving xs:QName as source type and xs:base64Binary as target t
    // XPath: not(xs:QName(\"ncname\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1462() {
    //  'castable as' involving xs:QName as source type and xs:hexBinary as target type
    // XPath: not(xs:QName(\"ncname\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1464() {
    //  'castable as' involving xs:QName as source type and xs:anyURI as target type sh
    // XPath: not(xs:QName(\"ncname\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1465() {
    //  'castable as' involving xs:QName as source type and xs:QName as target type sho
    // XPath: xs:QName(\"ncname\") castable as xs:QName
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_1() {
    //  Cast to xs:QName where the prefix is declared in the prolog.
    // XPath: \"myPrefix:ncname\" cast as xs:QName eq QName(\"http://examp
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_3() {
    //  Pass a small, the smallest if such a limit exists, to nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_4() {
    //  Pass a small, the smallest if such a limit exists, to negativeInteger.
    // XPath: xs:negativeInteger(\"-1\")
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_5() {
    //  Pass a small, the smallest if such a limit exists, to long.
    // XPath: xs:long(\"9223372036854775807\")
    // Expected: 9223372036854775807
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_6() {
    //  Pass a small, the smallest if such a limit exists, to int.
    // XPath: xs:int(\"2147483647\")
    // Expected: 2147483647
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_7() {
    //  Pass a small, the smallest if such a limit exists, to short.
    // XPath: xs:short(\"-32768\")
    // Expected: -32768
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_8() {
    //  Pass a small, the smallest if such a limit exists, to byte.
    // XPath: xs:byte(\"-128\")
    // Expected: -128
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_9() {
    //  Pass a small, the smallest if such a limit exists, to nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_10() {
    //  Pass a small, the smallest if such a limit exists, to unsignedLong.
    // XPath: xs:unsignedLong(\"0\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
