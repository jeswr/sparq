//! Test chunk 0 for prod:DirElemContent.whitespace
//! Contains 22 tests

use dep::xpath::stub_proddirelemcontent_whitespace;

#[test]
fn constr_ws_tag_5() {
    //  count text nodes when stripping whitespace
    // XPath: declare boundary-space strip; fn:count((<elem> <a> </a> <a>
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn constr_ws_tag_10() {
    //  count text nodes when preserving whitespace
    // XPath: declare boundary-space preserve; fn:count((<elem> <a> </a> <
    // Expected: 9
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_1() {
    //  Apply fn:string() on a direct element constructor with preserved whitespace.
    // XPath: declare boundary-space preserve; string(<e> <b/> </e>)
    // Expected:
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_3() {
    //  Apply fn:string to an element constructor with two text nodes.
    // XPath: string(<a>aaaa<b/>aaaa</a>)
    // Expected: aaaaaaaa
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_5() {
    //  xml:space declarations are ignored.
    // XPath: declare boundary-space strip; string(<e xml:space=\"preserve
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_7() {
    //  xml:space declarations are ignored(#3).
    // XPath: declare boundary-space preserve; string(<e xml:space=\"defau
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_9() {
    //  No escaping mechanism applies for quotes/apostrophes inside element content.
    // XPath: string(<e>'a''a'''a\"a\"\"a\"\"\"a\"</e>) eq \"'a''a'''a\"\"
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_10() {
    //  Ensure non-boundary characters, a character reference, are handled properly.
    // XPath: string(<e> &#32; </e>)
    // Expected:
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_11() {
    //  Ensure non-boundary characters, CDATA, are handled properly.
    // XPath: string(<e> <![CDATA[ ]]> </e>)
    // Expected:
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_12() {
    //  Ensure extracting the string value from mixed content involving elements is pro
    // XPath: string(<e>123<b>XX</b>abc</e>)
    // Expected: 123XXabc
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_13() {
    //  Ensure extracting the string value from mixed content involving comments is pro
    // XPath: string(<e>123<!-- a comment -->ab<!-- another comment -->c</
    // Expected: 123abc
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_14() {
    //  Ensure extracting the string value from mixed content involving processing inst
    // XPath: string(<e>123<?target content ?>ab<?target2 content?>c</e>)
    // Expected: 123abc
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_15() {
    //  Ensure only one text node is constructed for contiguous data.
    // XPath: count(<elem>str{\"\"}asdas{\"asd\", \"asd\", \"''\", \"\"}{'
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_16() {
    //  Test empty element content result in an element with no children.
    // XPath: count(<a></a>/node())
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_17() {
    //  Test empty element content result in an element with no children(#2).
    // XPath: count(<a/>/node())
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_18() {
    //  Empty CDATA sections nevertheless causes whitespace to be preserved.
    // XPath: string(<elem> <![CDATA[]]> </elem>)
    // Expected:
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_19() {
    //  Extract the string value of nested elements.
    // XPath: string(<e>e<b>ddd</b></e>)
    // Expected: eddd
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_22() {
    //  It is valid to put ']]>' in element content.
    // XPath: string(<e>]]></e>)
    // Expected: ]]>
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_23() {
    //  Extract the string value of mixed content.
    // XPath: string(<elem><![CDATA[cdat]]><!-- a comment --><?target cont
    // Expected: cdat
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_24() {
    //  Extract the string value of mixed content(#2).
    // XPath: string(<elem> content <![CDATA[ content ]]> content </elem>)
    // Expected: content  content  content
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_25() {
    //  Extract the string value of mixed content(#3).
    // XPath: string(<elem><![CDATA[cdata&<>'\"< ]]>asda <?target content?
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}

#[test]
fn k2_directconelemwhitespace_26() {
    //  Extract the string value of a couple of nested elements.
    // XPath: string(<a> {1} <b> {1} </b> </a>)
    // Expected: 11
    // Calls stub function - will fail until prod:DirElemContent.whitespace is implemented
    let _ = stub_proddirelemcontent_whitespace();
}
