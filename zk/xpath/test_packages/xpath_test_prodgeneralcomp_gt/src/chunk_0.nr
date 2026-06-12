//! Test chunk 0 for prod:GeneralComp.gt
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_gt;

#[test]
fn generalexpression397() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression398() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression399() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression400() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression401() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression402() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression403() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression404() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression405() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > (/works/employee[1]/hours[1],/works/employee[6]/hours[1
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression406() {
    //  Test of a General Expression with the operands set as follows operand1 = Empty
    // XPath: () > ($works/works/employee[1]/hours[1],$staff/staff/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression407() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression408() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression409() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression410() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression411() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression412() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression413() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression414() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > (/works/employee[1]/hours[1],/works/employee[6]/hour
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression415() {
    //  Test of a General Expression with the operands set as follows operand1 = Atomic
    // XPath: 10000 > ($works/works/employee[1]/hours[1],$staff/staff/empl
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression416() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression417() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression418() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression419() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression420() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression421() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression422() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression423() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression424() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > (/works/employee[1]/hours[1],/works/employee[6]/ho
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression425() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (50000) > ($works/works/employee[1]/hours[1],$staff/staff/em
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression426() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression427() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression428() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression429() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression430() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression431() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression432() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression433() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (/works/employee[1]/hours[1])
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression434() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > (/works/employee[1]/hours[1],/works/employee
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression435() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (10000,50000) > ($works/works/employee[1]/hours[1],$staff/st
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression436() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression437() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression438() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression439() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression440() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression441() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression442() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression443() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression444() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > (/works/employee[1]/hours[1],/works/employee[
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression445() {
    //  Test of a General Expression with the operands set as follows operand1 = Elemen
    // XPath: <a>10000</a> > ($works/works/employee[1]/hours[1],$staff/sta
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn generalexpression446() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) > ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}
