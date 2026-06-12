//! Test chunk 2 for prod:TryCatchExpr
//! Contains 5 tests

use dep::xpath::stub_prodtrycatchexpr;

#[test]
fn try_catch_fn_error_16() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XQST0089\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_17() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XQDY0061\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_18() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:FOTY0014\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_19() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:FORG0001\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_20() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:FONS0004\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}
