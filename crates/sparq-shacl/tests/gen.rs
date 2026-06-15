//! [OPUS-4.8] sq-55c1 — seeded generator of VALID SHACL shapes + data graphs for
//! the differential fuzzer (`diff_fuzz.rs`).
//!
//! Deterministic SplitMix64 (the same RNG `sparq-bench/src/fuzz.rs` uses), so any
//! case reproduces from its seed alone. A `Scenario` is one node shape with a
//! `sh:targetClass`, a handful of property shapes (each carrying one or more
//! constraint components from the in-scope set below), plus a small data graph of
//! instances of that class — each instance independently nudged toward conforming
//! or violating one of the property shapes.
//!
//! ## In-scope constraint components (the comparable subset — what is GENERATED)
//! These are generated and their reports compared against the reference engine:
//!   sh:minCount, sh:maxCount (cardinality, on every property shape) and one value
//!   component per property shape drawn from: sh:datatype, sh:nodeKind, sh:class,
//!   sh:pattern, sh:in, sh:minLength, sh:maxLength, sh:hasValue, sh:minInclusive,
//!   sh:maxInclusive, sh:minExclusive, sh:maxExclusive.
//!
//! ## Deliberately OUT of scope (documented; tracked as TODO beads where useful)
//!   - the logical components sh:and / sh:or / sh:not / sh:xone and sh:node
//!     (nested-shape reference) — sparq supports them, but generating VALID nested
//!     shapes + the matching reference-report normalisation is a follow-up bead.
//!   - sh:closed, sh:qualifiedValueShape, sh:disjoint/equals/lessThan*,
//!     sh:uniqueLang, sh:languageIn — future generator extensions (their report
//!     shapes / pySHACL message-paths need extra normalisation).
//!   - sh:sparql / SPARQL constraint COMPONENTS — pySHACL and sparq both support
//!     them but their report focus/value semantics differ enough to need bespoke
//!     comparison; left for a follow-up bead.
//!   - complex property paths (sequence / inverse / */+) as the SHAPE path — the
//!     generator uses simple predicate paths so the (focus, component, path) key
//!     is byte-comparable; a complex-path generation mode is a follow-up bead.
//!   - sh:severity other than the default sh:Violation — the differential compares
//!     the conformance bit + violated-component set, which is severity-agnostic by
//!     construction here (everything is a Violation).

#![allow(dead_code)] // shared test module: not every helper is used by every test

/// Deterministic SplitMix64 — no clock/entropy, so every case reproduces from
/// its seed. Same construction as `sparq-bench/src/fuzz.rs`.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(1))
    }
    // Named `next_u64` (not `next`) so it cannot be confused with
    // `Iterator::next` (clippy::should_implement_trait).
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

const EX: &str = "http://example.org/";

