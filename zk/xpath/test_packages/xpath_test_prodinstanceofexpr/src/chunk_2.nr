//! Test chunk 2 for prod:InstanceofExpr
//! Contains 50 tests

use dep::xpath::stub_prodinstanceofexpr;

#[test]
fn instanceof101() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:time
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof102() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:date
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof103() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:boolean
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof104() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:float
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof105() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:double
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof106() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:anyURI
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof107() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:string
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof108() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof109() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof110() {
    // purpose: Evaluation of 'instance of' expression for pattern 'integer instance of
    // XPath: 12678967543233 instance of xs:int
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof121() {
    // Instance of with function types
    // XPath: name#1 instance of function(*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof122() {
    // Instance of with function types
    // XPath: name#1 instance of function(node()?) as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof123() {
    // Instance of with function types
    // XPath: name#1 instance of function(node()) as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof124() {
    // Instance of with function types
    // XPath: name#1 instance of function(element(A)?) as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof125() {
    // Instance of with function types
    // XPath: name#1 instance of function(element(A)) as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof126() {
    // Instance of with function types
    // XPath: name#1 instance of function(element(A)) as xs:anyAtomicType
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof127() {
    // Instance of with function types
    // XPath: name#1 instance of function(element(A)) as xs:string+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof128() {
    // Instance of with function types
    // XPath: name#1 instance of function(element(A)) as xs:NCName
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof129() {
    // Instance of with function types
    // XPath: name#1 instance of function(node()*) as xs:string
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof130() {
    // Instance of with function types
    // XPath: name#1 instance of function(item()) as xs:string
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof131() {
    // Instance of with function types
    // XPath: name#0 instance of function(item()) as xs:string
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof132() {
    // Instance of with function types
    // XPath: filter#2 instance of function(item()*, function(item()) as x
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof133() {
    // Instance of with function types
    // XPath: filter#2 instance of function(item()*, function(item()*) as
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn instanceof134() {
    // Instance of with function types
    // XPath: filter#2 instance of function(function(*), item()*) as item(
    // Expected: false
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_1() {
    //  A test whose essence is: `1 instance of item()`.
    // XPath: 1 instance of item()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_2() {
    //  item() type with comment appearing inside the paranteses. comment
    // XPath: 1 instance of item()
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_3() {
    //  A test whose essence is: `1 instance of xs:integer`.
    // XPath: 1 instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_4() {
    //  A test whose essence is: `1 instance of xs:decimal`.
    // XPath: 1 instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_5() {
    //  A test whose essence is: `1 instance of xs:integer?`.
    // XPath: 1 instance of xs:integer?
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_6() {
    //  A test whose essence is: `1 instance of xs:integer+`.
    // XPath: 1 instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_7() {
    //  A test whose essence is: `1 instance of xs:integer*`.
    // XPath: 1 instance of xs:integer*
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_8() {
    //  A test whose essence is: `false() instance of xs:boolean`.
    // XPath: false() instance of xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_9() {
    //  A test whose essence is: `false() instance of xs:anyAtomicType`.
    // XPath: false() instance of xs:anyAtomicType
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_10() {
    //  A test whose essence is: `'a string' instance of xs:string`.
    // XPath: \"a string\" instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_11() {
    //  A test whose essence is: `not('false' instance of xs:boolean)`.
    // XPath: not(\"false\" instance of xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_12() {
    //  A test whose essence is: `1.1 instance of xs:decimal`.
    // XPath: 1.1 instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_13() {
    //  A test whose essence is: `not(1.1 instance of xs:integer)`.
    // XPath: not(1.1 instance of xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_14() {
    //  A test whose essence is: `xs:anyURI('http://www.example.com/') instance of xs:a
    // XPath: xs:anyURI(\"http://www.example.com/\") instance of xs:anyURI
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_15() {
    //  A test whose essence is: `not(xs:anyURI('http://www.example.com/') instance of
    // XPath: not(xs:anyURI(\"http://www.example.com/\") instance of xs:st
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_16() {
    //  A test whose essence is: `not('a string' instance of xs:untypedAtomic)`.
    // XPath: not(\"a string\" instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_17() {
    //  A test whose essence is: `(1, 2, 3, 4, 5) instance of xs:integer+`.
    // XPath: (1, 2, 3, 4, 5) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_18() {
    //  A test whose essence is: `not((1, 2, 3, 4, 5) instance of xs:integer)`.
    // XPath: not((1, 2, 3, 4, 5) instance of xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_19() {
    //  A test whose essence is: `not((1, 2, 3, 4, 5) instance of xs:integer?)`.
    // XPath: not((1, 2, 3, 4, 5) instance of xs:integer?)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_20() {
    //  A test whose essence is: `(1, 2, 3, 4, 5) instance of xs:integer*`.
    // XPath: (1, 2, 3, 4, 5) instance of xs:integer*
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_21() {
    //  A test whose essence is: `(1, 2, 3, 4, 5) instance of item()+`.
    // XPath: (1, 2, 3, 4, 5) instance of item()+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_22() {
    //  A test whose essence is: `not((1, 2, 'a string', 4, 5) instance of xs:integer*)
    // XPath: not((1, 2, \"a string\", 4, 5) instance of xs:integer*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_23() {
    //  A test whose essence is: `not((1, 2, 1.1, 4, 5) instance of xs:integer*)`.
    // XPath: not((1, 2, 1.1, 4, 5) instance of xs:integer*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_24() {
    //  A test whose essence is: `not((1, 2, 'a string', 4, 5) instance of xs:string*)`
    // XPath: not((1, 2, \"a string\", 4, 5) instance of xs:string*)
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_25() {
    //  A test whose essence is: `(1, 2, 'a string', 4, 5) instance of xs:anyAtomicType
    // XPath: (1, 2, \"a string\", 4, 5) instance of xs:anyAtomicType*
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}

#[test]
fn k_seqexprinstanceof_26() {
    //  A test whose essence is: `(1, 2, count('one'), 4, 5) instance of xs:integer+`.
    // XPath: (1, 2, count(\"one\"), 4, 5) instance of xs:integer+
    // Expected: true
    // Calls stub function - will fail until prod:InstanceofExpr is implemented
    let _ = stub_prodinstanceofexpr();
}
