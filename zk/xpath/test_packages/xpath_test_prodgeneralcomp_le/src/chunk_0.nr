//! Test chunk 0 for prod:GeneralComp.le
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_le;

#[test]
fn generalexpression298() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression299() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression300() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression301() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression302() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression303() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression304() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression305() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression306() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= (/works/employee[1]/hours[1],/works/employee[6]/hours[
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression307() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () <= ($works/works/employee[1]/hours[1],$staff/staff/employ
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression308() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression309() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression310() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression311() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression312() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression313() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression314() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression315() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= (/works/employee[1]/hours[1],/works/employee[6]/hou
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression316() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 <= ($works/works/employee[1]/hours[1],$staff/staff/emp
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression317() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression318() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression319() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression320() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression321() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression322() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression323() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression324() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression325() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= (/works/employee[1]/hours[1],/works/employee[6]/h
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression326() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) <= ($works/works/employee[1]/hours[1],$staff/staff/e
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression327() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression328() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression329() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression330() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression331() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression332() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression333() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression334() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression335() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= (/works/employee[1]/hours[1],/works/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression336() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) <= ($works/works/employee[1]/hours[1],$staff/s
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression337() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression338() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression339() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression340() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression341() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression342() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression343() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression344() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression345() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= (/works/employee[1]/hours[1],/works/employee
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression346() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> <= ($works/works/employee[1]/hours[1],$staff/st
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn generalexpression347() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) <= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}
