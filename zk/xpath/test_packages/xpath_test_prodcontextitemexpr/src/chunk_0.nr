//! Test chunk 0 for prod:ContextItemExpr
//! Contains 39 tests

use dep::xpath::stub_prodcontextitemexpr;

#[test]
fn externalcontextitem_2() {
    //  context item expression where context item is used as string.
    // XPath: for $var in /works/employee[1] return $var/xs:string(exactly
    // Expected: E1
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_3() {
    //  context item expression where context item is used as an integer.
    // XPath: for $var in (/works/employee[1]) return $var/xs:integer(exac
    // Expected: 40
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_4() {
    //  context item expression where context item is used as a decimal.
    // XPath: for $var in (/works/employee[1]) return $var/xs:decimal(exac
    // Expected: 40
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_5() {
    //  context item expression where context item isused as a float.
    // XPath: for $var in /works/employee[1] return $var/xs:float(exactly-
    // Expected: 40
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_6() {
    //  context item expression where context item is used sa a double.
    // XPath: for $var in (/works/employee[1]) return $var/xs:double(exact
    // Expected: 40
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_7() {
    //  context item expression where context item is used as an xs:boolean.
    // XPath: for $var in (/works/employee[1]) return $var/xs:boolean(exac
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_8() {
    //  context item expression where context item is an xs:boolean used with fn:not().
    // XPath: for $var in (/works/employee[1]) return $var/fn:not(xs:boole
    // Expected: false
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_9() {
    //  context item expression where context item used as argumet to fn:sum.
    // XPath: for $var in (/works/employee[1]) return $var/fn:sum((hours,h
    // Expected: 80
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_10() {
    //  context item expression where context item used in addition operation.
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 80
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_11() {
    //  context item expression where context item used in subtraction operation.
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 0
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_12() {
    //  context item expression where context item used in multiplication operation.
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 1600
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_13() {
    //  context item expression where context item used in mod operation.
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 0
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_14() {
    //  context item expression where context item used in division (div operator) oper
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 1
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_15() {
    //  context item expression where context item used in division (idiv operator) ope
    // XPath: for $var in (/works/employee[1]) return $var/(exactly-one(ho
    // Expected: 1
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_16() {
    //  context item expression where context item used in a boolean (and operator)expr
    // XPath: for $var in (/works/employee[1]) return $var/(xs:boolean(exa
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_17() {
    //  context item expression where context item used in a boolean (or operator)expre
    // XPath: for $var in (/works/employee[1]) return $var/(xs:boolean(exa
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_18() {
    //  context item expression where context item used with string-length function.
    // XPath: for $var in (/works/employee[1]) return $var/fn:string-lengt
    // Expected: 2
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_19() {
    //  context item expression where context item used with 'avg' function.
    // XPath: for $var in (/works/employee[1]) return $var/fn:avg((hours,h
    // Expected: 40
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_20() {
    //  context item expression where context item used with 'min' function.
    // XPath: for $var in (/works/employee[1]) return $var/fn:min((hours,h
    // Expected: 22
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn externalcontextitem_21() {
    //  context item expression where context item used with 'max' function.
    // XPath: for $var in (/works/employee[1]) return $var/fn:max((hours,e
    // Expected: 41
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_2() {
    //  context item expression wher context item is string.
    // XPath: (\"A\",\"B\",\"C\")[xs:string(.)]
    // Expected: A B C
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_3() {
    //  context item expression wher context item is an integer.
    // XPath: (1,2,3)[xs:integer(.)]
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_4() {
    //  context item expression wher context item is a decimal.
    // XPath: (1,2,3)[xs:decimal(.)]
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_5() {
    //  context item expression where context item is a float.
    // XPath: (1,2,3)[xs:float(.)]
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_6() {
    //  context item expression where context item is a double.
    // XPath: (1,2,3)[xs:double(.)]
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_7() {
    //  context item expression where context item is an xs:boolean.
    // XPath: (fn:true(),fn:false(),fn:true())[xs:boolean(.)]
    // Expected: true true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_8() {
    //  context item expression where context item is an xs:boolean used with fn:not().
    // XPath: (fn:false(),fn:true(),fn:false())[fn:not(xs:boolean(.))]
    // Expected: false false
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_10() {
    //  context item expression where context item used in addition operation.
    // XPath: (1,2,3)[(. + .) gt 1]
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_11() {
    //  context item expression where context item used in subtraction operation.
    // XPath: (3,4,5)[(xs:integer(5) - xs:integer(.)) gt 1]
    // Expected: 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_12() {
    //  context item expression where context item used in multiplication operation.
    // XPath: (3,4,5)[(xs:integer(.) * xs:integer(.)) gt 2]
    // Expected: 3 4 5
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_13() {
    //  context item expression where context item used in mod operation.
    // XPath: (6,10,14)[(xs:integer(.) mod xs:integer(3)) gt 1]
    // Expected: 14
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_14() {
    //  context item expression where context item used in division (div operator) oper
    // XPath: (6,10,14)[(xs:integer(.) div xs:integer(3)) gt 2]
    // Expected: 10 14
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_15() {
    //  context item expression where context item used in division (idiv operator) ope
    // XPath: (6,10,14)[(xs:integer(.) idiv xs:integer(3)) gt 2]
    // Expected: 10 14
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_16() {
    //  context item expression where context item used in a boolean (and operator)expr
    // XPath: (fn:true(),fn:false(),fn:true())[xs:boolean(.) and xs:boolea
    // Expected: true true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_17() {
    //  context item expression where context item used in a boolean (or operator)expre
    // XPath: (fn:true(),fn:false(),fn:true())[xs:boolean(.) or xs:boolean
    // Expected: true true
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_18() {
    //  context item expression where context item used with string-length function.
    // XPath: (\"ABC\", \"DEF\",\"A\")[fn:string-length(.) gt 2]
    // Expected: ABC DEF
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_19() {
    //  context item expression where context item used with 'avg' function.
    // XPath: (1,2,3)[fn:avg((.,2,3)) gt 2]
    // Expected: 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_20() {
    //  context item expression where context item used with 'min' function.
    // XPath: (1,2,3)[fn:min((.,2)) eq 2]
    // Expected: 2 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}

#[test]
fn internalcontextitem_21() {
    //  context item expression where context item used with 'max' function.
    // XPath: (1,2,3)[fn:min((.,3)) eq 3]
    // Expected: 3
    // Calls stub function - will fail until prod:ContextItemExpr is implemented
    let _ = stub_prodcontextitemexpr();
}
