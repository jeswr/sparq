//! Test chunk 1 for prod:GeneralComp.eq
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_eq;

#[test]
fn generalexpression51() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression52() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression53() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression54() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression55() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression56() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression57() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression58() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = (/works/employee[1]/hours[1],/works/employe
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression59() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>) = ($works/works/employee[1]/hours[1],$staff/s
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression60() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression61() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = 10000
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression62() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression63() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (10000,50000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression64() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = <a>10000</a>
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression65() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (<a>10000</a>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression66() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (<a>10000</a>,<b>50000</b>)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression67() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (/works/employee[1]/hours[1])
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression68() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = (/works/employee[1]/hours[1],/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression69() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (<a>10000</a>,<b>50000</b>) = ($works/works/employee[1]/hour
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression70() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = ()
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression71() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = 10000
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression72() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression73() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (10000,50000)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression74() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = <a>10000</a>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression75() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (<a>10000</a>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression76() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (<a>10000</a>,<b>50000</b>)
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression77() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (/works/employee[1]/hours[1]
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression78() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1]) = (/works/employee[1]/hours[1]
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression79() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1]) = ($works/works/employee
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression80() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression81() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression82() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression83() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression84() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression85() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression86() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression87() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression88() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: (/works/employee[1]/hours[1],/works/employee[6]/hours[1]) =
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression89() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$works/works/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression90() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression91() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression92() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression93() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression94() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression95() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression96() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression97() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression98() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn generalexpression99() {
    //  Test of a General Expression with the operands set as follows operand1 = Sequen
    // XPath: ($works/works/employee[1]/hours[1],$staff/staff/employee[6]/
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_1() {
    //  General comparisons involving the empty sequence.
    // XPath: not(() = ())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}
