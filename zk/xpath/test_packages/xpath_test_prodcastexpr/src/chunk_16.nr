//! Test chunk 16 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_783() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:boolean as tar
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_785() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:base64Binary a
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:base64Bina
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_787() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:hexBinary as t
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_789() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:anyURI as targ
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_791() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:QName as targe
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_795() {
    //  Verify with 'instance of' that the xs:dateTime constructor function produces va
    // XPath: subsequence((\"dummy\", 1.1, xs:dateTime(\"2002-10-10T12:00:
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_796() {
    //  A simple test exercising the whitespace facet for type xs:dateTime.
    // XPath: xs:dateTime(\" 2002-10-10T12:00:00-05:00 \") eq xs:dateTime(
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_798() {
    //  Cast a simple xs:untypedAtomic value to '2002-10-10T12:00:00-05:00' .
    // XPath: xs:dateTime(xs:untypedAtomic( \"2002-10-10T12:00:00-05:00\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_800() {
    //  Casting from xs:dateTime to xs:untypedAtomic is allowed and should always succe
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:untype
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_801() {
    //  'castable as' involving xs:dateTime as source type and xs:untypedAtomic as targ
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:un
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_802() {
    //  Casting from xs:dateTime to xs:string is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_803() {
    //  'castable as' involving xs:dateTime as source type and xs:string as target type
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:st
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_805() {
    //  'castable as' involving xs:dateTime as source type and xs:float as target type
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_807() {
    //  'castable as' involving xs:dateTime as source type and xs:double as target type
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_809() {
    //  'castable as' involving xs:dateTime as source type and xs:decimal as target typ
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_811() {
    //  'castable as' involving xs:dateTime as source type and xs:integer as target typ
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_813() {
    //  'castable as' involving xs:dateTime as source type and xs:duration as target ty
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_815() {
    //  'castable as' involving xs:dateTime as source type and xs:yearMonthDuration as
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_817() {
    //  'castable as' involving xs:dateTime as source type and xs:dayTimeDuration as ta
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_818() {
    //  Casting from xs:dateTime to xs:dateTime is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:dateTi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_819() {
    //  'castable as' involving xs:dateTime as source type and xs:dateTime as target ty
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:da
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_820() {
    //  Casting from xs:dateTime to xs:time is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:time n
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_821() {
    //  'castable as' involving xs:dateTime as source type and xs:time as target type s
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:ti
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_822() {
    //  Casting from xs:dateTime to xs:date is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:date n
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_823() {
    //  'castable as' involving xs:dateTime as source type and xs:date as target type s
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:da
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_824() {
    //  Casting from xs:dateTime to xs:gYearMonth is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:gYearM
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_825() {
    //  'castable as' involving xs:dateTime as source type and xs:gYearMonth as target
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:gY
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_826() {
    //  Casting from xs:dateTime to xs:gYear is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_827() {
    //  'castable as' involving xs:dateTime as source type and xs:gYear as target type
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:gY
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_828() {
    //  Casting from xs:dateTime to xs:gMonthDay is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_829() {
    //  'castable as' involving xs:dateTime as source type and xs:gMonthDay as target t
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:gM
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_830() {
    //  Casting from xs:dateTime to xs:gDay is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:gDay n
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_831() {
    //  'castable as' involving xs:dateTime as source type and xs:gDay as target type s
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:gD
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_832() {
    //  Casting from xs:dateTime to xs:gMonth is allowed and should always succeed.
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") cast as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_833() {
    //  'castable as' involving xs:dateTime as source type and xs:gMonth as target type
    // XPath: xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as xs:gM
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_835() {
    //  'castable as' involving xs:dateTime as source type and xs:boolean as target typ
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_837() {
    //  'castable as' involving xs:dateTime as source type and xs:base64Binary as targe
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_839() {
    //  'castable as' involving xs:dateTime as source type and xs:hexBinary as target t
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_841() {
    //  'castable as' involving xs:dateTime as source type and xs:anyURI as target type
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_843() {
    //  'castable as' involving xs:dateTime as source type and xs:QName as target type
    // XPath: not(xs:dateTime(\"2002-10-10T12:00:00-05:00\") castable as x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_847() {
    //  Verify with 'instance of' that the xs:time constructor function produces values
    // XPath: subsequence((\"dummy\", 1.1, xs:time(\"03:20:00-05:00\")), 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_848() {
    //  A simple test exercising the whitespace facet for type xs:time.
    // XPath: xs:time(\" 03:20:00-05:00 \") eq xs:time(\" 03:20:00-05:00 \
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_850() {
    //  Cast a simple xs:untypedAtomic value to '03:20:00-05:00' .
    // XPath: xs:time(xs:untypedAtomic( \"03:20:00-05:00\" )) eq xs:time(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_852() {
    //  Casting from xs:time to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:time(\"03:20:00-05:00\") cast as xs:untypedAtomic ne xs:u
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_853() {
    //  'castable as' involving xs:time as source type and xs:untypedAtomic as target t
    // XPath: xs:time(\"03:20:00-05:00\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_854() {
    //  Casting from xs:time to xs:string is allowed and should always succeed.
    // XPath: xs:time(\"03:20:00-05:00\") cast as xs:string ne xs:string(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_855() {
    //  'castable as' involving xs:time as source type and xs:string as target type sho
    // XPath: xs:time(\"03:20:00-05:00\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_857() {
    //  'castable as' involving xs:time as source type and xs:float as target type shou
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_859() {
    //  'castable as' involving xs:time as source type and xs:double as target type sho
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_861() {
    //  'castable as' involving xs:time as source type and xs:decimal as target type sh
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
