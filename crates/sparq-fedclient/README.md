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

## Phase 0 — what this is, and what it is NOT

This crate is currently the **Phase-0 skeleton** (design §6, Phase 0): a compiling
crate that establishes the public module layout, the opt-in feature, and the
dependency-boundary proof — **before any federation logic**. There is **no discovery, no
source adapters, no planner bridge, no pushdown, and no streaming operators yet**; those
land in Phases 1-7 (each a future bead under epic sq-dnko). The modules exist as
`todo!()`-free placeholders so the public surface is visible and the boundary is provable
now.

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

Phase 0 (this crate): skeleton + boundary proof. Phases 1-7 (discovery, source
abstraction + Endpoint adapter, planner bridge, capability-aware pushdown, streaming
operators, brTPF/TPF adapters, adaptive re-planning) are tracked as future beads under
epic **sq-dnko**. No performance numbers appear here: any "better than Comunica" claim in
the design record is an *architectural prediction* to be validated head-to-head before
being asserted as fact.

[OPUS-4.8] sq-s1uy — flagged for Fable re-review.
