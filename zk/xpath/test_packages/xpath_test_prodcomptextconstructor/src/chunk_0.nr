//! Test chunk 0 for prod:CompTextConstructor
//! Contains 22 tests

use dep::xpath::stub_prodcomptextconstructor;

#[test]
fn constr_text_enclexpr_1() {
    //  enclosed expression in text node - atomic values
    // XPath: text {1,'string',3.14,xs:float('1.2345e-2'),xs:dateTime('200
    // Expected: 1 string 3.14 0.012345 2002-04-02T12:00:
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_enclexpr_2() {
    //  enclosed expression in text node - nodes
    // XPath: text {<elem>123</elem>, (<elem attr='456'/>)/@attr, (<elem>7
    // Expected: 123 456 789
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_enclexpr_3() {
    //  enclosed expression in text node - empty string
    // XPath: text {1,'',2}
    // Expected: 1  2
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_enclexpr_4() {
    //  enclosed expression in text node - empty node
    // XPath: text {1,<a/>,2}
    // Expected: 1  2
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_enclexpr_5() {
    //  enclosed expression in text node - nodes
    // XPath: text {/root}
    // Expected: texttext
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_doc_1() {
    //  strip document nodes
    // XPath: text {., .}
    // Expected: texttext texttext
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_parent_1() {
    //  empty parent
    // XPath: count((text {'text'})/..)
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_string_1() {
    //  string value of text node
    // XPath: fn:string(text {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_data_1() {
    //  typed value of text node
    // XPath: fn:data(text {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_count_1() {
    //  count text nodes
    // XPath: count(text {''})
    // Expected: 1
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_count_2() {
    //  count text nodes
    // XPath: count(text {()})
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_count_3() {
    //  count text nodes
    // XPath: count(element elem {text {''}}/text())
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_count_5() {
    //  count text nodes
    // XPath: count(<a>{text {''}}<b/>{text {''}}<b/>{text {''}}</a>/text(
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_adjtext_1() {
    //  merge adjacent text nodes
    // XPath: count(<elem>{text {'te'}, text {'xt'}}</elem>/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_adjtext_2() {
    //  merge adjacent text nodes
    // XPath: count(document {text {'te'}, text {'xt'}}/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn constr_text_nested_1() {
    //  nested text node constructor
    // XPath: text {text {'one', text {'two'}}, 'three', text {'four'}}
    // Expected: one two three four
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_3() {
    //  An empty text node is nevertheless a text node, not the empty sequence.
    // XPath: string(text {\"\"}) eq \"\"
    // Expected: true
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_6() {
    //  Test the atomized type.
    // XPath: data(text {\"content\"}) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_15() {
    //  Ensure that an empty text node between two atomic values are serialized correct
    // XPath: string(<elem>{1}{text{\"\"}}{2}</elem>)
    // Expected: 12
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_16() {
    // Argument to text constructor is an empty sequence (Saxon bug 5061)
    // XPath: let $t := text { <_/>[text()] }                      return
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_17() {
    // Argument to text constructor is an empty sequence (Saxon bug 5061)
    // XPath: count(data(text { <_/>[text()] }))
    // Expected: 0
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}

#[test]
fn k2_context_18() {
    // Argument to text constructor is an empty sequence (Saxon bug 5061)
    // XPath: count(string(text { }))
    // Expected: 1
    // Calls stub function - will fail until prod:CompTextConstructor is implemented
    let _ = stub_prodcomptextconstructor();
}
