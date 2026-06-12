//! Test chunk 0 for fn:has-children
//! Contains 22 tests

use dep::xpath::stub_fnhas_children;

#[test]
fn fn_has_children_001() {
    // Attempts to reference the 'has-children' function with arity zero.
    // XPath: fn:exists( fn:has-children#0 )
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_002() {
    // Attempts to reference the 'has-children' function with arity one.
    // XPath: fn:exists( fn:has-children#1 )
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_018() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/@attribute/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_019() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/@attribute )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_020() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/namespace::fots/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_021() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/namespace::fots )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_022() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/comment()/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_023() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/comment() )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_024() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/processing-instruction()/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_025() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/processing-instruction() )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_026() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/text()/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_027() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/text() )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_028() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/empty/fn:has-children()
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_029() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: fn:has-children( /root/empty )
    // Expected: false
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_030() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/fn:has-children()
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_031() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root )
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_032() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/non-empty/fn:has-children()
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_033() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( /root/non-empty )
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_034() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /fn:has-children()
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_035() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: fn:has-children( / )
    // Expected: true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_036() {
    // Evaluates the 'has-children' function with the argument set as follows: context
    // XPath: /root/node()/fn:has-children()
    // Expected: false false false false true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}

#[test]
fn fn_has_children_037() {
    // Evaluates the 'has-children' function with the argument set as follows: $node of
    // XPath: /root/node()/fn:has-children(.)
    // Expected: false false false false true
    // Calls stub function - will fail until fn:has-children is implemented
    let _ = stub_fnhas_children();
}
