# sparq-fedclient

A **streaming federation CLIENT** over heterogeneous remote RDF sources — the query
*consumer* half of federation, **opt-in** and OFF by default.

Given one SPARQL query and a set of heterogeneous remote sources (full SPARQL endpoints,
bindings-restricted brTPF servers, plain TPF servers, and the *local* sparq engine), this
crate — when complete — **discovers** each source's capability, **plans** a federated
execution that pushes the most precise sub-query each source can answer (reusing the
`sparq-fedplan` cost-based planner), and **streams** results back through non-blocking
federation operators. See `research/federation-client-design.md` for the full design (§4
architecture, §6 phased build plan, §7 honest risks).

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Bead **sq-s1uy** · epic **sq-dnko** / **sq-3183** (streaming federation client).

## What has landed, and what is still ahead

The crate began as the **Phase-0 skeleton** (design §6) — the public module layout, the
opt-in feature, and the dependency-boundary proof, before any federation logic. Landed
since (each behind the same default-OFF `fedclient` feature):

- **Phase 1 — capability discovery** (`sq-nfxl`): the `discovery` module GETs a source's
  SPARQL Service Description + `/.well-known/void`, parses them to a `Capability` +
  `SourceDescriptor`, with a FedX-style ASK-probe fallback, all behind an SSRF-guarded
  fetch seam.
- **Phase 2 — source-type abstraction + Endpoint adapter** (`sq-rsxf`): the `source` module
  — `SourceType` (`Endpoint | BrTpf | Tpf | Local`), the `FederatedSource` trait, the
  fine-grained `Capability` descriptor, and the real `Endpoint` SRJ adapter over a
  `Transport` seam behind a **default-deny SSRF egress guard**.
- **Phase 6 — brTPF + TPF fragment adapters** (`sq-2qze`): the real Triple-Pattern-Fragments
  adapters (see below).

Still ahead (future beads under epic sq-dnko): the planner bridge (§4.2), capability-aware
pushdown (§4.3), the streaming operators (§4.4 / §5), and adaptive re-planning (§7).

## brTPF + TPF fragment adapters (Phase 6, `source` module)

A single triple pattern is the access unit of a Triple Pattern Fragments server. Both
adapters wrap a `FragmentTransport` seam (fetch one fragment page for a pattern, optionally
with an attached binding block → matched triples + the page's `hydra:totalItems` count + an
optional `hydra:next` token) and return a **complete** answer for one pattern as typed
`FragBinding`s:

- **`TpfSource` (plain TPF)** — fetches the fragment to exhaustion (follows `hydra:next`),
  binds every matched triple into the pattern's variables, and returns the whole (selective)
  fragment. There is no bind-join: a plain-TPF source shifts every join client-side, so the
  planner hash-joins the materialised fragments locally, driven by the count metadata.
- **`BrTpfSource` (bindings-restricted TPF)** — additionally pushes a block of *at most
  `maxMpR`* upstream bindings with each request (the standardised brTPF bind-join). It
  chunks the upstream bindings into `maxMpR`-sized blocks, issues one paginated request per
  block, and concatenates the per-block matches — complete by construction, and the block
  size never exceeds `maxMpR`.
- **Count-metadata cardinality** — both expose `cardinality(pattern)` and a one-pattern
  `SourceDescriptor` from `discover()` seeded with the fragment's `hydra:totalItems`, so the
  `sparq-fedplan` CostFed estimate keys on the *served* count. For brTPF the descriptor uses
  the unbound-pattern count (a recall-safe upper bound the bound block only narrows).

A fragment server speaks triples, not SPARQL-Results-JSON, so the adapters answer through
the typed `solutions(...)` methods; their `FederatedSource::execute` (the SRJ entry point)
is a deliberate `Unsupported` that points the caller at `solutions` — no lossy SRJ
re-serialisation, no overclaim. The adapters are tested against an in-memory fixture
fragment server (real fetch → parse → bind → paginate → bind-join, zero network); the
native HTTP `FragmentTransport` (ureq + the default-deny SSRF resolver, Hydra URI-template
serialisation, Turtle/TriG fragment parsing) lands with the streaming phase.

## Public module layout (design §4)

| Module        | Design  | What it will hold (later phase)                                     |
|---------------|---------|----------------------------------------------------------------------|
| `source`      | §4.1    | `SourceType` (Endpoint \| BrTpf \| Tpf \| Local) + `FederatedSource` trait |
| `discovery`   | §4.1    | VoID/SD discovery → `Capability`; reuses `from_void_nt`; ASK fallback |
| `planner`     | §4.2    | lower BGP → `sparq-fedplan`, `select_sources` + `plan_bgp`, index→adapter |
| `pushdown`    | §4.3    | maximal pushable sub-algebra per exclusive group; VALUES bind-join   |
| `operators`   | §4.4    | `JoinTree` → Bind / Hash / Streaming / Local operators               |
| `stream`      | §4.4    | the `SolutionStream` boundary the client owns (engine stays materialised) |

## Opt-in (hard constraint)

The whole client is behind the **`fedclient` cargo feature, OFF by default**, and the
crate is a standalone workspace member with `publish = false`. A build that does not
enable `fedclient` compiles an empty crate (mirrors `sparq-fedplan`'s `fedplan` feature).

```toml
[dependencies]
sparq-fedclient = { path = "crates/sparq-fedclient", features = ["fedclient"] }
```

Enabling `fedclient` pulls in `sparq-fedplan` (`fedplan` planner + `StreamJoin`) and
`sparq-engine` (`service` SRJ transport + VALUES bind-join + SSRF egress guard + local
eval) — the two reuse seams §4 names.

## The dependency boundary (load-bearing, enforced)

`sparq-core` and `sparq-engine` **never** depend on `sparq-fedclient`. The dependency
arrow points one-way *into* the engine — the client reuses the engine, the engine never
reuses the client — so the default engine build and the WASM artifact are byte-identical
with or without this crate. That invariant is enforced two ways, both of which **fail if
a future edit introduces such an edge**, in both feature states:

- **`scripts/fedclient-boundary-guard.sh`** — a CI step (wired into `feature-matrix.yml`)
  that inverts the dependency graph with `cargo tree -i sparq-fedclient --all-features`
  and fails if `sparq-core` or `sparq-engine` appears as a dependent (any such edge forms
  a dependency cycle, which the guard detects and reports with the cycle path).
- **`tests/boundary.rs`** — a hermetic `cargo test` that reads `cargo metadata`'s resolve
  graph and asserts neither lean-core member transitively reaches `sparq-fedclient`, plus
  the positive check that the client *does* reach its reuse seams under `--all-features`.

Run the guard locally:

```sh
scripts/fedclient-boundary-guard.sh        # exit 0 = boundary intact
cargo test -p sparq-fedclient --features fedclient --test boundary
```

## Status / roadmap

Landed: Phase 0 (skeleton + boundary proof, `sq-s1uy`), Phase 1 (discovery, `sq-nfxl`),
Phase 2 (source abstraction + Endpoint adapter, `sq-rsxf`), Phase 6 (brTPF/TPF adapters,
`sq-2qze`). Still ahead under epic **sq-dnko**: the planner bridge (§4.2), capability-aware
pushdown (§4.3), the streaming operators (§4.4 / §5), and adaptive re-planning (§7). No
performance numbers appear here: any "better than Comunica" claim in the design record is an
*architectural prediction* to be validated head-to-head before being asserted as fact.

[OPUS-4.8] sq-s1uy — flagged for Fable re-review.
