//! Test chunk 0 for prod:QuantifiedExpr
//! Contains 50 tests

use dep::xpath::stub_prodquantifiedexpr;

#[test]
fn quantexpr_1() {
    //  Simple quantified expression using 'some' keyword and addition expression. Retu
    // XPath: some $x in (1, 2) satisfies $x + $x = 3
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_2() {
    //  Simple quantified expression using 'some' keyword and addition expression. Retu
    // XPath: some $x in (1, 2) satisfies $x + $x = 2
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_3() {
    //  Simple quantified expression using 'some' keyword and a subtraction expression.
    // XPath: some $x in (1, 2) satisfies $x - 2 = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_4() {
    //  Simple quantified expression using 'some' keyword and a subtraction expression.
    // XPath: some $x in (1, 2) satisfies $x - 2 = 0
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_5() {
    //  Simple quantified expression using 'some' keyword and a multiplication expressi
    // XPath: some $x in (1, 2) satisfies $x * 2 = 4
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_6() {
    //  Simple quantified expression using 'some' keyword and a division expression.
    // XPath: some $x in (1, 2) satisfies $x div 2 = 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_7() {
    //  Simple quantified expression using 'some' keyword and a division (idiv) express
    // XPath: some $x in (1, 2) satisfies $x idiv 2 = 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_8() {
    //  Simple quantified expression using 'some' keyword and usage of avg function exp
    // XPath: some $x in (1, 2) satisfies fn:avg(($x, 1)) = 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_9() {
    //  Simple quantified expression using 'some' keyword and usage of string function
    // XPath: some $x in (1, 2) satisfies fn:string($x) = \"1\"
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_10() {
    //  Simple quantified expression using 'some' keyword and usage of string-length fu
    // XPath: some $x in (1, 2) satisfies fn:string-length(xs:string($x))
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_11() {
    //  Simple quantified expression using 'some' keyword and usage of 'fn:count' funct
    // XPath: some $x in (1, 2) satisfies fn:count(($x)) = 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_12() {
    //  Simple quantified expression using 'some' keyword and usage of 'fn:true' functi
    // XPath: some $x in (1, 2) satisfies fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_13() {
    //  Simple quantified expression using 'some' keyword and usage of 'fn:false' funct
    // XPath: some $x in (1, 2) satisfies fn:false()
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_14() {
    //  Simple quantified expression using 'some' keyword and usage of 'fn:not' functio
    // XPath: some $x in (1, 2) satisfies fn:not($x)
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_15() {
    //  Simple quantified expression using 'some' keyword and use of lofical expression
    // XPath: some $x in (1, 2) satisfies $x = 1 or $x = 2
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_16() {
    //  Simple quantified expression using 'some' keyword and use of lofical expression
    // XPath: some $x in (1, 2) satisfies $x = 1 and ($x +1) = 2
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_17() {
    //  Simple quantified expression using 'some' keyword and and use of string data.
    // XPath: some $x in (\"A\",\"B\",\"C\") satisfies $x = \"A\"
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_18() {
    //  Simple quantified expression using 'some' keyword with multiple variables and a
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies $x + $y = 5
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_19() {
    //  Simple quantified expression using 'some' keyword with multiple variables and s
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies $x - $y = 5
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_20() {
    //  Simple quantified expression using 'some' keyword with multiple variables and m
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies $x * $y = 10
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_21() {
    //  Simple quantified expression using 'some' keyword with multiple variables and d
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies $x div $y = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_22() {
    //  Simple quantified expression using 'some' keyword with multiple variables and d
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies $x idiv $y = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_23() {
    //  Simple quantified expression using 'some' keyword with multiple variables, and
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies fn:string($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_24() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies xs:integer($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_25() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies xs:decimal($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_26() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies xs:float($x) = x
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_27() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (1,2,3), $y in (4,5,6) satisfies xs:double($x) =
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_28() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (\"true\", \"false\"), $y in (\"false\",\"true\")
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_29() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (\"1980-05-05T13:13:13Z\", \"1980-05-05T13:13:13Z
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_30() {
    //  Simple quantified expression using 'some' keyword, use of multiple variable and
    // XPath: some $x in (\"1985-07-05Z\", \"1985-07-05Z\"), $y in (\"1985
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_31() {
    //  Simple quantified expression using 'every' keyword and addition expression. Ret
    // XPath: every $x in (1, 2) satisfies $x + $x = 3
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_32() {
    //  Simple quantified expression using 'every' keyword and addition expression. Ret
    // XPath: every $x in (1, 1) satisfies $x + $x = 2
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_33() {
    //  Simple quantified expression using 'every' keyword and a subtraction expression
    // XPath: every $x in (1, 2) satisfies $x - 2 = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_34() {
    //  Simple quantified expression using 'every' keyword and a subtraction expression
    // XPath: every $x in (2, 2) satisfies $x - 2 = 0
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_35() {
    //  Simple quantified expression using 'every' keyword and a multiplication express
    // XPath: every $x in (1, 2) satisfies $x * 2 = 4
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_36() {
    //  Simple quantified expression using 'every' keyword and a division expression.
    // XPath: every $x in (1, 2) satisfies $x div 2 = 1
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_37() {
    //  Simple quantified expression using 'every' keyword and a division (idiv) expres
    // XPath: every $x in (1, 2) satisfies $x idiv 2 = 1
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_38() {
    //  Simple quantified expression using 'every' keyword and usage of avg function ex
    // XPath: every $x in (1, 2) satisfies fn:avg(($x, 1)) = 1
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_39() {
    //  Simple quantified expression using 'every' keyword and usage of string function
    // XPath: every $x in (1, 2) satisfies fn:string($x) = \"1\"
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_40() {
    //  Simple quantified expression using 'every' keyword and usage of string-length f
    // XPath: every $x in (1, 2) satisfies fn:string-length(xs:string($x))
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_41() {
    //  Simple quantified expression using 'every' keyword and usage of 'fn:count' func
    // XPath: every $x in (1, 2) satisfies fn:count(($x)) = 1
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_42() {
    //  Simple quantified expression using 'every' keyword and usage of 'fn:true' funct
    // XPath: every $x in (1, 2) satisfies fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_43() {
    //  Simple quantified expression using 'some' keyword and usage of 'fn:false' funct
    // XPath: some $x in (1, 2) satisfies fn:false()
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_44() {
    //  Simple quantified expression using 'every' keyword and usage of 'fn:not' functi
    // XPath: every $x in (1, 2) satisfies fn:not($x)
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_45() {
    //  Simple quantified expression using 'every' keyword and use of lofical expressio
    // XPath: every $x in (1, 2) satisfies $x = 1 or $x = 2
    // Expected: true
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_46() {
    //  Simple quantified expression using 'every' keyword and use of lofical expressio
    // XPath: every $x in (1, 2) satisfies $x = 1 and ($x +1) = 2
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_47() {
    //  Simple quantified expression using 'every' keyword and and use of string data.
    // XPath: every $x in (\"A\",\"B\",\"C\") satisfies $x = \"A\"
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_48() {
    //  Simple quantified expression using 'every' keyword with multiple variables and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies $x + $y = 5
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_49() {
    //  Simple quantified expression using 'every' keyword with multiple variables and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies $x - $y = 5
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}

#[test]
fn quantexpr_50() {
    //  Simple quantified expression using 'every' keyword with multiple variables and
    // XPath: every $x in (1,2,3), $y in (4,5,6) satisfies $x * $y = 10
    // Expected: false
    // Calls stub function - will fail until prod:QuantifiedExpr is implemented
    let _ = stub_prodquantifiedexpr();
}
