//! Test chunk 0 for prod:AxisStep
//! Contains 50 tests

use dep::xpath::stub_prodaxisstep;

#[test]
fn axes001_1() {
    //  Path 'child::*' from an element.
    // XPath: fn:count(//center/child::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes001_2() {
    //  Path 'child::*' from an element.
    // XPath: fn:count(//center/child::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes001_3() {
    //  Path 'child::*' from an element.
    // XPath: fn:count(//center/child::*)
    // Expected: 6
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes002_1() {
    //  Path 'child::' with element name from an element.
    // XPath: fn:count(//center/child::south-east)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes002_2() {
    //  Path 'child::' with element name from an element.
    // XPath: fn:count(//center/child::south-east)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes002_3() {
    //  Path 'child::' with element name from an element.
    // XPath: fn:count(//center/child::south-east)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes002_4() {
    //  Path 'child::' with element name from an element.
    // XPath: fn:count(//center/child::south-east)
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes003_1() {
    //  Path 'child::node()' from an element.
    // XPath: fn:count(//center/child::node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes003_2() {
    //  Path 'child::node()' from an element.
    // XPath: fn:count(//center/child::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes003_3() {
    //  Path 'child::node()' from an element.
    // XPath: fn:count(//center/child::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes003_4() {
    //  Path 'child::node()' from an element.
    // XPath: fn:count(//center/child::node())
    // Expected: 19
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes004_1() {
    //  Test implied child axis in '*' from an element.
    // XPath: fn:count(//center/*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes004_2() {
    //  Test implied child axis in '*' from an element.
    // XPath: fn:count(//center/*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes004_3() {
    //  Test implied child axis in '*' from an element.
    // XPath: fn:count(//center/*)
    // Expected: 6
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes005_1() {
    //  Test implied child axis with element name, from an element.
    // XPath: fn:count(//center/south-east)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes005_2() {
    //  Test implied child axis with element name, from an element.
    // XPath: fn:count(//center/south-east)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes005_3() {
    //  Test implied child axis with element name, from an element.
    // XPath: fn:count(//center/south-east)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes005_4() {
    //  Test implied child axis with element name, from an element.
    // XPath: fn:count(//center/south-east)
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes006_1() {
    //  Test implied child axis in 'node()' from an element.
    // XPath: fn:count(//center/node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes006_2() {
    //  Test implied child axis in 'node()' from an element.
    // XPath: fn:count(//center/node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes006_3() {
    //  Test implied child axis in 'node()' from an element.
    // XPath: fn:count(//center/node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes006_4() {
    //  Test implied child axis in 'node()' from an element.
    // XPath: fn:count(//center/node())
    // Expected: 19
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes007_1() {
    //  Path 'attribute::*' from an element.
    // XPath: fn:count(//west/attribute::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes007_2() {
    //  Path 'attribute::*' from an element.
    // XPath: fn:count(//west/attribute::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes007_3() {
    //  Path 'attribute::*' from an element.
    // XPath: fn:count(//west/attribute::*)
    // Expected: 4
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes008_1() {
    //  Path 'attribute::' with name of attribute, from an element.
    // XPath: fn:count(//west/attribute::west-attr-2)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes008_2() {
    //  Path 'attribute::' with name of attribute, from an element.
    // XPath: fn:count(//west/attribute::west-attr-2)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes008_3() {
    //  Path 'attribute::' with name of attribute, from an element.
    // XPath: fn:count(//west/attribute::west-attr-2)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes009_1() {
    //  Path 'attribute::node()' from an element.
    // XPath: fn:count(//west/attribute::node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes009_2() {
    //  Path 'attribute::node()' from an element.
    // XPath: fn:count(//west/attribute::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes009_3() {
    //  Path 'attribute::node()' from an element.
    // XPath: fn:count(//west/attribute::node())
    // Expected: 4
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes010_1() {
    //  Path '@*' (abbreviated syntax) from an element.
    // XPath: fn:count(//west/@*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes010_2() {
    //  Path '@*' (abbreviated syntax) from an element.
    // XPath: fn:count(//west/@*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes010_3() {
    //  Path '@*' (abbreviated syntax) from an element.
    // XPath: fn:count(//west/@*)
    // Expected: 4
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes011_1() {
    //  Path '@' with name of attribute, from an element.
    // XPath: fn:count(//west/@west-attr-2)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes011_2() {
    //  Path '@' with name of attribute, from an element.
    // XPath: fn:count(//west/@west-attr-2)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes011_3() {
    //  Path '@' with name of attribute, from an element.
    // XPath: fn:count(//west/@west-attr-2)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes012_1() {
    //  Test '/' alone.
    // XPath: fn:count( / )
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes013_1() {
    //  Path 'parent::*' from an element.
    // XPath: fn:count(//center/parent::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes014_1() {
    //  Path 'parent::*' from document element.
    // XPath: fn:count(/far-north/parent::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes015_1() {
    //  Path 'parent::' with specified element name, from an element.
    // XPath: fn:count(//center/parent::near-north)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes016_1() {
    //  Path 'parent::' with specified element name that is not found, from an element.
    // XPath: fn:count(//center/parent::nowhere)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes017_1() {
    //  Path 'parent::node()' from an element.
    // XPath: fn:count(//center/parent::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes018_1() {
    //  Path 'parent::node()' from document element.
    // XPath: fn:count(/far-north/parent::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes019_1() {
    //  Test abbreviated '..' syntax from an element.
    // XPath: fn:count(//center/..)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes020_1() {
    //  Path 'self::*' from an element.
    // XPath: fn:count(//center/self::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes021_1() {
    //  Path 'self::' with specified element name, from an element.
    // XPath: fn:count(//center/self::center)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes023_1() {
    //  Path 'self::node()' from an element.
    // XPath: fn:count(//center/self::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes027_1() {
    //  Path 'self::node()' axis from an attribute.
    // XPath: fn:count(//center/@center-attr-3/self::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes030_1() {
    //  Path 'self::node()' from a text node.
    // XPath: fn:count(//center/text()/self::node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}
