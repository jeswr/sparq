//! Test chunk 1 for prod:TryCatchExpr
//! Contains 50 tests

use dep::xpath::stub_prodtrycatchexpr;

#[test]
fn try_catch_err_description_variable_1() {
    // err:description returns a description string or the empty sequence.
    // XPath: try { 1 div 0 } catch * { count($err:description) le 1 }
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_description_variable_2() {
    // err:code returns a description string or the empty sequence.
    // XPath: try { 1 div 0 } catch * { if (($err:description)) then $err:
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_description_variable_3() {
    // err:code returns a description string or the empty sequence.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: Description
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_value_variable_1() {
    // err:value returns a sequence of error values.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: Value
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_value_variable_2() {
    // err:value returns a sequence of error values.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: 4
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_value_variable_3() {
    // err:value returns a sequence of error values.
    // XPath: try { fn:error(fn:QName('http://www.w3.org/2005/xqt-errors',
    // Expected: 3
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_module_variable_1() {
    // err:module must be defined.
    // XPath: try { fn:error() } catch * { let $n := $err:module return tr
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_line_number_variable_1() {
    // err:line-number must be defined.
    // XPath: try { fn:error() } catch * { let $n := $err:line-number retu
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_column_number_variable_1() {
    // err:column-number must be defined.
    // XPath: try { fn:error() } catch * { let $n := $err:column-number re
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_dynamic_error_in_catch_clause_3() {
    // A dynamic error raised by a non-executed catch clause must be ignored.
    // XPath: try { 0 } catch err:FOER0000 { fn:error() }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_dynamic_error_in_catch_clause_4() {
    // A dynamic error raised by a non-executed catch clause must be ignored.
    // XPath: try { fn:error() } catch err:FOER0001 { fn:error() } catch e
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_err_dynamic_error_in_catch_clause_5() {
    // A dynamic error raised by a non-executed catch clause must be ignored.
    // XPath: try { fn:error() } catch err:FOER0000 { 0 } catch err:FOER00
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_function_call_1() {
    // If a function call occurs within a try clause, errors raised by evaluating the c
    // XPath: try { fn:error() } catch err:FOER0000 { 0 }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_function_call_2() {
    // If a function call occurs within a try clause, errors raised by evaluating the c
    // XPath: try { fn:one-or-more(()) } catch err:FORG0004 { 0 }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_function_call_3() {
    // If a function call occurs within a try clause, errors raised by evaluating the c
    // XPath: try { function() { fn:error() } () } catch err:FOER0000 { 0
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_function_call_4() {
    // If a function call occurs within a try clause, errors raised by evaluating the c
    // XPath: try { function() { 1 div 0 } () } catch err:FOAR0001 { 0 }
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_error_function_with_static_error_code() {
    // An error raised with fn:error() must be handled as a dynamic error.
    // XPath: try { fn:error(xs:QName(\"err:XPST0008\")) } catch err:XPST0
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_variable_binding_outside_4() {
    // If a variable reference is used in a try clause, errors raised by binding a valu
    // XPath: declare variable $str external := \"\";        try {
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_spec_example_1() {
    // Example from the specification.
    // XPath: let $x := \"\" return try {       $x cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_spec_example_2() {
    // Example from the specification.
    // XPath: let $x := \"\" return try {       $x cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_spec_example_3() {
    // Example from the specification.
    // XPath: let $x := \"\" return try {       $x cast as xs:integer
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_spec_example_4() {
    // Example from the specification (must parse).
    // XPath: let $output := (try {           fn:error(fn:QName('http://ww
    // Expected: true
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_1() {
    // XPDY0002 must be caught.
    // XPath: try { . } catch err:XPDY0002 { \"Context item not set.\" }
    // Expected: Context item not set.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_2() {
    // XPDY0050 must be caught.
    // XPath: try { \"\" treat as element() } catch err:XPDY0050 { \"Seque
    // Expected: Sequence type mismatch.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_3() {
    // XQDY0025 must be caught.
    // XPath: try { element  element { attribute a {\"\"}, attribute a {\"
    // Expected: Attribute name duplicate.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_4() {
    // XQDY0026 must be caught.
    // XPath: try { processing-instruction name { \"?>\" } } catch err:XQD
    // Expected: Invalid PI.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_5() {
    // XQDY0041 must be caught.
    // XPath: try { processing-instruction  { \"prefix:name\" } {} } catch
    // Expected: Invalid PI.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_6() {
    // XQDY0044 must be caught.
    // XPath: try { attribute xmlns {} } catch err:XQDY0044 { \"Invalid at
    // Expected: Invalid attribute.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_8() {
    // XQDY0064 must be caught.
    // XPath: try { processing-instruction XML {} } catch err:XQDY0064 { \
    // Expected: Invalid PI.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_9() {
    // XQDY0072 must be caught.
    // XPath: try { comment { \"--\" } } catch err:XQDY0072 { \"Invalid co
    // Expected: Invalid comment.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_10() {
    // XQDY0074 must be caught.
    // XPath: try { element { \"prefix:name\" } {} } catch err:XQDY0074 {
    // Expected: Invalid element.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_13() {
    // XQDY0096 must be caught.
    // XPath: try { element { \"xmlns:name\" } {} } catch err:XQDY0096 | e
    // Expected: Invalid element.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_13b() {
    // XQDY0096 must be caught.
    // XPath: try { element { QName(\"http://www.w3.org/2000/xmlns/\", \"x
    // Expected: Invalid element.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_14() {
    // XQDY0101 must be caught.
    // XPath: try { namespace xmlns { \"http://www.example.com\" } } catch
    // Expected: Invalid namespace node.
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_all_dynamic_errors_caught_15() {
    // FAOR0002 must be caught, even when detectable statically.
    // XPath: try { 10000000000000000000000000001 - 1000000000000000000000
    // Expected: 1
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_nest_1() {
    // Try-catch expressions must nest.
    // XPath: try { try { 1 div 0 } catch * { \"Division by zero\" } } cat
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_nest_2() {
    // Try-catch expressions must nest.
    // XPath: try { fn:error() } catch * { try { 1 div 0 } catch * { \"Div
    // Expected: Division by zero
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_2() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_3() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_4() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_5() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_7() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 'http://www.example.com/'
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_8() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 'example'
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_9() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 'EXER3141'
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_10() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 'Description'
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_11() {
    // Try-catch must match fn:error semantics.
    // XPath: try { fn:error(fn:QName(\"http://www.example.com/\", \"examp
    // Expected: 2
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_12() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XPTY0004\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_13() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XPST0001\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_14() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XPDY0002\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}

#[test]
fn try_catch_fn_error_15() {
    // Try-catch must match any error raised by fn:error.
    // XPath: try { fn:error(xs:QName(\"err:XQTY0105\")) } catch Q{http://
    // Expected: 0
    // Calls stub function - will fail until prod:TryCatchExpr is implemented
    let _ = stub_prodtrycatchexpr();
}
