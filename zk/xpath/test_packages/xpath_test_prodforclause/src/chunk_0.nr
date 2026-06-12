//! Test chunk 0 for prod:ForClause
//! Contains 50 tests

use dep::xpath::stub_prodforclause;

#[test]
fn forexpr001() {
    //  For+Return - to iterate a constant sequence
    // XPath: for $a in (\"test\", \"test1\", \"test2\") return $a
    // Expected: test test1 test2
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr005() {
    //  For+Return - use sequence(security/right) as the predicate
    // XPath: for $f in /MyComputer//File[@creation_date=\"08/06/00\"]/Sec
    // Expected: so00000000001
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr012() {
    //  Nested FLWOR expressions
    // XPath: for $fileName in for $file in //Folder/File return $file/Fil
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr015() {
    //  Return expression contains nested for expression
    // XPath: for $folder in //Folder return for $file in $folder/File ret
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr021() {
    //  Test order of iteration between two variables in the same 'for' statement
    // XPath: for $i in (1, 2), $j in (3, 4) return ($i, $j)
    // Expected: 1 3 1 4 2 3 2 4
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr022() {
    //  FLWOR expressions with positional variable
    // XPath: for $file at $pos in (//Folder)[1]/File return (string($file
    // Expected: File00000000000 1 File00000000001 2 File
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr023() {
    //  Use positional variable in binding for new variable
    // XPath: for $file at $pos in (//Folder)[1]/File, $pos2 in $pos+1 ret
    // Expected: File00000000000 1 2 File00000000001 2 3
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr029() {
    //  Embedded FLOWR expression that binds same variable on boths expressions (two fo
    // XPath: for $var in (1,2) for $var in (2,2) return $var * $var
    // Expected: 4 4 4 4
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr030() {
    //  Embedded FLOWR expression that binds same variable on boths expressions (use co
    // XPath: for $var in (1,2), $var in (2,2) return $var * $var
    // Expected: 4 4 4 4
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn forexpr032() {
    //  FLOWR expression with positional binding
    // XPath: declare variable $j external := 1;          for $x at $i in
    // Expected: 2
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_5() {
    //  A test whose essence is: `deep-equal(1 to 5, (for $i in (1,2,3,4,5) return $i))
    // XPath: deep-equal(1 to 5, (for $i in (1,2,3,4,5) return $i))
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_6() {
    //  A test whose essence is: `3 eq (for $foo in 1 return 3)`.
    // XPath: 3 eq (for $foo in 1 return 3)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_7() {
    //  A test whose essence is: `1 eq (for $foo in 1 return $foo)`.
    // XPath: 1 eq (for $foo in 1 return $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_8() {
    //  A test whose essence is: `2 eq (for $foo in 1 return $foo + 1)`.
    // XPath: 2 eq (for $foo in 1 return $foo + 1)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_9() {
    //  A test whose essence is: `3 eq (for $foo in 1 return for $foo in 3 return $foo)
    // XPath: 3 eq (for $foo in 1 return for $foo in 3 return $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_10() {
    //  A test whose essence is: `deep-equal((for $var in (1, 2, 3) return $var), (1, 2
    // XPath: deep-equal((for $var in (1, 2, 3) return $var), (1, 2, 3))
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_11() {
    //  A test whose essence is: `deep-equal((2, 2), (for $foo in (1, 1) return 1 + $fo
    // XPath: deep-equal((2, 2), (for $foo in (1, 1) return 1 + $foo))
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_12() {
    //  A test whose essence is: `4 eq (for $foo in 1, $bar in 2, $moo in 3 return 4)`.
    // XPath: 4 eq (for $foo in 1, $bar in 2, $moo in 3 return 4)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_13() {
    //  A test whose essence is: `3 eq (for $foo in 1 return for $bar in 2 return $bar
    // XPath: 3 eq (for $foo in 1 return for $bar in 2 return $bar + $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_14() {
    //  A test whose essence is: `3 eq (for $foo in 1 return for $bar in 2 return $bar
    // XPath: 3 eq (for $foo in 1 return for $bar in 2 return $bar + $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_15() {
    //  A test whose essence is: `3 eq (for $foo in 1, $foo in 3 return $foo)`.
    // XPath: 3 eq (for $foo in 1, $foo in 3 return $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_16() {
    //  A test whose essence is: `8 eq (for $foo in 1, $foo in 3, $moo in 5 return $moo
    // XPath: 8 eq (for $foo in 1, $foo in 3, $moo in 5 return $moo + $foo
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_17() {
    //  A test whose essence is: `6 eq (for $foo in 1, $foo in 3 return $foo + $foo)`.
    // XPath: 6 eq (for $foo in 1, $foo in 3 return $foo + $foo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_22() {
    //  A for variable binding to the empty sequence, combined with value comparison.
    // XPath: empty(for $var in subsequence((current-time(), 1), 4) return
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_23() {
    //  An for-return expression which only is the binding variable.
    // XPath: deep-equal(remove(for $var in (current-time(), 2, 3, 4) retu
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_24() {
    //  A test whose essence is: `(for $fn:name in (1, 1) return $fn:name) instance of
    // XPath: (for $fn:name in (1, 1) return $fn:name) instance of xs:inte
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_25() {
    //  A test whose essence is: `(for $xs:name in (1, 1) return $xs:name) instance of
    // XPath: (for $xs:name in (1, 1) return $xs:name) instance of xs:inte
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_48() {
    //  Parser test: variable names in a for expression that only exist of one characte
    // XPath: 1 eq (for $a in 1 return $a)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_49() {
    //  For-expression involving a simple return statement that in some implementations
    // XPath: deep-equal((for $fo in trace((1, 2, 3), \"msg\") return $fo)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_50() {
    //  Ensure scanners/parsers accept small QNames in for-expressions.
    // XPath: 1 eq (for $a in 1 return $a)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_51() {
    //  Ensure scanners/parsers accept small QNames in for-expressions(#2).
    // XPath: 1 eq (for $xs:a in 1 return $xs:a)
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_52() {
    //  A binding in a for-expression shadows global variables.
    // XPath: declare variable $i := false();          declare variable $t
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_53() {
    //  Ensure the correct variable is used in an for-expression whose return sequence
    // XPath: declare variable $my := 3; (for $i in 1 return $my) eq 3
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_54() {
    //  Ensure the correct variable is used in an for-expression whose return sequence
    // XPath: declare variable $i := 3; (for $i in 1 return $i) eq 1
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_55() {
    //  Apply fn:count() to a for-expression.
    // XPath: count(for $i in (1, 2, current-time()) return ($i, $i)) eq 6
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k_forexprwithout_56() {
    //  Apply fn:count() to a for-expression(#2).
    // XPath: count(for $i in (1, 2, timezone-from-time(current-time())) r
    // Expected: true
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_9() {
    //  Ensure positional variables and bindings handles nesting.
    // XPath: for $a at $pos1 in (1, 2, 3) return for $b at $pos2 in (4, 5
    // Expected: 1 1 4 1 7 1 1 1 4 1 8 1 1 1 4 1 9 1 1 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_11() {
    //  The return expression does not depend on the for iteration.
    // XPath: for $a in (3, 2, 1, 1) stable order by $a return 5
    // Expected: 5 5 5 5
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_12() {
    //  The order by expression does not depend on the for iteration.
    // XPath: for $a in (3, 2, 1, 1) stable order by 1 return $a
    // Expected: 3 2 1 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_15() {
    //  Use a name test named 'node' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_16() {
    //  Use a name test named 'document' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_17() {
    //  Use a name test named 'document' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_18() {
    //  Use a name test named 'attribute' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_19() {
    //  Use a name test named 'comment' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_20() {
    //  Use a name test named 'processing-instruction' inside a for loop, inside a user
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_21() {
    //  Use a name test named 'processing-instruction' inside a for loop, inside a user
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_22() {
    //  Use a name test named 'text' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_23() {
    //  Use a name test named 'typeswitch' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_24() {
    //  Use a name test named 'if' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}

#[test]
fn k2_forexprwithout_25() {
    //  Use a name test named 'for' inside a for loop, inside a user function.
    // XPath: declare function local:func($arg as element()* ) as element(
    // Expected: 1
    // Calls stub function - will fail until prod:ForClause is implemented
    let _ = stub_prodforclause();
}
