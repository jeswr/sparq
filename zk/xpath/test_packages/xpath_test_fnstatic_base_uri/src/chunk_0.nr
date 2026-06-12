//! Test chunk 0 for fn:static-base-uri
//! Contains 14 tests

use dep::xpath::stub_fnstatic_base_uri;

#[test]
fn fn_static_base_2() {
    //  Evaluation of fn:static-base-uri function using 'ftp'. Uses fn:string .
    // XPath: declare base-uri \"ftp://ftp.is.co.za/rfc/somefile.txt\"; fn
    // Expected: ftp://ftp.is.co.za/rfc/somefile.txt
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_3() {
    //  Evaluation of fn:static-base-uri function using 'mailto'. Uses fn:string .
    // XPath: declare base-uri \"mailto:John.Doe@example.com\"; fn:string(
    // Expected: mailto:John.Doe@example.com
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_4() {
    //  Evaluation of fn:static-base-uri function using 'news'. Uses fn:string .
    // XPath: declare base-uri \"news:comp.infosystems.www.servers.unix\";
    // Expected: news:comp.infosystems.www.servers.unix
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_5() {
    //  Evaluation of fn:static-base-uri function using 'telnet'. Uses fn:string .
    // XPath: declare base-uri \"telnet://192.0.2.16:80/\"; fn:string(fn:s
    // Expected: telnet://192.0.2.16:80/
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_6() {
    //  Evaluation of fn:static-base-uri function using 'tel'. Uses fn:string .
    // XPath: declare base-uri \"tel:+1-816-555-1212\"; fn:string(fn:stati
    // Expected: tel:+1-816-555-1212
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_7() {
    //  Evaluation of fn:static-base-uri function using 'ldap' scheme. Uses fn:string .
    // XPath: declare base-uri \"urn:oasis:names:specification:docbook:dtd
    // Expected: urn:oasis:names:specification:docbook:dt
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_8() {
    //  Evaluation of fn:static-base-uri function using Used as argument to upper-case
    // XPath: declare base-uri \"http://www.example.com\"; fn:upper-case(f
    // Expected: HTTP://WWW.EXAMPLE.COM
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_9() {
    //  Evaluation of fn:static-base-uri function using Used as argument to lower-case
    // XPath: declare base-uri \"http://www.example.com\"; fn:lower-case(f
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_10() {
    //  Evaluation of fn:static-base-uri function using Used as argument to fn:concat f
    // XPath: declare base-uri \"http://www.example.com\"; fn:concat(fn:st
    // Expected: http://www.example.comanother string
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_11() {
    //  Evaluation of fn:static-base-uri function using Used as argument to fn:string-j
    // XPath: declare base-uri \"http://www.example.com\"; fn:string-join(
    // Expected: http://www.example.comanother string
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_12() {
    //  Evaluation of fn:static-base-uri function using Used as argument to fn:string-l
    // XPath: declare base-uri \"http://www.example.com\"; fn:string-lengt
    // Expected: 22
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_13() {
    //  Evaluation of fn:static-base-uri function using Used as argument to fn:substrin
    // XPath: declare base-uri \"http://www.example.com\"; fn:substring-be
    // Expected: http
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn fn_static_base_14() {
    //  Evaluation of fn:static-base-uri function using Used as argument to fn:substrin
    // XPath: declare base-uri \"http://www.example.com\"; fn:substring-af
    // Expected: //www.example.com
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}

#[test]
fn k_staticbaseurifunc_3() {
    //  A test whose essence is: `if(static-base-uri()) then true() else true()`.
    // XPath: if(static-base-uri()) then true() else true()
    // Expected: true
    // Calls stub function - will fail until fn:static-base-uri is implemented
    let _ = stub_fnstatic_base_uri();
}
