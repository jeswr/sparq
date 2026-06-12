//! Test chunk 1 for fn:deep-equal
//! Contains 50 tests

use dep::xpath::stub_fndeep_equal;

#[test]
fn fn_deep_equalnpi2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnpi2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonPositiveInteger(\"-475688437271870490\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnpi2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonPositiveInteger(\"0\")),(xs:nonPositive
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnpi2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnpi2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonPositiveInteger(\"-999999999999999999\"
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnni2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonNegativeInteger(\"0\")),(xs:nonNegative
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnni2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonNegativeInteger(\"303884545991464527\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnni2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonNegativeInteger(\"999999999999999999\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnni2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonNegativeInteger(\"0\")),(xs:nonNegative
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalnni2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:nonNegativeInteger(\"0\")),(xs:nonNegative
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalsht2args_1() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:short(\"-32768\")),(xs:short(\"-32768\")))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalsht2args_2() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:short(\"-5324\")),(xs:short(\"-32768\")))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalsht2args_3() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:short(\"32767\")),(xs:short(\"-32768\")))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalsht2args_4() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:short(\"-32768\")),(xs:short(\"-5324\")))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equalsht2args_5() {
    //  Evaluates The 'deep-equal' function with the arguments set as follows: $paramet
    // XPath: fn:deep-equal((xs:short(\"-32768\")),(xs:short(\"32767\")))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_001() {
    //  The order of elements in sequence is important
    // XPath: fn:deep-equal( (1,2) , (2,1))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_002() {
    //  Two sequences are fn:deep-equal if items have same value and same order
    // XPath: fn:deep-equal( (1,2) , (1,2))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_003() {
    //  comparing arguments of type string
    // XPath: fn:deep-equal( xs:string(\"A\") , \"A\")
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_004() {
    //  arg of type string, but case is different
    // XPath: fn:deep-equal( xs:string(\"A\") , \"a\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_005() {
    //  arg1: string, arg2: sequence with 1 element of type string
    // XPath: fn:deep-equal( xs:string(\"A\") , (\"A\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_006() {
    //  Comparing null strings
    // XPath: fn:deep-equal( xs:string(\"\") , (\"\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_007() {
    //  Comparing null sequences
    // XPath: fn:deep-equal( () , ())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_008() {
    //  Comparing nested null & a null sequence
    // XPath: fn:deep-equal( (()) , ())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_009() {
    //  Comparing arg1: sequence with a space with agr2: empty sequence
    // XPath: fn:deep-equal( ( ) , ())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_010() {
    //  Comparing strings with different value
    // XPath: fn:deep-equal(xs:string(\"abc\"), xs:string(\"cba\"))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_011() {
    //  Comparing arg1: anyURI, arg2: string
    // XPath: fn:deep-equal( xs:anyURI(\"www.example.com\") , \"www.exampl
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_012() {
    //  Comparing arg1:anyURI and arg2:string
    // XPath: fn:deep-equal( xs:anyURI(\"www.example.com\") , xs:string(\"
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_013() {
    //  Comparing arg1:integer and arg2:decimal
    // XPath: fn:deep-equal( xs:integer(1) , xs:decimal(1.0))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_014() {
    //  Comapring integer args with different values
    // XPath: fn:deep-equal( xs:integer(1) , xs:integer(-1))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_015() {
    //  Comparing arg1:integer and arg2:float
    // XPath: fn:deep-equal( xs:integer(1) , xs:float(1.0))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_016() {
    //  Comparing arg1:integer, arg2:double
    // XPath: fn:deep-equal( xs:integer(1) , xs:double(1.0))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_017() {
    //  Comparing arg1:decimal , arg2:float
    // XPath: fn:deep-equal( xs:decimal(1.01) , xs:float(1.01))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_018() {
    //  Comparing arg1:decimal, arg2:double
    // XPath: fn:deep-equal( xs:decimal(1.01) , xs:double(1.01))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_019() {
    //  Comparing arg1:float , arg2:double
    // XPath: fn:deep-equal( xs:float(1.01) , xs:double(1.01))
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_020() {
    //  Comparing arg1:float, arg2:double values INF
    // XPath: fn:deep-equal( xs:float(\"INF\") , xs:double(\"INF\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_021() {
    //  Comparing arg1:float, arg2:double values -INF
    // XPath: fn:deep-equal( xs:float(\"-INF\") , xs:double(\"-INF\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_022() {
    //  Comparing arg1:float, arg2:double values NaN
    // XPath: fn:deep-equal( xs:float(\"NaN\") , xs:double(\"NaN\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_023() {
    //  Comparing two boolean args
    // XPath: fn:deep-equal( xs:boolean(\"1\") , xs:boolean(\"true\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_024() {
    //  Comparing two boolean args
    // XPath: fn:deep-equal( xs:boolean(\"true\") , xs:boolean(\"true\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_025() {
    //  Comparing arg1:boolean arg2:value returned by true fn
    // XPath: fn:deep-equal( xs:boolean(\"true\") , true())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_026() {
    //  Comparing two boolean args
    // XPath: fn:deep-equal( xs:boolean(\"0\") , xs:boolean(\"false\"))
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_027() {
    //  Comparing arg1:boolean arg2:value of false fn
    // XPath: fn:deep-equal( xs:boolean(\"false\") , false())
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_028() {
    //  Comparing arg1:date, arg2: string
    // XPath: fn:deep-equal( xs:date(\"1993-03-31\") , \"1993-03-31\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_029() {
    //  Comparing arg1:date, arg2: string
    // XPath: fn:deep-equal( xs:date(\"1993-03-31\") , xs:string(\"1993-03
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_030() {
    //  Comparing arg1:dateTime, arg2: string
    // XPath: fn:deep-equal(xs:dateTime(\"1972-12-31T00:00:00\"), \"1972-1
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_mix_args_031() {
    //  Comparing arg1:time, arg2: string
    // XPath: fn:deep-equal(xs:time(\"12:30:00\"), \"12:30:00\")
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_node_args_1() {
    //  Apply deep-equal to nodes
    // XPath: fn:deep-equal(<a> {/bib/node()} </a>/node(), <b> {/bib/node(
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_node_args_2() {
    //  Apply deep-equal to nodes
    // XPath: fn:deep-equal(<a> {/bib/node(), /bib/node()} </a>/node(), <b
    // Expected: false
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_1() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([], [])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}

#[test]
fn fn_deep_equal_arrays_2() {
    //  Apply deep-equal to arrays
    // XPath: fn:deep-equal([1,2], [1,2])
    // Expected: true
    // Calls stub function - will fail until fn:deep-equal is implemented
    let _ = stub_fndeep_equal();
}
