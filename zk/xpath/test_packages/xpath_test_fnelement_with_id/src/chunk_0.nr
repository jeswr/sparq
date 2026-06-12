//! Test chunk 0 for fn:element-with-id
//! Contains 5 tests

use dep::xpath::stub_fnelement_with_id;

#[test]
fn fn_element_with_id_1() {
    // Contrast behaviour of id() vs element-with-id(). tests Erratum FO.E31 - new elem
    // XPath: id('alpha beta iota kappa')/local-name(), \"|\", element-wit
    // Expected: Element-with-ID-attribute Element-with-I
    // Calls stub function - will fail until fn:element-with-id is implemented
    let _ = stub_fnelement_with_id();
}

#[test]
fn fn_element_with_id_2() {
    // Contrast behaviour of id() vs element-with-id(). Element of type derived from xs
    // XPath: id('lambda mu')/local-name(), \"|\", element-with-id('lambda
    // Expected: id id | Element-with-Restricted-ID-child
    // Calls stub function - will fail until fn:element-with-id is implemented
    let _ = stub_fnelement_with_id();
}

#[test]
fn fn_element_with_id_3() {
    // Contrast behaviour of id() vs element-with-id(). Element of complex type with si
    // XPath: id('nu')/local-name(), \"|\", element-with-id('nu')/local-na
    // Expected: id | Element-with-complex-ID-child
    // Calls stub function - will fail until fn:element-with-id is implemented
    let _ = stub_fnelement_with_id();
}

#[test]
fn fn_element_with_id_4() {
    // Contrast behaviour of id() vs element-with-id(). Element of with type list of xs
    // XPath: id('xi')/local-name(), \"|\", element-with-id('xi')/local-na
    // Expected: id | Element-with-ID-list-child | 0 | 0
    // Calls stub function - will fail until fn:element-with-id is implemented
    let _ = stub_fnelement_with_id();
}

#[test]
fn fn_element_with_id_5() {
    // Contrast behaviour of id() vs element-with-id(). Element of with type union of x
    // XPath: id('omicron')/local-name(), \"|\",  element-with-id('omicron
    // Expected: id | Element-with-ID-union-child | 0 | 0
    // Calls stub function - will fail until fn:element-with-id is implemented
    let _ = stub_fnelement_with_id();
}
