//! Real test vectors for fn:string-length (F&O sec. 7.4.1)
//! Counts Unicode codepoints (lead bytes in well-formed UTF-8), NOT bytes.
//! Python oracle: len(str) - Python strings are codepoint sequences.
//! NUL = logical terminator (not U+0000 content); padded bytes not counted.
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::string_length;

#[test]
fn fnstring_length_hello() {
    // Python: len("hello") -> 5
    let s: str<5> = "hello";
    assert(string_length::<5>(s) == 5);
}

#[test]
fn fnstring_length_empty() {
    // F&O sec.7.4.1: fn:string-length("") = 0
    // Python: len("") -> 0
    let s: str<0> = "";
    assert(string_length::<0>(s) == 0);
}

#[test]
fn fnstring_length_single_char() {
    // Python: len("a") -> 1
    let s: str<1> = "a";
    assert(string_length::<1>(s) == 1);
}

#[test]
fn fnstring_length_e_acute_one_codepoint() {
    // Python: len(U+00E9 e-acute) -> 1 (2 UTF-8 bytes, 1 codepoint)
    // UTF-8 bytes: [0xC3, 0xA9] = [195, 169]; lead byte count = 1.
    let s: str<2> = "é";
    assert(string_length::<2>(s) == 1);
}

#[test]
fn fnstring_length_mixed_ascii_multibyte() {
    // Python: len("a" + U+00E9) -> 2 (1 ASCII + 1 two-byte codepoint = 3 bytes, 2 codepoints)
    let s: str<3> = "aé";
    assert(string_length::<3>(s) == 2);
}

#[test]
fn fnstring_length_nul_padded_stops_at_nul() {
    // "hello" stored in str<10>; logical content = 5 bytes; NUL padding ignored.
    // Python: len("hello") -> 5
    let s: str<10> = "hello\0\0\0\0\0";
    assert(string_length::<10>(s) == 5);
}

#[test]
fn fnstring_length_full_buffer() {
    // Full-buffer: "abcdefgh" fills str<8>; all 8 bytes are content (no NUL).
    // Python: len("abcdefgh") -> 8
    let s: str<8> = "abcdefgh";
    assert(string_length::<8>(s) == 8);
}

#[test]
fn fnstring_length_known_example() {
    // F&O spec example: fn:string-length("Harp not on that string, madam; that string") = 43
    // Python: len("Harp not on that string, madam; that string") -> 43
    let s: str<43> = "Harp not on that string, madam; that string";
    assert(string_length::<43>(s) == 43);
}

#[test]
fn fnstring_length_logically_empty_padded_buffer() {
    // Capacity-4 buffer with all NUL bytes -> logical length = 0.
    // Python: len("") -> 0
    let s: str<4> = "\0\0\0\0";
    assert(string_length::<4>(s) == 0);
}
