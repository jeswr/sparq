<!-- [SONNET-4.6] sq-3x7dl.14.2: internal-stub README for a publish=false standalone harness. -->

# sparq-xpath-differential

PROOF **M1** (`sq-3x7dl.14.2`): differential harness for the `noir_XPath` circuit
primitives, using **sparq's own trusted Rust SPARQL/XSD scalar evaluator** as the oracle.

The `noir_XPath` source is **not in this repo** — it was externalized to the
[`sparq-org/noir_XPath`](https://github.com/sparq-org/noir_XPath) face repo
(`sq-5reoy` / #1599). This harness is the **oracle half**: it lives where the trusted Rust
evaluator lives, generates a Noir test file pinning that evaluator's answers, and
`scripts/run_differential_harness.sh` compiles that file against the pinned released face
repo.

## What it checks

For every sampled input, the Noir function output **must equal** the Rust XSD-evaluator
output — over a unicode-aware corpus that deliberately re-covers the edges the W3C qt3
corpus lacks:

| Primitive | Edge coverage |
| --- | --- |
| `fn:string-length` (STRLEN) | multibyte (2/3/4-byte), combining marks, astral, NUL-padded buffers |
| `fn:starts-with` / `ends-with` / `contains` | multibyte needles, empty needle, NUL-padded needle *and* haystack |
| `fn:substring` (SUBSTR) | `start < 1` window, zero length, length past the end, start past the end |
| `op:numeric-divide` | **non-exact** quotients (`1 div 3`, `100 div 7`) and sign combinations, plus the `idiv` de-aliasing control |
| mixed `xs:integer` ↔ `xs:double` compare | every integer **outside `[-128, 127]`**, including `2^53 + 1` |
| `xs:integer(xs:double)` | truncation toward zero, negatives, `2^53` |
| `fn:round` | ties toward `+∞`, negatives |
| `xs:dateTime` | **pre-1970** (negative epoch) component extraction + epoch-straddling ordering |

Those are exactly the cases the correctness beads `sq-3x7dl.4`/`.5`/`.6`/`.7` fixed, so the
harness doubles as their **regression oracle**.

## Two cross-checks, and what happens when they fire

No expected value is hand-written; each is read back from a real `BIND(<expr> AS ?out)`.
Each answer is then cross-checked, and a mismatch **aborts generation**:

- **IEEE** — every `xs:double` answer is recomputed with native Rust `f64` and compared
  bit-for-bit, so a lossy serialization can never pin a wrong bit pattern.
- **F&O** — every `fn:substring` answer is recomputed against an explicit XPath F&O §5.4.3
  window reference.

Where sparq's evaluator is itself wrong against the spec that `noir_XPath` implements, the
case is **not** dropped or downgraded: it stays a **live assertion**, but against the **F&O
spec value** rather than the oracle's, and is labelled **SPEC-REFERENCE** both at the row and
in the generated file's header. Read such a row as `noir_XPath == XPath F&O`, not as
`noir_XPath == sparq`. Keeping it live is the point: these are edges `noir_XPath` has already
*fixed*, so a regression on one must fail the run — a commented-out assertion cannot fail
and would verify nothing. Two divergences are recorded today — `SUBSTR` with `start < 1`
(the engine shifts the window instead of keeping it) and `ROUND` losing the sign of a
negative zero.

Three unit tests hold that arrangement in place: one asserts no assertion is ever emitted
commented out and that `substring("12345", 0, 3)` and `round_double(-0.5)` in particular
reach the circuit live; two assert each divergence still reproduces, so the special-casing
**expires** (goes red) the day the engine is fixed.

## TCB — stated honestly

This is **VERIFICATION, not proof**. Three things are trusted and unproven:

1. **The sparq Rust XSD evaluator.** It is the repo's reference semantics, *not* an audited
   or proven-correct implementation. Two live divergences from XPath F&O are already
   recorded above; there may be more that the corpus does not reach.
2. **The SAMPLE.** Coverage is hand-picked edge cases, not exhaustive. A wrong answer on an
   unsampled input is not caught. (Exhaustive coverage is milestone **M2**, and only for
   unary binary32 ops.)
3. **The Noir → ACIR → Barretenberg lowering.** `nargo test` exercises witness generation
   only. Nothing below ACIR is covered by any tier of this program.

**Known scope limit.** `fn:substring` cases are ASCII-only by construction. `noir_XPath`'s
`substring` indexes **byte** positions in the logical content (its own documented caveat;
codepoint-positional substring is bead `sq-hjvte`) while SPARQL `SUBSTR` is codepoint-
indexed. They agree exactly on single-byte content and only there, so sampling multibyte
would pin a known beaded gap rather than detect a regression.

A green run makes **no soundness or privacy claim**. The ZK estate remains research-grade
and **NOT externally audited** (`sq-qhy4`).

## Running it

```sh
bash zk/xpath/scripts/run_differential_harness.sh                # cargo + nargo, full run
bash zk/xpath/scripts/run_differential_harness.sh --generate-only  # cargo only (drift guard)
bash zk/xpath/scripts/run_differential_harness.sh --update-committed  # refresh the golden
```

The committed golden is `zk/xpath/tests/differential_oracle/src/lib.nr`; CI diffs the
generator's output against it, so a corpus or oracle change lands as a reviewable diff.

Point the run at a different `noir_XPath` with `XPATH_GIT` / `XPATH_TAG` / `XPATH_DIRECTORY`,
or at a local checkout with `XPATH_PATH`. Both path forms name the library PACKAGE, not the
repo root: the face repo's root `Nargo.toml` is a `[workspace]`, so the git dep carries
`directory = "xpath"` (default) and `XPATH_PATH` wants `<checkout>/xpath`.

**License:** MIT
