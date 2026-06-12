//! Test chunk 0 for fn:doc
//! Contains 11 tests

use dep::xpath::stub_fndoc;

#[test]
fn fn_doc_5() {
    //  Evaluation of fn:doc as per example 1 from the functions and Ops. for this func
    // XPath: fn:doc($uri) is fn:doc($uri)
    // Expected: true
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_6() {
    //  Evaluation of fn:doc used with different resourses and the 'is' operator.
    // XPath: fn:doc($works) is fn:doc($staff)
    // Expected: false
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_15() {
    //  Evaluation of fn:doc as an argument to the fn:nilled function.
    // XPath: fn:count(fn:nilled(fn:doc($uri)))
    // Expected: 0
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_16() {
    //  Evaluation of fn:doc as an argument to the fn:node-name function. Use fn:count
    // XPath: fn:count(fn:node-name(fn:doc($uri)))
    // Expected: 0
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_18() {
    //  Evaluation of fn:doc used with 'is' operator and the fn:not function.
    // XPath: fn:not(fn:doc($uri) is fn:doc($uri))
    // Expected: false
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_19() {
    //  Evaluation of fn:doc used with 'is' operator and the fn:true function with 'and
    // XPath: (fn:doc($uri) is fn:doc($uri)) and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_20() {
    //  Evaluation of fn:doc used with 'is' operator and the fn:false function with 'an
    // XPath: (fn:doc($uri) is fn:doc($uri)) and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_21() {
    //  Evaluation of fn:doc used with 'is' operator and the fn:true function with 'or'
    // XPath: (fn:doc($uri) is fn:doc($uri)) or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_22() {
    //  Evaluation of fn:doc used with 'is' operator and the fn:false function with 'or
    // XPath: (fn:doc($uri) is fn:doc($uri)) or fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn fn_doc_36() {
    //  Count text nodes in a document containing character references.
    // XPath: count(doc($uri)//text())
    // Expected: 9
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}

#[test]
fn k2_seqdocfunc_1() {
    //  Invoke fn:doc() on the empty sequence.
    // XPath: empty(fn:doc(()))
    // Expected: true
    // Calls stub function - will fail until fn:doc is implemented
    let _ = stub_fndoc();
}
