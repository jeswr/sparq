//! Test chunk 1 for prod:GeneralComp.gt
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_gt;

#[test]
fn generalexpression447() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression448() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression449() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression450() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression451() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression452() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression453() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression454() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > (/works/employee[1]/hours[1],/works/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression455() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > ($works/works/employee[1]/hours[1],$staff/s
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression456() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression457() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression458() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression459() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression460() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression461() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression462() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression463() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression464() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > (/works/employee[1]/hours[1],/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression465() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) > ($works/works/employee[1]/hour
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression466() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression467() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression468() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression469() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression470() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression471() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression472() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression473() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (/works/employee[1]/hours[1]
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression474() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) > (/works/employee[1]/hours[1]
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression475() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1]) > ($works/works/employee
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression476() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression477() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression478() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression479() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression480() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression481() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression482() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression483() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression484() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) >
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression485() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$works/works/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression486() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression487() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression488() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression489() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression490() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression491() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression492() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression493() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression494() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn greaterthanonanyuri_1() {
    // Test - greaterthanonanyuri-1  Evaluation of 'gt' operator on xs:anyURI datatype.
    // XPath: (xs:anyURI(\"http://www.example/com\")) > (xs:anyURI(\"http:
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_1() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() > () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}
