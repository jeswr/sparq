//! Test chunk 3 for prod:NamedFunctionRef
//! Contains 50 tests

use dep::xpath::stub_prodnamedfunctionref;

#[test]
fn function_literal_156() {
    // Attempts to invoke function fn:tokenize.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}tokenize#2('string'
    // Expected: str ng
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_156a() {
    // Attempts to invoke the one argument function fn:tokenize.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}tokenize#1('a b c '
    // Expected: a b c
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_157() {
    // Attempts to look up function fn:tokenize.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}tokenize#3)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_158() {
    // Attempts to invoke function fn:tokenize.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}tokenize#3('string'
    // Expected: str ng
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_159() {
    // Attempts to look up function fn:analyze-string.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}analyze-stri
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_161() {
    // Attempts to look up function fn:analyze-string.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}analyze-stri
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_163() {
    // Attempts to look up function fn:resolve-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}resolve-uri#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_165() {
    // Attempts to look up function fn:resolve-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}resolve-uri#
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_166() {
    // Attempts to invoke function fn:resolve-uri.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}resolve-uri#2('/200
    // Expected: http://www.w3.org/2005/xpath-functions
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_167() {
    // Attempts to look up function fn:encode-for-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}encode-for-u
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_168() {
    // Attempts to invoke function fn:encode-for-uri.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}encode-for-uri#1('
    // Expected: %20
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_169() {
    // Attempts to look up function fn:iri-to-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}iri-to-uri#1
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_170() {
    // Attempts to invoke function fn:iri-to-uri.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}iri-to-uri#1('http:
    // Expected: http://www.example.com/
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_171() {
    // Attempts to look up function fn:escape-html-uri.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}escape-html-
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_172() {
    // Attempts to invoke function fn:escape-html-uri.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}escape-html-uri#1('
    // Expected: http://www.example.com/
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_173() {
    // Attempts to look up function fn:true.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}true#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_174() {
    // Attempts to invoke function fn:true.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}true#0()
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_175() {
    // Attempts to look up function fn:false.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}false#0)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_176() {
    // Attempts to invoke function fn:false.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}false#0()
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_177() {
    // Attempts to look up function fn:boolean.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}boolean#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_178() {
    // Attempts to invoke function fn:boolean.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}boolean#1(\"string\
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_179() {
    // Attempts to look up function fn:not.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}not#1)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_180() {
    // Attempts to invoke function fn:not.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}not#1(\"string\")
    // Expected: false
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_181() {
    // Attempts to look up function fn:years-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}years-from-d
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_182() {
    // Attempts to invoke function fn:years-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}years-from-duration
    // Expected: 21
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_183() {
    // Attempts to look up function fn:months-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}months-from-
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_184() {
    // Attempts to invoke function fn:months-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}months-from-duratio
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_185() {
    // Attempts to look up function fn:days-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}days-from-du
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_186() {
    // Attempts to invoke function fn:days-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}days-from-duration#
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_187() {
    // Attempts to look up function fn:hours-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}hours-from-d
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_188() {
    // Attempts to invoke function fn:hours-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}hours-from-duration
    // Expected: 10
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_189() {
    // Attempts to look up function fn:minutes-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}minutes-from
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_190() {
    // Attempts to invoke function fn:minutes-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}minutes-from-durati
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_191() {
    // Attempts to look up function fn:seconds-from-duration.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}seconds-from
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_192() {
    // Attempts to invoke function fn:seconds-from-duration.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}seconds-from-durati
    // Expected: 12.5
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_193() {
    // Attempts to look up function fn:dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}dateTime#2)
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_194() {
    // Attempts to invoke function fn:dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}dateTime#2(xs:date(
    // Expected: 2012-01-01T00:00:00Z
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_195() {
    // Attempts to look up function fn:year-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}year-from-da
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_196() {
    // Attempts to invoke function fn:year-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}year-from-dateTime#
    // Expected: 2012
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_197() {
    // Attempts to look up function fn:month-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}month-from-d
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_198() {
    // Attempts to invoke function fn:month-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}month-from-dateTime
    // Expected: 4
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_199() {
    // Attempts to look up function fn:day-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}day-from-dat
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_200() {
    // Attempts to invoke function fn:day-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}day-from-dateTime#1
    // Expected: 3
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_201() {
    // Attempts to look up function fn:hours-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}hours-from-d
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_202() {
    // Attempts to invoke function fn:hours-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}hours-from-dateTime
    // Expected: 2
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_203() {
    // Attempts to look up function fn:minutes-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}minutes-from
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_204() {
    // Attempts to invoke function fn:minutes-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}minutes-from-dateTi
    // Expected: 1
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_205() {
    // Attempts to look up function fn:seconds-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}seconds-from
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_206() {
    // Attempts to invoke function fn:seconds-from-dateTime.
    // XPath: Q{http://www.w3.org/2005/xpath-functions}seconds-from-dateTi
    // Expected: 0
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}

#[test]
fn function_literal_207() {
    // Attempts to look up function fn:timezone-from-dateTime.
    // XPath: exists(Q{http://www.w3.org/2005/xpath-functions}timezone-fro
    // Expected: true
    // Calls stub function - will fail until prod:NamedFunctionRef is implemented
    let _ = stub_prodnamedfunctionref();
}
