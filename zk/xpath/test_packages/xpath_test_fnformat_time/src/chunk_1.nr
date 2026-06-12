//! Test chunk 1 for fn:format-time
//! Contains 15 tests

use dep::xpath::stub_fnformat_time;

#[test]
fn millisecs_011() {
    // Fractional seconds: grouping separators
    // XPath: format-time(xs:time('12:01:01.135'), \"[f0'0'0]\")
    // Expected: "1'3'5"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_012() {
    // Fractional seconds: irregular grouping separators
    // XPath: format-time(xs:time('12:01:01.135'), \"[f00'0]\")
    // Expected: "13'5"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_013() {
    // Fractional seconds: minimum width modifier
    // XPath: format-time(xs:time('12:01:01.135'), \"[f,6-*]\")
    // Expected: "135000"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_014() {
    // Fractional seconds: maximum width modifier
    // XPath: format-time(xs:time('12:01:01.133'), \"[f,*-2]\")
    // Expected: "13"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_015() {
    // Fractional seconds: min and maximum width modifier
    // XPath: format-time(xs:time('12:01:01.133'), \"[f,2-4]\")
    // Expected: "133"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_016() {
    // Fractional seconds: min and maximum width modifier
    // XPath: format-time(xs:time('12:01:01'), \"[f,2-4]\")
    // Expected: "00"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_017() {
    // Fractional seconds: default, when value is zero
    // XPath: format-time(xs:time('12:01:01'), \"[f]\")
    // Expected: "0"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_018() {
    // Fractional seconds: both width modifier and picture
    // XPath: format-time(xs:time('12:01:01.123'), \"[f111,2-2]\")
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_020() {
    // Fractional seconds: non-ascii digits
    // XPath: format-time(xs:time('12:01:01.123'), \"[f???]\")
    // Expected: "???"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_021() {
    // Fractional seconds: non-ascii digits with grouping separators
    // XPath: format-time(xs:time('12:01:01.123'), \"[f?~?~?]\")
    // Expected: "?~?~?"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_022() {
    // Fractional seconds: rounding edge case - bug 29749
    // XPath: format-time(xs:time('12:01:01.999'), \"[f99]\")
    // Expected: "99"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_023() {
    // Fractional seconds: whitespace within a variable marker is ignored
    // XPath: format-time(xs:time('12:01:01.123'), \"[ f 0 0 0 ]\")
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_024() {
    // Fractional seconds: both width modifier and picture
    // XPath: format-time(xs:time('12:01:01.189'), \"[f?#,3-3]\")
    // Expected: "???"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_025() {
    // Fractional seconds: both width modifier and picture
    // XPath: format-time(xs:time('12:01:01.1'), \"[f1###,3-3]\")
    // Expected: "100"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_026() {
    // Fractional seconds: both width modifier and picture
    // XPath: format-time(xs:time('12:01:01.5'), \"[f1###,2-3]\")
    // Expected: "50"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}
