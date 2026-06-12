//! Test chunk 2 for prod:QuantifiedExpr
//! Contains 40 tests

use dep::xpath::stub_prodquantifiedexpr;

#[test]
fn k_quantexprwithout_44() {
    //  A binding in a 'every' quantification shadows global variables.
    // XPath: declare variable $i := false();          declare variable $t
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_45() {
    //  A binding in a 'every' quantification shadows global variables.
    // XPath: declare variable $i := false();          every $i in (true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_46() {
    //  A test whose essence is: `some $var in (1, 2, 3) satisfies $var eq 3`.
    // XPath: some $var in (1, 2, 3) satisfies $var eq 3
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_47() {
    //  A test whose essence is: `not(every $var in (1, 2, 3) satisfies $var eq 3)`.
    // XPath: not(every $var in (1, 2, 3) satisfies $var eq 3)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_48() {
    //  A test whose essence is: `every $var in (1, 2, 3) satisfies $var eq 1 or $var e
    // XPath: every $var in (1, 2, 3) satisfies $var eq 1 or $var eq 2 or
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_49() {
    //  A test whose essence is: `some $var in (1, 2, 3) satisfies $var eq 1 or $var eq
    // XPath: some $var in (1, 2, 3) satisfies $var eq 1 or $var eq 2 or $
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_50() {
    //  A test whose essence is: `some $aaa in (1, 2, 3), $bbb in (3, 2, 1) satisfies $
    // XPath: some $aaa in (1, 2, 3), $bbb in (3, 2, 1) satisfies $aaa + $
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_51() {
    //  A test whose essence is: `every $aaa in (3, 3, 3), $bbb in (3, 3, 3) satisfies
    // XPath: every $aaa in (3, 3, 3), $bbb in (3, 3, 3) satisfies $aaa +
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_52() {
    //  A test whose essence is: `not(every $var in (1, 2, 3) satisfies $var eq 3)`.
    // XPath: not(every $var in (1, 2, 3) satisfies $var eq 3)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_53() {
    //  A test whose essence is: `some $var in (1, 2, 3) satisfies $var eq 3`.
    // XPath: some $var in (1, 2, 3) satisfies $var eq 3
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_54() {
    //  A test whose essence is: `true() eq (some $a in 1 satisfies $a)`.
    // XPath: true() eq (some $a in 1 satisfies $a)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_55() {
    //  A test whose essence is: `true() eq (every $a in 1 satisfies $a)`.
    // XPath: true() eq (every $a in 1 satisfies $a)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_56() {
    //  A test whose essence is: `true() eq (some $fn:name in (1, 2) satisfies $fn:name
    // XPath: true() eq (some $fn:name in (1, 2) satisfies $fn:name)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_57() {
    //  A test whose essence is: `true() eq (some $xs:name in (1, 2) satisfies $xs:name
    // XPath: true() eq (some $xs:name in (1, 2) satisfies $xs:name)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_58() {
    //  A test whose essence is: `true() eq (every $fn:name in (1, 2) satisfies $fn:nam
    // XPath: true() eq (every $fn:name in (1, 2) satisfies $fn:name)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_59() {
    //  A test whose essence is: `true() eq (every $xs:name in (1, 2) satisfies $xs:nam
    // XPath: true() eq (every $xs:name in (1, 2) satisfies $xs:name)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_89() {
    //  Nested variable bindings can reference each other.
    // XPath: every $a in (1, 2, 3), $b in ($a, 4) satisfies $b gt 0
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_90() {
    //  Nested variable bindings can reference each other.
    // XPath: some $a in (1, 2, 3), $b in ($a, 4) satisfies $b gt 0
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_91() {
    //  Nested variable bindings can reference each other.
    // XPath: every $a in (1, 2), $b in $a satisfies $b
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_92() {
    //  Nested variable bindings can reference each other.
    // XPath: some $a in (1, 2), $b in $a satisfies $b
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_93() {
    //  Nested variable bindings can reference each other.
    // XPath: deep-equal((for $a in 1, $b in $a, $c in $a, $d in $c return
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_3() {
    //  Ensure the focus is defined.
    // XPath: <people id=\"\"/>/(some $id in @id satisfies true())
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_5() {
    //  Use the focus from within a some-expression.
    // XPath: <e/>/(some $v in self::node() satisfies $v)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_6() {
    //  Use the focus from within a every-expression.
    // XPath: <e/>/(every $v in self::node() satisfies $v)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_7() {
    //  Use the focus from within a every-expression's tail.
    // XPath: <e/>/(every $b in 1, $v in self::node() satisfies $v)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_8() {
    //  Use the focus from within a some-expression's tail.
    // XPath: <e/>/(some $b in 1, $v in self::node() satisfies $v)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k2_quantexprwithout_9() {
    //  Combine some with fn:deep-equal().
    // XPath: let $firstSeq := (<a/>, <b/>, <e><c/></e>) let $secondSeq :=
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_1() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: every $a as item()* in (1, 2), $b as item()* in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_2() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: some $a as item()* in (1, 2), $b as item()* in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_3() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: every $a as item()? in (1, 2), $b as item()? in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_4() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: some $a as item()? in (1, 2), $b as item()? in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_5() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: every $a as item()+ in (1, 2), $b as item()+ in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_6() {
    //  Every-quantification carrying type declarations(cardinalities have no impact).
    // XPath: some $a as item()+ in (1, 2), $b as item()+ in $a satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_17() {
    //  Every-quantification carrying invalid type declarations.
    // XPath: every $a as xs:integer+ in (1, 2), $b as xs:integer in $a sa
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_18() {
    //  Every-quantification carrying invalid type declarations.
    // XPath: every $a as item()* in (1, 2), $b as xs:integer in $a satisf
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_23() {
    //  Every-quantification with type-declaration.
    // XPath: every $a as xs:integer in (1, 2), $b as xs:integer in $a sat
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwith_24() {
    //  Every-quantification with type-declaration.
    // XPath: some $a as xs:integer in (1, 2), $b as xs:integer in $a sati
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn cbcl_every_001() {
    //  Test optimization when the domain may be empty and return type is a value
    // XPath: every $x in (1 to 10)[. div 2 = 11] satisfies false()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn cbcl_some_001() {
    //  Test optimization where the domain may be empty and the return type a value
    // XPath: some $x in (1 to 10)[. div 2 = 11] satisfies true()
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn cbcl_every_002() {
    //  Tests optimization of every $x in .... satisfies true()
    // XPath: every $x in (1 to 10)[. mod 2 = 0] satisfies true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}
