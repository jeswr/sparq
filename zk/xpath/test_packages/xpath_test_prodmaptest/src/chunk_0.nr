//! Test chunk 0 for prod:MapTest
//! Contains 40 tests

use dep::xpath::stub_prodmaptest;

#[test]
fn maptest_001() {
    // instance of generic map type
    // XPath: map{} instance of map(*)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_002() {
    // instance of generic map type
    // XPath: map{1:\"London\"} instance of map(*)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_003() {
    // instance of generic map type
    // XPath: map{1:\"London\", \"London\":1} instance of map(*)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_004() {
    // instance of specific map type
    // XPath: map{} instance of map(xs:integer, xs:string)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_005() {
    // instance of generic map type
    // XPath: map{1:\"London\"} instance of map(xs:integer, xs:string)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_006() {
    // instance of generic map type
    // XPath: map{1:\"London\", \"London\":1} instance of map(xs:integer,
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_009() {
    // instance of map type after remove
    // XPath: map:remove(map{1:\"London\", \"London\":1}, \"London\") inst
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_010() {
    // instance of map type after addition
    // XPath: map:put(map{1:\"London\", 2:\"Paris\"}, 3, current-date()) i
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_011() {
    // instance of a less specific map type
    // XPath: map:put(map{1:\"London\", 2:\"Paris\"}, 3, \"Berlin\") insta
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_012() {
    // map test requiring empty sequence
    // XPath: map{\"London\":(), \"Paris\":(), \"Berlin\":()} instance of
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_013() {
    // map test requiring empty sequence
    // XPath: map{\"London\":(), \"Paris\":(), \"Berlin\":(), \"Rome\":5}
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_014() {
    // map test with cardinality constraint
    // XPath: map{\"London\":1, \"Paris\":2, \"Berlin\":3, \"Rome\":()} in
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_015() {
    // map test with cardinality constraint
    // XPath: map{\"London\":1, \"Paris\":2, \"Berlin\":3, \"Rome\":()} in
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_017() {
    // sequence of maps
    // XPath: map{\"London\":1, \"Paris\":2, \"Berlin\":3, \"Rome\":5} ins
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_018() {
    // sequence of maps
    // XPath: (map{\"London\":1, \"Paris\":2, \"Berlin\":3, \"Rome\":5}, m
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_019() {
    // sequence of maps
    // XPath: (map{\"London\":1, \"Paris\":2, \"Berlin\":3, \"Rome\":5}, m
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_040() {
    // map type subsumption
    // XPath: function($m as map(xs:integer, xs:string)) as xs:integer {ma
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_041() {
    // map type subsumption
    // XPath: function($m as map(xs:integer, xs:string)) as xs:integer {ma
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_042() {
    // map type subsumption
    // XPath: function($m as map(*)) as xs:integer {map:size($m)}
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_043() {
    // map type subsumption
    // XPath: function($m as map(xs:integer, xs:string)) as xs:integer {ma
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_044() {
    // map type subsumption
    // XPath: function($m as map(xs:integer, xs:string)) as xs:integer {ma
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_045() {
    // map type subsumption
    // XPath: function($m as map(xs:integer, xs:string)) as xs:integer {ma
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_046() {
    // empty map type
    // XPath: map{} instance of map(xs:integer, map(xs:integer, xs:string)
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_047() {
    // empty map type (assumes dynamic type checking)
    // XPath: let $f := function() as map(xs:integer, xs:string) {map{}},
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_048() {
    // map type subsumption
    // XPath: let $f := function($m as map(xs:integer, xs:string)) as xs:i
    // Expected: 0
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_049() {
    // map type subsumption
    // XPath: declare function local:f ($m as map(xs:integer, xs:string))
    // Expected: 0
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_050() {
    // map type subsumption
    // XPath: function($m as map(*)) as xs:integer {map:size($m)}
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_051() {
    // map type subsumption
    // XPath: function($m as map(xs:decimal, xs:string+)) as xs:integer {m
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_052() {
    // map type subsumption
    // XPath: function($m as function(*)) as xs:integer {function-arity($m
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_053() {
    // map type subsumption
    // XPath: function($m as function(*)) as xs:integer {function-arity($m
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_054() {
    // map type subsumption
    // XPath: function($m as function(xs:anyAtomicType) as item()*) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_055() {
    // Dynamic function call with operand a map
    // XPath: map{1:2, 2:4, 3:6}(2)
    // Expected: 4
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_059() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map { 1:'A','x':'B' } instance of function(xs:integer) as xs
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_060() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map { 1:'A','x':'B' } instance of function(xs:integer) as it
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_061() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map { 1:'A','x':'B' } instance of function(xs:integer) as xs
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_062() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map { 1:'A','x':'B' } instance of function(xs:integer) as it
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_063() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map {} instance of function(xs:integer) as empty-sequence()
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_064() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map {12: ()} instance of function(xs:decimal) as empty-seque
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_065() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map {12: ()} instance of function(xs:decimal) as xs:string*
    // Expected: true
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}

#[test]
fn maptest_066() {
    // Map as a function, invoking transitivity (see https://github.com/w3c/qt3tests/is
    // XPath: map {12: \"z\"} instance of function(xs:decimal) as xs:strin
    // Expected: false
    // Calls stub function - will fail until prod:MapTest is implemented
    let _ = stub_prodmaptest();
}
