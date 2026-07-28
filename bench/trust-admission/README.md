# Trust-admission deterministic metrics

This directory holds regenerable evidence from the opt-in `sparq-trust` certification
graph and unchanged admission gate. Generate the sample from the repository root:

```sh
cargo run -p sparq-trust --features cert-graph --example admission_cost -- \
  --emit bench/trust-admission/sample.json
```

The document is a JSON object with `schema_version` (currently `1`) and `cases`, an
ordered array. Every case contains these integer fields:

- `fixture_size`: synthetic rules and certification edges constructed for the case.
- `depth_bound`: closure bound supplied to `derive_effective_rules`.
- `direct_rule_count`: anchor rules supplied to closure and admission.
- `certification_edges_considered`: edges inspected by the evidence harness.
- `edges_rejected`: counts keyed by every public `EdgeRejection` reason:
  `no_anchor`, `signature_invalid`, `out_of_window`, `cyclic`, `broadening`, and
  `over_depth`.
- `max_closure_depth_reached`: actual closure rounds reached. The library closure is
  depth-N (`sq-13096`), but this harness fixes `depth_bound` at zero or one and its
  fixtures contain no multi-hop chains, so this is either zero or one.
- `visited_set_size`: unique certifier and certified-issuer identities observed. This is
  the deterministic visited-set input size for evaluating cycle handling. It is an input
  measure of the fixture, not a measurement of the library's own allocation.
- `derived_rule_count`: effective rules minus direct rules.

All values are deterministic functions of fixed fixtures. The schema excludes clock,
elapsed-time, host, platform, and allocator measurements. Such measurements are not
canonical evidence for this repository.

[GPT-5.6] `sq-r78pf`; evidence substrate for `sq-pfae.9` only. It does not close that
cost/decidability spike or make a complexity/soundness claim.
