---
name: substrate
description: The opt-in shared zero-overhead evaluation substrate for the sparq engine and the reasoners — id-tuple row/key/posting vocabulary (`rows`), the XSD numeric value tower (`numeric`), the four id-tuple join kernels (`join`), and the SPARQL term total order (`compare`). All features are DEFAULT-OFF; opt in to exactly the slice you need. Use this crate when wiring a new reasoner that shares joins or numeric evaluation with the engine, or when working on the sparq-substrate crate itself. [SONNET-4.6] sq-qonbz.4
---

# sparq-substrate

`sparq-substrate` is the leaf crate (below `sparq-engine`, zero-overhead) that hosts the
id-tuple evaluation primitives shared between the SPARQL engine and every reasoner. It depends
ONLY on `sparq-core` — no cycle with `sparq-engine` — so both consumers can share one join
kernel / numeric value tower with no dependency problem.

All four modules are gated behind **DEFAULT-OFF** cargo features; enabling one never forces a
heavy dependency on the engine's or the wasm bundle's build graph.

## Four public modules

### `rows` — `#[cfg(feature = "rows")]`

The shared id-tuple vocabulary: `SmallVec`-based `Row` (`[Id; 4]`), `Key` (`[Id; 2]`) and
`Posting` (`[usize; 2]`) aliases over the `sparq-core` dictionary `Id = u32`, plus the
re-exported inline-integer id helpers `inline_id_of_int`, `is_inline`, and `NO_ID`.

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["rows"] }
```

```rust,ignore
use sparq_substrate::rows::{Id, Row, Key, Posting, inline_id_of_int, is_inline, NO_ID};

let mut row: Row = Row::new();       // SmallVec<[Id; 4]> — inline up to 4 columns, no alloc
row.extend_from_slice(&[1, 2, 3]);
let int_id: Option<Id> = inline_id_of_int(42);
```

### `numeric` — `#[cfg(feature = "numeric")]`

The XSD numeric value tower: `Num` (`Int(i64)` / `Dec(Dec)` / `Float(f32)` / `Double(f64)`),
the fixed-point `Dec` struct (EXACT integer/decimal arithmetic, no f64 rounding), `ArithOp`
(`Add`/`Sub`/`Mul`/`Div`), `RoundMode`, `as_numeric` (classifies an `oxrdf::Literal` into the
tower), and the XSD lexical helpers `split_decimal`, `parse_xsd_f64`, `parse_xsd_f32`,
`fmt_xsd_double`. `parse_xsd_f64` delegates to `sparq_core::parse_xsd_f64` (sq-9781x) — one
shared body with the `sparq-core` numeric-value cache, so cache-hit ⟺ evaluator-accepts.
Pulls `oxrdf` only when enabled. Two ordering methods on `Num`:
- `Num::cmp_total` — ORDER BY / MIN/MAX total order; NaN totalised FIRST.
- `Num::cmp_relational` — SPARQL `<`/`>` / D-entailment / RIF numeric equality; NaN → `None`
  (type error). [OPUS-4.8] sq-v5evr.

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["numeric"] }
oxrdf = { version = "0.3", features = ["rdf-12"] }
```

```rust,ignore
use sparq_substrate::numeric::{Num, Dec, ArithOp, as_numeric};

// Integer arithmetic — exact
let a = Num::Int(3);
let b = Num::Int(4);
let sum = a.binop(b, ArithOp::Add); // Some(Num::Int(7))

// Decimal arithmetic — exact fixed-point, no f64 rounding
let x = Num::Dec(Dec { mant: 1, scale: 1 }); // 0.1
let y = Num::Dec(Dec { mant: 2, scale: 1 }); // 0.2
let xy = x.binop(y, ArithOp::Add);           // Some(Num::Dec(...)) == 0.3 exact

// Classify a literal
let lit = oxrdf::Literal::new_typed_literal("42", oxrdf::vocab::xsd::INTEGER);
let n: Option<Num> = as_numeric(&lit);        // Some(Num::Int(42))
```

### `join` — `#[cfg(feature = "join")]`

