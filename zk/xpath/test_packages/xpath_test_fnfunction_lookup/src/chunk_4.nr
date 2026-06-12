//! Test chunk 4 for fn:function-lookup
//! Contains 50 tests

use dep::xpath::stub_fnfunction_lookup;

#[test]
fn fn_function_lookup_212() {
    // Attempts to invoke function fn:month-from-date.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_213() {
    // Attempts to look up function fn:day-from-date.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_214() {
    // Attempts to invoke function fn:day-from-date.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_215() {
    // Attempts to look up function fn:timezone-from-date.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_216() {
    // Attempts to invoke function fn:timezone-from-date.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: PT0S
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_217() {
    // Attempts to look up function fn:hours-from-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_218() {
    // Attempts to invoke function fn:hours-from-time.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_219() {
    // Attempts to look up function fn:minutes-from-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_220() {
    // Attempts to invoke function fn:minutes-from-time.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_221() {
    // Attempts to look up function fn:seconds-from-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_222() {
    // Attempts to invoke function fn:seconds-from-time.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 0
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_223() {
    // Attempts to look up function fn:timezone-from-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_224() {
    // Attempts to invoke function fn:timezone-from-time.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: PT0S
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_225() {
    // Attempts to look up function fn:adjust-dateTime-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_227() {
    // Attempts to look up function fn:adjust-dateTime-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_228() {
    // Attempts to invoke function fn:adjust-dateTime-to-timezone.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1969-12-31T14:00:00-10:00
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_229() {
    // Attempts to look up function fn:adjust-date-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_231() {
    // Attempts to look up function fn:adjust-date-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_232() {
    // Attempts to invoke function fn:adjust-date-to-timezone.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 1969-12-31-10:00
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_233() {
    // Attempts to look up function fn:adjust-time-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_235() {
    // Attempts to look up function fn:adjust-time-to-timezone.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_236() {
    // Attempts to invoke function fn:adjust-time-to-timezone.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 14:00:00-10:00
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_237() {
    // Attempts to look up function fn:format-dateTime.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_239() {
    // Attempts to look up function fn:format-dateTime.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_240() {
    // Attempts to invoke function fn:format-dateTime.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2012
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_241() {
    // Attempts to look up function fn:format-date.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_243() {
    // Attempts to look up function fn:format-date.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_244() {
    // Attempts to invoke function fn:format-date.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 2012
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_245() {
    // Attempts to look up function fn:format-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_247() {
    // Attempts to look up function fn:format-time.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_248() {
    // Attempts to invoke function fn:format-time.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: 00
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_249() {
    // Attempts to look up function fn:resolve-QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_250() {
    // Attempts to invoke function fn:resolve-QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: ns:local
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_251() {
    // Attempts to look up function fn:QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_252() {
    // Attempts to invoke function fn:QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: ns:local
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_253() {
    // Attempts to look up function fn:prefix-from-QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_254() {
    // Attempts to invoke function fn:prefix-from-QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: foo
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_255() {
    // Attempts to look up function fn:local-name-from-QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_256() {
    // Attempts to invoke function fn:local-name-from-QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: bar
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_257() {
    // Attempts to look up function fn:namespace-uri-from-QName.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_258() {
    // Attempts to invoke function fn:namespace-uri-from-QName.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: http://www.example.org
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_259() {
    // Attempts to look up function fn:namespace-uri-for-prefix.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_260() {
    // Attempts to invoke function fn:namespace-uri-for-prefix.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: http://www.example.org/
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_261() {
    // Attempts to look up function fn:in-scope-prefixes.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_262() {
    // Attempts to invoke function fn:in-scope-prefixes.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: xml
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_263() {
    // Attempts to look up function fn:name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_264() {
    // Attempts to invoke function fn:name.
    // XPath: /root/function-lookup(fn:QName('http://www.w3.org/2005/xpath
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_265() {
    // Attempts to look up function fn:name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_266() {
    // Attempts to invoke function fn:name.
    // XPath: function-lookup(fn:QName('http://www.w3.org/2005/xpath-funct
    // Expected: root
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}

#[test]
fn fn_function_lookup_267() {
    // Attempts to look up function fn:local-name.
    // XPath: exists(function-lookup(fn:QName('http://www.w3.org/2005/xpat
    // Expected: true
    // Calls stub function - will fail until fn:function-lookup is implemented
    let _ = stub_fnfunction_lookup();
}
