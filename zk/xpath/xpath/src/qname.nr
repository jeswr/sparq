//! QName type and operations for XPath/SPARQL
//!
//! Implements xsd:QName (Qualified Name) type for XML namespace handling.
//! QNames consist of a namespace URI and a local name.
//!
//! XPath operators implemented:
//! - op:QName-equal

// ============================================================================
// QName Type
// ============================================================================

/// Maximum length for namespace URI in bytes
global MAX_NAMESPACE_LEN: u32 = 256;

/// Maximum length for local name in bytes
global MAX_LOCALNAME_LEN: u32 = 128;

/// XSD QName representation
/// A qualified name consists of a namespace URI and a local name
pub struct XsdQName<let NS_LEN: u32, let LOCAL_LEN: u32> {
    /// Namespace URI as bytes
    pub namespace_uri: [u8; NS_LEN],
    /// Actual length of namespace URI
    pub namespace_len: u32,
    /// Local name as bytes
    pub local_name: [u8; LOCAL_LEN],
    /// Actual length of local name
    pub local_name_len: u32,
}

impl<let NS_LEN: u32, let LOCAL_LEN: u32> XsdQName<NS_LEN, LOCAL_LEN> {
    /// Create a new QName from components
    pub fn new(
        namespace_uri: [u8; NS_LEN],
        namespace_len: u32,
        local_name: [u8; LOCAL_LEN],
        local_name_len: u32,
    ) -> Self {
        XsdQName { namespace_uri, namespace_len, local_name, local_name_len }
    }

    /// Create a QName with no namespace (empty namespace URI)
    pub fn from_local_name(local_name: [u8; LOCAL_LEN], local_name_len: u32) -> Self {
        XsdQName { namespace_uri: [0; NS_LEN], namespace_len: 0, local_name, local_name_len }
    }

    /// Check if QName has a namespace
    pub fn has_namespace(self) -> bool {
        self.namespace_len > 0
    }
}

// ============================================================================
// QName Comparison
// ============================================================================

/// op:QName-equal
/// Two QNames are equal if both their namespace URIs and local names are equal
pub fn qname_equal<let NS1: u32, let LOCAL1: u32, let NS2: u32, let LOCAL2: u32>(
    a: XsdQName<NS1, LOCAL1>,
    b: XsdQName<NS2, LOCAL2>,
) -> bool {
    // First check if lengths match
    if a.namespace_len != b.namespace_len {
        false
    } else if a.local_name_len != b.local_name_len {
        false
    } else {
        // Compare namespace URIs byte by byte
        let mut ns_equal = true;
        for i in 0..NS1 {
            if (i as u32) < a.namespace_len {
                if (i as u32) < NS2 {
                    if a.namespace_uri[i] != b.namespace_uri[i] {
                        ns_equal = false;
                    }
                } else {
                    ns_equal = false;
                }
            }
        }

        // Compare local names byte by byte
        let mut local_equal = true;
        for i in 0..LOCAL1 {
            if (i as u32) < a.local_name_len {
                if (i as u32) < LOCAL2 {
                    if a.local_name[i] != b.local_name[i] {
                        local_equal = false;
                    }
                } else {
                    local_equal = false;
                }
            }
        }

        ns_equal & local_equal
    }
}

// ============================================================================
// QName Constructor Functions
// ============================================================================

/// fn:QName - Constructs an xs:QName from a namespace URI and local name
/// XPath: fn:QName($paramURI as xs:string?, $paramQName as xs:string) as xs:QName
/// The paramQName can be a prefixed name "prefix:local" or just "local".
/// For simplicity, this implementation extracts only the local name portion.
pub fn fn_qname<let NS_LEN: u32, let LOCAL_LEN: u32>(
    namespace_uri: [u8; NS_LEN],
    namespace_len: u32,
    local_name: [u8; LOCAL_LEN],
    local_name_len: u32,
) -> XsdQName<NS_LEN, LOCAL_LEN> {
    XsdQName::new(namespace_uri, namespace_len, local_name, local_name_len)
}

