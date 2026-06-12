//! Test chunk 0 for prod:StepExpr
//! Contains 12 tests

use dep::xpath::stub_prodstepexpr;

#[test]
fn k2_steps_5() {
    //  A direct element constructor as step, followed by a name test.
    // XPath: empty(<a/>/a)
    // Expected: true
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn k2_steps_7() {
    //  Apply fn:count to an atomic step.
    // XPath: count((<a/>, <!--comment-->)/3)
    // Expected: 2
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn k2_steps_13() {
    //  Ensure the focus is available through a for-expression.
    // XPath: empty(<e/>/(for $i in e return $i))
    // Expected: true
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_1a() {
    //  Verify xgc:leading-lone-slash implementation This expression is a valid express
    // XPath: declare variable $var := document {<a>123</a>}; $var[(/)*5]
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_2a() {
    //  Verify xgc:leading-lone-slash implementation This expression is a valid express
    // XPath: declare variable $var := document {<a>123</a>}; $var[(/)<a]
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_3a() {
    //  Verify xgc:leading-lone-slash implementation This expression is a valid express
    // XPath: declare variable $var := document {<a>123</a>}; $var[(/)<5]
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_5a() {
    //  Verify xgc:leading-lone-slash implementation This expression is a valid express
    // XPath: declare variable $var := document {<a>123</a>}; $var[(/)<a d
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_7a() {
    //  Verify xgc:leading-lone-slash implementation This expression is a valid express
    // XPath: declare variable $var := document {<a>123</a>};          dec
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_8a() {
    //  Verify xgc:leading-lone-slash implementation        This expression is a valid
    // XPath: declare variable $var := document {<a>123</a>};          $va
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_14() {
    //  Verify xgc:leading-lone-slash implementation        This expression is a valid
    // XPath: declare variable $var := document {<a>123</a>};          $va
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_15() {
    //  Verify xgc:leading-lone-slash implementation        This expression is a valid
    // XPath: declare variable $var := document {<a>123</a>};          $va
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}

#[test]
fn steps_leading_lone_slash_17() {
    //  Verify xgc:leading-lone-slash implementation        This expression is a valid
    // XPath: declare variable $var := document {<a>123</a>};          dec
    // Expected:
    // Calls stub function - will fail until prod:StepExpr is implemented
    let _ = stub_prodstepexpr();
}
