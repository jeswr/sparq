//! Test chunk 2 for fn:avg
//! Contains 50 tests

use dep::xpath::stub_fnavg;

#[test]
fn fn_avgsht2args_3() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:avg((xs:short(\"32767\"),xs:short(\"-32768\")))
    // Expected: -0.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht2args_4() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:avg((xs:short(\"-32768\"),xs:short(\"-5324\")))
    // Expected: -19046
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avgsht2args_5() {
    //  Evaluates The 'avg' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:avg((xs:short(\"-32768\"),xs:short(\"32767\")))
    // Expected: -0.5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_001() {
    //  arg: seq of integer
    // XPath: fn:avg( (3, 4, 5) )
    // Expected: 4
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_002() {
    //  arg: yearMonthDuration
    // XPath: fn:avg(( xs:yearMonthDuration(\"P20Y\") , xs:yearMonthDurati
    // Expected: P10Y5M
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_004() {
    //  arg: float
    // XPath: fn:avg(( xs:float('INF'), xs:float('-INF')))
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_005() {
    //  arg: seq of integer, float
    // XPath: fn:avg(( (3, 4, 5), xs:float('NaN') ))
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_006() {
    //  arg: seq of integer
    // XPath: fn:avg(( fn:string-length(\"Hello\") ))
    // Expected: 5
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_007() {
    //  arg: seq of integer
    // XPath: fn:avg(( fn:count(\"Hello\") ))
    // Expected: 1
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_008() {
    //  arg: seq of integer
    // XPath: fn:avg( ( ( xs:integer(\"100\"), xs:integer(\"-100\"))))
    // Expected: 0
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_010() {
    //  arg: seq of integer,decimal
    // XPath: fn:avg(( (xs:decimal(\"1.01\"), xs:integer(\"12\") )))
    // Expected: 6.505
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_011() {
    //  arg: seq of integer,float,empty seq
    // XPath: fn:avg(( (xs:float(\"NaN\"), 100, (), 2)))
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_012() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"-3.4028235E38\"), xs:decimal(\"-9999999
    // Expected: xs:float("-1.7014117E38")
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_013() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"0\"), xs:decimal(\"-999999999999999999\
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_014() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"3.4028235E38\"), xs:decimal(\"-99999999
    // Expected: xs:float("1.7014117E38")
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_015() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"-0\"), xs:decimal(\"-999999999999999999
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_016() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"NaN\"), xs:decimal(\"-99999999999999999
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_017() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"INF\"), xs:decimal(\"-99999999999999999
    // Expected: INF
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_019() {
    //  arg: seq of float,decimal
    // XPath: fn:avg(( (xs:float(\"-INF\"), xs:decimal(\"2.34\"))))
    // Expected: -INF
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_020() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"-1.7976931348623157E308\"), xs:integer
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_021() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"0\"), xs:integer(\"-999999999999999999
    // Expected: -5.0E17
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_022() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"1.7976931348623157E308\"), xs:integer(
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_023() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"-0\"), xs:integer(\"-99999999999999999
    // Expected: -5.0E17
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_024() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"-INF\"), xs:integer(\"-999999999999999
    // Expected: -INF
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_025() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"NaN\"), xs:integer(\"-9999999999999999
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_026() {
    //  arg: seq of double,float
    // XPath: fn:avg(( (xs:double(\"1.34\"), xs:float(\"INF\"))))
    // Expected: INF
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_mix_args_027() {
    //  arg: seq of double,integer
    // XPath: fn:avg(( (xs:double(\"INF\"), 2, 3)))
    // Expected: INF
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_3() {
    //  A test whose essence is: `empty(avg(()))`.
    // XPath: empty(avg(()))
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_4() {
    //  A test whose essence is: `avg((3, 3, 3)) eq 3`.
    // XPath: avg((3, 3, 3)) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_5() {
    //  A test whose essence is: `avg((-3, -3, -3)) eq -3`.
    // XPath: avg((-3, -3, -3)) eq -3
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_6() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(3), xs:float(3))) instan
    // XPath: avg((xs:float(1), xs:integer(3), xs:float(3))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_7() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(3), xs:decimal(3))) inst
    // XPath: avg((xs:float(1), xs:integer(3), xs:decimal(3))) instance of
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_8() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(3), xs:double(3))) insta
    // XPath: avg((xs:float(1), xs:integer(3), xs:double(3))) instance of
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_9() {
    //  A test whose essence is: `avg((xs:integer(1), xs:integer(3), xs:decimal(3))) in
    // XPath: avg((xs:integer(1), xs:integer(3), xs:decimal(3))) instance
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_10() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(0), xs:float(5))) eq 2.0
    // XPath: avg((xs:float(1), xs:integer(0), xs:float(5))) eq 2.0
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_11() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(0), xs:untypedAtomic(-4)
    // XPath: avg((xs:float(1), xs:integer(0), xs:untypedAtomic(-4))) eq -
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_12() {
    //  A test whose essence is: `avg((xs:float(1), xs:integer(0), xs:untypedAtomic(3))
    // XPath: avg((xs:float(1), xs:integer(0), xs:untypedAtomic(3))) insta
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_13() {
    //  A test whose essence is: `avg((xs:untypedAtomic(3), xs:integer(0), xs:decimal(1
    // XPath: avg((xs:untypedAtomic(3), xs:integer(0), xs:decimal(1))) ins
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_14() {
    //  A test whose essence is: `string(avg((3, 3, xs:double('NaN')))) eq 'NaN'`.
    // XPath: string(avg((3, 3, xs:double(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_15() {
    //  A test whose essence is: `string(avg((3, xs:double('NaN'), 3))) eq 'NaN'`.
    // XPath: string(avg((3, xs:double(\"NaN\"), 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_16() {
    //  A test whose essence is: `string(avg((xs:double('NaN'), 3, 3))) eq 'NaN'`.
    // XPath: string(avg((xs:double(\"NaN\"), 3, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_17() {
    //  A test whose essence is: `empty(avg(()))`.
    // XPath: empty(avg(()))
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_18() {
    //  A test whose essence is: `empty(avg(((), ())))`.
    // XPath: empty(avg(((), ())))
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_19() {
    //  A test whose essence is: `avg((-5, -0, -3, -6)) eq -3.5`.
    // XPath: avg((-5, -0, -3, -6)) eq -3.5
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_20() {
    //  A test whose essence is: `string(avg((1, 2, 3, xs:float('NaN')))) eq 'NaN'`.
    // XPath: string(avg((1, 2, 3, xs:float(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_21() {
    //  A test whose essence is: `string(avg((1, 2, 3, xs:double('NaN')))) eq 'NaN'`.
    // XPath: string(avg((1, 2, 3, xs:double(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_22() {
    //  A test whose essence is: `string(avg((xs:double('NaN'), 1, 2, 3))) eq 'NaN'`.
    // XPath: string(avg((xs:double(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_23() {
    //  A test whose essence is: `string(avg((xs:float('NaN'), 1, 2, 3))) eq 'NaN'`.
    // XPath: string(avg((xs:float(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_24() {
    //  A test whose essence is: `string(avg((1, 2, xs:double('NaN'), 1, 2, 3))) eq 'Na
    // XPath: string(avg((1, 2, xs:double(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_25() {
    //  A test whose essence is: `string(avg((1, 2, xs:float('NaN'), 1, 2, 3))) eq 'NaN
    // XPath: string(avg((1, 2, xs:float(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}
