//! Test chunk 0 for prod:ArrayTest
//! Contains 39 tests

use dep::xpath::stub_prodarraytest;

#[test]
fn arraytest_028() {
    // atomize array with nodes
    // XPath: let $x := <root><elem>1</elem><elem>2</elem></root> return [
    // Expected: 3
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_029() {
    // atomize array with nodes
    // XPath: fn:data([<root><elem>1</elem><elem>2</elem></root> + 4])
    // Expected: 16
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_035() {
    // instance of, wrong cardinality
    // XPath: [(\"A\", \"B\"), \"C\"] instance of array(xs:string)
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_036() {
    // instance of, wrong cardinality
    // XPath: [(), \"A\"] instance of array(xs:string)
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_037() {
    // instance of, wrong type
    // XPath: [(), \"A\"] instance of array(xs:integer*)
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_038() {
    // instance of nested array
    // XPath: [[\"A\"], [\"B\"]] instance of array(array(xs:string))
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_039() {
    // instance of array with items
    // XPath: [[\"A\"], [\"B\"]] instance of array(item())
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_040() {
    // instance of array, xs:integer is subtype of xs:decimal
    // XPath: [xs:integer(\"10\")] instance of array(xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_041() {
    // instance of item
    // XPath: [[\"A\"], [\"B\"]] instance of item()
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_042() {
    // instance of function
    // XPath: [[\"A\"], [\"B\"]] instance of function(*)
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_043() {
    // instance of function
    // XPath: [[\"A\"], [\"B\"]] instance of function(xs:integer) as item(
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_045() {
    // general comparison
    // XPath: ['A'] = 'A'
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_046() {
    // general comparison
    // XPath: ['A', 'B', 'C'] = 'A'
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_050() {
    // attribute content
    // XPath: <chapter ref=\"{[1, 5 to 7, 9]}\"/>/@ref/string()
    // Expected: "1 5 6 7 9"
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_052() {
    // Dynamic function call with operand an array
    // XPath: [1, 2, 3](2)
    // Expected: 2
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_053() {
    // Dynamic function call with operand an array
    // XPath: array{1, 2, 3}(2)
    // Expected: 2
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_055() {
    // Array type testing
    // XPath: let $array:= ['a','b','c','d','e', 0] return $array instance
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_056() {
    // Array type testing
    // XPath: let $array:= ['a','b','c','d','e', ('f', 'g')] return $array
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_057() {
    // Array type testing
    // XPath: let $array:= ['a','b','c','d','e', ()] return $array instanc
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_058() {
    // Array type testing
    // XPath: let $array:= ['a','b','c','d','e', (1 to 8)] return $array i
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_059() {
    // Array type testing
    // XPath: let $array:= ['a','b','c','d','e', map{}] return $array inst
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_060() {
    // Array type testing
    // XPath: let $array:= [['a','b'],['c','d'], [], ['e']] return $array
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_061() {
    // Array type testing
    // XPath: let $array:= [(['a','b'],['c','d']), ([], ['e'])] return $ar
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_062() {
    // Array type testing
    // XPath: let $array:= [(['a','b'],['c','d']), ([], ['e'])] return $ar
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_063() {
    // Array type testing
    // XPath: let $array:= [floor#1, ceiling#1, round#1] return $array ins
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_064() {
    // Array type testing
    // XPath: let $array:= [floor#1, ceiling#1, round#1, name#1] return $a
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_065() {
    // Array type testing
    // XPath: let $array:= [] return $array instance of array(function(xs:
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_066() {
    // Array type testing
    // XPath: let $array:= [map{1: map{2: map{}}}] return $array instance
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_067() {
    // Array type testing
    // XPath: let $array:= [map{1: map{2: map{true(): false()}}}] return $
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_076() {
    // Array type testing
    // XPath: let $f := function($a as array(xs:string+)) as xs:boolean {a
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_077() {
    // Array type testing
    // XPath: let $f := function($a as array(xs:string?)) as xs:boolean {a
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_078() {
    // Array type testing
    // XPath: let $f := function($a as array(xs:anyAtomicType+)) as xs:boo
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_080() {
    // Array type testing
    // XPath: let $f := function($a as array(array(xs:string*))) as xs:boo
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_082() {
    // Array type testing
    // XPath: let $f := function($a as array(array(xs:string*)+)) as xs:bo
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_083() {
    // Array type testing
    // XPath: let $f := function($a as array(function(xs:numeric?) as xs:n
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_085() {
    // Array type testing
    // XPath: let $f := function($a as array(function(xs:numeric?) as xs:n
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_086() {
    // Array type testing
    // XPath: let $f := function($a as array(map(xs:integer, map(xs:intege
    // Expected: true
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_088() {
    // Array type testing - empty array satisfies otherwise incompatible types
    // XPath: let $f := function($a as array(xs:integer)) as xs:boolean {a
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}

#[test]
fn arraytest_089() {
    // Array type testing - empty array satisfies otherwise incompatible types
    // XPath: declare function local:f($a as array(xs:integer)) as xs:bool
    // Expected: false
    // Calls stub function - will fail until prod:ArrayTest is implemented
    let _ = stub_prodarraytest();
}
