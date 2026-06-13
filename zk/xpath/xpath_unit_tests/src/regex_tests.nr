//! Unit tests for the bounded circuit-friendly regex subset.
//!
//! [OPUS-4.8] sq-y73. These exercise the public API re-exports (the bounded
//! `matches_*` / `replace_literal` primitives). Full `fn:matches` / `fn:replace`
//! (XML-Schema PCRE) remain stubs by design -- see xpath/src/regex.nr.

use dep::xpath::{
    CharClass, matches_anchored, matches_char_class_plus, matches_char_class_star, matches_literal,
    matches_prefix, replace_literal,
};

#[test]
fn test_matches_literal_via_api() {
    let s: str<11> = "hello world";
    let p: str<5> = "world";
    assert(matches_literal::<11, 5>(s, p, false));
    let q: str<5> = "WORLD";
    assert(!matches_literal::<11, 5>(s, q, false));
    assert(matches_literal::<11, 5>(s, q, true)); // 'i' flag
}

#[test]
fn test_matches_char_class_via_api() {
    // matches('abZ', '[A-Za-z]*') -> true
    let s: str<3> = "abZ";
    let class: CharClass<0, 2> = CharClass {
        singles: [],
        n_singles: 0,
        range_lo: [65, 97],
        range_hi: [90, 122],
        n_ranges: 2,
        negated: false,
    };
    assert(matches_char_class_star::<3, 0, 2>(s, class, false));
    assert(matches_char_class_plus::<3, 0, 2>(s, class, false));
}

#[test]
fn test_matches_char_class_singles_via_api() {
    // [aeiou]+ over "oui" -> true ; over "xyz" -> false
    let vowels: CharClass<5, 0> = CharClass {
        singles: [97, 101, 105, 111, 117],
        n_singles: 5,
        range_lo: [],
        range_hi: [],
        n_ranges: 0,
        negated: false,
    };
    let yes: str<3> = "oui";
    let no: str<3> = "xyz";
    assert(matches_char_class_plus::<3, 5, 0>(yes, vowels, false));
    assert(!matches_char_class_plus::<3, 5, 0>(no, vowels, false));
}

#[test]
fn test_matches_anchored_via_api() {
    let s: str<3> = "abc";
    let exact: str<3> = "abc";
    let partial: str<2> = "ab";
    assert(matches_anchored::<3, 3>(s, exact, false));
    assert(!matches_anchored::<3, 2>(s, partial, false));
}

#[test]
fn test_matches_prefix_via_api() {
    let s: str<7> = "testing";
    let p: str<4> = "test";
    assert(matches_prefix::<7, 4>(s, p, false));
}

#[test]
fn test_replace_literal_via_api() {
    // replace("a.b.c", ".", "/") -> "a/b/c"
    let s: str<5> = "a.b.c";
    let p: str<1> = ".";
    let r: str<1> = "/";
    let (out, len) = replace_literal::<5, 1, 1, 5>(s, p, r);
    assert(len == 5);
    let expected = "a/b/c".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}
