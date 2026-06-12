//! Test chunk 0 for prod:ConstructionDecl
//! Contains 22 tests

use dep::xpath::stub_prodconstructiondecl;

#[test]
fn constprolog_2() {
    //  Evaluation of a prolog with construction declaration set to 'strip' and used wi
    // XPath: declare construction strip;          let $anElement := <anEl
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_3() {
    //  Evaluation of a prolog with construction declaration set to 'strip' and used wi
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_4() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' and used
    // XPath: declare construction preserve;          let $anElement := <a
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_5() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' and used
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_6() {
    //  Evaluation of a prolog with construction declaration set to 'strip' and used wi
    // XPath: declare construction strip;          let $anElement := <anEl
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_7() {
    //  Evaluation of a prolog with construction declaration set to 'strip' and used wi
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_8() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for a direc
    // XPath: declare construction strip;          let $anElement := <some
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_9() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for a compu
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_10() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for a di
    // XPath: declare construction preserve;          let $anElement := <s
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_11() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for a co
    // XPath: declare construction preserve;          let $anElement := el
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_12() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for a direc
    // XPath: declare construction strip;          fn:not(<someElement>som
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_13() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for a co
    // XPath: declare construction preserve;          fn:not(element someE
    // Expected: false
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_14() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for direct
    // XPath: declare construction strip;          let $var := <anElement>
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_15() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for compute
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_16() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for dire
    // XPath: declare construction preserve;          let $anElement := <s
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_17() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for comp
    // XPath: declare construction preserve;          let $anElement := el
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_18() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for direct
    // XPath: declare construction strip;          let $var := <anElement>
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_19() {
    //  Evaluation of a prolog with construction declaration set to 'strip' for compute
    // XPath: declare construction strip;          let $anElement := eleme
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_20() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for dire
    // XPath: declare construction preserve;          let $anElement := <s
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn constprolog_21() {
    //  Evaluation of a prolog with construction declaration set to 'preserve' for comp
    // XPath: declare construction preserve;          let $anElement := el
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn k_constructionprolog_1() {
    //  Test that 'declare construction' with the preserve keyword is parsed properly.
    // XPath: (::)declare(::)construction(::)preserve(::);(::)1(::)eq(::)1
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}

#[test]
fn k_constructionprolog_3() {
    //  Test that 'declare construction' with the strip keyword is parsed properly.
    // XPath: declare(::)construction(::)strip;(::)1(::)eq(::)1
    // Expected: true
    // Calls stub function - will fail until prod:ConstructionDecl is implemented
    let _ = stub_prodconstructiondecl();
}
