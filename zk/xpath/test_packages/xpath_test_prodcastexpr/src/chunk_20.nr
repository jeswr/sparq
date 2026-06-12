//! Test chunk 20 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_1131() {
    //  'castable as' involving xs:gDay as source type and xs:dateTime as target type s
    // XPath: not(xs:gDay(\"---03\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1133() {
    //  'castable as' involving xs:gDay as source type and xs:time as target type shoul
    // XPath: not(xs:gDay(\"---03\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1135() {
    //  'castable as' involving xs:gDay as source type and xs:date as target type shoul
    // XPath: not(xs:gDay(\"---03\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1137() {
    //  'castable as' involving xs:gDay as source type and xs:gYearMonth as target type
    // XPath: not(xs:gDay(\"---03\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1139() {
    //  'castable as' involving xs:gDay as source type and xs:gYear as target type shou
    // XPath: not(xs:gDay(\"---03\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1141() {
    //  'castable as' involving xs:gDay as source type and xs:gMonthDay as target type
    // XPath: not(xs:gDay(\"---03\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1142() {
    //  Casting from xs:gDay to xs:gDay is allowed and should always succeed.
    // XPath: xs:gDay(\"---03\") cast as xs:gDay eq xs:gDay(\"---03\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1143() {
    //  'castable as' involving xs:gDay as source type and xs:gDay as target type shoul
    // XPath: xs:gDay(\"---03\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1145() {
    //  'castable as' involving xs:gDay as source type and xs:gMonth as target type sho
    // XPath: not(xs:gDay(\"---03\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1147() {
    //  'castable as' involving xs:gDay as source type and xs:boolean as target type sh
    // XPath: not(xs:gDay(\"---03\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1149() {
    //  'castable as' involving xs:gDay as source type and xs:base64Binary as target ty
    // XPath: not(xs:gDay(\"---03\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1151() {
    //  'castable as' involving xs:gDay as source type and xs:hexBinary as target type
    // XPath: not(xs:gDay(\"---03\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1153() {
    //  'castable as' involving xs:gDay as source type and xs:anyURI as target type sho
    // XPath: not(xs:gDay(\"---03\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1155() {
    //  'castable as' involving xs:gDay as source type and xs:QName as target type shou
    // XPath: not(xs:gDay(\"---03\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1159() {
    //  Verify with 'instance of' that the xs:gMonth constructor function produces valu
    // XPath: subsequence((\"dummy\", 1.1, xs:gMonth(\"--11\")), 3, 1) ins
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1160() {
    //  A simple test exercising the whitespace facet for type xs:gMonth.
    // XPath: xs:gMonth(\" --11 \") eq xs:gMonth(\" --11 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1162() {
    //  Cast a simple xs:untypedAtomic value to '--11' .
    // XPath: xs:gMonth(xs:untypedAtomic( \"--11\" )) eq xs:gMonth(\"--11\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1164() {
    //  Casting from xs:gMonth to xs:untypedAtomic is allowed and should always succeed
    // XPath: xs:gMonth(\"--11\") cast as xs:untypedAtomic ne xs:untypedAt
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1165() {
    //  'castable as' involving xs:gMonth as source type and xs:untypedAtomic as target
    // XPath: xs:gMonth(\"--11\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1166() {
    //  Casting from xs:gMonth to xs:string is allowed and should always succeed.
    // XPath: xs:gMonth(\"--11\") cast as xs:string ne xs:string(\"an arbi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1167() {
    //  'castable as' involving xs:gMonth as source type and xs:string as target type s
    // XPath: xs:gMonth(\"--11\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1169() {
    //  'castable as' involving xs:gMonth as source type and xs:float as target type sh
    // XPath: not(xs:gMonth(\"--11\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1171() {
    //  'castable as' involving xs:gMonth as source type and xs:double as target type s
    // XPath: not(xs:gMonth(\"--11\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1173() {
    //  'castable as' involving xs:gMonth as source type and xs:decimal as target type
    // XPath: not(xs:gMonth(\"--11\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1175() {
    //  'castable as' involving xs:gMonth as source type and xs:integer as target type
    // XPath: not(xs:gMonth(\"--11\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1177() {
    //  'castable as' involving xs:gMonth as source type and xs:duration as target type
    // XPath: not(xs:gMonth(\"--11\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1179() {
    //  'castable as' involving xs:gMonth as source type and xs:yearMonthDuration as ta
    // XPath: not(xs:gMonth(\"--11\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1181() {
    //  'castable as' involving xs:gMonth as source type and xs:dayTimeDuration as targ
    // XPath: not(xs:gMonth(\"--11\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1183() {
    //  'castable as' involving xs:gMonth as source type and xs:dateTime as target type
    // XPath: not(xs:gMonth(\"--11\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1185() {
    //  'castable as' involving xs:gMonth as source type and xs:time as target type sho
    // XPath: not(xs:gMonth(\"--11\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1187() {
    //  'castable as' involving xs:gMonth as source type and xs:date as target type sho
    // XPath: not(xs:gMonth(\"--11\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1189() {
    //  'castable as' involving xs:gMonth as source type and xs:gYearMonth as target ty
    // XPath: not(xs:gMonth(\"--11\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1191() {
    //  'castable as' involving xs:gMonth as source type and xs:gYear as target type sh
    // XPath: not(xs:gMonth(\"--11\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1193() {
    //  'castable as' involving xs:gMonth as source type and xs:gMonthDay as target typ
    // XPath: not(xs:gMonth(\"--11\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1195() {
    //  'castable as' involving xs:gMonth as source type and xs:gDay as target type sho
    // XPath: not(xs:gMonth(\"--11\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1196() {
    //  Casting from xs:gMonth to xs:gMonth is allowed and should always succeed.
    // XPath: xs:gMonth(\"--11\") cast as xs:gMonth eq xs:gMonth(\"--11\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1197() {
    //  'castable as' involving xs:gMonth as source type and xs:gMonth as target type s
    // XPath: xs:gMonth(\"--11\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1199() {
    //  'castable as' involving xs:gMonth as source type and xs:boolean as target type
    // XPath: not(xs:gMonth(\"--11\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1201() {
    //  'castable as' involving xs:gMonth as source type and xs:base64Binary as target
    // XPath: not(xs:gMonth(\"--11\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1203() {
    //  'castable as' involving xs:gMonth as source type and xs:hexBinary as target typ
    // XPath: not(xs:gMonth(\"--11\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1205() {
    //  'castable as' involving xs:gMonth as source type and xs:anyURI as target type s
    // XPath: not(xs:gMonth(\"--11\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1207() {
    //  'castable as' involving xs:gMonth as source type and xs:QName as target type sh
    // XPath: not(xs:gMonth(\"--11\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1211() {
    //  Verify with 'instance of' that the xs:boolean constructor function produces val
    // XPath: subsequence((\"dummy\", 1.1, xs:boolean(\"true\")), 3, 1) in
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1212() {
    //  A simple test exercising the whitespace facet for type xs:boolean.
    // XPath: xs:boolean(\" true \") eq xs:boolean(\" true \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1214() {
    //  Cast a simple xs:untypedAtomic value to 'true' .
    // XPath: xs:boolean(xs:untypedAtomic( \"true\" )) eq xs:boolean(\"tru
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1215() {
    //  Applying fn:boolean() to a value of type xs:boolean that has the lexical value
    // XPath: boolean(xs:boolean(\"true\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1216() {
    //  Casting from xs:boolean to xs:untypedAtomic is allowed and should always succee
    // XPath: xs:boolean(\"true\") cast as xs:untypedAtomic ne xs:untypedA
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1217() {
    //  'castable as' involving xs:boolean as source type and xs:untypedAtomic as targe
    // XPath: xs:boolean(\"true\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1218() {
    //  Casting from xs:boolean to xs:string is allowed and should always succeed.
    // XPath: xs:boolean(\"true\") cast as xs:string ne xs:string(\"an arb
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_1219() {
    //  'castable as' involving xs:boolean as source type and xs:string as target type
    // XPath: xs:boolean(\"true\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
