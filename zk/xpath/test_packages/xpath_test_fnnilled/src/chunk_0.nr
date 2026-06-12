//! Test chunk 0 for fn:nilled
//! Contains 21 tests

use dep::xpath::stub_fnnilled;

#[test]
fn fn_nilled_1() {
    //  Evaluation of nilled function with argument set to empty sequence. Use fn:count
    // XPath: fn:count(fn:nilled(()))
    // Expected: 0
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_2() {
    //  Evaluation of nilled function with argument set to comment node. Use fn:count t
    // XPath: fn:count(fn:nilled(/works[1]/employee[2]/child::text()[last(
    // Expected: 0
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_3() {
    //  Evaluation of nilled function with argument set to an element node.
    // XPath: fn:nilled(/works[1]/employee[2])
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_4() {
    //  Evaluation of nilled function with argument set to an element node.
    // XPath: fn:nilled(<shoe size = \"5\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_5() {
    //  Evaluation of nilled function with argument set to an element node with xsi:nil
    // XPath: fn:nilled(<shoe xsi:nil=\"true\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_6() {
    //  Evaluation of nilled function with argument set to an element node with xsi:nil
    // XPath: fn:nilled(<shoe xsi:nil=\"false\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_7() {
    //  Evaluation of nilled function with argument set to an element node with xsi:nil
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_8() {
    //  Evaluation of nilled function with argument set to an element node with xsi:nil
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:false()}\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_10() {
    //  Evaluation of nilled function as argument to fn:not function. returns true.
    // XPath: fn:not(fn:nilled(<shoe xsi:nil=\"{fn:false()}\"/>))
    // Expected: true
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_11() {
    //  Evaluation of nilled function as argument to fn:not function. returns false.
    // XPath: fn:not(fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>))
    // Expected: true
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_12() {
    //  Evaluation of nilled function used a part of boolean expression ('and' operator
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>) and fn:true()
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_13() {
    //  Evaluation of nilled function used a part of boolean expression ('and' operator
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>) and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_14() {
    //  Evaluation of nilled function used a part of boolean expression ('or' operator
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>) or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_15() {
    //  Evaluation of nilled function used a part of boolean expression ('or' operator
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>) or fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_16() {
    //  Evaluation of nilled function used as argument to fn:string function.
    // XPath: fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>)
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_17() {
    //  Evaluation of nilled function used as argument to xs:boolean function. Returns
    // XPath: xs:boolean(fn:nilled(<shoe xsi:nil=\"{fn:true()}\"/>))
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_18() {
    //  Evaluation of nilled function used as argument to xs:boolean function. Returns
    // XPath: xs:boolean(fn:nilled(<shoe xsi:nil=\"{fn:false()}\"/>))
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_24() {
    //   nilled() applied to element node (Allowed in 3.0).
    // XPath: /*/nilled()
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn fn_nilled_27() {
    //   nilled() applied to untyped element node with xsi:nil=true (Allowed in 3.0).
    // XPath: (<shoe xsi:nil=\"true\"/>)/fn:nilled()
    // Expected: false
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn k_nilledfunc_3() {
    //  A test whose essence is: `empty(nilled( () ))`.
    // XPath: empty(nilled( () ))
    // Expected: true
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}

#[test]
fn cbcl_nilled_007() {
    //  Test fn:nilled on non-element nodes.
    // XPath: empty(nilled( <?foo ?> ))
    // Expected: true
    // Calls stub function - will fail until fn:nilled is implemented
    let _ = stub_fnnilled();
}
