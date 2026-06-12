//! Stub functions for XPath functions that cannot be implemented in Noir/ZK context.
//!
//! These functions are categorized by the reason they cannot be implemented:
//!
//! ## Categories of Unimplementable Functions
//!
//! ### 1. External Resource Access (EXTERNAL)
//! Functions requiring file system, network, or environment access.
//! ZK circuits cannot access external resources at proof generation time.
//! - fn:doc, fn:doc-available, fn:collection
//! - fn:unparsed-text, fn:unparsed-text-available, fn:unparsed-text-lines
//! - fn:environment-variable, fn:available-environment-variables
//! - fn:json-doc (URL loading)
//!
//! ### 2. Runtime Context (CONTEXT)
//! Functions requiring access to system state that doesn't exist in ZK proofs.
//! - fn:current-date, fn:current-dateTime, fn:current-time (no system clock)
//! - fn:static-base-uri (no execution context)
//! - fn:random-number-generator (no randomness in deterministic ZK)
//!
//! ### 3. Higher-Order Functions (HIGHER_ORDER)
//! Noir doesn't support passing functions as arguments.
//! - fn:filter, fn:fold-left, fn:fold-right, fn:for-each, fn:for-each-pair
//! - fn:apply
//!
//! ### 4. Function Introspection (INTROSPECTION)
//! Noir doesn't support runtime function reflection.
//! - fn:function-arity, fn:function-lookup, fn:function-name
//!
//! ### 5. XML/DOM Operations (XML_DOM)
//! Requires XML parsing and DOM tree operations not available in Noir.
//! - fn:parse-xml, fn:parse-xml-fragment, fn:serialize
//! - fn:json-to-xml, fn:xml-to-json
//! - fn:base-uri, fn:document-uri, fn:root, fn:path
//! - fn:id, fn:idref, fn:element-with-id
//! - fn:local-name, fn:name, fn:namespace-uri, fn:node-name
//! - fn:lang, fn:nilled, fn:has-children, fn:innermost, fn:outermost
//! - All prod:* (XQuery grammar productions)
//!
//! ### 6. Regex Operations (REGEX) - POTENTIAL EXTERNAL SUPPORT
//! Full regex support not available in native Noir.
//! - fn:analyze-string, fn:matches, fn:replace
//! **FUTURE**: zkRegEx (https://github.com/zkemail/zk-regex) has "Noir support coming soon"
//! Once available, fn:matches could potentially be implemented using zkRegEx.
//!
//! ### 7. Collation/Locale (COLLATION)
//! Locale-aware string operations not supported.
//! - fn:collation-key
//!
//! ### 8. Complex Transformations (TRANSFORM)
//! XSLT transformations not supported.
//! - fn:transform
//!
//! ### 9. Format Functions (FORMAT)
//! Output formatting to strings is limited since Noir can't create runtime strings.
//! - fn:format-date, fn:format-dateTime, fn:format-time
//! - fn:format-integer, fn:format-number
//!
//! ### 10. Partial Implementation Available (PARTIAL)
//! Some functionality exists but with different API.
//! - fn:parse-json -> Use JSON512b::parse_json_from_string()
//! - fn:normalize-unicode -> ASCII strings work, Unicode normalization not supported
//!
//! ## External Noir Libraries (Potential Future Integrations)
//!
//! The following libraries from awesome-noir could potentially extend XPath support:
//!
//! ### String Operations
//! - **noir_string_search** (https://github.com/noir-lang/noir_string_search)
//!   Proves substring existence within a larger string.
//!   Could support: fn:contains (proving), fn:substring-before/after (position finding)
//!   API: `haystack.substring_match(needle)` returns `(bool, u32)` for match and position
//!   Types: StringBody32-16384 (haystack), SubString32-1024 (needle)
//!
//! - **noir-string-utils** (https://github.com/madztheo/noir-string-utils)
//!   Common string operations wrapper.
//!   Provides: contains(), index_of(), last_index_of(), char_at(), to_uint()
//!   Note: Last updated for Noir 0.24, may need updates for current version.
//!
//! ### Regex (Future)
//! - **zkRegEx** (https://github.com/zkemail/zk-regex)
//!   ZK proof of regex verification. Noir support "coming soon".
//!   Would enable: fn:matches, potentially fn:analyze-string
//!
//! ### Date/Time
//! - **noir-date** (https://github.com/madztheo/noir-date) - Date parsing
//! - **DateTimeNr** (https://github.com/doctoruber/DateTimeNr) - DateTime parsing
//!   Note: Our xpath already has comprehensive datetime support in datetime.nr
//!
//! Note: Many functions have real implementations in xpath_fn.nr and xpath_op.nr.
//! Check those modules first before assuming a function is not implemented.

// ============================================================================
// REGEX: Regular Expression Functions
// ============================================================================
// FUTURE: zkRegEx (https://github.com/zkemail/zk-regex) has "Noir support coming soon"
// Once available, these functions could potentially be implemented using zkRegEx.
// Alternative: noir_string_search can prove substring existence (not full regex)

/// fn:analyze-string - CANNOT IMPLEMENT (requires regex engine)
/// Analyzes a string using a regular expression and returns an XML structure.
/// FUTURE: May be partially implementable with zkRegEx for match extraction.
pub fn stub_fnanalyze_string() -> bool {
    assert(false, "fn:analyze-string requires regex engine - not available in ZK");
    false
}

/// fn:matches - CANNOT IMPLEMENT (requires regex engine)
/// Returns true if the string matches the regular expression pattern.
/// FUTURE: Could be implemented with zkRegEx once Noir support is available.
/// ALTERNATIVE: For simple substring matching, use noir_string_search library.
pub fn stub_fnmatches() -> bool {
    assert(false, "fn:matches requires regex engine - not available in ZK");
    false
}

/// fn:matches.re - CANNOT IMPLEMENT (requires regex engine)
/// FUTURE: Could be implemented with zkRegEx once Noir support is available.
pub fn stub_fnmatches_re() -> bool {
    assert(false, "fn:matches.re requires regex engine - not available in ZK");
    false
}

/// fn:replace - CANNOT IMPLEMENT (requires regex engine + string creation)
/// Replaces occurrences of a pattern in a string.
/// Note: Even with regex support, creating new strings at runtime is limited in Noir.
pub fn stub_fnreplace() -> bool {
    assert(false, "fn:replace requires regex engine - not available in ZK");
    false
}

// ============================================================================
// HIGHER_ORDER: Higher-Order Functions
// ============================================================================

/// fn:apply - CANNOT IMPLEMENT (requires first-class functions)
/// Applies a function to a sequence of arguments.
pub fn stub_fnapply() -> bool {
    assert(false, "fn:apply requires first-class functions - not supported in Noir");
    false
}

/// fn:filter - CANNOT IMPLEMENT (requires first-class functions)
/// Filters a sequence using a predicate function.
pub fn stub_fnfilter() -> bool {
    assert(false, "fn:filter requires first-class functions - not supported in Noir");
    false
}

/// fn:fold-left - CANNOT IMPLEMENT (requires first-class functions)
/// Reduces a sequence from left to right using a function.
pub fn stub_fnfold_left() -> bool {
    assert(false, "fn:fold-left requires first-class functions - not supported in Noir");
    false
}

/// fn:fold-right - CANNOT IMPLEMENT (requires first-class functions)
/// Reduces a sequence from right to left using a function.
pub fn stub_fnfold_right() -> bool {
    assert(false, "fn:fold-right requires first-class functions - not supported in Noir");
    false
}

/// fn:for-each - CANNOT IMPLEMENT (requires first-class functions)
/// Applies a function to each item in a sequence.
pub fn stub_fnfor_each() -> bool {
    assert(false, "fn:for-each requires first-class functions - not supported in Noir");
    false
}

/// fn:for-each-pair - CANNOT IMPLEMENT (requires first-class functions)
/// Applies a function to pairs of items from two sequences.
pub fn stub_fnfor_each_pair() -> bool {
    assert(false, "fn:for-each-pair requires first-class functions - not supported in Noir");
    false
}

// ============================================================================
// INTROSPECTION: Function Introspection
// ============================================================================

/// fn:function-arity - CANNOT IMPLEMENT (requires runtime reflection)
/// Returns the arity (number of arguments) of a function.
pub fn stub_fnfunction_arity() -> bool {
    assert(false, "fn:function-arity requires runtime reflection - not supported in Noir");
    false
}

