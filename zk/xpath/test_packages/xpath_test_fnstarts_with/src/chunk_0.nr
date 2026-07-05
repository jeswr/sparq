//! Real test vectors for fn:starts-with (F&O sec. 7.5.1)
//! Python oracle: str.startswith; empty prefix always matches.
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::starts_with;

#[test]
fn fnstarts_with_tattoo_tat() {
    // Python: "tattoo".startswith("tat") -> True
    let s: str<6> = "tattoo";
    let pfx: str<3> = "tat";
    assert(starts_with::<6, 3>(s, pfx));
}

#[test]
fn fnstarts_with_tattoo_tow_false() {
    // Python: "tattoo".startswith("tow") -> False
    let s: str<6> = "tattoo";
    let pfx: str<3> = "tow";
    assert(!starts_with::<6, 3>(s, pfx));
}

#[test]
fn fnstarts_with_tattoo_full() {
    // Python: "tattoo".startswith("tattoo") -> True (full match)
    let s: str<6> = "tattoo";
    let pfx: str<6> = "tattoo";
    assert(starts_with::<6, 6>(s, pfx));
}

#[test]
fn fnstarts_with_tattoo_too_long_false() {
    // Python: "tattoo".startswith("tattoox") -> False
    let s: str<6> = "tattoo";
    let pfx: str<7> = "tattoox";
    assert(!starts_with::<6, 7>(s, pfx));
}

#[test]
fn fnstarts_with_empty_prefix() {
    // F&O sec.7.5.1: fn:starts-with($arg1, "") = true for any $arg1
    // Python: "tattoo".startswith("") -> True
    let s: str<6> = "tattoo";
    let pfx: str<0> = "";
    assert(starts_with::<6, 0>(s, pfx));
}

#[test]
fn fnstarts_with_empty_both() {
    // Python: "".startswith("") -> True
    let s: str<0> = "";
    let pfx: str<0> = "";
    assert(starts_with::<0, 0>(s, pfx));
}

#[test]
fn fnstarts_with_empty_haystack_nonempty_prefix() {
    // Python: "".startswith("x") -> False
    let s: str<1> = "\0";
    let pfx: str<1> = "x";
    assert(!starts_with::<1, 1>(s, pfx));
}

#[test]
fn fnstarts_with_nul_padded_prefix() {
    // "he" stored in str<5> (NUL-padded); logical prefix length=2.
    // Python: "hello".startswith("he") -> True
    let s: str<5> = "hello";
    let pfx: str<5> = "he\0\0\0";
    assert(starts_with::<5, 5>(s, pfx));
}

#[test]
fn fnstarts_with_nul_padded_haystack() {
    // "hello" in str<10>; logical content = 5 bytes.
    // Python: "hello".startswith("hel") -> True
    let s: str<10> = "hello\0\0\0\0\0";
    let pfx: str<3> = "hel";
    assert(starts_with::<10, 3>(s, pfx));
}

#[test]
fn fnstarts_with_full_buffer() {
    // Full-buffer haystack, prefix at start.
    // Python: "abcdefgh".startswith("abc") -> True
    let s: str<8> = "abcdefgh";
    let pfx: str<3> = "abc";
    assert(starts_with::<8, 3>(s, pfx));
}
