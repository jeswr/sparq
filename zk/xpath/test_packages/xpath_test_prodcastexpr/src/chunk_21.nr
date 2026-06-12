//! Test chunk 21 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_1220() {
    //  Casting from xs:boolean to xs:float is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:float ne xs:float(\"3.4e5\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1221() {
    //  'castable as' involving xs:boolean as source type and xs:float as target type s
    // XPath: xs:boolean(\"true\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1222() {
    //  Casting from xs:boolean to xs:double is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:double ne xs:double(\"3.3e3\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1223() {
    //  'castable as' involving xs:boolean as source type and xs:double as target type
    // XPath: xs:boolean(\"true\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1224() {
    //  Casting from xs:boolean to xs:decimal is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:decimal ne xs:decimal(\"10.0
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1225() {
    //  'castable as' involving xs:boolean as source type and xs:decimal as target type
    // XPath: xs:boolean(\"true\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1226() {
    //  Casting from xs:boolean to xs:integer is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:integer ne xs:integer(\"6789
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1227() {
    //  'castable as' involving xs:boolean as source type and xs:integer as target type
    // XPath: xs:boolean(\"true\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1229() {
    //  'castable as' involving xs:boolean as source type and xs:duration as target typ
    // XPath: not(xs:boolean(\"true\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1231() {
    //  'castable as' involving xs:boolean as source type and xs:yearMonthDuration as t
    // XPath: not(xs:boolean(\"true\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1233() {
    //  'castable as' involving xs:boolean as source type and xs:dayTimeDuration as tar
    // XPath: not(xs:boolean(\"true\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1235() {
    //  'castable as' involving xs:boolean as source type and xs:dateTime as target typ
    // XPath: not(xs:boolean(\"true\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1237() {
    //  'castable as' involving xs:boolean as source type and xs:time as target type sh
    // XPath: not(xs:boolean(\"true\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1239() {
    //  'castable as' involving xs:boolean as source type and xs:date as target type sh
    // XPath: not(xs:boolean(\"true\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1241() {
    //  'castable as' involving xs:boolean as source type and xs:gYearMonth as target t
    // XPath: not(xs:boolean(\"true\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1243() {
    //  'castable as' involving xs:boolean as source type and xs:gYear as target type s
    // XPath: not(xs:boolean(\"true\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1245() {
    //  'castable as' involving xs:boolean as source type and xs:gMonthDay as target ty
    // XPath: not(xs:boolean(\"true\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1247() {
    //  'castable as' involving xs:boolean as source type and xs:gDay as target type sh
    // XPath: not(xs:boolean(\"true\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1249() {
    //  'castable as' involving xs:boolean as source type and xs:gMonth as target type
    // XPath: not(xs:boolean(\"true\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1250() {
    //  Casting from xs:boolean to xs:boolean is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:boolean eq xs:boolean(\"true
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1251() {
    //  'castable as' involving xs:boolean as source type and xs:boolean as target type
    // XPath: xs:boolean(\"true\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1253() {
    //  'castable as' involving xs:boolean as source type and xs:base64Binary as target
    // XPath: not(xs:boolean(\"true\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1255() {
    //  'castable as' involving xs:boolean as source type and xs:hexBinary as target ty
    // XPath: not(xs:boolean(\"true\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1257() {
    //  'castable as' involving xs:boolean as source type and xs:anyURI as target type
    // XPath: not(xs:boolean(\"true\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1259() {
    //  'castable as' involving xs:boolean as source type and xs:QName as target type s
    // XPath: not(xs:boolean(\"true\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1263() {
    //  Verify with 'instance of' that the xs:base64Binary constructor function produce
    // XPath: subsequence((\"dummy\", 1.1, xs:base64Binary(\"aaaa\")), 3,
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1264() {
    //  A simple test exercising the whitespace facet for type xs:base64Binary.
    // XPath: xs:base64Binary(\" aaaa \") eq xs:base64Binary(\" aaaa \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1265() {
    //  An empty string is a valid lexical representation of xs:base64Binary.
    // XPath: xs:base64Binary(\"\") eq xs:base64Binary(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1266() {
    //  Cast a simple xs:untypedAtomic value to 'aaaa' .
    // XPath: xs:base64Binary(xs:untypedAtomic( \"aaaa\" )) eq xs:base64Bi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1268() {
    //  Casting from xs:base64Binary to xs:untypedAtomic is allowed and should always s
    // XPath: xs:base64Binary(\"aaaa\") cast as xs:untypedAtomic ne xs:unt
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1269() {
    //  'castable as' involving xs:base64Binary as source type and xs:untypedAtomic as
    // XPath: xs:base64Binary(\"aaaa\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1270() {
    //  Casting from xs:base64Binary to xs:string is allowed and should always succeed.
    // XPath: xs:base64Binary(\"aaaa\") cast as xs:string ne xs:string(\"a
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1271() {
    //  'castable as' involving xs:base64Binary as source type and xs:string as target
    // XPath: xs:base64Binary(\"aaaa\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1273() {
    //  'castable as' involving xs:base64Binary as source type and xs:float as target t
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1275() {
    //  'castable as' involving xs:base64Binary as source type and xs:double as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1277() {
    //  'castable as' involving xs:base64Binary as source type and xs:decimal as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1279() {
    //  'castable as' involving xs:base64Binary as source type and xs:integer as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1281() {
    //  'castable as' involving xs:base64Binary as source type and xs:duration as targe
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1283() {
    //  'castable as' involving xs:base64Binary as source type and xs:yearMonthDuration
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:yearMonthDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1285() {
    //  'castable as' involving xs:base64Binary as source type and xs:dayTimeDuration a
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1287() {
    //  'castable as' involving xs:base64Binary as source type and xs:dateTime as targe
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1289() {
    //  'castable as' involving xs:base64Binary as source type and xs:time as target ty
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1291() {
    //  'castable as' involving xs:base64Binary as source type and xs:date as target ty
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1293() {
    //  'castable as' involving xs:base64Binary as source type and xs:gYearMonth as tar
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1295() {
    //  'castable as' involving xs:base64Binary as source type and xs:gYear as target t
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1297() {
    //  'castable as' involving xs:base64Binary as source type and xs:gMonthDay as targ
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1299() {
    //  'castable as' involving xs:base64Binary as source type and xs:gDay as target ty
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1301() {
    //  'castable as' involving xs:base64Binary as source type and xs:gMonth as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1303() {
    //  'castable as' involving xs:base64Binary as source type and xs:boolean as target
    // XPath: not(xs:base64Binary(\"aaaa\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1304() {
    //  Casting from xs:base64Binary to xs:base64Binary is allowed and should always su
    // XPath: xs:base64Binary(\"aaaa\") cast as xs:base64Binary eq xs:base
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
