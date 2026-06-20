# sparq-replan-policy

The shared **adaptive-re-plan POLICY core** for sparq's two adaptive query re-planners.

A re-planner watches a running query, and when the planner's *estimates* turn out to be
badly wrong it re-orders the part of the query that has not yet executed. Two knobs govern
the decision, and they are the **only** thing this crate holds, so the two re-planners
**cannot drift apart** on the rule:

- `divergence_factor` `k` — a re-plan is *considered* only when an observed cardinality `o`
  diverges from its estimate `e` by more than `k` either way (`o > k·e` or `e > k·o`).
- `improvement_margin` `m` — the re-planned alternative is *adopted* only when it beats the
  current plan's estimated cost by more than `m` (anti-thrash hysteresis).

Divergence *triggers*; the margin *gates adoption*.

## Consumers (both OFF by default)

- **Federated** — [`sparq-fedplan`](../sparq-fedplan)'s `adaptive-replan` feature
  (`AdaptiveExecutor`), planning over served source descriptors. It keeps its own richer
  `ReplanPolicy` (with extra latency knobs) and delegates the cardinality trigger +
  hysteresis here.
- **Local** — [`sparq-engine`](../sparq-engine)'s `adaptive-replan-local` feature, whose
  greedy BGP join loop materialises every intermediate join, so the *true* running
  cardinality is known for free at every stage boundary.

The two executors operate on completely different intermediate representations, so only this
tiny policy core is shared. Everything here is plain `f64` arithmetic — no RDF terms, no
graph, no I/O, **no dependencies**, no `unsafe` (`forbid(unsafe_code)`). The functions are
`#[inline]`, so a consumer pays nothing over an in-crate copy.

Nothing in the workspace's **default** build depends on this crate — it is pulled in only by
the two opt-in features above.

## License

[MIT](../../LICENSE).
