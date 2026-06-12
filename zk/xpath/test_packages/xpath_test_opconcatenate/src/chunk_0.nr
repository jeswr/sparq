//! Test chunk 0 for op:concatenate
//! Contains 14 tests

use dep::xpath::stub_opconcatenate;

#[test]
fn sequenceexpressionhc6() {
    //  Constructing Sequences. Simple sequence expression resulting in an empty sequen
    // XPath: count((15 to 10))
    // Expected: 0
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn constseq_20() {
    //  Constructing sequences. Constructing a sequence, where both of its members cont
    // XPath: (xs:date(\"2004-12-25Z\"),xs:date(\"2004-12-26Z\"))
    // Expected: 2004-12-25Z 2004-12-26Z
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn constseq_21() {
    //  Constructing sequences. Constructing a sequence, where both of its members cont
    // XPath: (xs:dateTime(\"1999-11-28T09:00:00Z\"),xs:dateTime(\"1998-11
    // Expected: 1999-11-28T09:00:00Z 1998-11-28T09:00:00
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn constseq_22() {
    //  Constructing sequences. Constructing a sequence, where both of its members cont
    // XPath: (xs:time(\"08:00:00+09:00\"),xs:time(\"08:00:00+10:00\"))
    // Expected: 08:00:00+09:00 08:00:00+10:00
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_008() {
    //  arg1 & arg2 : float
    // XPath: xs:float(\"1.01\"), xs:float(\"NaN\")
    // Expected: 1.01 NaN
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_009() {
    //  arg1:float, arg2: double
    // XPath: xs:float(\"INF\") , xs:double(\"NaN\")
    // Expected: INF NaN
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_010() {
    //  arg1: double, arg2: double, arg3:float
    // XPath: xs:double(\"INF\"), xs:double(\"-INF\"), xs:float(\"-INF\")
    // Expected: INF -INF -INF
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_013() {
    //  arg1:date, arg2: boolean, arg3: string
    // XPath: xs:date(\"1993-03-31\") , xs:boolean(\"true\"), xs:string(\"
    // Expected: 1993-03-31 true abc
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_014() {
    //  arg1:dateTime, arg2: empty sequence
    // XPath: xs:dateTime(\"1972-12-31T00:00:00Z\") , (())
    // Expected: 1972-12-31T00:00:00Z
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_015() {
    //  arg1:time, arg2: string , arg3: decimal
    // XPath: xs:time(\"12:30:00Z\") , xs:string(\" \") , xs:decimal(\"2.0
    // Expected: 12:30:00Z   2.000000000000002
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn op_concatenate_mix_args_016() {
    //  arg1:empty seq, arg2: string , arg3: decimal
    // XPath: () , xs:string(\" \") , xs:decimal(\"2.000000000000002\")
    // Expected: 2.000000000000002
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn k_commaop_1() {
    //  A heavily nested sequence of expressions with the comma operator. On some imple
    // XPath: deep-equal(((1, (2, (3, 4, (5, 6)), 7), 8, (9, 10), 11)), (1
    // Expected: true
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn k_commaop_2() {
    //  An expression sequence containing only empty sequences. On some implementations
    // XPath: empty(((), (), ((), (), ((), (), (())), ()), (), (())))
    // Expected: true
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}

#[test]
fn k_commaop_3() {
    //  An expression sequence containing many empty sequences and one xs:string. On so
    // XPath: ((), (), ((), (), ((), (), (\"str\")), ()), (), (())) eq \"s
    // Expected: true
    // Calls stub function - will fail until op:concatenate is implemented
    let _ = stub_opconcatenate();
}
