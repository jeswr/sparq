//! Test chunk 1 for prod:Literal
//! Contains 28 tests

use dep::xpath::stub_prodliteral;

#[test]
fn k_literals_4() {
    //  '''''' is a valid string literal.
    // XPath: \"\"\"\" eq '\"'
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_5() {
    //  '''''' is a valid string literal.
    // XPath: '''' eq \"'\"
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_6() {
    //  ''fo''''' is a valid string literal.
    // XPath: \"fo\"\"o\" eq concat(\"fo\", \"\"\"\", \"o\")
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_8() {
    //  '3.' is a valid number literal.
    // XPath: 3. eq 3.
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_9() {
    //  '.3' is a valid number literal.
    // XPath: .3 eq .3
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_10() {
    //  A test whose essence is: `1.3e-3 instance of xs:double`.
    // XPath: 1.3e-3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_11() {
    //  A test whose essence is: `1.e+3 instance of xs:double`.
    // XPath: 1.e+3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_12() {
    //  A test whose essence is: `1.e3 instance of xs:double`.
    // XPath: 1.e3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_13() {
    //  A test whose essence is: `1231.123e3 instance of xs:double`.
    // XPath: 1231.123e3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_14() {
    //  A test whose essence is: `1.E3 instance of xs:double`.
    // XPath: 1.E3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_15() {
    //  A test whose essence is: `-1231.123e3 instance of xs:double`.
    // XPath: -1231.123e3 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_16() {
    //  A test whose essence is: `3 instance of xs:integer`.
    // XPath: 3 instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_17() {
    //  A test whose essence is: `3.3 instance of xs:decimal`.
    // XPath: 3.3 instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_18() {
    //  A test whose essence is: `3.3e1 instance of xs:double`.
    // XPath: 3.3e1 instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_19() {
    //  A test whose essence is: `'a xs:string' instance of xs:string`.
    // XPath: \"a xs:string\" instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_20() {
    //  A test whose essence is: `not(xs:double('NaN'))`.
    // XPath: not(xs:double(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_21() {
    //  A test whose essence is: `not(xs:float('NaN'))`.
    // XPath: not(xs:float(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_22() {
    //  A test whose essence is: `xs:float('NaN') instance of xs:float`.
    // XPath: xs:float(\"NaN\") instance of xs:float
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_28() {
    //  '.3' is a valid literal.
    // XPath: 0.3 eq .3
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_31a() {
    //  An invalid character reference is not invalid in XPath.
    // XPath: \"a string &;\"
    // Expected: a string &;
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_37() {
    //  Simple test of a decimal character reference.
    // XPath: \"t\" eq \"t\"
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_46() {
    //  Test containing all predefined character references and one hexa and decimal ch
    // XPath: \"&lt; &gt; &amp; &quot; &apos; &#x48; &#48;\" eq \"< > &amp
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_47() {
    //  Simple test of a hexa decimal character reference.
    // XPath: \"t\" eq \"&#x74;\"
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_47a() {
    //  Simple test of a hexa decimal character reference: not recognized in XPath
    // XPath: \"t\" eq \"&#x74;\"
    // Expected: false
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k_literals_49() {
    //  An '&' character reference embedded in other text.
    // XPath: \"I love brownies&amp;cookies.\" eq \"I love brownies&amp;co
    // Expected: true
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k2_literals_3() {
    //  Preceeding zeros are allowed in character references.
    // XPath: \"&#0000045;\"
    // Expected: -
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k2_literals_30() {
    //  Check EBNF 142.
    // XPath: .5
    // Expected: 0.5
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}

#[test]
fn k2_literals_32() {
    //  Numeric literals can end with a dot.
    // XPath: 465.
    // Expected: 465
    // Calls stub function - will fail until prod:Literal is implemented
    let _ = stub_prodliteral();
}