/// One in-scope constraint component on a property shape, with the Turtle it
/// contributes to the shapes graph and a generator for object values that
/// conform vs violate it.
#[derive(Clone, Debug)]
enum Constraint {
    Datatype(&'static str), // xsd local name (integer/string/boolean/decimal)
    NodeKind(&'static str), // sh:IRI / sh:Literal / sh:BlankNode
    Class(String),          // sh:class <iri>
    Pattern(&'static str),  // sh:pattern (a regex string)
    In(Vec<String>),        // sh:in ( ... ) — object terms (Turtle)
    MinLength(u64),
    MaxLength(u64),
    HasValue(String), // sh:hasValue <term> (object Turtle)
    MinInclusive(i64),
    MaxInclusive(i64),
    MinExclusive(i64),
    MaxExclusive(i64),
}

impl Constraint {
    /// The Turtle predicate-object pairs this component adds to a property shape
    /// (joined into the shape body by the caller).
    fn shape_lines(&self) -> Vec<String> {
        match self {
            Constraint::Datatype(dt) => vec![format!("sh:datatype xsd:{dt}")],
            Constraint::NodeKind(nk) => vec![format!("sh:nodeKind {nk}")],
            Constraint::Class(c) => vec![format!("sh:class <{c}>")],
            Constraint::Pattern(p) => vec![format!("sh:pattern \"{p}\"")],
            Constraint::In(vals) => vec![format!("sh:in ( {} )", vals.join(" "))],
            Constraint::MinLength(n) => vec![format!("sh:minLength {n}")],
            Constraint::MaxLength(n) => vec![format!("sh:maxLength {n}")],
            Constraint::HasValue(v) => vec![format!("sh:hasValue {v}")],
            Constraint::MinInclusive(n) => vec![format!("sh:minInclusive {n}")],
            Constraint::MaxInclusive(n) => vec![format!("sh:maxInclusive {n}")],
            Constraint::MinExclusive(n) => vec![format!("sh:minExclusive {n}")],
            Constraint::MaxExclusive(n) => vec![format!("sh:maxExclusive {n}")],
        }
    }

    /// An object term (Turtle) that SATISFIES this component.
    fn conforming_value(&self, rng: &mut Rng) -> String {
        match self {
            // [OPUS-4.8] ~1-in-3 of the time draw a boundary-but-still-valid
            // literal (0, negatives, empty string, …) to widen coverage; these
            // are still lexically valid for the datatype, so they keep CONFORMING
            // to a bare `sh:datatype` constraint while exercising edge values.
            Constraint::Datatype(dt) => {
                let valid = !rng.chance(1, 3);
                typed_literal(dt, rng, valid)
            }
            Constraint::NodeKind(nk) => match *nk {
                "sh:IRI" => format!("<{EX}thing{}>", rng.below(5)),
                "sh:Literal" => format!("\"lit{}\"", rng.below(5)),
                "sh:BlankNode" => "[ ]".to_string(),
                _ => "\"x\"".to_string(),
            },
            // sh:class — an instance typed as the class.
            Constraint::Class(_) => "<__CLASS_INSTANCE__>".to_string(),
            Constraint::Pattern(_) => "\"abc123\"".to_string(), // matches the patterns we emit
            Constraint::In(vals) => vals[rng.below(vals.len() as u64) as usize].clone(),
            Constraint::MinLength(n) => format!("\"{}\"", "x".repeat((*n as usize).max(1) + 1)),
            Constraint::MaxLength(n) => format!("\"{}\"", "x".repeat((*n as usize).min(2))),
            Constraint::HasValue(v) => v.clone(),
            Constraint::MinInclusive(n) => format!("{}", n + rng.below(3) as i64),
            Constraint::MaxInclusive(n) => format!("{}", n - rng.below(3) as i64),
            Constraint::MinExclusive(n) => format!("{}", n + 1 + rng.below(3) as i64),
            Constraint::MaxExclusive(n) => format!("{}", n - 1 - rng.below(3) as i64),
        }
    }

    /// An object term (Turtle) that VIOLATES this component.
    fn violating_value(&self, rng: &mut Rng) -> String {
        match self {
            // wrong datatype: a string where an integer is wanted, and vice-versa.
            Constraint::Datatype(dt) => match *dt {
                "integer" | "decimal" => "\"not-a-number\"".to_string(),
                "boolean" => "\"maybe\"".to_string(),
                _ => format!("{}", rng.below(100)), // bare integer where a string wanted
            },
            Constraint::NodeKind(nk) => match *nk {
                // wants an IRI -> give a literal; wants a literal -> give an IRI;
                // wants a blank node -> give an IRI.
                "sh:IRI" => "\"a-literal\"".to_string(),
                "sh:Literal" => format!("<{EX}an-iri>"),
                "sh:BlankNode" => format!("<{EX}an-iri>"),
                _ => "\"x\"".to_string(),
            },
            // sh:class — an instance NOT typed as the class.
            Constraint::Class(_) => format!("<{EX}untyped{}>", rng.below(5)),
            Constraint::Pattern(_) => "\"ZZZ_no_match\"".to_string(),
            Constraint::In(_) => "\"definitely-not-in-the-list\"".to_string(),
            Constraint::MinLength(n) => {
                format!("\"{}\"", "x".repeat((*n as usize).saturating_sub(1)))
            }
            Constraint::MaxLength(n) => format!("\"{}\"", "x".repeat(*n as usize + 3)),
            Constraint::HasValue(_) => format!("\"other-value-{}\"", rng.below(5)),
            Constraint::MinInclusive(n) => format!("{}", n - 1 - rng.below(3) as i64),
            Constraint::MaxInclusive(n) => format!("{}", n + 1 + rng.below(3) as i64),
            Constraint::MinExclusive(n) => format!("{}", n - rng.below(2) as i64), // n itself violates min-exclusive
            Constraint::MaxExclusive(n) => format!("{}", n + rng.below(2) as i64),
        }
    }

    /// Does this constraint need a typed class instance in the data graph?
    fn class_iri(&self) -> Option<&str> {
        match self {
            Constraint::Class(c) => Some(c),
            _ => None,
        }
    }
}

/// A typed literal of the given xsd local name. Both `valid` modes produce a
/// value that is lexically valid for the datatype (so it still conforms to a
/// bare `sh:datatype` constraint — wrong-datatype violations are minted by
/// `violating_value`). `valid=false` instead picks boundary values (zero,
/// negatives, the empty string, …) to widen the fuzz coverage. [OPUS-4.8]
fn typed_literal(dt: &str, rng: &mut Rng, valid: bool) -> String {
    match dt {
        "integer" if !valid => {
            // boundary integers: 0, and small negatives.
            format!("{}", -(rng.below(3) as i64))
        }
        "integer" => format!("{}", rng.below(100)),
        "decimal" if !valid => "\"-0.0\"^^xsd:decimal".to_string(),
        "decimal" => format!("\"{}.5\"^^xsd:decimal", rng.below(100)),
        "boolean" if !valid => {
            // the lexical alternatives "0"/"1" for xsd:boolean.
            if rng.chance(1, 2) {
                "\"1\"^^xsd:boolean"
            } else {
                "\"0\"^^xsd:boolean"
            }
            .to_string()
        }
        "boolean" => if rng.chance(1, 2) { "true" } else { "false" }.to_string(),
        "string" if !valid => "\"\"".to_string(), // empty string is a valid xsd:string
        "string" => format!("\"s{}\"", rng.below(10)),
        _ if !valid => "\"\"".to_string(),
        _ => format!("\"v{}\"", rng.below(10)),
    }
}

/// One property shape: a path predicate, a min/maxCount pair, and a value
/// constraint. The generator decides per-instance whether to satisfy or violate.
struct PropShape {
    path_pred: String,
    min_count: Option<u64>,
    max_count: Option<u64>,
    value: Option<Constraint>,
}

impl PropShape {
    fn shape_turtle(&self) -> String {
        let mut lines = vec![format!("sh:path <{}>", self.path_pred)];
        if let Some(n) = self.min_count {
            lines.push(format!("sh:minCount {n}"));
        }
        if let Some(n) = self.max_count {
            lines.push(format!("sh:maxCount {n}"));
        }
        if let Some(c) = &self.value {
            lines.extend(c.shape_lines());
        }
        format!(
            "    sh:property [\n      {} ;\n    ]",
            lines.join(" ;\n      ")
        )
    }
}

/// A full generated scenario: a node shape, its property shapes, and a data graph.
pub struct Scenario {
    target_class: String,
    props: Vec<PropShape>,
    instances: Vec<Instance>,
    /// Supporting type declarations the data graph needs (typed class instances
    /// for sh:class conforming values, etc.).
    support: Vec<String>,
}

/// One data-graph instance of the target class, with the property objects to emit.
struct Instance {
    subject: String,
    /// (predicate, object-turtle) pairs.
    objects: Vec<(String, String)>,
}

impl Scenario {
    pub fn generate(rng: &mut Rng) -> Scenario {
        let class_n = rng.below(1000);
        let target_class = format!("{EX}C{class_n}");
        let n_props = 1 + rng.below(4); // 1..=4 property shapes

        let mut props = Vec::new();
        let mut support = Vec::new();
        let mut used_preds = std::collections::HashSet::new();
        for i in 0..n_props {
            // distinct predicate per property shape so cardinality is unambiguous.
            let mut pred = format!("{EX}p{i}");
            while !used_preds.insert(pred.clone()) {
                pred = format!("{EX}p{}_{}", i, rng.below(1000));
            }
            let value = gen_constraint(rng, &mut support);
            // counts: keep min<=max and small so violations are reachable.
            let min_count = if rng.chance(2, 3) {
                Some(rng.below(2))
            } else {
                None
            };
            let max_count = if rng.chance(1, 2) {
                Some(1 + rng.below(2))
            } else {
                None
            };
            props.push(PropShape {
                path_pred: pred,
                min_count,
                max_count,
                value,
            });
        }

        // Generate instances: each is a fresh subject of the target class, with
        // each property independently satisfied or violated.
        let n_inst = 1 + rng.below(5); // 1..=5 instances
        let mut instances = Vec::new();
        for i in 0..n_inst {
            let subject = format!("{EX}i{}_{}", class_n, i);
            let mut objects = Vec::new();
            for p in &props {
                // Decide how many values to emit for this property (0..=3), then
                // make each conform or violate the value constraint.
                let k = rng.below(4); // 0,1,2,3 values
                for _ in 0..k {
                    let obj = match &p.value {
                        Some(c) => {
                            let v = if rng.chance(1, 2) {
                                c.conforming_value(rng)
                            } else {
                                c.violating_value(rng)
                            };
                            // Resolve the class-instance placeholder into a real
                            // typed instance recorded in `support`.
                            resolve_value(v, c, &mut support, rng)
                        }
                        None => format!("\"v{}\"", rng.below(10)),
                    };
                    objects.push((p.path_pred.clone(), obj));
                }
            }
            instances.push(Instance { subject, objects });
        }

        Scenario {
            target_class,
            props,
            instances,
            support,
        }
    }

    pub fn shapes_turtle(&self) -> String {
        let mut s = String::from(PREFIXES);
        s.push_str(&format!("<{EX}Shape> a sh:NodeShape ;\n"));
        s.push_str(&format!("    sh:targetClass <{}> ;\n", self.target_class));
        let bodies: Vec<String> = self.props.iter().map(|p| p.shape_turtle()).collect();
        s.push_str(&bodies.join(" ;\n"));
        s.push_str(" .\n");
        s
    }

    pub fn data_turtle(&self) -> String {
        let mut s = String::from(PREFIXES);
        for inst in &self.instances {
            s.push_str(&format!("<{}> a <{}> .\n", inst.subject, self.target_class));
            for (pred, obj) in &inst.objects {
                s.push_str(&format!("<{}> <{}> {} .\n", inst.subject, pred, obj));
            }
        }
        // Supporting declarations (typed class instances for sh:class, etc.).
        for line in &self.support {
            s.push_str(line);
            s.push('\n');
        }
        s
    }
}

/// Picks a random in-scope value constraint.
///
/// [OPUS-4.8] `_support` is intentionally unused: every constraint we currently
/// generate fills its data-graph support per-value later in `resolve_value`
/// (e.g. `sh:class` mints its typed instance there), so nothing is accumulated
/// up front. The parameter is kept so callers needn't change if a future
/// constraint does need up-front support declarations.
fn gen_constraint(rng: &mut Rng, _support: &mut Vec<String>) -> Option<Constraint> {
    // 1-in-6: a count-only property shape (no value constraint).
    if rng.chance(1, 6) {
        return None;
    }
    let choice = rng.below(12);
    Some(match choice {
        0 => Constraint::Datatype(rng.pick(&["integer", "string", "boolean", "decimal"])),
        1 => Constraint::NodeKind(rng.pick(&["sh:IRI", "sh:Literal", "sh:BlankNode"])),
        2 => {
            // sh:class: conforming values get a typed instance minted lazily by
            // `resolve_value` (so each conforming value points at its own typed
            // node); no class-level support declaration is needed up front.
            let cls = format!("{EX}Kind{}", rng.below(100));
            Constraint::Class(cls)
        }
        3 => Constraint::Pattern(rng.pick(&["^[a-z]+[0-9]+$", "^a.*$", "[0-9]"])),
        4 => {
            let n = 2 + rng.below(3);
            let vals: Vec<String> = (0..n).map(|j| format!("\"opt{j}\"")).collect();
            Constraint::In(vals)
        }
        5 => Constraint::MinLength(1 + rng.below(4)),
        6 => Constraint::MaxLength(2 + rng.below(4)),
        7 => Constraint::HasValue(format!("\"required-{}\"", rng.below(5))),
        8 => Constraint::MinInclusive(rng.below(50) as i64),
        9 => Constraint::MaxInclusive(50 + rng.below(50) as i64),
        10 => Constraint::MinExclusive(rng.below(50) as i64),
        _ => Constraint::MaxExclusive(50 + rng.below(50) as i64),
    })
}

/// Resolves the `<__CLASS_INSTANCE__>` placeholder produced by a conforming
/// sh:class value into a real, typed instance recorded in `support`. All other
/// values pass through unchanged.
fn resolve_value(v: String, c: &Constraint, support: &mut Vec<String>, rng: &mut Rng) -> String {
    if v == "<__CLASS_INSTANCE__>" {
        if let Some(cls) = c.class_iri() {
            let inst = format!("{EX}inst_{}", rng.below(100000));
            support.push(format!("<{inst}> a <{cls}> ."));
            return format!("<{inst}>");
        }
    }
    v
}

const PREFIXES: &str = "\
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
";
