//! Test chunk 7 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn castas684() {
    // Negative timezone with zero hour part
    // XPath: xs:gMonth('--07-00:30')
    // Expected: --07-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas685() {
    // Negative timezone with zero hour part
    // XPath: xs:gMonthDay('--07-12-00:30')
    // Expected: --07-12-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas686() {
    // Negative timezone with zero hour part
    // XPath: xs:gDay('---12-00:30')
    // Expected: ---12-00:30
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castas687() {
    // Cast a large xs:decimal value to xs:integer
    // XPath: xs:integer(12345678901234567.3)
    // Expected: 12345678901234567
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn castasnamespacesensitivetype_3() {
    // Namespace-sensitive type during a function call (without cast) is allowed.
    // XPath: declare function local:clarkname($q as xs:QName) as xs:strin
    // Expected: {http://www.w3.org/2001/XMLSchema}intege
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_14() {
    //  A test whose essence is: `'1' cast as xs:boolean eq true()`.
    // XPath: '1' cast as xs:boolean eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_15() {
    //  A test whose essence is: `'true' cast as xs:boolean eq true()`.
    // XPath: 'true' cast as xs:boolean eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_16() {
    //  A test whose essence is: `'false' cast as xs:boolean eq false()`.
    // XPath: 'false' cast as xs:boolean eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_17() {
    //  A test whose essence is: `'0' cast as xs:boolean eq false()`.
    // XPath: '0' cast as xs:boolean eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_18() {
    //  A test whose essence is: `false() cast as xs:boolean eq false()`.
    // XPath: false() cast as xs:boolean eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_19() {
    //  A test whose essence is: `not(true() cast as xs:boolean eq false())`.
    // XPath: not(true() cast as xs:boolean eq false())
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_20() {
    //  A test whose essence is: `xs:boolean(xs:double(3)) eq true()`.
    // XPath: xs:boolean(xs:double(3)) eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_21() {
    //  A test whose essence is: `xs:boolean(xs:string(1)) eq true()`.
    // XPath: xs:boolean(xs:string(1)) eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_22() {
    //  A test whose essence is: `xs:boolean(xs:integer(3)) eq true()`.
    // XPath: xs:boolean(xs:integer(3)) eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_23() {
    //  A test whose essence is: `xs:boolean(xs:decimal(3)) eq true()`.
    // XPath: xs:boolean(xs:decimal(3)) eq true()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_24() {
    //  A test whose essence is: `xs:boolean(xs:double(0)) eq false()`.
    // XPath: xs:boolean(xs:double(0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_25() {
    //  A test whose essence is: `xs:boolean(xs:string(0)) eq false()`.
    // XPath: xs:boolean(xs:string(0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_26() {
    //  A test whose essence is: `xs:boolean(xs:integer(0)) eq false()`.
    // XPath: xs:boolean(xs:integer(0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_27() {
    //  A test whose essence is: `xs:boolean(xs:decimal(0)) eq false()`.
    // XPath: xs:boolean(xs:decimal(0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_28() {
    //  A test whose essence is: `xs:boolean(xs:decimal(0.0)) eq false()`.
    // XPath: xs:boolean(xs:decimal(0.0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_29() {
    //  A test whose essence is: `xs:boolean(+0) eq false()`.
    // XPath: xs:boolean(+0) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_30() {
    //  A test whose essence is: `xs:boolean(-0) eq false()`.
    // XPath: xs:boolean(-0) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_31() {
    //  A test whose essence is: `xs:boolean(xs:double(+0)) eq false()`.
    // XPath: xs:boolean(xs:double(+0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_32() {
    //  A test whose essence is: `xs:boolean(xs:double(-0)) eq false()`.
    // XPath: xs:boolean(xs:double(-0)) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_33() {
    //  A test whose essence is: `xs:boolean(0.0E0) eq false()`.
    // XPath: xs:boolean(0.0E0) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_34() {
    //  A test whose essence is: `xs:boolean(xs:double('NaN')) eq false()`.
    // XPath: xs:boolean(xs:double(\"NaN\")) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_35() {
    //  A test whose essence is: `xs:boolean(xs:float('NaN')) eq false()`.
    // XPath: xs:boolean(xs:float(\"NaN\")) eq false()
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_37() {
    //  A test whose essence is: `false() cast as xs:string eq 'false'`.
    // XPath: false() cast as xs:string eq \"false\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_38() {
    //  A test whose essence is: `true() cast as xs:string eq 'true'`.
    // XPath: true() cast as xs:string eq \"true\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_39() {
    //  A test whose essence is: `('1' cast as xs:boolean) cast as xs:string eq 'true'`
    // XPath: ('1' cast as xs:boolean) cast as xs:string eq 'true'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_40() {
    //  A test whose essence is: `xs:boolean('0') cast as xs:string eq 'false'`.
    // XPath: xs:boolean('0') cast as xs:string eq 'false'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_41() {
    //  A test whose essence is: `' white space left alone' cast as xs:string eq xs:str
    // XPath: \" white space left alone\" cast as xs:string eq xs:string(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_42() {
    //  A test whose essence is: `(xs:QName('fn:local-name') cast as xs:string) eq 'fn:
    // XPath: (xs:QName(\"fn:local-name\") cast as xs:string) eq \"fn:loca
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_43() {
    //  A test whose essence is: `(xs:QName('local-name') cast as xs:string) eq 'local-
    // XPath: (xs:QName(\"local-name\") cast as xs:string) eq \"local-name
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_44() {
    //  A test whose essence is: `xs:string(xs:anyURI('example.com/')) eq 'example.com/
    // XPath: xs:string(xs:anyURI('example.com/')) eq 'example.com/'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_45() {
    //  A test whose essence is: `xs:string(xs:double(3)) eq '3'`.
    // XPath: xs:string(xs:double(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_46() {
    //  A test whose essence is: `xs:string(xs:string(3)) eq '3'`.
    // XPath: xs:string(xs:string(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_47() {
    //  A test whose essence is: `xs:string(xs:integer(3)) eq '3'`.
    // XPath: xs:string(xs:integer(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_48() {
    //  A test whose essence is: `xs:string(xs:decimal(3)) eq '3'`.
    // XPath: xs:string(xs:decimal(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_49() {
    //  A test whose essence is: `'untyped a ' cast as xs:untypedAtomic eq xs:untypedAt
    // XPath: \"untyped a \" cast as xs:untypedAtomic eq xs:untypedAtomic(
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_50() {
    //  A test whose essence is: `false() cast as xs:untypedAtomic eq xs:untypedAtomic(
    // XPath: false() cast as xs:untypedAtomic eq xs:untypedAtomic(\"false
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_51() {
    //  A test whose essence is: `true() cast as xs:untypedAtomic eq xs:untypedAtomic('
    // XPath: true() cast as xs:untypedAtomic eq xs:untypedAtomic(\"true\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_52() {
    //  A test whose essence is: `false() cast as xs:untypedAtomic instance of xs:untyp
    // XPath: false() cast as xs:untypedAtomic instance of xs:untypedAtomi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_53() {
    //  A test whose essence is: `true() cast as xs:untypedAtomic instance of xs:untype
    // XPath: true() cast as xs:untypedAtomic instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_54() {
    //  A test whose essence is: `(xs:QName('local-name') cast as xs:untypedAtomic) eq
    // XPath: (xs:QName(\"local-name\") cast as xs:untypedAtomic) eq xs:un
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_55() {
    //  A test whose essence is: `(xs:QName('fn:local-name') cast as xs:untypedAtomic)
    // XPath: (xs:QName(\"fn:local-name\") cast as xs:untypedAtomic) eq xs
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_56() {
    //  A test whose essence is: `xs:untypedAtomic(xs:anyURI('example.com/')) eq 'examp
    // XPath: xs:untypedAtomic(xs:anyURI('example.com/')) eq 'example.com/
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_57() {
    //  A test whose essence is: `xs:untypedAtomic(xs:double(3)) eq '3'`.
    // XPath: xs:untypedAtomic(xs:double(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_58() {
    //  A test whose essence is: `xs:untypedAtomic(xs:string(3)) eq '3'`.
    // XPath: xs:untypedAtomic(xs:string(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_59() {
    //  A test whose essence is: `xs:untypedAtomic(xs:integer(3)) eq '3'`.
    // XPath: xs:untypedAtomic(xs:integer(3)) eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
