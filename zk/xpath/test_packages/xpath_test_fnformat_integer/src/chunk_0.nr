//! Test chunk 0 for fn:format-integer
//! Contains 50 tests

use dep::xpath::stub_fnformat_integer;

#[test]
fn format_integer_001() {
    // format-integer with picture='1'
    // XPath: format-integer(123, '1')
    // Expected: '123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_002() {
    // format-integer with picture='001'
    // XPath: format-integer(123, '001')
    // Expected: '123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_003() {
    // format-integer with picture='00001'
    // XPath: format-integer(123, '00001')
    // Expected: '00123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_004() {
    // format-integer negative integer with picture='9'
    // XPath: format-integer(-123, '9')
    // Expected: '-123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_005() {
    // format-integer negative integer with picture='999'
    // XPath: format-integer(-123, '999')
    // Expected: '-123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_006() {
    // format-integer negative integer with picture='99999'
    // XPath: format-integer(-123, '99999')
    // Expected: '-00123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_007() {
    // format-integer zero with picture='0'
    // XPath: format-integer(0, '0')
    // Expected: '0'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_008() {
    // format-integer zero with picture='000'
    // XPath: format-integer(0, '000')
    // Expected: '000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_009() {
    // format-integer zero with picture='00000'
    // XPath: format-integer(0, '00000')
    // Expected: '00000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_010() {
    // format-integer with picture='a'
    // XPath: string-join(for $i in 1 to 4 return format-integer($i, 'a'))
    // Expected: 'abcd'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_011() {
    // format-integer with picture='A'
    // XPath: string-join(for $i in 1 to 4 return format-integer($i, 'A'))
    // Expected: 'ABCD'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_012() {
    // format-integer with picture='i' (lower-case roman numerals)
    // XPath: string-join(for $i in 1 to 20 return format-integer($i, 'i')
    // Expected: 'i|ii|iii|iv|v|vi|vii|viii|ix|x|xi|xii|x
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_013() {
    // format-integer with picture='I' (upper-case roman numerals)
    // XPath: string-join(for $i in 1 to 20 return format-integer($i, 'I')
    // Expected: 'I|II|III|IV|V|VI|VII|VIII|IX|X|XI|XII|X
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_014() {
    // format-integer using Arabic digits
    // XPath: string-join(for $i in 1 to 20 return format-integer($i, '?')
    // Expected: '?|?|?|?|?|?|?|?|?|??|??|??|??|??|??|??|
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_015() {
    // format-integer using Arabic digits
    // XPath: string-join(for $i in 1 to 20 return format-integer($i, '?')
    // Expected: '?|?|?|?|?|?|?|?|?|??|??|??|??|??|??|??|
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_016() {
    // format-integer with title case words
    // XPath: string-join(for $i in 1 to 10 return format-integer($i, 'Ww'
    // Expected: 'One|Two|Three|Four|Five|Six|Seven|Eight
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_017() {
    // format-integer with upper-case words
    // XPath: string-join(for $i in 1 to 10 return format-integer($i, 'W')
    // Expected: 'ONE|TWO|THREE|FOUR|FIVE|SIX|SEVEN|EIGHT
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_018() {
    // format-integer with lower-case words
    // XPath: string-join(for $i in 1 to 10 return format-integer($i, 'w')
    // Expected: 'one|two|three|four|five|six|seven|eight
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_021() {
    // format-integer - picture grouping
    // XPath: format-integer(1500000, '0,000')
    // Expected: '1,500,000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_022() {
    // format-integer - picture grouping, with hash symbols at the end
    // XPath: format-integer(1500000, '#,###,000')
    // Expected: '1,500,000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_025() {
    // format-integer - picture grouping error
    // XPath: format-integer(1500000, '#')
    // Expected: '1500000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_026() {
    // format-integer - picture grouping error
    // XPath: format-integer(1500000, '#a')
    // Expected: '1500000'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_029() {
    // format-integer - picture grouping, with hash symbols at the end; non-BMP groupin
    // XPath: format-integer(1500000, '###?0,00')
    // Expected: '1500?0,00'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_030() {
    // format-integer - picture with different separators, which are irregular
    // XPath: format-integer(602347826, '#(000)000-000')
    // Expected: '602)347-826'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_031() {
    // format-integer - Ordinal number output request
    // XPath: format-integer(2, 'W;o')
    // Expected: 'SECOND'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_032() {
    // format-integer - Ordinal number output request - German
    // XPath: format-integer(1, 'Ww;o(-er)', 'de' cast as xs:language)
    // Expected: 'Erster'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_032_fr() {
    // format-integer - Ordinal number output request - French
    // XPath: format-integer(2, 'Ww;o', 'fr')
    // Expected: 'Deuxi?me'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_032_it() {
    // format-integer - Ordinal number output request - Italian
    // XPath: format-integer(5, 'Ww;o(-o)', 'it') || ';' || format-integer
    // Expected: 'Quinto;Quinta'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_033() {
    // format-integer - Ordinal number output request
    // XPath: format-integer(1, '1;o(-en)')
    // Expected: '1st'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_035() {
    // format-integer - format modifier t
    // XPath: format-integer(1, 'Ww;t')
    // Expected: 'One'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_038() {
    // format-integer - misplaced parens in picture
    // XPath: format-integer(1234, '()Ww;o')
    // Expected: 1234th
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_039() {
    // format-integer - disallowed characters in picture
    // XPath: format-integer(1234, ' ')
    // Expected: 1234
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_041() {
    // format-integer - non-BMP digits (Osmanya digits)
    // XPath: format-integer(1234, '#,???')
    // Expected: '?,???'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_042() {
    // format-integer - unrecognized numbering sequence - fallback
    // XPath: format-integer(1234, '?')
    // Expected: '1234'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_043() {
    // format-integer - negative ordinal
    // XPath: format-integer(-85, '1;o')
    // Expected: '-85th'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_044() {
    // format-integer - negative ordinal in words
    // XPath: format-integer(-5, 'Ww;o')
    // Expected: '-Fifth'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_045() {
    // format-integer - zero in words
    // XPath: format-integer(0, 'Ww')
    // Expected: 'Zero'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_046() {
    // format-integer with circled digits
    // XPath: string-join(for $i in 1 to 5 return format-integer($i, '?'),
    // Expected: '?|?|?|?|?'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_047() {
    // format-integer with parenthesized digits
    // XPath: string-join(for $i in 1 to 5 return format-integer($i, '?'),
    // Expected: '?|?|?|?|?'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_048() {
    // format-integer with full-stopped digits
    // XPath: string-join(for $i in 1 to 5 return format-integer($i, '?'),
    // Expected: '?|?|?|?|?'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_049() {
    //           format-integer with greek uppercase          Note: see bug 21449. An i
    // XPath: string-join(for $i in 1 to 5 return format-integer($i, '?'),
    // Expected: '?|?|?|?|?'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_050() {
    //           format-integer with greek lowercase.          Note: see bug 21449. An
    // XPath: string-join(for $i in 1 to 5 return format-integer($i,'?'),
    // Expected: '?|?|?|?|?'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_051() {
    // format-integer with space as separator
    // XPath: format-integer(12345678901,'# 000')
    // Expected: '12 345 678 901'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_052() {
    //           format-integer using Kanji sequence (see http://en.wikipedia.org/wiki/
    // XPath: string-join(                 for $i in (1 to 23, 151, 302, 4
    // Expected: '1=?|2=?|3=?|4=?|5=?|6=?|7=?|8=?|9=?|10=
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_053() {
    // format-integer with leading optional-digit-sign
    // XPath: format-integer(123,'#0')
    // Expected: '123'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_059() {
    // format-integer - semicolon at end is ignored
    // XPath: format-integer(1,'001;')
    // Expected: '001'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_060() {
    // format-integer - semicolon as grouping separator
    // XPath: format-integer(1234,'#;##1;')
    // Expected: '1;234'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_062() {
    // Third argument as a string in the form of a language code
    // XPath: format-integer(11, 'Ww', 'en')
    // Expected: 'Eleven'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_063() {
    // Third argument as a string not in the form of a language code
    // XPath: format-integer(11, 'Ww', '@*!+%')
    // Expected: 'Eleven'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}

#[test]
fn format_integer_065() {
    // format-integer - Ordinal number output request - German - using CLDR
    // XPath: format-integer(20, 'Ww;o(%spellout-ordinal)', 'de')
    // Expected: 'Zwanzigste'
    // Calls stub function - will fail until fn:format-integer is implemented
    let _ = stub_fnformat_integer();
}
