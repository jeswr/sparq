//! Test chunk 19 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_1037() {
    //  'castable as' involving xs:gYear as source type and xs:gMonthDay as target type
    // XPath: not(xs:gYear(\"1999\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1039() {
    //  'castable as' involving xs:gYear as source type and xs:gDay as target type shou
    // XPath: not(xs:gYear(\"1999\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1041() {
    //  'castable as' involving xs:gYear as source type and xs:gMonth as target type sh
    // XPath: not(xs:gYear(\"1999\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1043() {
    //  'castable as' involving xs:gYear as source type and xs:boolean as target type s
    // XPath: not(xs:gYear(\"1999\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1045() {
    //  'castable as' involving xs:gYear as source type and xs:base64Binary as target t
    // XPath: not(xs:gYear(\"1999\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1047() {
    //  'castable as' involving xs:gYear as source type and xs:hexBinary as target type
    // XPath: not(xs:gYear(\"1999\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1049() {
    //  'castable as' involving xs:gYear as source type and xs:anyURI as target type sh
    // XPath: not(xs:gYear(\"1999\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1051() {
    //  'castable as' involving xs:gYear as source type and xs:QName as target type sho
    // XPath: not(xs:gYear(\"1999\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1055() {
    //  Verify with 'instance of' that the xs:gMonthDay constructor function produces v
    // XPath: subsequence((\"dummy\", 1.1, xs:gMonthDay(\"--11-13\")), 3,
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1056() {
    //  A simple test exercising the whitespace facet for type xs:gMonthDay.
    // XPath: xs:gMonthDay(\" --11-13 \") eq xs:gMonthDay(\" --11-13 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1058() {
    //  Cast a simple xs:untypedAtomic value to '--11-13' .
    // XPath: xs:gMonthDay(xs:untypedAtomic( \"--11-13\" )) eq xs:gMonthDa
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1060() {
    //  Casting from xs:gMonthDay to xs:untypedAtomic is allowed and should always succ
    // XPath: xs:gMonthDay(\"--11-13\") cast as xs:untypedAtomic ne xs:unt
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1061() {
    //  'castable as' involving xs:gMonthDay as source type and xs:untypedAtomic as tar
    // XPath: xs:gMonthDay(\"--11-13\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1062() {
    //  Casting from xs:gMonthDay to xs:string is allowed and should always succeed.
    // XPath: xs:gMonthDay(\"--11-13\") cast as xs:string ne xs:string(\"a
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1063() {
    //  'castable as' involving xs:gMonthDay as source type and xs:string as target typ
    // XPath: xs:gMonthDay(\"--11-13\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1065() {
    //  'castable as' involving xs:gMonthDay as source type and xs:float as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1067() {
    //  'castable as' involving xs:gMonthDay as source type and xs:double as target typ
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1069() {
    //  'castable as' involving xs:gMonthDay as source type and xs:decimal as target ty
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1071() {
    //  'castable as' involving xs:gMonthDay as source type and xs:integer as target ty
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1073() {
    //  'castable as' involving xs:gMonthDay as source type and xs:duration as target t
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1075() {
    //  'castable as' involving xs:gMonthDay as source type and xs:yearMonthDuration as
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:yearMonthDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1077() {
    //  'castable as' involving xs:gMonthDay as source type and xs:dayTimeDuration as t
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:dayTimeDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1079() {
    //  'castable as' involving xs:gMonthDay as source type and xs:dateTime as target t
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1081() {
    //  'castable as' involving xs:gMonthDay as source type and xs:time as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1083() {
    //  'castable as' involving xs:gMonthDay as source type and xs:date as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1085() {
    //  'castable as' involving xs:gMonthDay as source type and xs:gYearMonth as target
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1087() {
    //  'castable as' involving xs:gMonthDay as source type and xs:gYear as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1088() {
    //  Casting from xs:gMonthDay to xs:gMonthDay is allowed and should always succeed.
    // XPath: xs:gMonthDay(\"--11-13\") cast as xs:gMonthDay eq xs:gMonthD
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1089() {
    //  'castable as' involving xs:gMonthDay as source type and xs:gMonthDay as target
    // XPath: xs:gMonthDay(\"--11-13\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1091() {
    //  'castable as' involving xs:gMonthDay as source type and xs:gDay as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1093() {
    //  'castable as' involving xs:gMonthDay as source type and xs:gMonth as target typ
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1095() {
    //  'castable as' involving xs:gMonthDay as source type and xs:boolean as target ty
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1097() {
    //  'castable as' involving xs:gMonthDay as source type and xs:base64Binary as targ
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1099() {
    //  'castable as' involving xs:gMonthDay as source type and xs:hexBinary as target
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1101() {
    //  'castable as' involving xs:gMonthDay as source type and xs:anyURI as target typ
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1103() {
    //  'castable as' involving xs:gMonthDay as source type and xs:QName as target type
    // XPath: not(xs:gMonthDay(\"--11-13\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1107() {
    //  Verify with 'instance of' that the xs:gDay constructor function produces values
    // XPath: subsequence((\"dummy\", 1.1, xs:gDay(\"---03\")), 3, 1) inst
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1108() {
    //  A simple test exercising the whitespace facet for type xs:gDay.
    // XPath: xs:gDay(\" ---03 \") eq xs:gDay(\" ---03 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1110() {
    //  Cast a simple xs:untypedAtomic value to '---03' .
    // XPath: xs:gDay(xs:untypedAtomic( \"---03\" )) eq xs:gDay(\"---03\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1112() {
    //  Casting from xs:gDay to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:gDay(\"---03\") cast as xs:untypedAtomic ne xs:untypedAto
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1113() {
    //  'castable as' involving xs:gDay as source type and xs:untypedAtomic as target t
    // XPath: xs:gDay(\"---03\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1114() {
    //  Casting from xs:gDay to xs:string is allowed and should always succeed.
    // XPath: xs:gDay(\"---03\") cast as xs:string ne xs:string(\"an arbit
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1115() {
    //  'castable as' involving xs:gDay as source type and xs:string as target type sho
    // XPath: xs:gDay(\"---03\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1117() {
    //  'castable as' involving xs:gDay as source type and xs:float as target type shou
    // XPath: not(xs:gDay(\"---03\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1119() {
    //  'castable as' involving xs:gDay as source type and xs:double as target type sho
    // XPath: not(xs:gDay(\"---03\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1121() {
    //  'castable as' involving xs:gDay as source type and xs:decimal as target type sh
    // XPath: not(xs:gDay(\"---03\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1123() {
    //  'castable as' involving xs:gDay as source type and xs:integer as target type sh
    // XPath: not(xs:gDay(\"---03\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1125() {
    //  'castable as' involving xs:gDay as source type and xs:duration as target type s
    // XPath: not(xs:gDay(\"---03\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1127() {
    //  'castable as' involving xs:gDay as source type and xs:yearMonthDuration as targ
    // XPath: not(xs:gDay(\"---03\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1129() {
    //  'castable as' involving xs:gDay as source type and xs:dayTimeDuration as target
    // XPath: not(xs:gDay(\"---03\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