/// fn:function-lookup - CANNOT IMPLEMENT (requires runtime reflection)
/// Looks up a function by name and arity.
pub fn stub_fnfunction_lookup() -> bool {
    assert(false, "fn:function-lookup requires runtime reflection - not supported in Noir");
    false
}

/// fn:function-name - CANNOT IMPLEMENT (requires runtime reflection)
/// Returns the name of a function.
pub fn stub_fnfunction_name() -> bool {
    assert(false, "fn:function-name requires runtime reflection - not supported in Noir");
    false
}

// ============================================================================
// EXTERNAL: External Resource Access
// ============================================================================

/// fn:available-environment-variables - CANNOT IMPLEMENT (requires OS access)
/// Returns a list of available environment variable names.
pub fn stub_fnavailable_environment_variables() -> bool {
    assert(false, "fn:available-environment-variables requires OS access - not available in ZK");
    false
}

/// fn:collection - CANNOT IMPLEMENT (requires external resources)
/// Returns a sequence of nodes from a collection.
pub fn stub_fncollection() -> bool {
    assert(false, "fn:collection requires external resources - not available in ZK");
    false
}

/// fn:doc - CANNOT IMPLEMENT (requires external resources)
/// Returns a document node from a URI.
pub fn stub_fndoc() -> bool {
    assert(false, "fn:doc requires external resources - not available in ZK");
    false
}

/// fn:doc-available - CANNOT IMPLEMENT (requires external resources)
/// Returns true if a document is available at the given URI.
pub fn stub_fndoc_available() -> bool {
    assert(false, "fn:doc-available requires external resources - not available in ZK");
    false
}

/// fn:environment-variable - CANNOT IMPLEMENT (requires OS access)
/// Returns the value of an environment variable.
pub fn stub_fnenvironment_variable() -> bool {
    assert(false, "fn:environment-variable requires OS access - not available in ZK");
    false
}

/// fn:json-doc - PARTIAL (URL loading not available)
/// Use JSON512b::parse_json_from_string() for JSON parsing instead.
pub fn stub_fnjson_doc() -> bool {
    assert(
        false,
        "fn:json-doc requires URL loading - use JSON512b::parse_json_from_string() instead",
    );
    false
}

/// fn:load-xquery-module - CANNOT IMPLEMENT (requires external resources)
/// Loads an XQuery module from a URI.
pub fn stub_fnload_xquery_module() -> bool {
    assert(false, "fn:load-xquery-module requires external resources - not available in ZK");
    false
}

/// fn:unparsed-text - CANNOT IMPLEMENT (requires external resources)
/// Returns the content of a text file.
pub fn stub_fnunparsed_text() -> bool {
    assert(false, "fn:unparsed-text requires external resources - not available in ZK");
    false
}

/// fn:unparsed-text-available - CANNOT IMPLEMENT (requires external resources)
/// Returns true if a text file is available at the given URI.
pub fn stub_fnunparsed_text_available() -> bool {
    assert(false, "fn:unparsed-text-available requires external resources - not available in ZK");
    false
}

/// fn:unparsed-text-lines - CANNOT IMPLEMENT (requires external resources)
/// Returns the lines of a text file.
pub fn stub_fnunparsed_text_lines() -> bool {
    assert(false, "fn:unparsed-text-lines requires external resources - not available in ZK");
    false
}

// ============================================================================
// CONTEXT: Runtime Context Functions
// ============================================================================

/// fn:current-date - CANNOT IMPLEMENT (requires system clock)
/// Returns the current date. In ZK, there is no system clock.
/// Workaround: Pass date as a public or private input to the circuit.
pub fn stub_fncurrent_date() -> bool {
    assert(false, "fn:current-date requires system clock - pass date as circuit input instead");
    false
}

/// fn:current-dateTime - CANNOT IMPLEMENT (requires system clock)
/// Returns the current date and time. In ZK, there is no system clock.
/// Workaround: Pass datetime as a public or private input to the circuit.
pub fn stub_fncurrent_datetime() -> bool {
    assert(
        false,
        "fn:current-dateTime requires system clock - pass datetime as circuit input instead",
    );
    false
}

/// fn:current-time - CANNOT IMPLEMENT (requires system clock)
/// Returns the current time. In ZK, there is no system clock.
/// Workaround: Pass time as a public or private input to the circuit.
pub fn stub_fncurrent_time() -> bool {
    assert(false, "fn:current-time requires system clock - pass time as circuit input instead");
    false
}

/// fn:random-number-generator - CANNOT IMPLEMENT (ZK must be deterministic)
/// Returns a random number generator. ZK proofs must be deterministic.
pub fn stub_fnrandom_number_generator() -> bool {
    assert(false, "fn:random-number-generator - ZK proofs must be deterministic");
    false
}

/// fn:static-base-uri - CANNOT IMPLEMENT (requires execution context)
/// Returns the static base URI from the static context.
pub fn stub_fnstatic_base_uri() -> bool {
    assert(false, "fn:static-base-uri requires execution context - not available in ZK");
    false
}

// ============================================================================
// XML_DOM: XML and DOM Operations
// ============================================================================

/// fn:base-uri - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the base URI of a node.
pub fn stub_fnbase_uri() -> bool {
    assert(false, "fn:base-uri requires XML DOM - not available in Noir");
    false
}

/// fn:document-uri - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the document URI of a node.
pub fn stub_fndocument_uri() -> bool {
    assert(false, "fn:document-uri requires XML DOM - not available in Noir");
    false
}

/// fn:element-with-id - CANNOT IMPLEMENT (requires XML DOM)
/// Returns elements with the given ID.
pub fn stub_fnelement_with_id() -> bool {
    assert(false, "fn:element-with-id requires XML DOM - not available in Noir");
    false
}

/// fn:generate-id - CANNOT IMPLEMENT (requires XML DOM)
/// Generates a unique identifier for a node.
pub fn stub_fngenerate_id() -> bool {
    assert(false, "fn:generate-id requires XML DOM - not available in Noir");
    false
}

/// fn:has-children - CANNOT IMPLEMENT (requires XML DOM)
/// Returns true if the node has children.
pub fn stub_fnhas_children() -> bool {
    assert(false, "fn:has-children requires XML DOM - not available in Noir");
    false
}

/// fn:id - CANNOT IMPLEMENT (requires XML DOM)
/// Returns elements with the given ID.
pub fn stub_fnid() -> bool {
    assert(false, "fn:id requires XML DOM - not available in Noir");
    false
}

/// fn:idref - CANNOT IMPLEMENT (requires XML DOM)
/// Returns elements with IDREF attributes matching the given ID.
pub fn stub_fnidref() -> bool {
    assert(false, "fn:idref requires XML DOM - not available in Noir");
    false
}

/// fn:in-scope-prefixes - CANNOT IMPLEMENT (requires XML DOM)
/// Returns namespace prefixes in scope for an element.
pub fn stub_fnin_scope_prefixes() -> bool {
    assert(false, "fn:in-scope-prefixes requires XML DOM - not available in Noir");
    false
}

/// fn:innermost - CANNOT IMPLEMENT (requires XML DOM)
/// Returns nodes that have no descendants in the sequence.
pub fn stub_fninnermost() -> bool {
    assert(false, "fn:innermost requires XML DOM - not available in Noir");
    false
}

/// fn:json-to-xml - CANNOT IMPLEMENT (requires XML construction)
/// Converts JSON to XML representation.
pub fn stub_fnjson_to_xml() -> bool {
    assert(false, "fn:json-to-xml requires XML construction - not available in Noir");
    false
}

/// fn:lang - CANNOT IMPLEMENT (requires XML DOM)
/// Tests whether the language of a node matches a pattern.
pub fn stub_fnlang() -> bool {
    assert(false, "fn:lang requires XML DOM - not available in Noir");
    false
}

/// fn:local-name - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the local name of a node.
pub fn stub_fnlocal_name() -> bool {
    assert(false, "fn:local-name requires XML DOM - not available in Noir");
    false
}

/// fn:name - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the name of a node.
pub fn stub_fnname() -> bool {
    assert(false, "fn:name requires XML DOM - not available in Noir");
    false
}

/// fn:namespace-uri - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the namespace URI of a node.
pub fn stub_fnnamespace_uri() -> bool {
    assert(false, "fn:namespace-uri requires XML DOM - not available in Noir");
    false
}

/// fn:namespace-uri-for-prefix - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the namespace URI for a prefix.
pub fn stub_fnnamespace_uri_for_prefix() -> bool {
    assert(false, "fn:namespace-uri-for-prefix requires XML DOM - not available in Noir");
    false
}

