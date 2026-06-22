<!-- [OPUS-4.8] sq-inzv: internal-stub README for a publish=false crate; full surface lives in skills/federated-planning/SKILL.md. -->
# sparq-fedplan

Cost-based **federated source selection** + **bind-vs-hash-vs-streaming join
planning** over already-fetched source descriptors — a small, opt-in, **pure and
deterministic** planner with no network I/O. From a SPARQL BGP and a set of
`SourceDescriptor`s (VoID property/class partitions + mined `scs:` characteristic
sets) it prunes sources per pattern (HiBISCuS-style **recall-safe**), estimates
cardinality (CostFed skew-aware + characteristic-set star joins), and builds a join
order; an opt-in `adaptive-replan` feature adds stage-boundary re-planning, and the
`StreamJoin` operator gives a memory-bounded non-blocking join with spill. The full
public-API surface, the recall-safety invariant, the multiset-equal correctness
proof, and the (explicitly hand-tuned, non-optimal) latency/EWMA heuristic caveats
live in [`skills/federated-planning/SKILL.md`](../../skills/federated-planning/SKILL.md).

> **Internal crate — not published** to crates.io (`publish = false`). Opt-in
> (`fedplan` / `adaptive-replan` features, OFF by default; the lean core is
> byte-identical without it). Any timing observed here is **non-canonical**
> (work-box, not a CI runner) and is therefore not recorded.

How-to: [`skills/federated-planning/SKILL.md`](../../skills/federated-planning/SKILL.md).
Design: [`research/feature-research-federation.md`](../../research/feature-research-federation.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
