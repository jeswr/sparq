//! Test chunk 0 for prod:Lookup
//! Contains 20 tests

use dep::xpath::stub_prodlookup;

#[test]
fn lookup_111() {
    // Binary lookup with no context item
    // XPath: let $d := function($x) {$x?2} return $d([12, 13])
    // Expected: 13
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_115() {
    // Result of Binary lookup is atomized
    // XPath: ([1, [2], [3]], [[2], 2, [4]])?3 = 3
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_150() {
    // Numeric Lookup followed by function call
    // XPath: [floor#1, ceiling#1, round#1, abs#1]?2(1.3)
    // Expected: 2.0
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_151() {
    // Name Lookup followed by function call
    // XPath: map{'f':floor#1, 'c':ceiling#1, 'r':round#1, 'a':abs#1}?c(1.
    // Expected: 2.0
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_152() {
    // Parenthesized Lookup followed by function call
    // XPath: map{'f':floor#1, 'c':ceiling#1, 'r':round#1, 'a':abs#1}?(\"c
    // Expected: 2.0
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_153() {
    // Parenthesized Lookup preceded by predicate
    // XPath: map{'f':floor#1, 'c':ceiling#1, 'r':round#1, 'a':abs#1}[1]?(
    // Expected: 2.0
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_155() {
    // Lookup by name in a map; name is a keyword
    // XPath: map{'or':true(), 'and':true(), 'but':false()} ? or or 2 = 3
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_158() {
    // Double lookup into an array of maps
    // XPath: [map{'decimal':true(), 'integer':true(), 'polygon':false()},
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_159() {
    // Double lookup into a map of arrays
    // XPath: map{'decimal':true(), 'integer':true(), 'polygon':[11,22,33]
    // Expected: 22
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_160() {
    // Process all values in a map
    // XPath: some $x in map{'decimal':true(), 'integer':true(), 'polygon'
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_161() {
    // Process all values in a map
    // XPath: count(map{'decimal':true(), 'integer':true(), 'polygon':fals
    // Expected: 2
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_162() {
    // Process all values in a map
    // XPath: max(map{'decimal':12, 'integer':18, 'polygon':-4}?*)
    // Expected: 18
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_165() {
    // Ensure that an operator symbol after binary '?' is parsed correctly
    // XPath: let $x := map{\"div\":18} return $x?div
    // Expected: 18
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_166() {
    // Ensure that an operator symbol after unary '?' is parsed correctly
    // XPath: let $x := map{\"else\":18} return exists($x[?else = 18])
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_167() {
    // Ensure that a hyphenated name after binary '?' is parsed correctly
    // XPath: let $x := map{\"div\":81, \"div-2\":18} return $x?div-2
    // Expected: 18
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_168() {
    // Ensure that a hyphenated name after unary '?' is parsed correctly
    // XPath: let $x := map{\"else\":81, \"else-2\":18} return exists($x[?
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_220() {
    // multilevel lookup in nested arrays
    // XPath: [['A', 'B', 'C'], ['D', 'E'], []]?1?2
    // Expected: 'B'
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_230() {
    // lookup whose operand needs atomizing
    // XPath: [0]?(<_>1</_>)
    // Expected: 0
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_232() {
    // Lookup operator when LHS type isn't known statically
    // XPath: parse-json('[\"' || current-date() || '\"]')?1 => starts-wit
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}

#[test]
fn lookup_233() {
    // Lookup operator when LHS type isn't known statically
    // XPath: parse-json('{\"a\":'  || year-from-date(current-date()) || '
    // Expected: true
    // Calls stub function - will fail until prod:Lookup is implemented
    let _ = stub_prodlookup();
}
