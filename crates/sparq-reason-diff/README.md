<!-- internal-stub -->
# sparq-reason-diff

Internal (`publish = false`) differential-test harness: diffs `sparq-reason`'s
OWL 2 RL closure (via its public `materialize(Profile::OwlRl, ..)`) against
pre-captured golden vectors from the Python `owlrl` reference implementation,
over the hand-picked corpus in `bench/reason-diff/rl/`.

Goldens are captured **offline** by `bench/reason-diff/rl/capture.py` (tool
versions pinned in each `.expected` header) — owlrl never runs in CI. Both
sides are normalized to a sorted canonical N-Triples multiset; any mismatch
fails loud unless the case carries an explicit `PERMANENT` disposition whose
pinned symmetric difference matches exactly (`*.disposition` ledger, per the
sq-pbz04.1 audit precedent).

Everything is behind the `rl-oracle` feature (default build is an empty lib):

```
cargo test -p sparq-reason-diff --features rl-oracle \
    rl_differential_vs_owlrl_golden -- --nocapture
```

## License

MIT OR Apache-2.0 (workspace).
