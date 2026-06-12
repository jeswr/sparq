//! Test chunk 0 for fn:serialize
//! Contains 24 tests

use dep::xpath::stub_fnserialize;

#[test]
fn serialize_xml_120() {
    // serialize test - params supplied as map: error - disallowed no-namespace entry
    // XPath: let $params := map { QName(\"\",\"indent\") : true() }
    // Expected: false
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_html_003() {
    // serialize test - html method - HTML5 fragment - no DOCTYPE
    // XPath: let $doc := <html><head/><body><p>Hello World!</p></body></h
    // Expected: <body><p>Hello World!</p></body>
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_008() {
    // serialize test - params supplied as map, json output method, $arg is a comment n
    // XPath: let $params := map {             \"method\" : \"json\",
    // Expected: true
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_009() {
    // serialize test - params supplied as map, json output method, $arg is an array of
    // XPath: let $params := map {             \"method\" : \"json\",
    // Expected: true
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_104() {
    // Boolean value
    // XPath: normalize-space(serialize(true(), map{'method':'json'}))
    // Expected: true
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_105() {
    // Boolean value
    // XPath: normalize-space(serialize(false(), map{'method':'json'}))
    // Expected: false
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_108() {
    // JSON array
    // XPath: let $r := serialize(parse-json('[1, 2, 3, \"four\", true, fa
    // Expected: [1,2,3,"four",true,false,null]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_109() {
    // JSON array
    // XPath: let $r := serialize([1, 2, 3, \"four\", true(), false()], ma
    // Expected: [1,2,3,"four",true,false]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_110() {
    // Serialize nested arrays
    // XPath: let $r := serialize(parse-json('[[1, 2], [3, 4], [5, 6], [7]
    // Expected: [[1,2],[3,4],[5,6],[7],[],[null]]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_111() {
    // Serialize a map containing an array
    // XPath: let $r := serialize(map{\"abc\":array{1 to 10}}, map{'method
    // Expected: {"abc":[1,2,3,4,5,6,7,8,9,10]}
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_112() {
    // Serialize an array of maps
    // XPath: let $r := serialize([map{\"abc\":1},map{\"def\":2},map{\"ghi
    // Expected: [{"abc":1},{"def":2},{"ghi":3}]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_114() {
    // Non-BMP string
    // XPath: let $r := serialize('?', map{'method':'json', 'encoding':'IS
    // Expected: "\uD834\uDD1E"
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_115() {
    // String with newline
    // XPath: let $r := serialize(' ', map{'method':'json'}) return
    // Expected: "\n"
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_116() {
    // Nested maps with indent='no' - require no optional space
    // XPath: serialize([map{\"abc\":map{\"abc\":1}},map{\"def\":map{\"def
    // Expected: [{"abc":{"abc":1}},{"def":{"def":2}},{"g
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_117() {
    // Nested maps with indent='yes' - (we can't check that it's actually             i
    // XPath: let $r :=             serialize([map{\"abc\":map{\"abc\":1}}
    // Expected: [{"abc":{"abc":1}},{"def":{"def":2}},{"g
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_123() {
    // untypedAtomic input
    // XPath: let $r := serialize([0,0,xs:untypedAtomic(\"abcd\")], map{'m
    // Expected: [0,0,"abcd"]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_125() {
    // JSON serialization converts date to string
    // XPath: let $r := serialize([0,0,xs:date('2011-04-06')], map{'method
    // Expected: [0,0,"2011-04-06"]
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_126() {
    // JSON serialization converts date to string
    // XPath: let $r := serialize(map{\"a\":xs:date('2011-04-06')}, map{'m
    // Expected: {"a":"2011-04-06"}
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_127() {
    // JSON serialization converts nodes to lexical XML
    // XPath: let $r := serialize(map{\"a\":doc($uri)}, map{'method':'json
    // Expected: {"a":"<a>text<\/a>"}
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_json_128() {
    // JSON serialization converts URI value to a string
    // XPath: serialize(map{\"uri\":xs:anyURI('http://www.w3.org/')}, map{
    // Expected: {"uri":"http:\/\/www.w3.org\/"}
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_adaptive_001() {
    // Adaptive method, atomic values
    // XPath: serialize((1,2,3), map{'method':'adaptive', 'item-separator'
    // Expected: 1;2;3
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_adaptive_002() {
    // Adaptive method, nodes
    // XPath: serialize((parse-xml('<a/>'), parse-xml('<b/>')),
    // Expected: <a/>;<b/>
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_adaptive_003() {
    // Adaptive method, free-standing attribute nodes
    // XPath: serialize((parse-xml('<a x=\"1\"/>')/a/@x, parse-xml('<b y=\
    // Expected: x="1";y="2"
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}

#[test]
fn serialize_adaptive_004() {
    // Adaptive method, maps
    // XPath: serialize((map{1:true(), 2:false()}, map{8:80, 9:90}),
    // Expected: map{1:true(),2:false()};map{8:80,9:90}
    // Calls stub function - will fail until fn:serialize is implemented
    let _ = stub_fnserialize();
}
