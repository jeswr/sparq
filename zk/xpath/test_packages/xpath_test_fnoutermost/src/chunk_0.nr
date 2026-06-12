//! Test chunk 0 for fn:outermost
//! Contains 22 tests

use dep::xpath::stub_fnoutermost;

#[test]
fn fn_outermost_005() {
    // Attempts to reference the 'outermost' function with arity one.
    // XPath: fn:exists( fn:outermost#1 )
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_013() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal(fn:outermost( / ), / )
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_016() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: deep-equal(fn:outermost( //*/@* ), //*/@*)
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_022() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //*/namespace::* ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_025() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //processing-instruction() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_028() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //comment() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_031() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //text() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_033() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:outermost( //* ) ! local-name(.)
    // Expected: ('root')
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_034() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //* ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_036() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( //node() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_038() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/node() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_040() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/descendant::node() ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_042() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/level[1]/level[1]/ancesto
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_044() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/level[1]/level[last()]/pr
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_046() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/level[1]/level[last()]/pr
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_048() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/level[1]/following-siblin
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_050() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/level[1]/level[1]/followi
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_052() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( /root/node()/.. ),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_054() {
    // Evaluates the 'outermost' function with the argument set as follows: $nodes of t
    // XPath: fn:deep-equal( fn:outermost( ($doc1//node(), $doc2//node())
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_055() {
    // Evaluates the 'outermost' function with the input sequence not in document order
    // XPath: let $in := for $x in //* order by local-name($x) return $x
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_056() {
    // Evaluates the 'outermost' function with the input sequence containing duplicates
    // XPath: let $in := for $x in //* order by local-name($x) return $x
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}

#[test]
fn fn_outermost_059() {
    // Check that outermost returns correct descendants that are not siblings
    // XPath: deep-equal(          outermost(descendant::comment()),
    // Expected: true
    // Calls stub function - will fail until fn:outermost is implemented
    let _ = stub_fnoutermost();
}
