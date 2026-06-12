//! Test chunk 1 for fn:xml-to-json
//! Contains 46 tests

use dep::xpath::stub_fnxml_to_json;

#[test]
fn xml_to_json_079() {
    // Unescaped quote after escaped backslash
    // XPath: xml-to-json(<fn:string escaped=\"true\">\\\\"\\</fn:string>)
    // Expected: '"\\\"\\"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_080() {
    // Unescaped solidus after escaped backslash
    // XPath: xml-to-json(<fn:string escaped=\"true\">\\\/\\</fn:string>)
    // Expected: '"\\\/\\"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_c_016() {
    // (Was) Invalid input
    // XPath: xml-to-json(//*:template[@name=\"t016\"]/*:variable/*)
    // Expected: "xxxx"
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_001() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t001\"]/*:variable/*)
    // Expected: '"a"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_002() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t002\"]/*:variable/*)
    // Expected: '"a"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_003() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t003\"]/*:variable/*)
    // Expected: '"\""'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_004() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t004\"]/*:variable/*)
    // Expected: '"\\"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_005() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t005\"]/*:variable/*)
    // Expected: '"\uD834\uDD1E"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_006() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t006\"]/*:variable/*)
    // Expected: '"\\\""'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_007() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t007\"]/*:variable/*)
    // Expected: '"\\\\"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_008() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t008\"]/*:variable/*)
    // Expected: '"\\uD834\\uDD1E"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_009() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t009\"]/*:variable/*)
    // Expected: '"  "'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_010() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t010\"]/*:variable/*)
    // Expected: '"-\"-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_011() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t011\"]/*:variable/*)
    // Expected: '"-\n-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_012() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t012\"]/*:variable/*)
    // Expected: '"-\r-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_013() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t013\"]/*:variable/*)
    // Expected: '"-\t-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_014() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t014\"]/*:variable/*)
    // Expected: '"-\u007F-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_016() {
    // Valid input - strings
    // XPath: xml-to-json(//*:template[@name=\"t014\"]/*:variable/*)
    // Expected: '"-\u007F-"'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_101() {
    // Valid input - booleans
    // XPath: xml-to-json(//*:template[@name=\"t101\"]/*:variable/*)
    // Expected: 'false'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_102() {
    // Valid input - booleans
    // XPath: xml-to-json(//*:template[@name=\"t102\"]/*:variable/*)
    // Expected: 'true'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_103() {
    // Valid input - booleans
    // XPath: xml-to-json(//*:template[@name=\"t103\"]/*:variable/*)
    // Expected: 'false'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_104() {
    // Valid input - booleans
    // XPath: xml-to-json(//*:template[@name=\"t104\"]/*:variable/*)
    // Expected: 'true'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_201() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t201\"]/*:variable/*)
    // Expected: '7'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_202() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t202\"]/*:variable/*)
    // Expected: '-0'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_203() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t203\"]/*:variable/*)
    // Expected: '1.0E6'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_204() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t204\"]/*:variable/*)
    // Expected: '-0.000001'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_205() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t205\"]/*:variable/*)
    // Expected: '0.001'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_206() {
    // Valid input - numbers
    // XPath: xml-to-json(//*:template[@name=\"t206\"]/*:variable/*)
    // Expected: '23'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_301() {
    // Valid input - null
    // XPath: xml-to-json(//*:template[@name=\"t301\"]/*:variable/*)
    // Expected: 'null'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_302() {
    // Valid input - null
    // XPath: xml-to-json(//*:template[@name=\"t302\"]/*:variable/*)
    // Expected: 'null'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_303() {
    // Valid input - null
    // XPath: xml-to-json(//*:template[@name=\"t303\"]/*:variable/*)
    // Expected: 'null'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_401() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t401\"]/*:variable/*)
    // Expected: '[]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_402() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t402\"]/*:variable/*)
    // Expected: '[]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_403() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t403\"]/*:variable/*)
    // Expected: '[]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_404() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t404\"]/*:variable/*)
    // Expected: '[null]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_405() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t405\"]/*:variable/*)
    // Expected: '["(",null,null,")"]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_406() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t406\"]/*:variable/*)
    // Expected: '["(",null,null,")"]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_407() {
    // Valid input - arrays
    // XPath: xml-to-json(//*:template[@name=\"t407\"]/*:variable/*)
    // Expected: '[null]'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_504() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t504\"]/*:variable/*)
    // Expected: '{"\"":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_505() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t505\"]/*:variable/*)
    // Expected: '{"\\":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_506() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t506\"]/*:variable/*)
    // Expected: '{"\uD834\uDD1E":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_507() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t507\"]/*:variable/*)
    // Expected: '{"\"":"1","\\\"":"0"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_508() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t508\"]/*:variable/*)
    // Expected: '{"A":"1","B":"  "}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_509() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t509\"]/*:variable/*)
    // Expected: '{"-\n-":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_510() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t510\"]/*:variable/*)
    // Expected: '{"-\r-":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}

#[test]
fn xml_to_json_d_511() {
    // Valid input - maps
    // XPath: xml-to-json(//*:template[@name=\"t511\"]/*:variable/*)
    // Expected: '{"-\t-":"1"}'
    // Calls stub function - will fail until fn:xml-to-json is implemented
    let _ = stub_fnxml_to_json();
}