/// fn:QName from strings - Constructs an xs:QName from string parameters
/// This is a convenience wrapper that works with str<N> types
pub fn fn_qname_from_strings<let NS_LEN: u32, let LOCAL_LEN: u32>(
    namespace_uri: str<NS_LEN>,
    local_name: str<LOCAL_LEN>,
) -> XsdQName<NS_LEN, LOCAL_LEN> {
    let ns_bytes = namespace_uri.as_bytes();
    let local_bytes = local_name.as_bytes();

    // Calculate actual lengths (excluding null terminators)
    let mut ns_len: u32 = 0;
    for i in 0..NS_LEN {
        if (ns_bytes[i] != 0) & (ns_len == (i as u32)) {
            ns_len += 1;
        }
    }

    let mut local_len: u32 = 0;
    for i in 0..LOCAL_LEN {
        if (local_bytes[i] != 0) & (local_len == (i as u32)) {
            local_len += 1;
        }
    }

    XsdQName::new(ns_bytes, ns_len, local_bytes, local_len)
}

// ============================================================================
// QName Accessor Functions
// ============================================================================

/// fn:local-name-from-QName - Returns the local name part of a QName
/// XPath: fn:local-name-from-QName($arg as xs:QName?) as xs:NCName?
pub fn local_name_from_qname<let NS_LEN: u32, let LOCAL_LEN: u32>(
    qname: XsdQName<NS_LEN, LOCAL_LEN>,
) -> ([u8; LOCAL_LEN], u32) {
    (qname.local_name, qname.local_name_len)
}

/// fn:namespace-uri-from-QName - Returns the namespace URI part of a QName
/// XPath: fn:namespace-uri-from-QName($arg as xs:QName?) as xs:anyURI?
pub fn namespace_uri_from_qname<let NS_LEN: u32, let LOCAL_LEN: u32>(
    qname: XsdQName<NS_LEN, LOCAL_LEN>,
) -> ([u8; NS_LEN], u32) {
    (qname.namespace_uri, qname.namespace_len)
}

/// fn:prefix-from-QName - Returns the prefix part of a QName
/// XPath: fn:prefix-from-QName($arg as xs:QName?) as xs:NCName?
/// Note: Our QName struct doesn't store the prefix, so this returns empty
/// In a full implementation, you would need to add a prefix field to XsdQName
pub fn prefix_from_qname<let NS_LEN: u32, let LOCAL_LEN: u32, let PREFIX_LEN: u32>(
    _qname: XsdQName<NS_LEN, LOCAL_LEN>,
) -> ([u8; PREFIX_LEN], u32) {
    // Prefix is not stored in our simple QName representation
    // Return empty array
    ([0; PREFIX_LEN], 0)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_qname_equal_same() {
    // Same QName
    let ns: [u8; 32] = [
        104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109, 47, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]; // "http://example.com/"
    let local: [u8; 16] = [102, 111, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "foo"

    let q1 = XsdQName::new(ns, 19, local, 3);
    let q2 = XsdQName::new(ns, 19, local, 3);

    assert(qname_equal(q1, q2) == true);
}

#[test]
fn test_qname_equal_different_local() {
    let ns: [u8; 32] = [
        104, 116, 116, 112, 58, 47, 47, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109, 47, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let local1: [u8; 16] = [102, 111, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "foo"
    let local2: [u8; 16] = [98, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "bar"

    let q1 = XsdQName::new(ns, 19, local1, 3);
    let q2 = XsdQName::new(ns, 19, local2, 3);

    assert(qname_equal(q1, q2) == false);
}

#[test]
fn test_qname_equal_different_namespace() {
    let ns1: [u8; 32] = [
        104, 116, 116, 112, 58, 47, 47, 97, 46, 99, 111, 109, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]; // "http://a.com/"
    let ns2: [u8; 32] = [
        104, 116, 116, 112, 58, 47, 47, 98, 46, 99, 111, 109, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]; // "http://b.com/"
    let local: [u8; 16] = [102, 111, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "foo"

    let q1 = XsdQName::new(ns1, 13, local, 3);
    let q2 = XsdQName::new(ns2, 13, local, 3);

    assert(qname_equal(q1, q2) == false);
}

#[test]
fn test_qname_no_namespace() {
    let local1: [u8; 16] = [102, 111, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "foo"
    let local2: [u8; 16] = [102, 111, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // "foo"

    let q1: XsdQName<32, 16> = XsdQName::from_local_name(local1, 3);
    let q2: XsdQName<32, 16> = XsdQName::from_local_name(local2, 3);

    assert(qname_equal(q1, q2) == true);
    assert(q1.has_namespace() == false);
}
