//! Test chunk 0 for fn:normalize-unicode
//! Contains 31 tests

use dep::xpath::stub_fnnormalize_unicode;

#[test]
fn fn_normalize_unicode1args_1() {
    // Test normalize-unicode with simple text input
    // XPath: normalize-unicode('Nothing to normalize.')
    // Expected: Nothing to normalize.
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode1args_2() {
    // Test normalize-unicode with empty sequence argument
    // XPath: normalize-unicode(())
    // Expected:
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode1args_3() {
    // Test fn:normalize-unicode on combining characters for LATIN CAPITAL LETTER A WIT
    // XPath: matches('?', normalize-unicode('A??'))
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode1args_4() {
    // Test fn:normalize-unicode on combining characters for LATIN CAPITAL LETTER A WIT
    // XPath: matches('?', normalize-unicode('A?'))
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode1args_5() {
    // Test fn:normalize-unicode on combining characters for LATIN CAPITAL LETTER A WIT
    // XPath: matches('A?', normalize-unicode('A?'))
    // Expected: false
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode1args_6() {
    // Test fn:normalize-unicode on combining characters for LATIN CAPITAL LETTER A WIT
    // XPath: (normalize-unicode('?') eq normalize-unicode('?'))
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode2args_1() {
    // Test normalize-unicode with simple text input and NFC
    // XPath: normalize-unicode('Nothing to normalize.', 'NFC')
    // Expected: Nothing to normalize.
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode2args_2() {
    // Test normalize-unicode with simple text input and NFC spelled differently
    // XPath: normalize-unicode('Nothing to normalize.', 'nFc')
    // Expected: Nothing to normalize.
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode2args_3() {
    // Test normalize-unicode with empty sequence argument and NFC
    // XPath: normalize-unicode((), 'NFC')
    // Expected:
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode2args_4() {
    // Test fn:normalize-unicode where the second argument is the zero-length string, n
    // XPath: (normalize-unicode('?', '') eq normalize-unicode('?', ''))
    // Expected: false
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_1() {
    //  Evaluation of fn:normalize-unicode to ensure that ' NFC ' is the same as 'NFC'.
    // XPath: fn:concat(fn:normalize-unicode(\"?\",\" NFC \"),fn:normalize
    // Expected: ??
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_3() {
    //  Evaluation of fn:normalize-unicode with nothing to normalize and used as argume
    // XPath: fn:upper-case(fn:normalize-unicode(\"normalizedstring\"))
    // Expected: NORMALIZEDSTRING
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_4() {
    //  Evaluation of fn:normalize-unicode with nothing to normalize and used as argume
    // XPath: fn:lower-case(fn:normalize-unicode(\"NORMALIZEDSTRING\"))
    // Expected: normalizedstring
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_5() {
    //  Evaluation of fn:normalize-unicode with nothing to normalize and used as argume
    // XPath: fn:string-length(fn:normalize-unicode(\"NORMALIZEDSTRING\"))
    // Expected: 16
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_6() {
    //  Evaluation of fn:normalize-unicode with nothing to normalize and used as argume
    // XPath: fn:concat(fn:normalize-unicode(\"NORMALIZEDSTRING\"),\"anoth
    // Expected: NORMALIZEDSTRINGanother string
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_7() {
    //  Evaluation of fn:normalize-unicode with nothing to normalize and used as argume
    // XPath: fn:substring(fn:normalize-unicode(\"NORMALIZEDSTRING\"),5)
    // Expected: ALIZEDSTRING
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn fn_normalize_unicode_8() {
    //  Evaluation of fn:normalize-unicode with noncharacter argument
    // XPath: string-to-codepoints(normalize-unicode('?'))
    // Expected: 64976
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_4() {
    //  A test whose essence is: `normalize-unicode('foo') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_5() {
    //  A test whose essence is: `normalize-unicode('foo', 'NFC') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\", \"NFC\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_6() {
    //  A test whose essence is: `normalize-unicode('foo', 'NFD') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\", \"NFD\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_7() {
    //  A test whose essence is: `normalize-unicode('foo', 'NFKD') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\", \"NFKD\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_8() {
    //  A test whose essence is: `normalize-unicode('foo', 'NFKC') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\", \"NFKC\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_9() {
    //  A test whose essence is: `normalize-unicode('f oo', '') eq 'f oo'`.
    // XPath: normalize-unicode(\"f oo\", \"\") eq \"f oo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_10() {
    //  A test whose essence is: `normalize-unicode('foo', '') eq 'foo'`.
    // XPath: normalize-unicode(\"foo\", \"\") eq \"foo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_11() {
    //  A test whose essence is: `normalize-unicode('f oo') eq 'f oo'`.
    // XPath: normalize-unicode(\"f oo\") eq \"f oo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn k_normalizeunicodefunc_12() {
    //  A test whose essence is: `normalize-unicode('f oo', 'NFC') eq 'f oo'`.
    // XPath: normalize-unicode(\"f oo\", \"NFC\") eq \"f oo\"
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn cbcl_fn_normalize_unicode_001() {
    //  tests FULLY-NORMALIZED
    // XPath: normalize-unicode(\"blah\",\"FULLY-NORMALIZED\")
    // Expected: blah
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn cbcl_fn_normalize_unicode_002() {
    //  Tests boolean(normalize-unicode(...))
    // XPath: boolean(normalize-unicode(\"blah\",\"NFC\"))
    // Expected: true
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn cbcl_fn_normalize_unicode_004() {
    //  tests normalize-unicode on empty
    // XPath: normalize-unicode((),\"NFC\")
    // Expected:
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn cbcl_fn_normalize_unicode_005() {
    //  Tests normalize-unicode on empty
    // XPath: normalize-unicode(\"\",\"NFC\")
    // Expected:
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}

#[test]
fn cbcl_fn_normalize_unicode_006() {
    //  Tests a composing character by itself
    // XPath: normalize-unicode(codepoints-to-string(2494),\"FULLY-NORMALI
    // Expected: ?
    // Calls stub function - will fail until fn:normalize-unicode is implemented
    let _ = stub_fnnormalize_unicode();
}
