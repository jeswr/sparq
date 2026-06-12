//! Test chunk 0 for fn:load-xquery-module
//! Contains 12 tests

use dep::xpath::stub_fnload_xquery_module;

#[test]
fn fn_load_xquery_module_015() {
    // Loads a module and sets the value for an external var in the module.
    // XPath: let $qn-var1 := QName(\"http://www.w3.org/fots/fn/load-xquer
    // Expected: some value
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_026() {
    // Loads a module, supplies value for external variable declared in transitively-im
    // XPath: let $qn-var1 := QName(\"http://www.w3.org/fots/fn/load-xquer
    // Expected: that
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_027() {
    // Loads a module, supplies value for external variable declared in indirectly impo
    // XPath: let $qn-var1 := QName(\"http://www.w3.org/fots/fn/load-xquer
    // Expected: that
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_028() {
    // Loads a module containing a function using variables from an imported module.
    // XPath: let $module-ns := \"http://www.w3.org/fots/fn/load-xquery-mo
    // Expected: this and that
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_029() {
    // Error loading a module which attempts to import itself directly
    // XPath: let $module := fn:load-xquery-module(\"http://www.w3.org/fot
    // Expected: this
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_030() {
    // Loads a module which imports itself circularly.
    // XPath: let $module := fn:load-xquery-module(\"http://www.w3.org/Tes
    // Expected: ok
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_037() {
    // Loads a module which loads itself.
    // XPath: let $module-ns := \"http://www.w3.org/fots/fn/load-xquery-mo
    // Expected: end
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_040() {
    // Loads a module that defines a decimal format, with a function that calls format-
    // XPath: let $module-ns := \"http://www.w3.org/fots/fn/load-xquery-mo
    // Expected: 1.234.567,76
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_041() {
    // Loads a module containing a function which uses 'declare default order empty lea
    // XPath: let $module-ns := \"http://www.w3.org/fots/fn/load-xquery-mo
    // Expected: 0 0 1 1 1
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_042() {
    // Loads a module containing a function which uses 'declare default order empty gre
    // XPath: let $module-ns := \"http://www.w3.org/fots/fn/load-xquery-mo
    // Expected: 1 1 1 0 0
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_043() {
    // Loads a module containing a function which uses 'declare default order empty lea
    // XPath: declare default order empty greatest;       declare namespac
    // Expected: 0 0 1 1 1 1 1 1 0 0
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}

#[test]
fn fn_load_xquery_module_044() {
    // Loads a module containing a function which uses 'declare default order empty gre
    // XPath: declare default order empty least;       declare namespace t
    // Expected: 1 1 1 0 0 0 0 1 1 1
    // Calls stub function - will fail until fn:load-xquery-module is implemented
    let _ = stub_fnload_xquery_module();
}
