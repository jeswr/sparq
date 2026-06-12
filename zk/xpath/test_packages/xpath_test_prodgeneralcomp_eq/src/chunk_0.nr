//! Test chunk 0 for prod:GeneralComp.eq
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_eq;

#[test]
fn generalexpression1() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression2() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression3() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression4() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression5() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression6() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression7() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression8() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression9() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = (/works/employee[1]/hours[1],/works/employee[6]/hours[1
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression10() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () = ($works/works/employee[1]/hours[1],$staff/staff/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression11() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression12() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression13() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression14() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression15() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression16() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression17() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression18() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = (/works/employee[1]/hours[1],/works/employee[6]/hour
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression19() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 = ($works/works/employee[1]/hours[1],$staff/staff/empl
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression20() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression21() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression22() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression23() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression24() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression25() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression26() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression27() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression28() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = (/works/employee[1]/hours[1],/works/employee[6]/ho
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression29() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) = ($works/works/employee[1]/hours[1],$staff/staff/em
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression30() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression31() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression32() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression33() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression34() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression35() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression36() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression37() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression38() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = (/works/employee[1]/hours[1],/works/employee
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression39() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) = ($works/works/employee[1]/hours[1],$staff/st
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression40() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression41() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression42() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression43() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression44() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression45() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression46() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression47() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression48() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = (/works/employee[1]/hours[1],/works/employee[
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression49() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> = ($works/works/employee[1]/hours[1],$staff/sta
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression50() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}
