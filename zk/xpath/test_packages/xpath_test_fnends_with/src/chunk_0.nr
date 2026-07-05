//! Real test vectors for fn:ends-with (F&O sec. 7.5.2)
//! Python oracle: str.endswith; empty suffix always matches.
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::ends_with;

#[test]
fn fnends_with_tattoo_oo() {
    // Python: "tattoo".endswith("oo") -> True
    let s: str<6> = "tattoo";
    let suf: str<2> = "oo";
    assert(ends_with::<6, 2>(s, suf));
}

#[test]
fn fnends_with_tattoo_ta_false() {
    // Python: "tattoo".endswith("ta") -> False (prefix, not suffix)
    let s: str<6> = "tattoo";
    let suf: str<2> = "ta";
    assert(!ends_with::<6, 2>(s, suf));
}

#[test]
fn fnends_with_tattoo_tattoo() {
    // Python: "tattoo".endswith("tattoo") -> True (full string suffix)
    let s: str<6> = "tattoo";
    let suf: str<6> = "tattoo";
    assert(ends_with::<6, 6>(s, suf));
}

#[test]
fn fnends_with_empty_suffix() {
    // F&O sec.7.5.2: fn:ends-with($arg1, "") = true for any $arg1
    // Python: "tattoo".endswith("") -> True
    let s: str<6> = "tattoo";
    let suf: str<0> = "";
    assert(ends_with::<6, 0>(s, suf));
}

#[test]
fn fnends_with_empty_both() {
    // Python: "".endswith("") -> True
    let s: str<0> = "";
    let suf: str<0> = "";
    assert(ends_with::<0, 0>(s, suf));
}

#[test]
fn fnends_with_empty_haystack_nonempty_suffix() {
    // Python: "".endswith("x") -> False
    let s: str<1> = "\0";
    let suf: str<1> = "x";
    assert(!ends_with::<1, 1>(s, suf));
}

#[test]
fn fnends_with_suffix_longer_than_haystack() {
    // Python: "lo".endswith("hello") -> False
    let s: str<2> = "lo";
    let suf: str<5> = "hello";
    assert(!ends_with::<2, 5>(s, suf));
}

#[test]
fn fnends_with_nul_padded_haystack_logical_length() {
    // "hello" stored in str<10>; logical end is after 'o', not at capacity.
    // Python: "hello".endswith("lo") -> True
    let s: str<10> = "hello\0\0\0\0\0";
    let suf: str<2> = "lo";
    assert(ends_with::<10, 2>(s, suf));
}

#[test]
fn fnends_with_nul_padded_suffix() {
    // Suffix "lo" stored in str<5> (NUL-padded); logical suffix length=2.
    // Python: "hello".endswith("lo") -> True
    let s: str<5> = "hello";
    let suf: str<5> = "lo\0\0\0";
    assert(ends_with::<5, 5>(s, suf));
}

#[test]
fn fnends_with_full_buffer() {
    // Full-buffer haystack, suffix at end.
    // Python: "abcdefgh".endswith("fgh") -> True
    let s: str<8> = "abcdefgh";
    let suf: str<3> = "fgh";
    assert(ends_with::<8, 3>(s, suf));
}
