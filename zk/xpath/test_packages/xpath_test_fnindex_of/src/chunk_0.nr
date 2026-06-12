//! Test chunk 0 for fn:index-of
//! Contains 46 tests

use dep::xpath::stub_fnindex_of;

#[test]
fn fn_indexof_mix_args_001() {
    //  arg1: Sequence of integers , arg2: integer
    // XPath: fn:index-of ((10, 20, 30, 30, 20, 10), 20)
    // Expected: 2 5
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_002() {
    //  arg1: Sequence of integers , arg2: integer
    // XPath: fn:index-of ((10, 20, 30, 40), 35)
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_003() {
    //  arg1: Sequence of string , arg2: string
    // XPath: fn:index-of ((\"a\", \"sport\", \"and\", \"a\", \"pastime\")
    // Expected: 1 4
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_004() {
    //  arg1: Sequence of string , arg2: string
    // XPath: fn:index-of((\"sport\", \"\", \"and\", \"\", \"\", \"pastime
    // Expected: 2 4 5
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_005() {
    //  arg1: empty Sequence , arg2: string
    // XPath: fn:index-of((),\"\")
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_006() {
    //  arg1: Sequence of string , arg2: string
    // XPath: fn:index-of((\"sport\"), \"\")
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_007() {
    //  arg1: empty Sequence , arg2: untypedAtomic
    // XPath: fn:index-of((),fn:exactly-one(xs:untypedAtomic('')))
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_008() {
    //  arg1: Sequence of float , arg2: float
    // XPath: fn:index-of((xs:float('NaN')), fn:exactly-one(xs:float('NaN'
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_009() {
    //  arg1: Sequence of double , arg2: double
    // XPath: fn:index-of((xs:double('NaN')), fn:exactly-one(xs:double('Na
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_010() {
    //  arg1: Sequence of double, arg2: double
    // XPath: fn:index-of((xs:double('INF')), fn:exactly-one(xs:double('IN
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_011() {
    //  arg1: Sequence of double, arg2: double
    // XPath: fn:index-of((xs:double('-INF')), fn:exactly-one(xs:double('-
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_012() {
    //  arg1: Sequence of decimal, arg2: decimal
    // XPath: fn:index-of((xs:decimal('9.99999999999999999999999999')), fn
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_013() {
    //  arg1: Sequence of decimal, arg2: decimal
    // XPath: fn:index-of((xs:decimal('9.99999999999999999999999999')), fn
    // Expected:
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_014() {
    //  arg1: Sequence of positiveInteger, arg2: positiveInteger
    // XPath: fn:index-of((xs:positiveInteger('1'),xs:positiveInteger('2')
    // Expected: 2
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_015() {
    //  arg1: Sequence of negativeInteger, arg2: negativeInteger
    // XPath: fn:index-of((xs:negativeInteger('-2'), xs:negativeInteger('-
    // Expected: 2
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_016() {
    //  Use a nested sequence in the sequence to search
    // XPath: fn:index-of((1, (1,2,3)),1)
    // Expected: 1 2
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_017() {
    //  Use multiple nested sequence in the sequence to search
    // XPath: fn:index-of(((1),(1), (2,1), (0,1)),1)
    // Expected: 1 2 4 6
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_018() {
    //  Use external variable for the sequence parameter
    // XPath: fn:index-of((/bib/book/publisher), \"Addison-Wesley\")
    // Expected: 1 2
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_019() {
    //  Use external variable both in sequence and search parameter
    // XPath: fn:index-of((/bib/book/publisher), /bib/book[1]/publisher[1]
    // Expected: 1 2
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_020() {
    //  Use empty string with other strings in sequence
    // XPath: fn:index-of((\"sport\", \"\", \"and\", \"\", \"\", \"pastime
    // Expected: 2 4 5
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn fn_indexof_mix_args_021() {
    //  Use empty string with integers in the sequence parameter
    // XPath: fn:index-of( (\"\", 1, \"\"), \"\")
    // Expected: 1 3
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_6() {
    //  A test whose essence is: `index-of('a string', 'a string', 'http://www.w3.org/2
    // XPath: index-of(\"a string\", \"a string\", \"http://www.w3.org/200
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_7() {
    //  A test whose essence is: `empty(index-of(xs:double('NaN'), xs:float('NaN')))`.
    // XPath: empty(index-of(xs:double(\"NaN\"), xs:float(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_8() {
    //  A test whose essence is: `empty(index-of(xs:float('NaN'), xs:double('NaN')))`.
    // XPath: empty(index-of(xs:float(\"NaN\"), xs:double(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_9() {
    //  A test whose essence is: `empty(index-of(xs:float('NaN'), xs:float('NaN')))`.
    // XPath: empty(index-of(xs:float(\"NaN\"), xs:float(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_10() {
    //  A test whose essence is: `empty(index-of(xs:double('NaN'), xs:double('NaN')))`.
    // XPath: empty(index-of(xs:double(\"NaN\"), xs:double(\"NaN\")))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_11() {
    //  A test whose essence is: `empty(index-of(4, '4'))`.
    // XPath: empty(index-of(4, \"4\"))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_12() {
    //  A test whose essence is: `index-of(4, 4)`.
    // XPath: index-of(4, 4)
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_13() {
    //  A test whose essence is: `empty(index-of((), 4))`.
    // XPath: empty(index-of((), 4))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_14() {
    //  A test whose essence is: `index-of(4, 4)`.
    // XPath: index-of(4, 4)
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_15() {
    //  A test whose essence is: `deep-equal((1, 2, 3, 4, 5, 6), index-of((4, 4, 4, 4,
    // XPath: deep-equal((1, 2, 3, 4, 5, 6), index-of((4, 4, 4, 4, 4, 4),
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_16() {
    //  A test whose essence is: `empty(index-of(xs:anyURI('example.com/'), xs:hexBinar
    // XPath: empty(index-of(xs:anyURI(\"example.com/\"), xs:hexBinary(\"F
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_17() {
    //  A test whose essence is: `index-of(xs:untypedAtomic('example.com/'), xs:anyURI(
    // XPath: index-of(xs:untypedAtomic(\"example.com/\"), xs:anyURI(\"exa
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_18() {
    //  A test whose essence is: `deep-equal(index-of((1, 2, 'three', 5, 5, 6), 5), (4,
    // XPath: deep-equal(index-of((1, 2, \"three\", 5, 5, 6), 5), (4, 5))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_19() {
    //  A test whose essence is: `empty(index-of((10, 20, 30, 40), 35))`.
    // XPath: empty(index-of((10, 20, 30, 40), 35))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_20() {
    //  A test whose essence is: `deep-equal(index-of((10, 20, 30, 30, 20, 10), 20), (2
    // XPath: deep-equal(index-of((10, 20, 30, 30, 20, 10), 20), (2, 5))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_21() {
    //  A test whose essence is: `deep-equal(index-of(('a', 'sport', 'and', 'a', 'pasti
    // XPath: deep-equal(index-of((\"a\", \"sport\", \"and\", \"a\", \"pas
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_22() {
    //  A test whose essence is: `count(index-of((1, 2, 3, 2, 1), 2)) eq 2`.
    // XPath: count(index-of((1, 2, 3, 2, 1), 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_23() {
    //  A test whose essence is: `count(index-of((1, 2, 3, 2, 1), 1)) eq 2`.
    // XPath: count(index-of((1, 2, 3, 2, 1), 1)) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_24() {
    //  A test whose essence is: `count(index-of((1, 2, 3, 2, 1), 3)) eq 1`.
    // XPath: count(index-of((1, 2, 3, 2, 1), 3)) eq 1
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn k_seqindexoffunc_25() {
    //  A test whose essence is: `count(index-of((1, 2, 3, 2, 1), 4)) eq 0`.
    // XPath: count(index-of((1, 2, 3, 2, 1), 4)) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn cbcl_fn_indexof_1() {
    //  Test an index-of on a range expression
    // XPath: index-of(1 to 5,4)
    // Expected: 4
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn cbcl_fn_indexof_2() {
    //  Test optimization of index-of to equal
    // XPath: index-of(exactly-one((1 to 10)[. div 5 = 1]),5)
    // Expected: 1
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn cbcl_fn_indexof_3() {
    //  Test boolean of indexof
    // XPath: boolean(index-of((1 to 10)[. mod 2 = 0],4))
    // Expected: true
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn cbcl_fn_indexof_4() {
    //  Tests false boolean of indexof
    // XPath: boolean(index-of((1 to 10)[. mod 2 = 0],5))
    // Expected: false
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}

#[test]
fn cbcl_fn_indexof_5() {
    //  Tests fn:index-of of a range value followed by an sequence
    // XPath: index-of((1 to 10,(1 to 10)[. mod 2 = 0]),4)
    // Expected: 4 12
    // Calls stub function - will fail until fn:index-of is implemented
    let _ = stub_fnindex_of();
}
