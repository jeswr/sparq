//! Unit tests for QName functions

use dep::xpath::{fn_qname, fn_qname_from_strings};

// ============================================================================
// Tests for fn_qname (fn:QName)
// ============================================================================

#[test]
fn test_fn_qname_with_namespace() {
    let namespace: [u8; 20] =
        [104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109, 0, 0]; // "http://example.com"
    let local: [u8; 5] = [108, 111, 99, 97, 108]; // "local"

    let qname = fn_qname::<20, 5>(namespace, 18, local, 5);

    assert(qname.namespace_len == 18);
    assert(qname.local_name_len == 5);
    assert(qname.namespace_uri[0] == 104); // 'h'
    assert(qname.local_name[0] == 108); // 'l'
}

#[test]
fn test_fn_qname_without_namespace() {
    let namespace: [u8; 5] = [0, 0, 0, 0, 0]; // empty
    let local: [u8; 4] = [110, 97, 109, 101]; // "name"

    let qname = fn_qname::<5, 4>(namespace, 0, local, 4);

    assert(qname.namespace_len == 0);
    assert(qname.local_name_len == 4);
    assert(qname.local_name[0] == 110); // 'n'
    assert(qname.local_name[1] == 97); // 'a'
    assert(qname.local_name[2] == 109); // 'm'
    assert(qname.local_name[3] == 101); // 'e'
}

// ============================================================================
// Tests for fn_qname_from_strings (fn:QName)
// ============================================================================

#[test]
fn test_fn_qname_from_strings_with_namespace() {
    let namespace: str<18> = "http://example.com";
    let local: str<5> = "local";

    let qname = fn_qname_from_strings(namespace, local);

    assert(qname.namespace_len == 18);
    assert(qname.local_name_len == 5);
}

#[test]
fn test_fn_qname_from_strings_empty_namespace() {
    let namespace: str<0> = "";
    let local: str<4> = "elem";

    let qname = fn_qname_from_strings(namespace, local);

    assert(qname.namespace_len == 0);
    assert(qname.local_name_len == 4);
}

#[test]
fn test_fn_qname_from_strings_xml_namespace() {
    let namespace: str<36> = "http://www.w3.org/XML/1998/namespace";
    let local: str<4> = "lang";

    let qname = fn_qname_from_strings(namespace, local);

    assert(qname.namespace_len == 36);
    assert(qname.local_name_len == 4);
}
