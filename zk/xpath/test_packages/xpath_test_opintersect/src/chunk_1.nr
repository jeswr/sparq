//! Test chunk 1 for op:intersect
//! Contains 3 tests

use dep::xpath::stub_opintersect;

#[test]
fn k2_seqintersect_46() {
    // General intersection in a boolean context
    // XPath: if (subsequence(//*, 1, 10) intersect subsequence(//*, 7, 2)
    // Expected: "foo"
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_47() {
    // Singleton intersection in a boolean context
    // XPath: if (head(//*) intersect subsequence(//*, 7, 2)) then \"foo\"
    // Expected: "bar"
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn combiningnodeseqintersecthc1() {
    //  Simple combination of node sequences involving different node sequences that re
    // XPath: count((<a>0</a>,<a>1</a>) intersect (<a>3</a>,<a>4</a>))
    // Expected: 0
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}
