# Policy-evaluation benchmark gap record — 2026-07

<!-- [GPT-5.6] sq-gpdwc. Do not put benchmark measurements in this record. -->

**Axis:** access-control policy evaluation (WAC-, ACP-, and ODRL-shaped decisions)  
**Epic:** sq-hmd7l  
**Bead:** sq-gpdwc

## Disposition

External throughput comparison is **NOT-COMPARABLE**. There is no pinned peer that
offers the same in-process decision surface across this WAC/ACP/ODRL mix. Selecting
an unrelated policy engine would produce an apples-to-oranges result rather than a
defensible competitor column.

The crate therefore has a self-relative micro-harness at
`crates/sparq-policy/examples/policy_bench.rs`. It evaluates a deterministic request
corpus against vendored policy fixtures. Before starting a timer or printing any
throughput row, the harness checks every result against a pinned allow/deny table.
Failure is fail-fast. Its machine-readable rows are:

```text
<policy-mix>\t<decisions>\t<us>
```

Run the acceptance tier with:

```sh
cargo run -p sparq-policy --release --example policy_bench -- --smoke
```

The WAC and ACP lanes reproduce their characteristic public/owner and group/deny
decision shapes using ODRL rules, so all lanes measure the same `sparq-policy`
evaluation path. This harness measures evaluation mechanics only. It makes no claim
about the soundness or security of an access-control deployment.

## Gap status

There is no peer-performance gap to report under the honest NOT-COMPARABLE
disposition. Future measurements may be stored by the benchmark infrastructure, but
timings are deliberately not committed to this Markdown record.
