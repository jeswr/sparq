//! Test chunk 0 for prod:Comment
//! Contains 19 tests

use dep::xpath::stub_prodcomment;

#[test]
fn xquerycomment009() {
    //  Comments inside a conditional expression test (: yada (: neato yada yada yada
    // XPath: (: Name: XQueryComment009 :) (: Description: Comments inside
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn xquerycomment010() {
    //  Comments inside a conditional expression comment this is the then case this is
    // XPath: (: Name: XQueryComment010 :) (: Description: Comments inside
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn xquerycomment011() {
    //  Comments inside a conditional expression test t2 t3
    // XPath: (: Name: XQueryComment011 :) (: Description: Comments inside
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn xquerycomment013() {
    //  Comments inside a sequence expression comment
    // XPath: (: Name: XQueryComment013 :) (: Description: Comments inside
    // Expected: 1 2 3 4
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn xquerycomment014() {
    //  Comments inside a cast expression type comment
    // XPath: (: Name: XQueryComment014 :) (: Description: Comments inside
    // Expected: 10
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_1() {
    //  A test whose essence is: `(3(: comment inbetween comment inbetween
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_2() {
    //  A test whose essence is: `1 (: a (: nested a (: nested
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_3() {
    //  A test whose essence is: `1 (: comment (: inside comment (: inside
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_4() {
    //  Comparison of two string literals, whose content reminds of comments.
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_5() {
    //  A test whose essence is: `5 instance (: strange place for a comment strange pla
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_6() {
    //  A test whose essence is: `1 (: simple comment simple comment
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_7() {
    //  A test whose essence is: `1 (: comment (: inside comment (: inside
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_9() {
    //  An empty comment inbetween the 'eq' operator and a number literal.
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_10() {
    //  An empty comment at the very beginning of an expression.
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_11() {
    //  An empty comment after a function's paranteses.
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_12() {
    //  A for loop with a comment inbetween. set up loop
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_17() {
    //  Colons and paranteses appearing freely in comment content. ((( : )) ))ladl: :()
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_18() {
    //  Three comments appearing serially inside a comment. (:one comment another comme
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}

#[test]
fn k_xquerycomment_19() {
    //  A test stressing many nested comments. (:(:(:(:(:(:(:
    // XPath: (:*******************************************************:)
    // Expected: true
    // Calls stub function - will fail until prod:Comment is implemented
    let _ = stub_prodcomment();
}