/// fn:nilled - CANNOT IMPLEMENT (requires XML DOM)
/// Returns true if a node is nilled.
pub fn stub_fnnilled() -> bool {
    assert(false, "fn:nilled requires XML DOM - not available in Noir");
    false
}

/// fn:node-name - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the name of a node as a QName.
pub fn stub_fnnode_name() -> bool {
    assert(false, "fn:node-name requires XML DOM - not available in Noir");
    false
}

/// fn:outermost - CANNOT IMPLEMENT (requires XML DOM)
/// Returns nodes that have no ancestors in the sequence.
pub fn stub_fnoutermost() -> bool {
    assert(false, "fn:outermost requires XML DOM - not available in Noir");
    false
}

/// fn:parse-xml - CANNOT IMPLEMENT (requires XML parser)
/// Parses an XML string into a document node.
pub fn stub_fnparse_xml() -> bool {
    assert(false, "fn:parse-xml requires XML parser - not available in Noir");
    false
}

/// fn:parse-xml-fragment - CANNOT IMPLEMENT (requires XML parser)
/// Parses an XML fragment string.
pub fn stub_fnparse_xml_fragment() -> bool {
    assert(false, "fn:parse-xml-fragment requires XML parser - not available in Noir");
    false
}

/// fn:path - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the path to a node.
pub fn stub_fnpath() -> bool {
    assert(false, "fn:path requires XML DOM - not available in Noir");
    false
}

/// fn:root - CANNOT IMPLEMENT (requires XML DOM)
/// Returns the root of the tree containing a node.
pub fn stub_fnroot() -> bool {
    assert(false, "fn:root requires XML DOM - not available in Noir");
    false
}

/// fn:serialize - CANNOT IMPLEMENT (requires XML serialization)
/// Serializes a node to a string.
pub fn stub_fnserialize() -> bool {
    assert(false, "fn:serialize requires XML serialization - not available in Noir");
    false
}

/// fn:xml-to-json - CANNOT IMPLEMENT (requires XML DOM)
/// Converts XML to JSON representation.
pub fn stub_fnxml_to_json() -> bool {
    assert(false, "fn:xml-to-json requires XML DOM - not available in Noir");
    false
}

// ============================================================================
// COLLATION: Collation Functions
// ============================================================================

/// fn:collation-key - CANNOT IMPLEMENT (requires locale-aware collation)
/// Returns a collation key for locale-aware string comparison.
pub fn stub_fncollation_key() -> bool {
    assert(false, "fn:collation-key requires locale-aware collation - not available in Noir");
    false
}

// ============================================================================
// FORMAT: Formatting Functions (limited support)
// ============================================================================

/// fn:format-date - CANNOT IMPLEMENT (requires runtime string creation)
/// Formats a date according to a picture string.
pub fn stub_fnformat_date() -> bool {
    assert(false, "fn:format-date requires runtime string creation - not supported in Noir");
    false
}

/// fn:format-dateTime - CANNOT IMPLEMENT (requires runtime string creation)
/// Formats a dateTime according to a picture string.
pub fn stub_fnformat_datetime() -> bool {
    assert(false, "fn:format-dateTime requires runtime string creation - not supported in Noir");
    false
}

/// fn:format-integer - CANNOT IMPLEMENT (requires runtime string creation)
/// Formats an integer according to a picture string.
pub fn stub_fnformat_integer() -> bool {
    assert(false, "fn:format-integer requires runtime string creation - not supported in Noir");
    false
}

/// fn:format-number - CANNOT IMPLEMENT (requires runtime string creation)
/// Formats a number according to a picture string.
pub fn stub_fnformat_number() -> bool {
    assert(false, "fn:format-number requires runtime string creation - not supported in Noir");
    false
}

/// fn:format-time - CANNOT IMPLEMENT (requires runtime string creation)
/// Formats a time according to a picture string.
pub fn stub_fnformat_time() -> bool {
    assert(false, "fn:format-time requires runtime string creation - not supported in Noir");
    false
}

// ============================================================================
// PARTIAL: Partial Implementations Available
// ============================================================================

/// fn:normalize-unicode - PARTIAL (ASCII works, Unicode normalization not supported)
/// For ASCII strings, no normalization is needed.
/// For Unicode, normalization forms (NFC, NFD, etc.) are not supported.
pub fn stub_fnnormalize_unicode() -> bool {
    assert(
        false,
        "fn:normalize-unicode - ASCII works directly, Unicode normalization not supported",
    );
    false
}

/// fn:parse-json - PARTIAL IMPLEMENTATION
/// JSON parsing is available via JSON512b::parse_json_from_string() and similar type aliases.
/// This stub exists for XPath conformance tests that require the exact fn:parse-json signature.
pub fn stub_fnparse_json() -> bool {
    assert(false, "fn:parse-json - use JSON512b::parse_json_from_string() for JSON parsing");
    false
}

/// fn:resolve-QName - PARTIAL (requires namespace context)
/// Resolves a QName using namespace bindings from an element.
pub fn stub_fnresolve_qname() -> bool {
    assert(false, "fn:resolve-QName requires namespace context - use fn_qname() instead");
    false
}

/// fn:resolve-uri - CANNOT IMPLEMENT (requires URI resolution)
/// Resolves a relative URI against a base URI.
pub fn stub_fnresolve_uri() -> bool {
    assert(false, "fn:resolve-uri requires URI resolution logic - not implemented");
    false
}

// ============================================================================
// TRANSFORM: Transformation Functions
// ============================================================================

/// fn:transform - CANNOT IMPLEMENT (requires XSLT processor)
/// Applies an XSLT transformation.
pub fn stub_fntransform() -> bool {
    assert(false, "fn:transform requires XSLT processor - not available in Noir");
    false
}

// ============================================================================
// OPERATORS: Node and Sequence Operators
// ============================================================================

/// op:bang - CANNOT IMPLEMENT (requires sequence mapping)
/// The simple map operator (!) for sequences.
pub fn stub_opbang() -> bool {
    assert(false, "op:bang requires sequence mapping - use explicit loops instead");
    false
}

/// op:concat - PARTIAL (string concat requires fixed output size)
/// Concatenates sequences. For strings, use BoundedVec or fixed arrays.
pub fn stub_opconcat() -> bool {
    assert(false, "op:concat - use explicit array operations for sequence concatenation");
    false
}

/// op:is-same-node - CANNOT IMPLEMENT (requires XML DOM)
/// Returns true if two expressions evaluate to the same node.
pub fn stub_opis_same_node() -> bool {
    assert(false, "op:is-same-node requires XML DOM - not available in Noir");
    false
}

/// op:node-after - CANNOT IMPLEMENT (requires XML DOM)
/// Returns true if the first node comes after the second in document order.
pub fn stub_opnode_after() -> bool {
    assert(false, "op:node-after requires XML DOM - not available in Noir");
    false
}

/// op:node-before - CANNOT IMPLEMENT (requires XML DOM)
/// Returns true if the first node comes before the second in document order.
pub fn stub_opnode_before() -> bool {
    assert(false, "op:node-before requires XML DOM - not available in Noir");
    false
}

/// op:same-key - CANNOT IMPLEMENT (requires map key comparison)
/// Returns true if two map keys are the same.
pub fn stub_opsame_key() -> bool {
    assert(false, "op:same-key - use explicit key comparison instead");
    false
}

// ============================================================================
// XQUERY PRODUCTIONS: XQuery-specific Grammar (prod:*)
// These are XQuery grammar productions, not XPath functions.
// They represent parsing constructs and are not meant to be called directly.
// ============================================================================

/// prod:Annotation - XQuery-only (annotations for functions/variables)
pub fn stub_prodannotation() -> bool {
    assert(false, "prod:Annotation is XQuery-specific - not applicable to XPath");
    false
}

/// prod:ArrayTest - XQuery-only (type test for arrays)
pub fn stub_prodarraytest() -> bool {
    assert(false, "prod:ArrayTest is XQuery-specific");
    false
}

/// prod:ArrowPostfix - Arrow operator (requires first-class functions)
pub fn stub_prodarrowpostfix() -> bool {
    assert(false, "prod:ArrowPostfix requires first-class functions");
    false
}

/// prod:AxisStep - XPath axis navigation (requires XML DOM)
pub fn stub_prodaxisstep() -> bool {
    assert(false, "prod:AxisStep requires XML DOM");
    false
}

/// prod:AxisStep.abbr - Abbreviated axis step (requires XML DOM)
pub fn stub_prodaxisstep_abbr() -> bool {
    assert(false, "prod:AxisStep.abbr requires XML DOM");
    false
}

