# sparq-substrate

**Shared zero-overhead evaluation substrate** for the sparq SPARQL engine and the
reasoners — an **opt-in**, **leaf** crate (epic sq-qonbz) that depends **only** on
`sparq-core`, never on `sparq-engine`. It hosts the parts of evaluation that are genuinely
common to both the query engine and every reasoner: the id-tuple **row/key** vocabulary, the
XSD **numeric value tower**, the four **join kernels**, and the SPARQL **term total order**.
Placing them in a leaf crate lets both consumers reach them with **no dependency cycle**, while
keeping `sparq-core` and the lean wasm bundle untouched.

> **Status (sq-vezew — Phase 4 of the epic).** The SPARQL **term total order** —
> `compare::compare_terms` (the engine's `compare_values`: error/unbound < blank < IRI <
> literal < triple, numeric-aware + strict typed/temporal + string fallback + recursive
> triple-term order) — has now **moved here** from `sparq-engine`, generalised over a tiny
> `CompareTerm` trait the consumer implements for its term type, so the engine AND a future
> reasoner share one ordering body **monomorphically** — no `Box<dyn>`. Phase 3 (sq-hknqs)
> moved the four **join kernels** (behind a generic `JoinKeys` descriptor + a `Budget` hook)
> and Phase 2 (sq-ev41x) the **numeric value tower** the same way. All are behaviour-neutral
> code-moves (the W3C SPARQL conformance floor is bit-identical; the join/scan/BGP/sort
> micro-benches are within noise). The engine's `Value` enum stays engine-resident — it also
> drives the relational operators — and is surfaced to the ordering algorithm through the
> trait. See `research/shared-eval-substrate.md` for the extraction plan and perf-neutrality proof.

## 🚀 Quickstart

Everything is behind **default-off** features — opt into exactly the slice you need:

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["rows", "numeric", "join", "compare"] }
```

```rust,ignore
// `rows`: the shared id-tuple vocabulary (the engine + reasoners agree on it).
use sparq_substrate::rows::{Id, Row, Key, Posting, inline_id_of_int, is_inline};

let mut row: Row = Row::new();          // SmallVec<[Id; 4]> — inline up to 4 columns
row.extend_from_slice(&[1, 2, 3]);
let int_id: Option<Id> = inline_id_of_int(42); // inline-integer id, no term construction

