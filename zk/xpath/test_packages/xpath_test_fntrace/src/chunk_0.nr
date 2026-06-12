//! Test chunk 0 for fn:trace
//! Contains 20 tests

use dep::xpath::stub_fntrace;

#[test]
fn fn_trace_2() {
    //  Simple call of 'fn:trace' function with integer value.
    // XPath: let $var := xs:integer(\"123\")           return fn:trace($v
    // Expected: 123
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_3() {
    //  Simple call of 'fn:trace' function used with an addition operation.
    // XPath: for $var in (1,2,3,4,5) return fn:trace($var + 1,\"The Value
    // Expected: 2 3 4 5 6
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_4() {
    //  Simple call of 'fn:trace' function used with a subtraction operation.
    // XPath: for $var in (2,3,4,5) return fn:trace($var - 1,\"The Value o
    // Expected: 1 2 3 4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_5() {
    //  Simple call of 'fn:trace' function used with a multiplication operation.
    // XPath: for $var in (2,3,4,5) return fn:trace($var * 2 ,\"The Value
    // Expected: 4 6 8 10
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_6() {
    //  Simple call of 'fn:trace' function used with a division (div operator) operatio
    // XPath: for $var in (2,4,6,8) return fn:trace($var div 2 ,\"The Valu
    // Expected: 1 2 3 4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_7() {
    //  Simple call of 'fn:trace' function used with a division (idiv operator) operati
    // XPath: for $var in (2,4,6,8) return fn:trace($var idiv 2 ,\"The Val
    // Expected: 1 2 3 4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_8() {
    //  Simple call of 'fn:trace' function used with a modulus operation.
    // XPath: for $var in (2,4,6,8) return fn:trace($var mod 2 ,\"The Valu
    // Expected: 0 0 0 0
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_9() {
    //  Simple call of 'fn:trace' function used with two variables, where the second on
    // XPath: for $var1 in (2,4,6,8), $var2 in (3 + $var1) - (4 * $var1)
    // Expected: -1 -5 -9 -13
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_10() {
    //  Simple call of 'fn:trace' function used with string manipulation (fn:concat).
    // XPath: for $var in (\"aa\",\"bb\",\"cc\",\"dd\",\"ee\") return fn:t
    // Expected: aaaa bbbb cccc dddd eeee
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_11() {
    //  Simple call of 'fn:trace' function used numbers manipulation queried from an xm
    // XPath: for $var in (/works//hours) return fn:trace(($var div 2) ,\"
    // Expected: 20 35 10 40 10 20 10 15 6 20 40 10 10 10
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_12() {
    //  Simple call of 'fn:trace' function used numbers manipulation queried from an xm
    // XPath: fn:trace((for $var in (/works//hours) return $var + $var) ,\
    // Expected: 80 140 40 160 40 80 40 60 24 80 160 40 4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_13() {
    //  Simple call of 'fn:trace' function used in a math expression involving the 'avg
    // XPath: fn:trace((fn:avg((1,3,3,4,5)) * 2) ,\"The Value of 'fn:avg((
    // Expected: 6.4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_14() {
    //  Simple call of 'fn:trace' function used in expression involving casting (floats
    // XPath: fn:trace((for $var in (1.1,2.2,3.3,4.4,5.5) return xs:intege
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_15() {
    //  Simple call of 'fn:trace' function that uses another fn-trace as argument.
    // XPath: fn:trace((fn:trace((2+2),\"The value of '2 + 2' is:\" )) ,\"
    // Expected: 4
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_16() {
    //  Simple call of 'fn:trace' function used in a math expression involving the 'abs
    // XPath: fn:trace((fn:count((1,2,-3,-4,5)) * 2) ,\"The Value of 'fn:c
    // Expected: 10
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_17() {
    //  Simple call of 'fn:trace' function used in a math expression involving boolean
    // XPath: fn:trace((for $var in (fn:true(),fn:false(),fn:true()) retur
    // Expected: true false true
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_18() {
    //  Simple call of 'fn:trace' function used in a math expression involving boolean
    // XPath: fn:trace((for $var in (fn:true(),fn:false(),fn:true()) retur
    // Expected: true true true
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_20() {
    //  Simple call of 'fn:trace' function where the first argument is the zero length
    // XPath: fn:count(fn:trace(\"\" ,\"The value of the zero length strin
    // Expected: 1
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn fn_trace_21() {
    //  Simple call of 'fn:trace' function involving string manipulations with upper/lo
    // XPath: fn:trace((for $var in (fn:upper-case(\"a\"),fn:lower-case(\"
    // Expected: a A b B
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}

#[test]
fn k_tracefunc_4() {
    //  A test whose essence is: `trace('a string', 'trace message')`.
    // XPath: trace(\"a string\", \"trace message\")
    // Expected: a string
    // Calls stub function - will fail until fn:trace is implemented
    let _ = stub_fntrace();
}
