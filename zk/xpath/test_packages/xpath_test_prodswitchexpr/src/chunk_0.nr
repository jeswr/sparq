//! Test chunk 0 for prod:SwitchExpr
//! Contains 5 tests

use dep::xpath::stub_prodswitchexpr;

#[test]
fn switch_014() {
    //  switch test, QName value
    // XPath: let $v := xs:QName('a')          return switch($v)
    // Expected: 1
    // Calls stub function - will fail until prod:SwitchExpr is implemented
    let _ = stub_prodswitchexpr();
}

#[test]
fn switch_015() {
    //  switch test, xs:date value
    // XPath: let $v := xs:date('2001-01-01')          return switch($v)
    // Expected: 1
    // Calls stub function - will fail until prod:SwitchExpr is implemented
    let _ = stub_prodswitchexpr();
}

#[test]
fn switch_016() {
    //  switch test, xs:date value one with timezone
    // XPath: let $v := xs:dateTime('2001-01-01T12:00:00')          return
    // Expected: 2
    // Calls stub function - will fail until prod:SwitchExpr is implemented
    let _ = stub_prodswitchexpr();
}

#[test]
fn switch_017() {
    //  switch test, xs:dateTime value one with timezone
    // XPath: let $v := current-dateTime()          return switch($v)
    // Expected: 2
    // Calls stub function - will fail until prod:SwitchExpr is implemented
    let _ = stub_prodswitchexpr();
}

#[test]
fn switch_018() {
    //  switch test, xs:time value both with timezone
    // XPath: let $v := xs:time('00:00:00Z')          return switch($v)
    // Expected: 1
    // Calls stub function - will fail until prod:SwitchExpr is implemented
    let _ = stub_prodswitchexpr();
}
