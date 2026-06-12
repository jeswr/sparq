//! Test chunk 9 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_136() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('f0'))) eq '8A
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"f0\"))) eq \"8A==\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_137() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('ffaabbddcceef
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"ffaabbddcceeff0134f
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_138() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('ffff'))) eq '
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"ffff\"))) eq \"//8=
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_139() {
    //  A test whose essence is: `xs:string(xs:base64Binary(xs:hexBinary('ffff34564321d
    // XPath: xs:string(xs:base64Binary(xs:hexBinary(\"ffff34564321deac987
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_140() {
    //  A test whose essence is: `xs:string(xs:hexBinary(xs:base64Binary('Ow=='))) eq '
    // XPath: xs:string(xs:hexBinary(xs:base64Binary(\"Ow==\"))) eq \"3B\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_141() {
    //  A test whose essence is: `xs:string(xs:hexBinary(xs:base64Binary('aaa a'))) eq
    // XPath: xs:string(xs:hexBinary(xs:base64Binary(\"aaa a\"))) eq \"69A
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_142() {
    //  An empty string is a valid lexical representation for xs:base64Binary, and mean
    // XPath: xs:string(xs:base64Binary(\"\")) eq \"\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_143() {
    //  Casting where the source type typically cannot be determined statically.
    // XPath: xs:hexBinary(remove((\"3B\", 1.1), 2) treat as xs:string)
    // Expected: 3B
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_154() {
    //  Ensure that when casting xs:duration to xs:string, preceding zeros are handled
    // XPath: xs:string(xs:duration(\"P0010Y0010M0010DT0010H0010M0010S\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_155() {
    //  Ensure that a negative xs:duration is properly serialized when cast to xs:strin
    // XPath: xs:string(xs:duration(\"-P0010DT0010H0010M0010S\")) eq \"-P1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_156() {
    //  Ensure that when casting xs:duration to xs:string, that empty fields are proper
    // XPath: xs:string(xs:duration(\"P0Y0M0DT00H00M00.000S\")) eq \"PT0S\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_157() {
    //  Ensure that when casting xs:duration to xs:string, that empty components are ha
    // XPath: xs:string(xs:duration(\"-PT8H23M0S\")) eq \"-PT8H23M\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_158() {
    //  Casting a xs:duration with zeroed time components to xs:string.
    // XPath: xs:string(xs:duration(\"-P2000Y11M5DT0H0M0.000S\")) eq \"-P2
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_159() {
    //  The canonical lexical representation for the xs:duration value P365D is 'P365D'
    // XPath: xs:string(xs:duration(\"P365D\")) eq \"P365D\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_160() {
    //  The canonical lexical representation for the xs:duration value P12M is 'P1Y'.
    // XPath: xs:string(xs:duration(\"P12M\")) eq \"P1Y\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_161() {
    //  The canonical lexical representation for the xs:duration value P31D is 'P31D'.
    // XPath: xs:string(xs:duration(\"P31D\")) eq \"P31D\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_162() {
    //  The canonical lexical representation for the xs:duration value P3Y0M is 'P3Y'.
    // XPath: xs:string(xs:yearMonthDuration(\"P3Y0M\")) eq \"P3Y\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_163() {
    //  Test that a xs:duration value with a small second component is serialized prope
    // XPath: xs:string(xs:duration(\"P31DT3H2M10.001S\")) eq \"P31DT3H2M1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_164() {
    //  The canonical form of the xs:duration value -PT0S is PT0S.
    // XPath: xs:string(xs:duration(\"-PT0S\")) eq \"PT0S\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_165() {
    //  Simple test of casting xs:dayTimeDuration to xs:duration.
    // XPath: xs:string(xs:duration(xs:dayTimeDuration(\"P31DT3H2M10.001S\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_166() {
    //  Simple test of casting xs:yearMonthDuration to xs:duration.
    // XPath: xs:string(xs:duration(xs:yearMonthDuration(\"P543Y456M\")))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_167() {
    //  Simple test of casting a negative xs:dayTimeDuration to xs:duration.
    // XPath: xs:string(xs:duration(xs:dayTimeDuration(\"-P31DT3H2M10.001S
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_168() {
    //  Simple test of casting a negative xs:yearMonthDuration to xs:duration.
    // XPath: xs:string(xs:duration(xs:yearMonthDuration(\"-P543Y456M\")))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_175() {
    //  Ensure that when casting xs:dayTimeDuration to xs:string, that empty fields are
    // XPath: xs:string(xs:dayTimeDuration(\"P0DT00H00M00.000S\")) eq \"PT
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_179() {
    //  Ensure that when casting xs:dayTimeDuration to xs:string, that preceding zeros
    // XPath: xs:string(xs:dayTimeDuration(\"P0010DT0010H0010M0010S\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_180() {
    //  Ensure that a negative xs:dayTimeDuration is properly serialized when cast to x
    // XPath: xs:string(xs:dayTimeDuration(\"-P0010DT0010H0010M0010S\")) e
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_181() {
    //  Ensure that when casting xs:dayTimeDuration to xs:string, that empty components
    // XPath: xs:string(xs:dayTimeDuration(\"-PT8H23M0S\")) eq \"-PT8H23M\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_182() {
    //  Casting a xs:duration with zeroed time components to xs:string.
    // XPath: xs:string(xs:dayTimeDuration(\"-P5DT0H0M0.000S\")) eq \"-P5D
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_183() {
    //  Test that a xs:dayTimeDuration value with a large day component is serialized p
    // XPath: xs:string(xs:dayTimeDuration(\"P9876DT1M\")) eq \"P9876DT1M\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_184() {
    //  Test that a xs:dayTimeDuration value with a large hour component is serialized
    // XPath: xs:string(xs:dayTimeDuration(\"PT9876H1M\")) eq \"P411DT12H1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_185() {
    //  Test that a xs:dayTimeDuration value with a large minute component is serialize
    // XPath: xs:string(xs:dayTimeDuration(\"PT6000M\")) eq \"P4DT4H\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_186() {
    //  Test that a xs:dayTimeDuration value with a large second component is serialize
    // XPath: xs:string(xs:dayTimeDuration(\"PT1M1231.432S\")) eq \"PT21M3
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_187() {
    //  Test that a xs:dayTimeDuration value with a small second component is serialize
    // XPath: xs:string(xs:dayTimeDuration(\"P31DT3H2M10.001S\")) eq \"P31
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_188() {
    //  The canonical form of the xs:dayTimeDuration value -PT0S is PT0S.
    // XPath: xs:string(xs:dayTimeDuration(\"-PT0S\")) eq \"PT0S\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_189() {
    //  Simple test of casting xs:duration to xs:dayTimeDuration.
    // XPath: xs:string(xs:dayTimeDuration(xs:duration(\"P3Y0M31DT3H2M10.0
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_190() {
    //  Simple test of casting xs:yearMonthDuration to xs:dayTimeDuration.
    // XPath: xs:string(xs:dayTimeDuration(xs:yearMonthDuration(\"P543Y456
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_191() {
    //  Simple test of casting a negative xs:duration to xs:dayTimeDuration.
    // XPath: xs:string(xs:dayTimeDuration(xs:duration(\"-P3Y0M31DT3H2M10.
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_192() {
    //  Simple test of casting a negative xs:yearMonthDuration to xs:dayTimeDuration.
    // XPath: xs:string(xs:dayTimeDuration(xs:yearMonthDuration(\"-P543Y45
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_193() {
    //  The canonical lexical representation for the xs:dayTimeDuration value P3D is 'P
    // XPath: xs:string(xs:dayTimeDuration(\"P3D\")) eq \"P3D\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_202() {
    //  Ensure that when casting xs:yearMonthDuration to xs:string, that preceding zero
    // XPath: xs:string(xs:yearMonthDuration(\"P0010Y0010M\")) eq \"P10Y10
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_203() {
    //  Ensure that a negative xs:yearMonthDuration is properly serialized when cast to
    // XPath: xs:string(xs:yearMonthDuration(\"-P0010Y0010M\")) eq \"-P10Y
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_204() {
    //  Ensure that when casting xs:yearMonthDuration to xs:string, that empty fields a
    // XPath: xs:string(xs:yearMonthDuration(\"P0Y0M\")) eq \"P0M\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_205() {
    //  Test that a xs:yearMonthDuration value with a large year component is serialize
    // XPath: xs:string(xs:yearMonthDuration(\"P2Y323M\")) eq \"P28Y11M\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_206() {
    //  Test that a xs:yearMonthDuration value with a large year and month component is
    // XPath: xs:string(xs:yearMonthDuration(\"-P543Y456M\")) eq \"-P581Y\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_207() {
    //  The canonical form of the xs:yearMonthDuration value -P0M is P0M.
    // XPath: xs:string(xs:yearMonthDuration(\"-P0M\")) eq \"P0M\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_208() {
    //  Simple test of casting xs:duration to xs:yearMonthDuration.
    // XPath: xs:string(xs:yearMonthDuration(xs:duration(\"P3Y0M31DT3H2M10
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_209() {
    //  Simple test of casting xs:dayTimeDuration to xs:yearMonthDuration.
    // XPath: xs:string(xs:yearMonthDuration(xs:dayTimeDuration(\"P31DT3H2
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_210() {
    //  Simple test of casting a negative xs:duration to xs:yearMonthDuration.
    // XPath: xs:string(xs:yearMonthDuration(xs:duration(\"-P3Y0M31DT3H2M1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_211() {
    //  Simple test of casting a negative xs:dayTimeDuration to xs:yearMonthDuration.
    // XPath: xs:string(xs:yearMonthDuration(xs:dayTimeDuration(\"-P31DT3H
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_231() {
    //  Ensure casting xs:gYear to xs:string, with timezone '-00:00' is properly handle
    // XPath: xs:string(xs:gYear(\"1999-00:00\")) eq \"1999Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