/// prod:AxisStep.ancestor - Ancestor axis (requires XML DOM)
pub fn stub_prodaxisstep_ancestor() -> bool {
    assert(false, "prod:AxisStep.ancestor requires XML DOM");
    false
}

/// prod:AxisStep.ancestor-or-self - Ancestor-or-self axis (requires XML DOM)
pub fn stub_prodaxisstep_ancestor_or_self() -> bool {
    assert(false, "prod:AxisStep.ancestor-or-self requires XML DOM");
    false
}

/// prod:AxisStep.following - Following axis (requires XML DOM)
pub fn stub_prodaxisstep_following() -> bool {
    assert(false, "prod:AxisStep.following requires XML DOM");
    false
}

/// prod:AxisStep.following-sibling - Following-sibling axis (requires XML DOM)
pub fn stub_prodaxisstep_following_sibling() -> bool {
    assert(false, "prod:AxisStep.following-sibling requires XML DOM");
    false
}

/// prod:AxisStep.preceding - Preceding axis (requires XML DOM)
pub fn stub_prodaxisstep_preceding() -> bool {
    assert(false, "prod:AxisStep.preceding requires XML DOM");
    false
}

/// prod:AxisStep.preceding-sibling - Preceding-sibling axis (requires XML DOM)
pub fn stub_prodaxisstep_preceding_sibling() -> bool {
    assert(false, "prod:AxisStep.preceding-sibling requires XML DOM");
    false
}

/// prod:AxisStep.unabbr - Unabbreviated axis step (requires XML DOM)
pub fn stub_prodaxisstep_unabbr() -> bool {
    assert(false, "prod:AxisStep.unabbr requires XML DOM");
    false
}

/// prod:BaseURIDecl - XQuery prolog declaration
pub fn stub_prodbaseuridecl() -> bool {
    assert(false, "prod:BaseURIDecl is XQuery-specific");
    false
}

/// prod:BoundarySpaceDecl - XQuery prolog declaration
pub fn stub_prodboundaryspacedecl() -> bool {
    assert(false, "prod:BoundarySpaceDecl is XQuery-specific");
    false
}

/// prod:CastExpr - Type casting (partially supported via xpath_xs module)
pub fn stub_prodcastexpr() -> bool {
    assert(false, "prod:CastExpr - use xpath_xs module for type conversions");
    false
}

/// prod:CastExpr.derived - Derived type casting
pub fn stub_prodcastexpr_derived() -> bool {
    assert(false, "prod:CastExpr.derived - derived types not supported");
    false
}

/// prod:CastExpr.schema - Schema-based type casting
pub fn stub_prodcastexpr_schema() -> bool {
    assert(false, "prod:CastExpr.schema - schema types not supported");
    false
}

/// prod:CastableExpr - Type castability test
pub fn stub_prodcastableexpr() -> bool {
    assert(false, "prod:CastableExpr - castability tests not supported");
    false
}

/// prod:Comment - XQuery comment constructor (requires XML)
pub fn stub_prodcomment() -> bool {
    assert(false, "prod:Comment requires XML construction");
    false
}

/// prod:CompAttrConstructor - Computed attribute constructor (requires XML)
pub fn stub_prodcompattrconstructor() -> bool {
    assert(false, "prod:CompAttrConstructor requires XML construction");
    false
}

/// prod:CompCommentConstructor - Computed comment constructor (requires XML)
pub fn stub_prodcompcommentconstructor() -> bool {
    assert(false, "prod:CompCommentConstructor requires XML construction");
    false
}

/// prod:CompDocConstructor - Computed document constructor (requires XML)
pub fn stub_prodcompdocconstructor() -> bool {
    assert(false, "prod:CompDocConstructor requires XML construction");
    false
}

/// prod:CompElemConstructor - Computed element constructor (requires XML)
pub fn stub_prodcompelemconstructor() -> bool {
    assert(false, "prod:CompElemConstructor requires XML construction");
    false
}

/// prod:CompNamespaceConstructor - Computed namespace constructor (requires XML)
pub fn stub_prodcompnamespaceconstructor() -> bool {
    assert(false, "prod:CompNamespaceConstructor requires XML construction");
    false
}

/// prod:CompPIConstructor - Computed PI constructor (requires XML)
pub fn stub_prodcomppiconstructor() -> bool {
    assert(false, "prod:CompPIConstructor requires XML construction");
    false
}

/// prod:CompTextConstructor - Computed text constructor (requires XML)
pub fn stub_prodcomptextconstructor() -> bool {
    assert(false, "prod:CompTextConstructor requires XML construction");
    false
}

/// prod:ConstructionDecl - XQuery prolog declaration
pub fn stub_prodconstructiondecl() -> bool {
    assert(false, "prod:ConstructionDecl is XQuery-specific");
    false
}

/// prod:ConstructionDecl.schema - Schema-based construction declaration
pub fn stub_prodconstructiondecl_schema() -> bool {
    assert(false, "prod:ConstructionDecl.schema is XQuery-specific");
    false
}

/// prod:ContextItemDecl - XQuery context item declaration
pub fn stub_prodcontextitemdecl() -> bool {
    assert(false, "prod:ContextItemDecl is XQuery-specific");
    false
}

/// prod:ContextItemExpr - Context item expression (.) in XPath
pub fn stub_prodcontextitemexpr() -> bool {
    assert(false, "prod:ContextItemExpr requires evaluation context");
    false
}

/// prod:CopyNamespacesDecl - XQuery prolog declaration
pub fn stub_prodcopynamespacesdecl() -> bool {
    assert(false, "prod:CopyNamespacesDecl is XQuery-specific");
    false
}

/// prod:CountClause - FLWOR count clause (XQuery)
pub fn stub_prodcountclause() -> bool {
    assert(false, "prod:CountClause is XQuery FLWOR-specific");
    false
}

/// prod:CurlyArrayConstructor - Array constructor using braces
pub fn stub_prodcurlyarrayconstructor() -> bool {
    assert(false, "prod:CurlyArrayConstructor - use Noir arrays instead");
    false
}

/// prod:DecimalFormatDecl - XQuery decimal format declaration
pub fn stub_proddecimalformatdecl() -> bool {
    assert(false, "prod:DecimalFormatDecl is XQuery-specific");
    false
}

/// prod:DefaultCollationDecl - XQuery default collation declaration
pub fn stub_proddefaultcollationdecl() -> bool {
    assert(false, "prod:DefaultCollationDecl is XQuery-specific");
    false
}

/// prod:DefaultNamespaceDecl - XQuery default namespace declaration
pub fn stub_proddefaultnamespacedecl() -> bool {
    assert(false, "prod:DefaultNamespaceDecl is XQuery-specific");
    false
}

/// prod:DirAttributeList - Direct attribute list (requires XML)
pub fn stub_proddirattributelist() -> bool {
    assert(false, "prod:DirAttributeList requires XML construction");
    false
}

/// prod:DirElemConstructor - Direct element constructor (requires XML)
pub fn stub_proddirelemconstructor() -> bool {
    assert(false, "prod:DirElemConstructor requires XML construction");
    false
}

/// prod:DirElemContent - Direct element content (requires XML)
pub fn stub_proddirelemcontent() -> bool {
    assert(false, "prod:DirElemContent requires XML construction");
    false
}

/// prod:DirElemContent.namespace - Namespace in element content
pub fn stub_proddirelemcontent_namespace() -> bool {
    assert(false, "prod:DirElemContent.namespace requires XML construction");
    false
}

/// prod:DirElemContent.whitespace - Whitespace handling in element content
pub fn stub_proddirelemcontent_whitespace() -> bool {
    assert(false, "prod:DirElemContent.whitespace requires XML construction");
    false
}

/// prod:DirectConstructor - Direct XML constructor
pub fn stub_proddirectconstructor() -> bool {
    assert(false, "prod:DirectConstructor requires XML construction");
    false
}

/// prod:EQName - Extended QName handling
pub fn stub_prodeqname() -> bool {
    assert(false, "prod:EQName - use XsdQName type instead");
    false
}

/// prod:EmptyOrderDecl - XQuery empty order declaration
pub fn stub_prodemptyorderdecl() -> bool {
    assert(false, "prod:EmptyOrderDecl is XQuery-specific");
    false
}

/// prod:ExtensionExpr - Extension expression
pub fn stub_prodextensionexpr() -> bool {
    assert(false, "prod:ExtensionExpr - extensions not supported");
    false
}

