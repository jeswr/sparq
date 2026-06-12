//! Test chunk 0 for fn:replace
//! Contains 50 tests

use dep::xpath::stub_fnreplace;

#[test]
fn fn_replace3args_1() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_2() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_3() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_4() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_5() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_6() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(\"This is a characte\",\"This is a characte\",\"T
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace3args_7() {
    //  Evaluates The 'replace' function with the arguments set as follows: $input = xs
    // XPath: fn:replace(xs:string(\"This is a characte\"),xs:string(\"Thi
    // Expected: This is a characte
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_1() {
    // Evaluation of replace function with replacement = '*' as an example 1 for this f
    // XPath: replace(\"abracadabra\", \"bra\", \"*\")
    // Expected: a*cada*
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_2() {
    // Evaluation of replace function with pattern = 'a.*a' and replacement = '*' as an
    // XPath: replace(\"abracadabra\", \"a.*a\", \"*\")
    // Expected: *
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_3() {
    // Evaluation of replace function with pattern = 'a.*?a' and replacement = '*' as a
    // XPath: replace(\"abracadabra\", \"a.*?a\", \"*\")
    // Expected: *c*bra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_4() {
    // Evaluation of replace function with pattern = 'a' and replacement = '' as an exa
    // XPath: replace(\"abracadabra\", \"a\", \"\")
    // Expected: brcdbr
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_5() {
    // Evaluation of replace function with pattern = 'a(.)' and replacement = 'a$1$1' a
    // XPath: replace(\"abracadabra\", \"a(.)\", \"a$1$1\")
    // Expected: abbraccaddabbra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_7() {
    // Evaluation of replace function with pattern = 'A+' and replacement = 'b' as an e
    // XPath: replace(\"AAAA\", \"A+\", \"b\")
    // Expected: b
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_8() {
    // Evaluation of replace function with pattern = 'A+?' and replacement = 'b' as an
    // XPath: replace(\"AAAA\", \"A+?\", \"b\")
    // Expected: bbbb
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_9() {
    // Evaluation of replace function with pattern = '^(.*?)d(.*)' and replacement = '$
    // XPath: replace(\"darted\", \"^(.*?)d(.*)$\", \"$1c$2\")
    // Expected: carted
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_10() {
    // Two alternatives within the pattern both match at the same position in the $inpu
    // XPath: fn:replace(\"abcd\", \"(ab)|(a)\", \"[1=$1][2=$2]\")
    // Expected: [1=ab][2=]cd
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_11() {
    // Evaluation of fn:replace function with input set to empty sequence. Uses the fn:
    // XPath: fn:count(fn:replace((), \"bra\", \"*\"))
    // Expected: 1
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_12() {
    // Evaluate that calling function with flags omitted is same as flags being the zer
    // XPath: replace(\"abracadabra\", \"bra\", \"*\", \"\")
    // Expected: a*cada*
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_13() {
    // Evaluation of replace function with pattern set to '\?' for an input string that
    // XPath: fn:replace(\"abracadabra?abracadabra\", \"\?\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_14() {
    // Evaluation of replace function with pattern set to '\*' for an input string that
    // XPath: fn:replace(\"abracadabra*abracadabra\", \"\*\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_15() {
    // Evaluation of replace function with pattern set to '\+' for an input string that
    // XPath: fn:replace(\"abracadabra+abracadabra\", \"\+\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_16() {
    // Evaluation of replace function with pattern set to '\{' for an input string that
    // XPath: fn:replace(\"abracadabra{abracadabra\", \"\{\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_17() {
    // Evaluation of replace function with pattern set to '\}' for an input string that
    // XPath: fn:replace(\"abracadabra}abracadabra\", \"\}\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_18() {
    // Evaluation of replace function with pattern set to '\(' for an input string that
    // XPath: fn:replace(\"abracadabra(abracadabra\", \"\(\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_19() {
    // Evaluation of replace function with pattern set to '\)' for an input string that
    // XPath: fn:replace(\"abracadabra)abracadabra\", \"\)\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_20() {
    // Evaluation of replace function with pattern set to '\[' for an input string that
    // XPath: fn:replace(\"abracadabra[abracadabra\", \"\[\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_21() {
    // Evaluation of replace function with pattern set to '\]' for an input string that
    // XPath: fn:replace(\"abracadabra]abracadabra\", \"\]\", \"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_22() {
    // Evaluation of replace function with pattern set to '\-' for an input string that
    // XPath: fn:replace(\"abracadabra-abracadabra\", \"\-\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_23() {
    // Evaluation of replace function with pattern set to '\.' for an input string that
    // XPath: fn:replace(\"abracadabra.abracadabra\", \"\.\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_24() {
    // Evaluation of replace function with pattern set to '\|' for an input string that
    // XPath: fn:replace(\"abracadabra|abracadabra\", \"\|\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_25() {
    // Evaluation of replace function with pattern set to '\\' for an input string that
    // XPath: fn:replace(\"abracadabra\abracadabra\", \"\\\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_26() {
    // Evaluation of replace function with pattern set to '\t' for an input string that
    // XPath: fn:replace(\"abracadabra abracadabra\", \"\t\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_27() {
    // Evaluation of replace function with pattern set to '\n' for an input string that
    // XPath: fn:replace(\"abracadabra abracadabra\", \"\n\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_28() {
    // Evaluation of replace function with pattern set to 'aa{1}' (exact quantity) for
    // XPath: fn:replace(\"abracadabraabracadabra\", \"aa{1}\",\"with\")
    // Expected: abracadabrwithbracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_29() {
    // Evaluation of replace function with pattern set to 'aa{1,}' (min quantity) for a
    // XPath: fn:replace(\"abracadabraabracadabraabracadabra\", \"aa{1,}\"
    // Expected: abracadabrwithbracadabrwithbracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_30() {
    // Evaluation of replace function with pattern set to 'aa{1,2}' (range quantity) fo
    // XPath: fn:replace(\"abracadabraabracadabraabracadabra\", \"aa{1,2}\
    // Expected: abracadabrwithbracadabrwithbracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_31() {
    // Evaluation of replace function with pattern set to '\^'.
    // XPath: fn:replace(\"abracadabra^abracadabra\", \"\^\",\"with\")
    // Expected: abracadabrawithabracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_32() {
    // Evaluation of replace function with pattern set to '^a'.
    // XPath: fn:replace(\"abracadabra\", \"^a\",\"with\")
    // Expected: withbracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_33() {
    // Evaluation of replace function with pattern that does not match the input string
    // XPath: fn:replace(\"abracadabra\", \"ww\",\"with\")
    // Expected: abracadabra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_34() {
    // Evaluation of replace function with q flag.
    // XPath: fn:replace(\"abracadabra\", \"a\", \"$1\", \"q\")
    // Expected: $1br$1c$1d$1br$1
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_35() {
    // Evaluation of replace function with q flag.
    // XPath: fn:replace(\"a?bracadabra?\", \"a?\", \"\$1\", \"q\")
    // Expected: \$1bracadabr\$1
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_36() {
    // Evaluation of replace function with non-capturing groups.
    // XPath: fn:replace(\"abracadabra\", \"([aA])(?:br)([aA])\", \"$1**$2
    // Expected: a**acada**a
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_37() {
    // Evaluation of replace function with escaped $ sign in replacement string.
    // XPath: fn:replace(\"abracadabra\", \"a\", \"\$\")
    // Expected: $br$c$d$br$
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_38() {
    // Evaluation of replace function with escaped $ sign in replacement string.
    // XPath: fn:replace(\"abracadabra\", \"(a)\", \"\$$1\")
    // Expected: $abr$ac$ad$abr$a
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_39() {
    // Evaluation of replace function with escaped \ sign in replacement string.
    // XPath: fn:replace(\"abracadabra\", \"a\", \"\\\")
    // Expected: \br\c\d\br\
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_40() {
    // Evaluation of replace with double-digit capture
    // XPath: fn:replace(\"abracadabra\", \"((((( ((((( (((((a))))) )))))
    // Expected: |aa|br|aa|c|aa|d|aa|br|aa|
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_41() {
    // Evaluation of replace with double-digit capture
    // XPath: fn:replace(\"abracadabra\", \"((((( ((((( (((((a))))) )))))
    // Expected: a20bra20ca20da20bra20
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_42() {
    // Evaluation of replace with double-digit capture beyond max capture value
    // XPath: fn:replace(\"abracadabra\", \"((((( ((((( (((((a)(b))))) )))
    // Expected: (ab.a.b.ab7)racad(ab.a.b.ab7)ra
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_43() {
    //  '.' does NOT match CR in default mode
    // XPath: fn:replace(concat('Mary', codepoints-to-string(13), 'Jones')
    // Expected: concat('Mary', codepoints-to-string(13),
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}

#[test]
fn fn_replace_44() {
    //  '.' does match CR in dot-all mode
    // XPath: fn:replace(concat('Mary', codepoints-to-string(13), 'Jones')
    // Expected: 'Jacob Jones'
    // Calls stub function - will fail until fn:replace is implemented
    let _ = stub_fnreplace();
}