The four id-tuple join kernels over `&[Row]` slices. Requires `rows`; pulls `rustc-hash`
(for `FxHashMap`) only when enabled.

| Function | Kind |
|----------|------|
| `merge_join` | sorted merge join — one shared key column |
| `build_table` + `build_partitioned` | build `JoinTable` (serial) or `Vec<JoinTable>` (radix-partitioned for parallel workers) |
| `probe_emit` | per-row probe emit — single-hash (FxHash once for partition + `raw_entry().from_hash`), `reserve` before materialising (batch-emission contract) |
| `probe_gather_indices` | M4 batch-emission primitive — collect build-row indices WITHOUT materialising `Row`s; morsel pipeline calls this, then materialises per output chunk (sq-pntvh.7) |
| `hash_probe_serial` | probe loop: calls `probe_emit` per row with a `Budget` cooperative-cancel poll |
| `bind_combine` | index-nested-loop combine step |
| `lftj_recurse` over `Trie`/`TrieIter` | leapfrog trie-join (WCOJ) |
| `join::delta::DeltaTable` | persistent build-side table for semi-naive Δ-vs-full shapes (built for the OWL-RL fixpoint; drives `sparq-rsp`'s Delta/Snapshot window diff) |

`JoinTable` is a public type alias for `hashbrown::HashMap<Key, Posting, FxBuildHasher>`.
All kernels are generic over a `JoinKeys` column descriptor and a `Budget` cooperative-cancel
hook — both monomorphised, never a trait object. Use `NoBudget` for an unbounded join.

**Single-hash optimisation ([SONNET-4.6] sq-7d3dj.19):** `probe_emit` and `probe_gather_indices`
compute `key_hash` ONCE — it selects the radix partition AND drives `raw_entry().from_hash` on
the `JoinTable`, eliminating the previous double-hash in the partitioned probe path. `probe_emit`
calls `out.reserve(matches.len())` before the emit loop (batch-emission contract the sq-pntvh M4
morsel pipeline inherits: reserve then materialise, not per-match).

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["join"] }  # implies "rows"
```

```rust,ignore
use sparq_substrate::join::{merge_join, build_table, hash_probe_serial, JoinKeys, NoBudget};
use sparq_substrate::rows::Row;

fn make_row(vals: &[u32]) -> Row {
    let mut r = Row::new();
    r.extend_from_slice(vals);
    r
}

let left = vec![make_row(&[1, 10]), make_row(&[2, 20])]; // sorted on col 0
let right = vec![make_row(&[1, 100]), make_row(&[2, 200])];

// Sorted merge join on key column 0, append right column 1
let mut out = Vec::new();
merge_join(&left, 0, &right, 0, &[], &[1], &NoBudget, &mut out);
// out: [[1, 10, 100], [2, 20, 200]]

// Hash join: build left side, probe right side
let keys = JoinKeys { key_cols: vec![(0, 0)], right_only: vec![1] };
let table = build_table(&left, &keys);
let mut out2 = Vec::new();
hash_probe_serial(&right, &keys, &left, std::slice::from_ref(&table), &[1], &NoBudget, &mut out2);
```

### `compare` — `#[cfg(feature = "compare")]`

