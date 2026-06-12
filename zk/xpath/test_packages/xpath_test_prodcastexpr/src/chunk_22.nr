//! Test chunk 22 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_1305() {
    //  'castable as' involving xs:base64Binary as source type and xs:base64Binary as t
    // XPath: xs:base64Binary(\"aaaa\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1306() {
    //  Casting from xs:base64Binary to xs:hexBinary is allowed and should always succe
    // XPath: xs:base64Binary(\"aaaa\") cast as xs:hexBinary ne xs:hexBina
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1307() {
    //  'castable as' involving xs:base64Binary as source type and xs:hexBinary as targ
    // XPath: xs:base64Binary(\"aaaa\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1309() {
    //  'castable as' involving xs:base64Binary as source type and xs:anyURI as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1311() {
    //  'castable as' involving xs:base64Binary as source type and xs:QName as target t
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1315() {
    //  Verify with 'instance of' that the xs:hexBinary constructor function produces v
    // XPath: subsequence((\"dummy\", 1.1, xs:hexBinary(\"0FB7\")), 3, 1)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1316() {
    //  A simple test exercising the whitespace facet for type xs:hexBinary.
    // XPath: xs:hexBinary(\" 0FB7 \") eq xs:hexBinary(\" 0FB7 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1317() {
    //  An empty string is a valid lexical representation of xs:hexBinary.
    // XPath: xs:hexBinary(\"\") eq xs:hexBinary(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1318() {
    //  Cast a simple xs:untypedAtomic value to '0FB7' .
    // XPath: xs:hexBinary(xs:untypedAtomic( \"0FB7\" )) eq xs:hexBinary(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1320() {
    //  Casting from xs:hexBinary to xs:untypedAtomic is allowed and should always succ
    // XPath: xs:hexBinary(\"0FB7\") cast as xs:untypedAtomic ne xs:untype
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1321() {
    //  'castable as' involving xs:hexBinary as source type and xs:untypedAtomic as tar
    // XPath: xs:hexBinary(\"0FB7\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1322() {
    //  Casting from xs:hexBinary to xs:string is allowed and should always succeed.
    // XPath: xs:hexBinary(\"0FB7\") cast as xs:string ne xs:string(\"an a
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1323() {
    //  'castable as' involving xs:hexBinary as source type and xs:string as target typ
    // XPath: xs:hexBinary(\"0FB7\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1325() {
    //  'castable as' involving xs:hexBinary as source type and xs:float as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1327() {
    //  'castable as' involving xs:hexBinary as source type and xs:double as target typ
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1329() {
    //  'castable as' involving xs:hexBinary as source type and xs:decimal as target ty
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1331() {
    //  'castable as' involving xs:hexBinary as source type and xs:integer as target ty
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1333() {
    //  'castable as' involving xs:hexBinary as source type and xs:duration as target t
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1335() {
    //  'castable as' involving xs:hexBinary as source type and xs:yearMonthDuration as
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1337() {
    //  'castable as' involving xs:hexBinary as source type and xs:dayTimeDuration as t
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1339() {
    //  'castable as' involving xs:hexBinary as source type and xs:dateTime as target t
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1341() {
    //  'castable as' involving xs:hexBinary as source type and xs:time as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1343() {
    //  'castable as' involving xs:hexBinary as source type and xs:date as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1345() {
    //  'castable as' involving xs:hexBinary as source type and xs:gYearMonth as target
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1347() {
    //  'castable as' involving xs:hexBinary as source type and xs:gYear as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1349() {
    //  'castable as' involving xs:hexBinary as source type and xs:gMonthDay as target
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1351() {
    //  'castable as' involving xs:hexBinary as source type and xs:gDay as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1353() {
    //  'castable as' involving xs:hexBinary as source type and xs:gMonth as target typ
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1355() {
    //  'castable as' involving xs:hexBinary as source type and xs:boolean as target ty
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1356() {
    //  Casting from xs:hexBinary to xs:base64Binary is allowed and should always succe
    // XPath: xs:hexBinary(\"0FB7\") cast as xs:base64Binary ne xs:base64B
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1357() {
    //  'castable as' involving xs:hexBinary as source type and xs:base64Binary as targ
    // XPath: xs:hexBinary(\"0FB7\") castable as xs:base64Binary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1358() {
    //  Casting from xs:hexBinary to xs:hexBinary is allowed and should always succeed.
    // XPath: xs:hexBinary(\"0FB7\") cast as xs:hexBinary eq xs:hexBinary(
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1359() {
    //  'castable as' involving xs:hexBinary as source type and xs:hexBinary as target
    // XPath: xs:hexBinary(\"0FB7\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1361() {
    //  'castable as' involving xs:hexBinary as source type and xs:anyURI as target typ
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1363() {
    //  'castable as' involving xs:hexBinary as source type and xs:QName as target type
    // XPath: not(xs:hexBinary(\"0FB7\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1367() {
    //  Verify with 'instance of' that the xs:anyURI constructor function produces valu
    // XPath: subsequence((\"dummy\", 1.1, xs:anyURI(\"http://www.example.
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1368() {
    //  A simple test exercising the whitespace facet for type xs:anyURI.
    // XPath: xs:anyURI(\" http://www.example.com/an/arbitrary/URI.ext \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1369() {
    //  An empty string is a valid lexical representation of xs:anyURI.
    // XPath: xs:anyURI(\"\") eq xs:anyURI(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1370() {
    //  Cast a simple xs:untypedAtomic value to 'http://www.example.com/an/arbitrary/UR
    // XPath: xs:anyURI(xs:untypedAtomic( \"http://www.example.com/an/arbi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1371() {
    //  Applying fn:boolean() to a value of type xs:anyURI that has the lexical value
    // XPath: boolean(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1372() {
    //  Casting from xs:anyURI to xs:untypedAtomic is allowed and should always succeed
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1373() {
    //  'castable as' involving xs:anyURI as source type and xs:untypedAtomic as target
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1374() {
    //  Casting from xs:anyURI to xs:string is allowed and should always succeed.
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1375() {
    //  'castable as' involving xs:anyURI as source type and xs:string as target type s
    // XPath: xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\") c
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1377() {
    //  'castable as' involving xs:anyURI as source type and xs:float as target type sh
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1379() {
    //  'castable as' involving xs:anyURI as source type and xs:double as target type s
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1381() {
    //  'castable as' involving xs:anyURI as source type and xs:decimal as target type
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1383() {
    //  'castable as' involving xs:anyURI as source type and xs:integer as target type
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1385() {
    //  'castable as' involving xs:anyURI as source type and xs:duration as target type
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1387() {
    //  'castable as' involving xs:anyURI as source type and xs:yearMonthDuration as ta
    // XPath: not(xs:anyURI(\"http://www.example.com/an/arbitrary/URI.ext\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
