//! Test chunk 0 for prod:GeneralComp.lt
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_lt;

#[test]
fn generalexpression199() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression200() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression201() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression202() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression203() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression204() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression205() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression206() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression207() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < (/works/employee[1]/hours[1],/works/employee[6]/hours[1
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression208() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () < ($works/works/employee[1]/hours[1],$staff/staff/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression209() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression210() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression211() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression212() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression213() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression214() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression215() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression216() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < (/works/employee[1]/hours[1],/works/employee[6]/hour
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression217() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 < ($works/works/employee[1]/hours[1],$staff/staff/empl
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression218() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression219() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression220() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression221() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression222() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression223() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression224() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression225() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression226() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < (/works/employee[1]/hours[1],/works/employee[6]/ho
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression227() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) < ($works/works/employee[1]/hours[1],$staff/staff/em
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression228() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression229() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression230() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression231() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression232() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression233() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression234() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression235() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression236() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < (/works/employee[1]/hours[1],/works/employee
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression237() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) < ($works/works/employee[1]/hours[1],$staff/st
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression238() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression239() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression240() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression241() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression242() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression243() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression244() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression245() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression246() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < (/works/employee[1]/hours[1],/works/employee[
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression247() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> < ($works/works/employee[1]/hours[1],$staff/sta
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn generalexpression248() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) < ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}
