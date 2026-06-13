//! XPath 2.0 Regex Functions -- circuit-friendly bounded subset
//!
//! [OPUS-4.8] Added for beads task sq-y73 (string/regex/hash XPath functions).
//! Flag for re-review when Fable returns.
//!
//! ## Scope and honesty note
//!
//! Full `fn:matches` / `fn:replace` per the XPath F&O regex grammar (XML Schema
//! regular expressions) is **NOT implementable in a ZK circuit** and is left as
//! a stub (`stubs::stub_fnmatches` / `stub_fnreplace`). The W3C qt3 `fnmatches`
//! corpus is dominated by features that are intractable in fixed-size,
//! straight-line, data-oblivious circuits:
//!
//! - Backreferences (`([A-Z])\1*`) -- require capture-group state + replay.
//! - Unicode category escapes (`\p{Lu}`, `\P{L}`, `\i`, `\c`) -- need the full
//!   Unicode character database in-circuit.
//! - Character-class subtraction (`[A-Z-[OI]]`), supplementary-plane codepoints.
//! - Unbounded quantifiers (`*`, `+`) over witness-length input -- a Thompson-NFA
//!   product would need an a-priori state bound and a step bound, and the cost
//!   grows with (pattern_states x input_len); only tiny bounded cases are cheap.
//! - Free-spacing (`x`) and case-insensitive (`i`) flag handling, anchors.
//!
//! So this module implements the *circuit-friendly* matching primitives that ARE
//! tractable -- enough to back the common SPARQL `REGEX(?s, "literal")`,
//! `REGEX(?s, "[class]")`, and anchored-pattern filters -- with explicit,
//! documented bounds. These are exposed under names that signal the bounded
//! semantics (`matches_literal`, `matches_char_class_*`, `matches_anchored`, ...)
//! rather than pretending to be a full `fn:matches`.
//!
//! ## Cost model
//!
//! Every function here is a single straight-line pass with no witness-dependent
//! control flow that branches on input length: the loops are `0..N` comptime
//! bounds and accumulate into a boolean with `&`/`|`. Cost is O(N*M) byte
//! comparisons (N = haystack capacity, M = pattern/class capacity), each a cheap
//! `u8` equality. No range proofs, no hashing, no bit-decomposition. This is the
//! cheapest shape for the bounded subset (see `noir-circuit-patterns` "Filters"
//! and the witness-vs-comptime-branch note).
//!
//! All strings are ASCII byte sequences (`str<N>` -> `[u8; N]`); there is no
//! Unicode normalization or case-folding beyond ASCII A-Z <-> a-z.

/// ASCII-lowercase a single byte (A-Z -> a-z), leaving everything else unchanged.
/// Used to implement the case-insensitive (`i`) flag for the bounded subset.
fn ascii_lower(b: u8) -> u8 {
    if (b >= 65) & (b <= 90) {
        b + 32
    } else {
        b
    }
}

/// Compare two bytes for equality, optionally case-insensitively (ASCII only).
fn byte_eq(a: u8, b: u8, ci: bool) -> bool {
    if ci {
        ascii_lower(a) == ascii_lower(b)
    } else {
        a == b
    }
}

/// `fn:matches($input, $pattern)` -- **literal, unanchored** subset.
///
/// Returns true iff `pattern` occurs anywhere in `input` as a literal byte
/// sequence (no metacharacters interpreted). This is `fn:contains` lifted into
/// the regex family, and is the honest answer for `REGEX(?s, "lit")` where the
/// pattern has no metacharacters. `ci` selects the case-insensitive (`i`) flag.
///
/// Bound: O(N*M). Trailing NUL bytes in either operand terminate the logical
/// string (the `str<N>` capacity convention used throughout this crate).
pub fn matches_literal<let N: u32, let M: u32>(input: str<N>, pattern: str<M>, ci: bool) -> bool {
    let hay = input.as_bytes();
    let pat = pattern.as_bytes();

    // Logical pattern length (up to first NUL).
    let mut plen: u32 = 0;
    let mut pat_ended = false;
    for j in 0..M {
        let is_nul = pat[j] == 0;
        if (!pat_ended) & (!is_nul) {
            plen += 1;
        }
        pat_ended = pat_ended | is_nul;
    }

    // Empty pattern matches everything.
    let empty_pat = plen == 0;
    let mut found = empty_pat;

    if (!empty_pat) & (M <= N) {
        // Try every start position; data-oblivious (no early break).
        for i in 0..N {
            // Can this position even fit the pattern?
            if (i as u32) + plen <= (N as u32) {
                let mut here = true;
                for j in 0..M {
                    if (j as u32) < plen {
                        let h = hay[i + j];
                        if !byte_eq(h, pat[j], ci) {
                            here = false;
                        }
                    }
                }
                found = found | here;
            }
        }
    }

    found
}

