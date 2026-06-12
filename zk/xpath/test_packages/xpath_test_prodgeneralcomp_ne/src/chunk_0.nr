//! Test chunk 0 for prod:GeneralComp.ne
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_ne;

#[test]
fn generalexpression100() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression101() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression102() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression103() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression104() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression105() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression106() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression107() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression108() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != (/works/employee[1]/hours[1],/works/employee[6]/hours[
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression109() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () != ($works/works/employee[1]/hours[1],$staff/staff/employ
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression110() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression111() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression112() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression113() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression114() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression115() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression116() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression117() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != (/works/employee[1]/hours[1],/works/employee[6]/hou
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression118() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 != ($works/works/employee[1]/hours[1],$staff/staff/emp
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression119() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression120() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression121() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression122() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression123() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression124() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression125() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression126() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression127() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != (/works/employee[1]/hours[1],/works/employee[6]/h
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression128() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) != ($works/works/employee[1]/hours[1],$staff/staff/e
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression129() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression130() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression131() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression132() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression133() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression134() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression135() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression136() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression137() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != (/works/employee[1]/hours[1],/works/employe
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression138() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) != ($works/works/employee[1]/hours[1],$staff/s
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression139() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression140() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression141() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression142() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression143() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression144() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression145() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression146() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression147() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != (/works/employee[1]/hours[1],/works/employee
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression148() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> != ($works/works/employee[1]/hours[1],$staff/st
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn generalexpression149() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) != ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}
