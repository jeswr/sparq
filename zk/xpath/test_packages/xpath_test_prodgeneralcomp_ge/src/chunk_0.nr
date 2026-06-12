//! Test chunk 0 for prod:GeneralComp.ge
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_ge;

#[test]
fn generalexpression496() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression497() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression498() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression499() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression500() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression501() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression502() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression503() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression504() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= (/works/employee[1]/hours[1],/works/employee[6]/hours[
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression505() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () >= ($works/works/employee[1]/hours[1],$staff/staff/employ
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression506() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression507() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression508() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression509() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression510() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression511() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression512() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression513() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= (/works/employee[1]/hours[1],/works/employee[6]/hou
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression514() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 >= ($works/works/employee[1]/hours[1],$staff/staff/emp
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression515() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression516() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression517() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression518() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression519() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression520() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression521() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression522() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression523() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= (/works/employee[1]/hours[1],/works/employee[6]/h
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression524() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) >= ($works/works/employee[1]/hours[1],$staff/staff/e
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression525() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression526() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression527() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression528() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression529() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression530() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression531() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression532() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression533() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= (/works/employee[1]/hours[1],/works/employe
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression534() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) >= ($works/works/employee[1]/hours[1],$staff/s
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression535() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression536() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression537() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression538() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression539() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression540() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression541() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression542() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression543() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= (/works/employee[1]/hours[1],/works/employee
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression544() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> >= ($works/works/employee[1]/hours[1],$staff/st
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn generalexpression545() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) >= ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}