/// A bounded single-character class, e.g. `[abc]`, `[a-z]`, `[^0-9]`, `[a-zA-Z_]`.
///
/// Represented as a fixed set of allowed bytes plus a fixed set of ranges; this
/// is the circuit-friendly encoding of an XML-Schema character class restricted
/// to ASCII literals and ranges (no `\p{...}` categories, no class subtraction).
///
/// - `singles` / `n_singles`: explicit member bytes (`[abc]`).
/// - `range_lo` / `range_hi` / `n_ranges`: inclusive ranges (`[a-z]`).
/// - `negated`: the `[^...]` form -- membership is inverted.
pub struct CharClass<let S: u32, let R: u32> {
    pub singles: [u8; S],
    pub n_singles: u32,
    pub range_lo: [u8; R],
    pub range_hi: [u8; R],
    pub n_ranges: u32,
    pub negated: bool,
}

impl<let S: u32, let R: u32> CharClass<S, R> {
    /// Does `byte` match this class? `ci` applies ASCII case-insensitivity to
    /// both the input byte and the class members/ranges.
    pub fn contains(self, byte: u8, ci: bool) -> bool {
        let b = if ci { ascii_lower(byte) } else { byte };

        let mut member = false;
        for k in 0..S {
            if (k as u32) < self.n_singles {
                let m = if ci { ascii_lower(self.singles[k]) } else { self.singles[k] };
                if m == b {
                    member = true;
                }
            }
        }
        for k in 0..R {
            if (k as u32) < self.n_ranges {
                if ci {
                    // Case-insensitive ranges: check both the raw byte and its
                    // lowercase against the lowercased bounds. ASCII letter
                    // ranges are the only ones where folding matters.
                    let lo = ascii_lower(self.range_lo[k]);
                    let hi = ascii_lower(self.range_hi[k]);
                    if (b >= lo) & (b <= hi) {
                        member = true;
                    }
                    if (byte >= self.range_lo[k]) & (byte <= self.range_hi[k]) {
                        member = true;
                    }
                } else {
                    if (byte >= self.range_lo[k]) & (byte <= self.range_hi[k]) {
                        member = true;
                    }
                }
            }
        }

        if self.negated {
            !member
        } else {
            member
        }
    }
}

/// `fn:matches` for a single-character-class pattern with an implicit `*`
/// (i.e. the qt3 `[A-Z]*` shape): returns true iff **every** logical byte of the
/// input is in the class. An empty input matches (consistent with `class*`).
///
/// This covers the common `matches('abZ', '[A-Z]*')`-style cases that don't use
/// backreferences/categories. `ci` selects the case-insensitive flag.
pub fn matches_char_class_star<let N: u32, let S: u32, let R: u32>(
    input: str<N>,
    class: CharClass<S, R>,
    ci: bool,
) -> bool {
    let hay = input.as_bytes();
    let mut all_in = true;
    for i in 0..N {
        let byte = hay[i];
        if byte != 0 {
            if !class.contains(byte, ci) {
                all_in = false;
            }
        }
    }
    all_in
}

/// `fn:matches` for a single-character-class pattern with an implicit `+`:
/// true iff the input is non-empty and every logical byte is in the class.
pub fn matches_char_class_plus<let N: u32, let S: u32, let R: u32>(
    input: str<N>,
    class: CharClass<S, R>,
    ci: bool,
) -> bool {
    let hay = input.as_bytes();
    let mut all_in = true;
    let mut any = false;
    for i in 0..N {
        let byte = hay[i];
        if byte != 0 {
            any = true;
            if !class.contains(byte, ci) {
                all_in = false;
            }
        }
    }
    all_in & any
}

/// Whether the input contains at least one byte matching the class (the
/// `fn:matches(s, '[class]')` default -- unanchored, one occurrence).
pub fn matches_char_class_any<let N: u32, let S: u32, let R: u32>(
    input: str<N>,
    class: CharClass<S, R>,
    ci: bool,
) -> bool {
    let hay = input.as_bytes();
    let mut any = false;
    for i in 0..N {
        let byte = hay[i];
        if byte != 0 {
            if class.contains(byte, ci) {
                any = true;
            }
        }
    }
    any
}

