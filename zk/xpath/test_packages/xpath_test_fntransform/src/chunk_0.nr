//! Test chunk 0 for fn:transform
//! Contains 6 tests

use dep::xpath::stub_fntransform;

#[test]
fn fn_transform_1a() {
    // Loads a stylesheet from the location render.xsl, applies it to a document loaded
    // XPath: let $result := fn:transform(map {\"stylesheet-location\" : $
    // Expected: true
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}

#[test]
fn fn_transform_1b() {
    // Loads a stylesheet from the location render.xsl, applies it to a document loaded
    // XPath: let $result := fn:transform(map {\"stylesheet-location\" : $
    // Expected: true
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}

#[test]
fn fn_transform_1d() {
    // Same as fn-transform-1a with XP30 syntax
    // XPath: let $result := fn:transform(map {\"stylesheet-location\" : $
    // Expected: true
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}

#[test]
fn fn_transform_1e() {
    // Same as fn-transform-1b with XP30 syntax
    // XPath: let $result := fn:transform(map {\"stylesheet-location\" : $
    // Expected: true
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}

#[test]
fn fn_transform_27() {
    // Transform using cache option false
    // XPath: let $xsl  :=\"<xsl:stylesheet version='3.0' xmlns:xsl='http:
    // Expected: false
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}

#[test]
fn fn_transform_30() {
    // Transforms using serialized delivery-format option, and serialization-params (ye
    // XPath: let $out1 := fn:transform(map {\"stylesheet-location\" : $re
    // Expected: false
    // Calls stub function - will fail until fn:transform is implemented
    let _ = stub_fntransform();
}
