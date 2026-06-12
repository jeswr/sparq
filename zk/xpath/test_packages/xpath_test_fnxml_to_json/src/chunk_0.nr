//! Test chunk 0 for fn:xml-to-json
//! Contains 50 tests

use dep::xpath::stub_fnxml_to_json;

#[test]
fn xml_to_json_001() {
    // simple 'null' content, not indented
    // XPath: xml-to-json(json-to-xml('null',map{'liberal':true()}))
    // Expected: "null"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_002() {
    // simple 'null' content, not indented
    // XPath: xml-to-json(json-to-xml('null',map{'liberal':true()}))
    // Expected: "null"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_003() {
    // simple 'null' content, not indented
    // XPath: xml-to-json(json-to-xml('null',map{'liberal':true()}))
    // Expected: "null"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_004() {
    // simple 'true' content, not indented
    // XPath: xml-to-json(json-to-xml('true'))
    // Expected: "true"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_005() {
    // simple 'false' content
    // XPath: xml-to-json(json-to-xml('false'))
    // Expected: "false"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_006() {
    // simple 'number' content
    // XPath: xml-to-json(json-to-xml('93.7'))
    // Expected: "93.7"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_007() {
    // simple variable content
    // XPath: declare variable $in := '\"London\"';           xml-to-json(
    // Expected: '"London"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_008() {
    // simple variable with empty string
    // XPath: declare variable $in := '\"\"';           xml-to-json(json-t
    // Expected: '""'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_009() {
    // empty array
    // XPath: xml-to-json(json-to-xml('[]'))
    // Expected: '[]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_010() {
    // singleton array
    // XPath: xml-to-json(json-to-xml('[34]'))
    // Expected: '[34]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_011() {
    // two-member array
    // XPath: xml-to-json(json-to-xml('[true, false]'))
    // Expected: '[true,false]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_012() {
    // simple variable with empty string
    // XPath: xml-to-json(json-to-xml('[\"A\", \"B\", \"C\", \"D\", \"E\"]
    // Expected: '["A","B","C","D","E"]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_013() {
    // empty map
    // XPath: xml-to-json(json-to-xml('{}'))
    // Expected: '{}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_014() {
    // 1 entry map
    // XPath: xml-to-json(json-to-xml('{\"A\":true}'))
    // Expected: '{"A":true}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_015() {
    // five entry map
    // XPath: xml-to-json(json-to-xml('{\"A\":1, \"B\":2, \"C\":3, \"D\":4
    // Expected: '{"A":1,"B":2,"C":3,"D":4,"E":5}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_016() {
    // complex map
    // XPath: xml-to-json(json-to-xml('{\"A\":[1,2], \"B\":[true,false], \
    // Expected: '{"A":[1,2],"B":[true,false],"C":["Paris
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_017() {
    // 1 entry map
    // XPath: xml-to-json(json-to-xml('\"\\\/\\"\r\t\n\u0020\"'))
    // Expected: '"\\\/\"\r\t\n "'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_021() {
    // simple 'null' content
    // XPath: xml-to-json(<null xmlns=\"http://www.w3.org/2005/xpath-funct
    // Expected: "null"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_022() {
    // simple true content
    // XPath: xml-to-json(<boolean xmlns=\"http://www.w3.org/2005/xpath-fu
    // Expected: "true"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_023() {
    // simple numeric content
    // XPath: xml-to-json(<number xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: "23"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_024() {
    // simple string content
    // XPath: xml-to-json(<string xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: '"banana"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_025() {
    // boolean with whitespace (bug 28180)
    // XPath: xml-to-json(<boolean xmlns=\"http://www.w3.org/2005/xpath-fu
    // Expected: "false"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_026() {
    // number with leading zeroes and whitespace (bug 28180)
    // XPath: xml-to-json(<number xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: "5"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_027() {
    // whitespace string (bug 28180)
    // XPath: declare boundary-space preserve;         xml-to-json(<string
    // Expected: '"   "'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_028() {
    // array with comments and whitespace
    // XPath: declare boundary-space preserve;         xml-to-json(<array
    // Expected: "[null,null]"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_029() {
    // map with comments and whitespace
    // XPath: declare boundary-space preserve;         xml-to-json(<map xm
    // Expected: '{"a":null,"b":null}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_030() {
    // Document node wrapper
    // XPath: declare boundary-space preserve;         xml-to-json(documen
    // Expected: "[null,null]"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_031() {
    // Use a namespace prefix
    // XPath: declare boundary-space preserve;         xml-to-json(documen
    // Expected: "[null,null]"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_037() {
    // String split by a comment
    // XPath: xml-to-json(<string xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: '"banana"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_038() {
    // Number split by a comment
    // XPath: xml-to-json(<number xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: '27'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_039() {
    // Boolean split by a comment
    // XPath: declare boundary-space preserve;         xml-to-json(<boolea
    // Expected: 'true'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_045() {
    // Effect of xsi:type (bug 28181)
    // XPath: xml-to-json(<number xsi:type=\"xs:short\" xmlns=\"http://www
    // Expected: "231"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_049() {
    // String that needs escaping
    // XPath: xml-to-json(<string xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: '"abc\ndef"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_050() {
    // String that needs escaping
    // XPath: xml-to-json(<string xmlns=\"http://www.w3.org/2005/xpath-fun
    // Expected: '"abc\u0007def"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_051() {
    // Key that needs escaping
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"\t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_052() {
    // Key marked as escaped
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"\t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_053() {
    // Key marked as not escaped
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"\\t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_054() {
    // String marked as escaped
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"t1":"\t","t2":"\\t","t3":"\t","t4":"\
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_055() {
    // String marked as not escaped
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"tab":"\\t"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_056() {
    // Indent
    // XPath: let $result := xml-to-json(<map xmlns=\"http://www.w3.org/20
    // Expected: '{"a":1,"b":2}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_057() {
    // Indent
    // XPath: let $result := xml-to-json(<map xmlns=\"http://www.w3.org/20
    // Expected: false
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_058() {
    // Indent=false
    // XPath: let $result := xml-to-json(<map xmlns=\"http://www.w3.org/20
    // Expected: true
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_060() {
    // Unknown options are ignored
    // XPath: let $result := xml-to-json(<map xmlns=\"http://www.w3.org/20
    // Expected: true
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_064() {
    // (Was) Invalid position of escaped-key attribute
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"\\t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_065() {
    // (Was) Invalid position of escaped attribute
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"\\t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_068() {
    // Attributes in other namespaces are OK
    // XPath: xml-to-json(<map xmlns=\"http://www.w3.org/2005/xpath-functi
    // Expected: '{"t":"tab"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_070() {
    //           Converting a subtree of a valid tree: test case for bug 29917.
    // XPath: let $x := json-to-xml('[{ \"givenNames\": [\"Michael\", \"Ho
    // Expected: '["Michael","Howard"]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_071() {
    // A valid JSON escape sequence is copied unchanged to the output
    // XPath: xml-to-json(<fn:string escaped=\"true\">\\\b\f\n\r\t\/\\\u00
    // Expected: '"\\\b\f\n\r\t\/\\\u00a0\u00A0\ud83d\udc
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_072() {
    // Any unescaped occurrence of .... is replaced by an escape sequence
    // XPath: xml-to-json(<fn:string escaped=\"true\">{codepoints-to-strin
    // Expected: '"\b\t\n\f\r\"\/"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_073() {
    // Any other character in the range 1-31, 127-159 .... is replaced by \uHHHH
    // XPath: xml-to-json(<fn:string escaped=\"true\">{codepoints-to-strin
    // Expected: '"\u0001\u0002\u0003\u0004\u0005\u0006\u
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}
