//! **Seeded deterministic query/data generator** — seed in, test case out, bit-for-bit
//! reproducible. [FABLE-5] sq-gum8.6
//!
//! No wall-clock, OS randomness, or external RNG crate is involved: the generator is
//! driven by an in-crate [`SplitMix64`] (Steele, Lea & Flood's fixed 64-bit mixing
//! constants), so a ledger entry's `seed` reproduces its exact test case on any
//! machine and any future version of the `rand` ecosystem.
//!
//! One [`GeneratedCase`] is a triple of N-Triples data, a group-graph-pattern body,
//! and a filter predicate, shaped to exercise all three EBV outcomes (see
//! [`crate::tlp`]):
//!
//! * the data mixes integers, decimals, doubles, strings, language-tagged strings,
//!   booleans, and IRIs under one predicate, so comparisons and arithmetic hit both
//!   comparable values (true/false branches) and **type errors** (error branch);
//! * the pattern carries an `OPTIONAL` clause whose object is present on only some
//!   subjects, so predicates over the optional variable hit **unbound-variable
//!   errors**; and
//! * the predicate grammar spans comparisons, arithmetic, `BOUND`/`COALESCE`
//!   (unbound-tolerant forms), the term-inspection functions `STR`/`DATATYPE`/`isIRI`,
//!   and the error-absorbing connectives `&&`/`||`/`!`.
//!
//! The grammar deliberately excludes everything that voids the metamorphic relations
//! (TLP scope preconditions): no `RAND`/`NOW`/`UUID`/`STRUUID`/`BNODE`
//! (nondeterminism), no `EXISTS`/`NOT EXISTS` (spec ambiguity), and no blank nodes in
//! the data (cross-engine label comparison is meaningless — see [`crate::differential`]).
//!
//! Grammar note: the term-inspection atoms are exactly the ones `predicate_atom`
//! emits — `STR`, `DATATYPE`, and `isIRI`. The generator already produces
//! language-tagged literals (`"…"@fr`) as EBV/type-error fuel, but no atom yet
//! inspects them via `LANG` or `isLiteral`; any new function must be added in
//! `predicate_atom` so this list, the TLP scope preconditions, and the exclusion
//! list above stay in lockstep. (The matching §6.1 paper grammar overclaim is
//! tracked for the `sq-gum8.7` revision pass.) [OPUS-4.8] sq-b89wc

/// Minimal deterministic PRNG: SplitMix64. Chosen over a `rand` dependency because its
/// output for a given seed is a *fixed function* specified by three constants — the
/// reproducibility contract survives dependency upgrades.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Create a generator from a seed. Equal seeds yield equal streams, forever.
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// Next 64 uniformly mixed bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform-ish value in `0..n` (`n > 0`; simple modulo — the tiny bias is
    /// irrelevant for test-case generation and keeps the function trivially portable).
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        self.next_u64() % n
    }
}

/// One reproducible test case: data + pattern + predicate (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCase {
    /// The seed that produced this case (carried for the found-bug ledger).
    pub seed: u64,
    /// N-Triples document to load into every engine under test.
    pub data_ntriples: String,
    /// Group-graph-pattern body (`?s <…/v> ?v . OPTIONAL { ?s <…/w> ?w }`).
    pub pattern: String,
    /// Deterministic filter expression over `?v` / `?w`.
    pub predicate: String,
}

const EX: &str = "http://example.org";
const XSD: &str = "http://www.w3.org/2001/XMLSchema";

/// Render one deterministic object term for subject index `i`.
fn object_term(rng: &mut SplitMix64, i: u64) -> String {
    match rng.below(7) {
        0 => format!("\"{}\"^^<{XSD}#integer>", rng.below(20)),
        1 => format!("\"{}.{}\"^^<{XSD}#decimal>", rng.below(10), rng.below(10)),
        2 => format!("\"{}.5E0\"^^<{XSD}#double>", rng.below(10)),
        3 => format!("\"str{i}\""),
        4 => format!("\"mot{i}\"@fr"),
        5 => format!("\"{}\"^^<{XSD}#boolean>", if rng.below(2) == 0 { "true" } else { "false" }),
        _ => format!("<{EX}/o{i}>"),
    }
}