/// prod:FLWORExpr - FLWOR expression (XQuery)
pub fn stub_prodflworexpr() -> bool {
    assert(false, "prod:FLWORExpr is XQuery-specific - use Noir for loops");
    false
}

/// prod:ForClause - FLWOR for clause (XQuery)
pub fn stub_prodforclause() -> bool {
    assert(false, "prod:ForClause is XQuery FLWOR-specific");
    false
}

/// prod:FunctionCall - Function call expression
pub fn stub_prodfunctioncall() -> bool {
    assert(false, "prod:FunctionCall - use direct function calls in Noir");
    false
}

/// prod:FunctionDecl - Function declaration (XQuery)
pub fn stub_prodfunctiondecl() -> bool {
    assert(false, "prod:FunctionDecl is XQuery-specific - define Noir functions");
    false
}

/// prod:GeneralComp.eq - General comparison (=)
pub fn stub_prodgeneralcomp_eq() -> bool {
    assert(false, "prod:GeneralComp.eq - use value comparison operators instead");
    false
}

/// prod:GeneralComp.ge - General comparison (>=)
pub fn stub_prodgeneralcomp_ge() -> bool {
    assert(false, "prod:GeneralComp.ge - use value comparison operators instead");
    false
}

/// prod:GeneralComp.gt - General comparison (>)
pub fn stub_prodgeneralcomp_gt() -> bool {
    assert(false, "prod:GeneralComp.gt - use value comparison operators instead");
    false
}

/// prod:GeneralComp.le - General comparison (<=)
pub fn stub_prodgeneralcomp_le() -> bool {
    assert(false, "prod:GeneralComp.le - use value comparison operators instead");
    false
}

/// prod:GeneralComp.lt - General comparison (<)
pub fn stub_prodgeneralcomp_lt() -> bool {
    assert(false, "prod:GeneralComp.lt - use value comparison operators instead");
    false
}

/// prod:GeneralComp.ne - General comparison (!=)
pub fn stub_prodgeneralcomp_ne() -> bool {
    assert(false, "prod:GeneralComp.ne - use value comparison operators instead");
    false
}

/// prod:GroupByClause - FLWOR group by clause (XQuery)
pub fn stub_prodgroupbyclause() -> bool {
    assert(false, "prod:GroupByClause is XQuery FLWOR-specific");
    false
}

/// prod:IfExpr - Conditional expression
pub fn stub_prodifexpr() -> bool {
    assert(false, "prod:IfExpr - use Noir if-else expressions");
    false
}

/// prod:InlineFunctionExpr - Inline function expression
pub fn stub_prodinlinefunctionexpr() -> bool {
    assert(false, "prod:InlineFunctionExpr requires first-class functions");
    false
}

/// prod:InstanceofExpr - Instance of expression
pub fn stub_prodinstanceofexpr() -> bool {
    assert(false, "prod:InstanceofExpr - type checking is compile-time in Noir");
    false
}

/// prod:LetClause - FLWOR let clause (XQuery)
pub fn stub_prodletclause() -> bool {
    assert(false, "prod:LetClause is XQuery FLWOR-specific - use Noir let");
    false
}

/// prod:Literal - Literal values
pub fn stub_prodliteral() -> bool {
    assert(false, "prod:Literal - use Noir literals directly");
    false
}

/// prod:Lookup - Postfix lookup operator
pub fn stub_prodlookup() -> bool {
    assert(false, "prod:Lookup - use array indexing in Noir");
    false
}

/// prod:MapConstructor - Map constructor
pub fn stub_prodmapconstructor() -> bool {
    assert(false, "prod:MapConstructor - use structs or arrays in Noir");
    false
}

/// prod:MapTest - Map type test
pub fn stub_prodmaptest() -> bool {
    assert(false, "prod:MapTest - type testing not supported");
    false
}

/// prod:ModuleImport - XQuery module import
pub fn stub_prodmoduleimport() -> bool {
    assert(false, "prod:ModuleImport is XQuery-specific - use Noir mod/use");
    false
}

/// prod:NameTest - Name test in XPath
pub fn stub_prodnametest() -> bool {
    assert(false, "prod:NameTest requires XML DOM");
    false
}

/// prod:NamedFunctionRef - Named function reference
pub fn stub_prodnamedfunctionref() -> bool {
    assert(false, "prod:NamedFunctionRef requires first-class functions");
    false
}

/// prod:NamespaceDecl - XQuery namespace declaration
pub fn stub_prodnamespacedecl() -> bool {
    assert(false, "prod:NamespaceDecl is XQuery-specific");
    false
}

/// prod:NodeTest - Node test in XPath
pub fn stub_prodnodetest() -> bool {
    assert(false, "prod:NodeTest requires XML DOM");
    false
}

/// prod:OptionDecl - XQuery option declaration
pub fn stub_prodoptiondecl() -> bool {
    assert(false, "prod:OptionDecl is XQuery-specific");
    false
}

/// prod:OptionDecl.serialization - Serialization option
pub fn stub_prodoptiondecl_serialization() -> bool {
    assert(false, "prod:OptionDecl.serialization is XQuery-specific");
    false
}

/// prod:OrExpr - Or expression
pub fn stub_prodorexpr() -> bool {
    assert(false, "prod:OrExpr - use Noir | operator or logical_or function");
    false
}

/// prod:OrderByClause - FLWOR order by clause (XQuery)
pub fn stub_prodorderbyclause() -> bool {
    assert(false, "prod:OrderByClause is XQuery FLWOR-specific");
    false
}

/// prod:OrderingModeDecl - XQuery ordering mode declaration
pub fn stub_prodorderingmodedecl() -> bool {
    assert(false, "prod:OrderingModeDecl is XQuery-specific");
    false
}

/// prod:ParenthesizedExpr - Parenthesized expression
pub fn stub_prodparenthesizedexpr() -> bool {
    assert(false, "prod:ParenthesizedExpr - use parentheses in Noir");
    false
}

/// prod:PathExpr - Path expression (requires XML DOM)
pub fn stub_prodpathexpr() -> bool {
    assert(false, "prod:PathExpr requires XML DOM");
    false
}

/// prod:PositionalVar - Positional variable in for
pub fn stub_prodpositionalvar() -> bool {
    assert(false, "prod:PositionalVar - use enumeration in Noir");
    false
}

/// prod:Predicate - Predicate expression
pub fn stub_prodpredicate() -> bool {
    assert(false, "prod:Predicate - use conditional expressions in Noir");
    false
}

/// prod:QuantifiedExpr - Quantified expression (some/every)
pub fn stub_prodquantifiedexpr() -> bool {
    assert(false, "prod:QuantifiedExpr - use loops with boolean accumulators");
    false
}

/// prod:ReturnClause - FLWOR return clause (XQuery)
pub fn stub_prodreturnclause() -> bool {
    assert(false, "prod:ReturnClause is XQuery FLWOR-specific");
    false
}

/// prod:SchemaImport - XQuery schema import
pub fn stub_prodschemaimport() -> bool {
    assert(false, "prod:SchemaImport is XQuery-specific");
    false
}

/// prod:SequenceType - Sequence type
pub fn stub_prodsequencetype() -> bool {
    assert(false, "prod:SequenceType - use Noir types");
    false
}

/// prod:StepExpr - Step expression (requires XML DOM)
pub fn stub_prodstepexpr() -> bool {
    assert(false, "prod:StepExpr requires XML DOM");
    false
}

/// prod:StringConstructor - String constructor
pub fn stub_prodstringconstructor() -> bool {
    assert(false, "prod:StringConstructor - use Noir string literals");
    false
}

/// prod:SwitchExpr - Switch expression (XQuery)
pub fn stub_prodswitchexpr() -> bool {
    assert(false, "prod:SwitchExpr - use if-else chains in Noir");
    false
}

/// prod:TreatExpr - Treat expression
pub fn stub_prodtreatexpr() -> bool {
    assert(false, "prod:TreatExpr - type treatment not supported");
    false
}

/// prod:TryCatchExpr - Try-catch expression (XQuery)
pub fn stub_prodtrycatchexpr() -> bool {
    assert(false, "prod:TryCatchExpr - use assertions in Noir");
    false
}

/// prod:TypeswitchExpr - Typeswitch expression (XQuery)
pub fn stub_prodtypeswitchexpr() -> bool {
    assert(false, "prod:TypeswitchExpr is XQuery-specific");
    false
}

/// prod:UnaryLookup - Unary lookup operator
pub fn stub_produnarylookup() -> bool {
    assert(false, "prod:UnaryLookup - use array indexing in Noir");
    false
}