The SPARQL term total order `compare_terms`: the spec-fixed class precedence error/unbound <
blank < IRI < literal < triple, then KIND-FIRST within the literal class (sq-wjl8i): a fixed
`LiteralKind` rank between literal kinds — numeric < boolean < dateTime < date < string < lang
< other, a documented extension where the spec leaves cross-kind order undefined — and value
order only WITHIN a kind (numerics by exact rational value with NaN totalised FIRST before
-INF and f64 ties rechecked exactly via `exact_cmp`, incl. the mixed int/decimal-vs-double tie
— `Num::cmp_total`; dateTimes by timeline; else lexical), plus the recursive component-wise
triple-term order. Generic over a `CompareTerm` trait
(`term_class`/`literal_kind`/`value_str`/`as_f64`/`exact_cmp`/`strict_cmp`/`triple_parts`) — a
monomorphisation seam, never a `dyn` object. Pure-`std`; pulls nothing new. The engine's
relational `<`/`=` are deliberately untouched (XPath promotion, NaN type errors): only the
ORDER BY total order refines promoted ties and positions NaN.

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["compare"] }
```

**Machine-checked order laws (Kani, sq-sqtk2.4 + sq-wjl8i).** `src/compare.rs` hosts a
`#[cfg(kani)]` bounded-proof module proving, over an adversarial model `CompareTerm` impl
whose numeric domain straddles the 2^53 f64-collapse boundary (2^53−1 / 2^53 / 2^53+1 /
2^53+2 and their shared f64 images, ±0.0, NaN): reflexivity, antisymmetry-consistency,
within-class totality and transitivity — per literal kind AND across MIXED literal kinds
(`transitivity_mixed_literal_kinds_incl_nan`), ALL with NaN included — and exact-order
agreement on the exact tier (the `exact_cmp` recheck's guarantee — delete the recheck and it
goes red). The three sq-sqtk2.4 intransitivity findings (mixed-tier collapse, numeric-vs-
string lexical fallback, NaN partiality) are FIXED and pinned by `witness_*` harnesses that
go red on regression. Run `cargo kani -p sparq-substrate --features compare` (Kani injects
the `kani` cfg; the normal build strips the module). Honest boundaries: this proves the
shared ALGORITHM over the bounded model, NOT the engine's `Value` impl
(`sparq-engine/src/exec.rs` — covered by unit tests, the sparq-reason engine-parity suite and
W3C conformance; next-wave in `research/mechanized-proof-program.md` §6); within the dateTime
kind the indeterminate mixed-timezone window still falls back lexically (residual seam,
beaded); and `exact_cmp` impls bounded by the i128 tower keep collapsed ties for lexicals
beyond ~38 significant digits (beaded).

## Cargo feature summary

| Feature | Enables | Extra deps |
|---------|---------|------------|
| `rows` | `sparq_substrate::rows` | — |
| `numeric` | `sparq_substrate::numeric` | `oxrdf` |
| `join` | `sparq_substrate::join` (implies `rows`) | `rustc-hash`, `hashbrown` |
| `compare` | `sparq_substrate::compare` | — |

All features are off by default. The default build compiles nothing from this crate (byte-
identical wasm bundle). The crate is `forbid(unsafe_code)`.

## When to use

- **Implementing a new sparq reasoner** that shares join kernels or numeric evaluation with
  the engine — depend on `sparq-substrate` directly (it is below `sparq-engine`, so no cycle).
  Direct consumers today: `sparq-engine` (all four seams), `sparq-reason` (`substrate-join`,
  opt-in), and `sparq-rsp` (`join::delta::DeltaTable` for the windowed-materialisation
  Delta/Snapshot diff). [FABLE-5] sq-2n1q3.4
- **Working on the substrate crate itself** — see `research/shared-eval-substrate.md` for the
  full design record (what is shareable vs engine-private, the options considered, the
  perf-neutrality proof).

## See also

- `research/shared-eval-substrate.md` — design record for the extraction strategy.
- `crates/sparq-core` — the storage substrate (`Id`, `Dict`, `Graph`) this crate's `rows`
  vocabulary comes from.
- `crates/sparq-engine` — the primary consumer: keeps its planner, `Bindings`, and `Value`
  private; calls the shared kernels through thin adapters.

_Status: publishable (sq-qonbz.4 [SONNET-4.6]). All four modules implemented and behaviour-
neutral vs the pre-move engine baseline (W3C SPARQL conformance floor bit-identical; join/
numeric/compare micro-benches within noise). Phase-5 reasoner adoption (consuming `join` from
`sparq-reason` / `sparq-reason-el`) is tracked separately._
