//! Test chunk 0 for prod:VarDecl
//! Contains 50 tests

use dep::xpath::stub_prodvardecl;

#[test]
fn vardecl001() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := \"\" ; $x
    // Expected:
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl002() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := '' ; $x
    // Expected:
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl003() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 'a string' ; $x
    // Expected: a string
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl004() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := \"a string\" ; $x
    // Expected: a string
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl005() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := \"This is a string, isn't it?\" ; $x
    // Expected: This is a string, isn't it?
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl006() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 'This is a \"String\"' ; $x
    // Expected: This is a "String"
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl007() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := \"a \"\" or a ' delimits a string lit
    // Expected: a " or a ' delimits a string literal
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl008() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 'a \" or a '' delimits a string liter
    // Expected: a " or a ' delimits a string literal
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl009() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := '&lt;bold&gt;A sample element.&lt;/bo
    // Expected: <bold>A sample element.</bold>
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl010() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 0 ; $x
    // Expected: 0
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl011() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 1 ; $x
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl012() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -1 ; $x
    // Expected: -1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl013() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := +1 ; $x
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl014() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 1.23 ; $x
    // Expected: 1.23
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl015() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -1.23 ; $x
    // Expected: -1.23
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl016() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 1.2e5 ; $x
    // Expected: 120000
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl017() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -1.2E5 ; $x
    // Expected: -120000
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl018() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 0.0E0 ; $x
    // Expected: 0
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl019() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 1e-5 ; $x
    // Expected: 0.00001
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl020() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 9.999999999999999; $x
    // Expected: 9.999999999999999
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl021() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -10000000 ; $x
    // Expected: -10000000
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl022() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 1 to 10 ; $x
    // Expected: 1 2 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl023() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:double('NaN'); $x
    // Expected: NaN
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl024() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:double('INF'); $x
    // Expected: INF
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl025() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:double('-INF'); $x
    // Expected: -INF
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl026() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := fn:false() ; $x
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl027() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := false(); $x
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl028() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := fn:true() ; $x
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl029() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := true() ; $x
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl030() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := true(); $x
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl031() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 2+2 ; $x
    // Expected: 4
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl032() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 2*2 ; $x
    // Expected: 4
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl033() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 3-2 ; $x
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl034() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 3 div 2 ; $x
    // Expected: 1.5
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl035() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 3 mod 2 ; $x
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl036() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 3 idiv 2 ; $x
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl037() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -1.7976931348623157E308 ; $x
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl038() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:double(\"-1.7976931348623157E308\"
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl039() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -999999999999999999 ; $x
    // Expected: -999999999999999999
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl040() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:decimal(\"-999999999999999999\") ;
    // Expected: -999999999999999999
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl041() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := 999999999999999999 ; $x
    // Expected: 999999999999999999
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl042() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:decimal(\"999999999999999999\") ;
    // Expected: 999999999999999999
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl043() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := -3.4028235E38 ; $x
    // Expected: -3.4028235E38
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl044() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:float(\"-3.4028235E38\") ; $x
    // Expected: -3.4028235E38
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl045() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:date(\"1970-01-01Z\") ; $x
    // Expected: 1970-01-01Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl046() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:date(\"1970-01-01Z\") + xs:dayTime
    // Expected: 1970-02-01Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl047() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:time(\"08:03:35Z\") ; $x
    // Expected: 08:03:35Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl048() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:time(\"08:03:35Z\") + xs:dayTimeDu
    // Expected: 08:03:35Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl049() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := xs:dateTime(\"2030-12-31T23:59:59Z\")
    // Expected: 2030-12-31T23:59:59Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl050() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (1,2,3) ; $x
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}
