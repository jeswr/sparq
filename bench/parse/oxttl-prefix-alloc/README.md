# bench/parse/oxttl-prefix-alloc — oxttl prefixed-name expansion A/B

Measurement harness for **sq-98w7z.3**. The bead was filed to upstream a low-copy
prefixed-name expansion to `oxigraph/oxigraph`, because `oxttl` 0.2.3 builds a
fresh `String` per prefixed name (`format!("{start}{local}")` in
`resolve_local_name`). This harness exists to measure that, and it found the fix
**already landed upstream** — see `research/oxttl-prefixed-name-alloc-2026-07.md`
for the verdict, the attribution table, and what sparq should do about it.

Nothing here is production code and **no sparq crate is involved**: the harness
depends on `oxttl` + `oxrdf` only, and no `oxttl` source is vendored into this
repository.

## Layout

One source file, compiled twice against two different `oxttl` revisions:

| path | what it links |
|---|---|
| `src/main.rs` | the harness — corpus generator, parser driver, digest, timing, allocation counting |
| `released/Cargo.toml` | `oxttl =0.2.3` from crates.io — the version sparq's workspace pins |
| `upstream/Cargo.toml` | `oxttl` from `oxigraph.git` at a **pinned rev** |
| `ab.sh` | builds and runs both legs, plus `--attribute <sha>` for a per-commit probe |

Both wrapper packages point their `[[bin]] path` at the same `src/main.rs`, so the
measured code is byte-identical and the only variable is which `oxttl` the linker
resolved. Each declares its own `[workspace]` (the `bench/parse` isolation
pattern), so neither reaches the root workspace, the root `Cargo.lock`, or the
wasm build.

## Running

`ab.sh` builds each leg in all five documented configurations — default, `mimalloc`,
`rdf-12`, `count-alloc`, `rdf-12,count-alloc`, each in its own target directory, ten
builds in all — so one command reproduces every row recorded in
`research/oxttl-prefixed-name-alloc-2026-07.md`.

```sh
# Both legs × all five configurations, generating the corpus on first run.
./ab.sh

# Against a real corpus — e.g. the sq-wrn61 slice, if you have it.
./ab.sh --corpus ../data/wikidata-slice.ttl

# Which upstream commit bought the delta?
./ab.sh --attribute f5383d8 --attribute 924d0b1
```

`HARNESS_TOOLCHAIN=<toolchain>` prepends `cargo +<toolchain>`, for a box that
cannot materialise the repo's pinned toolchain (which this harness does not need).

## Corpus

The sq-wrn61 corpus (`bench/parse/data/wikidata-slice.ttl`) is gitignored and not
redistributable, so `gen` produces a deterministic stand-in with the same salient
shape: the `wd:`/`wdt:` prefix set and an object column that is overwhelmingly
prefixed names rather than IRIREFs — which is what makes a corpus exercise
`resolve_local_name`, the function under test. Pass `--corpus` to use the real
slice and label the row accordingly.

## What the numbers mean

- **Timings are NON-CANONICAL.** They are whatever box you ran on, under whatever
  toolchain; the standing rule is that only the canonical bench instance produces
  quotable throughput. Every emitted `--json` document carries a `note` saying so.
- **Allocation counts come from no clock.** They are read off a counting shim over
  the system allocator (`--features count-alloc`), so they do not move with box
  speed or load: repeated runs of the *same binary* over the same corpus return
  the same counts, which is what makes them the honest metric for an
  allocation-reduction claim. That is **not** box-independence — the shim counts
  every allocation the whole binary makes, so a different toolchain, target,
  dependency resolution (the lockfiles here are deliberately uncommitted) or
  feature set can move them. Quote a count together with the configuration that
  produced it, and re-derive rather than copy it when any of those change. The
  harness prints them separately from the timings because the shim's atomics
  perturb the clock.
- **`--features mimalloc`** (a leg `ab.sh` always runs) re-times under the allocator
  `sparq-cli ingest` ships with, so an allocation win measured under system malloc
  is not quietly over-claimed against the allocator production actually uses.
- **`--features rdf-12`** (also always run, both timed and under `count-alloc`) uses
  the feature set the sparq workspace actually enables on `oxttl` (root
  `Cargo.toml`: `oxttl = { version = "0.2", features = ["rdf-12"] }`). The default
  build leaves it off, and `ab.sh` prints both — quote the `rdf-12` row when the
  question is about sparq.

## The invariant

Every run prints an order-sensitive FNV-1a **digest** over the raw component
strings of every parsed triple, alongside the triple count. Equal digest + equal
count across two legs is the harness's evidence for the bead's "parse is
byte/count-identical" invariant. The digest is deliberately computed from
`as_str()`/`value()`/`datatype()` and **not** from any `Display` impl, so a
serializer change between the two `oxttl` revisions cannot masquerade as a parse
divergence.
