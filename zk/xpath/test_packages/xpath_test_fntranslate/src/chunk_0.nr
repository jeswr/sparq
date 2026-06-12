//! Test chunk 0 for fn:translate
//! Contains 35 tests

use dep::xpath::stub_fntranslate;

#[test]
fn fn_translate3args_1() {
    // Test simple translate expression
    // XPath: translate('---abcABCxyz---','-abcABCxyz','1ABCabcXYZ')
    // Expected: 111ABCabcXYZ111
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate3args_2() {
    // Test translate on space, tab, and newline
    // XPath: translate('newline tab space ','   ','123')
    // Expected: newline1tab2space3
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate3args_3() {
    // Test translate with zero-length string argument
    // XPath: translate('','-','x')
    // Expected:
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate3args_4() {
    // Test translate with an empty sequence argument
    // XPath: translate((),'-','x')
    // Expected:
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_1() {
    //  Simple test of translate function as per example one for this function from the
    // XPath: fn:translate(\"bar\",\"abc\",\"ABC\")
    // Expected: BAr
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_2() {
    //  Simple test of translate function as per example two for this function from the
    // XPath: fn:translate(\"--aaa--\",\"abc-\",\"ABC\")
    // Expected: AAA
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_3() {
    //  Simple test of translate function as per example three for this function from t
    // XPath: fn:translate(\"abcdabc\", \"abc\", \"AB\")
    // Expected: ABdAB
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_4() {
    //  Evaluation of translate function. Translate lower case letters to upper case le
    // XPath: fn:translate(\"acdefghijklmnopqrstuvwxyz\", \"abcdefghijklmn
    // Expected: ACDEFGHIJKLMNOPQRSTUVWXYZ
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_5() {
    //  Evaluation of translate function. Translate upper case letters to lower case le
    // XPath: fn:translate(\"ABCDEFGHIJKLMNOPQRSTUVWXYZ\",\"ABCDEFGHIJKLMN
    // Expected: abcdefghijklmnopqrstuvwxyz
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_6() {
    //  Evaluation of translate function, where all three arguments are the zero length
    // XPath: fn:count(fn:translate(\"\",\"\",\"\"))
    // Expected: 1
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_7() {
    //  Evaluation of translate function, where the first and third arguments are the s
    // XPath: fn:translate(\"ABC\", \"ABC\", \"ABC\")
    // Expected: ABC
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_8() {
    //  Evaluation of translate function, where all arguments are the same (numbers).
    // XPath: fn:translate(\"123\", \"123\", \"123\")
    // Expected: 123
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_9() {
    //  Evaluation of translate function, there is a mixture of numbers and letters.
    // XPath: fn:translate(\"123ABC\", \"123ABC\", \"123ABC\")
    // Expected: 123ABC
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_10() {
    //  Evaluation of translate function, where all three arguments are an invocation t
    // XPath: fn:translate(fn:string(\"ABC\"), fn:string(\"ABC\"), fn:stri
    // Expected: ABC
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_11() {
    //  Evaluation of translate function as an argument to the 'fn:string' function.
    // XPath: fn:string(fn:translate(\"ABC\", \"ABC\", \"ABC\"))
    // Expected: ABC
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_12() {
    //  Evaluation of translate function as an argument to the 'fn:string-length' funct
    // XPath: fn:string-length(fn:translate(\"ABC\",\"ABC\",\"ABC\"))
    // Expected: 3
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_13() {
    //  Evaluation of translate function as an argument to the 'xs:decimal' constructor
    // XPath: xs:decimal(fn:translate(\"123\",\"123\",\"123\"))
    // Expected: 123
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_14() {
    //  Evaluation of translate function as an argument to the 'xs:integer' constructor
    // XPath: xs:integer(fn:translate(\"123\",\"123\",\"123\"))
    // Expected: 123
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_15() {
    //  Evaluation of translate function as an argument to the 'xs:float' constructor f
    // XPath: xs:float(fn:translate(\"123\",\"123\",\"123\"))
    // Expected: 123
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_16() {
    //  Evaluation of translate function as an argument to the 'xs:double' constructor
    // XPath: xs:double(fn:translate(\"123\",\"123\",\"123\"))
    // Expected: 123
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_17() {
    // Evaluation of translate function using non-BMP characters.
    // XPath: translate(\"abcd\", \"?a\", \"xy\")
    // Expected: "ybcd"
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_18() {
    // Evaluation of translate function using non-BMP characters.
    // XPath: translate(\"abcd\", \"xa\", \"?y\")
    // Expected: "ybcd"
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_19() {
    // Evaluation of translate function using non-BMP characters.
    // XPath: translate(\"abcd?e\", \"a?e\", \"XYZ\")
    // Expected: "XbcdYZ"
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_20() {
    // Evaluation of translate function using non-BMP characters.
    // XPath: translate(\"abcd?e\", \"a?e\", \"?YZ\")
    // Expected: "?bcdYZ"
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_21() {
    // Exercise possible optimization.
    // XPath: current-date() => string() => substring(1, 10) => translate(
    // Expected: "xxxx-xx-xx"
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn fn_translate_22() {
    // Exercise possible optimization.
    // XPath: current-date()[current-date() => string() => string-length()
    // Expected: ""
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_4() {
    //  A test whose essence is: `translate('--aaa--','-','') eq 'aaa'`.
    // XPath: translate(\"--aaa--\",\"-\",\"\") eq \"aaa\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_5() {
    //  A test whose essence is: `translate('--aaa--','bbb++','') eq '--aaa--'`.
    // XPath: translate(\"--aaa--\",\"bbb++\",\"\") eq \"--aaa--\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_6() {
    //  A test whose essence is: `translate('argstr', '', 'matrs') eq 'argstr'`.
    // XPath: translate(\"argstr\", \"\", \"matrs\") eq \"argstr\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_7() {
    //  A test whose essence is: `translate((), 'map', 'trans') eq ''`.
    // XPath: translate((), \"map\", \"trans\") eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_8() {
    //  A test whose essence is: `translate('abcdabc', 'abc', 'AB') eq 'ABdAB'`.
    // XPath: translate(\"abcdabc\", \"abc\", \"AB\") eq \"ABdAB\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_9() {
    //  A test whose essence is: `translate('bar','abc','ABC') eq 'BAr'`.
    // XPath: translate(\"bar\",\"abc\",\"ABC\") eq \"BAr\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn k_translatefunc_10() {
    //  A test whose essence is: `translate('--aaa--','abc-','ABC') eq 'AAA'`.
    // XPath: translate(\"--aaa--\",\"abc-\",\"ABC\") eq \"AAA\"
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn cbcl_fn_translate_001() {
    //  Tests EffectiveBooleanValue on fn:translate
    // XPath: boolean(translate(string-join(for $x in 1 to 10 return \"bla
    // Expected: true
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}

#[test]
fn cbcl_fn_translate_002() {
    //  Tests with surrogates
    // XPath: translate(codepoints-to-string(65536 to 65537),codepoints-to
    // Expected: l
    // Calls stub function - will fail until fn:translate is implemented
    let _ = stub_fntranslate();
}
