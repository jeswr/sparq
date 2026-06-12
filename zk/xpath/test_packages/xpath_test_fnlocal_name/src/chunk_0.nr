//! Test chunk 0 for fn:local-name
//! Contains 50 tests

use dep::xpath::stub_fnlocal_name;

#[test]
fn fn_local_name_4() {
    //  Name: fn-local-name-4 Description: Evaluation of the fn:local-name function
    // XPath: fn:local-name(())
    // Expected: ""
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_5() {
    //  Name: fn-local-name-5 Description: Evaluation of the fn:local-name function
    // XPath: fn:local-name((//text())[1])
    // Expected: ""
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_6() {
    //  Evaluation of the fn:local-name function argument set computed comment node. Us
    // XPath: fn:count(fn:local-name((comment {\"A Comment Node\"})))
    // Expected: 1
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_7() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:count(fn:local-name(<!-- A Comment Node -->))
    // Expected: 1
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_8() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:count(fn:local-name(document {<aDocument>some content</aD
    // Expected: 1
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_8a() {
    //  Name: fn-local-name-8 Description: Evaluation of the fn:local-name function
    // XPath: fn:count(fn:local-name(/))
    // Expected: 1
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_9() {
    //  Evaluation of the fn:local-name function argument set to a computed element nod
    // XPath: fn:string(fn:local-name(element anElement {\"Some content\"}
    // Expected: anElement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_10() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:string(fn:local-name(<anElement>Some content</anElement>)
    // Expected: anElement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_11() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:string(fn:local-name(<p1:anElement xmlns:p1=\"http://exam
    // Expected: anElement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_11a() {
    //  Name: fn-local-name-11 Description: Evaluation of the fn:local-name function
    // XPath: fn:local-name(/*)
    // Expected: root
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_12() {
    //  Evaluation of the fn:local-name function argument set to a computed attribute n
    // XPath: fn:string(fn:local-name(attribute anAttribute {\"Attribute V
    // Expected: anAttribute
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_13() {
    //  Evaluation of the fn:local-name function argument set to a computed attribute n
    // XPath: declare namespace p1 = \"http://example.org\";          fn:s
    // Expected: anAttribute
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_13a() {
    //  Name: fn-local-name-13 Description: Evaluation of the fn:local-name function
    // XPath: fn:local-name((//@*:attr)[1])
    // Expected: attr
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_14() {
    //  Evaluation of the fn:local-name function argument set to a computed PI node wit
    // XPath: fn:string(fn:local-name(processing-instruction PITarget {\"P
    // Expected: PITarget
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_15() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:string(fn:local-name(<?format role=\"output\" ?>))
    // Expected: format
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_16() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: declare namespace p1 = \"http://example.org\"; fn:string(fn:
    // Expected: anElement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_17() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:string-length(fn:string(fn:local-name(<anElement>Some con
    // Expected: 9
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_18() {
    //  Evaluation of the fn:local-name function argument set to a computed element nod
    // XPath: declare namespace p1 = \"http://example.org\";          fn:s
    // Expected: anElement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_19() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:upper-case(fn:string(fn:local-name(<anElement>Some conten
    // Expected: ANELEMENT
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_20() {
    //  Evaluation of the fn:local-name function argument set to a directly constructed
    // XPath: fn:lower-case(fn:string(fn:local-name(<anElement>Some conten
    // Expected: anelement
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_21() {
    //  Evaluation of the fn:local-name function argument set to a computed attribite n
    // XPath: fn:upper-case(fn:string(fn:local-name(attribute anAttribute
    // Expected: ANATTRIBUTE
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_22() {
    //  Evaluation of the fn:local-name function argument set to a computed attribite n
    // XPath: fn:lower-case(fn:string(fn:local-name(attribute anAttribute
    // Expected: anattribute
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_51() {
    //  Name: fn-local-name-51 Description: Evaluation of the fn:local-name function
    // XPath: fn:string-length(fn:local-name(()))
    // Expected: 0
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_52() {
    //  Name: fn-local-name-52 Description: Evaluation of the fn:local-name function
    // XPath: (fn:local-name(./works[1]/employee[1]))
    // Expected: employee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_53() {
    //  Name: fn-local-name-53 Description: Evaluation of the fn:local-name function
    // XPath: (fn:local-name(./works[1]/employee[1]/@name))
    // Expected: name
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_54() {
    //  Name: fn-local-name-54 Description: Evaluation of the fn:local-name function
    // XPath: fn:string-length(fn:local-name(.))
    // Expected: 0
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_55() {
    //  Name: fn-local-name-55 Description: Evaluation of the fn:local-name function
    // XPath: fn:string-length(fn:local-name(./works[1]/nonexistent[1]))
    // Expected: 0
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_56() {
    //  Name: fn-local-name-56 Description: Evaluation of the fn:local-name function
    // XPath: for $h in ./works[1]/employee[2] return          fn:string-l
    // Expected: 0
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_58() {
    //  Name: fn-local-name-58 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:upper-case(fn:loca
    // Expected: EMPLOYEE
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_59() {
    //  Name: fn-local-name-59 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:lower-case(fn:loca
    // Expected: employee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_60() {
    //  Name: fn-local-name-60 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:local-name($h/pare
    // Expected: works
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_61() {
    //  Name: fn-local-name-61 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:local-name($h/desc
    // Expected: empnum
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_62() {
    //  Name: fn-local-name-62 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return          fn:local-nam
    // Expected: empnum
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_63() {
    //  Name: fn-local-name-63 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:substring(fn:local
    // Expected: mployee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_64() {
    //  Name: fn-local-name-64 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (/works/employee[2]) return fn:concat(fn:local-nam
    // Expected: employeeA String
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_65() {
    //  Name: fn-local-name-65 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (./works/employee[2]) return fn:local-name($h/self
    // Expected: employee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_67() {
    //  Name: fn-local-name-67 Description: Evaluation of the fn:local-name function
    // XPath: for $h in (/works/employee[2]/@name) return fn:local-name($h
    // Expected: employee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_68() {
    //  Name: fn-local-name-68 Description: Evaluation of the fn:local-name function as
    // XPath: fn:string-length(fn:local-name(./works[1]/employee[2]/@name)
    // Expected: 4
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_71() {
    //  Name: fn-local-name-71 Description: Evaluation of the fn:local-name function
    // XPath: string-join((fn:local-name(./works[1]/employee[1]),fn:local-
    // Expected: employee employee
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_72() {
    //  Name: fn-local-name-72 Description: Evaluation of the fn:local-name function as
    // XPath: fn:count(((fn:local-name(/works[1]/employee[1]),fn:local-nam
    // Expected: 2
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_74() {
    //  Name: fn-local-name-74 Description: Get the name of an element in a namespace.
    // XPath: name(/*)
    // Expected: ma:AuctionWatchList
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_75() {
    //  Name: fn-local-name-75 Description: Get the name of an attribute in a namespace
    // XPath: name((//*:Start)[1]/@*)
    // Expected: ma:currency
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_76() {
    //  Name: fn-local-name-76 Description: Get the name of the xml:lang attribute.
    // XPath: name((//@xml:*)[1])
    // Expected: xml:lang
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_77() {
    //  Description: Get the name of a namespace node.
    // XPath: name((//namespace::*[.='http://www.w3.org/1999/xlink'])[1])
    // Expected: xlink
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_78() {
    //  Get the name of a processing-instruction node.
    // XPath: name((//processing-instruction())[1])
    // Expected: xml-stylesheet
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_79() {
    //  Get the name of an element in a default but non-null namespace.
    // XPath: name((//*[.='1983'])[1])
    // Expected: recorded
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_80() {
    //  Get the name of a comment node
    // XPath: name((//comment())[1]) = ''
    // Expected: true
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn fn_local_name_81() {
    //  Get the name of a text node
    // XPath: name((//text())[1]) = ''
    // Expected: true
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn k_nodelocalnamefunc_3() {
    // A test whose essence is: `local-name(()) eq ''`.
    // XPath: local-name(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}

#[test]
fn k2_nodelocalnamefunc_1() {
    //  Call fn:local-name() on an attribute node.
    // XPath: local-name(/works/employee[1]/@name)
    // Expected: name
    // Calls stub function - will fail until fn:local-name is implemented
    let _ = stub_fnlocal_name();
}
