//! Test chunk 0 for prod:TryCatchExpr
//! Contains 50 tests

use dep::xpath::stub_prodtrycatchexpr;

#[test]
fn try_001() {
    // XPath: try { doc('rubbish.xml') } catch * {\"ok\"}
    // Expected: ok
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_002() {
    // XPath: declare namespace err = \"http://www.w3.org/2005/xqt-errors\
    // Expected: ok
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_003() {
    // XPath: try { doc('rubbish.xml') } catch *:FODC0001 | *:FODC0002 | *
    // Expected: ok
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_008() {
    // does catch error evaluating function call
    // XPath: declare function local:f() { doc('rubbish.xml') };
    // Expected: ok
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_009() {
    // does catch error evaluating function call
    // XPath: declare function local:f($d as xs:integer) { 10 div $d };
    // Expected: ok
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_010() {
    // Use err:code local variable, implicitly declared
    // XPath: declare function local:f($d as xs:integer) { 10 div $d };
    // Expected: FOAR0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_011() {
    // Use err:code local variable in a nested tryCatch. The cast as expr should throw
    // XPath: declare function local:f($d as xs:integer) { 10 div $d };
    // Expected: FORG0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_014() {
    // Integer literal out of range is a dynamic error
    // XPath: try { 999999999999999999999999999999999999999999999999999999
    // Expected: 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_015() {
    // Bad date is a dynamic error
    // XPath: try { xs:date('2013-02-29') }           catch err:FORG0001 {
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_016() {
    // No context item is a dynamic error
    // XPath: declare function local:f() { .+3 };       try {local:f()} ca
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_018() {
    // Test that the right catch clause is evaluated. Saxon bug report from Florent Geo
    // XPath: try {1 div 0}       catch local:foo | local:bar {'first'}
    // Expected: "second"
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_dynamic_error_1() {
    // A division by zero (dynamic error) is caught.
    // XPath: try { 1 div 0 } catch * { \"Division by zero\" }
    // Expected: "Division by zero"
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_dynamic_error_2() {
    // A division by zero (dynamic error) is caught when the error is specified.
    // XPath: try { 1 div 0 } catch err:FOAR0001 { \"Division by zero\" }
    // Expected: "Division by zero"
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_dynamic_error_3() {
    // A division by zero (dynamic error) is caught when the error namespace is specifi
    // XPath: try { 1 div 0 } catch err:* { \"Division by zero\" }
    // Expected: "Division by zero"
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_dynamic_error_4() {
    // A division by zero (dynamic error) is caught when the error local name is specif
    // XPath: try { 1 div 0 } catch *:FOAR0001 { \"Division by zero\" }
    // Expected: "Division by zero"
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_no_error_1() {
    // If no error is raised, the result of the try catch expression is the result of t
    // XPath: try { 0 } catch * { \"No error\" }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_no_error_2() {
    // If no error is raised, the result of the try catch expression is the result of t
    // XPath: try { 0 } catch err:FOAR0001 { \"No error\" }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_no_error_3() {
    // If no error is raised, the result of the try catch expression is the result of t
    // XPath: try { 0 } catch err:* { \"No error\" }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_no_error_4() {
    // If no error is raised, the result of the try catch expression is the result of t
    // XPath: try { 0 } catch *:FOAR0001 { \"No error\" }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_no_error_5() {
    // If no error is raised, the result of the try catch expression is the result of t
    // XPath: try { 0 } catch err:FOAR0001 { \"No error\" } catch *:FOAR00
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_name_tests_namespace_1() {
    // URI literals are supported in name tests.
    // XPath: try { 1 div 0 } catch Q{http://www.w3.org/2005/xqt-errors}FO
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_name_tests_namespace_2() {
    // URI literals are supported in name tests.
    // XPath: try { 1 div 0 } catch Q{http://www.w3.org/2005/xqt-errors}*
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_several_name_tests_1() {
    // A catch clause with one or more NameTests matches any error whose error code mat
    // XPath: try { 1 div 0 } catch err:XPST0008 | err:FOAR0001 { \"Divisi
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_several_name_tests_2() {
    // A catch clause with one or more NameTests matches any error whose error code mat
    // XPath: try { 1 div 0 } catch err:FOAR0001 | err:XPST0008 { \"Divisi
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_several_name_tests_3() {
    // A catch clause with one or more NameTests matches any error whose error code mat
    // XPath: try { 1 div 0 } catch err:XPTY0004 | err:FOAR0001 | err:XPST
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_1() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:FOAR0001 { \"Clause 1\" } catch er
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_2() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:XQST008 { \"Clause 1\" } catch err
    // Expected: Clause 2
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_3() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:FOAR0001 { \"Clause 1\" } catch er
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_4() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:FOAR0001 { \"Clause 1\" } catch er
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_5() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:FOAR0001 { \"Clause 1\" } catch er
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_6() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:XPTY0004 { \"Clause 1\" } catch er
    // Expected: Clause 2
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_7() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:XPTY0004 { \"Clause 1\" } catch er
    // Expected: Clause 3
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_8() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch err:* { \"Clause 1\" } catch xs:* { \"
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_9() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch xs:* { \"Clause 1\" } catch err:* { \"
    // Expected: Clause 2
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_10() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch Q{http://www.w3.org/2005/xqt-errors}*
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_11() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch Q{http://www.w3.org/2001/XMLSchema}* {
    // Expected: Clause 2
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_first_matching_catch_clause_12() {
    // The first matching catch clause is used.
    // XPath: try { 1 div 0 } catch * { \"Clause 1\" } catch err:FOAR0001
    // Expected: Clause 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_1() {
    // err:code returns the error code.
    // XPath: try { 1 div 0 } catch * { $err:code }
    // Expected: err:FOAR0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_2() {
    // err:code returns the error code.
    // XPath: try { 1 div 0 } catch * { fn:prefix-from-QName($err:code) }
    // Expected: err
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_3() {
    // err:code returns the error code.
    // XPath: try { 1 div 0 } catch * { fn:local-name-from-QName($err:code
    // Expected: FOAR0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_4() {
    // err:code returns the error code.
    // XPath: try { 1 div 0 } catch * { fn:namespace-uri-from-QName($err:c
    // Expected: http://www.w3.org/2005/xqt-errors
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_5() {
    // err:code returns the error code.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: err:FOER0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_6() {
    // err:code returns the error code.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: err
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_7() {
    // err:code returns the error code.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: FOER0001
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_8() {
    // err:code returns the error code.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: http://www.w3.org/2005/xqt-errors
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_9() {
    // err:code returns the error code.
    // XPath: try { fn:error() } catch * { $err:code }
    // Expected: err:FOER0000
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_10() {
    // err:code returns the error code.
    // XPath: try { fn:error() } catch * { fn:prefix-from-QName($err:code)
    // Expected: err
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_11() {
    // err:code returns the error code.
    // XPath: try { fn:error() } catch * { fn:local-name-from-QName($err:c
    // Expected: FOER0000
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_12() {
    // err:code returns the error code.
    // XPath: try { fn:error() } catch * { fn:namespace-uri-from-QName($er
    // Expected: http://www.w3.org/2005/xqt-errors
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_code_variable_14() {
    //          err:code used in a loop (test case from Gunther Rademacher).         Se
    // XPath: for $p in (\"p1\", \"p2\") return          try {parse-xml($p
    // Expected: err:FODC0006 err:FODC0006
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}
