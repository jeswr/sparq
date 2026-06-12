//! Test chunk 0 for fn:document-uri
//! Contains 31 tests

use dep::xpath::stub_fndocument_uri;

#[test]
fn fn_document_uri_2() {
    //  Evaluation of fn:document-uri with argument set to empty sequence. Use the fn:c
    // XPath: fn:count(fn:document-uri(()))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_3() {
    //  Evaluation of fn:document-uri with argument set to a computed element node. Use
    // XPath: fn:count(fn:document-uri(element anElement {\"some content\"
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_4() {
    //  Evaluation of fn:document-uri with argument set to a computed comment node. Use
    // XPath: fn:count(fn:document-uri(comment {\"a comment node\"}))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_5() {
    //  Evaluation of fn:document-uri with argument set to a computed text node. Use th
    // XPath: fn:count(fn:document-uri(text {\"a text node\"}))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_6() {
    //  Evaluation of fn:document-uri with argument set to a computed attribute node. U
    // XPath: fn:count(fn:document-uri(attribute anAttribute {\"an attribu
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_7() {
    //  Evaluation of fn:document-uri with argument set to a computed PI node. Use the
    // XPath: fn:count(fn:document-uri(processing-instruction {\"PITarget\
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_8() {
    //  Evaluation of fn:document-uri with argument set to a directly constructed PI no
    // XPath: fn:count(fn:document-uri(<?audio-output beep?>))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_9() {
    //  Evaluation of fn:document-uri with argument set to a directly constructed comme
    // XPath: fn:count(fn:document-uri(<!-- A comment node -->))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_10() {
    //  Evaluation of fn:document-uri with argument set to a directly constructed eleme
    // XPath: fn:count(fn:document-uri(<anElement>element content</anEleme
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_11() {
    //  Evaluation of fn:document-uri with argument set to a directly constructed docum
    // XPath: fn:count(fn:document-uri(document {<anElement>element conten
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_13() {
    //  Evaluation of fn:document-uri with argument set to element node from xml file.
    // XPath: fn:count(fn:document-uri(/works[1]/employee[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_14() {
    //  Evaluation of fn:document-uri with argument set to an attribute node from xml f
    // XPath: fn:count(fn:document-uri(/works[1]/employee[1]/@name))
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_20() {
    //  Evaluation of fn:document-uri used as argument to an fn:substring-before functi
    // XPath: fn:substring-before(fn:string(fn:contains(fn:document-uri(/)
    // Expected: tru
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_21() {
    //  Evaluation of fn:document-uri used as argument to an fn:substring-after functio
    // XPath: fn:substring-after(fn:string(fn:contains(fn:document-uri(/),
    // Expected: rue
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_29() {
    //  Evaluation of fn:document-uri#0 with context item set to element node from xml
    // XPath: fn:count((/works[1]/employee[1])/document-uri())
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_30() {
    //  Evaluation of fn:document-uri#0 with context item set to an attribute node from
    // XPath: fn:count((/works[1]/employee[1]/@name)/document-uri())
    // Expected: 0
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_36() {
    //  Evaluation of fn:document-uri#0 used as argument to an fn:substring-before func
    // XPath: fn:substring-before(fn:string(fn:contains((/)/document-uri()
    // Expected: tru
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_37() {
    //  Evaluation of fn:document-uri#0 used as argument to an fn:substring-after funct
    // XPath: fn:substring-after(fn:string(fn:contains((/)/document-uri(),
    // Expected: rue
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k_documenturifunc_2a() {
    //  A test whose essence is: `document-uri()`.
    // XPath: ends-with(document-uri(),\"works-mod.xml\")
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k_documenturifunc_3() {
    //  A test whose essence is: `empty(document-uri(()))`.
    // XPath: empty(document-uri(()))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_1() {
    //  Invoke on a comment node.
    // XPath: empty(document-uri(<!-- comment -->))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_2() {
    //  Invoke on an attribute node.
    // XPath: empty(document-uri(attribute name {\"content\"}))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_3() {
    //  Invoke on a processing instruction node.
    // XPath: empty(document-uri(<?target data?>))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_4() {
    //  Invoke on a processing instruction node.
    // XPath: empty(document-uri(processing-instruction name {123}))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_5() {
    //  Invoke on a text node.
    // XPath: empty(document-uri(text {123}))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_6() {
    //  Invoke on a single element node.
    // XPath: empty(document-uri(<elem/>))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_7() {
    //  Invoke on a single attribute node.
    // XPath: empty(document-uri(<elem attr=\"f\"/>/@attr))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_8() {
    //  Invoke on a single document node.
    // XPath: empty(document-uri(document {1}))
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_9() {
    //  Invoke on a tree document node.
    // XPath: let $i := document { <e> <a/> <a/> <a/> <b/> <b/> <a/> <a/>
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn k2_documenturifunc_11() {
    //  Ensure that the return value of document-uri() is of correct type.
    // XPath: for $i in (1, document-uri(.), 3) return typeswitch($i) case
    // Expected: int xs:anyURI int
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}

#[test]
fn fn_document_uri_0_ok() {
    // fn:document-uri with zero arguments given context via a predicate - predicate sh
    // XPath: empty(//works[fn:document-uri()])
    // Expected: true
    // Calls stub function - will fail until fn:document-uri is implemented
    let _ = stub_fndocument_uri();
}
