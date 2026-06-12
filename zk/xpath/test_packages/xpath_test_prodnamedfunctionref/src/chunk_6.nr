//! Test chunk 6 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_318() {
    // Attempts to invoke function fn:distinct-values.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}distinct-values#2((
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_319() {
    // Attempts to look up function fn:index-of.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}index-of#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_320() {
    // Attempts to invoke function fn:index-of.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}index-of#2((1, 'str
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_321() {
    // Attempts to look up function fn:index-of.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}index-of#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_322() {
    // Attempts to invoke function fn:index-of.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}index-of#3((1, 'str
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_323() {
    // Attempts to look up function fn:deep-equal.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}deep-equal#2
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_324() {
    // Attempts to invoke function fn:deep-equal.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}deep-equal#2((1, tr
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_325() {
    // Attempts to look up function fn:deep-equal.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}deep-equal#3
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_326() {
    // Attempts to invoke function fn:deep-equal.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}deep-equal#3((1, tr
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_327() {
    // Attempts to look up function fn:zero-or-one.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}zero-or-one#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_328() {
    // Attempts to invoke function fn:zero-or-one.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}zero-or-one#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_329() {
    // Attempts to look up function fn:one-or-more.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}one-or-more#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_330() {
    // Attempts to invoke function fn:one-or-more.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}one-or-more#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_331() {
    // Attempts to look up function fn:exactly-one.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}exactly-one#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_332() {
    // Attempts to invoke function fn:exactly-one.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}exactly-one#1(1)
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_333() {
    // Attempts to look up function fn:count.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}count#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_334() {
    // Attempts to invoke function fn:count.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}count#1((1, true())
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_335() {
    // Attempts to look up function fn:avg.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}avg#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_336() {
    // Attempts to invoke function fn:avg.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}avg#1((1, 3))
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_337() {
    // Attempts to look up function fn:max.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}max#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_338() {
    // Attempts to invoke function fn:max.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}max#1((1, 3))
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_339() {
    // Attempts to look up function fn:max.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}max#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_340() {
    // Attempts to invoke function fn:max.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}max#2((1, 3), 'http
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_341() {
    // Attempts to look up function fn:min.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}min#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_342() {
    // Attempts to invoke function fn:min.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}min#1((1, 3))
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_343() {
    // Attempts to look up function fn:min.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}min#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_344() {
    // Attempts to invoke function fn:min.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}min#2((1, 3), 'http
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_345() {
    // Attempts to look up function fn:sum.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}sum#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_346() {
    // Attempts to invoke function fn:sum.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}sum#1((1, 2))
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_347() {
    // Attempts to look up function fn:sum.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}sum#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_348() {
    // Attempts to invoke function fn:sum.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}sum#2((1, 2), 0)
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_349() {
    // Attempts to look up function fn:id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}id#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_351() {
    // Attempts to look up function fn:id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}id#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_353() {
    // Attempts to look up function fn:element-with-id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}element-with
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_355() {
    // Attempts to look up function fn:element-with-id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}element-with
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_357() {
    // Attempts to look up function fn:idref.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}idref#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_359() {
    // Attempts to look up function fn:idref.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}idref#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_361() {
    // Attempts to look up function fn:generate-id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}generate-id#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_363() {
    // Attempts to look up function fn:generate-id.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}generate-id#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_365() {
    // Attempts to look up function fn:doc.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}doc#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_367() {
    // Attempts to look up function fn:doc-available.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}doc-availabl
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_368() {
    // Attempts to invoke function fn:doc-available.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}doc-available#1('ht
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_369() {
    // Attempts to look up function fn:collection.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}collection#0
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_371() {
    // Attempts to look up function fn:collection.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}collection#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_373() {
    // Attempts to look up function fn:uri-collection.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}uri-collecti
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_375() {
    // Attempts to look up function fn:uri-collection.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}uri-collecti
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_377() {
    // Attempts to look up function fn:unparsed-text.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}unparsed-tex
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_379() {
    // Attempts to look up function fn:unparsed-text.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}unparsed-tex
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_381() {
    // Attempts to look up function fn:unparsed-text-lines.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}unparsed-tex
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_383() {
    // Attempts to look up function fn:unparsed-text-lines.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}unparsed-tex
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
