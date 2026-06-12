//! Test chunk 18 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_945() {
    //  'castable as' involving xs:date as source type and xs:anyURI as target type sho
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_947() {
    //  'castable as' involving xs:date as source type and xs:QName as target type shou
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_951() {
    //  Verify with 'instance of' that the xs:gYearMonth constructor function produces
    // XPath: subsequence((\"dummy\", 1.1, xs:gYearMonth(\"1999-11\")), 3,
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_952() {
    //  A simple test exercising the whitespace facet for type xs:gYearMonth.
    // XPath: xs:gYearMonth(\" 1999-11 \") eq xs:gYearMonth(\" 1999-11 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_954() {
    //  Cast a simple xs:untypedAtomic value to '1999-11' .
    // XPath: xs:gYearMonth(xs:untypedAtomic( \"1999-11\" )) eq xs:gYearMo
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_956() {
    //  Casting from xs:gYearMonth to xs:untypedAtomic is allowed and should always suc
    // XPath: xs:gYearMonth(\"1999-11\") cast as xs:untypedAtomic ne xs:un
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_957() {
    //  'castable as' involving xs:gYearMonth as source type and xs:untypedAtomic as ta
    // XPath: xs:gYearMonth(\"1999-11\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_958() {
    //  Casting from xs:gYearMonth to xs:string is allowed and should always succeed.
    // XPath: xs:gYearMonth(\"1999-11\") cast as xs:string ne xs:string(\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_959() {
    //  'castable as' involving xs:gYearMonth as source type and xs:string as target ty
    // XPath: xs:gYearMonth(\"1999-11\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_961() {
    //  'castable as' involving xs:gYearMonth as source type and xs:float as target typ
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_963() {
    //  'castable as' involving xs:gYearMonth as source type and xs:double as target ty
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_965() {
    //  'castable as' involving xs:gYearMonth as source type and xs:decimal as target t
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_967() {
    //  'castable as' involving xs:gYearMonth as source type and xs:integer as target t
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_969() {
    //  'castable as' involving xs:gYearMonth as source type and xs:duration as target
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_971() {
    //  'castable as' involving xs:gYearMonth as source type and xs:yearMonthDuration a
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:yearMonthDurat
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_973() {
    //  'castable as' involving xs:gYearMonth as source type and xs:dayTimeDuration as
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:dayTimeDuratio
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_975() {
    //  'castable as' involving xs:gYearMonth as source type and xs:dateTime as target
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_977() {
    //  'castable as' involving xs:gYearMonth as source type and xs:time as target type
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_979() {
    //  'castable as' involving xs:gYearMonth as source type and xs:date as target type
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_980() {
    //  Casting from xs:gYearMonth to xs:gYearMonth is allowed and should always succee
    // XPath: xs:gYearMonth(\"1999-11\") cast as xs:gYearMonth eq xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_981() {
    //  'castable as' involving xs:gYearMonth as source type and xs:gYearMonth as targe
    // XPath: xs:gYearMonth(\"1999-11\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_983() {
    //  'castable as' involving xs:gYearMonth as source type and xs:gYear as target typ
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_985() {
    //  'castable as' involving xs:gYearMonth as source type and xs:gMonthDay as target
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_987() {
    //  'castable as' involving xs:gYearMonth as source type and xs:gDay as target type
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_989() {
    //  'castable as' involving xs:gYearMonth as source type and xs:gMonth as target ty
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_991() {
    //  'castable as' involving xs:gYearMonth as source type and xs:boolean as target t
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_993() {
    //  'castable as' involving xs:gYearMonth as source type and xs:base64Binary as tar
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_995() {
    //  'castable as' involving xs:gYearMonth as source type and xs:hexBinary as target
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_997() {
    //  'castable as' involving xs:gYearMonth as source type and xs:anyURI as target ty
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_999() {
    //  'castable as' involving xs:gYearMonth as source type and xs:QName as target typ
    // XPath: not(xs:gYearMonth(\"1999-11\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1003() {
    //  Verify with 'instance of' that the xs:gYear constructor function produces value
    // XPath: subsequence((\"dummy\", 1.1, xs:gYear(\"1999\")), 3, 1) inst
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1004() {
    //  A simple test exercising the whitespace facet for type xs:gYear.
    // XPath: xs:gYear(\" 1999 \") eq xs:gYear(\" 1999 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1006() {
    //  Cast a simple xs:untypedAtomic value to '1999' .
    // XPath: xs:gYear(xs:untypedAtomic( \"1999\" )) eq xs:gYear(\"1999\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1008() {
    //  Casting from xs:gYear to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:gYear(\"1999\") cast as xs:untypedAtomic ne xs:untypedAto
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1009() {
    //  'castable as' involving xs:gYear as source type and xs:untypedAtomic as target
    // XPath: xs:gYear(\"1999\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1010() {
    //  Casting from xs:gYear to xs:string is allowed and should always succeed.
    // XPath: xs:gYear(\"1999\") cast as xs:string ne xs:string(\"an arbit
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1011() {
    //  'castable as' involving xs:gYear as source type and xs:string as target type sh
    // XPath: xs:gYear(\"1999\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1013() {
    //  'castable as' involving xs:gYear as source type and xs:float as target type sho
    // XPath: not(xs:gYear(\"1999\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1015() {
    //  'castable as' involving xs:gYear as source type and xs:double as target type sh
    // XPath: not(xs:gYear(\"1999\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1017() {
    //  'castable as' involving xs:gYear as source type and xs:decimal as target type s
    // XPath: not(xs:gYear(\"1999\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1019() {
    //  'castable as' involving xs:gYear as source type and xs:integer as target type s
    // XPath: not(xs:gYear(\"1999\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1021() {
    //  'castable as' involving xs:gYear as source type and xs:duration as target type
    // XPath: not(xs:gYear(\"1999\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1023() {
    //  'castable as' involving xs:gYear as source type and xs:yearMonthDuration as tar
    // XPath: not(xs:gYear(\"1999\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1025() {
    //  'castable as' involving xs:gYear as source type and xs:dayTimeDuration as targe
    // XPath: not(xs:gYear(\"1999\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1027() {
    //  'castable as' involving xs:gYear as source type and xs:dateTime as target type
    // XPath: not(xs:gYear(\"1999\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1029() {
    //  'castable as' involving xs:gYear as source type and xs:time as target type shou
    // XPath: not(xs:gYear(\"1999\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1031() {
    //  'castable as' involving xs:gYear as source type and xs:date as target type shou
    // XPath: not(xs:gYear(\"1999\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1033() {
    //  'castable as' involving xs:gYear as source type and xs:gYearMonth as target typ
    // XPath: not(xs:gYear(\"1999\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1034() {
    //  Casting from xs:gYear to xs:gYear is allowed and should always succeed.
    // XPath: xs:gYear(\"1999\") cast as xs:gYear eq xs:gYear(\"1999\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1035() {
    //  'castable as' involving xs:gYear as source type and xs:gYear as target type sho
    // XPath: xs:gYear(\"1999\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