/// `fn:matches` with both `^` and `$` anchors over a literal pattern: true iff
/// the whole logical input equals the pattern exactly (i.e. `^lit$`). This is
/// the anchored counterpart to `matches_literal`.
pub fn matches_anchored<let N: u32, let M: u32>(input: str<N>, pattern: str<M>, ci: bool) -> bool {
    let hay = input.as_bytes();
    let pat = pattern.as_bytes();

    let mut hlen: u32 = 0;
    let mut h_end = false;
    for i in 0..N {
        let z = hay[i] == 0;
        if (!h_end) & (!z) {
            hlen += 1;
        }
        h_end = h_end | z;
    }
    let mut plen: u32 = 0;
    let mut p_end = false;
    for j in 0..M {
        let z = pat[j] == 0;
        if (!p_end) & (!z) {
            plen += 1;
        }
        p_end = p_end | z;
    }

    let mut eq = hlen == plen;
    // Compare overlapping prefix; lengths-equal guard above makes this exact.
    let lim = if N < M { N } else { M };
    for i in 0..lim {
        if (i as u32) < plen {
            if !byte_eq(hay[i], pat[i], ci) {
                eq = false;
            }
        }
    }
    eq
}

/// `^prefix` anchored match: true iff the input begins with `pattern` (literal).
/// Equivalent to `fn:starts-with` lifted into the regex family.
pub fn matches_prefix<let N: u32, let M: u32>(input: str<N>, pattern: str<M>, ci: bool) -> bool {
    let hay = input.as_bytes();
    let pat = pattern.as_bytes();

    let mut plen: u32 = 0;
    let mut p_end = false;
    for j in 0..M {
        let z = pat[j] == 0;
        if (!p_end) & (!z) {
            plen += 1;
        }
        p_end = p_end | z;
    }

    if plen > (N as u32) {
        false
    } else {
        let mut ok = true;
        for j in 0..M {
            if (j as u32) < plen {
                if !byte_eq(hay[j], pat[j], ci) {
                    ok = false;
                }
            }
        }
        ok
    }
}

