# sparq-substrate

**Shared zero-overhead evaluation substrate** for the sparq SPARQL engine and the
reasoners — an **opt-in**, **leaf** crate (epic sq-qonbz) that depends **only** on
`sparq-core`, never on `sparq-engine`. It hosts the parts of evaluation that are genuinely
common to both the query engine and every reasoner: the id-tuple **row/key** vocabulary and
the XSD **numeric value tower**. Placing them in a leaf crate lets both consumers reach them
with **no dependency cycle**, while keeping `sparq-core` and the lean wasm bundle untouched.

> **Status (sq-ev41x — Phase 2 of the epic).** The XSD **numeric value tower** (`Num` /
> `Dec` / `as_numeric` + the arithmetic ops) has now **moved here** from `sparq-engine` and
> the engine consumes it — a behaviour-neutral code-move (the W3C SPARQL conformance floor +
> ORDER BY / numeric / relop tests are bit-identical). Still pending in later beads: the join
> kernels (merge / hash / bind / leapfrog-trie) and the engine's `compare_values` total order
> (which moves with the engine's `Value` enum, not here). See
> `research/shared-eval-substrate.md` for the full extraction plan and the perf-neutrality
> proof strategy.

## 🚀 Quickstart

Everything is behind **default-off** features — opt into exactly the slice you need:

```toml
[dependencies]
sparq-substrate = { version = "0.1.0", features = ["rows", "numeric"] }
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
  `ceil` / `floor` / `round`), the XSD-canonical `lexical` / `canonical_lexical`, and the
  shared lexical helpers (`split_decimal`, `parse_xsd_f32` / `parse_xsd_f64`, `fmt_xsd_double`).

Both features are **off by default**. The crate is `forbid(unsafe_code)`.

### Zero-overhead intent

Every item is monomorphic over `Id = u32` and the concrete numeric tiers — **never**
`Box<dyn>` / `&dyn` / a vtable on a hot path. Each `numeric` item carries `#[inline]`, so
cross-crate inlining (with the workspace LTO profile) keeps the engine's FILTER / BIND /
ORDER BY hot loops identical to pre-move codegen. The join kernels arriving in later beads
keep the same contract. This crate introduces no dynamic dispatch.

## 📚 Learn more

- `research/shared-eval-substrate.md` — the design record: what is shareable vs
  engine-private, the options considered, and the layered perf-neutrality proof.
- `crates/sparq-core` — the storage substrate this crate's `Id` / dictionary types come from.
- `crates/sparq-engine` — the consumer that keeps its planner and will call the shared
  kernels through a thin adapter once they move.

## License

MIT — see the workspace [LICENSE](../../LICENSE).
