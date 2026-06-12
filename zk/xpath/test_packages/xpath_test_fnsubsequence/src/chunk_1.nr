//! Test chunk 1 for fn:subsequence
//! Contains 18 tests

use dep::xpath::stub_fnsubsequence;

#[test]
fn cbcl_subsequence_004() {
    //  Test unbounded fn:subsequence with starting position -INF
    // XPath: count(subsequence(1 to 10, xs:double(\"-INF\")))
    // Expected: 10
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_005() {
    //  Test unbounded fn:subsequence with starting position NaN
    // XPath: count(subsequence(1 to 10, xs:double(\"NaN\")))
    // Expected: 0
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_006() {
    //  Test subsequence optimizations.
    // XPath: subsequence((1, 2, for $x in 1 to 10 return 2*$x), 2, year-f
    // Expected: 2 2 4 6 8 10 12 14 16 18 20
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_007() {
    //  Test subsequence optimizations.
    // XPath: subsequence((1, 2, for $x in 1 to 10 return 2*$x), -1, sum((
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_009() {
    //  Test subsequence on starting value out of int range.
    // XPath: count(subsequence(1 to 100, -2147483648, 20))
    // Expected: 0
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_015() {
    //  Test subsequence on map expression.
    // XPath: subsequence(for $x in (1 to 100) return -$x, 3, 5)
    // Expected: -3 -4 -5 -6 -7
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_016() {
    //  Test subsequence on something that looks a bit like last but isn't
    // XPath: subsequence( for $x in 1 to 10 return 1 to $x, count(for $x
    // Expected: 10
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_017() {
    //  Test subsequence with infinite length starting at first item
    // XPath: subsequence(for $x in 1 to 10 return 1 to $x, 1.2, xs:double
    // Expected: 1 1 2 1 2 3 1 2 3 4 1 2 3 4 5 1 2 3 4 5
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_018() {
    //  Test subsequence with infinite length starting after first item
    // XPath: subsequence(for $x in 1 to 10 return 1 to $x, 4.2, xs:double
    // Expected: 1 2 3 1 2 3 4 1 2 3 4 5 1 2 3 4 5 6 1 2
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_019() {
    //  Test subsequence that gets the last item
    // XPath: let $y := for $x in 1 to 10 return $x * $x return subsequenc
    // Expected: 100
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_020() {
    //  Tests subsequence with static start and length
    // XPath: subsequence(for $x in 1 to 10 return $x[. mod 2 = 0],2,4)
    // Expected: 4 6 8 10
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_021() {
    //  Tests an unbounded subsequence with a static start
    // XPath: subsequence(for $x in 1 to 10 return $x[. mod 2 = 0],2)
    // Expected: 4 6 8 10
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_022() {
    //  Test skip on initial subsequence enumerator
    // XPath: subsequence((1 to 20)[. mod 2 = 0][position() < 5],2)
    // Expected: 4 6 8
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_023() {
    //  Test subsequence(for .....)
    // XPath: subsequence(for $x in 1 to 10 return $x * $x,2)
    // Expected: 4 9 16 25 36 49 64 81 100
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_024() {
    //  Test subsequence(...,1,INF)
    // XPath: subsequence((1 to 100)[. mod 2 = 0],1,xs:double(string-join(
    // Expected: 2 4 6 8 10 12 14 16 18 20 22 24 26 28 30
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_025() {
    //  Tries to call GetExpressionProperties on static-subsequence. In fact shows an e
    // XPath: count(<a><b/></a>/*/subsequence(.,1,1)/..)
    // Expected: 1
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_026() {
    //  Tests for subsequence on divided sequence
    // XPath: let $x := (1 to 10)[. mod 2 = 0] return subsequence((0,$x),3
    // Expected: 4 6 8
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}

#[test]
fn cbcl_subsequence_027() {
    //  Tests for subsequence on a map expression
    // XPath: let $z := (1 to 10)[. mod 2 = 0] return subsequence(for $x i
    // Expected: 4 6 8 10
    // Calls stub function - will fail until fn:subsequence is implemented
    let _ = stub_fnsubsequence();
}
