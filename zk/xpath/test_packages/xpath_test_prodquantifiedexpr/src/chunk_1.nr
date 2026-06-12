//! Test chunk 1 for prod:QuantifiedExpr
//! Contains 50 tests

use dep::xpath::stub_prodquantifiedexpr;

#[test]
fn quantexpr_51() {
    //  Simple quantified expression using 'every' keyword with multiple variables and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies $x div $y = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_52() {
    //  Simple quantified expression using 'every' keyword with multiple variables and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies $x idiv $y = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_53() {
    //  Simple quantified expression using 'every' keyword with multiple variables, and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies fn:string($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_54() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies xs:integer($x)
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_55() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies xs:decimal($x)
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_56() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies xs:float($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_57() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies xs:double($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_58() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (\"true\", \"false\"), $y in (\"false\",\"true\"
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_59() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (\"1980-05-05T13:13:13Z\", \"1980-05-05T13:13:13
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_60() {
    //  Simple quantified expression using 'every' keyword, use of multiple variable an
    // XPath: every $x in (\"1985-07-05Z\", \"1985-07-05Z\"), $y in (\"198
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_61() {
    //  Simple quantified expression using 'some' keyword that binds the declared varia
    // XPath: some $x as xs:integer in (1, 2, 3) , $y as xs:integer in (2,
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_62() {
    //  Simple quantified expression using 'some' keyword that binds the declared varia
    // XPath: some $x as xs:string in (\"cat\",\"dog\",\"rat\") satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_63() {
    //  Simple quantified expression using 'every' keyword that binds the declared vari
    // XPath: every $x as xs:string in (\"cat\",\"dog\",\"rat\") satisfies
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_64() {
    //  Simple quantified expression using 'every' keyword that binds the declared vari
    // XPath: every $x as xs:string in (\"cat\",\"dog\",\"rat\"), $y as xs
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_65() {
    //  Simple quantified expression using 'some' keyword that binds the declared varia
    // XPath: some $x as xs:integer in (1, 2, 3), $y as xs:float in (xs:fl
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_67() {
    //  Another cose (added 'or' operator) which crashed Saxon
    // XPath: declare function local:f($x, $y) {             let $a := $x
    // Expected: x
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_68() {
    //  A variant of a case (added 'or' operator) which crashed Saxon
    // XPath: declare function local:f($x, $y) {             let $a := $x
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_5() {
    //  A some-quantification applied on fn:count().
    // XPath: some $i in (0, 2, 3) satisfies count($i)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_6() {
    //  A some-quantification applied on fn:count().
    // XPath: every $i in (1, 2, 3) satisfies count($i)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_7() {
    //  some-quantification combined with fn:boolean().
    // XPath: some $i in subsequence((0, 1, 2, current-time()), 1, 3) sati
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_8() {
    //  every-quantification combined with fn:boolean().
    // XPath: every $i in subsequence((1, 2, 3, current-time()), 1, 3) sat
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_9() {
    //  every-quantification combined with empty variable binding.
    // XPath: every $i in () satisfies $i
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_10() {
    //  some-quantification combined with empty variable binding.
    // XPath: not(some $i in () satisfies $i)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_11() {
    //  A test whose essence is: `some $foo in 1 satisfies true()`.
    // XPath: some $foo in 1 satisfies true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_12() {
    //  A test whose essence is: `every $foo in 1 satisfies true()`.
    // XPath: every $foo in 1 satisfies true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_13() {
    //  A test whose essence is: `not(some $foo in 1 satisfies false())`.
    // XPath: not(some $foo in 1 satisfies false())
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_14() {
    //  A test whose essence is: `not(every $foo in 1 satisfies false())`.
    // XPath: not(every $foo in 1 satisfies false())
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_15() {
    //  A test whose essence is: `some $foo in 1 satisfies 1`.
    // XPath: some $foo in 1 satisfies 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_16() {
    //  A test whose essence is: `every $foo in 1 satisfies 1`.
    // XPath: every $foo in 1 satisfies 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_17() {
    //  A test whose essence is: `some $foo in 1 satisfies 'a string'`.
    // XPath: some $foo in 1 satisfies \"a string\"
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_18() {
    //  A test whose essence is: `every $foo in 1 satisfies 'a string'`.
    // XPath: every $foo in 1 satisfies \"a string\"
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_19() {
    //  A test whose essence is: `every $var in (true(), true(), true()) satisfies $var
    // XPath: every $var in (true(), true(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_20() {
    //  A test whose essence is: `not(every $var in (true(), false(), true()) satisfies
    // XPath: not(every $var in (true(), false(), true()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_21() {
    //  A test whose essence is: `not(every $var in (false(), true(), true()) satisfies
    // XPath: not(every $var in (false(), true(), true()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_22() {
    //  A test whose essence is: `not(every $var in (true(), true(), false()) satisfies
    // XPath: not(every $var in (true(), true(), false()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_23() {
    //  A test whose essence is: `some $var in (true(), true(), true()) satisfies $var`
    // XPath: some $var in (true(), true(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_24() {
    //  A test whose essence is: `some $var in (true(), false(), true()) satisfies $var
    // XPath: some $var in (true(), false(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_25() {
    //  A test whose essence is: `some $var in (false(), true(), true()) satisfies $var
    // XPath: some $var in (false(), true(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_26() {
    //  A test whose essence is: `some $var in (true(), true(), false()) satisfies $var
    // XPath: some $var in (true(), true(), false()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_27() {
    //  A test whose essence is: `not(some $var in (false(), false(), false()) satisfie
    // XPath: not(some $var in (false(), false(), false()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_34() {
    //  A test whose essence is: `not(every $var in (false(), true(), true()) satisfies
    // XPath: not(every $var in (false(), true(), true()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_35() {
    //  A test whose essence is: `not(every $var in (true(), false(), true()) satisfies
    // XPath: not(every $var in (true(), false(), true()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_36() {
    //  A test whose essence is: `not(every $var in (true(), true(), false()) satisfies
    // XPath: not(every $var in (true(), true(), false()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_37() {
    //  A test whose essence is: `some $var in (true(), true(), true()) satisfies $var`
    // XPath: some $var in (true(), true(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_38() {
    //  A test whose essence is: `some $var in (true(), false(), true()) satisfies $var
    // XPath: some $var in (true(), false(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_39() {
    //  A test whose essence is: `some $var in (false(), true(), true()) satisfies $var
    // XPath: some $var in (false(), true(), true()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_40() {
    //  A test whose essence is: `some $var in (true(), true(), false()) satisfies $var
    // XPath: some $var in (true(), true(), false()) satisfies $var
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_41() {
    //  A test whose essence is: `not(some $var in (false(), false(), false()) satisfie
    // XPath: not(some $var in (false(), false(), false()) satisfies $var)
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_42() {
    //  A binding in a 'some' quantification shadows global variables.
    // XPath: declare variable $i := false();          some $i in (true(),
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn k_quantexprwithout_43() {
    //  A binding in a 'some' quantification shadows global variables.
    // XPath: declare variable $i := false();          declare variable $t
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}
