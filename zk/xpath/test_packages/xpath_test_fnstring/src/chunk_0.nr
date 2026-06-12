//! Test chunk 0 for fn:string
//! Contains 50 tests

use dep::xpath::stub_fnstring;

#[test]
fn fn_stringint1args_1() {
    // stringint1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:int(\"-2147483648\"))
    // Expected: -2147483648
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringint1args_2() {
    // stringint1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:int(\"-1873914410\"))
    // Expected: -1873914410
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringint1args_3() {
    // stringint1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:int(\"2147483647\"))
    // Expected: 2147483647
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringintg1args_1() {
    // stringintg1args-1 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:integer(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringintg1args_2() {
    // stringintg1args-2 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:integer(\"830993497117024304\"))
    // Expected: 830993497117024304
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringintg1args_3() {
    // stringintg1args-3 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:integer(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdec1args_1() {
    // stringdec1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:decimal(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdec1args_2() {
    // stringdec1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:decimal(\"617375191608514839\"))
    // Expected: 617375191608514839
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdec1args_3() {
    // stringdec1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:decimal(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdbl1args_1() {
    // stringdbl1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:double(\"-1.7976931348623157E308\"))
    // Expected: -1.7976931348623157E308
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdbl1args_2() {
    // stringdbl1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:double(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringdbl1args_3() {
    // stringdbl1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:double(\"1.7976931348623157E308\"))
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringflt1args_1() {
    // stringflt1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:float(\"-3.4028235E38\"))
    // Expected: -3.4028235E38
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringflt1args_2() {
    // stringflt1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:float(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringflt1args_3() {
    // stringflt1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:float(\"3.4028235E38\"))
    // Expected: 3.4028235E38
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringlng1args_1() {
    // stringlng1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:long(\"-92233720368547758\"))
    // Expected: -92233720368547758
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringlng1args_2() {
    // stringlng1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:long(\"-47175562203048468\"))
    // Expected: -47175562203048468
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringlng1args_3() {
    // stringlng1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:long(\"92233720368547758\"))
    // Expected: 92233720368547758
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringusht1args_1() {
    // stringusht1args-1 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedShort(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringusht1args_2() {
    // stringusht1args-2 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedShort(\"44633\"))
    // Expected: 44633
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringusht1args_3() {
    // stringusht1args-3 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedShort(\"65535\"))
    // Expected: 65535
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnint1args_1() {
    // stringnint1args-1 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:negativeInteger(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnint1args_2() {
    // stringnint1args-2 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:negativeInteger(\"-297014075999096793\"))
    // Expected: -297014075999096793
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnint1args_3() {
    // stringnint1args-3 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:negativeInteger(\"-1\"))
    // Expected: -1
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringpint1args_1() {
    // stringpint1args-1 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:positiveInteger(\"1\"))
    // Expected: 1
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringpint1args_2() {
    // stringpint1args-2 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:positiveInteger(\"52704602390610033\"))
    // Expected: 52704602390610033
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringpint1args_3() {
    // stringpint1args-3 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:positiveInteger(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringulng1args_1() {
    // stringulng1args-1 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedLong(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringulng1args_2() {
    // stringulng1args-2 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedLong(\"130747108607674654\"))
    // Expected: 130747108607674654
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringulng1args_3() {
    // stringulng1args-3 The 'string' function with the arguments set as follows: $arg
    // XPath: fn:string(xs:unsignedLong(\"184467440737095516\"))
    // Expected: 184467440737095516
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnpi1args_1() {
    // stringnpi1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonPositiveInteger(\"-999999999999999999\"))
    // Expected: -999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnpi1args_2() {
    // stringnpi1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonPositiveInteger(\"-475688437271870490\"))
    // Expected: -475688437271870490
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnpi1args_3() {
    // stringnpi1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonPositiveInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnni1args_1() {
    // stringnni1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonNegativeInteger(\"0\"))
    // Expected: 0
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnni1args_2() {
    // stringnni1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonNegativeInteger(\"303884545991464527\"))
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringnni1args_3() {
    // stringnni1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:nonNegativeInteger(\"999999999999999999\"))
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringsht1args_1() {
    // stringsht1args-1 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:short(\"-32768\"))
    // Expected: -32768
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringsht1args_2() {
    // stringsht1args-2 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_stringsht1args_3() {
    // stringsht1args-3 The 'string' function with the arguments set as follows: $arg =
    // XPath: fn:string(xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_5() {
    // string() applied to a time
    // XPath: string(xs:time(\"24:00:00\"))
    // Expected: "00:00:00"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_6() {
    // string() applied to a dateTime
    // XPath: string(dateTime(xs:date(\"2011-06-29\"), xs:time(\"24:00:00\
    // Expected: "2011-06-29T00:00:00"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_7() {
    // string() applied to a duration
    // XPath: string(xs:duration(\"P24M\"))
    // Expected: "P2Y"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_8() {
    // string() applied to a QName
    // XPath: string(QName(\"http://www.w3.org/\", \"test:example\"))
    // Expected: "test:example"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_24() {
    // Get the string-value of an element in a namespace.
    // XPath: string((//*:Open)[1])
    // Expected: 2000-03-21:07:41:34-05:00
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_25() {
    // Get the string-value of an attribute in a namespace.
    // XPath: string((//*:Start)[1]/@*)
    // Expected: USD
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_26() {
    // Get the string-value of the xml:lang attribute.
    // XPath: string((//@xml:*)[1])
    // Expected: en
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_27() {
    // Get the string value of a namespace node.
    // XPath: string((//namespace::*[.='http://www.w3.org/1999/xlink'])[1]
    // Expected: http://www.w3.org/1999/xlink
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_28() {
    // Get the string-value of a processing-instruction node.
    // XPath: string((//processing-instruction())[1])
    // Expected: href="none"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_29() {
    // Get the string value of an element with children.
    // XPath: normalize-space(string((//*:Price)[1]))
    // Expected: "3.00 10.00 5"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn fn_string_31() {
    // Get the string value of a text node
    // XPath: string((//text())[normalize-space(.)][1])
    // Expected: "2000-03-21:07:41:34-05:00"
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}
