//! Test chunk 0 for prod:AxisStep.abbr
//! Contains 22 tests

use dep::xpath::stub_prodaxisstep_abbr;

#[test]
fn abbreviatedsyntax_1() {
    //  Evaluates 'hours'. Selects the 'hours' element children of the context node.
    // XPath: for $h in (/works/employee[4]) return $h/hours/string()
    // Expected: 20 40
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_2() {
    //  Evaluates 'text()'. Selects all text node children of the context node.
    // XPath: for $h in (/works/employee[2]) return $h/text()
    // Expected: Text data from Employee[2]
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_3() {
    //  Evaluates '@name'. Selects the name attribute of the context node.
    // XPath: for $h in (/works/employee[10]) return $h/@name
    // Expected: John Doe 10
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_5() {
    //  Evaluates 'employee[1]'. Selects the first employee child of the context node.
    // XPath: for $h in (/works) return $h/employee[1]/@name
    // Expected: Jane Doe 1
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_6() {
    //  Evaluates 'para[fn:last()]'. Selects the last employee child of the context nod
    // XPath: for $h in (/works) return $h/employee[fn:last()]/@name
    // Expected: Jane Doe 13
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_7() {
    //  Evaluates '*/hours'. Selects all hours grandchildren of the context node.
    // XPath: for $h in (/works) return $h/*/hours/string()
    // Expected: 40 70 20 80 20 40 20 30 12 40 80 20 20 2
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_8() {
    //  Evaluates '/works/employee[5]/hours[2]' selects the second hours of the fifth e
    // XPath: /works/employee[5]/hours[2]
    // Expected: 30
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_9() {
    //  Evaluates 'employee//hours'. Selects the hours element descendants of the emplo
    // XPath: for $h in (/works) return $h/employee//hours/string()
    // Expected: 40 70 20 80 20 40 20 30 12 40 80 20 20 2
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_10() {
    //  Evaluates '//hours'. Selects all the hours descendants of the root document nod
    // XPath: for $h in (/works) return $h//hours/string()
    // Expected: 40 70 20 80 20 40 20 30 12 40 80 20 20 2
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_12() {
    //  Evaluates '//overtime/day'. Selects all the day elements in the same document a
    // XPath: for $h in (/works) return $h//overtime/day/string()
    // Expected: Monday Tuesday
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_13() {
    //  Evaluates './/day'. Selects the day element descendants of the context node.
    // XPath: for $h in (/works) return $h/.//day/string()
    // Expected: Monday Tuesday
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_14() {
    //  Evaluates '..'. Selects the parent of the context node.
    // XPath: for $h in (/works/employee[12]/overtime) return $h/../@name
    // Expected: John Doe 12
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_16() {
    //  Evaluates 'employee[@name='Jane Doe 11']'. Selects all employee children of the
    // XPath: for $h in (/works) return $h/employee[@name=\"Jane Doe 11\"]
    // Expected: Jane Doe 11
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_17() {
    //  Evaluates 'employee[@gender='female'][5]'. Selects the fifth element child of t
    // XPath: for $h in (/works) return $h/employee[@gender=\"female\"][5]
    // Expected: Jane Doe 9
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_18() {
    //  Evaluates 'employee[5][@gender='female']'. Selects the fifth employee child of
    // XPath: for $h in (/works) return $h/employee[5][@gender=\"female\"]
    // Expected: Jane Doe 5
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_19() {
    //  Evaluates 'employee[status='active']'. Selects the employee children of the con
    // XPath: for $h in (/works) return $h/employee[status=\"active\"]/@na
    // Expected: Jane Doe 13
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_20() {
    //  Evaluates 'employee[overtime]'. Selects the employee children of the context no
    // XPath: for $h in (/works) return $h/employee[overtime]/@name
    // Expected: John Doe 12
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_21() {
    //  Evaluates 'employee[@name and @type]'. Selects all the employee children of the
    // XPath: for $h in (/works) return $h/employee[@name and @type]/@name
    // Expected: Jane Doe 13
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_22() {
    //  Evaluates 'employee/(status|overtime)/day'. Selects every day element that has
    // XPath: for $h in (/works) return $h/employee/(status|overtime)/day/
    // Expected: Monday Tuesday
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_24() {
    //  Evaluates 'employee/(status|overtime)/day'.        Selects every day element th
    // XPath: for $h in (/works) return $h/employee/(status union overtime
    // Expected: Monday Tuesday
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_25() {
    //  Evaluates 'employee[@name = condition @type=condition]'.        Selects all the
    // XPath: for $h in (/works) return $h/employee[@name = \"Jane Doe 13\
    // Expected: Jane Doe 13
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}

#[test]
fn abbreviatedsyntax_26() {
    //  Use an expression that returns atomic values after '//'
    // XPath: let $in := <a><b>ABC</b><b>XYZ</b></a> return $in//string-to
    // Expected: 65 66 67 88 89 90 65 66 67 65 66 67 88 8
    // Calls stub function - will fail until prod:AxisStep.abbr is implemented
    let _ = stub_prodaxisstep_abbr();
}
