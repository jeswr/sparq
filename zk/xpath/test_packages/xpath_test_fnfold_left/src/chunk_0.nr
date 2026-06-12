//! Test chunk 0 for fn:fold-left
//! Contains 16 tests

use dep::xpath::stub_fnfold_left;

#[test]
fn fold_left_001() {
    // Higher Order Functions fold-left function
    // XPath: fold-left(1 to 5, 0, function($a, $b) { $a + $b })
    // Expected: 15
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_002() {
    // Higher Order Functions fold-left function
    // XPath: fold-left((2,3,5,7), 1, function($a, $b) { $a * $b })
    // Expected: 210
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_003() {
    // Higher Order Functions fold-left function
    // XPath: fold-left((true(), false(), false()), false(), function($a,
    // Expected: true
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_004() {
    // Higher Order Functions fold-left function
    // XPath: fold-left((true(), false(), false()), false(), function($a,
    // Expected: false
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_006() {
    // Higher Order Functions fold-left function
    // XPath: fold-left(1 to 5, \"\", fn:concat(?, \".\", ?))
    // Expected: '.1.2.3.4.5'
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_007() {
    // Higher Order Functions fold-left function
    // XPath: fold-left(1 to 5, \"$zero\", fn:concat(\"$f(\", ?, \", \", ?
    // Expected: $f($f($f($f($f($zero, 1), 2), 3), 4), 5)
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_009() {
    // fold-left-009 author Michael Kay, Saxonica implement eg:distinct-nodes-stable()
    // XPath: declare function local:distinct-nodes-stable($seq as node()*
    // Expected: a b d c
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_016() {
    // Higher Order Functions fold-left function. Dynamic node selection.
    // XPath: let $html := <html>   <body>  <div id=\"main\">    <p class=
    // Expected: Hello World!
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_017() {
    // Higher Order Functions fold-left function. Display fold-left structural transfor
    // XPath: fold-left(1 to 13, 0, concat(\"(\", ?, \"+\", ?, \")\"))
    // Expected: (((((((((((((0+1)+2)+3)+4)+5)+6)+7)+8)+9
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_019() {
    // Higher Order Functions fold-left function. Returns the average of the sequence v
    // XPath: fold-left((13, 14, 9, 6), (), function($a, $b){ if(empty($a)
    // Expected: 8.625
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_020() {
    // Higher Order Functions fold-left function. Count the number of items in a sequen
    // XPath: fold-left(1 to 1000000, 0, function($a, $b){ $a + 1})
    // Expected: 1000000
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_021() {
    // fold-left used to apply a finite-state machine, after a problem raised by Roger
    // XPath: declare variable $in := (<doc><A/><B/>{(1 to 10000)!<C/>}<B/
    // Expected: false
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_101() {
    // As fold-left-001: test equivalent function given in spec
    // XPath: declare function local:fold-left(           $seq as item()*,
    // Expected: 15
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_102() {
    // As fold-left-002: test equivalent function given in spec
    // XPath: declare function local:fold-left(           $seq as item()*,
    // Expected: 210
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_103() {
    // As fold-left-003: test equivalent function given in spec
    // XPath: declare function local:fold-left(           $seq as item()*,
    // Expected: true
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}

#[test]
fn fold_left_104() {
    // As fold-left-004: test equivalent function given in spec
    // XPath: declare function local:fold-left(           $seq as item()*,
    // Expected: false
    // Calls stub function - will fail until fn:fold-left is implemented
    let _ = stub_fnfold_left();
}
