//! Test chunk 0 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_001() {
    // Attempts to look up function fn:node-name.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}node-name#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_002() {
    // Attempts to invoke function fn:node-name.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}node-name#0()
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_003() {
    // Attempts to look up function fn:node-name.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}node-name#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_004() {
    // Attempts to invoke function fn:node-name.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}node-name#1(/root)
    // Expected: root
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_005() {
    // Attempts to look up function fn:nilled.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}nilled#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_006() {
    // Attempts to invoke function fn:nilled.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}nilled#0()
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_007() {
    // Attempts to look up function fn:nilled.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}nilled#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_008() {
    // Attempts to invoke function fn:nilled.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}nilled#1(/root)
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_009() {
    // Attempts to look up function fn:string.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_010() {
    // Attempts to invoke function fn:string.
    // XPath: /root/child/Q{http://www.w3.org/2005/xpath-functions}string#
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_011() {
    // Attempts to look up function fn:string.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}string#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_012() {
    // Attempts to invoke function fn:string.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}string#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_013() {
    // Attempts to look up function fn:data.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}data#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_014() {
    // Attempts to invoke function fn:data.
    // XPath: /root/child/Q{http://www.w3.org/2005/xpath-functions}data#0(
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_015() {
    // Attempts to look up function fn:data.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}data#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_016() {
    // Attempts to invoke function fn:data.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}data#1(/root/child[
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_017() {
    // Attempts to look up function fn:base-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}base-uri#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_018() {
    // Attempts to invoke function fn:base-uri.
    // XPath: /root/Q{http://www.w3.org/2005/xpath-functions}base-uri#0()
    // Expected: http://www.w3.org/fots/fn/function-looku
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_019() {
    // Attempts to look up function fn:base-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}base-uri#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_021() {
    // Attempts to look up function fn:document-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}document-uri
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_023() {
    // Attempts to look up function fn:document-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}document-uri
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_025() {
    // Attempts to look up function fn:error.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}error#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_027() {
    // Attempts to look up function fn:error.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}error#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_029() {
    // Attempts to look up function fn:error.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}error#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_031() {
    // Attempts to look up function fn:error.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}error#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_033() {
    // Attempts to look up function fn:trace.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}trace#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_033a() {
    // Attempts to look up function fn:trace.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}trace#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_034() {
    // Attempts to invoke function fn:trace.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}trace#2(1, 'label')
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_034a() {
    // Attempts to invoke function fn:trace.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}trace#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_035() {
    // Attempts to look up function fn:abs.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}abs#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_036() {
    // Attempts to invoke function fn:abs.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}abs#1(-1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_037() {
    // Attempts to look up function fn:ceiling.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}ceiling#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_038() {
    // Attempts to invoke function fn:ceiling.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}ceiling#1(0.9)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_039() {
    // Attempts to look up function fn:floor.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}floor#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_040() {
    // Attempts to invoke function fn:floor.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}floor#1(1.1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_041() {
    // Attempts to look up function fn:round.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}round#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_042() {
    // Attempts to invoke function fn:round.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}round#1(1.1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_043() {
    // Attempts to look up function fn:round.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}round#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_044() {
    // Attempts to invoke function fn:round.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}round#2(1.1, 0)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_045() {
    // Attempts to look up function fn:round-half-to-even.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}round-half-t
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_046() {
    // Attempts to invoke function fn:round-half-to-even.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}round-half-to-even#
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_047() {
    // Attempts to look up function fn:round-half-to-even.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}round-half-t
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_048() {
    // Attempts to invoke function fn:round-half-to-even.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}round-half-to-even#
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_049() {
    // Attempts to look up function fn:number.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}number#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_050() {
    // Attempts to invoke function fn:number.
    // XPath: /root/child/Q{http://www.w3.org/2005/xpath-functions}number#
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_051() {
    // Attempts to look up function fn:number.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}number#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_052() {
    // Attempts to invoke function fn:number.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}number#1('1')
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_053() {
    // Attempts to look up function fn:format-integer.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}format-integ
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_054() {
    // Attempts to invoke function fn:format-integer.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}format-integer#2(1,
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_055() {
    // Attempts to look up function fn:format-integer.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}format-integ
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
