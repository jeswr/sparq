//! Test chunk 2 for prod:AxisStep
//! Contains 50 tests

use dep::xpath::stub_prodaxisstep;

#[test]
fn axes059_2() {
    //  Test '/descendant-or-self::*' absolute path.
    // XPath: fn:count(/descendant-or-self::*)
    // Expected: 15
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes060_1() {
    //  Absolute path '/descendant-or-self::' with specified element name.
    // XPath: fn:count(/descendant-or-self::south)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes060_2() {
    //  Absolute path '/descendant-or-self::' with specified element name.
    // XPath: fn:count(/descendant-or-self::south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes060_3() {
    //  Absolute path '/descendant-or-self::' with specified element name.
    // XPath: fn:count(/descendant-or-self::south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes060_4() {
    //  Absolute path '/descendant-or-self::' with specified element name.
    // XPath: fn:count(/descendant-or-self::south)
    // Expected: 8
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes061_1() {
    //  Test '/descendant-or-self::node()' absolute path.
    // XPath: fn:count(/descendant-or-self::node())
    // Expected: 57
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes061_2() {
    //  Test '/descendant-or-self::node()' absolute path.
    // XPath: fn:count(/descendant-or-self::node())
    // Expected: 59
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes062_1() {
    //  Test '//child::*' absolute path.
    // XPath: fn:count(//child::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes062_2() {
    //  Test '//child::*' absolute path.
    // XPath: fn:count(//child::*)
    // Expected: 15
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes063_1() {
    //  Absolute path '//child::' with specified element name.
    // XPath: fn:count(//child::south)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes063_2() {
    //  Absolute path '//child::' with specified element name.
    // XPath: fn:count(//child::south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes063_3() {
    //  Absolute path '//child::' with specified element name.
    // XPath: fn:count(//child::south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes063_4() {
    //  Absolute path '//child::' with specified element name.
    // XPath: fn:count(//child::south)
    // Expected: 8
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes064_1() {
    //  Test '//child::node()' absolute path.
    // XPath: fn:count(//child::node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes064_2() {
    //  Test '//child::node()' absolute path.
    // XPath: fn:count(//child::node())
    // Expected: 56
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes064_3() {
    //  Test '//child::node()' absolute path.
    // XPath: fn:count(//child::node())
    // Expected: 58
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes065_1() {
    //  Test implied child axis in '//*' absolute path.
    // XPath: fn:count(//*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes065_2() {
    //  Test implied child axis in '//*' absolute path.
    // XPath: fn:count(//*)
    // Expected: 15
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes066_1() {
    //  Absolute path with element name after // implies child axis.
    // XPath: fn:count(//south)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes066_2() {
    //  Absolute path with element name after // implies child axis.
    // XPath: fn:count(//south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes066_3() {
    //  Absolute path with element name after // implies child axis.
    // XPath: fn:count(//south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes066_4() {
    //  Absolute path with element name after // implies child axis.
    // XPath: fn:count(//south)
    // Expected: 8
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes067_1() {
    //  Test implied child axis in '//node()' absolute path.
    // XPath: fn:count(//node())
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes067_2() {
    //  Test implied child axis in '//node()' absolute path.
    // XPath: fn:count(//node())
    // Expected: 56
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes067_3() {
    //  Test implied child axis in '//node()' absolute path.
    // XPath: fn:count(//node())
    // Expected: 58
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes068_1() {
    //  Test '//attribute::*' absolute path.
    // XPath: fn:count(//attribute::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes068_2() {
    //  Test '//attribute::*' absolute path.
    // XPath: fn:count(//attribute::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes068_3() {
    //  Test '//attribute::*' absolute path.
    // XPath: fn:count(//attribute::*)
    // Expected: 14
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes069_1() {
    //  Absolute path '//attribute::' with specified name.
    // XPath: fn:count(//attribute::mark)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes069_2() {
    //  Absolute path '//attribute::' with specified name.
    // XPath: fn:count(//attribute::mark)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes069_3() {
    //  Absolute path '//attribute::' with specified name.
    // XPath: fn:count(//attribute::mark)
    // Expected: 6
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes070_1() {
    //  Absolute path '//@*' has abbreviated syntax.
    // XPath: fn:count(//@*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes070_2() {
    //  Absolute path '//@*' has abbreviated syntax.
    // XPath: fn:count(//@*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes070_3() {
    //  Absolute path '//@*' has abbreviated syntax.
    // XPath: fn:count(//@*)
    // Expected: 14
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes071_1() {
    //  Absolute path '//@' with specified name.
    // XPath: fn:count(//@mark)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes071_2() {
    //  Absolute path '//@' with specified name.
    // XPath: fn:count(//@mark)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes071_3() {
    //  Absolute path '//@' with specified name.
    // XPath: fn:count(//@mark)
    // Expected: 6
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes072_1() {
    //  Test '//self::*' absolute path.
    // XPath: fn:count(//self::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes072_2() {
    //  Test '//self::*' absolute path.
    // XPath: fn:count(//self::*)
    // Expected: 15
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes073_1() {
    //  Test '//self::node()' absolute path.
    // XPath: fn:count(//self::node())
    // Expected: 57
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes073_2() {
    //  Test '//self::node()' absolute path.
    // XPath: fn:count(//self::node())
    // Expected: 59
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes074_1() {
    //  Path '//child::*' from an element.
    // XPath: fn:count(//center//child::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes074_2() {
    //  Path '//child::*' from an element.
    // XPath: fn:count(//center//child::*)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes074_3() {
    //  Path '//child::*' from an element.
    // XPath: fn:count(//center//child::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes074_4() {
    //  Path '//child::*' from an element.
    // XPath: fn:count(//center//child::*)
    // Expected: 12
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes075_1() {
    //  Path '//child::' with element name, from an element.
    // XPath: fn:count(//center//child::south)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes075_2() {
    //  Path '//child::' with element name, from an element.
    // XPath: fn:count(//center//child::south)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes075_3() {
    //  Path '//child::' with element name, from an element.
    // XPath: fn:count(//center//child::south)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes075_4() {
    //  Path '//child::' with element name, from an element.
    // XPath: fn:count(//center//child::south)
    // Expected: 8
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn axes076_1() {
    //  Path '//child::node()' from an element.
    // XPath: fn:count(//center//child::node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}
