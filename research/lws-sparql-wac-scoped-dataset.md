# WAC-scoped dataset assembly for the lws-core `/sparql` endpoint (sq-r1ei8)

**Status:** design record. The soundness-critical half of `sq-r1ei8` (the `sparql-endpoint`
feature). The feature-tier *mechanism* (full vs core, the cargo flag, the build matrix) is settled
in [`wasm-solid-server-npm-design.md`](wasm-solid-server-npm-design.md) §5.4; **this record designs
the one authz-soundness-sensitive part flagged there: how a query evaluates over *only* the
resources the authenticated agent may read, without leaking any it may not.** The endpoint is
query-only v1 (SELECT/ASK/CONSTRUCT); SPARQL UPDATE is a later follow-up. `[OPUS-4.8]`

> **Honest scope.** Clear-path server-side authorisation over the pod's own resources — no
> cryptographic / unlinkability guarantee. The invariant below is an *access* invariant (no
> unreadable triple reaches a result), enforced by construction, not a privacy claim.

## The invariant (no-leak, fail-closed)

Let `A` = the authenticated agent (or the public principal). Let `read(r)` mean
`WacAuthorizer::authorize_read(r, AccessMode::Read, A) == ReadDecision::Allow` (the SAME per-resource
gate the LDP GET path uses — one code path, no second authz model to keep in sync).

Define the **authorized dataset**

```text
D_auth(A) = ⋃ { parse_rdf(bytes(r)) : r ∈ RDF resources of the pod ∧ read(r) }
```

**Invariant.** Every `/sparql` result (SELECT bindings, ASK boolean, CONSTRUCT graph) is a function
of `D_auth(A)` **only**. No triple from a resource `r` with `¬read(r)` appears in, or influences, any
result — including via `OPTIONAL` / `MINUS` / `NOT EXISTS` / aggregates / `COUNT` / negation.

## Why assembly, not query-rewriting or post-filtering

The endpoint **assembles `D_auth(A)` as the actual dataset the engine evaluates**, exactly the
sound-by-construction move used for pattern-scoped masking (see
[`odrl-pattern-scoped-targets-2026-07.md`](odrl-pattern-scoped-targets-2026-07.md) and the
usage-control-policy SKILL): because the evaluated dataset *is* `D_auth`, the invariant is an
**identity, not a per-operator audit obligation**. Negation-as-failure is the trap that kills the
two rejected alternatives:

- **Query rewriting** (inject `FROM`/`GRAPH` filters): a `FILTER NOT EXISTS { <secret> ?p ?o }`
  still observes the *absence* of a triple in a graph the agent can't read — a one-bit leak per
  probe. Getting rewriting sound against all of SPARQL negation is the audit obligation we refuse.
- **Post-filtering results**: cannot unwind an aggregate or an `EXISTS` that already consumed an
  unreadable triple.

Assembly has none of these: an unreadable resource is *physically absent from the dataset*, so no
operator can observe it. Any future engine fast path inherits the invariant for free.

## Algorithm (v1 — correctness first)

1. **Enumerate** the pod's resources by walking `ldp:contains` from the storage root (the existing
   `store::sparql::select_children` containment query drives the traversal). Enumeration is
   **server-authoritative**: it does not depend on the agent being able to *list* a container —
   discovery is the server's, admission is per-resource (step 2). (A resource the agent may read but
   whose parent container they cannot list is still included — per-resource ACLs are the WAC truth;
   excluding it would under-grant, not leak.)
2. **Admit** each RDF resource `r` iff `read(r)` (per-resource `authorize_read`, reusing the
   `AclCache` so the walk pays parse-once per effective ACL). **Fail-closed:** any non-`Allow`
   outcome — `Deny`, an unreadable/unparseable `.acl`, an error, or an ambiguous resolution —
   **excludes** `r`. Never admit on doubt. Non-RDF (binary) resources contribute no triples and are
   skipped.
3. **Assemble** one **named graph per admitted resource**, graph name = the resource's canonical
   IRI, quads = `parse_rdf(bytes(r))`. Blank nodes are scoped per source graph (the engine's normal
   per-graph bnode labelling) so identity cannot be correlated across resources.
4. **Default graph = empty** by default (the spec-compliant [solid-sparql-query] default-graph
   contract already adopted by `sparq-solid`/`sparq-server`, issue #1546): a bare `{ ?s ?p ?o }`
   matches nothing until the query opts into the authorized union via
   `FROM <…#union-default-graph>`. This is itself fail-safe — the union is never the silent default.
5. **Evaluate** SELECT/ASK/CONSTRUCT over the assembled dataset via the embedded `sparq-engine`. A
   CONSTRUCT graph is a function of `D_auth` only, so its output cannot re-expose an unreadable
   triple.

## Consistency + snapshot

Assembly MUST read one **consistent snapshot** — reuse the server's existing generation-pinned read
snapshot (the same `state.current().snapshot()` the facets/complete endpoints pin) so a concurrent
write cannot interleave a half-admitted resource, and so `read(r)` and `bytes(r)` are evaluated
against the same generation. The assembled `D_auth` is the cache unit: build once per
`(agent, generation)`, query many times; rebuild when the pinned generation advances or the agent's
effective ACLs change (v1 = rebuild-on-staleness, mirroring `/complete`).

## Test surface (the acceptance obligation)

A differential/adversarial suite in `crates/sparq-lws-core` (feature `sparql-endpoint`):

1. **No-leak, direct:** agent reads `a` but not `b`; `SELECT * { GRAPH ?g { ?s ?p ?o } }` returns
   `a`'s triples and **zero** rows mentioning `b`; a query naming `<b>`'s graph is empty.
2. **No-leak via negation:** `ASK { FILTER NOT EXISTS { <b-subject> ?p ?o } }` returns the SAME
   answer whether or not `b` exists on disk — the unreadable resource is invisible to negation.
3. **Fail-closed:** a resource with an unparseable `.acl` (or a resolution error) is **excluded**,
   not included-on-error.
4. **Equivalence to LDP:** the set of resources whose triples appear ≡ the set the agent gets `200`
   on via LDP GET (one authz truth).
5. **core-tier absence:** without `sparql-endpoint`, `/sparql` is `404` and the handler/route +
   `sparq-engine` query surface are compiled out; LDP CRUD + WAC round-trip unchanged; feature-off
   build byte-stable.

## Open questions for the maintainer / implementer

- **Enumeration cost.** v1 walks the whole pod per `(agent, generation)`. Large pods want an
  index (readable-resource set materialised + reconciled on ACL/write change) — a perf follow-up,
  not a v1 blocker. Flag the O(pod) cost in the endpoint doc; do not hide it.
- **`acl:Read` on `.acl` resources.** Effective-ACL resources are admitted only under the normal
  `acl:Control`-gated rule (an agent without Control does not see raw `.acl` triples in the query
  dataset). Confirm this falls out of reusing `authorize_read` as-is.
- **CONSTRUCT over the union vs named graphs.** v1 keeps the empty-default-graph contract; confirm
  the `FROM <union>` opt-in is the desired ergonomics for the query-only surface, or whether the
  endpoint should default the union for `/sparql` specifically (a deliberate divergence to weigh).

## Follow-ups this unblocks

- `sq-r1ei8` implementation: the net-new `sparql-endpoint` feature + `GET/POST /sparql` handler
  assembling `D_auth` per this record, with **mandatory Opus review of the no-leak suite**.
- SPARQL UPDATE tier (write-scoped assembly — a strictly harder authz problem; separate design).
