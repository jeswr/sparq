//! Test chunk 0 for fn:innermost
//! Contains 20 tests

use dep::xpath::stub_fninnermost;

#[test]
fn fn_innermost_005() {
    // Attempts to reference the 'innermost' function with arity one.
    // XPath: fn:exists( fn:innermost#1 )
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_013() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal(fn:innermost( / ), / )
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_016() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: deep-equal(fn:innermost( //*/@* ), //*/@*)
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_022() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //*/namespace::* ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_025() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //processing-instruction() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_028() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //comment() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_031() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //text() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_034() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //* ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_036() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( //node() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_038() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/node() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_040() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/descendant::node() ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_042() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/level[1]/level[1]/ancesto
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_044() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/level[1]/level[last()]/pr
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_046() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/level[1]/level[last()]/pr
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_048() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/level[1]/following-siblin
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_050() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/level[1]/level[1]/followi
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_052() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( /root/node()/.. ),
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_054() {
    // Evaluates the 'innermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:innermost( ($doc1//node(), $doc2//node())
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_055() {
    // Evaluates the 'innermost' function with the input sequence not in document order
    // XPath: let $in := for $x in //* order by local-name($x) return $x
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}

#[test]
fn fn_innermost_056() {
    // Evaluates the 'innermost' function with the input sequence containing duplicates
    // XPath: let $in := for $x in //* order by local-name($x) return $x
    // Expected: true
    // Calls stub function - will fail until fn:innermost is implemented
    let _ = stub_fninnermost();
}