// `numeric`: the XSD numeric value tower for value-space reasoning / arithmetic.
use sparq_substrate::numeric::{Num, as_numeric};
let lit = oxrdf::Literal::new_typed_literal("0.1", oxrdf::vocab::xsd::DECIMAL);
let n: Option<Num> = as_numeric(&lit);  // exact xsd:decimal (no f64 rounding)
```

## ✨ Features

- **`rows`** — the `SmallVec`-based `Row` (`[Id; 4]`), `Key` (`[Id; 2]`) and `Posting`
  (`[usize; 2]`) aliases over the `sparq-core` dictionary `Id`, plus the re-exported
  inline-integer id helpers (`inline_id_of_int`, `is_inline`, `NO_ID`). These mirror the
  engine's private `exec` aliases byte for byte so the eventual join-kernel move is a pure
  code-move. The aliases are concrete monomorphic types, **not** generic over a `dyn` trait.
- **`numeric`** — the XSD numeric value tower: `Num` (`Int` / `Dec` / `Float` / `Double`,
  the XPath promotion ranks), `as_numeric` (classifies an `oxrdf::Literal` into the tower
  while keeping the **exact** integer / fixed-point `Dec` representation — a high-precision
  decimal is not silently flattened to `f64`), the arithmetic ops (`binop` / `neg` / `abs` /
  `ceil` / `floor` / `round`), the two serialisation surfaces — `canonical_lexical` (a
  *finite* float/double in the XSD-mandatory scientific form, e.g. `6.0E0`; the specials keep
  their XSD spellings `INF` / `-INF` / `NaN`, which are not scientific) and `lexical` (the plain
  form the W3C SPARQL expected-result files use for computed arithmetic, e.g. `6`) — and the
  shared lexical helpers (`split_decimal`, `parse_xsd_f32` / `parse_xsd_f64`, `fmt_xsd_double`).
  `parse_xsd_f64`/`f32` accept the XSD `INF` / `+INF` / `-INF` / `NaN` spellings and reject the
  Rust-`FromStr`-only `inf` / `infinity` / `nan`. `Num::cmp_relational` is the XPath relational
  comparison (`<`/`>` FILTER, value-space equality) — partial (NaN → `None`), vs `cmp_total`
  which totalises NaN for `ORDER BY` [OPUS-4.8] sq-v5evr.
- **`join`** — the four id-tuple join kernels over `&[Row]` slices: `merge_join` (sorted),
  `build_table` / `build_partitioned` / `probe_emit` / `probe_gather_indices` / `hash_probe_serial`
  (hash, `JoinTable` type alias backed by `hashbrown::HashMap<Key, Posting, FxBuildHasher>`,
  single-hash probe + batch-reserve), `bind_combine` (index-nested-loop), `lftj_recurse` over
  `Trie` / `TrieIter` (WCOJ), plus `compatible` / `merge_rows` / `any_unbound` helpers.
  `probe_gather_indices` is the M4 batch-emission contract: gather build indices, materialise
  once per output chunk (sq-pntvh.7). Also `join::delta::DeltaTable` — persistent build-side
  table with insertion-order-deterministic enumeration for the OWL-RL Δ⋈full fixpoint
  (sq-qonbz.1). Each kernel is generic over `JoinKeys` + `Budget` (monomorphised, no vtable).
  Pulls `rustc-hash` + `hashbrown` when enabled; implies `rows`. [SONNET-4.6] sq-7d3dj.19
- **`compare`** — the SPARQL **term total order** `compare_terms` (the engine's `compare_values`):
  the spec-fixed class precedence error/unbound < blank < IRI < literal < triple, then a
  **kind-first** literal order (sq-wjl8i): a fixed `LiteralKind` rank BETWEEN literal kinds
  (numeric < boolean < dateTime < date < string < lang < other — a documented extension where
  the spec leaves cross-kind order undefined), value order only WITHIN a kind — numerics by
  exact rational value (`NaN` totalised FIRST, before `-INF`; an f64 tie rechecked exactly via
  `exact_cmp`, incl. the MIXED int/decimal-vs-float/double tie — `Num::cmp_total` under
  `numeric`), dateTimes by timeline, else lexically — and the recursive component-wise
  triple-term order. Generic over a tiny `CompareTerm` trait (`term_class` / `literal_kind` /
  `value_str` / `as_f64` / `exact_cmp` / `strict_cmp` / `triple_parts`) the consumer implements
  for its own term type — a **monomorphisation seam**, never a `dyn` object. Pure-`std`. The
  order laws — reflexivity, within-class totality, antisymmetry-consistency and transitivity
  (per kind AND across mixed kinds, all NaN INCLUDED, incl. the 2^53 collapse) — are
  machine-checked by Kani bounded-proof harnesses over a model `CompareTerm` impl —
  `cargo kani -p sparq-substrate --features compare` (sq-sqtk2.4 + sq-wjl8i; the three former
  intransitivity witnesses are pinned FIXED). Honest boundaries: the proofs cover the shared
  ALGORITHM over the bounded model (per-harness domains in the module), not the engine's
  `Value` impl; and within the dateTime kind the indeterminate mixed-timezone window still
  falls back lexically (a residual partial-order seam, tracked separately).

All features are **off by default**. The crate is `forbid(unsafe_code)`.

### Zero-overhead intent

Every item is monomorphic over `Id = u32` and the concrete numeric tiers — **never**
`Box<dyn>` / `&dyn` / a vtable on a hot path — including between a join's probe loop and its
key projection or its cooperative-cancel poll (the `JoinKeys` / `Budget` parameters are
monomorphised, not trait objects). Each item carries `#[inline]`, so cross-crate inlining
(with the workspace LTO profile) keeps the engine's FILTER / BIND / ORDER BY arithmetic and
its join hot loops identical to pre-move codegen. This crate introduces no dynamic dispatch.

## 📚 Learn more

- `research/shared-eval-substrate.md` — the design record: what is shareable vs
  engine-private, the options considered, and the layered perf-neutrality proof.
- `research/mechanized-proof-program.md` — the proof program the `compare` Kani harnesses
  belong to (property B-1), incl. the claim-tier vocabulary and the not-tractable-now ledger.
- `crates/sparq-core` — the storage substrate this crate's `Id` / dictionary types come from.
- `crates/sparq-engine` — the consumer that keeps its planner private and calls the shared
  numeric + join kernels through a thin `Bindings`-side adapter.

## License

MIT — see the workspace [LICENSE](../../LICENSE).
