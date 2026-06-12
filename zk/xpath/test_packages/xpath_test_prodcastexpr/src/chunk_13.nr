//! Test chunk 13 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_536() {
    //  A simple test exercising the whitespace facet for type xs:decimal.
    // XPath: xs:decimal(\" 10.01 \") eq xs:decimal(\" 10.01 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_538() {
    //  Cast a simple xs:untypedAtomic value to '10.01' .
    // XPath: xs:decimal(xs:untypedAtomic( \"10.01\" )) eq xs:decimal(\"10
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_539() {
    //  Applying fn:boolean() to a value of type xs:decimal that has the lexical value
    // XPath: boolean(xs:decimal(\"10.01\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_540() {
    //  Casting from xs:decimal to xs:untypedAtomic is allowed and should always succee
    // XPath: xs:decimal(\"10.01\") cast as xs:untypedAtomic ne xs:untyped
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_541() {
    //  'castable as' involving xs:decimal as source type and xs:untypedAtomic as targe
    // XPath: xs:decimal(\"10.01\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_542() {
    //  Casting from xs:decimal to xs:string is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:string ne xs:string(\"an ar
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_543() {
    //  'castable as' involving xs:decimal as source type and xs:string as target type
    // XPath: xs:decimal(\"10.01\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_544() {
    //  Casting from xs:decimal to xs:float is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:float ne xs:float(\"3.4e5\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_545() {
    //  'castable as' involving xs:decimal as source type and xs:float as target type s
    // XPath: xs:decimal(\"10.01\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_546() {
    //  Casting from xs:decimal to xs:double is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:double ne xs:double(\"3.3e3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_547() {
    //  'castable as' involving xs:decimal as source type and xs:double as target type
    // XPath: xs:decimal(\"10.01\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_548() {
    //  Casting from xs:decimal to xs:decimal is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:decimal eq xs:decimal(\"10.
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_549() {
    //  'castable as' involving xs:decimal as source type and xs:decimal as target type
    // XPath: xs:decimal(\"10.01\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_550() {
    //  Casting from xs:decimal to xs:integer is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:integer ne xs:integer(\"678
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_551() {
    //  'castable as' involving xs:decimal as source type and xs:integer as target type
    // XPath: xs:decimal(\"10.01\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_553() {
    //  'castable as' involving xs:decimal as source type and xs:duration as target typ
    // XPath: not(xs:decimal(\"10.01\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_555() {
    //  'castable as' involving xs:decimal as source type and xs:yearMonthDuration as t
    // XPath: not(xs:decimal(\"10.01\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_557() {
    //  'castable as' involving xs:decimal as source type and xs:dayTimeDuration as tar
    // XPath: not(xs:decimal(\"10.01\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_559() {
    //  'castable as' involving xs:decimal as source type and xs:dateTime as target typ
    // XPath: not(xs:decimal(\"10.01\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_561() {
    //  'castable as' involving xs:decimal as source type and xs:time as target type sh
    // XPath: not(xs:decimal(\"10.01\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_563() {
    //  'castable as' involving xs:decimal as source type and xs:date as target type sh
    // XPath: not(xs:decimal(\"10.01\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_565() {
    //  'castable as' involving xs:decimal as source type and xs:gYearMonth as target t
    // XPath: not(xs:decimal(\"10.01\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_567() {
    //  'castable as' involving xs:decimal as source type and xs:gYear as target type s
    // XPath: not(xs:decimal(\"10.01\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_569() {
    //  'castable as' involving xs:decimal as source type and xs:gMonthDay as target ty
    // XPath: not(xs:decimal(\"10.01\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_571() {
    //  'castable as' involving xs:decimal as source type and xs:gDay as target type sh
    // XPath: not(xs:decimal(\"10.01\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_573() {
    //  'castable as' involving xs:decimal as source type and xs:gMonth as target type
    // XPath: not(xs:decimal(\"10.01\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_574() {
    //  Casting from xs:decimal to xs:boolean is allowed and should always succeed.
    // XPath: xs:decimal(\"10.01\") cast as xs:boolean eq xs:boolean(\"tru
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_575() {
    //  'castable as' involving xs:decimal as source type and xs:boolean as target type
    // XPath: xs:decimal(\"10.01\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_577() {
    //  'castable as' involving xs:decimal as source type and xs:base64Binary as target
    // XPath: not(xs:decimal(\"10.01\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_579() {
    //  'castable as' involving xs:decimal as source type and xs:hexBinary as target ty
    // XPath: not(xs:decimal(\"10.01\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_581() {
    //  'castable as' involving xs:decimal as source type and xs:anyURI as target type
    // XPath: not(xs:decimal(\"10.01\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_583() {
    //  'castable as' involving xs:decimal as source type and xs:QName as target type s
    // XPath: not(xs:decimal(\"10.01\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_587() {
    //  Verify with 'instance of' that the xs:integer constructor function produces val
    // XPath: subsequence((\"dummy\", 1.1, xs:integer(\"6789\")), 3, 1) in
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_588() {
    //  A simple test exercising the whitespace facet for type xs:integer.
    // XPath: xs:integer(\" 6789 \") eq xs:integer(\" 6789 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_590() {
    //  Cast a simple xs:untypedAtomic value to '6789' .
    // XPath: xs:integer(xs:untypedAtomic( \"6789\" )) eq xs:integer(\"678
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_591() {
    //  Applying fn:boolean() to a value of type xs:integer that has the lexical value
    // XPath: boolean(xs:integer(\"6789\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_592() {
    //  Casting from xs:integer to xs:untypedAtomic is allowed and should always succee
    // XPath: xs:integer(\"6789\") cast as xs:untypedAtomic ne xs:untypedA
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_593() {
    //  'castable as' involving xs:integer as source type and xs:untypedAtomic as targe
    // XPath: xs:integer(\"6789\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_594() {
    //  Casting from xs:integer to xs:string is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:string ne xs:string(\"an arb
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_595() {
    //  'castable as' involving xs:integer as source type and xs:string as target type
    // XPath: xs:integer(\"6789\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_596() {
    //  Casting from xs:integer to xs:float is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:float ne xs:float(\"3.4e5\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_597() {
    //  'castable as' involving xs:integer as source type and xs:float as target type s
    // XPath: xs:integer(\"6789\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_598() {
    //  Casting from xs:integer to xs:double is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:double ne xs:double(\"3.3e3\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_599() {
    //  'castable as' involving xs:integer as source type and xs:double as target type
    // XPath: xs:integer(\"6789\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_600() {
    //  Casting from xs:integer to xs:decimal is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:decimal ne xs:decimal(\"10.0
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_601() {
    //  'castable as' involving xs:integer as source type and xs:decimal as target type
    // XPath: xs:integer(\"6789\") castable as xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_602() {
    //  Casting from xs:integer to xs:integer is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:integer eq xs:integer(\"6789
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_603() {
    //  'castable as' involving xs:integer as source type and xs:integer as target type
    // XPath: xs:integer(\"6789\") castable as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_605() {
    //  'castable as' involving xs:integer as source type and xs:duration as target typ
    // XPath: not(xs:integer(\"6789\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_607() {
    //  'castable as' involving xs:integer as source type and xs:yearMonthDuration as t
    // XPath: not(xs:integer(\"6789\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
