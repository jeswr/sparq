//! Test chunk 1 for prod:GeneralComp.lt
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_lt;

#[test]
fn generalexpression249() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression250() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression251() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression252() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression253() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression254() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression255() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression256() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < (/works/employee[1]/hours[1],/works/employe
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression257() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < ($works/works/employee[1]/hours[1],$staff/s
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression258() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression259() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression260() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression261() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression262() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression263() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression264() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression265() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression266() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) < (/works/employee[1]/hours[1],/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression267() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (          <a>10000</a>,<b>50000</b>) < ($works/works/employ
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression268() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression269() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression270() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression271() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression272() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression273() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression274() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression275() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (/works/employee[1]/hours[1]
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression276() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) < (/works/employee[1]/hours[1]
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression277() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1]) < ($works/works/employee
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression278() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression279() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression280() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression281() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression282() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression283() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression284() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression285() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression286() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) <
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression287() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$works/works/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression288() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression289() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression290() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression291() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression292() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression293() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression294() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression295() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression296() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression297() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn lessthanonanyuri_1() {
    // Test - lessthanonanyuri-1  Evaluation of 'lt' operator on xs:anyURI datatype.
    // XPath: (xs:anyURI(\"http://www.example/com\")) < (xs:anyURI(\"http:
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}
