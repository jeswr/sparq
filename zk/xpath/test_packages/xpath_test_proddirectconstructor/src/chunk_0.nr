//! Test chunk 0 for prod:DirectConstructor
//! Contains 19 tests

use dep::xpath::stub_proddirectconstructor;

#[test]
fn constr_pi_content_2() {
    //  PI constructor - regular content
    // XPath: fn:data(<?pi content?>) = \"content\"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn constr_pi_content_4() {
    //  PI constructor - empty content
    // XPath: fn:data(<?pi ?>) = \"\"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn constr_comment_2() {
    //  comment constructor - regular content
    // XPath: fn:data(<!--comment-->) = \"comment\"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn constr_comment_4() {
    //  comment constructor - empty content
    // XPath: fn:data(<!---->) = \"\"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_1() {
    //  Test that processing-instruction data does not receive any special treatment.
    // XPath: string(<?target \"\"''content&amp;amp;ss&amp;#00; &amp;#x201
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_29() {
    //  Extract the name from a PI.
    // XPath: local-name(<?xmlSUFFIX content?>)
    // Expected: xmlSUFFIX
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_30() {
    //  Extract the name from a PI.
    // XPath: local-name(<?PREFIXxml content?>)
    // Expected: PREFIXxml
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_31() {
    //  Extract the name from a PI.
    // XPath: local-name(<?PREFIXxmlSUFFIX content?>)
    // Expected: PREFIXxmlSUFFIX
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_32() {
    //  Extract the data from a directly constructed PI.
    // XPath: <?validchar ???<<???? <? >?hm???> eq \"???<<???? <? >?hm??\"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_33() {
    //  Extract the data from a directly constructed PI.
    // XPath: <?validchar content ?> eq \"content \"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_34() {
    //  Extract the data from a directly constructed PI.
    // XPath: <?validchar content a b c asdada dad ?> eq \"content a b c a
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_35() {
    //  Extract the content from a comment.
    // XPath: <!-- some - - - - content - - - >>>>> << >>><>& ;& --> eq \"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_36() {
    //  Extract the content from a comment.
    // XPath: <!-- - - - - - - - - - - - - - - - - --> eq \" - - - - - - -
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_38() {
    //  Space after PI data isn't stripped.
    // XPath: string(<?target content ?>) eq \"content \"
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_40() {
    //  Mixed content involving two text nodes and one comment node.
    // XPath: string(<e>a<!--data tar-->b</e>)
    // Expected: ab
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_41() {
    //  Ensure comment content doesn't receive special interpretation.
    // XPath: string(<!-- content&amp;amp;ss&amp;amp;#00; &amp;#x2014;-->)
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_44() {
    //  Test the atomized type of a PI.
    // XPath: data(<?target data?>) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_45() {
    //  Test the atomized type of a comment.
    // XPath: data(<!-- a comment -->) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}

#[test]
fn k2_directconother_70() {
    //  Ensure that EOLs are normalized in text nodes, but not when specified with char
    // XPath: string-to-codepoints(<a>     |&#xD; &#xD;&#xA;</a>)
    // Expected: 10 10 10 32 10 124 13 32 13 10
    // Calls stub function - will fail until prod:DirectConstructor is implemented
    let _ = stub_proddirectconstructor();
}
