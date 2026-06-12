//! Test chunk 0 for prod:Literal
//! Contains 50 tests

use dep::xpath::stub_prodliteral;

#[test]
fn literals001() {
    //  Simple use case for string literals
    // XPath: \"test\"
    // Expected: test
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals002() {
    //  Simple use case for string literals
    // XPath: 'test'
    // Expected: test
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals003() {
    //  Test case where string literal contains a new line
    // XPath: \"line1 line2\"
    // Expected: line1 line2
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals004() {
    //  Test case where string literal contains a new line
    // XPath: 'line1 line2'
    // Expected: line1 line2
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals010() {
    //  Test for valid integer literal
    // XPath: 65535032
    // Expected: 65535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals011() {
    //  Test for valid integer literal
    // XPath: -65535032
    // Expected: -65535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals012() {
    //  Test for valid decimal literal
    // XPath: 65535032.0023
    // Expected: 65535032.0023
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals013() {
    //  Test for valid decimal literal
    // XPath: .65535032
    // Expected: 0.65535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals014() {
    //  Test for valid decimal literal
    // XPath: -.65535032
    // Expected: -0.65535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals015() {
    //  Test for valid decimal literal
    // XPath: +.65535032
    // Expected: 0.65535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals016() {
    //  Test for valid double literal
    // XPath: 65535032e2
    // Expected: 6.5535032E9
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals017() {
    //  Test for valid double literal
    // XPath: 65535.032e2
    // Expected: 6.5535032E6
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals018() {
    //  Test for valid double literal
    // XPath: .65535032e2
    // Expected: 65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals019() {
    //  Test for valid double literal
    // XPath: -.65535032e2
    // Expected: -65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals020() {
    //  Test for valid double literal
    // XPath: +.65535032e2
    // Expected: 65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals021() {
    //  Test for valid double literal
    // XPath: 65535.032e-2
    // Expected: 655.35032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals022() {
    //  Test for valid double literal
    // XPath: .65535032e-2
    // Expected: 0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals023() {
    //  Test for valid double literal
    // XPath: -.65535032e-2
    // Expected: -0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals024() {
    //  Test for valid double literal
    // XPath: +.65535032e-2
    // Expected: 0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals025() {
    //  Test for valid double literal
    // XPath: -65535.032e2
    // Expected: -6.5535032E6
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals026() {
    //  Test for valid double literal
    // XPath: -65535.032e-2
    // Expected: -655.35032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals027() {
    //  Test for valid double literal
    // XPath: 65535032E2
    // Expected: 6.5535032E9
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals028() {
    //  Test for valid double literal
    // XPath: 65535.032E2
    // Expected: 6.5535032E6
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals029() {
    //  Test for valid double literal
    // XPath: .65535032E2
    // Expected: 65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals030() {
    //  Test for valid double literal
    // XPath: -.65535032E2
    // Expected: -65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals031() {
    //  Test for valid double literal
    // XPath: +.65535032E2
    // Expected: 65.535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals032() {
    //  Test for valid double literal
    // XPath: 65535.032E-2
    // Expected: 655.35032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals033() {
    //  Test for valid double literal
    // XPath: .65535032E-2
    // Expected: 0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals034() {
    //  Test for valid double literal
    // XPath: -.65535032E-2
    // Expected: -0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals035() {
    //  Test for valid double literal
    // XPath: +.65535032E-2
    // Expected: 0.0065535032
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals056() {
    //  Test for string literal containing the predefined entity reference '&amp;' (XQu
    // XPath: \"&amp;\"
    // Expected: &
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals056a() {
    //  Test for string literal containing the predefined entity reference '&amp;' (XPa
    // XPath: \"&amp;\"
    // Expected: &amp;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals057() {
    //  Test for string literal containing the predefined entity reference '&quot;' (XQ
    // XPath: \"&quot;\"
    // Expected: "
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals057a() {
    //  Test for string literal containing the predefined entity reference '&quot;' (XP
    // XPath: \"&quot;\"
    // Expected: &quot;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals058() {
    //  Test for string literal containing the predefined entity reference '&apos;' (XQ
    // XPath: \"&apos;\"
    // Expected: '
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals058a() {
    //  Test for string literal containing the predefined entity reference '&apos;' (XP
    // XPath: \"&apos;\"
    // Expected: &apos;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals059() {
    //  Test for string literal containing the predefined entity reference '&lt;' (XQue
    // XPath: \"&lt;\"
    // Expected: <
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals059a() {
    //  Test for string literal containing the predefined entity reference '&lt;' (XPat
    // XPath: \"&lt;\"
    // Expected: &lt;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals060() {
    //  Test for string literal containing the predefined entity reference '&gt;' (XQue
    // XPath: \"&gt;\"
    // Expected: >
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals060a() {
    //  Test for string literal containing the predefined entity reference '&gt;' (XPat
    // XPath: \"&gt;\"
    // Expected: &gt;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals061() {
    //  Test for string literal containing the character reference '&#8364;' which tran
    // XPath: \"&#8364;\"
    // Expected: ?
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals061a() {
    //  Test for string literal containing the character reference '&#8364;' which tran
    // XPath: \"&#8364;\"
    // Expected: &#8364;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals062() {
    //  Test the escaping of the ' (quotation) character in XPath/XQuery.
    // XPath: \"\"\"\"
    // Expected: "
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals063() {
    //  Test the escaping of the ' (quotation) character in XPath/XQuery.
    // XPath: ''''
    // Expected: '
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals064() {
    //  Test the escaping of the ' (apostrophe) and ' (quotation) characters in XPath/X
    // XPath: \"He said, \"\"I don't like it.\"\"\"
    // Expected: He said, "I don't like it."
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals065() {
    //  Test the escaping of the ' (apostrophe) and ' (quotation) characters in XPath/X
    // XPath: 'He said, \"I don''t like it.\"'
    // Expected: He said, "I don't like it."
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals068() {
    //  Test the escaping of the &apos; (apostrophe) and &quot; (quotation) characters
    // XPath: text{ 'He said, \"I don''t like it.\"' }
    // Expected: He said, "I don't like it."
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn literals069() {
    //  Test the escaping of the &apos; (apostrophe) and &quot; (quotation) characters
    // XPath: text{ \"He said, \"\"I don't like it.\"\"\" }
    // Expected: He said, "I don't like it."
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_1() {
    //  A test whose essence is: `'fo''o' eq 'fo''o'`.
    // XPath: 'fo''o' eq 'fo''o'
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_2() {
    //  A test whose essence is: `'foo' eq 'foo'`.
    // XPath: 'foo' eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}
