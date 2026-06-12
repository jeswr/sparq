//! Test chunk 0 for fn:collection
//! Contains 7 tests

use dep::xpath::stub_fncollection;

#[test]
fn fn_collection_4() {
    //  Count the number of nodes in a named collection.
    // XPath: count(fn:collection($collection-uri))
    // Expected: 2
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_4d() {
    //  Count the number of nodes in the default collection.
    // XPath: count(fn:collection())
    // Expected: 2
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_5() {
    //  Count the number of nodes in the collection.
    // XPath: count(fn:collection($collection-uri))
    // Expected: 3
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_5d() {
    //  Count the number of nodes in the collection.
    // XPath: count(fn:collection())
    // Expected: 3
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_9() {
    //  Count the title elements in each document.
    // XPath: for $d in fn:collection($collection-uri) order by count($d//
    // Expected: 3 4 5
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_10() {
    //  Test that two uses of fn:collection are stable.
    // XPath: let $c1 := fn:collection($collection-uri) let $c2 := fn:coll
    // Expected: true true true
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}

#[test]
fn fn_collection_10d() {
    //  Test that two uses of fn:collection are stable.
    // XPath: let $c1 := fn:collection() let $c2 := fn:collection() for $c
    // Expected: true true true
    // Calls stub function - will fail until fn:collection is implemented
    let _ = stub_fncollection();
}
