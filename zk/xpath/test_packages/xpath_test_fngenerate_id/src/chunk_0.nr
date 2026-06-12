//! Test chunk 0 for fn:generate-id
//! Contains 14 tests

use dep::xpath::stub_fngenerate_id;

#[test]
fn generate_id_008() {
    // generate-id() with no arguments applied to a document node
    // XPath: generate-id() eq generate-id(/)
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_009() {
    // generate-id() with no arguments applied to an element node
    // XPath: /*/(generate-id() eq generate-id(.))
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_010() {
    // Uniqueness of generated IDs
    // XPath: let $nodes := (/ | //*/(.|@*|comment()|processing-instructio
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_011() {
    // Uniqueness of generated IDs including namespaces
    // XPath: let $nodes := //*/(@* | namespace::*)              return co
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_012() {
    // Uniqueness of generated IDs for document nodes across multiple documents
    // XPath: let $nodes := collection()             return count($nodes)
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_013() {
    // Uniqueness of generated IDs for element nodes across multiple documents
    // XPath: let $nodes := collection()//*             return count($node
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_016() {
    // generate-id() changes when a node is copied
    // XPath: import module namespace copy=\"http://www.w3.org/QT3/copy\";
    // Expected: false
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_017() {
    // generate-id() applied to a parentless attribute node
    // XPath: import module namespace copy=\"http://www.w3.org/QT3/copy\";
    // Expected: false
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_018() {
    // generate-id() applied to a sequence of element nodes in backwards-compatibility
    // XPath: generate-id(//*) eq generate-id(/*)
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_019() {
    // All generated IDs are ASCII alphanumeric
    // XPath: let $nodes := (/ | //*/(.|@*|comment()|processing-instructio
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_020() {
    // generate-id() applied to constructed nodes
    // XPath: let $nodes := (<a lang='de' xml:lang='de'>Insel</a>, <a lang
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_021() {
    // generate-id() applied to constructed nodes
    // XPath: let $nodes := (<a lang='de' xml:lang='de'>Insel</a>, <a lang
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_022() {
    // generate-id() applied to constructed nodes
    // XPath: let $nodes := (text{''}, text{''}, text{''}, text{''})
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}

#[test]
fn generate_id_023() {
    // generate-id() applied to constructed nodes
    // XPath: let $nodes := (text{''}, text{''}, text{''}, text{''})
    // Expected: true
    // Calls stub function - will fail until fn:generate-id is implemented
    let _ = stub_fngenerate_id();
}