/// `fn:replace` -- **literal, fixed-replacement** subset.
///
/// Replaces every non-overlapping literal occurrence of `pattern` in `input`
/// with `repl`. No regex metacharacters, no `$N` capture references in the
/// replacement (those need capture-group state, which the bounded engine does
/// not model). This backs `REPLACE(?s, "lit", "rep")` for literal patterns.
///
/// Returns `([u8; OUT], len)` in the byte-array+length convention used by the
/// rest of this crate's string transforms. `OUT` must be large enough for the
/// worst-case expansion; bytes beyond `OUT` are dropped (length is clamped).
///
/// Bound: O(N*M) for matching + O(N) for emission. An empty pattern is a no-op
/// (returns the input unchanged), matching neither XPath's error nor an infinite
/// insertion -- documented divergence chosen for circuit determinism.
pub fn replace_literal<let N: u32, let M: u32, let P: u32, let OUT: u32>(
    input: str<N>,
    pattern: str<M>,
    repl: str<P>,
) -> ([u8; OUT], u32) {
    let hay = input.as_bytes();
    let pat = pattern.as_bytes();
    let rep = repl.as_bytes();

    let mut plen: u32 = 0;
    let mut p_end = false;
    for j in 0..M {
        let z = pat[j] == 0;
        if (!p_end) & (!z) {
            plen += 1;
        }
        p_end = p_end | z;
    }
    let mut rlen: u32 = 0;
    let mut r_end = false;
    for j in 0..P {
        let z = rep[j] == 0;
        if (!r_end) & (!z) {
            rlen += 1;
        }
        r_end = r_end | z;
    }

    // Precompute, for each start index i, whether `pattern` matches at i.
    let mut match_at: [bool; N] = [false; N];
    if (plen > 0) & (plen <= (N as u32)) {
        for i in 0..N {
            if (i as u32) + plen <= (N as u32) {
                let mut here = true;
                for j in 0..M {
                    if (j as u32) < plen {
                        if hay[i + j] != pat[j] {
                            here = false;
                        }
                    }
                }
                match_at[i] = here;
            }
        }
    }

    let mut result: [u8; OUT] = [0; OUT];
    let mut out_len: u32 = 0;
    // `skip` counts how many further input bytes are consumed by an in-progress
    // (already-emitted) replacement, so matches are non-overlapping.
    let mut skip: u32 = 0;

    for i in 0..N {
        let byte = hay[i];
        if skip > 0 {
            skip -= 1;
        } else if (plen > 0) & match_at[i] {
            // Emit replacement, consume `plen` input bytes.
            for k in 0..P {
                if ((k as u32) < rlen) & (out_len < (OUT as u32)) {
                    result[out_len] = rep[k];
                    out_len += 1;
                }
            }
            skip = plen - 1;
        } else if byte != 0 {
            if out_len < (OUT as u32) {
                result[out_len] = byte;
                out_len += 1;
            }
        }
    }

    (result, out_len)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_matches_literal_found() {
    let s: str<11> = "Hello World";
    let p: str<5> = "World";
    assert(matches_literal::<11, 5>(s, p, false));
}

#[test]
fn test_matches_literal_not_found() {
    let s: str<11> = "Hello World";
    let p: str<3> = "xyz";
    assert(!matches_literal::<11, 3>(s, p, false));
}

#[test]
fn test_matches_literal_case_insensitive() {
    let s: str<3> = "abc";
    let p: str<3> = "ABC";
    assert(matches_literal::<3, 3>(s, p, true)); // 'i' flag
    assert(!matches_literal::<3, 3>(s, p, false));
}

#[test]
fn test_matches_literal_empty_pattern() {
    let s: str<3> = "abc";
    let p: str<0> = "";
    assert(matches_literal::<3, 0>(s, p, false));
}

#[test]
fn test_char_class_range() {
    // [A-Z]
    let class: CharClass<0, 1> = CharClass {
        singles: [],
        n_singles: 0,
        range_lo: [65],
        range_hi: [90],
        n_ranges: 1,
        negated: false,
    };
    assert(class.contains(70, false)); // 'F'
    assert(!class.contains(102, false)); // 'f'
    assert(class.contains(102, true)); // 'f' with 'i' flag folds to 'F'
}

#[test]
fn test_char_class_negated() {
    // [^Q]
    let class: CharClass<1, 0> = CharClass {
        singles: [81],
        n_singles: 1,
        range_lo: [],
        range_hi: [],
        n_ranges: 0,
        negated: true,
    };
    assert(!class.contains(81, false)); // 'Q' excluded
    assert(class.contains(82, false)); // 'R' allowed
}

#[test]
fn test_matches_char_class_star() {
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
}

#[test]
fn test_matches_char_class_star_reject() {
    // matches('ab9', '[A-Za-z]*') -> false (digit present)
    let s: str<3> = "ab9";
    let class: CharClass<0, 2> = CharClass {
        singles: [],
        n_singles: 0,
        range_lo: [65, 97],
        range_hi: [90, 122],
        n_ranges: 2,
        negated: false,
    };
    assert(!matches_char_class_star::<3, 0, 2>(s, class, false));
}

#[test]
fn test_matches_anchored_exact() {
    let s: str<5> = "hello";
    let p: str<5> = "hello";
    assert(matches_anchored::<5, 5>(s, p, false));
}

#[test]
fn test_matches_anchored_reject_substring() {
    // ^hel$ must NOT match "hello"
    let s: str<5> = "hello";
    let p: str<3> = "hel";
    assert(!matches_anchored::<5, 3>(s, p, false));
}

#[test]
fn test_matches_prefix() {
    let s: str<11> = "Hello World";
    let p: str<5> = "Hello";
    assert(matches_prefix::<11, 5>(s, p, false));
    let q: str<5> = "World";
    assert(!matches_prefix::<11, 5>(s, q, false));
}

#[test]
fn test_replace_literal_basic() {
    // replace("abracadabra", "bra", "*") -> "a*cada*"
    let s: str<11> = "abracadabra";
    let p: str<3> = "bra";
    let r: str<1> = "*";
    let (out, len) = replace_literal::<11, 3, 1, 11>(s, p, r);
    assert(len == 7);
    let expected = "a*cada*".as_bytes();
    for i in 0..7 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_replace_literal_grow() {
    // replace("aaa", "a", "xy") -> "xyxyxy"
    let s: str<3> = "aaa";
    let p: str<1> = "a";
    let r: str<2> = "xy";
    let (out, len) = replace_literal::<3, 1, 2, 6>(s, p, r);
    assert(len == 6);
    let expected = "xyxyxy".as_bytes();
    for i in 0..6 {
        assert(out[i] == expected[i]);
    }
}

#[test]
fn test_replace_literal_no_match() {
    let s: str<5> = "hello";
    let p: str<2> = "zz";
    let r: str<1> = "Q";
    let (out, len) = replace_literal::<5, 2, 1, 5>(s, p, r);
    assert(len == 5);
    let expected = "hello".as_bytes();
    for i in 0..5 {
        assert(out[i] == expected[i]);
    }
}
