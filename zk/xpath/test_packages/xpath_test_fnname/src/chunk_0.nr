//! Test chunk 0 for fn:name
//! Contains 29 tests

use dep::xpath::stub_fnname;

#[test]
fn fn_name_1() {
    // argument set to the empty sequence. Uses the fn:string-length function to avoid
    // XPath: fn:string-length(fn:name(()))
    // Expected: 0
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_2() {
    // argument set to an element node.
    // XPath: (fn:name(./works[1]/employee[1]))
    // Expected: employee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_3() {
    // argument set to an attribute node. insert-start insert-end
    // XPath: (fn:name(./works[1]/employee[1]/@name))
    // Expected: name
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_4() {
    // argument set to a document node. Use of 'fn:string-length' to avoid empty file.
    // XPath: fn:string-length(fn:name(.))
    // Expected: 0
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_5() {
    // argument set to a non existing element. Use of 'fn:string-length' to avoid empty
    // XPath: fn:string-length(fn:name(./works[1]/nonexistent[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_6() {
    // argument set to a non existing element. Use of 'fn:string-length' to avoid empty
    // XPath: for $h in ./works[1]/employee[2] return          fn:string-l
    // Expected: 0
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_8() {
    // argument to the fn:upper-case function
    // XPath: for $h in (./works/employee[2]) return fn:upper-case(fn:name
    // Expected: EMPLOYEE
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_9() {
    // argument to the fn:lower-case function
    // XPath: for $h in (./works/employee[2]) return fn:lower-case(fn:name
    // Expected: employee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_10() {
    // that uses the 'parent' axes
    // XPath: for $h in (./works/employee[2]) return fn:name($h/parent::no
    // Expected: works
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_11() {
    // that uses the 'descendant' axes
    // XPath: for $h in (./works/employee[2]) return fn:name($h/descendant
    // Expected: empnum
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_12() {
    // that uses the 'descendant-or-self' axes
    // XPath: for $h in (./works/employee[2]) return fn:name($h/descendant
    // Expected: empnum
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_13() {
    // argument to the fn-subtstring function.
    // XPath: for $h in (./works/employee[2]) return fn:substring(fn:name(
    // Expected: mployee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_14() {
    // argument to the fn:concat function.
    // XPath: for $h in (/works/employee[2]) return fn:concat(fn:name($h),
    // Expected: employeeA String
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_15() {
    // that uses the 'self' axes. Returns a string
    // XPath: for $h in (./works/employee[2]) return fn:name($h/self::empl
    // Expected: employee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_17() {
    // that uses the 'parent::node()'. The context node is an attribute node.
    // XPath: for $h in (/works/employee[2]/@name) return fn:name($h/paren
    // Expected: employee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_18() {
    // Evaluation of the fn:name function as an argument to the string-length function.
    // XPath: fn:string-length(fn:name(./works[1]/employee[2]/@name))
    // Expected: 4
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_19() {
    // Evaluation of the fn:name function, for which the argument is a direct element c
    // XPath: fn:name(<anElement>Content</anElement>)
    // Expected: anElement
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_20() {
    // Evaluation of the fn:name function, for which the argument is a direct element c
    // XPath: fn:name(<anElement name=\"attribute1\">Content</anElement>)
    // Expected: anElement
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_21() {
    // Evaluation of the fn:name function used as part of a sequence.
    // XPath: (fn:name(./works[1]/employee[1]),fn:name(./works[1]/employee
    // Expected: employee employee
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_22() {
    // fn:count.
    // XPath: fn:count(((fn:name(/works[1]/employee[1]),fn:name(/works[1]/
    // Expected: 2
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_24() {
    // XPath: name(/*)
    // Expected: ma:AuctionWatchList
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_25() {
    // XPath: name((//*:Start)[1]/@*)
    // Expected: ma:currency
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_26() {
    // XPath: name((//@xml:*)[1])
    // Expected: xml:lang
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_27() {
    //  Get the name of a namespace node.
    // XPath: name((//namespace::*[.='http://www.w3.org/1999/xlink'])[1])
    // Expected: xlink
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_28() {
    // Get the name of a processing-instruction node.
    // XPath: name((//processing-instruction())[1])
    // Expected: xml-stylesheet
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_29() {
    // Get the name of an element in a default but non-null namespace.
    // XPath: name((//*[.='1983'])[1])
    // Expected: recorded
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_30() {
    // Get the name of a comment node
    // XPath: name((//comment())[1]) = ''
    // Expected: true
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn fn_name_31() {
    // Get the name of a text node
    // XPath: name((//text())[1]) = ''
    // Expected: true
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}

#[test]
fn k_namefunc_3() {
    // Test: K-NameFunc-3 `name(()) eq ''`.
    // XPath: name(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:name is implemented
    let _ = stub_fnname();
}
