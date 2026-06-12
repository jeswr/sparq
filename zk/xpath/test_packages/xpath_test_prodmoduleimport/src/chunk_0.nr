//! Test chunk 0 for prod:ModuleImport
//! Contains 18 tests

use dep::xpath::stub_prodmoduleimport;

#[test]
fn modules_circular() {
    //  Importing circular modules
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: ok
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_4() {
    //  Evaluates actual usage of a variable declared in an imported module.
    // XPath: import module namespace defs=\"http://www.w3.org/TestModules
    // Expected: 2
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_5() {
    //  Module that uses a variable from an imported module and modifies a variable.
    // XPath: import module namespace defs=\"http://www.w3.org/TestModules
    // Expected: 4
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_6() {
    //  Evaluation of usage of variable from imported module and usage of variable with
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 4
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_7() {
    //  Evaluation of of an importing module that uses both a variable and a function f
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 1ok
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_8() {
    //  Evaluation of usage of same function name from imported/importing module, but d
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: okok
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_9() {
    //  Evaluation of import module feature that applies the upper case function to a v
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: OK
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_10() {
    //  Evaluation of import module feature that applies the lower case function to a v
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: ok
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_11() {
    //  Evaluation of import module feature that applies the string-length function to
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 2
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_12() {
    //  Evaluation of import module feature that performs and addition operation to a v
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 2
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_13() {
    //  Evaluation of import module feature that performs a subtraction operation to a
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 0
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_14() {
    //  Evaluation of import module feature that applies the xs:integer function to a v
    // XPath: import module namespace test1=\"http://www.w3.org/TestModule
    // Expected: 1
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_18() {
    //  Evaluation of module import with two imports, one of which import the other and
    // XPath: import module namespace mod1=\"http://www.w3.org/TestModules
    // Expected: x y x y
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn modules_30() {
    // Test loading two modules with the same module namespace (see https://github.com/
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/m3
    // Expected: 3
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn errata8_002a() {
    //  Test XQST0093 by importing a module with a circular dependency
    // XPath: import module namespace errata8_2a=\"http://www.w3.org/TestM
    // Expected: 10
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn errata8_003() {
    //  Test lack of XQST0093 by importing a module without a circular dependency
    // XPath: import module namespace errata8_3a=\"http://www.w3.org/TestM
    // Expected: 10
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn cbcl_module_002() {
    //  Test that the correct base URI is assigned to nodes constructed in a library.
    // XPath: import module namespace lib=\"http://www.xqsharp.com/test/ba
    // Expected: http://www.example.org/correct/
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}

#[test]
fn cbcl_module_004() {
    //  Test that the context item can be used in a library
    // XPath: import module namespace context=\"http://www.xqsharp.com/tes
    // Expected: 6
    // Calls stub function - will fail until prod:ModuleImport is implemented
    let _ = stub_prodmoduleimport();
}
