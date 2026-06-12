//! Test chunk 0 for prod:ArrowPostfix
//! Contains 13 tests

use dep::xpath::stub_prodarrowpostfix;

#[test]
fn arrowpostfix_002() {
    // Arrow operator used with contains function
    // XPath: \"this, that and the other\"=>contains(\"the\")
    // Expected: true
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_016() {
    // Arrow operator; precedence test
    // XPath: let $x:= 4 return $x<=3.2=>ceiling()
    // Expected: true
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_017() {
    // Arrow operator; precedence test
    // XPath: let $x:= 4 return ($x<=3.2)=>boolean()
    // Expected: false
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_026() {
    // Precedence of '=>' is less than '//'
    // XPath: //hours => count()
    // Expected: 16
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_102() {
    // Bug 29346
    // XPath: 1 => Q{http://www.w3.org/2005/xpath-functions}sum()
    // Expected: 1
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_103() {
    // Bug 29346
    // XPath: (1, 2, 3, 4) => Q{http://www.w3.org/2005/xpath-functions}sum
    // Expected: 10
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_104() {
    // Bug 29346
    // XPath: (1, 2, 3, 4) => (Q{http://www.w3.org/2005/xpath-functions}su
    // Expected: 10
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_105() {
    // Bug 29346
    // XPath: (1, 2, 3, 4) => (Q{http://www.w3.org/2005/xpath-functions}su
    // Expected: 10
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_106() {
    // Bug 29346
    // XPath: () => (Q{http://www.w3.org/2005/xpath-functions}sum#2(?, 'fo
    // Expected: "foo"
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_107() {
    //              I added this test with XQueryX in mind.  The salient feature is par
    // XPath: 1 => ([2,3],[4,5][6])()
    // Expected: 2
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_108() {
    // ArgumentList may include a placeholder
    // XPath: let $usd := \"$\" => concat(?) return $usd(3)
    // Expected: "$3"
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_109() {
    // A map may be used as a function
    // XPath: let $days := map{\"Mo\": \"Monday\", \"Tu\": \"Tuesday\"} re
    // Expected: "Tuesday"
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}

#[test]
fn arrowpostfix_110() {
    // A map may be used as a function
    // XPath: \"Tu\" => (map{\"Mo\": \"Monday\", \"Tu\": \"Tuesday\"})()
    // Expected: "Tuesday"
    // Calls stub function - will fail until prod:ArrowPostfix is implemented
    let _ = stub_prodarrowpostfix();
}