/// prod:UnorderedExpr - Unordered expression
pub fn stub_produnorderedexpr() -> bool {
    assert(false, "prod:UnorderedExpr - ordering hints not applicable");
    false
}

/// prod:ValidateExpr - Validate expression (XQuery)
pub fn stub_prodvalidateexpr() -> bool {
    assert(false, "prod:ValidateExpr is XQuery-specific");
    false
}

/// prod:ValueComp - Value comparison operators
pub fn stub_prodvaluecomp() -> bool {
    assert(false, "prod:ValueComp - use comparison functions from xpath_op");
    false
}

/// prod:VarDecl - Variable declaration (XQuery)
pub fn stub_prodvardecl() -> bool {
    assert(false, "prod:VarDecl is XQuery-specific - use Noir let");
    false
}

/// prod:VarDecl.external - External variable declaration (XQuery)
pub fn stub_prodvardecl_external() -> bool {
    assert(false, "prod:VarDecl.external is XQuery-specific");
    false
}

/// prod:VarDefaultValue - Variable default value (XQuery)
pub fn stub_prodvardefaultvalue() -> bool {
    assert(false, "prod:VarDefaultValue is XQuery-specific");
    false
}

/// prod:VersionDecl - XQuery version declaration
pub fn stub_prodversiondecl() -> bool {
    assert(false, "prod:VersionDecl is XQuery-specific");
    false
}

/// prod:WhereClause - FLWOR where clause (XQuery)
pub fn stub_prodwhereclause() -> bool {
    assert(false, "prod:WhereClause is XQuery FLWOR-specific - use Noir if");
    false
}

/// prod:WindowClause - FLWOR window clause (XQuery)
pub fn stub_prodwindowclause() -> bool {
    assert(false, "prod:WindowClause is XQuery FLWOR-specific");
    false
}

// ============================================================================
// AUTO-GENERATED STUBS: Additional stubs needed by test packages
// ============================================================================
// These stubs were auto-generated because test packages reference them.
// Feel free to add documentation explaining why each cannot be implemented.

/// fn:QName - stub for unimplemented fn function
pub fn stub_fnqname() -> bool {
    assert(false, "fn:QName is not yet implemented");
    false
}

/// fn:adjust-date-to-timezone - stub for unimplemented fn function
pub fn stub_fnadjust_date_to_timezone() -> bool {
    assert(false, "fn:adjust-date-to-timezone is not yet implemented");
    false
}

/// fn:adjust-dateTime-to-timezone - stub for unimplemented fn function
pub fn stub_fnadjust_datetime_to_timezone() -> bool {
    assert(false, "fn:adjust-dateTime-to-timezone is not yet implemented");
    false
}

/// fn:adjust-time-to-timezone - stub for unimplemented fn function
pub fn stub_fnadjust_time_to_timezone() -> bool {
    assert(false, "fn:adjust-time-to-timezone is not yet implemented");
    false
}

/// fn:avg - stub for unimplemented fn function
pub fn stub_fnavg() -> bool {
    assert(false, "fn:avg is not yet implemented");
    false
}

/// fn:boolean - stub for unimplemented fn function
pub fn stub_fnboolean() -> bool {
    assert(false, "fn:boolean is not yet implemented");
    false
}

/// fn:codepoint-equal - stub for unimplemented fn function
pub fn stub_fncodepoint_equal() -> bool {
    assert(false, "fn:codepoint-equal is not yet implemented");
    false
}

/// fn:codepoints-to-string - stub for unimplemented fn function
pub fn stub_fncodepoints_to_string() -> bool {
    assert(false, "fn:codepoints-to-string is not yet implemented");
    false
}

/// fn:compare - stub for unimplemented fn function
pub fn stub_fncompare() -> bool {
    assert(false, "fn:compare is not yet implemented");
    false
}

/// fn:concat - stub for unimplemented fn function
pub fn stub_fnconcat() -> bool {
    assert(false, "fn:concat is not yet implemented");
    false
}

/// fn:contains-token - stub for unimplemented fn function
pub fn stub_fncontains_token() -> bool {
    assert(false, "fn:contains-token is not yet implemented");
    false
}

/// fn:count - stub for unimplemented fn function
pub fn stub_fncount() -> bool {
    assert(false, "fn:count is not yet implemented");
    false
}

/// fn:data - stub for unimplemented fn function
pub fn stub_fndata() -> bool {
    assert(false, "fn:data is not yet implemented");
    false
}

/// fn:dateTime - stub for unimplemented fn function
pub fn stub_fndatetime() -> bool {
    assert(false, "fn:dateTime is not yet implemented");
    false
}

/// fn:day-from-date - stub for unimplemented fn function
pub fn stub_fnday_from_date() -> bool {
    assert(false, "fn:day-from-date is not yet implemented");
    false
}

/// fn:deep-equal - stub for unimplemented fn function
pub fn stub_fndeep_equal() -> bool {
    assert(false, "fn:deep-equal is not yet implemented");
    false
}

/// fn:default-collation - stub for unimplemented fn function
pub fn stub_fndefault_collation() -> bool {
    assert(false, "fn:default-collation is not yet implemented");
    false
}

/// fn:default-language - stub for unimplemented fn function
pub fn stub_fndefault_language() -> bool {
    assert(false, "fn:default-language is not yet implemented");
    false
}

/// fn:distinct-values - stub for unimplemented fn function
pub fn stub_fndistinct_values() -> bool {
    assert(false, "fn:distinct-values is not yet implemented");
    false
}

/// fn:empty - stub for unimplemented fn function
pub fn stub_fnempty() -> bool {
    assert(false, "fn:empty is not yet implemented");
    false
}

/// fn:encode-for-uri - stub for unimplemented fn function
pub fn stub_fnencode_for_uri() -> bool {
    assert(false, "fn:encode-for-uri is not yet implemented");
    false
}

/// fn:error - stub for unimplemented fn function
pub fn stub_fnerror() -> bool {
    assert(false, "fn:error is not yet implemented");
    false
}

/// fn:escape-html-uri - stub for unimplemented fn function
pub fn stub_fnescape_html_uri() -> bool {
    assert(false, "fn:escape-html-uri is not yet implemented");
    false
}

/// fn:exactly-one - stub for unimplemented fn function
pub fn stub_fnexactly_one() -> bool {
    assert(false, "fn:exactly-one is not yet implemented");
    false
}

/// fn:exists - stub for unimplemented fn function
pub fn stub_fnexists() -> bool {
    assert(false, "fn:exists is not yet implemented");
    false
}

/// fn:false - stub for unimplemented fn function
pub fn stub_fnfalse() -> bool {
    assert(false, "fn:false is not yet implemented");
    false
}

/// fn:head - stub for unimplemented fn function
pub fn stub_fnhead() -> bool {
    assert(false, "fn:head is not yet implemented");
    false
}

/// fn:hours-from-time - stub for unimplemented fn function
pub fn stub_fnhours_from_time() -> bool {
    assert(false, "fn:hours-from-time is not yet implemented");
    false
}

/// fn:implicit-timezone - stub for unimplemented fn function
pub fn stub_fnimplicit_timezone() -> bool {
    assert(false, "fn:implicit-timezone is not yet implemented");
    false
}

/// fn:index-of - stub for unimplemented fn function
pub fn stub_fnindex_of() -> bool {
    assert(false, "fn:index-of is not yet implemented");
    false
}

/// fn:insert-before - stub for unimplemented fn function
pub fn stub_fninsert_before() -> bool {
    assert(false, "fn:insert-before is not yet implemented");
    false
}

/// fn:iri-to-uri - stub for unimplemented fn function
pub fn stub_fniri_to_uri() -> bool {
    assert(false, "fn:iri-to-uri is not yet implemented");
    false
}

/// fn:last - stub for unimplemented fn function
pub fn stub_fnlast() -> bool {
    assert(false, "fn:last is not yet implemented");
    false
}

/// fn:local-name-from-QName - stub for unimplemented fn function
pub fn stub_fnlocal_name_from_qname() -> bool {
    assert(false, "fn:local-name-from-QName is not yet implemented");
    false
}

/// fn:lower-case - stub for unimplemented fn function
pub fn stub_fnlower_case() -> bool {
    assert(false, "fn:lower-case is not yet implemented");
    false
}

/// fn:max - stub for unimplemented fn function
pub fn stub_fnmax() -> bool {
    assert(false, "fn:max is not yet implemented");
    false
}

/// fn:min - stub for unimplemented fn function
pub fn stub_fnmin() -> bool {
    assert(false, "fn:min is not yet implemented");
    false
}

