# sparq-substrate

**Shared zero-overhead evaluation substrate** for the sparq SPARQL engine and the
reasoners — an **opt-in**, **leaf** crate (epic sq-qonbz) that depends **only** on
`sparq-core`, never on `sparq-engine`. It hosts the parts of evaluation that are genuinely
common to both the query engine and every reasoner: the id-tuple **row/key** vocabulary and
the XSD **numeric value tower**. Placing them in a leaf crate lets both consumers reach them
with **no dependency cycle**, while keeping `sparq-core` and the lean wasm bundle untouched.

> **Scaffold status (sq-fmprw — Phase 1 of the epic).** This crate currently defines /
> re-exports the shared **types** only; **no evaluation logic has moved yet**. The join
> kernels (merge / hash / bind / leapfrog-trie) and the engine's `compare_values` total
> order + `Num` arithmetic land in later beads. Nothing in the workspace depends on this
> crate yet, so the default engine build does not even compile it. See
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
  the XPath promotion ranks) and `as_numeric`, which classifies an `oxrdf::Literal` into the
  tower while keeping the **exact** integer / fixed-point `Dec` representation (a
  high-precision decimal is not silently flattened to `f64`).

Both features are **off by default**. The crate is `forbid(unsafe_code)`.

### Zero-overhead intent

The shared kernels (arriving in later beads) are designed as **free functions monomorphic
over `Id = u32`** and the `SmallVec` row aliases — **never** `Box<dyn>` / `&dyn` / a vtable
between a join's probe and its key comparison. The compiler then emits one specialised,
inlinable body per call site, so the engine's hot loops keep identical codegen after the
move. This scaffold introduces no dynamic dispatch.

## 📚 Learn more

- `research/shared-eval-substrate.md` — the design record: what is shareable vs
  engine-private, the options considered, and the layered perf-neutrality proof.
- `crates/sparq-core` — the storage substrate this crate's `Id` / dictionary types come from.
- `crates/sparq-engine` — the consumer that keeps its planner and will call the shared
  kernels through a thin adapter once they move.

## License

MIT — see the workspace [LICENSE](../../LICENSE).