/// Render one deterministic predicate (filter-expression) atom over `?v` / `?w`.
fn predicate_atom(rng: &mut SplitMix64) -> String {
    let k = rng.below(10);
    let m = rng.below(20);
    match rng.below(10) {
        0 => format!("?v < {m}"),
        1 => format!("?v = {m}"),
        2 => format!("?v >= {k}"),
        3 => format!("?w < {k}"),
        4 => "BOUND(?w)".to_string(),
        5 => "isIRI(?v)".to_string(),
        6 => format!("DATATYPE(?v) = <{XSD}#integer>"),
        7 => format!("STR(?v) = \"str{k}\""),
        8 => format!("?v + {k} < {m}"),
        _ => format!("COALESCE(?w, {k}) < {m}"),
    }
}

/// Render one deterministic predicate expression of bounded depth.
fn predicate_expr(rng: &mut SplitMix64, depth: u32) -> String {
    if depth == 0 {
        return predicate_atom(rng);
    }
    match rng.below(4) {
        0 => predicate_atom(rng),
        1 => format!(
            "( {} && {} )",
            predicate_expr(rng, depth - 1),
            predicate_expr(rng, depth - 1)
        ),
        2 => format!(
            "( {} || {} )",
            predicate_expr(rng, depth - 1),
            predicate_expr(rng, depth - 1)
        ),
        _ => format!("! ( {} )", predicate_expr(rng, depth - 1)),
    }
}

/// Generate the test case for `seed`. Pure function of the seed: calling it twice with
/// the same seed yields identical output (asserted by a self-test).
pub fn generate_case(seed: u64) -> GeneratedCase {
    let mut rng = SplitMix64::new(seed);
    let n_subjects = 4 + rng.below(9);
    let mut data = String::new();
    for i in 0..n_subjects {
        let object = object_term(&mut rng, i);
        data.push_str(&format!("<{EX}/s{i}> <{EX}/v> {object} .\n"));
        // The optional predicate is present on roughly half the subjects, so ?w is
        // genuinely unbound on the rest (the error-branch fuel).
        if rng.below(2) == 0 {
            let w = rng.below(10);
            data.push_str(&format!(
                "<{EX}/s{i}> <{EX}/w> \"{w}\"^^<{XSD}#integer> .\n"
            ));
        }
    }
    let pattern = format!("?s <{EX}/v> ?v . OPTIONAL {{ ?s <{EX}/w> ?w }}");
    let predicate = predicate_expr(&mut rng, 2);
    GeneratedCase {
        seed,
        data_ntriples: data,
        pattern,
        predicate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_is_the_reference_function() {
        // The first output for seed 0, pinned against the published SplitMix64
        // reference so any accidental constant change is caught.
        let mut rng = SplitMix64::new(0);
        let first = rng.next_u64();
        let mut rng2 = SplitMix64::new(0);
        assert_eq!(first, rng2.next_u64(), "same seed, same stream");
        assert_ne!(SplitMix64::new(1).next_u64(), first, "different seed, different stream");
        // Fixed-point pin: seed 0 first output of SplitMix64 with the standard constants.
        assert_eq!(first, 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = SplitMix64::new(42);
        for _ in 0..1000 {
            assert!(rng.below(7) < 7);
        }
    }

    #[test]
    fn generate_case_is_deterministic_in_the_seed() {
        let a = generate_case(99);
        let b = generate_case(99);
        assert_eq!(a, b, "same seed must reproduce the identical case");
        let c = generate_case(100);
        assert_ne!(a, c, "a different seed produces a different case");
        assert_eq!(a.seed, 99);
        assert!(a.data_ntriples.contains("<http://example.org/s0>"));
        assert!(a.pattern.contains("OPTIONAL"));
        assert!(!a.predicate.is_empty());
    }

    #[test]
    fn generated_predicates_respect_the_exclusion_list() {
        for seed in 0..200 {
            let case = generate_case(seed);
            for banned in ["RAND", "NOW", "UUID", "STRUUID", "BNODE", "EXISTS"] {
                assert!(
                    !case.predicate.contains(banned),
                    "seed {seed}: predicate contains banned construct {banned}: {}",
                    case.predicate
                );
            }
            assert!(!case.data_ntriples.contains("_:"), "no blank nodes in data");
        }
    }
}
