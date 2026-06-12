//! Test chunk 8 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_60() {
    //  A test whose essence is: `xs:untypedAtomic(xs:decimal(3)) eq '3'`.
    // XPath: xs:untypedAtomic(xs:decimal(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_70() {
    //  Casting a xs:QName value to xs:QName is possible.
    // XPath: (xs:QName(\"local-name\") cast as xs:QName) eq xs:QName(\"lo
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_71b() {
    //  Casting xs:untypedAtomic to xs:QName is an error.
    // XPath: xs:untypedAtomic(\"ncname\") cast as xs:QName?
    // Expected: ncname
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_72() {
    //  A test whose essence is: `'example.com/' cast as xs:anyURI eq xs:anyURI('exampl
    // XPath: \"example.com/\" cast as xs:anyURI eq xs:anyURI('example.com
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_73() {
    //  A test whose essence is: `xs:untypedAtomic('example.com/') cast as xs:anyURI eq
    // XPath: xs:untypedAtomic(\"example.com/\") cast as xs:anyURI eq xs:a
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_74() {
    //  A test whose essence is: `xs:anyURI('example.com/') cast as xs:anyURI eq xs:any
    // XPath: xs:anyURI(\"example.com/\") cast as xs:anyURI eq xs:anyURI('
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_75() {
    //  A test whose essence is: `xs:double(3) eq 3`.
    // XPath: xs:double(3) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_76() {
    //  A test whose essence is: `xs:double(3.1) eq 3.1`.
    // XPath: xs:double(3.1) eq 3.1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_77() {
    //  A test whose essence is: `xs:double(6) instance of xs:double`.
    // XPath: xs:double(6) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_78() {
    //  A test whose essence is: `xs:string(xs:double('INF')) eq 'INF'`.
    // XPath: xs:string(xs:double(\"INF\")) eq \"INF\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_79() {
    //  A test whose essence is: `xs:string(xs:double('-INF')) eq '-INF'`.
    // XPath: xs:string(xs:double(\"-INF\")) eq \"-INF\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_80() {
    //  A test whose essence is: `xs:float(3) eq 3`.
    // XPath: xs:float(3) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_81() {
    //  A test whose essence is: `xs:float(3.1) eq 3.1`.
    // XPath: xs:float(3.1) eq 3.1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_82() {
    //  A test whose essence is: `xs:float(6) instance of xs:float`.
    // XPath: xs:float(6) instance of xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_83() {
    //  A test whose essence is: `xs:string(xs:float('INF')) eq 'INF'`.
    // XPath: xs:string(xs:float(\"INF\")) eq \"INF\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_84() {
    //  A test whose essence is: `xs:string(xs:float('-INF')) eq '-INF'`.
    // XPath: xs:string(xs:float(\"-INF\")) eq \"-INF\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_85() {
    //  A test whose essence is: `xs:decimal(3) eq 3`.
    // XPath: xs:decimal(3) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_86() {
    //  A test whose essence is: `xs:decimal(3.1) eq 3.1`.
    // XPath: xs:decimal(3.1) eq 3.1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_87() {
    //  A test whose essence is: `xs:decimal(6) instance of xs:decimal`.
    // XPath: xs:decimal(6) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_88() {
    //  A test whose essence is: `xs:integer(3) eq 3`.
    // XPath: xs:integer(3) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_89() {
    //  A test whose essence is: `xs:integer(3.1) eq 3`.
    // XPath: xs:integer(3.1) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_90() {
    //  Test that when casting xs:decimal to xs:integer that conversion happens.
    // XPath: xs:integer(3.1) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_91() {
    //  A test whose essence is: `xs:integer(6) instance of xs:integer`.
    // XPath: xs:integer(6) instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_92() {
    //  A test whose essence is: `xs:integer(6) instance of xs:decimal`.
    // XPath: xs:integer(6) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_98() {
    //  A test whose essence is: `exists(xs:hexBinary('Ab08bcFFAA08b6'))`.
    // XPath: exists(xs:hexBinary(\"Ab08bcFFAA08b6\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_99() {
    //  A test whose essence is: `exists(xs:hexBinary('FF'))`.
    // XPath: exists(xs:hexBinary(\"FF\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_100() {
    //  A test whose essence is: `exists(xs:hexBinary('aa'))`.
    // XPath: exists(xs:hexBinary(\"aa\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_101() {
    //  A test whose essence is: `count(xs:hexBinary(xs:hexBinary('03'))) eq 1`.
    // XPath: count(xs:hexBinary(xs:hexBinary(\"03\"))) eq 1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_102() {
    //  A test whose essence is: `xs:string(xs:hexBinary('aa')) eq 'AA'`.
    // XPath: xs:string(xs:hexBinary(\"aa\")) eq \"AA\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_103() {
    //  A test whose essence is: `xs:string(xs:hexBinary('0a')) eq '0A'`.
    // XPath: xs:string(xs:hexBinary(\"0a\")) eq \"0A\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_104() {
    //  A test whose essence is: `xs:string(xs:hexBinary('a0')) eq 'A0'`.
    // XPath: xs:string(xs:hexBinary(\"a0\")) eq \"A0\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_105() {
    //  A test whose essence is: `xs:string(xs:hexBinary('a4')) eq 'A4'`.
    // XPath: xs:string(xs:hexBinary(\"a4\")) eq \"A4\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_106() {
    //  A test whose essence is: `xs:string(xs:hexBinary('c0')) eq 'C0'`.
    // XPath: xs:string(xs:hexBinary(\"c0\")) eq \"C0\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_107() {
    //  A test whose essence is: `xs:string(xs:hexBinary('fA')) eq 'FA'`.
    // XPath: xs:string(xs:hexBinary(\"fA\")) eq \"FA\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_108() {
    //  A test whose essence is: `xs:string(xs:hexBinary('10')) eq '10'`.
    // XPath: xs:string(xs:hexBinary(\"10\")) eq \"10\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_109() {
    //  A test whose essence is: `xs:string(xs:hexBinary('031a34123b')) eq '031A34123B'
    // XPath: xs:string(xs:hexBinary(\"031a34123b\")) eq \"031A34123B\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_110() {
    //  A test whose essence is: `xs:string(xs:hexBinary('03')) eq '03'`.
    // XPath: xs:string(xs:hexBinary(\"03\")) eq \"03\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_111() {
    //  A test whose essence is: `xs:string(xs:hexBinary('0c')) eq '0C'`.
    // XPath: xs:string(xs:hexBinary(\"0c\")) eq \"0C\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_112() {
    //  A test whose essence is: `xs:string(xs:hexBinary('0b')) eq '0B'`.
    // XPath: xs:string(xs:hexBinary(\"0b\")) eq \"0B\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_113() {
    //  A test whose essence is: `xs:string(xs:hexBinary('3a')) eq '3A'`.
    // XPath: xs:string(xs:hexBinary(\"3a\")) eq \"3A\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_114() {
    //  An empty string is a valid lexical representation for xs:hexBinary, and means '
    // XPath: xs:string(xs:hexBinary(\"\")) eq \"\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_126() {
    //  A test whose essence is: `xs:string(xs:base64Binary('0w==')) eq '0w=='`.
    // XPath: xs:string(xs:base64Binary(\"0w==\")) eq \"0w==\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_127() {
    //  A test whose essence is: `xs:string(xs:base64Binary('aaa a')) eq 'aaaa'`.
    // XPath: xs:string(xs:base64Binary(\"aaa a\")) eq \"aaaa\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_128() {
    //  A test whose essence is: `xs:string(xs:base64Binary('frfhforlksid7453')) eq 'fr
    // XPath: xs:string(xs:base64Binary(\"frfhforlksid7453\")) eq \"frfhfo
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_130() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('03'))) eq 'Aw
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"03\"))) eq \"Aw==\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_131() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('0f3c'))) eq '
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"0f3c\"))) eq \"Dzw=
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_132() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('12'))) eq 'Eg
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"12\"))) eq \"Eg==\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_133() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('3B'))) eq 'Ow
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"3B\"))) eq \"Ow==\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_134() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('69A69A'))) eq
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"69A69A\"))) eq \"aa
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_135() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('F43D1234ce8f'
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"F43D1234ce8f\"))) e
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
