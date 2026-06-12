//! Test chunk 0 for prod:GroupByClause
//! Contains 3 tests

use dep::xpath::stub_prodgroupbyclause;

#[test]
fn group_016() {
    // In the first grouping spec, the grouping variable does not reference the generat
    // XPath: count(          for $y in 1 to 10          group by $y := $y
    // Expected: 2
    // Calls stub function - will fail until prod:GroupByClause is implemented
    let _ = stub_prodgroupbyclause();
}

#[test]
fn group_017() {
    // Use case-blind collation, if supported
    // XPath: count(          for $y in (\"ax\", \"bx\", \"cx\", \"Ay\", \
    // Expected: 3
    // Calls stub function - will fail until prod:GroupByClause is implemented
    let _ = stub_prodgroupbyclause();
}

#[test]
fn group_018() {
    // Use case-blind collation, if supported
    // XPath: count(          for $y in (\"ax\", \"bx\", \"cx\", \"Ay\", \
    // Expected: 3
    // Calls stub function - will fail until prod:GroupByClause is implemented
    let _ = stub_prodgroupbyclause();
}
