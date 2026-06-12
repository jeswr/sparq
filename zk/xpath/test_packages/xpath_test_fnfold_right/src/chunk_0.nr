//! Test chunk 0 for fn:fold-right
//! Contains 15 tests

use dep::xpath::stub_fnfold_right;

#[test]
fn fold_right_001() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right(1 to 5, 0,function($a, $b) { $a + $b })
    // Expected: 15
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_002() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right((2,3,5,7), 1,function($a, $b) { $a * $b })
    // Expected: 210
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_003() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right((true(), false(), false()), false(),function($a,
    // Expected: true
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_004() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right((true(), false(), false()), false(),function($a,
    // Expected: false
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_006() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right(1 to 5, \"\", fn:concat(?, \".\", ?))
    // Expected: '1.2.3.4.5.'
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_007() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right(1 to 5, \"$zero\", fn:concat(\"$f(\", ?, \", \",
    // Expected: $f(1, $f(2, $f(3, $f(4, $f(5, $zero)))))
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_013() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right(1 to 1000000, 0, function($a, $b){ $b + 1 })
    // Expected: 1000000
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_017() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right((1 to 13), 0, concat(\"(\", ?, \"+\", ?, \")\"))
    // Expected: (1+(2+(3+(4+(5+(6+(7+(8+(9+(10+(11+(12+(
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_018() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right((\"Hello\", \"World\"), 0, function($a, $b){ stri
    // Expected: 10
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_019() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right( 1 to 10, (), function($a, $b){ if(empty($b)) the
    // Expected: 10
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_020() {
    //              Higher Order Functions              fold-right function
    // XPath: fold-right(1 to 10, (), function($a, $b){ if(empty($b)) then
    // Expected: 1
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_101() {
    // As fold-right-001: test equivalent function given in spec
    // XPath: declare function local:fold-right(           $seq as item()*
    // Expected: 15
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_102() {
    // As fold-right-002: test equivalent function given in spec
    // XPath: declare function local:fold-right(           $seq as item()*
    // Expected: 210
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_103() {
    // As fold-right-003: test equivalent function given in spec
    // XPath: declare function local:fold-right(           $seq as item()*
    // Expected: true
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}

#[test]
fn fold_right_104() {
    // As fold-right-004: test equivalent function given in spec
    // XPath: declare function local:fold-right(           $seq as item()*
    // Expected: false
    // Calls stub function - will fail until fn:fold-right is implemented
    let _ = stub_fnfold_right();
}
