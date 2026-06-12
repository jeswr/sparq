//! Test chunk 25 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k2_seqexprcast_133() {
    //  Pass preceding zeros to nonPositiveInteger.
    // XPath: xs:nonPositiveInteger(\"-000000000\")
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_134() {
    //  Pass preceding zeros to negativeInteger.
    // XPath: xs:negativeInteger(\"-000000001\")
    // Expected: -1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_135() {
    //  Pass preceding zeros to long.
    // XPath: xs:long(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_136() {
    //  Pass preceding zeros to int.
    // XPath: xs:int(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_137() {
    //  Pass preceding zeros to short.
    // XPath: xs:short(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_138() {
    //  Pass preceding zeros to byte.
    // XPath: xs:byte(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_139() {
    //  Pass preceding zeros to nonNegativeInteger.
    // XPath: xs:nonNegativeInteger(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_140() {
    //  Pass preceding zeros to unsignedLong.
    // XPath: xs:unsignedLong(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_141() {
    //  Pass preceding zeros to unsignedInt.
    // XPath: xs:unsignedInt(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_142() {
    //  Pass preceding zeros to unsignedByte.
    // XPath: xs:unsignedByte(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_143() {
    //  Pass preceding zeros to unsignedShort.
    // XPath: xs:unsignedShort(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_144() {
    //  Pass preceding zeros to positiveInteger.
    // XPath: xs:positiveInteger(\"000000001\")
    // Expected: 1
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_157() {
    //  Cast fn:false() to xs:language.
    // XPath: xs:language(false())
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_158() {
    //  Cast fn:true() to xs:language.
    // XPath: xs:language(true())
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_171() {
    //  Cast string with invalid characters to xs:language.
    // XPath: xs:language(\"abc-ab1efikl\")
    // Expected: abc-ab1efikl
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_172() {
    //  Cast string with invalid characters to xs:language.
    // XPath: xs:language(\"abc-a-1b1efikl\")
    // Expected: abc-a-1b1efikl
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_173() {
    //  Cast string with invalid characters to xs:language.
    // XPath: xs:language(\"abc-a-1-1b1efil\")
    // Expected: abc-a-1-1b1efil
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_174() {
    //  Cast string with invalid characters to xs:language.
    // XPath: xs:language(\"abc-a-1-1b01efil\")
    // Expected: abc-a-1-1b01efil
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_186() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\"::::\")
    // Expected: ::::
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_187() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\"_\")
    // Expected: _
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_188() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\":\")
    // Expected: :
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_189() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\"a\")
    // Expected: a
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_190() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\" a \")
    // Expected: a
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_191() {
    //  Pass tricky, valid input to xs:Name.
    // XPath: xs:Name(\"a1123-_--__-__--....-.--.-..12309asd\")
    // Expected: a1123-_--__-__--....-.--.-..12309asd
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_192() {
    //  Pass an empty string to normalizedString.
    // XPath: xs:normalizedString(\"\")
    // Expected:
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_193() {
    //  Pass an empty string to token.
    // XPath: xs:token(\"\")
    // Expected:
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_201() {
    //  When casting to xs:QName, the declared default element namespace is considered.
    // XPath: namespace-uri-from-QName(xs:QName(\"ncname\"))
    // Expected: http://example.com/defelementns
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_202() {
    //  Use complex input to xs:NMTOKEN.
    // XPath: xs:NMTOKEN(\" ::::...:SDA21455313 \")
    // Expected: ::::...:SDA21455313
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_208() {
    //  Use a large chunk of whitespace in an xs:anyURI.
    // XPath: concat(\"b\", xs:anyURI(\" \"), \"b\")
    // Expected: bb
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_209() {
    //  Ensure that a large Unicode codepoint is handled correctly in xs:anyURI.
    // XPath: xs:anyURI(\"?\")
    // Expected: ?
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_213() {
    //  Cast an xs:positiveInteger value to xs:double.
    // XPath: xs:double(xs:positiveInteger(\"52\"))
    // Expected: 52
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_214() {
    //  Cast an xs:int value to xs:double.
    // XPath: xs:double(xs:int(\"-1873914410\"))
    // Expected: -1.87391441E9
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_221() {
    //  Whitespace is ignored for xs:float's lexical space.
    // XPath: xs:float(\" NaN \")
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_222() {
    //  Whitespace is ignored for xs:double's lexical space.
    // XPath: xs:double(\" NaN \")
    // Expected: NaN
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_223() {
    //  Whitespace is ignored for xs:float's lexical space(#2).
    // XPath: xs:float(\" INF \")
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_224() {
    //  Whitespace is ignored for xs:double's lexical space(#2).
    // XPath: xs:double(\" INF \")
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_231a() {
    //  Test derived from K2-SeqExprCast-231 '+INF' is a valid value for xs:double unde
    // XPath: xs:double(\"+INF\")
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_232a() {
    //  '+INF' is a valid value for xs:float.
    // XPath: xs:float(\"+INF\")
    // Expected: INF
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_333() {
    //  Cast xs:byte to xs:double.
    // XPath: xs:double(xs:byte(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_334() {
    //  Cast xs:int to xs:double.
    // XPath: xs:double(xs:int(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_335() {
    //  Cast xs:long to xs:double.
    // XPath: xs:double(xs:long(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_336() {
    //  Cast xs:negativeInteger to xs:double.
    // XPath: xs:double(xs:negativeInteger(-3))
    // Expected: -3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_337() {
    //  Cast xs:nonNegativeInteger to xs:double.
    // XPath: xs:double(xs:nonNegativeInteger(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_338() {
    //  Cast xs:nonPositiveInteger to xs:double.
    // XPath: xs:double(xs:nonPositiveInteger(0))
    // Expected: 0
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_339() {
    //  Cast xs:short to xs:double.
    // XPath: xs:double(xs:short(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_340() {
    //  Cast xs:unsignedByte to xs:double.
    // XPath: xs:double(xs:unsignedByte(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_341() {
    //  Cast xs:unsignedInt to xs:double.
    // XPath: xs:double(xs:unsignedInt(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_342() {
    //  Cast xs:unsignedInt to xs:double.
    // XPath: xs:double(xs:unsignedInt(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_343() {
    //  Cast xs:unsignedLong to xs:double.
    // XPath: xs:double(xs:unsignedLong(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k2_seqexprcast_344() {
    //  Cast xs:unsignedShort to xs:double.
    // XPath: xs:double(xs:unsignedShort(3))
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
