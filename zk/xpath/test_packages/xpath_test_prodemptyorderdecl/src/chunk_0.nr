//! Test chunk 0 for prod:EmptyOrderDecl
//! Contains 16 tests

use dep::xpath::stub_prodemptyorderdecl;

#[test]
fn emptyorderdecl_4() {
    //  Evaluation of a prolog that specifies 'empty greatest' with a set of numbers (o
    // XPath: declare default order empty greatest;  for $i in (1,4,0 div
    // Expected: 1 4 7 NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_5() {
    //  Evaluation of a prolog that specifies 'empty greatest' with a set of numbers (t
    // XPath: declare default order empty greatest;  for $i in (1,4,0 div
    // Expected: 1 4 7 NaN NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_8() {
    //  Evaluation of a prolog that specifies 'empty greatest' with a set of numbers (o
    // XPath: declare default order empty greatest;  for $i in (1,4,0 div
    // Expected: 1 4 NaN 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_9() {
    //  Evaluation of a prolog that specifies 'empty least' with a set of numbers (one
    // XPath: declare default order empty least;  for $i in (1,4,0 div 0E0
    // Expected: 1 4 NaN 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_12() {
    //  Evaluation of empty order declaration set to 'empty greatest' and a set of numb
    // XPath: declare default order empty greatest;  for $i in (1,4,0 div
    // Expected: NaN 7 4 1
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_13() {
    //  Evaluation of empty order declaration set to 'empty greatest' and a set of numb
    // XPath: declare default order empty greatest;  for $i in (1,4,0 div
    // Expected: NaN NaN 7 4 1
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_16() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,0 div 0E0
    // Expected: NaN 1 4 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_17() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,0 div 0E0
    // Expected: NaN NaN 1 4 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_20() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,0 div 0E0
    // Expected: 7 4 1 NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_21() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,0 div 0E0
    // Expected: 7 4 1 NaN NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_23() {
    //  Evaluation of empty order declaration set to 'empty greatest' and a set of numb
    // XPath: declare default order empty greatest;  for $i in (1,4,3,0 di
    // Expected: NaN 1 3 4 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_25() {
    //  Evaluation of empty order declaration set to 'empty greatest' and a set of numb
    // XPath: declare default order empty greatest;  for $i in (1,4,3,0 di
    // Expected: 1 3 4 7 NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_27() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,3,0 div 0
    // Expected: 1 3 4 7 NaN
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn emptyorderdecl_29() {
    //  Evaluation of empty order declaration set to 'empty least' and a set of numbers
    // XPath: declare default order empty least;  for $i in (1,4,3,0 div 0
    // Expected: NaN 1 3 4 7
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn k_emptyorderprolog_1() {
    //  A simple 'declare default order empty' declaration, specifying 'greatest'.
    // XPath: declare(::)default order empty(::)greatest; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}

#[test]
fn k_emptyorderprolog_2() {
    //  A simple 'declare default order empty' declaration, specifying 'least'.
    // XPath: declare(::)default order empty(::)least; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:EmptyOrderDecl is implemented
    let _ = stub_prodemptyorderdecl();
}
