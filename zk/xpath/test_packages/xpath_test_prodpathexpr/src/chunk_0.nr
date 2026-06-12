//! Test chunk 0 for prod:PathExpr
//! Contains 13 tests

use dep::xpath::stub_prodpathexpr;

#[test]
fn pathexpr_1() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[5 * /])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_2() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[(/) * 5])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_4() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[(/) < 5])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_5p() {
    //  In XPath, /< doesn't trigger leading-lone-slash syntax contraint
    // XPath: fn:count(.[/ < 5])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_6() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[5</])
    // Expected: 1
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_7p() {
    //  In XPath, /< doesn't trigger leading-lone-slash syntax contraint
    // XPath: fn:count(.[/ < a])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_8p() {
    //  In XPath, /< doesn't trigger leading-lone-slash syntax contraint
    // XPath: fn:count(.[/ < /b])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_9p() {
    //  In XPath, /< doesn't trigger leading-lone-slash syntax contraint
    // XPath: fn:count(.[/<a div 3])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_14() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[/<a/>])
    // Expected: 1
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_15() {
    //  Leading lone slash syntax contraints
    // XPath: fn:count(.[/-5])
    // Expected: 0
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_16() {
    //  Leading lone slash syntax contraints
    // XPath: let $a := . return fn:count(.[/=$a])
    // Expected: 1
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_20() {
    //  Leading lone slash syntax contraints - function call is allowed
    // XPath: /count(bid)
    // Expected: 1
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}

#[test]
fn pathexpr_24() {
    //  Leading lone slash syntax contraints - numeric literal is allowed
    // XPath: /42
    // Expected: 42
    // Calls stub function - will fail until prod:PathExpr is implemented
    let _ = stub_prodpathexpr();
}