/// fn:minutes-from-time - stub for unimplemented fn function
pub fn stub_fnminutes_from_time() -> bool {
    assert(false, "fn:minutes-from-time is not yet implemented");
    false
}

/// fn:month-from-date - stub for unimplemented fn function
pub fn stub_fnmonth_from_date() -> bool {
    assert(false, "fn:month-from-date is not yet implemented");
    false
}

/// fn:months-from-duration - stub for unimplemented fn function
pub fn stub_fnmonths_from_duration() -> bool {
    assert(false, "fn:months-from-duration is not yet implemented");
    false
}

/// fn:namespace-uri-from-QName - stub for unimplemented fn function
pub fn stub_fnnamespace_uri_from_qname() -> bool {
    assert(false, "fn:namespace-uri-from-QName is not yet implemented");
    false
}

/// fn:normalize-space - stub for unimplemented fn function
pub fn stub_fnnormalize_space() -> bool {
    assert(false, "fn:normalize-space is not yet implemented");
    false
}

/// fn:number - stub for unimplemented fn function
pub fn stub_fnnumber() -> bool {
    assert(false, "fn:number is not yet implemented");
    false
}

/// fn:one-or-more - stub for unimplemented fn function
pub fn stub_fnone_or_more() -> bool {
    assert(false, "fn:one-or-more is not yet implemented");
    false
}

/// fn:position - stub for unimplemented fn function
pub fn stub_fnposition() -> bool {
    assert(false, "fn:position is not yet implemented");
    false
}

/// fn:prefix-from-QName - stub for unimplemented fn function
pub fn stub_fnprefix_from_qname() -> bool {
    assert(false, "fn:prefix-from-QName is not yet implemented");
    false
}

/// fn:remove - stub for unimplemented fn function
pub fn stub_fnremove() -> bool {
    assert(false, "fn:remove is not yet implemented");
    false
}

/// fn:reverse - stub for unimplemented fn function
pub fn stub_fnreverse() -> bool {
    assert(false, "fn:reverse is not yet implemented");
    false
}

/// fn:round-half-to-even - stub for unimplemented fn function
pub fn stub_fnround_half_to_even() -> bool {
    assert(false, "fn:round-half-to-even is not yet implemented");
    false
}

/// fn:seconds-from-time - stub for unimplemented fn function
pub fn stub_fnseconds_from_time() -> bool {
    assert(false, "fn:seconds-from-time is not yet implemented");
    false
}

/// fn:sort - stub for unimplemented fn function
pub fn stub_fnsort() -> bool {
    assert(false, "fn:sort is not yet implemented");
    false
}

/// fn:string - stub for unimplemented fn function
pub fn stub_fnstring() -> bool {
    assert(false, "fn:string is not yet implemented");
    false
}

/// fn:string-join - stub for unimplemented fn function
pub fn stub_fnstring_join() -> bool {
    assert(false, "fn:string-join is not yet implemented");
    false
}

/// fn:string-to-codepoints - stub for unimplemented fn function
pub fn stub_fnstring_to_codepoints() -> bool {
    assert(false, "fn:string-to-codepoints is not yet implemented");
    false
}

/// fn:subsequence - stub for unimplemented fn function
pub fn stub_fnsubsequence() -> bool {
    assert(false, "fn:subsequence is not yet implemented");
    false
}

/// fn:substring - stub for unimplemented fn function
pub fn stub_fnsubstring() -> bool {
    assert(false, "fn:substring is not yet implemented");
    false
}

/// fn:substring-after - stub for unimplemented fn function
pub fn stub_fnsubstring_after() -> bool {
    assert(false, "fn:substring-after is not yet implemented");
    false
}

/// fn:substring-before - stub for unimplemented fn function
pub fn stub_fnsubstring_before() -> bool {
    assert(false, "fn:substring-before is not yet implemented");
    false
}

/// fn:sum - stub for unimplemented fn function
pub fn stub_fnsum() -> bool {
    assert(false, "fn:sum is not yet implemented");
    false
}

/// fn:tail - stub for unimplemented fn function
pub fn stub_fntail() -> bool {
    assert(false, "fn:tail is not yet implemented");
    false
}

/// fn:timezone-from-date - stub for unimplemented fn function
pub fn stub_fntimezone_from_date() -> bool {
    assert(false, "fn:timezone-from-date is not yet implemented");
    false
}

/// fn:timezone-from-time - stub for unimplemented fn function
pub fn stub_fntimezone_from_time() -> bool {
    assert(false, "fn:timezone-from-time is not yet implemented");
    false
}

/// fn:tokenize - stub for unimplemented fn function
pub fn stub_fntokenize() -> bool {
    assert(false, "fn:tokenize is not yet implemented");
    false
}

/// fn:trace - stub for unimplemented fn function
pub fn stub_fntrace() -> bool {
    assert(false, "fn:trace is not yet implemented");
    false
}

/// fn:translate - stub for unimplemented fn function
pub fn stub_fntranslate() -> bool {
    assert(false, "fn:translate is not yet implemented");
    false
}

/// fn:true - stub for unimplemented fn function
pub fn stub_fntrue() -> bool {
    assert(false, "fn:true is not yet implemented");
    false
}

/// fn:unordered - stub for unimplemented fn function
pub fn stub_fnunordered() -> bool {
    assert(false, "fn:unordered is not yet implemented");
    false
}

/// fn:upper-case - stub for unimplemented fn function
pub fn stub_fnupper_case() -> bool {
    assert(false, "fn:upper-case is not yet implemented");
    false
}

/// fn:year-from-date - stub for unimplemented fn function
pub fn stub_fnyear_from_date() -> bool {
    assert(false, "fn:year-from-date is not yet implemented");
    false
}

/// fn:years-from-duration - stub for unimplemented fn function
pub fn stub_fnyears_from_duration() -> bool {
    assert(false, "fn:years-from-duration is not yet implemented");
    false
}

/// fn:zero-or-one - stub for unimplemented fn function
pub fn stub_fnzero_or_one() -> bool {
    assert(false, "fn:zero-or-one is not yet implemented");
    false
}

/// op:QName-equal - stub for unimplemented operator
pub fn stub_opqname_equal() -> bool {
    assert(false, "op:QName-equal is not yet implemented");
    false
}

/// op:add-dayTimeDuration-to-date - stub for unimplemented operator
pub fn stub_opadd_daytimeduration_to_date() -> bool {
    assert(false, "op:add-dayTimeDuration-to-date is not yet implemented");
    false
}

/// op:add-dayTimeDuration-to-time - stub for unimplemented operator
pub fn stub_opadd_daytimeduration_to_time() -> bool {
    assert(false, "op:add-dayTimeDuration-to-time is not yet implemented");
    false
}

/// op:add-yearMonthDuration-to-date - stub for unimplemented operator
pub fn stub_opadd_yearmonthduration_to_date() -> bool {
    assert(false, "op:add-yearMonthDuration-to-date is not yet implemented");
    false
}

/// op:add-yearMonthDuration-to-dateTime - stub for unimplemented operator
pub fn stub_opadd_yearmonthduration_to_datetime() -> bool {
    assert(false, "op:add-yearMonthDuration-to-dateTime is not yet implemented");
    false
}

/// op:add-yearMonthDurations - stub for unimplemented operator
pub fn stub_opadd_yearmonthdurations() -> bool {
    assert(false, "op:add-yearMonthDurations is not yet implemented");
    false
}

/// op:anyURI-equal - stub for unimplemented operator
pub fn stub_opanyuri_equal() -> bool {
    assert(false, "op:anyURI-equal is not yet implemented");
    false
}

/// op:anyURI-greater-than - stub for unimplemented operator
pub fn stub_opanyuri_greater_than() -> bool {
    assert(false, "op:anyURI-greater-than is not yet implemented");
    false
}

/// op:anyURI-less-than - stub for unimplemented operator
pub fn stub_opanyuri_less_than() -> bool {
    assert(false, "op:anyURI-less-than is not yet implemented");
    false
}

/// op:base64Binary-equal - stub for unimplemented operator
pub fn stub_opbase64binary_equal() -> bool {
    assert(false, "op:base64Binary-equal is not yet implemented");
    false
}

/// op:base64Binary-greater-than - stub for unimplemented operator
pub fn stub_opbase64binary_greater_than() -> bool {
    assert(false, "op:base64Binary-greater-than is not yet implemented");
    false
}

