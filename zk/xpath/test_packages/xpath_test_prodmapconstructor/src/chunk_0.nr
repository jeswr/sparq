//! Test chunk 0 for prod:MapConstructor
//! Contains 26 tests

use dep::xpath::stub_prodmapconstructor;

#[test]
fn mapconstructor_001() {
    // empty map
    // XPath: map:size(map{})
    // Expected: 0
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_002() {
    // empty map
    // XPath: map:size(map{     })
    // Expected: 0
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_003() {
    // empty map
    // XPath: map:size(map {})
    // Expected: 0
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_004() {
    // singleton map
    // XPath: map:size(map{\"a\":2})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_005() {
    // singleton map
    // XPath: map:size(map{ \"a\" : 2 })
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_006() {
    // doubleton map
    // XPath: map:size(map{\"a\":2,\"b\":3})
    // Expected: 2
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_007() {
    // doubleton map
    // XPath: map:size(map { \"a\" : 2 , \"b\" : 3 })
    // Expected: 2
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_012() {
    // key can be a node constructor
    // XPath: map:size(map {<a>x</a>:2})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_013() {
    // value can be a node constructor
    // XPath: map:size(map {2:<a>x</a>})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_014() {
    // value can be a node constructor with embedded content
    // XPath: map:size(map {2:<a>x{25}y</a>})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_015() {
    // colon after NCName is OK
    // XPath: map:size(<a><b>x</b></a>/map{b:2})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_017() {
    // colon here should be OK
    // XPath: map:size(<a><b>x</b></a>/map{self::a: b})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_019() {
    // colon here should be OK
    // XPath: map:size(<a><b>x</b></a>/map{*:b:b})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_020() {
    // colon here should be OK
    // XPath: map:size(<a><b>x</b></a>/map{* :b})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_021() {
    // colon here should be OK
    // XPath: map:size(<a><self>x</self></a>/map{self:2})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_022() {
    // rhs can be a map
    // XPath: map:size(map{2:map{3:4}})
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_026() {
    // colon after a QName is OK
    // XPath: declare namespace z = \"http://z.com\"; map:size(<a><z:b>x</
    // Expected: 1
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_027() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_028() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_029() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_030() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_031() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_032() {
    // Bug 29501
    // XPath: declare namespace a = \"http://example.com\";          <dot>
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_033() {
    // Bug 29501
    // XPath: <e><f>foo</f></e> ! map{*:*}         =>         deep-equal(m
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_034() {
    // Bug 29501
    // XPath: <e><f>foo</f></e> ! map{*:f:*:f}         =>         deep-equ
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}

#[test]
fn mapconstructor_035() {
    // Bug 29501
    // XPath: <e><f>4</f></e>!map{*:*div*,*||*:*}          =>          dee
    // Expected: true
    // Calls stub function - will fail until prod:MapConstructor is implemented
    let _ = stub_prodmapconstructor();
}
