//! Real test vectors for fn:contains (F&O sec. 7.5.3)
//! Python oracle: str.__contains__; "" in x is always True; NUL padding = logical terminator.
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::contains;

#[test]
fn fncontains_tattoo_tat() {
    // Python: "tat" in "tattoo" -> True
    let s: str<6> = "tattoo";
    let n: str<3> = "tat";
    assert(contains::<6, 3>(s, n));
}

#[test]
fn fncontains_tattoo_tow_false() {
    // Python: "tow" in "tattoo" -> False
    let s: str<6> = "tattoo";
    let n: str<3> = "tow";
    assert(!contains::<6, 3>(s, n));
}

#[test]
fn fncontains_any_empty_needle() {
    // F&O sec.7.5.3: fn:contains($arg1, "") = true for any $arg1
    // Python: "" in "tattoo" -> True
    let s: str<6> = "tattoo";
    let n: str<0> = "";
    assert(contains::<6, 0>(s, n));
}

#[test]
fn fncontains_empty_both() {
    // Python: "" in "" -> True
    let s: str<0> = "";
    let n: str<0> = "";
    assert(contains::<0, 0>(s, n));
}

#[test]
fn fncontains_empty_haystack_nonempty_needle() {
    // Python: "x" in "" -> False
    let s: str<1> = "\0";
    let n: str<1> = "x";
    assert(!contains::<1, 1>(s, n));
}

#[test]
fn fncontains_padded_needle_midstring() {
    // Python: "ell" in "hello" -> True; needle "ell" in str<6> (NUL-padded)
    // Logical-length semantics: NUL padding is not content.
    let s: str<5> = "hello";
    let n: str<6> = "ell\0\0\0";
    assert(contains::<5, 6>(s, n));
}

#[test]
fn fncontains_needle_equals_haystack() {
    // Python: "hello" in "hello" -> True (exact match)
    let s: str<5> = "hello";
    let n: str<5> = "hello";
    assert(contains::<5, 5>(s, n));
}

#[test]
fn fncontains_needle_longer_than_haystack() {
    // Python: "hello world" in "hello" -> False
    let s: str<5> = "hello";
    let n: str<5> = "world";
    assert(!contains::<5, 5>(s, n));
}

#[test]
fn fncontains_at_start_of_full_buffer() {
    // Full-buffer: haystack fills capacity, needle at start.
    // Python: "abc" in "abcdefgh" -> True
    let s: str<8> = "abcdefgh";
    let n: str<3> = "abc";
    assert(contains::<8, 3>(s, n));
}

#[test]
fn fncontains_at_end_of_full_buffer() {
    // Full-buffer: haystack fills capacity, needle at end.
    // Python: "fgh" in "abcdefgh" -> True
    let s: str<8> = "abcdefgh";
    let n: str<3> = "fgh";
    assert(contains::<8, 3>(s, n));
}

#[test]
fn fncontains_logically_empty_haystack() {
    // Haystack has capacity but logical content is empty (all NUL).
    // Python: "" in "" -> True (empty needle), "x" in "" -> False.
    let s: str<4> = "\0\0\0\0";
    let n_empty: str<0> = "";
    assert(contains::<4, 0>(s, n_empty));
}