/// op:base64Binary-less-than - stub for unimplemented operator
pub fn stub_opbase64binary_less_than() -> bool {
    assert(false, "op:base64Binary-less-than is not yet implemented");
    false
}

/// op:concatenate - stub for unimplemented operator
pub fn stub_opconcatenate() -> bool {
    assert(false, "op:concatenate is not yet implemented");
    false
}

/// op:date-equal - stub for unimplemented operator
pub fn stub_opdate_equal() -> bool {
    assert(false, "op:date-equal is not yet implemented");
    false
}

/// op:date-greater-than - stub for unimplemented operator
pub fn stub_opdate_greater_than() -> bool {
    assert(false, "op:date-greater-than is not yet implemented");
    false
}

/// op:date-less-than - stub for unimplemented operator
pub fn stub_opdate_less_than() -> bool {
    assert(false, "op:date-less-than is not yet implemented");
    false
}

/// op:divide-dayTimeDuration - stub for unimplemented operator
pub fn stub_opdivide_daytimeduration() -> bool {
    assert(false, "op:divide-dayTimeDuration is not yet implemented");
    false
}

/// op:divide-dayTimeDuration-by-dayTimeDuration - stub for unimplemented operator
pub fn stub_opdivide_daytimeduration_by_daytimeduration() -> bool {
    assert(false, "op:divide-dayTimeDuration-by-dayTimeDuration is not yet implemented");
    false
}

/// op:divide-yearMonthDuration - stub for unimplemented operator
pub fn stub_opdivide_yearmonthduration() -> bool {
    assert(false, "op:divide-yearMonthDuration is not yet implemented");
    false
}

/// op:divide-yearMonthDuration-by-yearMonthDuration - stub for unimplemented operator
pub fn stub_opdivide_yearmonthduration_by_yearmonthduration() -> bool {
    assert(false, "op:divide-yearMonthDuration-by-yearMonthDuration is not yet implemented");
    false
}

/// op:duration-equal - stub for unimplemented operator
pub fn stub_opduration_equal() -> bool {
    assert(false, "op:duration-equal is not yet implemented");
    false
}

/// op:except - stub for unimplemented operator
pub fn stub_opexcept() -> bool {
    assert(false, "op:except is not yet implemented");
    false
}

/// op:gDay-equal - stub for unimplemented operator
pub fn stub_opgday_equal() -> bool {
    assert(false, "op:gDay-equal is not yet implemented");
    false
}

/// op:gMonth-equal - stub for unimplemented operator
pub fn stub_opgmonth_equal() -> bool {
    assert(false, "op:gMonth-equal is not yet implemented");
    false
}

/// op:gMonthDay-equal - stub for unimplemented operator
pub fn stub_opgmonthday_equal() -> bool {
    assert(false, "op:gMonthDay-equal is not yet implemented");
    false
}

/// op:gYear-equal - stub for unimplemented operator
pub fn stub_opgyear_equal() -> bool {
    assert(false, "op:gYear-equal is not yet implemented");
    false
}

/// op:gYearMonth-equal - stub for unimplemented operator
pub fn stub_opgyearmonth_equal() -> bool {
    assert(false, "op:gYearMonth-equal is not yet implemented");
    false
}

/// op:hexBinary-equal - stub for unimplemented operator
pub fn stub_ophexbinary_equal() -> bool {
    assert(false, "op:hexBinary-equal is not yet implemented");
    false
}

/// op:hexBinary-greater-than - stub for unimplemented operator
pub fn stub_ophexbinary_greater_than() -> bool {
    assert(false, "op:hexBinary-greater-than is not yet implemented");
    false
}

/// op:hexBinary-less-than - stub for unimplemented operator
pub fn stub_ophexbinary_less_than() -> bool {
    assert(false, "op:hexBinary-less-than is not yet implemented");
    false
}

/// op:intersect - stub for unimplemented operator
pub fn stub_opintersect() -> bool {
    assert(false, "op:intersect is not yet implemented");
    false
}

/// op:multiply-dayTimeDuration - stub for unimplemented operator
pub fn stub_opmultiply_daytimeduration() -> bool {
    assert(false, "op:multiply-dayTimeDuration is not yet implemented");
    false
}

/// op:multiply-yearMonthDuration - stub for unimplemented operator
pub fn stub_opmultiply_yearmonthduration() -> bool {
    assert(false, "op:multiply-yearMonthDuration is not yet implemented");
    false
}

/// op:string-equal - stub for unimplemented operator
pub fn stub_opstring_equal() -> bool {
    assert(false, "op:string-equal is not yet implemented");
    false
}

/// op:string-greater-than - stub for unimplemented operator
pub fn stub_opstring_greater_than() -> bool {
    assert(false, "op:string-greater-than is not yet implemented");
    false
}

/// op:string-less-than - stub for unimplemented operator
pub fn stub_opstring_less_than() -> bool {
    assert(false, "op:string-less-than is not yet implemented");
    false
}

/// op:subtract-dates - stub for unimplemented operator
pub fn stub_opsubtract_dates() -> bool {
    assert(false, "op:subtract-dates is not yet implemented");
    false
}

/// op:subtract-dayTimeDuration-from-date - stub for unimplemented operator
pub fn stub_opsubtract_daytimeduration_from_date() -> bool {
    assert(false, "op:subtract-dayTimeDuration-from-date is not yet implemented");
    false
}

/// op:subtract-dayTimeDuration-from-time - stub for unimplemented operator
pub fn stub_opsubtract_daytimeduration_from_time() -> bool {
    assert(false, "op:subtract-dayTimeDuration-from-time is not yet implemented");
    false
}

/// op:subtract-times - stub for unimplemented operator
pub fn stub_opsubtract_times() -> bool {
    assert(false, "op:subtract-times is not yet implemented");
    false
}

/// op:subtract-yearMonthDuration-from-date - stub for unimplemented operator
pub fn stub_opsubtract_yearmonthduration_from_date() -> bool {
    assert(false, "op:subtract-yearMonthDuration-from-date is not yet implemented");
    false
}

/// op:subtract-yearMonthDuration-from-dateTime - stub for unimplemented operator
pub fn stub_opsubtract_yearmonthduration_from_datetime() -> bool {
    assert(false, "op:subtract-yearMonthDuration-from-dateTime is not yet implemented");
    false
}

/// op:subtract-yearMonthDurations - stub for unimplemented operator
pub fn stub_opsubtract_yearmonthdurations() -> bool {
    assert(false, "op:subtract-yearMonthDurations is not yet implemented");
    false
}

/// op:time-equal - stub for unimplemented operator
pub fn stub_optime_equal() -> bool {
    assert(false, "op:time-equal is not yet implemented");
    false
}

/// op:time-greater-than - stub for unimplemented operator
pub fn stub_optime_greater_than() -> bool {
    assert(false, "op:time-greater-than is not yet implemented");
    false
}

/// op:time-less-than - stub for unimplemented operator
pub fn stub_optime_less_than() -> bool {
    assert(false, "op:time-less-than is not yet implemented");
    false
}

/// op:to - stub for unimplemented operator
pub fn stub_opto() -> bool {
    assert(false, "op:to is not yet implemented");
    false
}

/// op:union - stub for unimplemented operator
pub fn stub_opunion() -> bool {
    assert(false, "op:union is not yet implemented");
    false
}

/// op:yearMonthDuration-greater-than - stub for unimplemented operator
pub fn stub_opyearmonthduration_greater_than() -> bool {
    assert(false, "op:yearMonthDuration-greater-than is not yet implemented");
    false
}

/// op:yearMonthDuration-less-than - stub for unimplemented operator
pub fn stub_opyearmonthduration_less_than() -> bool {
    assert(false, "op:yearMonthDuration-less-than is not yet implemented");
    false
}

// ============================================================================
// AUTO-GENERATED STUBS: Additional stubs needed by test packages
// ============================================================================
// These stubs were auto-generated because test packages reference them.
// Feel free to add documentation explaining why each cannot be implemented.

/// fn:parse-ietf-date - stub for unimplemented fn function
pub fn stub_fnparse_ietf_date() -> bool {
    assert(false, "fn:parse-ietf-date is not yet implemented");
    false
}

// ============================================================================
// AUTO-GENERATED STUBS: Additional stubs needed by test packages
// ============================================================================
// These stubs were auto-generated because test packages reference them.
// Feel free to add documentation explaining why each cannot be implemented.

/// fn:timezone-from-dateTime - stub for unimplemented fn function
pub fn stub_fntimezone_from_datetime() -> bool {
    assert(false, "fn:timezone-from-dateTime is not yet implemented");
    false
}
