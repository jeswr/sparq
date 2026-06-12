//! Test chunk 15 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_698() {
    //  Casting from xs:yearMonthDuration to xs:string is allowed and should always suc
    // XPath: xs:yearMonthDuration(\"P1Y12M\") cast as xs:string ne xs:str
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_699() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:string as ta
    // XPath: xs:yearMonthDuration(\"P1Y12M\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_701() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:float as tar
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_703() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:double as ta
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_705() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:decimal as t
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_707() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:integer as t
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_708() {
    //  Casting from xs:yearMonthDuration to xs:duration is allowed and should always s
    // XPath: xs:yearMonthDuration(\"P1Y12M\") cast as xs:duration ne xs:d
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_709() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:duration as
    // XPath: xs:yearMonthDuration(\"P1Y12M\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_710() {
    //  Casting from xs:yearMonthDuration to xs:yearMonthDuration is allowed and should
    // XPath: xs:yearMonthDuration(\"P1Y12M\") cast as xs:yearMonthDuratio
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_711() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:yearMonthDur
    // XPath: xs:yearMonthDuration(\"P1Y12M\") castable as xs:yearMonthDur
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_712() {
    //  Casting from xs:yearMonthDuration to xs:dayTimeDuration is allowed and should a
    // XPath: xs:yearMonthDuration(\"P1Y12M\") cast as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_713() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:dayTimeDurat
    // XPath: xs:yearMonthDuration(\"P1Y12M\") castable as xs:dayTimeDurat
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_715() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:dateTime as
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_717() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:time as targ
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_719() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:date as targ
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_721() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:gYearMonth a
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:gYearMon
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_723() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:gYear as tar
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_725() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:gMonthDay as
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:gMonthDa
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_727() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:gDay as targ
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_729() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:gMonth as ta
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_731() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:boolean as t
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_733() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:base64Binary
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:base64Bi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_735() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:hexBinary as
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:hexBinar
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_737() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:anyURI as ta
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_739() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:QName as tar
    // XPath: not(xs:yearMonthDuration(\"P1Y12M\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_743() {
    //  Verify with 'instance of' that the xs:dayTimeDuration constructor function prod
    // XPath: subsequence((\"dummy\", 1.1, xs:dayTimeDuration(\"P3DT2H\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_744() {
    //  A simple test exercising the whitespace facet for type xs:dayTimeDuration.
    // XPath: xs:dayTimeDuration(\" P3DT2H \") eq xs:dayTimeDuration(\" P3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_746() {
    //  Cast a simple xs:untypedAtomic value to 'P3DT2H' .
    // XPath: xs:dayTimeDuration(xs:untypedAtomic( \"P3DT2H\" )) eq xs:day
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_748() {
    //  Casting from xs:dayTimeDuration to xs:untypedAtomic is allowed and should alway
    // XPath: xs:dayTimeDuration(\"P3DT2H\") cast as xs:untypedAtomic ne x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_749() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:untypedAtomic
    // XPath: xs:dayTimeDuration(\"P3DT2H\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_750() {
    //  Casting from xs:dayTimeDuration to xs:string is allowed and should always succe
    // XPath: xs:dayTimeDuration(\"P3DT2H\") cast as xs:string ne xs:strin
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_751() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:string as targ
    // XPath: xs:dayTimeDuration(\"P3DT2H\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_753() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:float as targe
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_755() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:double as targ
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_757() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:decimal as tar
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_759() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:integer as tar
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_760() {
    //  Casting from xs:dayTimeDuration to xs:duration is allowed and should always suc
    // XPath: xs:dayTimeDuration(\"P3DT2H\") cast as xs:duration ne xs:dur
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_761() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:duration as ta
    // XPath: xs:dayTimeDuration(\"P3DT2H\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_762() {
    //  Casting from xs:dayTimeDuration to xs:yearMonthDuration is allowed and should a
    // XPath: xs:dayTimeDuration(\"P3DT2H\") cast as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_763() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:yearMonthDurat
    // XPath: xs:dayTimeDuration(\"P3DT2H\") castable as xs:yearMonthDurat
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_764() {
    //  Casting from xs:dayTimeDuration to xs:dayTimeDuration is allowed and should alw
    // XPath: xs:dayTimeDuration(\"P3DT2H\") cast as xs:dayTimeDuration eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_765() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:dayTimeDuratio
    // XPath: xs:dayTimeDuration(\"P3DT2H\") castable as xs:dayTimeDuratio
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_767() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:dateTime as ta
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_769() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:time as target
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_771() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:date as target
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_773() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:gYearMonth as
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_775() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:gYear as targe
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_777() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:gMonthDay as t
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_779() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:gDay as target
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_781() {
    //  'castable as' involving xs:dayTimeDuration as source type and xs:gMonth as targ
    // XPath: not(xs:dayTimeDuration(\"P3DT2H\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
